
extern crate sqlite;


pub use spring_dvs::enums::{Failure,Success};
pub use spring_dvs::node::{Node,NodeRole,NodeService,NodeState,ParseFailure};
pub use spring_dvs::spaces::{Netspace,NetspaceFailure};
pub use config::{NodeConfig, Config};



use self::sqlite::{State,Statement};

/*
 * Fix:
 * Using `priority` and `service` as interchangable in the generation of
 * of nodes from Database results is very sketchy and will eventually 
 * lead to ruin!
 */
  

pub struct NetspaceIo {
	db: sqlite::Connection,
}


impl NetspaceIo {
	
	pub fn new(database: &str) -> NetspaceIo {
		NetspaceIo {
			db : sqlite::open(database).unwrap()
		}
	}
	
	pub fn db(&self) -> &sqlite::Connection {
		&self.db
	}
	
	fn fill_node(&self, statement: &sqlite::Statement) -> Result<Node,NetspaceFailure> {
		let spring = statement.read::<String>(1).unwrap();
		let host = statement.read::<String>(2).unwrap();
		let addr = statement.read::<String>(3).unwrap();
		let service = NodeService::from_i64(statement.read::<i64>(4).unwrap());
		let state = NodeState::from_i64(statement.read::<i64>(5).unwrap());
		let role =  NodeRole::from_i64(statement.read::<i64>(6).unwrap());
		
		Ok(Node::new(&spring, &host, &addr, service, state, role))
	}
	
	fn vector_from_statement(&self, statement: &mut Statement) -> Vec<Node> {
		
		let mut v: Vec<Node> = Vec::new();
		
		while let State::Row = statement.next().unwrap() {
			match self.fill_node(&statement) {
				Ok(node) => v.push(node),
				_ => {}
			}; 		   
		}
		
		v
	}
	
	fn node_from_statement(&self, statement: &mut Statement) -> Result<Node,NetspaceFailure> {

		match statement.next() {
			Ok(state) => match state {
				
							State::Row => self.fill_node(&statement),
			 				_ => Err(NetspaceFailure::NodeNotFound)
			 				
						},

			_ => Err(NetspaceFailure::DatabaseError)

		}
		
	}
	
	#[allow(dead_code)]
	fn debug_print_rows(&self, statement: &mut Statement) {
		
		while let State::Row = statement.next().unwrap() {
			
			println!("id = {}", statement.read::<i64>(0).unwrap());
			println!("spring = {}", statement.read::<String>(1).unwrap());
			println!("host = {}", statement.read::<String>(2).unwrap());
			println!("address = {}", statement.read::<String>(3).unwrap());
			println!("service = {}", statement.read::<i64>(4).unwrap());
			println!("status = {}", statement.read::<i64>(5).unwrap());
			println!("types = {}", statement.read::<i64>(6).unwrap());
			    			
		}
		
		match statement.reset() {
			_ => return
		};
		
	}
}

impl Netspace for NetspaceIo {

