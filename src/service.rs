#![allow(dead_code)]
extern crate epoll;
extern crate unix_socket;

use std::str;
use std::str::FromStr;

use std::fs::remove_file;
use std::io::prelude::*;
use std::os::unix::io::{AsRawFd, RawFd};
use std::net::{UdpSocket,SocketAddr};
use std::net::{TcpListener,TcpStream};

use std::thread;

use spring_dvs::enums::{Response};
use spring_dvs::protocol::{ProtocolObject,Message};
use spring_dvs::protocol::{Port};

use spring_dvs::http::HttpWrapper;
use self::unix_socket::UnixListener;


use netspace::*;
use management::management_handler;

use self::epoll::*;
use self::epoll::util::*;

use unit_test_env::*;


/* ToDo:
 * -The UDP is running on a single thread, thus making the
 *  use of epoll redundent. Spawn threads or create a thread
 *  pool for handling the UDP connections
 *
 * -The response from an outbound HTTP service layer request 
 *  can come in Transfer-Encoding chunked. This needs to be 
 *  handled so the requests can use HTTP/1.1 again. 
 */



use protocol::{Protocol,Svr,response};
use chain::ChainService;



fn content_len(bytes: &[u8]) -> Option<(usize,usize)> {

	if bytes.len() < 4 || &bytes[0..3] != b"200" {
		return None
	}

	
	let bytestr = match str::from_utf8(&bytes[4..]) {
		Ok(s) => s,
		Err(_) => return None
	};
	
	
	
	let s = match String::from_str(bytestr) {
		Ok(s) => s,
		Err(_) => return None
	};

	let index = s.find(" ").unwrap();
	let (sl,_) = s.split_at(index);
	
	Some(match sl.parse() {
			Ok(n) => (n,(4+index+1)),
			Err(_) => return None
	})
}

/*

*/

pub struct Tcp;
pub struct Dvsp;
pub struct Management;

impl Dvsp {
	pub fn start(config: &Config) -> Result<Success,Failure> {
		
		let sa = SocketAddr::from_str(&format!("0.0.0.0:{}",Port::Dvsp)).unwrap();
		let socket = match UdpSocket::bind(sa) {
				Ok(s) => s,
				Err(_) => return Err(Failure::InvalidArgument)
		};
		
		let epfd = epoll::create1(0).unwrap();
		let sfd = socket.as_raw_fd();
	
		let mut event = EpollEvent {
			data: sfd as u64,
			events: (event_type::EPOLLIN | event_type::EPOLLRDHUP)
		};
		
		match epoll::ctl(epfd, ctl_op::ADD, sfd, &mut event) {
			Ok(()) => { },
			Err(e) => println!("[Error] CtlError on add: {}", e)
		};
		
		let cfg_clone = config.clone();
		
		thread::spawn(move|| {
			
			Dvsp::epoll_wait(epfd, socket, cfg_clone);	    
		});
	
/*		match s.join() {
			Ok(_) => { },
			_ => println!("[Error] Error on UDP thread join"),
		}	
*/
		Ok(Success::Ok)
	
	}
	
	
	
	
	fn epoll_wait(epfd: RawFd, socket: UdpSocket, config: Config) {
	
		let mut bytes = [0;4096];
	
		let mut events = Vec::<EpollEvent>::with_capacity(100);
	  
	    unsafe { events.set_len(100); }
	    
	    let nio = match config.live_test {
			false => {
				println!("[Alert] Live System");
				NetspaceIo::new("/var/lib/springdvs/gsn.db") 
			},
			true => {
				println!("[Alert] Warning: Live testing enabled; using testing database");
				let nio = NetspaceIo::new("live-testing.db");
				
				setup_live_test_env(&nio, &config);
				nio
			
			}
		};
	    
	    netspace_add_self(&nio, &config);

	    println!("[System] UDP Service Online");
	    loop {
		    match epoll::wait(epfd, &mut events[..], -1) {
		
		        Ok(num_events) => {
		            
		            
		            for _ in 0..num_events {

		       			let (sz, from) = match socket.recv_from(&mut bytes) {
							Err(_) => return,
							Ok(s) => s
						};

						let svr = Svr::new(from, Box::new(config.clone()), &nio);
						let outbound : Message = match Message::from_bytes(&bytes[0..sz]) {
							Ok(m) => Protocol::process(&m, svr, Box::new(ChainService{})),
							Err(e) => {

								let mut v : Vec<u8> = Vec::new();

								v.extend_from_slice(&bytes[0..sz]);
								println!("[Error] Parse Error: {:?}\nDump:\n{:?}", e, v);
								response(Response::MalformedContent)
							}

						}; 	

		            	match socket.send_to(outbound.to_bytes().as_slice(), from) {
		            		Err(_) => return,
							_ => { }
		            	};

		            }
		        }

		        Err(e) => println!("[Error] Error on epoll::wait(): {}", e)
			}
	    }
	}

}


