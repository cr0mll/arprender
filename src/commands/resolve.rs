use std::net::Ipv4Addr;
use std::time::Duration;

use crate::arp;

pub fn resolve(interface: String, address: Ipv4Addr, timeout: u16) {
    match arp::nic::get_interface_by_name(&interface) {
        Ok(interface) => match arp::resolve_mac(&interface, address, Duration::from_secs(timeout.into())) {
            Ok(mac) => match mac {
                Some(mac) => {
                    println!("IP {} has MAC address {}", address, mac);
                }
                None => {
                    println!("Failed to resolve address!");
                }
            },
            Err(err) => {
                println!("{}", err.to_string());
                std::process::exit(1);
            }
        },
        Err(err) => {
            println!("{}", err.to_string());
            std::process::exit(1);
        }
    }
}