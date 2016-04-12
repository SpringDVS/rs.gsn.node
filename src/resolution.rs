use spring_dvs::model::{Url,Node,Netspace};
use spring_dvs::enums::{Failure,DvspMsgType};
use spring_dvs::protocol::{Packet, FrameResolution};
use spring_dvs::serialise::{NetSerial};

use service::chain_request;
use netspace::NetspaceIo;
use node_config::node_springname;

/*
 * ToDo:
 * The resolution and request chaining should not be
 * trying to resolve the IP of a Geosub directly, but
 * check which nodes are acting as roots for the GSN
 * and resolve those individual nodes.
 * 
 * The Springname is NOT the GSN
 */

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
		
		// Note -- we should be checking the GSN of this node,
		//         NOT the sringname
		if url.route().last().unwrap().as_ref() == node_springname() {
			url.route_mut().pop();
		}
	}
	
	if url.route().len() == 1 {
		
		let node_str = url.route().last().unwrap();
		// This might be a node, this might be a GSN --
		// We need to handle for both
		match nio.gsn_node_by_springname(&node_str) {
			Ok(n) => ResolutionResult::Node(n),
			Err(_) => ResolutionResult::Err(Failure::InvalidArgument)
		}

	} else if url.route().len() > 1 {

		println!("Pass through");
		
		// Here we can implement caching to reduce the amount of
		// request chaining, so reduce load on network and also
		// provide faster results for regular requests

		// Wrong, Wrong, Wrong
		// We want to get a root node for the supplied GSN,
		// We DON'T want to resolve a node based on the GSN
		// name
		let node = match nio.gsn_node_by_springname(url.route().last().unwrap().as_ref()) {
			Ok(n) => n,
			Err(_) => return ResolutionResult::Err(Failure::InvalidArgument)
		};
		url.route_mut().pop();
		let frame  = FrameResolution::new(&url.to_string());
		let mut p = Packet::new(DvspMsgType::GsnResolution);
		p.write_content(&frame.serialise().as_ref()).unwrap();
		
		// ToDo:  Handle timeout from bad route
		
		match chain_request(p.serialise(), &node) {
			Ok(bytes) => ResolutionResult::Chain(bytes),
			Err(f) => ResolutionResult::Err(f),
		}

	} else {
		ResolutionResult::Err(Failure::InvalidArgument)
	}
}
