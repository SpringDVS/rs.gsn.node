use spring_dvs::node::*;
use spring_dvs::spaces::Netspace;
//use service::chain_request;

use node_config::*;

/*
 * ToDo:
 * The resolution and request chaining should not be
 * trying to resolve the IP of a Geosub directly, but
 * check which nodes are acting as roots for the GSN
 * and resolve those individual nodes.
 * 
 * The Springname is NOT the GSN
 */

pub enum ResolutionFailure {
	InvalidUri,
	InvalidRoute,
	NoHubs,
}

pub enum ResolutionResult {
	Err(ResolutionFailure),
	Network(Vec<Node>),
	Node(Node),
	Chain(Vec<u8>),
}

pub fn resolve_uri(uri: &str, nio: &Netspace) -> ResolutionResult {
	
	let mut uri : Uri = match Uri::new(uri) {
		Err(e) => return ResolutionResult::Err(ResolutionFailure::InvalidUri),
		Ok(u) => u
	};

	
	
	if uri.gtn() != "" {
		
		match uri.query_param("__meta") {
			Some(v) => {
				// *** Geolocation query goes here here
				return ResolutionResult::Err(ResolutionFailure::InvalidRoute)
			},
			None => {
				uri.route_mut().pop();
			}
		}
	}

	
	// Check to see if we are one and the same with the top GSN
	if uri.route().len() > 1 {
		
		// Now changed to checking a geosub -- FIXED
		if uri.route().last().unwrap().as_ref() == node_geosub() {
			uri.route_mut().pop();
		}
	}
	
	if uri.route().len() == 1 {
		
		let node_str = uri.route().last().unwrap();
		// This might be a node, this might be a GSN --
		// We need to handle for both

		match nio.gsn_node_by_springname(&node_str) {
			Ok(n) => return ResolutionResult::Node(n),
			Err(_) => {}
		}
		
		if(node_str == node_geosub().as_str()) {
			// ToDo: Handle for resolving Hub nodes of GSN root
			// for now we'll just resolve as this node
			let res = format!("spring:{},host:{},address:{},role:hub,service:dvsp", 
				node_springname(),
				node_hostname(),
				node_address()
			);

			let mut n = Node::from_str(res.as_str()).unwrap();
			ResolutionResult::Node(n)
		} else {
			ResolutionResult::Err(ResolutionFailure::InvalidRoute)
		}
	} else if uri.route().len() > 1 {

		println!("Pass through");
		
		// Here we can implement caching to reduce the amount of
		// request chaining, so reduce load on network and also
		// provide faster results for regular requests

		let nodes = nio.gtn_geosub_root_nodes(uri.route().last().unwrap().as_ref());
		uri.route_mut().pop();

		// Note: For now we'll just use the first one for testing
		// purposes
		if nodes.is_empty() { return ResolutionResult::Err(ResolutionFailure::NoHubs) }
		println!("Chaining to: {}", nodes[0].to_node_triple().unwrap());
		let m = Message {
			cmd: CmdType::Resolve,
			content: MessageContent::Resolve( ContentUri { uri: uri } )
		};
		
		// ToDo:  Handle timeout from bad route
		
		//match chain_request(m.as_bytes(), &nodes[0]) {
			//Ok(bytes) => ResolutionResult::Chain(bytes),
			//Err(f) => ResolutionResult::Err(f),
		//}
		ResolutionResult::Err(ResolutionFailure::InvalidUri)
	} else {
		ResolutionResult::Err(ResolutionFailure::InvalidRoute)
	}
}
