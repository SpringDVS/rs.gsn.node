extern crate sqlite;

use ::netspace::NetspaceIo;
use ::config::Config;



pub fn setup_live_test_env(nio: &NetspaceIo, config: &Config) {
	if config.live_test == false { return }
	let _ = nio.db().execute("
		CREATE TABLE \"geosub_netspace\" (
			`id`			INTEGER PRIMARY KEY AUTOINCREMENT,
			`springname`	TEXT UNIQUE,
			`hostname`		TEXT,
			`address`		TEXT,
			`service`		INTEGER,
			`status`		INTEGER,
			`types`			INTEGER,
			`key`			TEXT
		);
		CREATE TABLE \"geosub_metaspace\" (
			`id`			INTEGER PRIMARY KEY AUTOINCREMENT,
			`settlement`	TEXT,
			`postcode`		TEXT,
			`county`		TEXT,
			`geosub`		TEXT
		);
		CREATE TABLE `geotop_netspace` (
			`id`			INTEGER PRIMARY KEY AUTOINCREMENT,
			`springname`	TEXT,
			`hostname`		TEXT,
			`address`		TEXT,
			`service`		INTEGER,
			`priority`		INTEGER,
			`geosub`		TEXT,
			`key`			TEXT
		);
		CREATE TABLE `geosub_tokens` (
			`id`	INTEGER PRIMARY KEY AUTOINCREMENT,
			`token`	TEXT,
			`spring`	TEXT
		);
		");
	
	reset_live_test_env(nio, config);
}

// 3858f62230ac3c915f300c664312c63f
pub fn reset_live_test_env(nio: &NetspaceIo, config: &Config) {
	if config.live_test == false { return }
	println!("[Database] Reset testing database");
	nio.db().execute("DELETE FROM \"geosub_netspace\"").unwrap();
	nio.db().execute("DELETE FROM \"geotop_netspace\"").unwrap();
	nio.db().execute("DELETE FROM \"geosub_metaspace\"").unwrap();
	nio.db().execute("DELETE FROM \"geosub_tokens\"").unwrap();

	let mut statement = nio.db().prepare(
					"
					INSERT  
					INTO `geosub_tokens`
					(token) VALUES (?)
					").unwrap();
	
	statement.bind(1, &sqlite::Value::String( "3858f62230ac3c915f300c664312c63f".to_string() ) ).unwrap();
	
	
	match statement.next() {
		_ => {},   
	}	
}

/*
pub fn update_address_test_env(nio: &NetspaceIo, nodestring: &str , config: &Config) {
	if config.live_test == false { return }
	let node : Node = match Node::from_str(nodestring) {
		Err(_) => return,
		Ok(n) => n
	};
	
	let mut statement = nio.db().prepare(
					"UPDATE  
					`geosub_netspace`
					SET address = ?
					WHERE springname = ?").unwrap();
	
	statement.bind(1, &sqlite::Value::String( String::from(node.address()) ) ).unwrap();
	statement.bind(2, &sqlite::Value::String( String::from(node.springname()) ) ).unwrap();
	
	match statement.next() {
		_ => {},   
	}
}

pub fn add_geosub_root_test_env(nio: &NetspaceIo, nodereggtn: &str, config: &Config) {
	if config.live_test == false { return }
	let mut node = Node::from_str(nodereggtn).unwrap();
	node.update_service(NodeService::Dvsp);
	let gsn = match geosub_from_node_register_gtn(nodereggtn) {
		Ok(g) => g,
		_ => return,
	};
	
	nio.gtn_geosub_register_node(&node, &gsn).unwrap();
}
*/