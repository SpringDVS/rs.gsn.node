pub mod cert;
pub mod database;

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum Module {
	Cert,
}


impl Module {
	pub fn from_str(s: &str) -> Option<Module> {
		match s {
			"cert" => Some(Module::Cert),
			_ => None
		}
	}
}