
extern crate unix_socket;

use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;

use std::io::prelude::*;
use std::mem;
use std::str::Split;


use self::unix_socket::UnixStream;


use netspace::*;


fn binary_split(msg: &str) -> Vec<&str> {
	msg.splitn(2, " ").collect()
}

#[macro_export]
macro_rules! cascade_none_nowrap {
	($opt: expr) => (
		match $opt {
			Some(s) => s,
			_ => return None,
		}
	)
}

#[macro_export]
macro_rules! extract_zone_network {
	($e: expr) => (
		match $e {
			ManagementZone::Network(s) => s,
			e => panic!("extract_zone_network -- Unexpected value: {:?}", e) 
		}
	)
}

#[macro_export]
macro_rules! extract_zone_validation {
	($e: expr) => (
		match $e {
			ManagementZone::Validation(s) => s,
			e => panic!("extract_zone_validation -- Unexpected value: {:?}", e) 
		}
	)
}

pub fn management_handler(mut stream: UnixStream, config: Config) {
	
	let nio = match config.live_test {
		false => {
			NetspaceIo::new("/var/lib/springdvs/gsn.db") 
		},
		true => {
			NetspaceIo::new("live-testing.db")
		}
	};

	let mut szin_buf = [0;4];
	
	stream.read_exact(&mut szin_buf).unwrap();
	
	let szin : u32 = unsafe { mem::transmute(szin_buf) };
	
	let mut bufin : Vec<u8> = Vec::new();
	bufin.resize(szin as usize, b'\0');
	stream.read_exact(bufin.as_mut_slice()).unwrap();
	let command = String::from_utf8(bufin).unwrap();
	
	let mi = ManagementInstance::new();
	
	let out = match mi.run(&command, &nio) {
		Some(s) => s,
		None => "Error: Unrecognised or malformed command".to_string() 
	};
	stream.write_all(out.as_bytes()).unwrap();
}

struct ManagementInstance;

impl ManagementInstance {
	pub fn new() -> Self {
		ManagementInstance
	}
	pub fn run(&self, command: &str, nio: &NetspaceIo) -> Option<String> {
		self.process_request(cascade_none_nowrap!(ManagementZone::from_str(command)), nio)
	}

	pub fn process_request(&self, request: ManagementZone, nio: &NetspaceIo) -> Option<String> {
		match request {
			ManagementZone::Network(nz) => self.process_network(nz, nio),
			ManagementZone::Validation(vz) => self.process_validation(vz, nio)
		}
	}
	
	fn process_network(&self, nz: NetworkZone, nio: &NetspaceIo) -> Option<String> {
		match nz.action {
			NetworkAction::View => NetworkZoneModel::view(nz.op1, nio),
			NetworkAction::Update => NetworkZoneModel::update(nz.op1, nz.op2, nio),
			NetworkAction::Remove => NetworkZoneModel::remove(nz.op1, nio),
		}
	}
	
	fn process_validation(&self, vz: ValidationZone, nio: &NetspaceIo) -> Option<String> {
		match vz.action {
			ValidationAction::View => ValidationZoneModel::view(vz.op1, nio),
			ValidationAction::Add => ValidationZoneModel::add(vz.op1, vz.op2, nio),
			ValidationAction::Remove => ValidationZoneModel::remove(vz.op1, nio),
		}
	}
}

#[derive(Clone, PartialEq, Debug)]
pub enum ManagementZone {
	Network(NetworkZone), Validation(ValidationZone)
}

