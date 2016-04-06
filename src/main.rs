/* Notice:  Copyright 2016, The Care Connections Initiative c.i.c.
 * Author:  Charlie Fyvie-Gauld (cfg@zunautica.org)
 * License: GPLv3 (http://www.gnu.org/licenses/gpl-3.0.txt)
 */
#![allow(unused_imports)]
extern crate spring_dvs;


use spring_dvs::protocol::*;
use spring_dvs::model::Netspace;

mod netspace;
mod service;
mod protocol;


fn main() {
    println!("Spring GSN Root Node");
    match service::start_dvsp() {
    	Ok(_) => println!("Service finished OK"),
    	Err(_) => println!("Service finished with error"),
    }
}
