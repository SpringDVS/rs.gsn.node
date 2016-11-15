use std::str::Split;

use netspace::*;

use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;

#[macro_export]
macro_rules! extract_zone_validation {
	($e: expr) => (
		match $e {
			ManagementZone::Validation(s) => s,
			e => panic!("extract_zone_validation -- Unexpected value: {:?}", e) 
		}
	)
}

#[derive(Copy,Clone, PartialEq, Debug)]
pub enum ValidationAction {
	View,
	Add,
	Remove
}

#[derive(Clone, PartialEq, Debug)]
pub enum ValidationOperand {
	None,
	All,
	Token(String),
	Node(String)
}

#[derive(Clone, PartialEq, Debug)]
pub struct ValidationZone {
	action: ValidationAction,
	op1: ValidationOperand,
	op2: ValidationOperand
}

impl ValidationZone {
	pub fn new(action: ValidationAction, op1: ValidationOperand, op2: ValidationOperand) -> Self {
		ValidationZone {
			action: action,
			op1: op1,
			op2: op2,
		}
	}

	pub fn from_str(msg: &str) -> Option<ValidationZone> {
		if msg.len() == 0 { return None }
		
		let mut atom = msg.split(" ");

		let action = match atom.next() {
			Some("view") => ValidationAction::View,
			Some("add") => ValidationAction::Add,
			Some("rem") | Some("remove") => ValidationAction::Remove,
			_ => return None,
		};
		
		let op1 = match cascade_none_nowrap!(Self::extract_operand(&mut atom)) {
			ValidationOperand::None => return None,
			s => s,
		} ;
		
		let op2 = cascade_none_nowrap!(Self::extract_operand(&mut atom));
		Some(ValidationZone::new(action, op1, op2))
	}
	
	fn extract_operand(atom: &mut Split<&str>) -> Option<ValidationOperand> {
		
		Some(match atom.next() {
			Some("all") =>
						ValidationOperand::All,

			Some("node") =>
						ValidationOperand::Node(
								cascade_none_nowrap!(atom.next()).to_string()
						),
						
			Some("springname") =>
						ValidationOperand::Node(
								cascade_none_nowrap!(atom.next()).to_string()
						),

			Some("token") =>
						ValidationOperand::Token(
								cascade_none_nowrap!(atom.next()).to_string()
						),
						
			_ => ValidationOperand::None
		})
	}
	
	pub fn process(vz: ValidationZone, nio: &Netspace) -> Option<String> {
		match vz.action {
			ValidationAction::View => ValidationZoneModel::view(vz.op1, nio),
			ValidationAction::Add => ValidationZoneModel::add(vz.op1, vz.op2, nio),
			ValidationAction::Remove => ValidationZoneModel::remove(vz.op1, nio),
		}
	}
}

struct ValidationZoneModel;


impl ValidationZoneModel {
	pub fn view(op: ValidationOperand, nio: &Netspace) -> Option<String> {
		Some(match op {
			ValidationOperand::All => {
				Self::tabulate_tokens(nio.gsn_tokens())
			},
			ValidationOperand::Node(s) => {
				Self::tabulate_tokens(nio.gsn_token_by_springname(&s))
			}
			e => format!("Error: Unsupported target filter ({:?})", e)
		})
		
	}
	
	pub fn add(op1: ValidationOperand, op2: ValidationOperand, nio: &Netspace) -> Option<String> {
		
		let mut token = "".to_string();
		let mut springname = "".to_string();
		
		match op1 {
			ValidationOperand::Token(s) => token = s,
			ValidationOperand::Node(s) => springname = s,
			e => return Some(format!("Error: Invalid operand ({:?})\n", e)),
		}
		
		match op2 {
			ValidationOperand::Token(s) => token = s,
			ValidationOperand::Node(s) => springname = s,
			e => return Some(format!("Error: Invalid operand ({:?})\n", e)),
		}
		
		if token.len() == 0 || springname.len() == 0 { return None }
		
		nio.gsn_add_token(&token, &springname);
		Some(format!("Added token {} for {}\n", token, springname)) 
	}
	
	pub fn remove(op1: ValidationOperand, nio: &Netspace) -> Option<String> {
		Some(match op1 {
			ValidationOperand::Token(s) => {
				nio.gsn_remove_token(&s);
				format!("Removed token {}\n", s)
			},
			ValidationOperand::Node(s) => {
				nio.gsn_remove_token_by_springname(&s);
				format!("Removed token for {}\n", s)
			},
			
			e => format!("Error: Unsupported target filter ({:?})\n", e)
		})	
	}
	
	fn add_headings(table: &mut Table) {
		table.add_row(row!["_token_", "_spring_"]);
	}
	
	fn tabulate_tokens(tokens: Vec<(String,String)>) -> String {
		let mut table = Table::new();
		Self::add_headings(&mut table);
		for token in tokens {
			table.add_row(Row::new(vec![
							Cell::new(&token.0),
							Cell::new(&token.1)]));
		}
		
		
		format!("{}", table)		
	}
}
#[cfg(test)]
mod tests {
	use super::*;
	use netspace::*;
	use management::ManagementZone;

	macro_rules! assert_match {
	
		($chk:expr, $pass:pat) => (
			assert!(match $chk {
						$pass => true,
						_ => false
			}))
	}
	
	macro_rules! unwrap_some {
		($chk:expr) => (
			match $chk {
						Some(s) => s,
						_ => panic!("Unwrapping a None")
			})		
	}
	
	#[test]
	fn ts_validation_view_all_p() {
		let mz = unwrap_some!(ManagementZone::from_str("validation view all"));
		let vz : ValidationZone = extract_zone_validation!(mz);
		assert_eq!(vz.action, ValidationAction::View);
		assert_eq!(vz.op1, ValidationOperand::All);
		assert_eq!(vz.op2, ValidationOperand::None);
	}
	
	#[test]
	fn ts_validation_view_token_p() {
		let mz = unwrap_some!(ManagementZone::from_str("validation view token abc"));
		let vz : ValidationZone = extract_zone_validation!(mz);
		assert_eq!(vz.action, ValidationAction::View);
		assert_eq!(vz.op1, ValidationOperand::Token("abc".to_string()));
		assert_eq!(vz.op2, ValidationOperand::None);
	}
	
	#[test]
	fn ts_validation_add_token_springname_p() {
		let mz = unwrap_some!(ManagementZone::from_str("validation add token abc springname foo"));
		let vz : ValidationZone = extract_zone_validation!(mz);
		assert_eq!(vz.action, ValidationAction::Add);
		assert_eq!(vz.op1, ValidationOperand::Token("abc".to_string()));
		assert_eq!(vz.op2, ValidationOperand::Node("foo".to_string()));
	}
	
	#[test]
	fn ts_validation_add_token_node_p() {
		let mz = unwrap_some!(ManagementZone::from_str("validation add token abc node foo"));
		let vz : ValidationZone = extract_zone_validation!(mz);
		assert_eq!(vz.action, ValidationAction::Add);
		assert_eq!(vz.op1, ValidationOperand::Token("abc".to_string()));
		assert_eq!(vz.op2, ValidationOperand::Node("foo".to_string()));
	}
	
	#[test]
	fn ts_validation_remove_token_p() {
		let mz = unwrap_some!(ManagementZone::from_str("validation remove token abc"));
		let vz : ValidationZone = extract_zone_validation!(mz);
		assert_eq!(vz.action, ValidationAction::Remove);
		assert_eq!(vz.op1, ValidationOperand::Token("abc".to_string()));
		assert_eq!(vz.op2, ValidationOperand::None);
	}
}