pub use spring_dvs::serialise::{NetSerial};
pub use spring_dvs::protocol::{Packet, PacketHeader};
pub use spring_dvs::protocol::{FrameResponse};
pub use spring_dvs::enums::*;


//fn forge_packet(t: DvspMsgType, content

pub fn process_packet(bytes: &[u8]) -> Vec<u8> {
	let packet : Packet = match  Packet::deserialise(&bytes) {
				Ok(p) => p,
				Err(_) => { 
					println!("Deserialise error");
					let fr = FrameResponse::new(DvspRcode::MalformedContent);
					return fr.serialise() 
				} 
			};
		
	println!("msg_type: {}", packet.header().msg_type as u8);
	let fr = FrameResponse::new(DvspRcode::Ok);
	
	let mut p = Packet::new(DvspMsgType::GsnResponse);
	match p.write_content(fr.serialise().as_ref()) {
		Err(f) => println!("Failed to write frame to packet: {}", f as u8),
		_ => println!("Wrote response correctly"),
	};
	
	
	p.serialise()
}