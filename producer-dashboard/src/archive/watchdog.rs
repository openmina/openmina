use crate::{archive::Block, evaluator::epoch::SlotStatus, storage::db_sled::Database, NodeStatus};

use super::ArchiveConnector;
use tokio::{task::JoinHandle, time::Duration};
use tracing::{error, info, instrument, trace};

#[derive(Debug)]
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
    #[instrument(name = "Archive watchdog", skip_all)]
    async fn run(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            interval.tick().await;

            trace!("Tick");
            let node_status = self.node_status.read().await.clone();

            let current_slot = node_status.current_slot();

            if self
                .db
                .has_won_slot(current_slot.global_slot().to_u32())
                .unwrap()
            {
                let old = current_slot.global_slot().to_u32() - 1;
                self.db
                    .set_current_slot(old, current_slot.global_slot().to_u32())
                    .unwrap();
            }

            if let Some(best_tip) = node_status.best_tip() {
                let (start, end) = best_tip.epoch_bounds().0;

                let cannonical_chain = match self
                    .archive_connector
                    .clone()
                    .get_canonical_chain(start.into(), end.into(), best_tip.state_hash())
                    .await
                {
                    Ok(blocks) => blocks,
                    Err(e) => {
                        error!("{e}");
                        continue;
                    }
                };

                let (canonical_pending, canonical): (Vec<Block>, Vec<Block>) = cannonical_chain
                    .into_iter()
                    .partition(|block| block.height >= (best_tip.height() - 290) as i64);

                // get blocks
                let blocks = match self
                    .archive_connector
                    .get_blocks_in_slot_range(start.into(), end.into())
                    .await
                {
                    Ok(blocks) => blocks,
                    Err(e) => {
                        error!("{e}");
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
                        if canonical.contains(block) {
                            self.db
                                .update_slot_status(slot, SlotStatus::Canonical)
                                .unwrap();
                        } else if canonical_pending.contains(block) {
                            self.db
                                .update_slot_status(slot, SlotStatus::CanonicalPending)
                                .unwrap();
                        } else if block.height >= (best_tip.height() - 290) as i64 {
                            self.db
                                .update_slot_status(slot, SlotStatus::OrphanedPending)
                                .unwrap();
                        } else {
                            self.db
                                .update_slot_status(slot, SlotStatus::Orphaned)
                                .unwrap();
                        }
                    } else if self.db.has_won_slot(slot).unwrap_or_default() {
                        info!("Saw produced block: {}", block.state_hash);
                        self.db.store_block(block.clone()).unwrap();
                        self.db
                            .update_slot_block(slot, block.into(), true, false)
                            .unwrap();
                    }
                });

                other_blocks.iter().for_each(|block| {
                    let slot = block.global_slot();
                    if self.db.has_won_slot(slot).ok().unwrap_or_default()
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
