use std::path::PathBuf;

use clap::Parser;

/// Awesome producer proxy
///
/// TODO
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// RPC port for the proxy
    #[arg(short, long, default_value_t = 3000)]
    pub rpc_port: u16,
    // TODO(adonagy)
    /// Path to the producer's private key file
    ///
    /// MINA_PRIVKEY_PASS environmental variable must be set!
    #[arg(short, long)]
    pub private_key_path: PathBuf,

    /// Path to the database
    #[arg(short, long, default_value = "/tmp/producer-dashboard")]
    pub database_path: PathBuf,

    #[arg(short, long)]
    pub node_url: String,
}
