use clap::Parser;
use cli::{Args, Commands};
use pnet::datalink;

pub mod cli;

fn main() {
    let args = Args::parse();
    
    match args.cmd {
        Commands::Interfaces => {
            let interfaces = datalink::interfaces();
            for interface in interfaces {
                println!("{} @ {}", interface.name, match interface.mac { Some(addr) => addr.to_string(), None => "None".to_string()});
            }
        }
    }
}
