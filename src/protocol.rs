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

impl Protocol {
	
	/// Run the action through the system
	pub fn process(msg: &Message, svr: Svr) -> Message {
		
		match msg.cmd {
			CmdType::Register => Protocol::register_action(msg, &svr),
			CmdType::Unregister => Protocol::unregister_action(msg, &svr),
			CmdType::Info => Protocol::info_action(msg, &svr),
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
		let addr = ipaddr_str(svr.sock.ip());
		match Protocol::source_valid(&n, &addr, svr) {
			Ok(_) => { }
			Err(r) => return response(r)
		}

		match svr.nio.gsn_node_unregister(&n) {
			Ok(_) => response(Response::Ok),
			_ => response(Response::NetspaceDuplication)
			
		}
	}
	
	fn info_action(msg: &Message, svr: &Svr) -> Message {
		let np : &ContentNodeProperty = msg_info_property!(msg.content);
		
		let node : Node = match svr.nio.gsn_node_by_springname(&np.spring) {
			Ok(n) => n,
			_ => return response(Response::NetspaceError)
		};
		
		let mut info = node.to_node_info_property(np.property.clone());
		let crni = ContentNodeInfo::new( info );
		
		response_content(Response::Ok, ResponseContent::NodeInfo(crni) )
		
	}
	
	fn source_valid(n: &Node, addr: &str, svr: &Svr) -> Result<Success,Response> {
		match svr.nio.gsn_node_by_springname(n.springname()) {
			Ok(n) =>  match n.address() == addr {
				true => Ok(Success::Ok),
				false => Err(Response::NetworkError),
			},
			Err(e) => Err(Response::NetspaceError)
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
		ns.gsn_node_register(&Node::from_str("spring:foo,host:foobar,address:172.168.1.1,role:hub,service:http,state:enabled").unwrap());
	} 
	
	#[test]
	fn ts_protocol_register_pass() {
		let ns = new_netspace();
		let svr = new_svr(&ns);
		let m = Protocol::process(&new_msg("register spring,host;org;http"), svr);
		assert_eq!(m.cmd, CmdType::Response);
		assert_match!(m.content, MessageContent::Response(_));
		
		assert_eq!(msg_response!(m.content).code, Response::Ok);
		let n : Node = try_panic!(ns.gsn_node_by_springname("spring"));
		assert_eq!( n.springname(), "spring");
		assert_eq!( n.role(), NodeRole::Org);
		assert_eq!( n.service(), NodeService::Http);
	}

	#[test]
	fn ts_protocol_register_fail_duplicate() {
		let ns = new_netspace();
		let svr = new_svr(&ns);

		//Add duplicate
		ns.gsn_node_register(&Node::from_str("spring").unwrap());
		
		let m = Protocol::process(&new_msg("register spring,host;org;http"), svr);
		assert_eq!(m.cmd, CmdType::Response);
		assert_match!(m.content, MessageContent::Response(_));
		
		assert_eq!(msg_response!(m.content).code, Response::NetspaceDuplication);
	}

	#[test]
	fn ts_protocol_unregister_pass() {
		let ns = new_netspace();
		let svr = new_svr(&ns);

		//Add already registered
		ns.gsn_node_register(&Node::from_str("spring:spring,address:192.168.1.2").unwrap());
		
		let m = Protocol::process(&new_msg("unregister spring"), svr);
		assert_eq!(m.cmd, CmdType::Response);
		assert_match!(m.content, MessageContent::Response(_));
		
		assert_eq!(msg_response!(m.content).code, Response::Ok);
		
		assert_match!(ns.gsn_node_by_springname("spring"), Err(NetspaceFailure::NodeNotFound) );
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
		ns.gsn_node_register(&Node::from_str("spring:spring,address:192.168.1.3").unwrap());
		
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
		
		let m = Protocol::process(&new_msg("info node foo hostname"), svr);
		assert_eq!(m.cmd, CmdType::Response);
		assert_match!(m.content, MessageContent::Response(_));
		
		assert_eq!(msg_response!(m.content).code, Response::Ok);
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
		
		let m = Protocol::process(&new_msg("info node foo address"), svr);
		assert_eq!(m.cmd, CmdType::Response);
		assert_match!(m.content, MessageContent::Response(_));
		
		assert_eq!(msg_response!(m.content).code, Response::Ok);
		assert_match!(msg_response!(m.content).content, ResponseContent::NodeInfo(_));
		let ni = msg_response_nodeinfo!(m.content);
		
		assert!(ni.info.host.is_empty());
		assert!(ni.info.spring.is_empty());
		assert_eq!(ni.info.address, "172.168.1.1");
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
		
		let m = Protocol::process(&new_msg("info node foo service"), svr);
		assert_eq!(m.cmd, CmdType::Response);
		assert_match!(m.content, MessageContent::Response(_));
		
		assert_eq!(msg_response!(m.content).code, Response::Ok);
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
		
		let m = Protocol::process(&new_msg("info node foo state"), svr);
		assert_eq!(m.cmd, CmdType::Response);
		assert_match!(m.content, MessageContent::Response(_));
		
		assert_eq!(msg_response!(m.content).code, Response::Ok);
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
		
		let m = Protocol::process(&new_msg("info node foo role"), svr);
		assert_eq!(m.cmd, CmdType::Response);
		assert_match!(m.content, MessageContent::Response(_));
		
		assert_eq!(msg_response!(m.content).code, Response::Ok);
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
		
		let m = Protocol::process(&new_msg("info node foo all"), svr);
		assert_eq!(m.cmd, CmdType::Response);
		assert_match!(m.content, MessageContent::Response(_));
		
		assert_eq!(msg_response!(m.content).code, Response::Ok);
		assert_match!(msg_response!(m.content).content, ResponseContent::NodeInfo(_));
		let ni = msg_response_nodeinfo!(m.content);
		
		assert_eq!(ni.info.host, "foobar");
		assert_eq!(ni.info.spring, "foo");
		assert_eq!(ni.info.address, "172.168.1.1");
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
		
		let m = Protocol::process(&new_msg("info node void hostname"), svr);
		assert_eq!(m.cmd, CmdType::Response);
		assert_match!(m.content, MessageContent::Response(_));
		
		assert_eq!(msg_response!(m.content).code, Response::NetspaceError);
		
	}
}
