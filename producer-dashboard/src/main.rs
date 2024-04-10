// use ledger::Ledger;

use std::collections::BTreeMap;

use clap::Parser;

use crate::epoch::EpochStorage;

mod ledger;
mod node;
mod storage;
mod epoch;
mod rpc;
mod config;

#[derive(Debug, thiserror::Error)]
pub enum StakingToolError {
    #[error("Empty graphql response")]
    EmptyGraphqlResponse,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

#[tokio::main]
async fn main() {
    // node::get_best_chain().await

    // // DEBUG loading ledger
    // let ledger = Ledger::load_from_file("producer-dashboard/staking-epoch-ledger.json".into()).unwrap();
    // let stuff = ledger.gather_producer_and_delegates("B62qmM4HnDHDwVXqBdCcXQJ9U2zAPNkhqmo8SNWAxKWoEok7GzQEkTv");
    // println!("Prod + del count: {}", stuff.len())

    let config = config::Config::parse();


    let best_tip = node::get_best_tip().await.unwrap();

    println!(
        "Block Height: {}, Epoch: {}, Epoch length(current): {}, Epoch length(next): {}",
        best_tip.consensus_state().block_height,
        best_tip.consensus_state().epoch,
        best_tip.consensus_state().staking_epoch_data.epoch_length,
        best_tip.consensus_state().next_epoch_data.epoch_length,
    );

    let epoch_storage = EpochStorage::default();

    let t_epoch_storage = epoch_storage.clone();
    let rpc_handle = rpc::spawn_rpc_server(3000, t_epoch_storage);

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

    drop(rpc_handle);
    println!("Shutdown successfull");
}
