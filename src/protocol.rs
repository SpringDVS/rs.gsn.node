use std::net::{SocketAddr, SocketAddrV4};
use netspace::*;
use config::Config;
use unit_test_env::{reset_live_test_env,update_address_test_env};

use spring_dvs::formats::*;
use spring_dvs::enums::{DvspRcode,DvspMsgType,UnitTestAction};

pub use spring_dvs::serialise::{NetSerial};
pub use spring_dvs::protocol::{Packet, PacketHeader};
use spring_dvs::protocol::{FrameRegister, FrameStateUpdate, FrameNodeRequest, FrameTypeRequest, FrameResolution, FrameUnitTest};
use spring_dvs::protocol::{FrameResponse, FrameNodeInfo, FrameNodeStatus, FrameNetwork};
use resolution::{resolve_url,ResolutionResult};



fn forge_packet<T: NetSerial>(msg_type: DvspMsgType, frame: &T) -> Result<Packet, Failure> {
	let mut p = Packet::new(msg_type);
	try!(p.write_content(frame.serialise().as_ref()));
	Ok(p)
}

fn forge_response_packet(rcode: DvspRcode) -> Result<Packet, Failure> {
	forge_packet(DvspMsgType::GsnResponse, &FrameResponse::new(rcode))
}



pub fn process_packet(bytes: &[u8], address: &SocketAddr, config: Config, nio: &NetspaceIo) -> Vec<u8> {
	
	let mut packet : Packet = match  Packet::deserialise(&bytes) {
				Ok(p) => { 
					if config.live_test { println!("{} | {:?}", address, p.header().msg_type) }
					p
				},
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
		
		DvspMsgType::GsnRegistration => process_frame_register(&packet,&address,&nio),
		DvspMsgType::GsnResolution => process_frame_resolution(&packet,&nio),
		DvspMsgType::GsnState => process_frame_state_update(&packet, &address,&nio),
		DvspMsgType::GsnNodeInfo => process_frame_node_info(&packet,&nio),
		DvspMsgType::GsnNodeStatus => process_frame_node_status(&packet,&nio),
		DvspMsgType::GsnArea => process_frame_area(&nio),
		DvspMsgType::GsnTypeRequest => process_frame_type_request(&packet,&nio),
		
		DvspMsgType::UnitTest => process_frame_unit_test(&packet, &config, &nio),
		
		_ => match forge_response_packet(DvspRcode::MalformedContent) {
			Ok(p) => p.serialise(),
			_ => Vec::new()
		}
	}
}

fn process_frame_register(packet: &Packet, address: &SocketAddr, nio: &NetspaceIo) -> Vec<u8> {
	let frame : FrameRegister = match packet.content_as::<FrameRegister>() {
		Ok(f) => f,
		Err(_) => return forge_response_packet(DvspRcode::MalformedContent).unwrap().serialise()
	};

	// Cracking out -- Check format!!!!!!!!
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


		// Check the IP Address
		let check_node : Node = match nio.gsn_node_by_springname(node.springname()) {
			Ok(n) => n,
			Err(_) => return forge_response_packet(DvspRcode::NetspaceError).unwrap().serialise()
		};
		
		
		let ipv4 = match address {
			&SocketAddr::V4(addr) => { addr.ip().octets() },
			_ => { [0,0,0,0] } // ToDo: Handle IPv6
		};
		
		if check_node.address() != ipv4 {
			forge_response_packet(DvspRcode::NetworkError).unwrap().serialise()
		} else {
		
			match registered {
				false => forge_response_packet(DvspRcode::NetspaceError).unwrap().serialise(),
				true => unregister_node(&node, &nio)
			}
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

fn process_frame_state_update(packet: &Packet, address: &SocketAddr, nio: &NetspaceIo) -> Vec<u8> {

	
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


fn process_frame_node_info(packet: &Packet, nio: &NetspaceIo) -> Vec<u8> {
	let frame : FrameNodeRequest = match packet.content_as::<FrameNodeRequest>() {
		Ok(f) => f,
		Err(_) => return forge_response_packet(DvspRcode::MalformedContent).unwrap().serialise()
	};

	let shi = match String::from_utf8(frame.shi) {
		Ok(s) => s,
		_ => return forge_response_packet(DvspRcode::MalformedContent).unwrap().serialise()
	};
	
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


fn process_frame_node_status(packet: &Packet, nio: &NetspaceIo) -> Vec<u8> {

	let frame : FrameNodeRequest = match packet.content_as::<FrameNodeRequest>() {
		Ok(f) => f,
		Err(_) => return forge_response_packet(DvspRcode::MalformedContent).unwrap().serialise()
	};

	let shi = match String::from_utf8(frame.shi) {
		Ok(s) => s,
		_ => return forge_response_packet(DvspRcode::MalformedContent).unwrap().serialise()
	};
	
	let node : Node = match nio.gsn_node_by_springname(&shi) {
		Ok(n) => n,
		Err(_) => return forge_response_packet(DvspRcode::NetspaceError).unwrap().serialise()
	};
	
	let info = FrameNodeStatus::new(node.state());
	
	forge_packet(DvspMsgType::GsnResponseStatus, &info).unwrap().serialise()
}


fn process_frame_area(nio: &NetspaceIo) -> Vec<u8> {
	let v = nio.gsn_nodes();
	
	let frame = FrameNetwork::new(&nodes_to_node_list(&v));
	forge_packet(DvspMsgType::GsnResponseNetwork, &frame).unwrap().serialise()
}

fn process_frame_type_request(packet: &Packet, nio: &NetspaceIo) -> Vec<u8> {
	
	let f : FrameTypeRequest = match packet.content_as::<FrameTypeRequest>() {
		Ok(f) => f,
		Err(_) => return forge_response_packet(DvspRcode::MalformedContent).unwrap().serialise()
	};
	
	let v = nio.gsn_nodes_by_type(f.ntype);
	
	let frame = FrameNetwork::new(&nodes_to_node_list(&v));
	forge_packet(DvspMsgType::GsnResponseNetwork, &frame).unwrap().serialise()
}


fn process_frame_resolution(packet: &Packet, nio: &NetspaceIo) -> Vec<u8> {
	let frame : FrameResolution = match packet.content_as::<FrameResolution>() {
		Ok(f) => f,
		Err(_) => return forge_response_packet(DvspRcode::MalformedContent).unwrap().serialise()
	};
	
	match resolve_url(&frame.url, nio) {
		ResolutionResult::Err(_) => forge_response_packet(DvspRcode::MalformedContent).unwrap().serialise(),
		ResolutionResult::Node(n) => {
			let node : Node = n;
			let frame = FrameNodeInfo::new(node.types(), node.service(), node.address(), &node.to_node_register());
			forge_packet(DvspMsgType::GsnNodeInfo, &frame).unwrap().serialise()
		},
		ResolutionResult::Network(nodes) => {
			let frame =	FrameNetwork::new(&nodes_to_node_list(&nodes));
			
			forge_packet(DvspMsgType::GsnResponseNetwork, &frame).unwrap().serialise()
		}
	}
	
}



// --------------------- UNIT TESTING MANAGEMENT -------------------

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

		_ => forge_response_packet(DvspRcode::MalformedContent).unwrap().serialise()
	}
	
}