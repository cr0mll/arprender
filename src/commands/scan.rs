use std::time::Duration;

use tabled::settings::{Alignment, Settings};

use crate::arp;

pub fn scan(interface: String, period: u16) {
    match arp::nic::get_interface_by_name(&interface) {
        Ok(interface) => {
            println!("Conducting ARP scan...");
            match arp::arp_scan(&interface, Duration::from_secs(period.into())) {
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
            }
        }
        Err(err) => {
            println!("{}", err.to_string());
            std::process::exit(1);
        }
    }
}
