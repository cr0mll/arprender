use std::net::Ipv4Addr;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author = "cr0mll")]
#[command(version)]
#[command(propagate_version = true)]
#[command(about = "A suite for advanced ARP-based attacks.")]
#[command(help_template = "Arprender v{version}\n{about-with-newline}{author} (C) cr0mll@protonmail.com \n\n{usage-heading} {usage}\n\n{all-args}")]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Commands
}

#[derive(Subcommand, Debug, Clone)]
#[command(author = "cr0mll")]
#[command(version)]
pub enum Commands {
    #[command(help_template = "Arprender v{version}\n{about-with-newline}cr0mll (C) cr0mll@protonmail.com \n\n{usage-heading} {usage}\n\n{all-args}")]
    /// Lists the available network interfaces.
    Interfaces,
    
    #[command(help_template = "Arprender v{version}\n{about-with-newline}cr0mll (C) cr0mll@protonmail.com \n\n{usage-heading} {usage}\n\n{all-args}")]
    /// Performs an ARP scan of the network.
    Scan {
        /// The network interface to use for the scan. 
        interface: String
    },

    #[command(help_template = "Arprender v{version}\n{about-with-newline}cr0mll (C) cr0mll@protonmail.com \n\n{usage-heading} {usage}\n\n{all-args}")]
    /// Resolves a single IP address to its corresponding MAC address via ARP.
    Resolve {
        /// The IP address to resolve.
        address: Ipv4Addr,

        /// The interface to use for the address resolution.
        interface: String
    },

    #[command(help_template = "Arprender v{version}\n{about-with-newline}cr0mll (C) cr0mll@protonmail.com \n\n{usage-heading} {usage}\n\n{all-args}")]
    /// Impersonates the target host by tricking all devices on the network to forward all traffic intended for the target to you.
    Impersonate {
        /// The IP address of the target.
        #[arg(required = true)]
        target: Ipv4Addr,

        /// The interface to use for the attack.
        #[arg(required = true)]
        interface: String,

        #[arg(default_value = "false", long, short)]
        /// Attempt a stealthier, but less reliable, form of the attack.
        stealthy: bool,

        /// The interval (in seconds) at which to repeat the attack in order to ensure it remains effective.
        #[arg(default_value = "5", long, short)]
        period: u16
    },
}