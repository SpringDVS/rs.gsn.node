use std::slice::Iter;

use ::netservice::database::ServiceDatabase;
use ::management::ManagedService;

macro_rules! cascade_none_nowrap {
	($opt: expr) => (
		match $opt {
			Some(s) => s,
			_ => return None,
		}
	)
}

pub struct CertManagementInterface;

impl CertManagementInterface {
	pub fn new() -> CertManagementInterface {
		CertManagementInterface{ }
	}
}

#[derive(Clone, PartialEq, Debug)]
enum Action {
	Import
}

impl Action {
	pub fn from_str(s:&str) -> Option<Action> {
		match s {
			"import" => Some(Action::Import),
			_ => None,
		}
	}
}

#[derive(Clone, PartialEq, Debug)]
enum Operand {
	None,
	Certificate(String)
}

struct Zone {
	pub action: Action,
	pub op1: Operand
}

impl Zone {
	pub fn new(action: Action, op1: Operand) -> Zone {
		Zone {
			action: action,
			op1: op1
		}
	}
	pub fn parse(v: &Vec<String>) -> Option<Zone> {
		let mut atom : Iter<String> = v.iter();
		
		let action =  match atom.next() {
			Some(s) => cascade_none_nowrap!(Action::from_str(&s)),
			None => return None,
		};
		
		let op1 = cascade_none_nowrap!(Zone::extract_operand(&mut atom));
		Some(Zone::new(action, op1))	
	}
	
	fn extract_operand(mut atom: &mut Iter<String>) -> Option<Operand> {
		
		Some(match atom.next() {
			Some(s) => match s.as_str() {
					
					"cert" => {
						Operand::Certificate(Zone::join_iter(&mut atom))
					},
					_ => Operand::None,
			},
			_ => Operand::None,
		})
	}
	
	fn join_iter(mut atom: &mut Iter<String>) -> String {
		let mut s = String::new();
		
		let mut i: Option<&String> = atom.next();
		
		while i != None {
			s.push_str(i.unwrap());
			
			i = atom.next();
			if i != None { s.push(' ') }
		}
		s
	}
	
	pub fn process(mz: Zone) -> String {
		match mz.action {
			Action::Import => ZoneModel::import(mz.op1)
		}	
	}
}


struct ZoneModel;

impl ZoneModel {
	pub fn import(op: Operand) -> String {
		match op {
			Operand::Certificate(s) => format!("Importing Certificate\n{}", s),
			_ => format!("Import action does not support operand ({:?})", op)
		}
	}
}


impl ManagedService for CertManagementInterface {
	
	fn init(&self) -> String {

		let db = ServiceDatabase::new();
		let mut statement = db.prepare("CREATE TABLE IF NOT EXISTS `certificates`( 
				`keyid` TEXT, 
				`name` TEXT, 
				`email` TEXT, 
				`sigs` TEXT, 
				`key` TEXT, 
				PRIMARY KEY(`keyid`) 
			);").unwrap();
		
		match statement.next() {
			Ok(_) => format!("Module `certificate` initialised successfully"),
			Err(e) => format!("Module `certificate` initialisation error: {:?}", e)   
		}

	}

	fn hook(&self, atom: &Vec<String>) -> String {
		
		let mz : Zone = match Zone::parse(atom) {
			Some(m) => m,
			None => return "Unknown or malformed action".to_string(),	
		};
		
		Zone::process(mz)
	}

}