	fn gsn_nodes(&self) -> Vec<Node> {
		let mut statement = self.db.prepare("
	    	SELECT * FROM geosub_netspace
			").unwrap();
			
			self.vector_from_statement(&mut statement)
	}
	
	fn gsn_nodes_by_address(&self, address: &str) -> Vec<Node> {
		
		let mut statement = self.db.prepare("
    	SELECT * FROM geosub_netspace WHERE address = ?
		").unwrap();

		statement.bind(1, &sqlite::Value::String( String::from(address) ) ).unwrap();
		
		self.vector_from_statement(&mut statement)
		
	}

	
	fn gsn_nodes_by_type(&self, types: NodeRole) -> Vec<Node> {
		
		let mut statement = self.db.prepare("
    	SELECT * FROM geosub_netspace WHERE types & ?
		").unwrap();

		statement.bind(1, &sqlite::Value::Integer( types as i64 ) ).unwrap();
		
		self.vector_from_statement(&mut statement)
	}

	fn gsn_nodes_by_state(&self, state: NodeState) -> Vec<Node> {
		
		let mut statement = self.db.prepare("
    	SELECT * FROM geosub_netspace WHERE status = ?
		").unwrap();

		statement.bind(1, &sqlite::Value::Integer( state as i64 ) ).unwrap();
		
		self.vector_from_statement(&mut statement)
	}
	
	fn gsn_node_by_springname(&self, name: &str) -> Result<Node, NetspaceFailure> {
		
		let mut statement = self.db.prepare("
    	SELECT * FROM geosub_netspace WHERE springname = ?
		").unwrap();

		statement.bind(1, &sqlite::Value::String( String::from(name) ) ).unwrap();
		self.node_from_statement(&mut statement)
	}
	
	fn gsn_node_by_hostname(&self, name: &str) -> Result<Node,NetspaceFailure> {
		
		let mut statement = self.db.prepare("
    	SELECT * FROM geosub_netspace WHERE hostname = ?
		").unwrap();

		statement.bind(1, &sqlite::Value::String( String::from(name) ) ).unwrap();
		self.node_from_statement(&mut statement)

	}
	
	fn gtn_root_nodes(&self) -> Vec<Node> {
		let v: Vec<Node> = Vec::new();
		
		v
	}
	fn gtn_geosubs(&self) -> Vec<String> {
		
		let mut statement = self.db.prepare(
			"SELECT DISTINCT `geosub` FROM 
			`geotop_netspace`
			").unwrap();
		
		let mut v: Vec<String> = Vec::new();
		
		while let State::Row = statement.next().unwrap() {
			v.push(statement.read::<String>(0).unwrap()); 		   
		}
		
		v
	}
	
	
	fn gsn_node_register(&self, node: &Node) -> Result<Success,NetspaceFailure> {
		
		if self.gsn_node_by_springname(node.springname()).is_ok() {
			return Err(NetspaceFailure::DuplicateNode)
		}
		
		let mut statement = self.db.prepare(
						"INSERT INTO 
						`geosub_netspace` 
						(springname,hostname,address,service,status,types) 
						VALUES (?,?,?,?,?,?)").unwrap();
		statement.bind(1, &sqlite::Value::String( String::from(node.springname()) ) ).unwrap();
		statement.bind(2, &sqlite::Value::String( String::from(node.hostname()) ) ).unwrap();
		statement.bind(3, &sqlite::Value::String( String::from(node.address()) )).unwrap();
		statement.bind(4, &sqlite::Value::Integer( node.service() as i64 ) ).unwrap();
		
		// Regardless of what is set -- the node should be disabled when it is registered
		statement.bind(5, &sqlite::Value::Integer( NodeState::Disabled as i64 ) ).unwrap();
		statement.bind(6, &sqlite::Value::Integer( node.role() as i64 ) ).unwrap();
		match statement.next() {
			Ok(_) => Ok(Success::Ok),
			Err(_) => Err(NetspaceFailure::NodeNotFound)   
		}
		
	}

	fn gsn_node_unregister(&self, node: &Node) -> Result<Success,NetspaceFailure> {
		if self.gsn_node_by_springname(node.springname()).is_err() {
			return Err(NetspaceFailure::NodeNotFound)
		}
		
		let mut statement = self.db.prepare(
						"DELETE FROM 
						`geosub_netspace` WHERE 
						springname = ?").unwrap();
		statement.bind(1, &sqlite::Value::String( String::from(node.springname()) ) ).unwrap();
		
		match statement.next() {
			Ok(_) => Ok(Success::Ok),
			Err(_) => Err(NetspaceFailure::DatabaseError)   
		}		
	}

	fn gsn_node_update_state(&self, node: &Node) -> Result<Success,NetspaceFailure> {
	
		if self.gsn_node_by_springname(node.springname()).is_err() {
			return Err(NetspaceFailure::NodeNotFound)
		}
		
		let mut statement = self.db.prepare(
						"UPDATE  
						`geosub_netspace`
						SET status = ?
						WHERE springname = ?").unwrap();
		
		statement.bind(1, &sqlite::Value::Integer( node.state() as i64 ) ).unwrap();
		statement.bind(2, &sqlite::Value::String( String::from(node.springname()) ) ).unwrap();
		
		match statement.next() {
			Ok(_) => Ok(Success::Ok),
			Err(_) => Err(NetspaceFailure::DatabaseError)   
		}
	}
	
	fn gsn_node_update_role(&self, node: &Node) -> Result<Success,NetspaceFailure> {
	
		if self.gsn_node_by_springname(node.springname()).is_err() {
			return Err(NetspaceFailure::NodeNotFound)
		}
		
		let mut statement = self.db.prepare(
						"UPDATE  
						`geosub_netspace`
						SET types = ?
						WHERE springname = ?").unwrap();
		
		statement.bind(1, &sqlite::Value::Integer( node.role() as i64 ) ).unwrap();
		statement.bind(2, &sqlite::Value::String( String::from(node.springname()) ) ).unwrap();
		
		match statement.next() {
			Ok(_) => Ok(Success::Ok),
			Err(_) => Err(NetspaceFailure::DatabaseError)   
		}
	}

	fn gsn_node_update_service(&self, node: &Node) -> Result<Success,NetspaceFailure> {
	
		if self.gsn_node_by_springname(node.springname()).is_err() {
			return Err(NetspaceFailure::NodeNotFound)
		}
		
		let mut statement = self.db.prepare(
						"UPDATE  
						`geosub_netspace`
						SET service = ?
						WHERE springname = ?").unwrap();
		
		statement.bind(1, &sqlite::Value::Integer( node.service() as i64 ) ).unwrap();
		statement.bind(2, &sqlite::Value::String( String::from(node.springname()) ) ).unwrap();
		
		match statement.next() {
			Ok(_) => Ok(Success::Ok),
			Err(_) => Err(NetspaceFailure::NodeNotFound)   
		}
	}
	
	fn gtn_geosub_root_nodes(&self, gsn: &str) -> Vec<Node> {
		let mut statement = self.db.prepare("
	    	SELECT * FROM `geotop_netspace`
	    	WHERE geosub = ?
	    	ORDER BY priority ASC
			").unwrap();
		
		statement.bind(1, &sqlite::Value::String( String::from(gsn) ) ).unwrap();
			
		self.vector_from_statement(&mut statement)		
	}
	
	
	fn gtn_geosub_node_by_springname(&self, name: &str, gsn: &str) -> Result<Node,NetspaceFailure> {
		let mut statement = self.db.prepare("
    	SELECT * FROM geotop_netspace 
    	WHERE springname = ?
    	AND geosub = ?
		").unwrap();

		statement.bind(1, &sqlite::Value::String( String::from(name) ) ).unwrap();
		statement.bind(2, &sqlite::Value::String( String::from(gsn)  ) ).unwrap();
		
		self.node_from_statement(&mut statement)
	}

	fn gtn_geosub_register_node(&self, node: &Node, gsn: &str) -> Result<Success,NetspaceFailure> {
		
		if self.gtn_geosub_node_by_springname(node.springname(), &gsn).is_ok() {
			return Err(NetspaceFailure::DuplicateNode)
		}
		
		let mut statement = self.db.prepare(
						"INSERT INTO 
						`geotop_netspace` 
						(springname,hostname,address,service,priority, geosub) 
						VALUES (?,?,?,?,?,?)").unwrap();
		statement.bind(1, &sqlite::Value::String( String::from(node.springname()) ) ).unwrap();
		statement.bind(2, &sqlite::Value::String( String::from(node.hostname()) ) ).unwrap();
		statement.bind(3, &sqlite::Value::String( String::from(node.address()) ) ).unwrap();
		statement.bind(4, &sqlite::Value::Integer( node.service() as i64 ) ).unwrap();
		statement.bind(5, &sqlite::Value::Integer( 1 as i64)).unwrap();
		statement.bind(6, &sqlite::Value::String( String::from(gsn) )).unwrap();
			
		match statement.next() {
			Ok(_) => Ok(Success::Ok),
			Err(_) => Err(NetspaceFailure::NodeNotFound)   
		}	
	}

	fn gtn_geosub_unregister_node(&self, node: &Node, gsn: &str) -> Result<Success,NetspaceFailure> {

		if self.gtn_geosub_node_by_springname(node.springname(), &gsn).is_err() {
			return Err(NetspaceFailure::NodeNotFound)
		}

		let mut statement = self.db.prepare(
						"DELETE FROM `geotop_netspace` 
						WHERE springname = ?
						AND geosub = ?").unwrap();

		statement.bind(1, &sqlite::Value::String( String::from(node.springname()) ) ).unwrap();
		statement.bind(2, &sqlite::Value::String( String::from(gsn) ) ).unwrap();

		match statement.next() {
			Ok(_) => Ok(Success::Ok),
			Err(_) => Err(NetspaceFailure::NodeNotFound)   
		}				
	}
	
	fn gsn_check_token(&self, token: &str) -> bool {
		let mut statement = self.db().prepare("
	    	SELECT * FROM geosub_tokens WHERE token = ?
			").unwrap();
	
		statement.bind(1, &sqlite::Value::String( String::from( String::from(token) ) ) ).unwrap();
			
		match statement.next() {
			Ok(State::Row) => true,
			_ => false
	}	
}
}

pub fn netspace_routine_is_registered(node: &Node, nio: &NetspaceIo) -> bool {
	
	match nio.gsn_node_by_hostname(&node.hostname()) {
		Ok(_) => true,
		
		Err(_) => { 
			match nio.gsn_node_by_springname(&node.springname()) {
				Ok(_) => true,
				Err(_) => false
			}
		}
	}

}

// Fix:
// Checking by address is unsafe -- this is where we need to 
// implement certificates after the prototype
pub fn netspace_routine_is_address_gsn_root(address: &str, gsn: &str, nio: &NetspaceIo) -> bool {
	let nodes = nio.gtn_geosub_root_nodes(gsn);
	
	for node in nodes {
		if node.address() == address { return true }
	}
	
	false
}


pub fn netspace_add_self(ns: &Netspace, cfg: &Config) {
	let s : String = format!("spring:{},host:{},address:{},service:dvsp,role:hub,state:enabled",cfg.springname(), cfg.hostname(), cfg.address());
	let n = Node::from_str(&s).unwrap();
	ns.gsn_node_register(&n).unwrap();
	ns.gsn_node_update_state(&n).unwrap();
	
	ns.gtn_geosub_register_node(&n, &cfg.geosub()).unwrap();
}

#[cfg(test)]
mod tests {
	extern crate sqlite;
	extern crate spring_dvs;
	
	#[allow(unused_imports)]
	use super::*;
	
	
	
	#[allow(dead_code)]
	fn setup_netspace(db: &sqlite::Connection) {
		db.execute("
		CREATE TABLE `geosub_netspace` (
			`id`	INTEGER PRIMARY KEY AUTOINCREMENT,
			`springname`	TEXT UNIQUE,
			`hostname`	TEXT,
			`address`	TEXT,
			`service`	INTEGER,
			`status`	INTEGER,
			`types`	INTEGER
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
		CREATE TABLE `geosub_tokens` (
			`id`	INTEGER PRIMARY KEY AUTOINCREMENT,
			`token`	TEXT
		);

		INSERT INTO `geosub_netspace` (id,springname,hostname,address,service,status,types) VALUES (1,'esusx','greenman.zu','192.168.1.1',1,1,1);
		INSERT INTO `geosub_netspace` (id,springname,hostname,address,service,status,types) VALUES (2,'cci','dvsnode.greenman.zu','192.168.1.2',2,1,2);
		INSERT INTO `geotop_netspace` (id,springname,hostname,address,service,priority,geosub) VALUES (1,'springa', 'greenman', '192.168.1.2', 1, 2, 'esusx');
		INSERT INTO `geotop_netspace` (id,springname,hostname,address,service,priority,geosub) VALUES (2,'springb', 'blueman', '192.168.1.3', 2, 1, 'esusx');
		INSERT INTO `geosub_tokens` (token) VALUES ('3858f62230ac3c915f300c664312c63f');
		").unwrap();
	}

	#[test]
	fn ts_netspaceio_gsn_nodes() {

		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let v = nsio.gsn_nodes();
		assert_eq!(2, v.len());
	}

	#[test]
	fn ts_netspaceio_gsn_nodes_by_address_p() {

		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let v = nsio.gsn_nodes_by_address("192.168.1.1");
		assert_eq!(1, v.len());
		assert_eq!("192.168.1.1", v[0].address());
	}

	#[test]
	fn ts_netspaceio_gsn_nodes_by_address_f() {

		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let v = nsio.gsn_nodes_by_address("192.168.1.3");
		assert_eq!(0, v.len());
	}
	
	#[test]
	fn ts_netspaceio_gsn_nodes_by_type_p() {

		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let v = nsio.gsn_nodes_by_type(NodeRole::Hub);
		assert_eq!(1, v.len());
		assert_eq!(NodeRole::Hub, v[0].role());
	}

	#[test]
	fn ts_netspaceio_gsn_nodes_by_type_f() {

		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let v = nsio.gsn_nodes_by_type(NodeRole::Undefined);
		assert_eq!(0, v.len());
	}
	
	#[test]
	fn ts_netspaceio_gsn_nodes_by_state_p() {

		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let v = nsio.gsn_nodes_by_state(NodeState::Enabled);
		assert_eq!(2, v.len());
		assert_eq!(NodeState::Enabled, v[0].state());
	}

	#[test]
	fn ts_netspaceio_gsn_nodes_by_state_f() {

		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let v = nsio.gsn_nodes_by_state(NodeState::Unresponsive);
		assert_eq!(0, v.len());
	}

	#[test]
	fn ts_netspaceio_gsn_nodes_by_springname_p() {

		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let r = nsio.gsn_node_by_springname("esusx");
		assert!(r.is_ok());
		assert_eq!("esusx", r.unwrap().springname());
	}

	#[test]
	fn ts_netspaceio_gsn_nodes_by_springname_f() {

		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let r = nsio.gsn_node_by_springname("morrowind");
		assert!(r.is_err());
	}
	
	#[test]
	fn ts_netspaceio_gsn_nodes_by_hostname_p() {

		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let r = nsio.gsn_node_by_hostname("greenman.zu");
		assert!(r.is_ok());
		assert_eq!("greenman.zu", r.unwrap().hostname());
	}

	#[test]
	fn ts_netspaceio_gsn_nodes_by_hostname_f() {

		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let r = nsio.gsn_node_by_hostname("morrowind");
		assert!(r.is_err());
	}
	
	#[test]
	fn ts_netspaceio_gsn_node_by_register_p() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let r = nsio.gsn_node_register((& Node::from_str("spring,host,192.172.1.1").unwrap()));
		assert!(r.is_ok());
		
		let r2 = nsio.gsn_node_by_springname("spring");
		assert!(r2.is_ok());
		let node = r2.unwrap();
		
		assert_eq!("host", node.hostname());
	}
	
	#[test]
	fn ts_netspaceio_gsn_node_by_register_f() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let r = nsio.gsn_node_register((& Node::from_str("esusx,host,192.172.1.1").unwrap()));
		assert!(r.is_err());
		
		let e = r.unwrap_err();
		assert_eq!(NetspaceFailure::DuplicateNode, e);
		
	}

	#[test]
	fn ts_netspaceio_gsn_node_by_unregister_p() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let r = nsio.gsn_node_unregister((& Node::from_str("cci,host,192.172.1.1").unwrap()));
		assert!(r.is_ok());
		
		let r2 = nsio.gsn_node_by_springname("cci");
		assert!(r2.is_err());
	}

	#[test]
	fn ts_netspaceio_gsn_node_by_unregister_f() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let r = nsio.gsn_node_unregister((& Node::from_str("nonname,host,192.172.1.1").unwrap()));
		assert!(r.is_err());
		assert_eq!(NetspaceFailure::NodeNotFound, r.unwrap_err());
	}
	
	#[test]
	fn ts_netspaceio_gsn_node_update_state_p() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let mut n = Node::from_str("cci").unwrap();
		n.update_state(NodeState::Unresponsive);
		let r = nsio.gsn_node_update_state(&n);
		assert!(r.is_ok());
		
		let node = nsio.gsn_node_by_springname("cci").unwrap();
		assert_eq!(NodeState::Unresponsive, node.state());
	}
	
	#[test]
	fn ts_netspaceio_gsn_node_update_state_f() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let mut n = Node::from_str("ccid").unwrap();
		n.update_state(NodeState::Unresponsive);
		let r = nsio.gsn_node_update_state(&n);
		assert!(r.is_err());
		assert_eq!(NetspaceFailure::NodeNotFound, r.unwrap_err());
	}

	#[test]
	fn ts_netspaceio_gsn_node_update_role_p() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let mut n = Node::from_str("cci").unwrap();
		n.update_role(NodeRole::Undefined);
		let r = nsio.gsn_node_update_role(&n);
		assert!(r.is_ok());
		
		let node = nsio.gsn_node_by_springname("cci").unwrap();
		assert_eq!(NodeRole::Undefined, node.role());
	}
	
	#[test]
	fn ts_netspaceio_gsn_node_update_role_f() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let mut n : Node = Node::from_str("ccid").unwrap();
		
		n.update_role(NodeRole::Undefined);
		let r = nsio.gsn_node_update_role(&n);
		assert!(r.is_err());
		assert_eq!(NetspaceFailure::NodeNotFound, r.unwrap_err());
	}

	#[test]
	fn ts_netspaceio_gsn_node_update_service_p() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let mut n = Node::from_str("cci").unwrap();
		n.update_service(NodeService::Undefined);
		let r = nsio.gsn_node_update_service(&n);
		assert!(r.is_ok());
		
		let node = nsio.gsn_node_by_springname("cci").unwrap();
		assert_eq!(NodeService::Undefined, node.service());
	}
	
	#[test]
	fn ts_netspaceio_gsn_node_update_service_f() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let mut n = Node::from_str("ccid").unwrap();
		n.update_service(NodeService::Undefined);
		let r = nsio.gsn_node_update_service(&n);
		assert!(r.is_err());
		assert_eq!(NetspaceFailure::NodeNotFound, r.unwrap_err());
	}
	
	
	#[test]
	fn ts_netspaceio_gtn_geosub_root_nodes_p() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let v = nsio.gtn_geosub_root_nodes("esusx");
		assert_eq!(2, v.len());
		
		assert_eq!("springb", v[0].springname());
		assert_eq!("192.168.1.3", v[0].address());
		assert_eq!(NodeService::Http, v[0].service());
		
		assert_eq!("greenman", v[1].hostname());
		assert_eq!("192.168.1.2", v[1].address());
		assert_eq!(NodeService::Dvsp, v[1].service());
	}
	
	#[test]
	fn ts_netspaceio_gtn_geosub_root_nodes_f() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let v = nsio.gtn_geosub_root_nodes("void");
		assert_eq!(0, v.len());
	}
	
	#[test]
	fn ts_netspaceio_gtn_geosub_node_by_springname_p() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let r = nsio.gtn_geosub_node_by_springname("springb", "esusx");
		
		assert!(r.is_ok());
		
		
		assert_eq!("springb", r.unwrap().springname());
	}

	#[test]
	fn ts_netspaceio_gtn_geosub_node_by_springname_f() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let r = nsio.gtn_geosub_node_by_springname("springc", "esusx");
		
		assert!(r.is_err());
	}
	
	#[test]
	fn ts_netspaceio_gtn_geosub_register_node_p() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let mut node = Node::from_str("springz,hostz,192.168.172.1").unwrap();
		node.update_service(NodeService::Dvsp);
		assert!(nsio.gtn_geosub_register_node(&node, "testnet").is_ok());
		
		let r = nsio.gtn_geosub_node_by_springname("springz", "testnet");
		assert!(r.is_ok());
		
		let n = r.unwrap();
		
		assert_eq!("springz", n.springname());
		assert_eq!("192.168.172.1", n.address());
		assert_eq!(NodeService::Dvsp, n.service());		
		
		
	}
	
	#[test]
	fn ts_netspaceio_gtn_geosub_register_node_f() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let node = Node::from_str("springb,hostz,192.168.172.1").unwrap();
		
		assert!(nsio.gtn_geosub_register_node(&node, "esusx").is_err());
	}
	
	#[test]
	fn ts_netspaceio_gtn_geosub_unregister_node_p() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let node = Node::from_str("springa").unwrap();
		let r = nsio.gtn_geosub_unregister_node(&node, "esusx");
		
		assert!(r.is_ok());
		
		assert!(nsio.gtn_geosub_node_by_springname("springc", "esusx").is_err());
	}
	#[test]
	fn ts_netspaceio_gtn_geosub_unregister_node_f() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let node = Node::from_str("springc").unwrap();
		assert!(nsio.gtn_geosub_unregister_node(&node, "esusx").is_err());
		assert!(nsio.gtn_geosub_unregister_node(&node, "esusxs").is_err());

	}
		#[test]
	fn ts_netspace_routine_is_registered_p() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let n = Node::from_str("cci").unwrap();
		
		assert!(netspace_routine_is_registered(&n, &nsio));
		
	}
	
