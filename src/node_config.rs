use std::fs::File;
use std::io::prelude::*;
#[allow(dead_code)]
pub fn node_springname() -> String {
	let mut f : File = match File::open("node.conf") {
		Ok(f) => f,
		_ => return "".to_string()
	};
	
	let mut s = String::new();
	
	match f.read_to_string(&mut s) {
		Ok(_) => { },
		_ => return "".to_string()
	};
	
	let lines = s.lines();
	for line in lines {
		let kvp : Vec<&str> = line.split('=').collect();
		match kvp[0] {
			"springname" => return String::from(kvp[1]),
			_ => {}
		}
	}
	return "".to_string();
	
}

pub fn node_geosub() -> String {
	let mut f : File = match File::open("node.conf") {
		Ok(f) => f,
		_ => return "".to_string()
	};
	
	let mut s = String::new();
	
	match f.read_to_string(&mut s) {
		Ok(_) => { },
		_ => return "".to_string()
	};
	
	let lines = s.lines();
	for line in lines {
		let kvp : Vec<&str> = line.split('=').collect();
		match kvp[0] {
			"geosub" => return String::from(kvp[1]),
			_ => {}
		}
	}
	return "".to_string();
	
}

pub fn node_hostname() -> String {
	let mut f : File = match File::open("node.conf") {
		Ok(f) => f,
		_ => return "".to_string()
	};
	
	let mut s = String::new();
	
	match f.read_to_string(&mut s) {
		Ok(_) => { },
		_ => return "".to_string()
	};
	
	let lines = s.lines();
	for line in lines {
		let kvp : Vec<&str> = line.split('=').collect();
		match kvp[0] {
			"hostname" => return String::from(kvp[1]),
			_ => {}
		}
	}
	return "".to_string();
	
}

pub fn node_resource() -> String {
	let mut f : File = match File::open("node.conf") {
		Ok(f) => f,
		_ => return "".to_string()
	};
	
	let mut s = String::new();
	
	match f.read_to_string(&mut s) {
		Ok(_) => { },
		_ => return "".to_string()
	};
	
	let lines = s.lines();
	for line in lines {
		let kvp : Vec<&str> = line.split('=').collect();
		match kvp[0] {
			"resource" => return String::from(kvp[1]),
			_ => {}
		}
	}
	return "".to_string();
	
}

pub fn node_address() -> String {
	let mut f : File = match File::open("node.conf") {
		Ok(f) => f,
		_ => return "".to_string()
	};
	
	let mut s = String::new();
	
	match f.read_to_string(&mut s) {
		Ok(_) => { },
		_ => return "".to_string()
	};
	
	let lines = s.lines();
	for line in lines {
		let kvp : Vec<&str> = line.split('=').collect();
		match kvp[0] {
			"address" => return String::from(kvp[1]),
			_ => {}
		}
	}
	return "".to_string();
	
}