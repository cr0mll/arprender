use std::time::Duration;

use arprender::{arp_scan, resolve_mac};
use clap::Parser;
use cli::{Args, Commands};

pub mod cli;
pub mod arprender;
pub mod utils;

fn main() {
    let args = Args::parse();
    
    match args.cmd {
        Commands::Interfaces => {
            let interfaces = arprender::nic::get_interfaces();
            for interface in interfaces {
                println!("{} @ {}", interface.name(), match interface.mac() { Some(addr) => addr.to_string(), None => "None".to_string()});
            }
        },
        Commands::Scan{ interface: interface_name } => {
            match arprender::nic::get_interface_by_name(&interface_name) {
                Ok(interface) => {
                    match arp_scan(&interface, Duration::from_secs(5))
                    {
                        Ok(hosts) => {
                            for host in &hosts {
                                println!("Found {} @ {}", host.0, host.1);
                            }
                        },
                        Err(err) => {
                            println!("{}", err.to_string());
                        }
                    }
                    
                },
                Err(err) => { println!("{}", err.to_string()); std::process::exit(1); }
            }
        },
        Commands::Resolve { interface, address } => {
            match arprender::nic::get_interface_by_name(&interface) {
                Ok(interface) => {
                    match resolve_mac(&interface, address, Duration::from_secs(10)) {
                        Ok(mac) => {
                            match mac {
                                Some(mac) => {
                                    println!("IP {} has MAC address {}", address, mac);
                                },
                                None => { println!("Failed to resolve address!"); }
                            }
                        },
                        Err(err) => { println!("{}", err.to_string()); std::process::exit(1); }
                    }
                },
                Err(err) => { println!("{}", err.to_string()); std::process::exit(1); }
            }
        }
    }
}
