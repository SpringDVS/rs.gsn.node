pub use spring_dvs::serialise::{NetSerial};
pub use spring_dvs::protocol::{Packet, PacketHeader};
pub use spring_dvs::protocol::{FrameResponse};
pub use spring_dvs::enums::*;


fn forge_packet<T: NetSerial>(msg_type: DvspMsgType, frame: &T) -> Result<Packet, Failure> {
	let mut p = Packet::new(msg_type);
	try!(p.write_content(frame.serialise().as_ref()));
	Ok(p)
}

fn forge_response_packet(rcode: DvspRcode) -> Result<Packet, Failure> {
	forge_packet(DvspMsgType::GsnResponse, &FrameResponse::new(rcode))
}

pub fn process_packet(bytes: &[u8]) -> Vec<u8> {

	let packet : Packet = match  Packet::deserialise(&bytes) {
				Ok(p) => p,
				Err(_) => { 
					println!("Deserialise error");
					let fr = FrameResponse::new(DvspRcode::MalformedContent);
					return fr.serialise() 
				} 
			};
		
	match packet.header().msg_type {
		DvspMsgType::GsnRegistration => process_frame_register(),
		_ => match forge_response_packet(DvspRcode::MalformedContent) {
			Ok(p) => p.serialise(),
			_ => Vec::new()
		}
	}
}

fn process_frame_register() -> Vec<u8> {
	
	match forge_response_packet(DvspRcode::Ok) {
		Ok(p) => p.serialise(),
		_ => Vec::new()
	}
	
}