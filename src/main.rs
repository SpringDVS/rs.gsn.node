/* Notice:  Copyright 2016, The Care Connections Initiative c.i.c.
 * Author:  Charlie Fyvie-Gauld (cfg@zunautica.org)
 * License: GPLv3 (http://www.gnu.org/licenses/gpl-3.0.txt)
 */

#[macro_use]
extern crate spring_dvs;

#[macro_use]
extern crate prettytable;

extern crate rustc_serialize;


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
			"--disable-man" => {
				if config.toggle_offline != true {
					config.toggle_man = false
				}
			},
			"--enable-offline" => {
								config.toggle_man = true;
								config.toggle_offline = true;
							},
			_ => { }
		}
	}

    println!("SpringNet Primary Node v0.5.0\n[Node] {}.{}.uk", config.springname(), config.geosub());
    println!("[Node] {}/spring/", config.hostname());
    
	if config.toggle_offline {
	    println!("[Alert] Server running in offline maintenance mode");
	}
 
    if config.toggle_man {
	    match service::Management::start(&config) {
	    	Ok(_) =>{  },
	    	Err(_) => println!("[Error]"),
	    }
    } else {
    	println!("[System] Management Service Disabled");
    }
    
    
    // If we're in offline mode we'll wait for the management
    // service to thread to end and the exit
    if config.toggle_offline { return }

	
    match service::Dvsp::start(&config) {
    	Ok(_) =>{  },
    	Err(_) => println!("[Error]"),
    }
    
    match service::Tcp::start(&config) {
    	Ok(_) => {},
    	Err(_) => {println!("[Error]")},
    }
}