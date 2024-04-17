use node::NodeData;
use openmina_node_account::AccountSecretKey;

use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::{mpsc, RwLock, oneshot::error};

use clap::Parser;

use crate::{
    archive::watchdog::ArchiveWatchdog,
    evaluator::epoch::EpochStorage,
    evaluator::{EpochInit, Evaluator},
    node::{watchdog::spawn_watchdog, Node},
    storage::db_sled::Database,
};

mod archive;
mod config;
pub mod evaluator;
mod node;
mod rpc;
mod storage;

#[derive(Debug, thiserror::Error)]
pub enum StakingToolError {
    #[error("Empty graphql response")]
    EmptyGraphqlResponse,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("Node offline")]
    NodeOffline,
}

pub type NodeStatus = Arc<RwLock<NodeData>>;

#[tokio::main]
async fn main() {
    // node::get_best_chain().await

    // TODO(adonagy): periodically dump from the node
    // let ledger =
    //     Ledger::load_from_file("producer-dashboard/staking-epoch-ledger.json".into()).unwrap();
    // let stuff = ledger.gather_producer_and_delegates("B62qmM4HnDHDwVXqBdCcXQJ9U2zAPNkhqmo8SNWAxKWoEok7GzQEkTv");
    // println!("Prod + del count: {}", stuff.len())

    let config = config::Config::parse();

    let db = Database::open(config.database_path).expect("Failed to open Database");
    println!("[main] DB opened");

    let epoch_storage = EpochStorage::default();

    let key = AccountSecretKey::from_encrypted_file(config.private_key_path)
        .expect("failed to decrypt secret key file");
    println!("[main] Producer key loaded");

    let (sender, receiver) = mpsc::unbounded_channel::<EpochInit>();

    let evaluator_handle = Evaluator::spawn_new(key.clone(), db.clone(), receiver);
    println!("[main] Evaluator created");
    let node_status = NodeStatus::default();
    let node = Node::new(config.node_url);
    let node_watchdog = spawn_watchdog(node, node_status.clone(), db.clone(), sender);
    println!("[main] Node watchdog created");
    let archive_watchdog = ArchiveWatchdog::spawn_new(db.clone(), key.public_key().to_string());
    println!("[main] Archive watchdog created");

    let rpc_handle = rpc::spawn_rpc_server(3000, db.clone(), node_status.clone());
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
