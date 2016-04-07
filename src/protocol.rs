use std::net::{SocketAddr, SocketAddrV4};
use netspace::*;
use spring_dvs::formats::*;
pub use spring_dvs::serialise::{NetSerial};
pub use spring_dvs::protocol::{Packet, PacketHeader};
pub use spring_dvs::protocol::{FrameResponse, FrameRegister};



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
					println!("Deserialise error");
					let fr = FrameResponse::new(DvspRcode::MalformedContent);
					return fr.serialise() 
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
		Err(_) => return forge_response_packet(DvspRcode::Ok).unwrap().serialise()
	};

	let node = Node::from_node_string( 
		&nodestring_from_node_register( &frame.nodereg, &packet.header().addr_orig )
	).unwrap();

	let registered = netspace_routine_is_registered(&node, &nio);
	
	if frame.register ==  true {
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