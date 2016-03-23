extern crate spring_dvs;
extern crate sqlite;



use self::sqlite::{State,Statement};

pub use self::spring_dvs::model::{Netspace,Node};
pub use self::spring_dvs::enums::{Failure,DvspNodeState,DvspNodeType};
use self::spring_dvs::protocol::{Ipv4,NodeTypeField, u8_service_type, u8_status_type};
use self::spring_dvs::formats::{ipv4_to_str_address,str_address_to_ipv4};


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
							State::Row => Ok(self.fill_node(&statement).unwrap()),
			 				_ => Err(Failure::InvalidArgument)
						},

			_ => Err(Failure::InvalidArgument)

		}
		
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

		statement.bind(1, &sqlite::Value::String( ipv4_to_str_address(address) ) ).unwrap();
		
		self.vector_from_statement(&mut statement)
		
	}

	fn gsn_nodes_by_type(&self, types: NodeTypeField) -> Vec<Node> {
		
		let mut statement = self.db.prepare("
    	SELECT * FROM geosub_netspace WHERE types = ?
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
		INSERT INTO `geosub_netspace` (id,springname,hostname,address,service,status,types) VALUES (1,'esusx','greenman.zu','192.168.1.1',1,1,1);
		INSERT INTO `geosub_netspace` (id,springname,hostname,address,service,status,types) VALUES (2,'cci','dvsnode.greenman.zu','192.168.1.2',2,1,2);
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
}