pub use std::net::{SocketAddr};

extern crate spring_dvs;

use spring_dvs::enums::Success;
pub use spring_dvs::spaces::{Netspace,NetspaceFailure};
pub use spring_dvs::node::*;

pub use netspace::NetspaceIo;
pub use config::Config;
//use unit_test_env;


pub struct Svr<'s> {
	pub sock: SocketAddr,
	pub config: Config,
	pub nio: &'s Netspace
}

impl<'s> Svr<'s> {
	fn new(sock: SocketAddr, config: Config, nio: &'s Netspace) -> Svr<'s> {
		Svr{ sock:sock, config:config, nio:nio }
	}
}

fn response_content(code: Response, content: ResponseContent) -> Message {
	Message {
		cmd: CmdType::Response,
		content: MessageContent::Response(ContentResponse{ code: code, content: content })
	}
}

fn response(code: Response) -> Message {
	Message {
		cmd: CmdType::Response,
		content: MessageContent::Response(ContentResponse{ code: code, content: ResponseContent::Empty })
	}
}

pub struct Protocol;


macro_rules! valid_src {
	($node: ident, $svr: ident) => (
		match Protocol::source_valid(&$node, $svr) {
			Ok(_) => { }
			Err(r) => return response(r)
		}		
	)
}
	

impl Protocol {
	

	/// Run the action through the system
	pub fn process(msg: &Message, svr: Svr) -> Message {
		
		match msg.cmd {
			CmdType::Register => Protocol::register_action(msg, &svr),
			CmdType::Unregister => Protocol::unregister_action(msg, &svr),
			CmdType::Info => Protocol::info_action(msg, &svr),
			CmdType::Update => Protocol::update_action(msg, &svr),
			_ => Message::from_bytes(b"104").unwrap()
		}
		
		
	}
	
	fn register_action(msg: &Message, svr: &Svr) -> Message {
		let reg = msg_registration!(msg.content);
		let addr = ipaddr_str(svr.sock.ip());
		let n = Node::from_registration(reg, &addr);
		
		match svr.nio.gsn_node_register(&n) {
			Ok(_) => response(Response::Ok),
			Err(NetspaceFailure::DuplicateNode) =>  response(Response::NetspaceDuplication),
			_ => response(Response::NetspaceError)
		}
	}
	
	fn unregister_action(msg: &Message, svr: &Svr) -> Message {
		let single : &ContentNodeSingle = msg_single!(msg.content);
		let n = Node::from_node_single(&single.nsingle);
		
		match Protocol::source_valid(&n, svr) {
			Ok(_) => { }
			Err(r) => return response(r)
		}

		match svr.nio.gsn_node_unregister(&n) {
			Ok(_) => response(Response::Ok),
			_ => response(Response::NetspaceDuplication)
			
		}
	}
	
	fn info_action(msg: &Message, svr: &Svr) -> Message {

		match msg_info!(msg.content).info {
			InfoContent::Node(ref np) => Protocol::info_action_node_property(np, svr),
			InfoContent::Network => Protocol::info_action_network(svr),
		}
	}
	
	fn info_action_node_property(np: &ContentNodeProperty, svr: &Svr) -> Message {
		
		let node : Node = match svr.nio.gsn_node_by_springname(&np.spring) {
			Ok(n) => n,
			_ => return response(Response::NetspaceError)
		};

		let info = node.to_node_info_property(np.property.clone());
		let crni = ContentNodeInfo::new( info );

		response_content(Response::Ok, ResponseContent::NodeInfo(crni) )
	}
	
	fn info_action_network(svr: &Svr) -> Message {
		
		let mut v : Vec<NodeQuadFmt> = Vec::new();
		for n in svr.nio.gsn_nodes() {
			v.push( match n.to_node_quad() {
					Some(n) => n,
					None => continue
				}
			)
		}

		response_content (
			Response::Ok,
			ResponseContent::Network( ContentNetwork{ network: v } )
		)
	}
	
	fn update_action(msg: &Message, svr: &Svr) -> Message {
		let np : &ContentNodeProperty = msg_update!(msg.content);
		
		let mut n : Node = match svr.nio.gsn_node_by_springname(&np.spring) {
			Ok(n) => n,
			_ => return response(Response::NetspaceError)
		};
		
		valid_src!(n, svr);
		
		let state = match np.property {
			NodeProperty::State(Some(s)) => s,
			_ => return response(Response::UnsupportedAction),
		};
		
		n.update_state(state);
		match svr.nio.gsn_node_update_state(&n) {
			Ok(Success::Ok) => response(Response::Ok),
			_ => response(Response::NetspaceError),
		}		
	}
	
