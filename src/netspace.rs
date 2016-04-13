
extern crate sqlite;


pub use spring_dvs::enums::{DvspNodeType,DvspNodeState,DvspService,Failure,Success};
use spring_dvs::protocol::{Ipv4,NodeTypeField, u8_service_type, u8_status_type};
use spring_dvs::formats::{ipv4_to_str_address,str_address_to_ipv4};

pub use spring_dvs::model::{Netspace,Node};

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
	
	
	fn fill_node(&self, statement: &sqlite::Statement) -> Result<Node,Failure> {
		let spring = statement.read::<String>(1).unwrap();
		let host = statement.read::<String>(2).unwrap();
		let addr = try!(str_address_to_ipv4(&statement.read::<String>(3).unwrap()));
		let service = match u8_service_type(statement.read::<i64>(4).unwrap() as u8) {
				Some(op) => op,
				None => return Err(Failure::InvalidBytes)
			};

		
		let status =  match u8_status_type(statement.read::<i64>(5).unwrap() as u8) {
				Some(op) => op,
				None => return Err(Failure::InvalidBytes)
			};
		
		let types =  statement.read::<i64>(6).unwrap() as u8;
		
		Ok(Node::new(spring, host, addr, service, status, types))
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
	
	fn node_from_statement(&self, statement: &mut Statement) -> Result<Node,Failure> {

		match statement.next() {
			Ok(state) => match state {
				
							State::Row => self.fill_node(&statement),
			 				_ => Err(Failure::InvalidArgument)
			 				
						},

			_ => Err(Failure::InvalidArgument)

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
	
	fn gsn_nodes_by_address(&self, address: Ipv4) -> Vec<Node> {
		
		let mut statement = self.db.prepare("
    	SELECT * FROM geosub_netspace WHERE address = ?
		").unwrap();

		statement.bind(1, &sqlite::Value::String( ipv4_to_str_address(&address) ) ).unwrap();
		
		self.vector_from_statement(&mut statement)
		
	}

	
	fn gsn_nodes_by_type(&self, types: NodeTypeField) -> Vec<Node> {
		
		let mut statement = self.db.prepare("
    	SELECT * FROM geosub_netspace WHERE types & ?
		").unwrap();

		statement.bind(1, &sqlite::Value::Integer( types as i64 ) ).unwrap();
		
		self.vector_from_statement(&mut statement)
	}

	fn gsn_nodes_by_state(&self, state: DvspNodeState) -> Vec<Node> {
		
		let mut statement = self.db.prepare("
    	SELECT * FROM geosub_netspace WHERE status = ?
		").unwrap();

		statement.bind(1, &sqlite::Value::Integer( state as i64 ) ).unwrap();
		
		self.vector_from_statement(&mut statement)
	}
	
	fn gsn_node_by_springname(&self, name: &str) -> Result<Node, Failure> {
		
		let mut statement = self.db.prepare("
    	SELECT * FROM geosub_netspace WHERE springname = ?
		").unwrap();

		statement.bind(1, &sqlite::Value::String( String::from(name) ) ).unwrap();
		
		self.node_from_statement(&mut statement)
	}
	
	fn gsn_node_by_hostname(&self, name: &str) -> Result<Node, Failure> {
		
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
		let v: Vec<String> = Vec::new();
		
		v
	}
	
	
	fn gsn_node_register(&self, node: &Node) -> Result<Success,Failure> {
		
		if self.gsn_node_by_springname(node.springname()).is_ok() {
			return Err(Failure::Duplicate)
		}
		
		let mut statement = self.db.prepare(
						"INSERT INTO 
						`geosub_netspace` 
						(springname,hostname,address,service,status,types) 
						VALUES (?,?,?,?,?,?)").unwrap();
		statement.bind(1, &sqlite::Value::String( String::from(node.springname()) ) ).unwrap();
		statement.bind(2, &sqlite::Value::String( String::from(node.hostname()) ) ).unwrap();
		statement.bind(3, &sqlite::Value::String( ipv4_to_str_address(&node.address() ) ) ).unwrap();
		statement.bind(4, &sqlite::Value::Integer( node.service() as i64 ) ).unwrap();
		statement.bind(5, &sqlite::Value::Integer( node.state() as i64 ) ).unwrap();
		statement.bind(6, &sqlite::Value::Integer( node.types() as i64 ) ).unwrap();
		match statement.next() {
			Ok(_) => Ok(Success::Ok),
			Err(_) => Err(Failure::InvalidArgument)   
		}
		
	}

	fn gsn_node_unregister(&self, node: &Node) -> Result<Success,Failure> {
		if self.gsn_node_by_springname(node.springname()).is_err() {
			return Err(Failure::InvalidArgument)
		}
		
		let mut statement = self.db.prepare(
						"DELETE FROM 
						`geosub_netspace` WHERE 
						springname = ?").unwrap();
		statement.bind(1, &sqlite::Value::String( String::from(node.springname()) ) ).unwrap();
		
		match statement.next() {
			Ok(_) => Ok(Success::Ok),
			Err(_) => Err(Failure::InvalidArgument)   
		}		
	}

	fn gsn_node_update_state(&self, node: &Node) -> Result<Success,Failure> {
	
		if self.gsn_node_by_springname(node.springname()).is_err() {
			return Err(Failure::InvalidArgument)
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
			Err(_) => Err(Failure::InvalidArgument)   
		}
	}
	
	fn gsn_node_update_types(&self, node: &Node) -> Result<Success,Failure> {
	
		if self.gsn_node_by_springname(node.springname()).is_err() {
			return Err(Failure::InvalidArgument)
		}
		
		let mut statement = self.db.prepare(
						"UPDATE  
						`geosub_netspace`
						SET types = ?
						WHERE springname = ?").unwrap();
		
		statement.bind(1, &sqlite::Value::Integer( node.types() as i64 ) ).unwrap();
		statement.bind(2, &sqlite::Value::String( String::from(node.springname()) ) ).unwrap();
		
		match statement.next() {
			Ok(_) => Ok(Success::Ok),
			Err(_) => Err(Failure::InvalidArgument)   
		}
	}

	fn gsn_node_update_service(&self, node: &Node) -> Result<Success,Failure> {
	
		if self.gsn_node_by_springname(node.springname()).is_err() {
			return Err(Failure::InvalidArgument)
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
			Err(_) => Err(Failure::InvalidArgument)   
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
	
	
	fn gtn_geosub_node_by_springname(&self, name: &str, gsn: &str) -> Result<Node,Failure> {
		let mut statement = self.db.prepare("
    	SELECT * FROM geotop_netspace 
    	WHERE springname = ?
    	AND geosub = ?
		").unwrap();

		statement.bind(1, &sqlite::Value::String( String::from(name) ) ).unwrap();
		statement.bind(2, &sqlite::Value::String( String::from(gsn) ) ).unwrap();
		
		self.node_from_statement(&mut statement)
	}

	fn gtn_geosub_register_node(&self, node: &Node, gsn: &str) -> Result<Success,Failure> {
		if self.gtn_geosub_node_by_springname(&node.springname(), &gsn).is_ok() {
			return Err(Failure::Duplicate)
		}
		
		let mut statement = self.db.prepare(
						"INSERT INTO 
						`geotop_netspace` 
						(springname,hostname,address,service,priority, geosub) 
						VALUES (?,?,?,?,?,?)").unwrap();
		statement.bind(1, &sqlite::Value::String( String::from(node.springname()) ) ).unwrap();
		statement.bind(2, &sqlite::Value::String( String::from(node.hostname()) ) ).unwrap();
		statement.bind(3, &sqlite::Value::String( ipv4_to_str_address(&node.address() ) ) ).unwrap();
		statement.bind(4, &sqlite::Value::Integer( node.service() as i64 ) ).unwrap();
		statement.bind(5, &sqlite::Value::Integer( 1 as i64)).unwrap();
		statement.bind(6, &sqlite::Value::String( String::from(gsn) )).unwrap();
			
		match statement.next() {
			Ok(_) => Ok(Success::Ok),
			Err(_) => Err(Failure::InvalidArgument)   
		}	
	}

	fn gtn_geosub_unregister_node(&self, node: &Node, gsn: &str) -> Result<Success,Failure> {

		if self.gtn_geosub_node_by_springname(&node.springname(), &gsn).is_err() {
			return Err(Failure::InvalidArgument)
		}

		let mut statement = self.db.prepare(
						"DELETE FROM `geotop_netspace` 
						WHERE springname = ?
						AND geosub = ?").unwrap();

		statement.bind(1, &sqlite::Value::String( String::from(node.springname()) ) ).unwrap();
		statement.bind(2, &sqlite::Value::String( String::from(gsn) ) ).unwrap();

		match statement.next() {
			Ok(_) => Ok(Success::Ok),
			Err(_) => Err(Failure::InvalidArgument)   
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

		INSERT INTO `geosub_netspace` (id,springname,hostname,address,service,status,types) VALUES (1,'esusx','greenman.zu','192.168.1.1',1,1,1);
		INSERT INTO `geosub_netspace` (id,springname,hostname,address,service,status,types) VALUES (2,'cci','dvsnode.greenman.zu','192.168.1.2',2,1,2);
		INSERT INTO `geotop_netspace` (id,springname,hostname,address,service,priority,geosub) VALUES (1,'springA', 'greenman', '192.168.1.2', 1, 2, 'esusx');
		INSERT INTO `geotop_netspace` (id,springname,hostname,address,service,priority,geosub) VALUES (2,'springB', 'blueman', '192.168.1.3', 2, 1, 'esusx');
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
		let v = nsio.gsn_nodes_by_address([192,168,1,1]);
		assert_eq!(1, v.len());
		assert_eq!([192,168,1,1], v[0].address());
	}

	#[test]
	fn ts_netspaceio_gsn_nodes_by_address_f() {

		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let v = nsio.gsn_nodes_by_address([192,168,1,3]);
		assert_eq!(0, v.len());
	}
	
	#[test]
	fn ts_netspaceio_gsn_nodes_by_type_p() {

		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let v = nsio.gsn_nodes_by_type(DvspNodeType::Root as u8);
		assert_eq!(1, v.len());
		assert_eq!(DvspNodeType::Root as u8, v[0].types());
	}

	#[test]
	fn ts_netspaceio_gsn_nodes_by_type_f() {

		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let v = nsio.gsn_nodes_by_type(DvspNodeType::Undefined as u8);
		assert_eq!(0, v.len());
	}
	
	#[test]
	fn ts_netspaceio_gsn_nodes_by_state_p() {

		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let v = nsio.gsn_nodes_by_state(DvspNodeState::Enabled);
		assert_eq!(2, v.len());
		assert_eq!(DvspNodeState::Enabled, v[0].state());
	}

	#[test]
	fn ts_netspaceio_gsn_nodes_by_state_f() {

		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let v = nsio.gsn_nodes_by_state(DvspNodeState::Unresponsive);
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
		let r = nsio.gsn_node_register((& Node::from_node_string("spring,host,192.172.1.1").unwrap()));
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
		let r = nsio.gsn_node_register((& Node::from_node_string("esusx,host,192.172.1.1").unwrap()));
		assert!(r.is_err());
		
		let e = r.unwrap_err();
		assert_eq!(Failure::Duplicate, e);
		
	}

	#[test]
	fn ts_netspaceio_gsn_node_by_unregister_p() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let r = nsio.gsn_node_unregister((& Node::from_node_string("cci,host,192.172.1.1").unwrap()));
		assert!(r.is_ok());
		
		let r2 = nsio.gsn_node_by_springname("cci");
		assert!(r2.is_err());
	}

	#[test]
	fn ts_netspaceio_gsn_node_by_unregister_f() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let r = nsio.gsn_node_unregister((& Node::from_node_string("nonname,host,192.172.1.1").unwrap()));
		assert!(r.is_err());
		assert_eq!(Failure::InvalidArgument, r.unwrap_err());
	}
	
	#[test]
	fn ts_netspaceio_gsn_node_update_state_p() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let mut n = Node::from_springname("cci").unwrap();
		n.update_state(DvspNodeState::Unresponsive);
		let r = nsio.gsn_node_update_state(&n);
		assert!(r.is_ok());
		
		let node = nsio.gsn_node_by_springname("cci").unwrap();
		assert_eq!(DvspNodeState::Unresponsive, node.state());
	}
	
	#[test]
	fn ts_netspaceio_gsn_node_update_state_f() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let mut n = Node::from_springname("ccid").unwrap();
		n.update_state(DvspNodeState::Unresponsive);
		let r = nsio.gsn_node_update_state(&n);
		assert!(r.is_err());
		assert_eq!(Failure::InvalidArgument, r.unwrap_err());
	}

	#[test]
	fn ts_netspaceio_gsn_node_update_types_p() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let mut n = Node::from_springname("cci").unwrap();
		n.update_types(DvspNodeType::Undefined as u8);
		let r = nsio.gsn_node_update_types(&n);
		assert!(r.is_ok());
		
		let node = nsio.gsn_node_by_springname("cci").unwrap();
		assert_eq!(DvspNodeType::Undefined as u8, node.types());
	}
	
	#[test]
	fn ts_netspaceio_gsn_node_update_types_f() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let mut n = Node::from_springname("ccid").unwrap();
		n.update_types(DvspNodeType::Undefined as u8);
		let r = nsio.gsn_node_update_types(&n);
		assert!(r.is_err());
		assert_eq!(Failure::InvalidArgument, r.unwrap_err());
	}

	#[test]
	fn ts_netspaceio_gsn_node_update_service_p() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let mut n = Node::from_springname("cci").unwrap();
		n.update_service(DvspService::Undefined);
		let r = nsio.gsn_node_update_service(&n);
		assert!(r.is_ok());
		
		let node = nsio.gsn_node_by_springname("cci").unwrap();
		assert_eq!(DvspService::Undefined, node.service());
	}
	
	#[test]
	fn ts_netspaceio_gsn_node_update_service_f() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let mut n = Node::from_springname("ccid").unwrap();
		n.update_service(DvspService::Undefined);
		let r = nsio.gsn_node_update_service(&n);
		assert!(r.is_err());
		assert_eq!(Failure::InvalidArgument, r.unwrap_err());
	}
	
	
	#[test]
	fn ts_netspaceio_gtn_geosub_root_nodes_p() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let v = nsio.gtn_geosub_root_nodes("esusx");
		assert_eq!(2, v.len());
		
		assert_eq!("springB", v[0].springname());
		assert_eq!([192,168,1,3], v[0].address());
		assert_eq!(DvspService::Http, v[0].service());
		
		assert_eq!("greenman", v[1].hostname());
		assert_eq!([192,168,1,2], v[1].address());
		assert_eq!(DvspService::Dvsp, v[1].service());
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
		
		let r = nsio.gtn_geosub_node_by_springname("springB", "esusx");
		
		assert!(r.is_ok());
		
		
		assert_eq!("springB", r.unwrap().springname());
	}

	#[test]
	fn ts_netspaceio_gtn_geosub_node_by_springname_f() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let r = nsio.gtn_geosub_node_by_springname("springC", "esusx");
		
		assert!(r.is_err());
	}
	
	#[test]
	fn ts_netspaceio_gtn_geosub_register_node_p() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let mut node = Node::from_node_string("springZ,hostZ,192.168.172.1").unwrap();
		node.update_service(DvspService::Dvsp);
		assert!(nsio.gtn_geosub_register_node(&node, "testnet").is_ok());
		
		let r = nsio.gtn_geosub_node_by_springname("springZ", "testnet");
		assert!(r.is_ok());
		
		let n = r.unwrap();
		
		assert_eq!("springZ", n.springname());
		assert_eq!([192,168,172,1], n.address());
		assert_eq!(DvspService::Dvsp, n.service());		
		
		
	}
	
	#[test]
	fn ts_netspaceio_gtn_geosub_register_node_f() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let node = Node::from_node_string("springB,hostZ,192.168.172.1").unwrap();
		assert!(nsio.gtn_geosub_register_node(&node, "esusx").is_err());
	}
	
	#[test]
	fn ts_netspaceio_gtn_geosub_unregister_node_p() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let node = Node::from_springname("springA").unwrap();
		let r = nsio.gtn_geosub_unregister_node(&node, "esusx");
		
		assert!(r.is_ok());
		
		assert!(nsio.gtn_geosub_node_by_springname("springC", "esusx").is_err());
	}
	#[test]
	fn ts_netspaceio_gtn_geosub_unregister_node_f() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		let node = Node::from_springname("springC").unwrap();
		assert!(nsio.gtn_geosub_unregister_node(&node, "esusx").is_err());
		assert!(nsio.gtn_geosub_unregister_node(&node, "esusxs").is_err());

	}
		#[test]
	fn ts_netspace_routine_is_registered_p() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let n = Node::from_springname("cci").unwrap();
		
		assert!(netspace_routine_is_registered(&n, &nsio));
		
	}
	
	#[test]
	fn ts_netspace_routine_is_registered_f() {
		let nsio = NetspaceIo::new(":memory:");
		setup_netspace(nsio.db());
		
		let n = Node::from_node_string("ccid,dvsnode.greenman.zus,192.168.1.2").unwrap();
		assert_eq!(false, netspace_routine_is_registered(&n, &nsio));
				
	}
}