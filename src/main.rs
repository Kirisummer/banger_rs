use std::fs;

use toml::Table;

mod bang_storage;
use crate::bang_storage::BangStorage;

mod request;

mod server;
use crate::server::serve;

mod response;

fn main() -> Result<(), String> {
    // Read CLI arguments
    let config_path = std::env::args().nth(1).ok_or("Config path not given")?;
    let listen_address = std::env::args()
        .nth(2)
        .ok_or("Address and port not given")?;

    // Parse config
    let content =
        fs::read_to_string(&config_path).map_err(|err| format!("{config_path}: {err}"))?;
    let table = content.parse::<Table>().map_err(|err| format!("{err}"))?;
    let storage = BangStorage::from_table(&table)?;

    // Serve
    serve(storage, &listen_address)
}
