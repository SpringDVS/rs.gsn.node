use std::io::prelude::*;
use std::collections::HashMap;
use std::fs::{File};



pub trait NodeConfig {
	fn springname(&self) -> String;
	fn hostname(&self) -> String;
	fn geosub(&self) -> String;
	fn address(&self) -> String;
}

#[derive(Clone)]
pub struct Config {
	node: HashMap<String,String>,
	pub live_test: bool,
}

impl Config {
	pub fn new() -> Config {
		Config {
			node: Config::load_kvs(),
			live_test: false,
		}
	}
	
	fn load_kvs() -> HashMap<String,String> {
		
		let mut kvs: HashMap<String,String> = HashMap::new();

		let mut f : File = match File::open("/etc/springdvs/node.conf") {
			Ok(f) => f,
			_ => return kvs
		};
		
		let mut s = String::new();
		
		match f.read_to_string(&mut s) {
			Ok(_) => { },
			_ => return kvs
		};
		

		for line in s.lines() {	
			let kvp : Vec<&str> = line.split('=').collect();
			if kvp.len() != 2 { continue }
			
			kvs.insert(String::from(kvp[0]), String::from(kvp[1]));
		};
		
		kvs
	}
	
	fn get_key(&self, key: &str) -> String {
		match self.node.get("spring") {
			Some(s) => s.clone(),
			None => String::new(),
		}
	}
}

impl NodeConfig for Config {
	fn springname(&self) -> String {
		self.get_key("springname")
	}

	fn hostname(&self) -> String {
		self.get_key("hostname")
	}

	fn geosub(&self) -> String {
		self.get_key("gsn")
	}

	fn address(&self) -> String {
		self.get_key("address")
	}	
}