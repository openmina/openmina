use openmina_node_account::AccountSecretKey;

use tokio::sync::mpsc;

use clap::Parser;

use producer_dashboard::{
    archive::watchdog::ArchiveWatchdog,
    evaluator::{EpochInit, Evaluator},
    node::{watchdog::spawn_watchdog, Node},
    storage::db_sled::Database,
    NodeStatus,
};

use tracing::{error, info, instrument};

#[tokio::main]
#[instrument]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let config = producer_dashboard::config::Config::parse();

    if config.force_recreate_db_unsafe {
        std::fs::remove_dir_all(&config.database_path).expect("Failed deleting databse dir");
    }

    let db = match Database::open(config.database_path) {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to open Database: {}", e);
            return;
        }
    };
    info!("DB opened");

    if config.force_recreate_db {
        db.clear().expect("Failed to clear DB");
    }

    let password = std::env::var("MINA_PRIVKEY_PASS")
        .expect("Expected password in the variable `MINA_PRIVKEY_PASS`");

    let key = match AccountSecretKey::from_encrypted_file(config.producer_key, &password) {
        Ok(key) => key,
        Err(e) => {
            error!("Failed to decrypt secret key file: {}", e);
            return;
        }
    };
    info!("Producer key loaded");

    let (sender, receiver) = mpsc::unbounded_channel::<EpochInit>();

    let evaluator_handle = Evaluator::spawn_new(key.clone(), db.clone(), receiver);
    info!("Evaluator created");
    let node_status = NodeStatus::default();
    let node = Node::new(config.node_graphql_url, config.node_client_url);
    let node_watchdog = spawn_watchdog(
        node,
        node_status.clone(),
        db.clone(),
        sender,
        key.public_key().to_string(),
    );
    info!("Node watchdog created");
    let archive_watchdog = ArchiveWatchdog::spawn_new(
        db.clone(),
        key.public_key().to_string(),
        node_status.clone(),
    );
    info!("Archive watchdog created");

    let rpc_handle = producer_dashboard::rpc::spawn_rpc_server(
        3000,
        db.clone(),
        node_status.clone(),
        key.public_key().to_string(),
    );
    info!("RPC server created");

    let mut signal_stream =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Unable to handle SIGTERM");

    tokio::select! {
        s = tokio::signal::ctrl_c() => {
            if let Err(e) = s {
                error!("Failed to listen for ctrl-c event: {}", e);
            } else {
                info!("Ctrl-c or SIGINT received!");
            }
        }
        _ = signal_stream.recv() => {
            info!("SIGTERM received!");
        }
    }

    drop(archive_watchdog);
    drop(rpc_handle);
    drop(node_watchdog);
    drop(evaluator_handle);
    info!("Shutdown successful");
}