impl ManagementZone {
	pub fn from_str(msg: &str) -> Option<ManagementZone> {
		if msg.len() == 0 { return None; }
		
		let atom = binary_split(msg);
		
		Some(match atom[0] {
			"network" => {
				ManagementZone::Network(cascade_none_nowrap!(NetworkZone::from_str(atom[1])))				
			},
			"validation" => {
				ManagementZone::Validation(cascade_none_nowrap!(ValidationZone::from_str(atom[1])))
			}
			_ => return None
		})
		
	}
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum NetworkAction {
	View,
	Remove,
	Update,
}

#[derive(Clone, PartialEq, Debug)]
pub enum NetworkOperand {
	None,
	All,
	Node(String),
	Role(NodeRole),
	State(NodeState),
	Service(NodeService),
	Host(String),
	Address(String),
}

#[derive(Clone, PartialEq, Debug)]
pub struct NetworkZone {
	action: NetworkAction,
	op1: NetworkOperand,
	op2: NetworkOperand,
}

impl NetworkZone {
	pub fn new(action: NetworkAction, op1: NetworkOperand, op2: NetworkOperand) -> NetworkZone {
		NetworkZone {
			action: action,
			op1: op1,
			op2: op2
		}
	}

	pub fn from_str(msg: &str) -> Option<NetworkZone> {
		if msg.len() == 0 { return None; }
		
		let mut atom = msg.split(" ");
		
		let action = match atom.next() {
			Some("view") => NetworkAction::View,
			Some("remove") => NetworkAction::Remove,
			Some("update") => NetworkAction::Update,
			_ => return None,
		};

		let op1 = match cascade_none_nowrap!(NetworkZone::extract_operand(&mut atom)) {
			NetworkOperand::None => return None,
			op => op
		};
		
		let op2 = cascade_none_nowrap!(NetworkZone::extract_operand(&mut atom));
		Some(NetworkZone::new(action, op1, op2))
	}
	
	fn extract_operand(atom: &mut Split<&str>) -> Option<NetworkOperand> {
		
		Some(match atom.next() {
			Some("all") =>
						NetworkOperand::All,

			Some("node") =>
						NetworkOperand::Node(
							String::from(
								cascade_none_nowrap!(atom.next())
							)
						),

			Some("springname") =>
						NetworkOperand::Node(
							String::from(
								cascade_none_nowrap!(atom.next())
							)
						),

			Some("role") =>
						NetworkOperand::Role(
							cascade_none_nowrap!(
								NodeRole::from_str(
									cascade_none_nowrap!(atom.next())
								)
							)
						),

			Some("state") =>
						NetworkOperand::State(
							cascade_none_nowrap!(
								NodeState::from_str(
									cascade_none_nowrap!(atom.next())
								)
							)
						),

			Some("service") =>
						NetworkOperand::Service(
							cascade_none_nowrap!(
								NodeService::from_str(
									cascade_none_nowrap!(atom.next())
								)
							)
						),

			Some("hostname") =>
						 NetworkOperand::Host(
							String::from(
								cascade_none_nowrap!(atom.next())
							)
						),
			
			Some("address") =>
						NetworkOperand::Address(
							String::from(
								cascade_none_nowrap!(atom.next())
							)
						),

			_ => NetworkOperand::None,
		})
	}
}

struct NetworkZoneModel;
	
impl NetworkZoneModel {
	pub fn view(op: NetworkOperand, nio: &NetspaceIo) -> Option<String> {
		match op {
			NetworkOperand::All =>
				Some( Self::tabulate_nodes(&nio.gsn_nodes()) ),

			NetworkOperand::Node(s) =>
				Some( Self::tabulate_node(nio.gsn_node_by_springname(&s)) ),
				
			NetworkOperand::Host(s) =>
				Some( Self::tabulate_node(nio.gsn_node_by_hostname(&s)) ),

			NetworkOperand::Role(r) =>
				Some( Self::tabulate_nodes(&nio.gsn_nodes_by_type(r)) ),
				
			NetworkOperand::State(s) =>
				Some( Self::tabulate_nodes(&nio.gsn_nodes_by_state(s)) ),
				
			NetworkOperand::Address(a) =>
				Some( Self::tabulate_nodes(&nio.gsn_nodes_by_address(&a)) ),

			_ => None
		}
		
	}
	