	fn source_valid(n: &Node, svr: &Svr) -> Result<Success,Response> {
		match svr.nio.gsn_node_by_springname(n.springname()) {
			Ok(n) =>  match n.address() == ipaddr_str(svr.sock.ip()) {
				true => Ok(Success::Ok),
				false => Err(Response::NetworkError),
			},
			Err(_) => Err(Response::NetspaceError)
		}
	}
	
}

// --------------------- UNIT TESTING MANAGEMENT -------------------
/*
fn process_frame_unit_test(packet: &Packet, config: &Config, nio: &NetspaceIo) -> Vec<u8> {
	if config.live_test == false { return forge_response_packet(DvspRcode::MalformedContent).unwrap().serialise() }
	let frame : FrameUnitTest = match packet.content_as::<FrameUnitTest>() {
		Ok(f) => f,
		Err(_) => return forge_response_packet(DvspRcode::MalformedContent).unwrap().serialise()
	};
	
	match frame.action {
		UnitTestAction::Reset => {
			reset_live_test_env(nio, config);
			forge_response_packet(DvspRcode::Ok).unwrap().serialise()
		},
		
		UnitTestAction::UpdateAddress => {
			update_address_test_env(nio, &frame.extra, config);
			forge_response_packet(DvspRcode::Ok).unwrap().serialise()
		},

		UnitTestAction::AddGeosubRoot => {
			add_geosub_root_test_env(nio, &frame.extra, config);
			forge_response_packet(DvspRcode::Ok).unwrap().serialise()
		},
		
		_ => forge_response_packet(DvspRcode::MalformedContent).unwrap().serialise()
	}
	
}
*/

#[cfg(test)]
mod tests {
	extern crate spring_dvs;
	
	use std::str::{FromStr};
	
	use super::*;
	
	macro_rules! assert_match {
		($e: expr, $p: pat) => (
			assert!(match $e {
				$p => true,
				_ => false,
			})
		)
	}
	
	macro_rules! try_panic{
		($e: expr) => (
			match $e {
				Ok(s) => s,
				Err(e) => panic!("try_panic! `{:?}`", e) 
			}
		)
	}
	
	macro_rules! process_assert_response {
		($msg: expr, $svr: ident, $response: expr) => ( {
			let m = Protocol::process(&new_msg($msg), $svr);
			assert_eq!(m.cmd, CmdType::Response);
			assert_match!(m.content, MessageContent::Response(_));
			assert_eq!(msg_response!(m.content).code, $response);
			m
		} )
	}
	macro_rules! process_assert_ok{
		($msg: expr, $svr: ident) => (
			process_assert_response!($msg, $svr, Response::Ok)
		 )
	}
	
	fn new_netspace() -> NetspaceIo {
		let ns = NetspaceIo::new(":memory:");
		ns.db().execute("
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
		);").unwrap();
		
		ns
	}
	
