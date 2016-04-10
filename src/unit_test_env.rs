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