	pub fn update(target: NetworkOperand, value: NetworkOperand, nio: &NetspaceIo) -> Option<String> {
		let mut v : Vec<String> = Vec::new();
		
		match target {
			NetworkOperand::All => {
				for node in nio.gsn_nodes() {
					v.push(Self::update_node(Ok(node), value.clone(), nio)) 
				}
			},
			
			NetworkOperand::Node(s) => {
				let node = nio.gsn_node_by_springname(&s);
				v.push(Self::update_node(node, value.clone(), nio))
			},

			NetworkOperand::Role(r) => {
				for node in nio.gsn_nodes_by_type(r) {
					v.push(Self::update_node(Ok(node), value.clone(), nio)) 
				}
			},

			NetworkOperand::State(s) => {
				for node in nio.gsn_nodes_by_state(s) {
					v.push(Self::update_node(Ok(node), value.clone(), nio)) 
				}
			},

			NetworkOperand::Address(a) => {
				for node in nio.gsn_nodes_by_address(&a) {
					v.push(Self::update_node(Ok(node), value.clone(), nio)) 
				}
			},

			_ => return None
		}
		
		Some(format!("{}\n",v.join("\n")))
	}
	
	fn update_node(node_result: Result<Node, NetspaceFailure>, value: NetworkOperand, nio: &NetspaceIo ) -> String {
		
		let mut node = match node_result {
			Ok(n) => n,
			Err(e) => return format!("Error requesting node {:?}", e)
		};

		match value {
			NetworkOperand::Role(r) => {
				let old = node.role(); 
				node.update_role(r);
				nio.gsn_node_update_role(&node).unwrap();
				format!("Updated {} role: {} -> {}", node.springname(), old, r)
			},
			
			NetworkOperand::State(s) => {
				let old = node.state(); 
				node.update_state(s);
				nio.gsn_node_update_state(&node).unwrap();
				format!("Updated {} state: {} -> {}", node.springname(), old, s)
			},

			NetworkOperand::Service(s) => {
				let old = node.service(); 
				node.update_service(s);
				nio.gsn_node_update_service(&node).unwrap();
				format!("Updated {} service: {} -> {}", node.springname(), old, s)
			},

			NetworkOperand::Host(s) => {
				let old = node.hostname().to_string(); 
				node.update_hostname(&s);
				nio.gsn_node_update_hostname(&node).unwrap();
				format!("Updated {} hostname: {} -> {}", node.springname(), old, s)
			},

			NetworkOperand::Address(s) => {
				let old = node.address().to_string(); 
				node.update_address(&s);
				nio.gsn_node_update_address(&node).unwrap();
				format!("Updated {} address: {} -> {}", node.springname(), old, s)
			},
			_ => "Error: Unknown or unsupported value for updating".to_string()
		}
	} 
	
	pub fn remove(op: NetworkOperand, nio: &NetspaceIo) -> Option<String> {
		
		Some(match op {
			NetworkOperand::Node(s) => {
				match nio.gsn_node_by_springname(&s) {
					Ok(n) => {
						nio.gsn_node_unregister(&n).unwrap();
						format!("Removed node {}\n", n.springname())
					},
					Err(e) => format!("Error: unabled to retrieve node ({:?})\n", e)
				}
								
			},
			e => format!("Error: Unknown or unsupported target filter ({:?})\n", e)		
		})
	}
	
	fn tabulate_nodes(nodes: &Vec<Node>) -> String {
		let mut table = Table::new();
		Self::add_headings(&mut table);
		for node in nodes {
			table.add_row(Row::new(vec![
							Cell::new(node.springname()),
							Cell::new(node.hostname()),
							Cell::new(node.address()),
							Cell::new( &format!("{}", node.role()) ),
							Cell::new( &format!("{}", node.state()) ),
							Cell::new( &format!("{}", node.service()) )
							]));
		}
		
		
		format!("{}", table)
	}
	
