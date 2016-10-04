use spring_dvs::enums::{NodeRole};
use spring_dvs::formats::{NodeQuadFmt};
use spring_dvs::protocol::{ProtocolObject,CmdType,Message,MessageContent,ContentUri};
use spring_dvs::node::{Node,nodevec_quadvec};
use spring_dvs::spaces::{Netspace};
use spring_dvs::uri::Uri;

use ::config::{NodeConfig};
use ::chain::Chain;

/*
 * ToDo:
 * The resolution and request chaining should not be
 * trying to resolve the IP of a Geosub directly, but
 * check which nodes are acting as roots for the GSN
 * and resolve those individual nodes.
 * 
 * The Springname is NOT the GSN
 */
#[derive(Debug,Clone,PartialEq)]
pub enum ResolutionFailure {
	InvalidUri,
	InvalidRoute,
	NoHubs,
	UnresponsiveChain,
	UnsupportedAction,
}

#[derive(Debug,Clone)]
pub enum ResolutionResult {
	Err(ResolutionFailure),
	Network(Vec<NodeQuadFmt>),
	Node(Node),
	Chain(Vec<u8>),
}

#[macro_export]
macro_rules! resolution_network {
	($result: expr) => {
		match $result {
			ResolutionResult::Network(n) => n,
			_ => panic!("resolution_network: Unexpected result {:?}", $result), 
		}
	}
}

#[macro_export]
macro_rules! resolution_node {
	($result: expr) => {
		match $result {
			ResolutionResult::Node(n) => n,
			_ => panic!("resolution_node: Unexpected result {:?}", $result), 
		}
	}
}

#[macro_export]
macro_rules! resolution_chain {
	($result: expr) => {
		match $result {
			ResolutionResult::Chain(n) => n,
			_ => panic!("resolution_chain: Unexpected result {:?}", $result), 
		}
	}
}

#[macro_export]
macro_rules! resolution_err {
	($result: expr) => {
		match $result {
			ResolutionResult::Err(n) => n,
			_ => panic!("resolution_err: Unexpected result {:?}", $result), 
		}
	}
}

