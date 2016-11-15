use std::collections::BTreeMap;

use rustc_serialize::json::{ToJson, Json};


use ::netservice::database::{ServiceDatabase,State, Statement,Value,Connection};

pub struct Keyring {
	db: Connection
}

impl Keyring {
	pub fn new() -> Keyring {
		Keyring {
			db: ServiceDatabase::new()
		}
	}
	
	pub fn init() -> bool {
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
			Ok(_) => true ,
			Err(_) => false   
		}
	}
	
	pub fn import(&self, certificate: &Certificate) -> bool {
		
		let mut statement = self.db.prepare("INSERT OR REPLACE INTO `certificates`
									(keyid,name,email,sigs,key)
									VALUES (?,?,?,?,?)").unwrap();
		
		statement.bind(1, &Value::String( certificate.keyid().to_string() ) ).unwrap();
		statement.bind(2, &Value::String( certificate.name().to_string() ) ).unwrap();
		statement.bind(3, &Value::String( certificate.email().to_string() ) ).unwrap();
		statement.bind(4, &Value::String( certificate.sigs().join(",") ) ).unwrap();
		statement.bind(5, &Value::String( certificate.armor().to_string() ) ).unwrap();
		
		match statement.next() {
			Ok(_) => true,
			Err(_) => false   
		}
	}
	
	pub fn listing(&self) -> Vec<Certificate> {
		let mut statement = self.db.prepare("SELECT * FROM `certificates`").unwrap();
		let mut v = Vec::new();
		while let State::Row = statement.next().unwrap() {
			v.push(self.certifcate_from_row(&statement))
		}
		
		v								
	}
	
	pub fn with_name(&self, name: &str) -> Option<Certificate> {
		let mut statement = self.db.prepare("SELECT * FROM `certificates`
											WHERE name=?").unwrap();
		statement.bind(1, &Value::String( name.to_string() ) ).unwrap();
		
		
		match statement.next().unwrap() {
			State::Row => Some(self.certifcate_from_row(&statement)),
			_ => None
		}
	}
	
	pub fn with_keyid(&self, keyid: &str) -> Option<Certificate> {
		let mut statement = self.db.prepare("SELECT * FROM `certificates`
											WHERE keyid=?").unwrap();
		statement.bind(1, &Value::String( keyid.to_string() ) ).unwrap();
		
		
		match statement.next().unwrap() {
			State::Row => Some(self.certifcate_from_row(&statement)),
			_ => None
		}
	}
	
	pub fn remove_keyid(&self, keyid: &str) -> bool {
		let mut statement = self.db.prepare("DELETE FROM `certificates`
											WHERE keyid=?").unwrap();
		statement.bind(1, &Value::String( keyid.to_string() ) ).unwrap();
		match statement.next() {
			Ok(_) => true,
			Err(_) => false   
		}			
	}

	pub fn remove_name(&self, name: &str) -> bool {
		let mut statement = self.db.prepare("DELETE FROM `certificates`
											WHERE name=?").unwrap();
		statement.bind(1, &Value::String( name.to_string() ) ).unwrap();
		match statement.next() {
			Ok(_) => true,
			Err(_) => false   
		}			
	}
	fn certifcate_from_row(&self, row: &Statement) -> Certificate {

		let keyid = row.read::<String>(0).unwrap();
		let name = row.read::<String>(1).unwrap();
		let email = row.read::<String>(2).unwrap();
		let sigs_str = row.read::<String>(3).unwrap();
		let sigs_split = sigs_str.split(",");
		let armor = row.read::<String>(4).unwrap();
		
		let mut sigs = Vec::new();
		for sig in sigs_split {
			sigs.push(sig.to_string())
		}
		
		Certificate::new(&name, &email, &keyid, sigs, &armor)
	}
}

#[derive(RustcEncodable,Debug,Clone)]
pub struct Certificate {
	name: String,
	email: String,
	keyid: String,
	sigs: Vec<String>,
	armor: String
}

impl Certificate {
	pub fn new(name: &str, email: &str, keyid: &str, sigs: Vec<String>, armor: &str) -> Certificate {
		Certificate {
			name: name.to_string(),
			email: email.to_string(),
			keyid: keyid.to_string(),
			sigs: sigs,
			armor: armor.to_string()
		}
	}

	pub fn error() -> Certificate {
		Certificate {
			name: "#error".to_string(),
			email: "#error".to_string(),
			keyid: "#error".to_string(),
			armor: "#error".to_string(),
			sigs: Vec::new() 
		}
	}
	
	pub fn name(&self) -> &str {
		&self.name
	}
	
	pub fn email(&self) -> &str {
		&self.email
	}
	
	pub fn keyid(&self) -> &str {
		&self.keyid
	}

	pub fn armor(&self) -> &str {
		&self.armor
	}
	
	pub fn sigs(&self) -> &Vec<String> {
		&self.sigs
	}
}

impl ToJson  for Certificate {
	fn to_json(&self) -> Json {
		let mut outer = BTreeMap::new();
		if self.name == "#error" {
			outer.insert("cert".to_string(), "error".to_string().to_json());		
		} else {
			let mut inner = BTreeMap::new();
			inner.insert("name".to_string(), self.name.to_json());
			inner.insert("email".to_string(), self.email.to_json());
			inner.insert("keyid".to_string(), self.keyid.to_json());
			inner.insert("sigs".to_string(), self.sigs.to_json());
			inner.insert("armor".to_string(), self.armor.to_json());
			outer.insert("cert".to_string(), Json::Object(inner));		
		}
		
		
		Json::Object(outer)
	}
}
#[derive(RustcEncodable,Debug,Clone)]
pub struct Key {
	key: String	
}



impl Key {
	pub fn new(key: &str) -> Key {
		Key {
			key: key.to_string()
		}
	}
	pub fn error() -> Key {
		Key {
			key: "error".to_string()
		}
	}
}
impl ToJson for Key {
	fn to_json(&self) -> Json {
		let mut outer = BTreeMap::new();
		if self.key == "#error" {
			outer.insert("key".to_string(), "error".to_string().to_json());		
		} else {
			outer.insert("key".to_string(), self.key.to_json());
		}
		
		Json::Object(outer)
	}
}