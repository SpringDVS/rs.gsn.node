extern crate sqlite;
pub use self::sqlite::{State,Statement,Value,Connection};
	
pub struct ServiceDatabase;

impl ServiceDatabase {
	pub fn new() -> Connection {
		sqlite::open("/var/lib/springdvs/services.db").unwrap()
	}
}