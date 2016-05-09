use std::thread;
use std::sync::mpsc::channel;

use spring_dvs::enums::*;
use spring_dvs::protocol::{Packet, PacketHeader, FrameResolution, FrameResponse};
use spring_dvs::model::{Node,Url};
use spring_dvs::serialise::NetSerial;

use protocol::forge_response_packet;
use service::Tcp;
// ToDo: Allow larger packet contents on TCP streams

#[allow(unused_variables)]
pub fn multicast_request(packet: &Packet, nodes: &Vec<Node>, url: &mut Url) -> Packet {

	let mut v : Vec<Packet> = Vec::new();
	
	let (tx,rx) = channel();
	
	for i in 0..nodes.len() {

		let tx = tx.clone();
		let node : Node = nodes[i].clone();
		
		url.route_mut().clear();
		url.route_mut().push(node.springname().to_string());
		let urlstr = url.to_string();
		let mut inp = Packet::new(DvspMsgType::GsnRequest);

		inp.write_content( FrameResolution::new(&urlstr).serialise().as_ref() ).unwrap();
		 
		thread::spawn(move|| {
			let outp = Tcp::make_request(&inp, &node.address(), node.hostname(), node.resource(), node.service());
			tx.send((i,outp)).unwrap();		
		});
	}
	
	
	for _ in 0..nodes.len() {
		let (i, p) = rx.recv().unwrap();
		match p { Ok(x) => v.push(x), _ => { }};
	}
	
	aggregate_responses(&v)
}

fn aggregate_responses(responses: &Vec<Packet>) -> Packet {
	
	let mut out = Packet::new(DvspMsgType::GsnResponseHigh);
	

	let mut v : Vec<u8> = Vec::new();

	for i in 0..responses.len() {

		match responses[i].header().msg_type {
			DvspMsgType::GsnResponseHigh =>{ 
				
				v.extend(responses[i].content_raw().as_slice());
				v.push('|' as u8);
			},
			DvspMsgType::GsnResponse => { },
			_ => {  }
		}
	}
	
	

	out.mut_header().msg_size = v.len() as u32;
	out.tcp_flag(true);
	match out.write_content(&v.as_slice()) {
		Ok(_) => out,
		Err(_) =>  forge_response_packet(DvspRcode::MalformedContent).unwrap(),
	}
	
}
