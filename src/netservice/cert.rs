use ::spring_dvs::protocol::{Message,ProtocolObject};
use ::spring_dvs::uri::Uri;

pub fn request(uri: &Uri) -> Message {
	Message::from_bytes(b"200").unwrap()
}