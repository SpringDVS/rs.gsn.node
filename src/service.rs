#![allow(dead_code)]
extern crate epoll;

use std::os::unix::io::{AsRawFd, RawFd};
use std::net::UdpSocket;
use std::thread;

use spring_dvs::protocol::Ipv4;
use spring_dvs::formats::ipv4_to_str_address;



use netspace::*;

use self::epoll::*;
use self::epoll::util::*;
use ::config::Config;
use ::unit_test_env::setup_live_test_env;

//pub use spring_dvs::enums::{Success, Failure};

use ::protocol::process_packet;

/*
 * ToDo:
 * There is not timeout handling going on
 * The server could potentially hang until
 * fossil fuel runs out.
 */

pub fn start_dvsp(config: &Config) -> Result<Success,Failure> {
	
	let socket = match UdpSocket::bind("0.0.0.0:55301") {
			Ok(s) => s,
			Err(_) => return Err(Failure::InvalidArgument)
	};
	
	let epfd = epoll::create1(0).unwrap();
	let sfd = socket.as_raw_fd();

	let mut event = EpollEvent {
		data: sfd as u64,
		//events: (event_type::EPOLLIN | event_type::EPOLLET | event_type::EPOLLRDHUP)
		events: (event_type::EPOLLIN | event_type::EPOLLRDHUP)
	};
	
	match epoll::ctl(epfd, ctl_op::ADD, sfd, &mut event) {
		Ok(()) => { },
		Err(e) => println!("CtlError on add: {}", e)
	};
	
	let cfg_clone = config.clone();
	
	let s = thread::spawn(move|| {
		
		epoll_wait(epfd, socket, cfg_clone);	    
	});

	match s.join() {
		Ok(_) => println!("Joined thread"),
		_ => println!("Error on join"),
	}
	
	Ok(Success::Ok)
}




fn epoll_wait(epfd: RawFd, socket: UdpSocket, config: Config) {

	let mut bytes = [0;768];

	let mut events = Vec::<EpollEvent>::with_capacity(100);
  
    unsafe { events.set_len(100); }
    
    let nio = match config.live_test {
		false => {
			println!("Live System");
			NetspaceIo::new("gsn.db") 
		},
		true => {
			println!("Warning: Live testing enabled; using in memory database");
			let nio = NetspaceIo::new(":memory:");
			setup_live_test_env(&nio);
			nio
		}
	};
	
    println!("Started");
    loop {
	    match epoll::wait(epfd, &mut events[..], -1) {
	
	        Ok(num_events) => {
	            
	            
	            for _ in 0..num_events {
	
	       			let (sz, from) = match socket.recv_from(&mut bytes) {
						Err(_) => return,
						Ok(s) => s
					};
	

	
	            	let bytes = process_packet(&bytes[0..sz], &from, config, &nio);
	            	match socket.send_to(bytes.as_slice(), from) {
	            		Err(_) => return,
						_ => { }
	            	};
	
	            }
	        }
	
	        Err(e) => println!("Error on epoll::wait(): {}", e)
		}
    }
}

// ToDo clean this lot up -- better failure states
pub fn chain_request(bytes: Vec<u8>, target: &Node) -> Result<Vec<u8>, Failure> {
	// ToDo: Handle HTTP service layers
	let address : String = match target.service() {
		DvspService::Dvsp => format!("{}:55301", ipv4_to_str_address(&target.address())),
		_ => return Err(Failure::InvalidArgument)
	};

	let socket = match UdpSocket::bind("0.0.0.0:0") {
			Ok(s) => s,
			Err(_) => return Err(Failure::InvalidArgument)
	};

	match socket.send_to(bytes.as_ref(), address.as_str()) {
		Ok(_) =>{ },
		_ => return Err(Failure::InvalidArgument),
	}

	let mut buf = [0;768];
	let (sz, _) = match socket.recv_from(&mut buf) {
		Ok(t) => t,
		_ => { return Err(Failure::InvalidArgument) }
	};
	
	Ok(Vec::from(&buf[0..sz]))
}


mod tests {
	extern crate spring_dvs;
	
	#[allow(unused_imports)]
	use super::*;
	
}

