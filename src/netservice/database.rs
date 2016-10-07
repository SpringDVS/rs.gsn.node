extern crate sqlite;
pub use self::sqlite::{State,Statement};
	
pub struct ServiceDatabase;

impl ServiceDatabase {
	pub fn new() -> sqlite::Connection {
		sqlite::open("/var/lib/springdvs/services.db").unwrap()
	}
}