	fn new_svr(ns: &Netspace) -> Svr {	
		Svr::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::from_str("192.168.1.2").unwrap()),55400), Config::new() , ns)
	}
	
	fn new_msg(s: &str) -> Message {
		match Message::from_bytes(s.as_bytes()) { 
			Ok(s) => s,
			Err(t) => panic!("new_msg( `{}` ) -> {:?}", s, t)
		}
	}
	
	fn add_node(ns: &Netspace) {
		try_panic!(ns.gsn_node_register(&Node::from_str("spring:foo,host:foobar,address:192.168.1.2,role:hub,service:http,state:enabled").unwrap()));
	}
	
	fn add_node_with_name(name: &str, ns: &Netspace) {
		try_panic!(ns.gsn_node_register(&Node::from_str(&format!("spring:{},host:foobar,address:192.168.1.2,role:hub,service:http,state:enabled",name)).unwrap()));
	}

	fn add_remote_node(ns: &Netspace) {
		try_panic!(ns.gsn_node_register(&Node::from_str("spring:foo,host:foobar,address:192.168.1.3,role:hub,service:http,state:enabled").unwrap()));
	}
	
	fn get_node(s: &str, ns: &Netspace) -> Node {
		try_panic!(ns.gsn_node_by_springname(s))
	}
	
	#[test]
	fn ts_protocol_register_pass() {
		let ns = new_netspace();
		let svr = new_svr(&ns);
		
		process_assert_ok!("register spring,host;org;http", svr);
		
		
		let n : Node = get_node("spring", &ns);
		assert_eq!( n.springname(), "spring");
		assert_eq!( n.role(), NodeRole::Org);
		assert_eq!( n.service(), NodeService::Http);
	}

	#[test]
	fn ts_protocol_register_fail_duplicate() {
		let ns = new_netspace();
		let svr = new_svr(&ns);

		//Add duplicate
		try_panic!(ns.gsn_node_register(&Node::from_str("spring").unwrap()));
		
		process_assert_response!("register spring,host;org;http", svr, Response::NetspaceDuplication);

	}

	#[test]
	fn ts_protocol_unregister_pass() {
		let ns = new_netspace();
		let svr = new_svr(&ns);

		//Add already registered
		try_panic!(ns.gsn_node_register(&Node::from_str("spring:spring,address:192.168.1.2").unwrap()));
		
		process_assert_ok!("unregister spring", svr);
		assert_match!( ns.gsn_node_by_springname("spring"), Err(NetspaceFailure::NodeNotFound) );
	}

	#[test]
	fn ts_protocol_unregister_fail_no_node() {
		let ns = new_netspace();
		let svr = new_svr(&ns);
		
		let m = Protocol::process(&new_msg("unregister spring"), svr);
		assert_eq!(m.cmd, CmdType::Response);
		assert_match!(m.content, MessageContent::Response(_));
		
		assert_eq!(msg_response!(m.content).code, Response::NetspaceError);
	}
	
	#[test]
	fn ts_protocol_unregister_fail_wrong_src() {
		let ns = new_netspace();
		let svr = new_svr(&ns);

		//Add already registered
		try_panic!(ns.gsn_node_register(&Node::from_str("spring:spring,address:192.168.1.3").unwrap()));
		
		let m = Protocol::process(&new_msg("unregister spring"), svr);
		assert_eq!(m.cmd, CmdType::Response);
		assert_match!(m.content, MessageContent::Response(_));
		
		assert_eq!(msg_response!(m.content).code, Response::NetworkError);
	}
	
	#[test]
	fn ts_protocol_info_hostname_pass() {
		let ns = new_netspace();
		let svr = new_svr(&ns);

		//Add already registered
		add_node(&ns);
		
		let m = process_assert_ok!("info node foo hostname", svr);

		assert_match!(msg_response!(m.content).content, ResponseContent::NodeInfo(_));
		let ni = msg_response_nodeinfo!(m.content);
		
		assert_eq!(ni.info.host, "foobar");
		assert!(ni.info.spring.is_empty());
		assert!(ni.info.address.is_empty());
		assert_eq!(ni.info.service, NodeService::Undefined);
		assert_eq!(ni.info.state, NodeState::Unspecified);
		assert_eq!(ni.info.role, NodeRole::Undefined);	
	}
	
	#[test]
	fn ts_protocol_info_address_pass() {
		let ns = new_netspace();
		let svr = new_svr(&ns);

		//Add already registered
		add_node(&ns);
		
		let m = process_assert_ok!("info node foo address", svr);

		assert_match!(msg_response!(m.content).content, ResponseContent::NodeInfo(_));
		let ni = msg_response_nodeinfo!(m.content);
		
		assert!(ni.info.host.is_empty());
		assert!(ni.info.spring.is_empty());
		assert_eq!(ni.info.address, "192.168.1.2");
		assert_eq!(ni.info.service, NodeService::Undefined);
		assert_eq!(ni.info.state, NodeState::Unspecified);
		assert_eq!(ni.info.role, NodeRole::Undefined);	
	}

	#[test]
	fn ts_protocol_info_service_pass() {
		let ns = new_netspace();
		let svr = new_svr(&ns);

		//Add already registered
		add_node(&ns);
		
		let m = process_assert_ok!("info node foo service", svr);

		assert_match!(msg_response!(m.content).content, ResponseContent::NodeInfo(_));
		let ni = msg_response_nodeinfo!(m.content);
		
		assert!(ni.info.host.is_empty());
		assert!(ni.info.spring.is_empty());
		assert!(ni.info.address.is_empty());
		assert_eq!(ni.info.service, NodeService::Http);
		assert_eq!(ni.info.state, NodeState::Unspecified);
		assert_eq!(ni.info.role, NodeRole::Undefined);	
	}

	#[test]
	fn ts_protocol_info_state_pass() {
		let ns = new_netspace();
		let svr = new_svr(&ns);

		//Add already registered
		add_node(&ns);
		
		let m = process_assert_ok!("info node foo state", svr);

		assert_match!(msg_response!(m.content).content, ResponseContent::NodeInfo(_));
		let ni = msg_response_nodeinfo!(m.content);
		
		assert!(ni.info.host.is_empty());
		assert!(ni.info.spring.is_empty());
		assert!(ni.info.address.is_empty());
		assert_eq!(ni.info.service, NodeService::Undefined);
		assert_eq!(ni.info.state, NodeState::Disabled);
		assert_eq!(ni.info.role, NodeRole::Undefined);	
	}

	#[test]
	fn ts_protocol_info_role_pass() {
		let ns = new_netspace();
		let svr = new_svr(&ns);

		//Add already registered
		add_node(&ns);
		
		let m = process_assert_ok!("info node foo role", svr);
		assert_match!(msg_response!(m.content).content, ResponseContent::NodeInfo(_));
		let ni = msg_response_nodeinfo!(m.content);
		
		assert!(ni.info.host.is_empty());
		assert!(ni.info.spring.is_empty());
		assert!(ni.info.address.is_empty());
		assert_eq!(ni.info.service, NodeService::Undefined);
		assert_eq!(ni.info.state, NodeState::Unspecified);
		assert_eq!(ni.info.role, NodeRole::Hub);	
	}
	
	#[test]
	fn ts_protocol_info_pass() {
		let ns = new_netspace();
		let svr = new_svr(&ns);

		//Add already registered
		add_node(&ns);
		
		let m = process_assert_ok!("info node foo all", svr);

		assert_match!(msg_response!(m.content).content, ResponseContent::NodeInfo(_));
		let ni = msg_response_nodeinfo!(m.content);
		
		assert_eq!(ni.info.host, "foobar");
		assert_eq!(ni.info.spring, "foo");
		assert_eq!(ni.info.address, "192.168.1.2");
		assert_eq!(ni.info.service, NodeService::Http);
		assert_eq!(ni.info.state, NodeState::Disabled);
		assert_eq!(ni.info.role, NodeRole::Hub);	
	}	
	#[test]
	fn ts_protocol_info_hostname_fail_no_node() {
		let ns = new_netspace();
		let svr = new_svr(&ns);

		//Add registered
		add_node(&ns);
		
		process_assert_response!("info node void hostname", svr, Response::NetspaceError);		
	}
	
	#[test]
	fn ts_protocol_info_network_pass() {
		let ns = new_netspace();
		let svr = new_svr(&ns);

		//Add already registered
		add_node(&ns);
		add_node_with_name("croc", &ns);
		
		let m = process_assert_ok!("info network", svr);
		assert_match!(msg_response!(m.content).content, ResponseContent::Network(_));
		let cn = msg_response_network!(m.content);
		
		assert_eq!(cn.network.len(), 2);
		assert_eq!(cn.network[0].spring , "foo");
		assert_eq!(cn.network[1].spring , "croc");
	}
	
	#[test]
	fn ts_protocol_update_state_unspecified_pass() {
		let ns = new_netspace();
		let svr = new_svr(&ns);

		//Add already registered
		add_node(&ns);
		
		process_assert_ok!("update foo state unspecified", svr);
		
		let n = get_node("foo", &ns);		
		assert_eq!(n.state(), NodeState::Unspecified);
			
	}
	
	#[test]
	fn ts_protocol_update_state_enabled_pass() {
		let ns = new_netspace();
		let svr = new_svr(&ns);

		//Add already registered
		add_node(&ns);
		
		process_assert_ok!("update foo state enabled", svr);
		
		let n = get_node("foo", &ns);
		assert_eq!(n.state(), NodeState::Enabled);
			
	}
	
	#[test]
	fn ts_protocol_update_state_disabled_pass() {
		let ns = new_netspace();
		let svr = new_svr(&ns);

		//Add already registered
		add_node(&ns);
		
		process_assert_ok!("update foo state disabled", svr);
		
		let n = get_node("foo", &ns);
		assert_eq!(n.state(), NodeState::Disabled);
			
	}
	
	#[test]
	fn ts_protocol_update_state_unresponsive_pass() {
		let ns = new_netspace();
		let svr = new_svr(&ns);

		//Add already registered
		add_node(&ns);
		
		process_assert_ok!("update foo state unresponsive", svr);

		let n = get_node("foo", &ns);
		assert_eq!(n.state(), NodeState::Unresponsive);
			
	}

	#[test]
	fn ts_protocol_update_state_fail_no_node() {
		let ns = new_netspace();
		let svr = new_svr(&ns);

		//Add already registered
		add_node(&ns);
		
		process_assert_response!("update void state unspecified", svr, Response::NetspaceError);	
			
	}
	
	#[test]
	fn ts_protocol_update_state_fail_unsupported_action() {
		let ns = new_netspace();
		let svr = new_svr(&ns);

		//Add already registered
		add_node(&ns);
		
		process_assert_response!("update foo role hub", svr, Response::UnsupportedAction);
	}
	
	#[test]
	fn ts_protocol_update_state_fail_network_error() {
		let ns = new_netspace();
		let svr = new_svr(&ns);

		//Add already registered
		add_remote_node(&ns);
		
		let m = Protocol::process(&new_msg("update foo state unresponsive"), svr);
		assert_eq!(m.cmd, CmdType::Response);
		assert_match!(m.content, MessageContent::Response(_));
		
		assert_eq!(msg_response!(m.content).code, Response::NetworkError);
		
			
	}
}
