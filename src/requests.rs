use std::thread;
use std::sync::mpsc::channel;

use spring_dvs::protocol::{Bytes,CmdType,ProtocolObject,Message,MessageContent,ResponseContent};
use spring_dvs::node::Node;
use spring_dvs::enums::{NodeService,Failure};
use spring_dvs::uri::Uri;
use spring_dvs::http;

use service::Tcp;

pub fn multicast_request(nodes: &Vec<Node>, uri: &mut Uri) -> Bytes {

	let mut v : Vec<Message> = Vec::new();
	let dbg_uri = uri.to_string();
	println!("[Service] Processing {}", dbg_uri);
	let (tx,rx) = channel();
	
	
	
	for i in 0..nodes.len() {
		
		let node : Node = nodes[i].clone();
		
		let tx = tx.clone();
		uri.route_mut().clear();
		uri.route_mut().push(node.springname().to_string());
		let uristr = uri.to_string();

		let outbound = match Message::from_bytes(format!("service {}", uristr).as_bytes()) {
			Ok(m) => m,
			Err(_) => continue
		};

		thread::spawn(move|| {
				
				
			let inbound = match node.service() {
				NodeService::Dvsp =>
					Tcp::make_request(&outbound, &node.address(), node.hostname(), node.service()),

				NodeService::Http =>
					match http::Outbound::request_node(&outbound, &node) {
						Some(m) => Ok(m),
						None => Err(Failure::InvalidBytes)
					},
				_ => Err(Failure::InvalidArgument),
			};
			tx.send((i,inbound)).unwrap();		
		});
	}
	
	
	for _ in 0..nodes.len() {
		let (_, p) = rx.recv().unwrap();
		match p { Ok(x) => v.push(x), _ => { }};
	}
	
	aggregate_responses(&v)
}

fn aggregate_responses(responses: &Vec<Message>) -> Bytes {
	
	
	let mut v : Vec<u8> = Vec::new();
	v.extend_from_slice(b"200 14 service/multi ");
	for i in 0..responses.len() {

		match responses[i].cmd {

			CmdType::Response => {
				let rc = msg_response!(responses[i].content);
				match rc.content { 
					ResponseContent::ServiceText(ref t) => {
						v.extend_from_slice(format!("201 {} service/text {} ", 13+t.content.len(), t.content).as_bytes())
					},
					_ => continue,
				}

			},

			_ => { }

		}
	}
	
	v.extend_from_slice(b"202");
	v	
}
