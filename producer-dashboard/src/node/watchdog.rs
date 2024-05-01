use tokio::time::Duration;
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};

use crate::evaluator::EpochInit;
use crate::{storage::db_sled::Database, NodeStatus};

use super::daemon_status::SyncStatus;
use super::Node;

// TODO(adonagy): move to struct
pub fn spawn_watchdog(
    node: Node,
    status: NodeStatus,
    db: Database,
    sender: UnboundedSender<EpochInit>,
    producer_pk: String,
) -> JoinHandle<()> {
    tokio::spawn(async move { watch(node, status, db, sender, producer_pk).await })
}

async fn watch(
    node: Node,
    status: NodeStatus,
    db: Database,
    sender: UnboundedSender<EpochInit>,
    _producer_pk: String,
) {
    // TODO(adonagy): From config
    let mut interval = tokio::time::interval(Duration::from_secs(5));

    // TODO(adonagy): do not ignore this error
    if node.wait_for_graphql().await.is_err() {
        println!("[dbg] Error: returning");
        return;
    }

    let genesis_timestamp = node.get_genesis_timestmap().await.unwrap();

    {
        let mut node_status = status.write().await;
        node_status.genesis_timestamp = genesis_timestamp;
    }

    loop {
        interval.tick().await;

        println!("[node-watchdog] Tick");

        let sync_status = node.sync_status().await;

        if matches!(sync_status, SyncStatus::SYNCED) {
            // TODO(adonagy): Probably won't need 2 calls
            let best_tip = node.get_best_tip().await.unwrap();
            let best_chain = node.get_best_chain().await.unwrap();
            {
                let mut status: tokio::sync::RwLockWriteGuard<'_, super::NodeData> =
                    status.write().await;
                status.best_tip = Some(best_tip.clone());
                status.best_chain = best_chain.clone();
                status.sync_status = sync_status.clone()
            }
            let current_epoch: u32 = best_tip.consensus_state().epoch.parse().unwrap();

            if !db.has_ledger(&current_epoch).unwrap() {
                println!("Dumping staking ledger for epoch {current_epoch}");
                let ledger = Node::get_staking_ledger(current_epoch);
                let seed = best_tip.consensus_state().staking_epoch_data.seed.clone();
                // TODO(adonagy): handle error
                let _ = db.store_ledger(current_epoch, &ledger);
                let _ = db.store_seed(current_epoch, seed.clone());

                let mut status = status.write().await;

                status.dumped_ledgers.insert(current_epoch);

                let epoch_init = EpochInit::new(
                    current_epoch,
                    ledger,
                    seed,
                    best_tip.epoch_bounds().0,
                    genesis_timestamp,
                );
                // TODO(adonagy): handle error
                let _ = sender.send(epoch_init);
            }
        } else {
            println!("Node not synced");
        }
    }
}
