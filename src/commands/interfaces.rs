use tabled::settings::{Alignment, Settings};
use crate::arp;

pub fn interfaces() {
    let interfaces = arp::nic::get_interfaces();

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
