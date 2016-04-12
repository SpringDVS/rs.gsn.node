use spring_dvs::model::{Url,Node};
use spring_dvs::enums::{Failure};
use netspace::NetspaceIo;


pub enum ResolutionResult {
	Err(Failure),
	Network(Vec<Node>),
	Node(Node),
}

pub fn resolve_url(url: &str, nio: &NetspaceIo) -> ResolutionResult {
	let url = match Url::new(url) {
		Err(e) => return ResolutionResult::Err(e),
		Ok(u) => u
	};
	
	ResolutionResult::Err(Failure::Duplicate)
}