use std::path::PathBuf;

use clap::Parser;

/// Awesome producer proxy
///
/// TODO
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// RPC port for the proxy
    #[arg(short, long, env = "OPENMINA_PDASH_RPC_PORT", default_value_t = 3000)]
    pub rpc_port: u16,
    // TODO(adonagy)
    /// Path to the producer's private key file
    ///
    /// MINA_PRIVKEY_PASS environmental variable must be set!
    #[arg(short, long, env = "OPENMINA_PDASH_PRODUCER_KEY")]
    pub producer_key: PathBuf,

    /// Path to the dashboard's database
    #[arg(
        short,
        long,
        env = "OPENMINA_PDASH_DATABASE_PATH",
        default_value = "/tmp/producer-dashboard"
    )]
    pub database_path: PathBuf,

    /// Forces the dashboard BE to start with a clean DB
    #[arg(
        short,
        long,
        env = "OPENMINA_PDASH_FORCE_RECREATE_DB",
        default_value_t = false
    )]
    pub force_recreate_db: bool,

    /// Deletes the dir at the databas_path, use carefully!
    ///
    /// Usefull when the shape of the DB changes
    ///
    /// Use carefully!
    #[arg(long, default_value_t = false)]
    pub force_recreate_db_unsafe: bool,

    /// Node's graphql endpoint URL
    #[arg(
        short,
        long,
        env = "OPENMINA_PDASH_NODE_GRAPHQL_URL",
        default_value = "http://mina:5000/graphql"
    )]
    pub node_graphql_url: String,

    /// Node's client access endpoint URL
    #[arg(
        short,
        long,
        env = "OPENMINA_PDASH_NODE_CLIENT_URL",
        default_value = "http://mina:8031"
    )]
    pub node_client_url: String,
}
