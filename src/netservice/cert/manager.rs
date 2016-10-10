use std::str::Split;

use ::netservice::database::ServiceDatabase;
use ::management::ManagedService;

pub struct CertManagementInterface;

impl CertManagementInterface {
	pub fn new() -> CertManagementInterface {
		CertManagementInterface{ }
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

	fn hook(&self, atom: &mut Split<&str>) -> Option<String> {
		Some("Foobar from the managament".to_string())
	}

}