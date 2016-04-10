#[derive(Copy,Clone)]
pub struct Config {
	pub live_test : bool,
}

impl Config {
	pub fn new() -> Config {
		Config {
			live_test: false
		}
	}
}