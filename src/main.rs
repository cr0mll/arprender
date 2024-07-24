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
        Commands::Interfaces =>commands::interfaces(),
        Commands::Scan {
            interface,
        } => commands::scan(interface, 5),
        Commands::Resolve { interface, address } => commands::resolve(interface, address, 10),
        Commands::Impersonate {
            target,
            interface,
            stealthy,
            period,
        } => commands::impersonate(interface, target, stealthy, period)
    }
}
