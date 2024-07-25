use clap::Parser;
use cli::{Args, Commands};

pub mod arp;
#[macro_use]
pub mod commands;
pub mod cli;
#[macro_use]
pub mod utils;

fn main() {
    let args = Args::parse();

    match args.cmd {
        Commands::Interfaces => commands::interfaces(),
        Commands::Scan { interface, timeout } => commands::scan(interface, timeout),
        Commands::Resolve { interface, address, timeout } => commands::resolve(interface, address, timeout),
        Commands::Impersonate {
            interface,
            target,
            stealthy,
            period,
        } => commands::impersonate(interface, target, stealthy, period),
    }
}
