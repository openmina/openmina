use std::collections::BTreeSet;

use crate::{archive::Block, evaluator::epoch::SlotStatus, storage::db_sled::Database, NodeStatus};

use super::ArchiveConnector;
use crate::archive::ChainStatus;
use tokio::{task::JoinHandle, time::Duration};
use tracing::{debug, error, info, instrument, trace};

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

            // Check if we have the current slot and a slot before evaluated (doing the slots in batches,
            // not sequentially, so the older slot could be missing but the current slot is already evaluated)
            if self
                .db
                .has_evaluated_slot(current_slot.global_slot().to_u32())
                .unwrap_or_default()
                && self
                    .db
                    .has_evaluated_slot(current_slot.global_slot().to_u32() - 1)
                    .unwrap_or_default()
            {
                debug!(
                    "Setting current slot to {}",
                    current_slot.global_slot().to_u32()
                );
                let old = current_slot.global_slot().to_u32() - 1;
                self.db
                    .set_current_slot(old, current_slot.global_slot().to_u32())
                    .unwrap();
            } else {
                debug!(
                    "Current slot [{}] not yet evaluated",
                    current_slot.global_slot().to_u32()
                );
            }

            if let Some(best_tip) = node_status.best_tip() {
                let (start, end) = best_tip.epoch_bounds().0;
                let (start_since_genesis, end_since_genesis) = (
                    node_status.to_global_slot_since_genesis(start),
                    node_status.to_global_slot_since_genesis(end),
                );

                info!("Best tip epoch bounds: {start} - {end} -- Since genesis {start_since_genesis} - {end_since_genesis}");
                info!("Best tip state hash: {}", best_tip.state_hash());

                let cannonical_chain = match self
                    .archive_connector
                    .clone()
                    .get_canonical_chain(
                        start_since_genesis.into(),
                        end_since_genesis.into(),
                        best_tip.state_hash(),
                    )
                    .await
                {
                    Ok(blocks) => blocks,
                    Err(e) => {
                        error!("{e}");
                        continue;
                    }
                };

                if cannonical_chain.is_empty() {
                    error!("Retrieved cannonical chain from archive DB is empty! Bounded by slots {start} - {end}");
                    continue;
                }

                debug!(
                    "Canonical chain in epoch bounds lenght: {}",
                    cannonical_chain.len()
                );

                trace!("Canonical chain from archive DB: {:?}", {
                    cannonical_chain
                        .clone()
                        .into_iter()
                        .filter(|b| b.creator_key == self.producer_pk)
                        .map(|b| b.state_hash)
                        .collect::<Vec<_>>()
                });

                let (canonical_pending, canonical): (Vec<Block>, Vec<Block>) = cannonical_chain
                    .into_iter()
                    .partition(|block| block.height >= (best_tip.height() - 290) as i64);

                // get blocks
                let blocks = match self
                    .archive_connector
                    .get_blocks_in_slot_range(start_since_genesis.into(), end_since_genesis.into())
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

                // Optimize: count all statuses in one iteration
                let (our_canonical, our_orphaned, our_pending) =
                    our_blocks
                        .iter()
                        .fold(
                            (0, 0, 0),
                            |(canonical, orphaned, pending), block| match block.chain_status {
                                ChainStatus::Canonical => (canonical + 1, orphaned, pending),
                                ChainStatus::Orphaned => (canonical, orphaned + 1, pending),
                                ChainStatus::Pending => (canonical, orphaned, pending + 1),
                            },
                        );

                debug!(
                    "Our blocks: [Canonical: {}, Orphaned: {}, Pending: {}]",
                    our_canonical, our_orphaned, our_pending
                );

                our_blocks.iter().for_each(|block| {
                    let slot = block.global_slot_since_hard_fork as u32;
                    let mut categorized_slots: BTreeSet<u32> = BTreeSet::new();
                    if self
                        .db
                        .seen_block(block.state_hash.clone())
                        .ok()
                        .unwrap_or_default()
                    {

                        // already categorized a slot, but we saw anoter block, if we have already saw the canonical
                        // block for that slow, we ignore
                        if categorized_slots.contains(&slot) && self.db.has_canonical_block_on_slot(slot).unwrap_or_default() {
                            return;
                        }

                        if canonical.contains(block) {
                            self.db
                                .update_slot_status(slot, SlotStatus::Canonical, &block.state_hash)
                                .unwrap();
                            debug!("{} -> Canonical", block.state_hash);
                        } else if canonical_pending.contains(block) {
                            self.db
                                .update_slot_status(slot, SlotStatus::CanonicalPending, &block.state_hash)
                                .unwrap();
                            debug!("{} -> CanonicalPending", block.state_hash);
                        } else if block.height >= (best_tip.height() - 290) as i64 {
                            self.db
                                .update_slot_status(slot, SlotStatus::OrphanedPending, &block.state_hash)
                                .unwrap();
                            debug!("{} -> OrphanedPending", block.state_hash);
                        } else {
                            self.db
                                .update_slot_status(slot, SlotStatus::Orphaned, &block.state_hash)
                                .unwrap();
                            debug!("{} -> Orphaned", block.state_hash);
                        }
                        categorized_slots.insert(slot);
                    } else if self.db.has_evaluated_slot(slot).unwrap_or_default() {
                        if !self.db.has_canonical_block_on_slot(slot).unwrap_or_default() {
                            info!("Saw produced block: {}", block.state_hash);
                            self.db.store_block(block.clone()).unwrap();
                            self.db
                                .update_slot_block(slot, block.into(), true, false)
                                .unwrap();
                        } else {
                            info!("Saw produced block: {}, but we already have a canonical block, ignoring", block.state_hash);
                        }
                    }
                });

                other_blocks.iter().for_each(|block| {
                    let slot = block.global_slot();
                    if self.db.has_evaluated_slot(slot).ok().unwrap_or_default()
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
