use std::path::PathBuf;

use clap::Parser;

/// Awesome producer proxy
///
/// TODO
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// RPC port for the proxy
    #[arg(short, long, env, default_value_t = 3000)]
    pub rpc_port: u16,
    // TODO(adonagy)
    /// Path to the producer's private key file
    ///
    /// MINA_PRIVKEY_PASS environmental variable must be set!
    #[arg(short, long, env)]
    pub producer_key: PathBuf,

    /// Path to the dashboard's database
    #[arg(short, long, env, default_value = "/tmp/producer-dashboard")]
    pub database_path: PathBuf,

    /// Node's graphql endpoint URL
    #[arg(short, long, env, default_value = "http://mina:5000/graphql")]
    pub node_url: String,
}
