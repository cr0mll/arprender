use core::fmt;
use std::net::Ipv4Addr;

#[derive(Debug, Clone)]
pub struct NetworkInterface {
    interface: pnet::datalink::NetworkInterface,
    ipv4_net: Option<pnet::ipnetwork::Ipv4Network>,
}

impl NetworkInterface {
    pub fn name(&self) -> &str {
        &self.interface.name
    }

    pub fn description(&self) -> &str {
        &self.interface.description
    }

    pub fn mac(&self) -> Option<pnet::datalink::MacAddr> {
        self.interface.mac
    }

    pub fn ipv4_address(&self) -> Option<Ipv4Addr> {
        match &self.ipv4_net {
            Some(network) => { Some(network.ip()) },
            None => None
        }
    }

    pub fn network_address(&self) -> Option<Ipv4Addr> {
        match &self.ipv4_net {
            Some(network) => Some(network.network()),
            None => None
        }
    }

    pub fn network(&self) -> Option<pnet::ipnetwork::Ipv4Network> {
        self.ipv4_net
    }
}


impl From<pnet::datalink::NetworkInterface> for NetworkInterface {
    fn from(value: pnet::datalink::NetworkInterface) -> Self {

        match value.ips.clone().into_iter().find(|net| net.is_ipv4()) {
            Some(ip_net) => Self {
                interface: value,
                ipv4_net: match ip_net {
                    pnet::ipnetwork::IpNetwork::V4(ipv4_net) => Some(ipv4_net),
                    pnet::ipnetwork::IpNetwork::V6(_) => None 
                }
            },
            None => Self {
                interface: value,
                ipv4_net: None
            }
        }
    }
}

impl Into<pnet::datalink::NetworkInterface> for NetworkInterface {
    fn into(self) -> pnet::datalink::NetworkInterface {
        self.interface
    }
}

pub fn get_interfaces() -> Vec<NetworkInterface> {
    pnet::datalink::interfaces().into_iter().map(|interface| interface.into()).collect()
}

pub fn get_interface_by_name(name: &str) -> Result<NetworkInterface, InterfaceError> {
    match get_interfaces().into_iter().
    filter(|iface: &NetworkInterface| iface.name().eq(name)).next() {
        Some(interface) => Ok(interface),
        None => Err(InterfaceError::InterfaceNotFound)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum InterfaceError {
    InterfaceNotFound,
    MissingIP,
    MissingMAC,
    NameAmbiguity,
    ChannelError
}

impl fmt::Display for InterfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::InterfaceNotFound => write!(f, "Could not find interface."),
            Self::NameAmbiguity => write!(f, "There is more than one interface with this name."),
            Self::MissingIP => write!(f, "This interface has no valid IPv4 address assigned."),
            Self::MissingMAC => write!(f, "This interface has no valid MAC address assigned."),
            Self::ChannelError => write!(f, "Failed to open transmission channels on the interface.")
        }
    }
}

impl std::error::Error for InterfaceError {}