	#[test]
	fn ts_netspace_routine_is_registered_f() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let n = Node::from_str("ccid,dvsnode.greenman.zus,192.168.1.2").unwrap();
		assert_eq!(false, netspace_routine_is_registered(&n, &nsio));
				
	}
	
	#[test]
	fn ts_netspace_routine_check_token_p() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		assert!(nsio.gsn_check_token("3858f62230ac3c915f300c664312c63f"));		
	}

	#[test]
	fn ts_netspace_routine_check_token_f() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		assert!(nsio.gsn_check_token("3858f62230ac3c915f300c66432c63f1") == false);		
	}

	#[test]
	fn ts_netspace_routine_is_address_gsn_root_p() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		assert!(netspace_routine_is_address_gsn_root("192.168.1.2", "esusx", &nsio));
	}

	#[test]
	fn ts_netspace_routine_is_address_gsn_root_f() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let addr = "192.168.1.8";
		
		assert_eq!(false, netspace_routine_is_address_gsn_root(&addr, "esusx", &nsio));
		assert_eq!(false, netspace_routine_is_address_gsn_root(&addr, "esusxs", &nsio));
	}
	
	#[test]
	fn ts_netspaceio_gtn_geosubs_p() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		Node::from_str("springa").unwrap();
		
		assert_eq!(nsio.gtn_geosubs().len(), 1);
		assert_eq!(nsio.gtn_geosubs()[0], "esusx");
	}
	
}