impl Tcp {

	pub fn start(cfg: &Config) -> Result<Success,Failure> {
		
		let listener = TcpListener::bind("0.0.0.0:55300").unwrap();

		let config = cfg.clone();
		

		let s = thread::spawn(move|| {
				
			let nio = match config.live_test {
				false => {
					NetspaceIo::new("/var/lib/springdvs/gsn.db") 
				},
				true => {
					NetspaceIo::new("live-testing.db")
				}
			};
		    
			println!("[System] TCP Service Online");
			for stream in listener.incoming() {
				
				match stream {
					Ok(mut stream) => {
	
						let mut buf = [0;4096];
						
						let mut address = match stream.peer_addr() {
							Ok(a) => a,
							Err(_) => continue
						};
						
						let size = match stream.read(&mut buf) {
							Ok(s) => s,
							Err(_) => 0
						};

						if size > 4 {
							let out : Vec<u8> = Tcp::handle_request(&buf[0..size], &mut address, &config, &nio);
	
							stream.write(out.as_slice()).unwrap();
	
						}
	
					},
					Err(_) => { }
				}
			}	    
		});
		
		match s.join() {
			Ok(_) => { },
			_ => println!("[Error] Error on TCP thread join"),
		}	
		Ok(Success::Ok)
		
	}
	
	pub fn handle_request(bytes: &[u8], address: &mut SocketAddr, config: &Config, nio: &NetspaceIo) -> Vec<u8> {
		let check = &bytes[0..4];
		
		if &check == &"POST".as_bytes() {
			// Here sort it as an HTTP service layer
			match HttpWrapper::deserialise_request(Vec::from(bytes), address) {
				Ok(msg) => {
					
					let svr = Svr::new(address.clone(), Box::new(config.clone()), nio);
					
					let m = Protocol::process(&msg, svr, Box::new(ChainService{}));
					return HttpWrapper::serialise_response_bytes(&m.to_bytes())
				},
				Err(_) => return HttpWrapper::serialise_response(&Message::from_bytes(b"104").unwrap())
			};
		}
		let svr = Svr::new(address.clone(), Box::new(config.clone()), nio);
		// Here we handle a straight DVSP TCP stream
		Protocol::process(&Message::from_bytes(bytes).unwrap(), svr, Box::new(ChainService{})).to_bytes()
	}


	pub fn make_request(msg: &Message, address: &str, host: &str, service: NodeService) -> Result<Message,Failure> {

		let (addr, serial) = match service {
			NodeService::Http => (
			 	format!("{}:{}", address, Port::Http),
			 	HttpWrapper::serialise_request(msg, host)
			),
			_ => (
				format!("{}:{}", address, Port::Stream),
				msg.to_bytes()
			)
		};

		let mut stream = match TcpStream::connect(addr.as_str()) {
			Ok(s) => s,
			Err(_) => return Err(Failure::InvalidArgument)
		};

		stream.write(serial.as_slice()).unwrap();

		let mut buf = [0;4096];
		let size = match stream.read(&mut buf) {
					Ok(s) => s,
					Err(_) => 0
		};

		if size == 0 { return Err(Failure::InvalidArgument) }
		
		if service == NodeService::Http {
			let (mut msgbuf, hdrend) = try!(HttpWrapper::deserialise_response(Vec::from(&buf[0..size])));

			match content_len(msgbuf.as_slice()) {
				Some((conlen,split)) => {
					let metalen = hdrend + split;

					if (metalen + conlen) > 4096 {
						let diff = conlen - (4096-metalen);
						let mut vbuf = Vec::new();
						vbuf.resize(diff, 0);
						match stream.read(&mut vbuf.as_mut_slice()) {
							Ok(s) => s,
							Err(_) =>  0
						};
						msgbuf.append(&mut vbuf);	
					}
				}
				_ => { }
			}
			
			let mstr = msgbuf.as_slice();


			match Message::from_bytes(mstr) {
				Ok(m) => Ok(m),
				Err(e) => {
					 println!("[Error] {:?}\nDumping:\n{}", e, str::from_utf8(mstr).unwrap());
					 Err(Failure::InvalidBytes)
				} 
			}

			
			
		} else {
			Ok(Message::from_bytes(&buf[0..size]).unwrap())
		}
	} 

}

impl Management {
	pub fn start(cfg: &Config) -> Result<Success,Failure> {
		let config = cfg.clone();
		
		thread::spawn(move|| {

			let _ = remove_file("primary.sock");
			let listener = UnixListener::bind("primary.sock").unwrap();				
			println!("[System] Management service online");
			
			for unix_stream in listener.incoming() {
				let c = config.clone();
				match unix_stream {
					Ok(stream) => {
						 thread::spawn(|| management_handler(stream, c));
						  },
					Err(_) => { break; }
				}
			}
			
			drop(listener);

		});
		
		Ok(Success::Ok)
	}
}
