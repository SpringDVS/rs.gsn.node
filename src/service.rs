#![allow(dead_code)]
extern crate epoll;

use std::os::unix::io::{AsRawFd, RawFd};
use std::net::UdpSocket;
use std::thread;

use self::epoll::*;
use self::epoll::util::*;


pub use spring_dvs::enums::{Success, Failure};

use ::protocol::process_packet;

pub fn start_dvsp() -> Result<Success,Failure> {
	
	let socket = match UdpSocket::bind("0.0.0.0:55301") {
			Ok(s) => s,
			Err(_) => return Err(Failure::InvalidArgument)
	};
	
	let epfd = epoll::create1(0).unwrap();
	let sfd = socket.as_raw_fd();

	let mut event = EpollEvent {
		data: sfd as u64,
		events: (event_type::EPOLLIN | event_type::EPOLLET | event_type::EPOLLRDHUP)
	};
	
	match epoll::ctl(epfd, ctl_op::ADD, sfd, &mut event) {
		Ok(()) => println!("Added successfully"),
		Err(e) => println!("CtlError on add: {}", e)
	};
	
	
	let s = thread::spawn(move|| {
		println!("Started");
		epoll_wait(epfd, socket);	    
	});

	match s.join() {
		Ok(_) => println!("Joined thread"),
		_ => println!("Error on join"),
	}
	
	Ok(Success::Ok)
}

fn epoll_wait(epfd: RawFd, socket: UdpSocket) {

	let mut bytes = [0;768];

	let mut events = Vec::<EpollEvent>::with_capacity(100);
  
    unsafe { events.set_len(100); }
    
    loop {
	    match epoll::wait(epfd, &mut events[..], -1) {
	
	        Ok(num_events) => {
	            
	            
	            for _ in 0..num_events {
	
	       			let (sz, from) = match socket.recv_from(&mut bytes) {
						Err(_) => return,
						Ok(s) => s
					};
	
	       			println!("From {}", from);
	
	            	let bytes = process_packet(&bytes[0..sz], &from);
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


mod tests {
	extern crate spring_dvs;
	
	#[allow(unused_imports)]
	use super::*;
	
}

