use std::net::{SocketAddr, SocketAddrV4};
use netspace::*;
use spring_dvs::formats::*;
pub use spring_dvs::serialise::{NetSerial};
pub use spring_dvs::protocol::{Packet, PacketHeader};
use spring_dvs::protocol::{FrameRegister, FrameStateUpdate, FrameNodeRequest};
use spring_dvs::protocol::{FrameResponse, FrameNodeInfo, FrameNodeStatus};



fn forge_packet<T: NetSerial>(msg_type: DvspMsgType, frame: &T) -> Result<Packet, Failure> {
	let mut p = Packet::new(msg_type);
	try!(p.write_content(frame.serialise().as_ref()));
	Ok(p)
}

fn forge_response_packet(rcode: DvspRcode) -> Result<Packet, Failure> {
	forge_packet(DvspMsgType::GsnResponse, &FrameResponse::new(rcode))
}

pub fn process_packet(bytes: &[u8], address: &SocketAddr) -> Vec<u8> {

	let mut packet : Packet = match  Packet::deserialise(&bytes) {
				Ok(p) => p,
				Err(_) => { 
					println!("Deserialise Packet error");
					return forge_response_packet(DvspRcode::MalformedContent).unwrap().serialise()
				} 
			};
	
	// this is the first hop, so we fill in the packet origin details here
	// which will be the public facing address of the host
	if packet.header().addr_orig == [0,0,0,0] {
		match address {
			&SocketAddr::V4(addr) => { packet.mut_header().addr_orig = addr.ip().octets() },
			_ => { } // ToDo: Handle IPv6
		}
	}

	match packet.header().msg_type {
		
		DvspMsgType::GsnRegistration => process_frame_register(&packet),
		DvspMsgType::GsnState => process_frame_state_update(&packet, &address),
		DvspMsgType::GsnNodeInfo => process_frame_node_info(&packet),
		DvspMsgType::GsnNodeStatus => process_frame_node_status(&packet),
		
		_ => match forge_response_packet(DvspRcode::MalformedContent) {
			Ok(p) => p.serialise(),
			_ => Vec::new()
		}
	}
}

fn process_frame_register(packet: &Packet) -> Vec<u8> {
	let nio = NetspaceIo::new("gsn.db");
	let frame : FrameRegister = match packet.content_as::<FrameRegister>() {
		Ok(f) => f,
		Err(_) => return forge_response_packet(DvspRcode::MalformedContent).unwrap().serialise()
	};

	let mut node = Node::from_node_string( 
		&nodestring_from_node_register( &frame.nodereg, &packet.header().addr_orig )
	).unwrap();

	let registered = netspace_routine_is_registered(&node, &nio);
	
	if frame.register ==  true {

		node.update_service(frame.service);
		node.update_types(frame.ntype);
		node.update_state(DvspNodeState::Disabled);

		match registered {
			true => forge_response_packet(DvspRcode::NetspaceDuplication).unwrap().serialise(),
			false => register_node(&node, &nio)
		}

	} else {

		match registered {
			false => forge_response_packet(DvspRcode::NetspaceError).unwrap().serialise(),
			true => unregister_node(&node, &nio)
		}

	}
	
}

fn register_node(node: &Node, nio: &NetspaceIo) -> Vec<u8> {
	match nio.gsn_node_register(&node) { 
		Ok(_) => forge_response_packet(DvspRcode::Ok).unwrap().serialise(),
		_ => forge_response_packet(DvspRcode::NetspaceError).unwrap().serialise(),
	}
}

fn unregister_node(node: &Node, nio: &NetspaceIo) -> Vec<u8> {
	
	match nio.gsn_node_unregister(&node) { 
		Ok(_) => forge_response_packet(DvspRcode::Ok).unwrap().serialise(),
		_ => forge_response_packet(DvspRcode::NetspaceError).unwrap().serialise(),
	}
}

fn process_frame_state_update(packet: &Packet, address: &SocketAddr) -> Vec<u8> {

	let nio = NetspaceIo::new("gsn.db");
	let frame : FrameStateUpdate = match packet.content_as::<FrameStateUpdate>() {
		Ok(f) => f,
		Err(_) => return forge_response_packet(DvspRcode::MalformedContent).unwrap().serialise()
	};

	let mut node : Node = match nio.gsn_node_by_springname(&frame.springname) {
		Ok(n) => n,
		Err(_) => return forge_response_packet(DvspRcode::NetspaceError).unwrap().serialise()
	};
	
	match address {
		&SocketAddr::V4(addr) => { 
			if node.address() != addr.ip().octets() {
				return forge_response_packet(DvspRcode::NetworkError).unwrap().serialise()
			}
		},
		_ => return forge_response_packet(DvspRcode::NetworkError).unwrap().serialise() // ToDo: Handle IPv6
	} 
	
	node.update_state(frame.status);
	
	match nio.gsn_node_update_state(&node) {
	 Ok(_) => forge_response_packet(DvspRcode::Ok).unwrap().serialise(),
	 Err(_) => return forge_response_packet(DvspRcode::NetspaceError).unwrap().serialise()
	}
}


fn process_frame_node_info(packet: &Packet) -> Vec<u8> {
	let nio = NetspaceIo::new("gsn.db");
	let frame : FrameNodeRequest = match packet.content_as::<FrameNodeRequest>() {
		Ok(f) => f,
		Err(_) => return forge_response_packet(DvspRcode::MalformedContent).unwrap().serialise()
	};
	let shi = String::from_utf8(frame.shi).unwrap();
	
	// ToDo:
	//	- Hostname
	//  - IP Address

	let node : Node = match nio.gsn_node_by_springname(&shi) {
		Ok(n) => n,
		Err(_) => return forge_response_packet(DvspRcode::NetspaceError).unwrap().serialise()
	};
	
	let info = FrameNodeInfo::new(node.types(), node.service(), node.address(), &node.to_node_register());
	
	forge_packet(DvspMsgType::GsnResponseNodeInfo, &info).unwrap().serialise()
}


fn process_frame_node_status(packet: &Packet) -> Vec<u8> {
	let nio = NetspaceIo::new("gsn.db");
	let frame : FrameNodeRequest = match packet.content_as::<FrameNodeRequest>() {
		Ok(f) => f,
		Err(_) => return forge_response_packet(DvspRcode::MalformedContent).unwrap().serialise()
	};
	let shi = String::from_utf8(frame.shi).unwrap();
	let node : Node = match nio.gsn_node_by_springname(&shi) {
		Ok(n) => n,
		Err(_) => return forge_response_packet(DvspRcode::NetspaceError).unwrap().serialise()
	};
	
	let info = FrameNodeStatus::new(node.state());
	
	forge_packet(DvspMsgType::GsnResponseStatus, &info).unwrap().serialise()
}

