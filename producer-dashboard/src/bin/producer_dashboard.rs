use openmina_node_account::AccountSecretKey;
use openmina_producer_dashboard::{
    archive::watchdog::ArchiveWatchdog,
    config,
    evaluator::{EpochInit, Evaluator},
    node::{watchdog::spawn_watchdog, Node},
    rpc,
    storage::db_sled::Database,
    NodeStatus,
};

use tokio::sync::mpsc;

use clap::Parser;

#[tokio::main]
async fn main() {
    let config = config::Config::parse();

    let db = Database::open(config.database_path).expect("Failed to open Database");
    println!("[main] DB opened");

    let password = std::env::var("MINA_PRIVKEY_PASS")
        .expect("Expected password in the variable `MINA_PRIVKEY_PASS`");

    let key = AccountSecretKey::from_encrypted_file(config.private_key_path, &password)
        .expect("failed to decrypt secret key file");
    println!("[main] Producer key loaded");

    let (sender, receiver) = mpsc::unbounded_channel::<EpochInit>();

    let evaluator_handle = Evaluator::spawn_new(key.clone(), db.clone(), receiver);
    println!("[main] Evaluator created");
    let node_status = NodeStatus::default();
    let node = Node::new(config.node_url);
    let node_watchdog = spawn_watchdog(
        node,
        node_status.clone(),
        db.clone(),
        sender,
        key.public_key().to_string(),
    );
    println!("[main] Node watchdog created");
    let archive_watchdog = ArchiveWatchdog::spawn_new(
        db.clone(),
        key.public_key().to_string(),
        node_status.clone(),
    );
    println!("[main] Archive watchdog created");

    let rpc_handle = rpc::spawn_rpc_server(
        3000,
        db.clone(),
        node_status.clone(),
        key.public_key().to_string(),
    );
    println!("[main] RPC server created");

    let mut signal_stream =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Unable to handle SIGTERM");

    tokio::select! {
        s = tokio::signal::ctrl_c() => {
            s.expect("Failed to listen for ctrl-c event");
            println!("Ctrl-c or SIGINT received!");
        }
        _ = signal_stream.recv() => {
            println!("SIGTERM received!");
        }
    }

    drop(archive_watchdog);
    drop(rpc_handle);
    drop(node_watchdog);
    drop(evaluator_handle);
    println!("Shutdown successfull");
}
