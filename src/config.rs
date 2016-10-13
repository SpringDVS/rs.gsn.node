use std::io::prelude::*;
use std::collections::HashMap;
use std::fs::{File};



pub trait NodeConfig {
	fn springname(&self) -> String;
	fn hostname(&self) -> String;
	fn geosub(&self) -> String;
	fn address(&self) -> String;
	fn uri(&self) -> String;
}

#[derive(Clone)]
pub struct Config {
	node: HashMap<String,String>,
	pub live_test: bool,
	pub toggle_man: bool,
	pub toggle_offline: bool,
}

impl Config {
	#[allow(dead_code)]
	pub fn new() -> Config {
		Config {
			node: Config::load_kvs(),
			live_test: false,
			toggle_man: true,
			toggle_offline: false,
		}
	}
	
	#[allow(dead_code)]
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
		match self.node.get(key) {
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
		self.get_key("geosub")
	}

	fn address(&self) -> String {
		self.get_key("address")
	}
	
	fn uri(&self) -> String {
		format!("{}.{}.uk", self.springname(),self.geosub())
	}
}

#[cfg(test)]
pub mod mocks {
	pub struct MockConfig {
		spring: String,
		host: String,
		geosub: String,
		address: String,
		uri: String
	}
	
	impl ::config::NodeConfig for MockConfig {
		
		fn springname(&self) -> String {
			self.spring.clone()
		}
		fn hostname(&self) -> String {
			self.host.clone()
		}
		fn geosub(&self) -> String {
			self.geosub.clone()
		}
		fn address(&self) -> String {
			self.address.clone()
		}
		fn uri(&self) -> String {
			self.uri.clone()
		}
	}
	
	impl MockConfig {
		pub fn dflt() -> MockConfig {
			MockConfig {
				spring: String::from("foohub"),
				host: String::from("barhub.zni.lan"),
				geosub: String::from("esusx"),
				address: String::from("127.0.0.1"),
				uri: String::from("foohub.esusx.uk"),
			}
		}
	}
	
}