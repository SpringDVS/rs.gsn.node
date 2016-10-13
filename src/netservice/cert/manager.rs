use std::slice::Iter;
use std::str::FromStr;

use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;

use ::spring_dvs::http::Outbound;

use ::protocol::Svr;
use ::netservice::cert::keyring::{Keyring,Certificate};

use ::management::ManagedService;
use rustc_serialize::json::{Json};

/* ToDo: Import: If key exists in keyring -- run an import against that key
 */

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


impl ManagedService for CertManagementInterface {
	
	fn init(&self) -> String {
		match Keyring::init() {
			true => format!("Module `certificate` initialised successfully"),
			false => format!("Module `certificate` initialisation error")
		}
	}

	fn hook(&self, atom: &Vec<String>, svr: &Svr) -> String {
		
		let mz : Zone = match Zone::parse(atom) {
			Some(m) => m,
			None => return "Unknown or malformed action".to_string(),	
		};
		
		Zone::process(mz, svr)
	}

}

#[derive(Clone, PartialEq, Debug)]
enum Action {
	View,
	Remove,
	Import
}

impl Action {
	pub fn from_str(s:&str) -> Option<Action> {
		match s {
			"imp" | "import" => Some(Action::Import),
			"viw" | "view" => Some(Action::View),
			"rem" | "remove" => Some(Action::Remove),
			_ => None,
		}
	}
}

#[derive(Clone, PartialEq, Debug)]
enum Operand {
	None,
	All,
	Certificate(String),
	Name(String),
	Key(String),
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
					"name" => {
						Operand::Name(
								String::from_str(cascade_none_nowrap!(atom.next())).unwrap()
						)
					},
					"key" => {
						Operand::Key(
								String::from_str(cascade_none_nowrap!(atom.next())).unwrap()
						)
					},
					"all" => Operand::All,
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
	
	pub fn process(mz: Zone, svr: &Svr) -> String {
		match mz.action {
			Action::Import => ZoneModel::import(mz.op1),
			Action::View => ZoneModel::view(mz.op1),
			Action::Remove => ZoneModel::remove(mz.op1)
		}	
	}
}


struct ZoneModel;

impl ZoneModel {
	
	// ToDo: If key exists in keyring -- run an import against that key
	pub fn import(op: Operand) -> String {
		let key = match op {
			Operand::Certificate(s) => s,
			_ => return format!("Import action does not support operand ({:?})", op)
		};
		
		let req : String = format!("IMPORT\nPUBLIC {{\n{}\n}}\n", key);
		let op = Outbound::request(req.as_bytes(), "217.194.223.50", "pkserv.spring-dvs.org", "process");
		
		let resp = match op {
			Some(v) => String::from_utf8(v).unwrap(),
			None => "Error Importing".to_string() 
		};
		
		let data = match Json::from_str(&resp) {
			Ok(s) => s,
			Err(e) => return format!("JSON parse error '{}'", e)
		};
		
		
		let json_cert = data.as_object().unwrap();
		
		let json_sigs = json_cert.get("sigs").unwrap().as_array().unwrap();
		let mut sigs : Vec<String> = Vec::new();
		
		for sig in json_sigs {
			sigs.push(sig.as_string().unwrap().to_string())
		}
		let cert = Certificate::new(
			json_cert.get("name").unwrap().as_string().unwrap(),
			json_cert.get("email").unwrap().as_string().unwrap(), 
			json_cert.get("keyid").unwrap().as_string().unwrap(),
			sigs,
			json_cert.get("armor").unwrap().as_string().unwrap()
		);
		
		if cert.name().len() == 0 || cert.keyid().len() == 0 || cert.email().len() == 0 {
			return format!("Error: Received malformed certificate")
		}
		
		let kr = Keyring::new();
		match kr.import(&cert) {
			true => format!("Imported certificate for `{}`\n", cert.name()),
			false => format!("Error importing certificate `{}` into keyring\n", cert.name())
		}
	}
	
	fn view(filter: Operand) -> String {
		match filter {
			Operand::All => ZoneModel::view_listing(),
			Operand::Key(s) => ZoneModel::view_with_id(&s),
			Operand::Name(s) => ZoneModel::view_with_name(&s),
			e => format!("Error: Unknown or unsupported target filter ({:?})\n", e)
		}
	}
	
	fn remove(filter: Operand) -> String {
		match filter {
			Operand::Key(s) => ZoneModel::remove_with_id(&s),
			Operand::Name(s) => ZoneModel::remove_with_name(&s),
			e => format!("Error: Unknown or unsupported target filter ({:?})\n", e)
		}
	}
	
	fn view_listing() -> String {
		let kr = Keyring::new();
		let mut table = Table::new();
		
		let certs : Vec<Certificate> = kr.listing();
		
		ZoneModel::add_listing_headings(&mut table);
		for cert in certs {
			table.add_row(Row::new(vec![
				Cell::new(cert.name()),
				Cell::new(cert.email()),
				Cell::new(cert.keyid())
				]));
		}
		
		format!("{}", table)
	}
	
	fn view_with_id(keyid: &str) -> String {
		let kr = Keyring::new();
		match kr.with_keyid(keyid) {
			Some(c) => ZoneModel::format_certificate(&c),
			None =>  format!("Error: Could not find certificate\n")
		}
	}
	
	
	fn view_with_name(name: &str) -> String {
		let kr = Keyring::new();
		match kr.with_name(name) {
			Some(c) => ZoneModel::format_certificate(&c),
			None =>  format!("Error: Could not find certificate\n")
		}
	}
	
	fn remove_with_id(keyid: &str) -> String {
		let kr = Keyring::new();
		match kr.remove_keyid(keyid) {
			true => format!("Removed certificate"),
			false =>  format!("Error: Removing certificate failed\n")
		}		
	}
	
	fn remove_with_name(name: &str) -> String {
		let kr = Keyring::new();
		match kr.remove_name(name) {
			true => format!("Removed certificate"),
			false =>  format!("Error: Removing certificate failed\n")
		}		
	}

	fn format_certificate(cert: &Certificate) -> String {
		let kr = Keyring::new();
		let mut out = String::new();
		
		out.push_str(&format!("Name:\n\t{}\n\n", cert.name()));
		out.push_str(&format!("Email:\n\t{}\n\n", cert.email()));
		out.push_str(&format!("KeyID:\n\t{}\n\n", cert.keyid()));
		
		out.push_str(&format!("Signatures:\n"));
		for sig in cert.sigs() {
			match kr.with_keyid(sig) {
				Some(c) =>  out.push_str(&format!("\t{} ({})\n", sig, c.name())),
				None => out.push_str(&format!("\t{} (unknown)\n", sig))
			}
		}
		
		
		out.push_str(&format!("\n\n{}", cert.armor()));
		
		out
	}
	
	fn add_listing_headings(table: &mut Table) {
		table.add_row(row!["_name_", "_email_",
							"_keyid_"]);
	}
	
}