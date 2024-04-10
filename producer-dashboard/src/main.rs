// use ledger::Ledger;

mod ledger;
mod node;

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

    let best_tip = node::get_best_tip().await.unwrap();

    println!(
        "Block Height: {}, Epoch: {}, Epoch length(current): {}, Epoch length(next): {}",
        best_tip.consensus_state().block_height,
        best_tip.consensus_state().epoch,
        best_tip.consensus_state().staking_epoch_data.epoch_length,
        best_tip.consensus_state().next_epoch_data.epoch_length,
    )
}
