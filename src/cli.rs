use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author = "cr0mll", version = "0.1.0", about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Commands
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Lists the available interfaces
    Interfaces
}