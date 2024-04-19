use crate::{archive::Block, evaluator::epoch::SlotStatus, storage::db_sled::Database, NodeStatus};

use super::{ArchiveConnector, ChainStatus};
use tokio::{task::JoinHandle, time::Duration};

pub struct ArchiveWatchdog {
    producer_pk: String,
    archive_connector: ArchiveConnector,
    db: Database,
    node_status: NodeStatus,
}

impl ArchiveWatchdog {
    pub fn spawn_new(db: Database, producer_pk: String, node_status: NodeStatus) -> JoinHandle<()> {
        tokio::spawn(async move {
            Self {
                producer_pk,
                archive_connector: ArchiveConnector::connect().await,
                db,
                node_status,
            }
            .run()
            .await;
        })
    }

    // TODO(adonagy): cleanup this mess...
    async fn run(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            interval.tick().await;

            println!("[archive-watchdog] Tick");
            let node_status = self.node_status.read().await.clone();

            let current_slot = node_status.current_slot();

            if self
                .db
                .has_slot(current_slot.global_slot().to_u32())
                .unwrap()
            {
                let old = current_slot.global_slot().to_u32() - 1;
                self.db
                    .set_current_slot(old, current_slot.global_slot().to_u32())
                    .unwrap();
            }

            if let Some(best_tip) = node_status.best_tip() {
                let (start, end) = best_tip.epoch_bounds().0;
                // get blocks
                let blocks = match self
                    .archive_connector
                    .get_blocks_in_slot_range(start.into(), end.into())
                    .await
                {
                    Ok(blocks) => blocks,
                    Err(e) => {
                        eprintln!("{e}");
                        continue;
                    }
                };

                let (our_blocks, other_blocks): (Vec<Block>, Vec<Block>) = blocks
                    .into_iter()
                    .partition(|block| block.creator_key == self.producer_pk);

                our_blocks.iter().for_each(|block| {
                    let slot = block.global_slot_since_hard_fork as u32;
                    if self
                        .db
                        .seen_block(block.state_hash.clone())
                        .ok()
                        .unwrap_or_default()
                    {
                        if !matches!(block.chain_status, ChainStatus::Pending) {
                            self.db
                                .update_slot_status(slot, block.chain_status.clone().into())
                                .unwrap();
                        } else {
                            let best_chain = node_status.best_chain();
                            if best_chain.contains(&(slot, block.state_hash.clone())) {
                                self.db
                                    .update_slot_status(slot, SlotStatus::CanonicalPending)
                                    .unwrap();
                            } else {
                                self.db
                                    .update_slot_status(slot, SlotStatus::OrphanedPending)
                                    .unwrap();
                            }
                        }
                    } else if self.db.has_slot(slot).unwrap_or_default() {
                        println!("[archive] saw produced block: {}", block.state_hash);
                        self.db.store_block(block.clone()).unwrap();
                        // TODO: access transition frontier here
                        self.db
                            .update_slot_block(slot, block.into(), true, false)
                            .unwrap();
                    }
                });

                other_blocks.iter().for_each(|block| {
                    let slot = block.global_slot();
                    if self.db.has_slot(slot).ok().unwrap_or_default()
                        && !self.db.seen_slot(slot).ok().unwrap_or_default()
                    {
                        if slot < current_slot.global_slot().to_u32() {
                            self.db
                                .update_slot_block(slot, block.into(), false, false)
                                .unwrap();
                        } else {
                            self.db
                                .update_slot_block(slot, block.into(), false, true)
                                .unwrap();
                        }
                    }
                });
            }
        }
    }
}
