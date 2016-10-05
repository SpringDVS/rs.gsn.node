/* Notice:  Copyright 2016, The Care Connections Initiative c.i.c.
 * Author:  Charlie Fyvie-Gauld (cfg@zunautica.org)
 * License: GPLv3 (http://www.gnu.org/licenses/gpl-3.0.txt)
 */

#[macro_use]
extern crate spring_dvs;

#[macro_use]
extern crate prettytable;


use std::env;

mod config;
mod management;
mod netspace;
mod protocol;
mod network;
mod chain;
mod resolution;
mod service;
mod requests;
mod netservice;
mod unit_test_env;



use config::{NodeConfig};

fn main() {
	
	let mut config = config::Config::new();
	config.live_test = false;

	for a in env::args() {		
		match a.as_ref() {
			"--testing" => { config.live_test = true },
			"--disable-man" => {config.toggle_man = false },
			_ => { }
		}
	}

    println!("SpringNet Primary Node v0.2\n[Node] {}.{}.uk", config.springname(), config.geosub());
    println!("[Node] {}/spring/", config.hostname());
    
    if config.toggle_man {
	    match service::Management::start(&config) {
	    	Ok(_) =>{  },
	    	Err(_) => println!("[Error]"),
	    }
    } else {
    	println!("[System] Management Service Disabled");
    }

    match service::Dvsp::start(&config) {
    	Ok(_) =>{  },
    	Err(_) => println!("[Error]"),
    }
    
    match service::Tcp::start(&config) {
    	Ok(_) => {},
    	Err(_) => {println!("[Error]")},
    }
}