extern crate sqlite;
use self::sqlite::{State,Statement};

use spring_dvs::model::{Node,Netspace};
use spring_dvs::formats::{ipv4_to_str_address, geosub_from_node_register_gtn};
use spring_dvs::enums::{DvspService};
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
		);
		CREATE TABLE `geotop_netspace` (
			`id`	INTEGER PRIMARY KEY AUTOINCREMENT,
			`springname`	TEXT,
			`hostname`	TEXT,
			`address`	TEXT,
			`service`	INTEGER,
			`priority`	INTEGER,
			`geosub`	TEXT
		);
		").unwrap();
}

pub fn reset_live_test_env(nio: &NetspaceIo, config: &Config) {
	if config.live_test == false { return }
	println!("[Database] Reset in-memory database");
	nio.db().execute("DELETE FROM \"geosub_netspace\"").unwrap();
	nio.db().execute("DELETE FROM \"geotop_netspace\"").unwrap();
	nio.db().execute("DELETE FROM \"geosub_metaspace\"").unwrap();
	
}

pub fn update_address_test_env(nio: &NetspaceIo, nodestring: &str , config: &Config) {
	if config.live_test == false { return }
	let node : Node = match Node::from_node_string(nodestring) {
		Err(_) => return,
		Ok(n) => n
	};
	
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

pub fn add_geosub_root_test_env(nio: &NetspaceIo, nodereggtn: &str, config: &Config) {
	if config.live_test == false { return }
	let mut node = Node::from_node_string(nodereggtn).unwrap();
	node.update_service(DvspService::Dvsp);
	let gsn = match geosub_from_node_register_gtn(nodereggtn) {
		Ok(g) => g,
		_ => return,
	};
	
	nio.gtn_geosub_register_node(&node, &gsn).unwrap();
}