extern crate sqlite;
use self::sqlite::{State,Statement};

use spring_dvs::model::Node;
use spring_dvs::formats::ipv4_to_str_address;
use ::netspace::NetspaceIo;
use ::config::Config;



pub fn setup_live_test_env(nio: &NetspaceIo) {
	nio.db().execute("
		CREATE TABLE \"geosub_netspace\" (
			`id`	INTEGER PRIMARY KEY AUTOINCREMENT,
			`springname`	TEXT UNIQUE,
			`hostname`	TEXT,
			`address`	TEXT,
			`service`	INTEGER,
			`status`	INTEGER,
			`types`	INTEGER
		);
		CREATE TABLE \"geosub_metaspace\" (
			`id`	INTEGER PRIMARY KEY AUTOINCREMENT,
			`settlement`	TEXT,
			`postcode`	TEXT,
			`county`	TEXT,
			`geosub`	TEXT
		);").unwrap();
}

pub fn reset_live_test_env(nio: &NetspaceIo, config: &Config) {
	if config.live_test == false { return }
	println!("Reset in-memory database");
	nio.db().execute("DELETE FROM \"geosub_netspace\"").unwrap();
	nio.db().execute("DELETE FROM \"geosub_metaspace\"").unwrap();
}

pub fn update_address_test_env(nio: &NetspaceIo, nodestring: &str , config: &Config) {
	if config.live_test == false { return }
	let node : Node = match Node::from_node_string(nodestring) {
		Err(_) => return,
		Ok(n) => n
	};
	
	println!("NodeString: {}", node.to_node_string());
	let mut statement = nio.db().prepare(
					"UPDATE  
					`geosub_netspace`
					SET address = ?
					WHERE springname = ?").unwrap();
	
	statement.bind(1, &sqlite::Value::String( ipv4_to_str_address(&node.address()) ) ).unwrap();
	statement.bind(2, &sqlite::Value::String( String::from(node.springname()) ) ).unwrap();
	
	match statement.next() {
		_ => {},   
	}
}