	fn tabulate_node(node_result: Result<Node, NetspaceFailure>) -> String {
		
		let node = match node_result {
			Ok(n) => n,
			Err(e) => return format!("Error requesting node {:?}", e)
		};

		let mut table = Table::new();
		Self::add_headings(&mut table);
		table.add_row(Row::new(vec![
						Cell::new(node.springname()),
						Cell::new(node.hostname()),
						Cell::new(node.address()),
						Cell::new( &format!("{}", node.role()) ),
						Cell::new( &format!("{}", node.state()) ),
						Cell::new( &format!("{}", node.service()) )
					]));
		
		format!("{}", table)	
	}
	
	fn add_headings(table: &mut Table) {
		table.add_row(row!["_spring_", "_host_",
							"_address_", "_role_", 
							"_state_", "_service_"]);
	}
}

#[derive(Copy,Clone, PartialEq, Debug)]
pub enum ValidationAction {
	View,
	Add,
	Remove
}

#[derive(Clone, PartialEq, Debug)]
pub enum ValidationOperand {
	None,
	All,
	Token(String),
	Node(String)
}

#[derive(Clone, PartialEq, Debug)]
pub struct ValidationZone {
	action: ValidationAction,
	op1: ValidationOperand,
	op2: ValidationOperand
}

impl ValidationZone {
	pub fn new(action: ValidationAction, op1: ValidationOperand, op2: ValidationOperand) -> Self {
		ValidationZone {
			action: action,
			op1: op1,
			op2: op2,
		}
	}

	pub fn from_str(msg: &str) -> Option<ValidationZone> {
		if msg.len() == 0 { return None }
		
		let mut atom = msg.split(" ");

		let action = match atom.next() {
			Some("view") => ValidationAction::View,
			Some("add") => ValidationAction::Add,
			Some("remove") => ValidationAction::Remove,
			_ => return None,
		};
		
		let op1 = match cascade_none_nowrap!(Self::extract_operand(&mut atom)) {
			ValidationOperand::None => return None,
			s => s,
		} ;
		
		let op2 = cascade_none_nowrap!(Self::extract_operand(&mut atom));
		Some(ValidationZone::new(action, op1, op2))
	}
	
	fn extract_operand(atom: &mut Split<&str>) -> Option<ValidationOperand> {
		
		Some(match atom.next() {
			Some("all") =>
						ValidationOperand::All,

			Some("node") =>
						ValidationOperand::Node(
								cascade_none_nowrap!(atom.next()).to_string()
						),
						
			Some("springname") =>
						ValidationOperand::Node(
								cascade_none_nowrap!(atom.next()).to_string()
						),

			Some("token") =>
						ValidationOperand::Token(
								cascade_none_nowrap!(atom.next()).to_string()
						),
						
			_ => ValidationOperand::None
		})
	}
}

struct ValidationZoneModel;


impl ValidationZoneModel {
	pub fn view(op: ValidationOperand, nio: &NetspaceIo) -> Option<String> {
		Some(match op {
			ValidationOperand::All => {
				Self::tabulate_tokens(nio.gsn_tokens())
			},
			ValidationOperand::Node(s) => {
				Self::tabulate_tokens(nio.gsn_token_by_springname(&s))
			}
			e => format!("Error: Unsupported target filter ({:?})", e)
		})
		
	}
	
	pub fn add(op1: ValidationOperand, op2: ValidationOperand, nio: &NetspaceIo) -> Option<String> {
		
		let mut token = "".to_string();
		let mut springname = "".to_string();
		
		match op1 {
			ValidationOperand::Token(s) => token = s,
			ValidationOperand::Node(s) => springname = s,
			e => return Some(format!("Error: Invalid operand ({:?})\n", e)),
		}
		
		match op2 {
			ValidationOperand::Token(s) => token = s,
			ValidationOperand::Node(s) => springname = s,
			e => return Some(format!("Error: Invalid operand ({:?})\n", e)),
		}
		
		if token.len() == 0 || springname.len() == 0 { return None }
		
		nio.gsn_add_token(&token, &springname);
		Some(format!("Added token {} for {}\n", token, springname)) 
	}
	
