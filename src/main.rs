/* Notice:  Copyright 2016, The Care Connections Initiative c.i.c.
 * Author:  Charlie Fyvie-Gauld (cfg@zunautica.org)
 * License: GPLv3 (http://www.gnu.org/licenses/gpl-3.0.txt)
 */
#![allow(unused_imports)]
extern crate spring_dvs;

use std::env;

use spring_dvs::model::Netspace;
use spring_dvs::protocol::*;


mod netspace;
mod service;
mod protocol;
mod config;
mod resolution;
mod node_config;
mod requests;
mod unit_test_env;

fn main() {
	
	let mut config = config::Config::new();
	config.live_test = false;

	for a in env::args() {		
		match a.as_ref() {
			"--testing" => { config.live_test = true },
			_ => { }
		}
	}

    println!("Spring GSN Root Node\n[Node] {}.{}.uk", node_config::node_springname(), node_config::node_geosub());
    println!("[Node] {}/{}", node_config::node_hostname(), node_config::node_resource());
    
    match service::Dvsp::start(&config) {
    	Ok(_) =>{  },
    	Err(_) => println!("[Error]"),
    }
    
    match service::Tcp::start(&config) {
    	Ok(_) => {},
    	Err(_) => {println!("[Error]")},
    }
}
