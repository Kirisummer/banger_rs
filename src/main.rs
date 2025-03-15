use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;

use clap::Parser;
use toml::Table;

mod config_lookup;
use crate::config_lookup::ConfigLookup;

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
    config: Option<PathBuf>,
    /// Address and port to bind to in <IP address>:<port> format
    #[arg(short, long)]
    address: Option<SocketAddr>,
}

fn get_address_from_config(table: &Table) -> Result<SocketAddr, String> {
    let value = table
        .get("address")
        .ok_or("Address is missing from config".to_string())?;
    let addr_str = value
        .as_str()
        .ok_or(format!("Address is not a string: {:?}", value))?;
    let addr = addr_str
        .parse::<SocketAddr>()
        .map_err(|_err| format!("Failed to parse {addr_str} into socket address"))?;
    Ok(addr)
}

fn main() -> Result<(), String> {
    // Read CLI arguments
    let args = Args::parse();

    // Parse config
    let config_path = ConfigLookup::new(args.config)
        .lookup()
        .ok_or("Failed to find config".to_string())?;
    eprintln!("Reading config from {}", config_path.display());

    let content = fs::read_to_string(&config_path)
        .map_err(|err| format!("{}: {}", config_path.display(), err))?;
    let table = content.parse::<Table>().map_err(|err| format!("{err}"))?;
    let storage = BangStorage::from_table(&table)?;

    // Serve
    let listen_address = match args.address {
        Some(addr) => addr,
        None => get_address_from_config(&table)?,
    };
    eprintln!("Listening on {listen_address}");
    serve(storage, listen_address)
}