	pub fn remove(op1: ValidationOperand, nio: &NetspaceIo) -> Option<String> {
		Some(match op1 {
			ValidationOperand::Token(s) => {
				nio.gsn_remove_token(&s);
				format!("Removed token {}\n", s)
			},
			ValidationOperand::Node(s) => {
				nio.gsn_remove_token_by_springname(&s);
				format!("Removed token for {}\n", s)
			},
			
			e => format!("Error: Unsupported target filter ({:?})\n", e)
		})	
	}
	
	fn add_headings(table: &mut Table) {
		table.add_row(row!["_token_", "_spring_"]);
	}
	
	fn tabulate_tokens(tokens: Vec<(String,String)>) -> String {
		let mut table = Table::new();
		Self::add_headings(&mut table);
		for token in tokens {
			table.add_row(Row::new(vec![
							Cell::new(&token.0),
							Cell::new(&token.1)]));
		}
		
		
		format!("{}", table)		
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use netspace::*;
	
	macro_rules! assert_match {
	
		($chk:expr, $pass:pat) => (
			assert!(match $chk {
						$pass => true,
						_ => false
			}))
	}
	
	macro_rules! unwrap_some {
		($chk:expr) => (
			match $chk {
						Some(s) => s,
						_ => panic!("Unwrapping a None")
			})		
	}
	
	#[test]
	fn ts_network_view_all_p() {
		let mz = unwrap_some!(ManagementZone::from_str("network view all"));
		let nz : NetworkZone = extract_zone_network!(mz);
		assert_eq!(nz.action, NetworkAction::View);
		assert_eq!(nz.op1, NetworkOperand::All);
	}
	
	#[test]
	fn ts_network_view_node_p() {
		let mz = unwrap_some!(ManagementZone::from_str("network view node foo"));
		let nz : NetworkZone = extract_zone_network!(mz);
		assert_eq!(nz.action, NetworkAction::View);
		assert_eq!(nz.op1, NetworkOperand::Node(String::from("foo")));
	}
	
	#[test]
	fn ts_network_view_spring_p() {
		let mz = unwrap_some!(ManagementZone::from_str("network view springname foo"));
		let nz : NetworkZone = extract_zone_network!(mz);
		assert_eq!(nz.action, NetworkAction::View);
		assert_eq!(nz.op1, NetworkOperand::Node(String::from("foo")));
	}
	
	#[test]
	fn ts_network_view_role_p() {
		let mz = unwrap_some!(ManagementZone::from_str("network view role hybrid"));
		let nz : NetworkZone = extract_zone_network!(mz);
		assert_eq!(nz.action, NetworkAction::View);
		assert_eq!(nz.op1, NetworkOperand::Role(NodeRole::Hybrid));
	}
	
	#[test]
	fn ts_network_view_state_p() {
		let mz = unwrap_some!(ManagementZone::from_str("network view state unresponsive"));
		let nz : NetworkZone = extract_zone_network!(mz);
		assert_eq!(nz.action, NetworkAction::View);
		assert_eq!(nz.op1, NetworkOperand::State(NodeState::Unresponsive));
	}
	
	#[test]
	fn ts_network_view_service_p() {
		let mz = unwrap_some!(ManagementZone::from_str("network view service http"));
		let nz : NetworkZone = extract_zone_network!(mz);
		assert_eq!(nz.action, NetworkAction::View);
		assert_eq!(nz.op1, NetworkOperand::Service(NodeService::Http));
	}
	
	#[test]
	fn ts_network_view_address_p() {
		let mz = unwrap_some!(ManagementZone::from_str("network view address 127.0.0.1"));
		let nz : NetworkZone = extract_zone_network!(mz);
		assert_eq!(nz.action, NetworkAction::View);
		assert_eq!(nz.op1, NetworkOperand::Address(String::from("127.0.0.1")));
	}
	
	#[test]
	fn ts_network_view_host_p() {
		let mz = unwrap_some!(ManagementZone::from_str("network view hostname foo.bar"));
		let nz : NetworkZone = extract_zone_network!(mz);
		assert_eq!(nz.action, NetworkAction::View);
		assert_eq!(nz.op1, NetworkOperand::Host(String::from("foo.bar")));
	}
	
	#[test]
	fn ts_network_update_node_spring_p() {
		let mz = unwrap_some!(ManagementZone::from_str("network update node foo springname bar"));
		let nz : NetworkZone = extract_zone_network!(mz);
		assert_eq!(nz.action, NetworkAction::Update);
		assert_eq!(nz.op1, NetworkOperand::Node(String::from("foo")));
		assert_eq!(nz.op2, NetworkOperand::Node(String::from("bar")));
	}
	
	#[test]
	fn ts_network_update_node_state_p() {
		let mz = unwrap_some!(ManagementZone::from_str("network update node foo state disabled"));
		let nz : NetworkZone = extract_zone_network!(mz);
		assert_eq!(nz.action, NetworkAction::Update);
		assert_eq!(nz.op1, NetworkOperand::Node(String::from("foo")));
		assert_eq!(nz.op2, NetworkOperand::State(NodeState::Disabled));
	}
	
	#[test]
	fn ts_network_update_role_service_p() {
		let mz = unwrap_some!(ManagementZone::from_str("network update role org service dvsp"));
		let nz : NetworkZone = extract_zone_network!(mz);
		assert_eq!(nz.action, NetworkAction::Update);
		assert_eq!(nz.op1, NetworkOperand::Role(NodeRole::Org));
		assert_eq!(nz.op2, NetworkOperand::Service(NodeService::Dvsp));
	}
	
	#[test]
	fn ts_validation_view_all_p() {
		let mz = unwrap_some!(ManagementZone::from_str("validation view all"));
		let vz : ValidationZone = extract_zone_validation!(mz);
		assert_eq!(vz.action, ValidationAction::View);
		assert_eq!(vz.op1, ValidationOperand::All);
		assert_eq!(vz.op2, ValidationOperand::None);
	}
	
	#[test]
	fn ts_validation_view_token_p() {
		let mz = unwrap_some!(ManagementZone::from_str("validation view token abc"));
		let vz : ValidationZone = extract_zone_validation!(mz);
		assert_eq!(vz.action, ValidationAction::View);
		assert_eq!(vz.op1, ValidationOperand::Token("abc".to_string()));
		assert_eq!(vz.op2, ValidationOperand::None);
	}
	
	#[test]
	fn ts_validation_add_token_springname_p() {
		let mz = unwrap_some!(ManagementZone::from_str("validation add token abc springname foo"));
		let vz : ValidationZone = extract_zone_validation!(mz);
		assert_eq!(vz.action, ValidationAction::Add);
		assert_eq!(vz.op1, ValidationOperand::Token("abc".to_string()));
		assert_eq!(vz.op2, ValidationOperand::Node("foo".to_string()));
	}
	
	#[test]
	fn ts_validation_add_token_node_p() {
		let mz = unwrap_some!(ManagementZone::from_str("validation add token abc node foo"));
		let vz : ValidationZone = extract_zone_validation!(mz);
		assert_eq!(vz.action, ValidationAction::Add);
		assert_eq!(vz.op1, ValidationOperand::Token("abc".to_string()));
		assert_eq!(vz.op2, ValidationOperand::Node("foo".to_string()));
	}
	
	#[test]
	fn ts_validation_remove_token_p() {
		let mz = unwrap_some!(ManagementZone::from_str("validation remove token abc"));
		let vz : ValidationZone = extract_zone_validation!(mz);
		assert_eq!(vz.action, ValidationAction::Remove);
		assert_eq!(vz.op1, ValidationOperand::Token("abc".to_string()));
		assert_eq!(vz.op2, ValidationOperand::None);
	}
}