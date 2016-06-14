pub use std::net::{SocketAddr};

extern crate spring_dvs;

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
			match $e {
				$p => true,
				_ => false,
			}
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
	
}
