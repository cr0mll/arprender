use std::{
    net::Ipv4Addr,
    time::{Duration, Instant},
};

use pnet::{ipnetwork::Ipv4Network, util::MacAddr};
use rand::{Rng, RngCore};

pub fn is_timer_expired(start: Instant, timer: Duration) -> bool {
    Instant::now().duration_since(start) > timer
}

pub fn random_mac() -> MacAddr {
    let mut mac_bytes: [u8; 6] = [0, 0, 0, 0, 0, 0];
    rand::thread_rng().fill_bytes(&mut mac_bytes);
    MacAddr::new(
        mac_bytes[0],
        mac_bytes[1],
        mac_bytes[2],
        mac_bytes[3],
        mac_bytes[4],
        mac_bytes[5],
    )
}

pub fn random_ip() -> Ipv4Addr {
    let mut ip_bytes: [u8; 4] = [0, 0, 0, 0];
    rand::thread_rng().fill_bytes(&mut ip_bytes);
    Ipv4Addr::new(ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3])
}

/// Generates a random IP address within the given network range.
pub fn random_ip_in_network(net: &Ipv4Network) -> Ipv4Addr {
    net.nth(rand::thread_rng().gen_range(0..net.size()))
        .unwrap()
}

macro_rules! loop_attack {
    ($attack:ident, $period:ident) => {
        let mut start = Instant::now();
        $attack();
        if !$period.is_zero() {
            loop {
                if is_timer_expired(start, $period) {
                    start = Instant::now();

                    $attack();
                }
            }
        } else {
            loop {
                $attack();
            }
        }
    };
}
