use std::thread;
use std::sync::mpsc::channel;

use spring_dvs::enums::*;
use spring_dvs::protocol::{Packet, PacketHeader};
use spring_dvs::model::{Node,Url};

use protocol::forge_response_packet;
use service::Tcp;
// ToDo: Allow larger packet contents on TCP streams

pub fn multicast_request(packet: &Packet, nodes: &Vec<Node>) -> Packet {
	let mut v : Vec<Packet> = Vec::new();
	v.reserve(nodes.len());
	
	let (tx,rx) = channel();
	
	for i in 0..10 {
		let tx = tx.clone();
		let node : Node = nodes[i].clone();
		let inp = packet.clone();
		thread::spawn(move|| {
	
			let outp = Tcp::make_request(&inp, &node.address(), node.hostname(), node.resource(), node.service());
			
			tx.send((i,outp)).unwrap();		
		});
	}
	
	for _ in 0..nodes.len() {
		let (i, p) = rx.recv().unwrap();
		match p { Ok(x) => v[i] = x, _ => { }};
	}
	
	aggregate_responses(&v)
}

fn aggregate_responses(responses: &Vec<Packet>) -> Packet {
	
	
	
	let mut out = Packet::new(DvspMsgType::GsnResponseHigh);
	
	
	let mut v : Vec<u8> = Vec::new();
	
	for i in 0..responses.len() {
		
		match responses[i].header().msg_type { 
			DvspMsgType::GsnResponseHigh => v.extend(responses[i].content_raw().as_slice()),
			_ => { }
		}
		
	}

	out.mut_header().msg_size = v.len() as u32;
	out.tcp_flag(true);
	match out.write_content(&v.as_slice()) {
		Ok(_) => out,
		Err(_) =>  forge_response_packet(DvspRcode::MalformedContent).unwrap(),
	}
	
}