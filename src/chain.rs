use std::net::UdpSocket;
use std::io::{ErrorKind};
use std::time::Duration;

use spring_dvs::enums::{NodeService};
use spring_dvs::node::Node;
use spring_dvs::protocol::Port;
pub use network::NetworkFailure;

pub trait Chain {
	fn request(&self, bytes: &Vec<u8>, target: &Node) -> Result<Vec<u8>, NetworkFailure> ;
}

// ToDo clean this lot up -- better failure states
// ToDo: Handle HTTP service layer chains

pub struct ChainService;

impl ChainService {
	fn dvsp(&self, bytes: &Vec<u8>, target: &Node) -> Result<Vec<u8>, NetworkFailure> {
		let address = format!("{}:{}", target.address(), Port::Dvsp);
		
		let socket = match UdpSocket::bind("0.0.0.0:0") {
				Ok(s) => s,
				Err(_) => return Err(NetworkFailure::Bind)
		};
		
		match socket.set_read_timeout(Some(Duration::new(20,0))) { // 20 Second Timeout
			Ok(_) => { },
			_ => return Err(NetworkFailure::SocketError)
		}
		
		match socket.send_to(bytes.as_ref(), address.as_str()) {
			Ok(_) =>{ },
			_ => return Err(NetworkFailure::SocketWrite),
		}
		
		let mut buf = [0;768];
		let (sz, _) = match socket.recv_from(&mut buf) {
			Ok(t) => t,
			Err(e) => {
				match e.kind() { 
					ErrorKind::TimedOut => return Err(NetworkFailure::TimedOut),
					_ => return Err(NetworkFailure::SocketRead) 
				}
			} 

		};
		
		Ok(Vec::from(&buf[0..sz]))		
	}
}

impl Chain for ChainService {
	fn request(&self, bytes: &Vec<u8>, target: &Node) -> Result<Vec<u8>, NetworkFailure> {
		// ToDo: Handle HTTP service layers
		match target.service() {
			NodeService::Dvsp => self.dvsp(bytes,target),
			_ => Err(NetworkFailure::UnsupportedAction)
		}
	}
}

#[cfg(test)]
pub mod mocks {
	extern crate spring_dvs;
	use spring_dvs::node::{Node};
	use spring_dvs::protocol::{ProtocolObject, Message};
	
	use super::*;

	pub struct MockChain {
		target: String
	}
	
	impl MockChain {
		pub fn new(target: &str) -> MockChain {
			MockChain {
				target: String::from(target),
			}
		}
	}
	
	impl Chain for MockChain {
		#[allow(unused_variables)]
		fn request(&self, bytes: &Vec<u8>, target: &Node) -> Result<Vec<u8>, NetworkFailure> {
			
			assert_eq!(target.springname(), self.target);
			Ok(Message::from_bytes(b"200").unwrap().to_bytes())
		}
	}
}