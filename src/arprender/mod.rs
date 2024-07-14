use core::fmt;
use std::net::Ipv4Addr;
use std::time::{Instant, Duration};

use nic::{InterfaceError, NetworkInterface};
use pnet::util::MacAddr;

use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, ArpPacket, MutableArpPacket};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket};

pub mod nic;

const ETHERNET_SIZE: usize = EthernetPacket::minimum_packet_size();
const ARP_OFFSET: usize = ETHERNET_SIZE;
const ARP_SIZE: usize = ArpPacket::minimum_packet_size();

/// Sends an ARP request for the specified destination IP through the given interface.
pub fn send_arp_request(
    interface: &NetworkInterface,
    dest_ip: Ipv4Addr,
    source_mac: Option<MacAddr>,
    source_ip: Option<Ipv4Addr>,
) -> Result<(), InterfaceError> {
    // Process the source MAC and IP addresses for spoofing
    let source_mac = if source_mac.is_some() {
        source_mac.unwrap()
    } else {
        if interface.mac().is_some() {
            interface.mac().unwrap()
        } else {
            return Err(InterfaceError::MissingMAC);
        }
    };

    let source_ip = if source_ip.is_some() {
        source_ip.unwrap()
    } else {
        if interface.ipv4_address().is_some() {
            interface.ipv4_address().unwrap()
        } else {
            return Err(InterfaceError::MissingIP);
        }
    };

    let mut packet_buf = [0u8; ETHERNET_SIZE + ARP_SIZE];

    let mut eth_layer = MutableEthernetPacket::new(&mut packet_buf).unwrap();

    eth_layer.set_destination(MacAddr::broadcast());
    eth_layer.set_source(source_mac);
    eth_layer.set_ethertype(EtherTypes::Arp);

    let mut arp_layer = MutableArpPacket::new(&mut packet_buf[ARP_OFFSET..]).unwrap();

    arp_layer.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_layer.set_protocol_type(EtherTypes::Ipv4);
    arp_layer.set_hw_addr_len(6);
    arp_layer.set_proto_addr_len(4);
    arp_layer.set_operation(ArpOperations::Request);
    arp_layer.set_sender_hw_addr(source_mac);
    arp_layer.set_sender_proto_addr(source_ip);
    arp_layer.set_target_hw_addr(MacAddr::zero());
    arp_layer.set_target_proto_addr(dest_ip);

    use pnet::datalink::*;

    let mut tx = match channel(&interface.clone().into(), Default::default()) {
        Ok(Channel::Ethernet(tx, _)) => tx,
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!(
            "An error occurred when creating the datalink channel: {}",
            e
        ),
    };

    tx.send_to(&packet_buf, None);

    Ok(())
}

/// Attempts to find the MAC associated with the given IP address
pub fn resolve_mac(interface: &NetworkInterface, dest_ip: Ipv4Addr, timeout: Duration) -> Result<Option<MacAddr>, InterfaceError> {
    
    use pnet::datalink::*;

    let mut rx = match channel(&interface.clone().into(), Default::default()) {
        Ok(Channel::Ethernet(_, rx)) => rx,
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!(
            "An error occurred when creating the datalink channel: {}",
            e
        ),
    };
    send_arp_request(interface, dest_ip, interface.mac(), interface.ipv4_address())?;

    let start = Instant::now();

    loop {
        let buf = rx.next().unwrap();

        if buf.len() < ETHERNET_SIZE + ARP_SIZE {
            if is_timeout_expired(start, timeout)
            {
                break; 
            }
            continue;
        }

        let arp_layer = ArpPacket::new(&buf[ARP_OFFSET..]).unwrap();

        if arp_layer.get_sender_proto_addr() == dest_ip
            && arp_layer.get_target_hw_addr() == interface.mac().unwrap()
        {
            return Ok(Some(arp_layer.get_sender_hw_addr()));
        }

        if is_timeout_expired(start, timeout) { break ;}
    }

    Ok(None)
}

fn is_timeout_expired(start: Instant, timeout: Duration ) -> bool {
    Instant::now().duration_since(start) > timeout 
}