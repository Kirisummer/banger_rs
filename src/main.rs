use std::fs;
use std::path::PathBuf;
use std::net::SocketAddr;

use clap::Parser;
use toml::Table;

mod bang_storage;
use crate::bang_storage::BangStorage;

mod request;

mod server;
use crate::server::serve;

mod response;

#[derive(Parser, Debug)]
struct Args {
    /// Path of config file for banger
    #[arg(short, long)]
    config: PathBuf,
    /// Address and port to bind to in <IP address>:<port> format
    #[arg(short, long)]
    address: SocketAddr,
}

fn main() -> Result<(), String> {
    // Read CLI arguments
    let args = Args::parse();
    let config_path = args.config;
    let listen_address = args.address;

    // Parse config
    let content =
        fs::read_to_string(&config_path).map_err(|err| format!("{}: {}", config_path.display(), err))?;
    let table = content.parse::<Table>().map_err(|err| format!("{err}"))?;
    let storage = BangStorage::from_table(&table)?;

    // Serve
    serve(storage, listen_address)
}
