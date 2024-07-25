use std::net::Ipv4Addr;
use std::thread;
use std::time::Duration;

use nic::{InterfaceError, NetworkInterface};
use pnet::util::MacAddr;

use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, ArpPacket, MutableArpPacket};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket};

use crate::utils::run_with_timeout;

pub mod nic;

const ETHERNET_SIZE: usize = EthernetPacket::minimum_packet_size();
const ARP_OFFSET: usize = ETHERNET_SIZE;
const ARP_SIZE: usize = ArpPacket::minimum_packet_size();

/// Sends an ARP request.
///
/// # Arguments
///
/// * `interface` - The network interface to use for the transmission.
/// * `target_proto_addr` - The destination IP address of the ARP request.
/// * `sender_mac_addr` - An optional MAC address to use as the source MAC for the ARP request. If `None` is specified, then the interface's MAC address is used.
/// * `sender_proto_addr` - An optional IP address to use as the source IP for the ARP request. If `None` is specified, then the interface's IP address is used.
///
/// Note: This function does not await a response. To resolve an IP address, use resolve_ip.
pub fn send_arp_request(
    interface: &NetworkInterface,
    target_proto_addr: Ipv4Addr,
    sender_hw_addr: Option<MacAddr>,
    sender_proto_addr: Option<Ipv4Addr>,
) -> Result<(), InterfaceError> {
    // Process the source MAC and IP addresses for spoofing
    let source_mac = if sender_hw_addr.is_some() {
        sender_hw_addr.unwrap()
    } else {
        if interface.mac().is_some() {
            interface.mac().unwrap()
        } else {
            return Err(InterfaceError::MissingMAC);
        }
    };

    let source_ip = if sender_proto_addr.is_some() {
        sender_proto_addr.unwrap()
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
    arp_layer.set_target_proto_addr(target_proto_addr);

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

/// Sends an ARP response packet.
///
/// # Arguments
///
/// * `interface` - The network interface to use for the transmission.
pub fn send_arp_reply(
    interface: &NetworkInterface,
    target_hw_addr: MacAddr,
    target_proto_addr: Ipv4Addr,
    sender_hw_addr: Option<MacAddr>,
    sender_proto_addr: Option<Ipv4Addr>,
) -> Result<(), InterfaceError> {
    // Process the source MAC and IP addresses for spoofing
    let source_mac = if sender_hw_addr.is_some() {
        sender_hw_addr.unwrap()
    } else {
        if interface.mac().is_some() {
            interface.mac().unwrap()
        } else {
            return Err(InterfaceError::MissingMAC);
        }
    };

    let source_ip = if sender_proto_addr.is_some() {
        sender_proto_addr.unwrap()
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
    arp_layer.set_operation(ArpOperations::Reply);
    arp_layer.set_sender_hw_addr(source_mac);
    arp_layer.set_sender_proto_addr(source_ip);
    arp_layer.set_target_hw_addr(target_hw_addr);
    arp_layer.set_target_proto_addr(target_proto_addr);

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
pub fn resolve_mac(
    interface: NetworkInterface,
    dest_ip: Ipv4Addr,
    timeout: Duration,
) -> Result<Option<MacAddr>, InterfaceError> {
    use pnet::datalink::*;

    let mut rx = match channel(&interface.clone().into(), Default::default()) {
        Ok(Channel::Ethernet(_, rx)) => rx,
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!(
            "An error occurred when creating the datalink channel: {}",
            e
        ),
    };
    send_arp_request(
        &interface,
        dest_ip,
        interface.mac(),
        interface.ipv4_address(),
    )?;

    match run_with_timeout(
        move || loop {
            let buf = rx.next().unwrap();

            if buf.len() < ETHERNET_SIZE + ARP_SIZE {
                continue;
            }

            let arp_layer = ArpPacket::new(&buf[ARP_OFFSET..]).unwrap();

            if arp_layer.get_sender_proto_addr() == dest_ip
                && arp_layer.get_target_hw_addr() == interface.mac().unwrap()
            {
                return arp_layer.get_sender_hw_addr();
            }
        },
        timeout,
    ) {
        Ok(addr) => Ok(Some(addr)),
        Err(_) => Ok(None),
    }
}

pub fn arp_scan(
    interface: &NetworkInterface,
    timeout: Duration,
) -> Result<Vec<(Ipv4Addr, MacAddr)>, InterfaceError> {
    let Some(network) = interface.network() else {
        return Err(InterfaceError::MissingIP);
    };

    let Some(interface_mac) = interface.mac() else {
        return Err(InterfaceError::MissingMAC);
    };

    use pnet::datalink::*;

    let mut packet_rx = match channel(&interface.clone().into(), Default::default()) {
        Ok(Channel::Ethernet(_, rx)) => rx,
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!(
            "An error occurred when creating the datalink channel: {}",
            e
        ),
    };

    let (hosts_tx, mut hosts_rx) = tokio::sync::mpsc::channel::<(Ipv4Addr, MacAddr)>(64);

    run_with_timeout(
        {
            let interface = interface.clone();
            let hosts_tx = hosts_tx.downgrade();

            move || {
                // Start the receiver thread
                let listener = thread::spawn(move || loop {
                    let buf = packet_rx.next().unwrap();

                    if buf.len() < ETHERNET_SIZE + ARP_SIZE {
                        continue;
                    }

                    let arp_layer = ArpPacket::new(&buf[ARP_OFFSET..]).unwrap();

                    if arp_layer.get_target_hw_addr() == interface_mac {
                        hosts_tx.upgrade().unwrap()
                            .blocking_send((
                                arp_layer.get_sender_proto_addr(),
                                arp_layer.get_sender_hw_addr(),
                            ))
                            .unwrap();
                    }
                });

                for ip in network.into_iter() {
                    send_arp_request(&interface, ip, None, None).unwrap();
                }

                listener.join().unwrap();
            }
        },
        timeout,
    )
    .ok();

    drop(hosts_tx);
    let mut hosts: Vec<(Ipv4Addr, MacAddr)> = Vec::new();

    while let Some(host) = hosts_rx.blocking_recv() {
        hosts.push(host);
    }

    Ok(hosts)
}
