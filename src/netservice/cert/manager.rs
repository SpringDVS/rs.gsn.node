use ::netservice::database::ServiceDatabase;

pub fn service_init() -> String {
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