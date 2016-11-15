extern crate unix_socket;

use std::io::prelude::*;
use std::mem;
use std::str::FromStr;

use ::protocol::{SocketAddr,Svr};
use netspace::{NetspaceIo,Config};

use self::unix_socket::UnixStream;



#[macro_export]
macro_rules! cascade_none_nowrap {
	($opt: expr) => (
		match $opt {
			Some(s) => s,
			_ => return None,
		}
	)
}


mod network;
mod validation;
mod service;

use self::validation::ValidationZone;
use self::network::NetworkZone;
use self::service::ServiceZone;

fn binary_split(msg: &str) -> Vec<&str> {
	msg.splitn(2, " ").collect()
}

pub trait ManagedService {
	fn init(&self) -> String;
	fn hook(&self, atom: &Vec<String>, svr: &Svr) -> String;
}


pub fn management_handler(mut stream: UnixStream, config: Config) {
	
	let nio = match config.live_test {
		false => {
			NetspaceIo::new("/var/lib/springdvs/gsn.db") 
		},
		true => {
			NetspaceIo::new("live-testing.db")
		}
	};
	
	let svr = Svr::new(SocketAddr::from_str("0.0.0.0:0").unwrap(), Box::new(config.clone()), &nio);
	
	let mut szin_buf = [0;4];
	
	stream.read_exact(&mut szin_buf).unwrap();
	
	let szin : u32 = unsafe { mem::transmute(szin_buf) };
	
	let mut bufin : Vec<u8> = Vec::new();
	bufin.resize(szin as usize, b'\0');
	stream.read_exact(bufin.as_mut_slice()).unwrap();
	let command = String::from_utf8(bufin).unwrap();
	
	let mi = ManagementInstance::new();
	
	let out = match mi.run(&command, &svr) {
		Some(s) => s,
		None => "Error: Unrecognised or malformed command".to_string() 
	};
	stream.write_all(out.as_bytes()).unwrap();
}

struct ManagementInstance;

impl ManagementInstance {
	pub fn new() -> Self {
		ManagementInstance
	}
	pub fn run(&self, command: &str, svr: &Svr) -> Option<String> {
		self.process_request(cascade_none_nowrap!(ManagementZone::from_str(command)), svr)
	}

	pub fn process_request(&self, request: ManagementZone, svr: &Svr) -> Option<String> {
		match request {
			ManagementZone::Network(nz) => NetworkZone::process(nz, svr.nio),
			ManagementZone::Validation(vz) => ValidationZone::process(vz, svr.nio),
			ManagementZone::Service(sz) => ServiceZone::process(sz, svr)
		}
	}
}

#[derive(Clone, PartialEq, Debug)]
pub enum ManagementZone {
	Network(network::NetworkZone), Validation(validation::ValidationZone),
	Service(service::ServiceZone)
}

impl ManagementZone {
	pub fn from_str(msg: &str) -> Option<ManagementZone> {
		if msg.len() == 0 { return None; }
		
		let atom = binary_split(msg);
		
		Some(match atom[0] {
			"net" | "network" => {
				ManagementZone::Network(cascade_none_nowrap!(NetworkZone::from_str(atom[1])))				
			},
			"val" | "validation" => {
				ManagementZone::Validation(cascade_none_nowrap!(ValidationZone::from_str(atom[1])))
			},
			"ser" | "service" => {
				ManagementZone::Service(cascade_none_nowrap!(ServiceZone::from_str(atom[1])))
			},
			_ => return None
		})
		
	}
}
