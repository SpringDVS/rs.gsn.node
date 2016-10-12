pub mod manager;
pub mod keyring;

use std::io::prelude::*;
use std::fs::File;
use std::collections::BTreeMap;

use rustc_serialize::json::{self, ToJson, Json};


use ::spring_dvs::protocol::{Message, generate_response_service_text};
use ::spring_dvs::uri::Uri;

use ::protocol::Svr;

use self::keyring::{Certificate,Key};

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