pub fn resolve_uri(suri: &str, nio: &Netspace, config: &NodeConfig, chain: Box<Chain>) -> ResolutionResult {
	
	let mut uri : Uri = match Uri::new(suri) {
		Err(_) => return ResolutionResult::Err(ResolutionFailure::InvalidUri),
		Ok(u) => u
	};
	
	if uri.route().len() == 0 {
		return ResolutionResult::Err(ResolutionFailure::InvalidUri)
	}
	
	if uri.gtn() != "" {
		
		match uri.query_param("__meta") {
			Some(_) => {
				 
				// *** Geolocation query goes here here
				return ResolutionResult::Err(ResolutionFailure::UnsupportedAction)
			},
			None => {
				// Get rid of the GTN
				uri.route_mut().pop();
			}
		}
	}

	
	// Check to see if we are one and the same with the top GSN
	if uri.route().len() > 1 {
		
		// If we are, then pop it off since we're checking this geosub
		if uri.route().last().unwrap().as_ref() == config.geosub() {
			uri.route_mut().pop();
		}
	}
	
	if uri.route().len() == 1 {
		
		// We only have a single name on the static route
		
		let node_str = uri.route().last().unwrap();
		// This might be a node, this might be a GSN --
		// We need to handle for both

		match nio.gsn_node_by_springname(&node_str) {
			Ok(n) => return ResolutionResult::Node(n), // route points to node
			Err(_) => {}
		}
		
		// It isn't a valid node, so it might be a GSN
		if node_str == config.geosub().as_str()  {
			// It is our GSN, so pass back the nodes
			let nodes : Vec<Node> = nio.gsn_nodes_by_type(NodeRole::Hub);
			
			if nodes.is_empty() { return ResolutionResult::Err(ResolutionFailure::NoHubs) }
				
			let nqf : Vec<NodeQuadFmt> = nodevec_quadvec(nodes);
			
			ResolutionResult::Network(nqf)
		} else {
			// The GSN is not ours so perhaps it is a remote GNS
			let gsns = nio.gtn_geosubs();
			for g in gsns {
				if &g == node_str {

					let nodes = nio.gtn_geosub_root_nodes(node_str);
					if nodes.is_empty() { return ResolutionResult::Err(ResolutionFailure::NoHubs) }
					
					let nqf : Vec<NodeQuadFmt> = nodevec_quadvec(nodes);
					return ResolutionResult::Network(nqf)

				}
			}
			
			// If we're here then it isn't a valid node or GSN
			return ResolutionResult::Err(ResolutionFailure::InvalidRoute)
			
		}
	} else if uri.route().len() > 1 {
		// Here we can implement caching to reduce the amount of
		// request chaining, so reduce load on network and also
		// provide faster results for regular requests

		let nodes = nio.gtn_geosub_root_nodes(uri.route().last().unwrap().as_ref());
		uri.route_mut().pop();

		// Note: For now we'll just use the first one for testing
		// purposes
		if nodes.is_empty() { return ResolutionResult::Err(ResolutionFailure::NoHubs) }

		let m = Message::new(
			CmdType::Resolve,
			MessageContent::Resolve( ContentUri { uri: uri } ) 
		);
		
		let out_bytes = m.to_bytes();
		for node in nodes {
			match chain.as_ref().request(&out_bytes, &node) {
				Ok(b) => return ResolutionResult::Chain(b),
				_ => continue
			}
		}
		
		ResolutionResult::Err(ResolutionFailure::UnresponsiveChain)

	} else {
		// Route isn't valid for resolving
		ResolutionResult::Err(ResolutionFailure::InvalidRoute)
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	//use spring_dvs::protocol::{ProtocolObject,Response,Message,MessageContent,CmdType,ContentResponse,ResponseContent,ContentNodeInfo,NodeInfoFmt};
	use ::config::{NodeConfig};
	use ::config::mocks::MockConfig;
	use ::netspace::{Netspace,NetspaceIo,Node,NodeRole};
	use ::chain::mocks::MockChain;
	
	
	macro_rules! try_panic{
		($e: expr) => (
			match $e {
				Ok(s) => s,
				Err(e) => panic!("try_panic! `{:?}`", e) 
			}
		)
	}
	
	macro_rules! assert_resolution{
		($exp: expr, $pat: pat) => (
			assert!(match $exp {
				$pat => true,
				_ => false,
			});
		)
	}
	
	
	
	
	pub fn add_self(ns: &Netspace, cfg: &MockConfig) {
		let s : String = format!("spring:{},host:{},address:{},service:dvsp,role:hub,state:enabled",cfg.springname(), cfg.hostname(), cfg.address());
		let n = Node::from_str(&s).unwrap();
		try_panic!(ns.gsn_node_register(&n));
		try_panic!(ns.gsn_node_update_state(&n));
		
		try_panic!(ns.gtn_geosub_register_node(&n, &cfg.geosub()));
	}
	
	fn add_node_with_name(name: &str, ns: &Netspace) {
		try_panic!(ns.gsn_node_register(&Node::from_str(&format!("spring:{},host:foobar,address:192.168.1.2,role:org,service:http,state:enabled",name)).unwrap()));
	}
	
	fn add_hub_in_gsn(name: &str, gsn: &str, ns: &Netspace) {
		try_panic!(ns.gtn_geosub_register_node(&Node::from_str(&format!("spring:{},host:foobar,address:192.168.1.2,role:org,service:http,state:enabled",name)).unwrap(), gsn));
	}
	
	fn change_node_role(name: &str, role: NodeRole, ns: &Netspace) {
		let n = try_panic!(Node::from_str(&format!("spring:{},role:{}",name,role)));
		try_panic!(ns.gsn_node_update_role(&n));
	}
	
	macro_rules! std_init {
		() => (
			{
				let cfg = MockConfig::dflt();
				let ns = new_netspace(&cfg);
				(ns,cfg)
			}
		)
	}
	
	fn new_netspace(cfg: &MockConfig) -> NetspaceIo {

		let ns = NetspaceIo::new(":memory:");
		ns.db().execute("
		CREATE TABLE `geosub_netspace` (
			`id`			INTEGER PRIMARY KEY AUTOINCREMENT,
			`springname`	TEXT UNIQUE,
			`hostname`		TEXT,
			`address`		TEXT,
			`service`		INTEGER,
			`status`		INTEGER,
			`types`			INTEGER,
			`key`			TEXT
		);
		
		CREATE TABLE `geotop_netspace` (
			`id`			INTEGER PRIMARY KEY AUTOINCREMENT,
			`springname`	TEXT,
			`hostname`		TEXT,
			`address`		TEXT,
			`service`		INTEGER,
			`priority`		INTEGER,
			`geosub`		TEXT,
			`key`			TEXT
		);
		CREATE TABLE `geosub_tokens` (
			`id`	INTEGER PRIMARY KEY AUTOINCREMENT,
			`token`	TEXT
		);").unwrap();
		
		add_self(&ns, &cfg);
		ns
	}
	
	
	// ----------- TESTS ----------- \\
	#[test]
	fn ts_resolution_pass_org_node() {
		
		let (ns,cfg) = std_init!();
		let chain = Box::new(MockChain::new(""));
		
		add_node_with_name("cci", &ns);
		
		let res = resolve_uri("spring://cci.esusx.uk", &ns, &cfg,chain);
		assert_resolution!(res, ResolutionResult::Node(_));
		assert_eq!(resolution_node!(res).springname(), "cci");
	}

	#[test]
	fn ts_resolution_org_node_route_fail() {
		
		let (ns,cfg) = std_init!();
		let chain = Box::new(MockChain::new(""));
		
		add_node_with_name("cci", &ns);
		
		let res = resolve_uri("spring://void.esusx.uk", &ns, &cfg,chain);
		assert_resolution!(res, ResolutionResult::Err(_));
		assert_eq!(resolution_err!(res), ResolutionFailure::InvalidRoute);
	}
	
	#[test]
	fn ts_resolution_org_node_fail() {
		
		let (ns,cfg) = std_init!();
		let chain = Box::new(MockChain::new(""));
		
		add_node_with_name("cci", &ns);
		
		let res = resolve_uri("spring://void", &ns, &cfg,chain);
		assert_resolution!(res, ResolutionResult::Err(_));
		assert_eq!(resolution_err!(res), ResolutionFailure::InvalidRoute);
	}

	#[test]
	fn ts_resolution_pass_local_hubs() {
		
		let (ns,cfg) = std_init!();
		let chain = Box::new(MockChain::new(""));
		
		add_node_with_name("cci", &ns);
		add_node_with_name("hub2", &ns);
		
		change_node_role("hub2", NodeRole::Hybrid, &ns);
		
		
		let res = resolve_uri("spring://esusx.uk", &ns, &cfg, chain);
		assert_resolution!(res, ResolutionResult::Network(_));
		let network = resolution_network!(res);

		assert_eq!(network.len(), 2);
	}
	
	#[test]
	fn ts_resolution_local_hubs_fail_no_hubs() {
		
		let (ns,cfg) = std_init!();
		let chain = Box::new(MockChain::new(""));
		
		add_node_with_name("cci", &ns);
		add_node_with_name("hub2", &ns);
		
		change_node_role("foohub", NodeRole::Org, &ns);
		
		
		let res = resolve_uri("spring://esusx.uk", &ns, &cfg, chain);
		assert_resolution!(res, ResolutionResult::Err(_));
		assert_eq!(resolution_err!(res), ResolutionFailure::NoHubs);
	}
	
	#[test]
	fn ts_resolution_remote_hubs_pass() {
		let (ns,cfg) = std_init!();
		let chain = Box::new(MockChain::new(""));
		
		add_hub_in_gsn("remotehub", "shire", &ns);
		let res = resolve_uri("spring://shire.uk", &ns, &cfg, chain);
		
		assert_resolution!(res, ResolutionResult::Network(_));
		let network = resolution_network!(res);
		assert_eq!(network.len(), 1);
		assert_eq!(network[0].spring, "remotehub");						
	}
	
	#[test]
	fn ts_resolution_chain_node_pass() {
		let (ns,cfg) = std_init!();
		let chain = Box::new(MockChain::new("remotehub"));
		
		add_hub_in_gsn("remotehub", "shire", &ns);
		let res = resolve_uri("spring://di.shire.uk", &ns, &cfg, chain);
		assert_resolution!(res, ResolutionResult::Chain(_));
	}
	
	#[test]
	fn ts_resolution_meta_fail() {
		let (ns,cfg) = std_init!();
		let chain = Box::new(MockChain::new("remotehub"));
		
		let res = resolve_uri("spring://uk?__meta=outcode", &ns, &cfg, chain);
		assert_resolution!(res, ResolutionResult::Err(ResolutionFailure::UnsupportedAction));
	}
	
	#[test]
	fn ts_resolution_top_invalid() {
		let (ns,cfg) = std_init!();
		let chain = Box::new(MockChain::new("remotehub"));
		
		let res = resolve_uri("spring://uk?", &ns, &cfg, chain);
		assert_resolution!(res, ResolutionResult::Err(ResolutionFailure::InvalidRoute));
	}
	
	#[test]
	fn ts_resolution_invalid_uri() {
			let (ns,cfg) = std_init!();
		let chain = Box::new(MockChain::new(""));
		let res = resolve_uri("spring://?", &ns, &cfg, chain);
		assert_resolution!(res, ResolutionResult::Err(ResolutionFailure::InvalidUri));
	}
}
