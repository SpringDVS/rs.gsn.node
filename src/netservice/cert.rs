use std::io::prelude::*;
use std::fs::File;
use std::collections::BTreeMap;

use rustc_serialize::json::{self, ToJson, Json};

use ::spring_dvs::protocol::{Message, generate_response_service_text};
use ::spring_dvs::uri::Uri;

use ::protocol::Svr;


#[derive(RustcEncodable,Debug,Clone)]
struct Certificate {
	name: String,
	email: String,
	keyid: String,
	sigs: Vec<String>,
	armor: String
}


impl Certificate {
	pub fn error() -> Certificate {
		Certificate {
			name: "#error".to_string(),
			email: "#error".to_string(),
			keyid: "#error".to_string(),
			armor: "#error".to_string(),
			sigs: Vec::new() 
		}
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
struct Key {
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

#[derive(RustcEncodable,Debug,Clone)]
enum Response {
	Certificate(Certificate),
	Key(Key)
}

#[derive(RustcEncodable,Debug,Clone)]
struct CertResponse {
	uri: String,
	response: Response
}

impl CertResponse {
	pub fn new(uri: String, response: Response) -> CertResponse {
		CertResponse{ uri: uri, response: response }
	}
}

impl ToJson for CertResponse {
	fn to_json(&self) -> Json {
		let mut d = BTreeMap::new();
		
		d.insert(self.uri.clone(), match &self.response {
				&Response::Certificate(ref c) => c.to_json(),
				&Response::Key(ref k) => k.to_json(),
		});
		
		Json::Object(d)
	}
}

pub fn request(uri: &Uri, svr: &Svr) -> Message {
	match uri.res_index(1) {
		None => request_certificate(svr),
		Some("key") => request_key(svr),
		_ => service_response(Response::Certificate(Certificate::error()), svr)
	}
	
}

fn request_certificate(svr: &Svr) -> Message {
	service_response(Response::Certificate(Certificate::error()), svr)
}

fn request_key(svr: &Svr) -> Message {
	
	let mut f = match File::open("/etc/springdvs/cert.asc") {
		Ok(f) => f,
		Err(_) => return service_response(Response::Key(Key::error()), svr)
	};

	let mut s = String::new();
	match f.read_to_string(&mut s) {
		Err(_) => return service_response(Response::Key(Key::error()), svr),
		_ => {}
	}
	
	service_response(Response::Key(Key::new(&s)), svr)
}

fn service_response(response: Response, svr: &Svr) -> Message {
	let r = CertResponse::new(format!("{}.{}.uk", svr.config.springname(), svr.config.geosub()), response);
	generate_response_service_text(&json::encode(&r.to_json()).unwrap())
}