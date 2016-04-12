use spring_dvs::model::{Url,Node,Netspace};
use spring_dvs::enums::{Failure,DvspMsgType};
use spring_dvs::protocol::{Packet, FrameResolution};
use spring_dvs::serialise::{NetSerial};

use service::chain_request;
use netspace::NetspaceIo;



pub enum ResolutionResult {
	Err(Failure),
	Network(Vec<Node>),
	Node(Node),
	Chain(Vec<u8>),
}

pub fn resolve_url(url: &str, nio: &NetspaceIo) -> ResolutionResult {
	let mut url : Url = match Url::new(url) {
		Err(e) => return ResolutionResult::Err(e),
		Ok(u) => u
	};

	
	if url.gtn() != "" {
		if url.glq() != "" {
			// Handle geolocation here
			println!("Geoloc");
			return ResolutionResult::Err(Failure::Duplicate)
		} else {
			// We don't need no GTN
			url.route_mut().pop();
		}
	}

	
	// Check to see if we are one and the same with the top GSN
	if url.route().len() > 1 {
		if url.route().last().unwrap() == "esusx" {
			url.route_mut().pop();
		}
	}
	
	if url.route().len() == 1 {
		
		let node_str = url.route().last().unwrap();
		match nio.gsn_node_by_springname(&node_str) {
			Ok(n) => ResolutionResult::Node(n),
			Err(_) => ResolutionResult::Err(Failure::InvalidArgument)
		}

	} else {

		println!("Pass through");
		
		// Here we can implement caching to reduce the amount of
		// request chaining, so reduce load on network and also
		// provide faster results for regular requests
		
		url.route_mut().pop();
		let frame  = FrameResolution::new(&url.to_string());
		let mut p = Packet::new(DvspMsgType::GsnResolution);
		p.write_content(&frame.serialise().as_ref()).unwrap();
		
		// ToDo:  Handle timeout from bad route
		
		match chain_request(p.serialise()) {
			Ok(bytes) => ResolutionResult::Chain(bytes),
			Err(f) => ResolutionResult::Err(f),
		}

	}
}