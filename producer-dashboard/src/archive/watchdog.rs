use crate::storage::db_sled::Database;

use super::{ArchiveConnector, ChainStatus};
use tokio::{task::JoinHandle, time::Duration};

pub struct ArchiveWatchdog {
    producer_pk: String,
    archive_connector: ArchiveConnector,
    db: Database,
}

impl ArchiveWatchdog {
    pub fn spawn_new(db: Database, producer_pk: String) -> JoinHandle<()> {
        tokio::spawn(async move {
            Self {
                producer_pk,
                archive_connector: ArchiveConnector::connect().await,
                db,
            }
            .run()
            .await;
        })
    }

    async fn run(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            interval.tick().await;

            println!("[archive-watchdog] Tick");

            // get blocks
            let blocks = match self
                .archive_connector
                .get_producer_blocks(&self.producer_pk)
                .await
            {
                Ok(blocks) => blocks,
                Err(e) => {
                    eprintln!("{e}");
                    continue;
                }
            };

            blocks.iter().for_each(|block| {
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
                    }
                } else if self.db.has_slot(slot).unwrap_or_default() {
                    println!("[archive] saw produced block: {}", block.state_hash);
                    self.db.store_block(block.state_hash.clone(), slot).unwrap();
                    self.db.update_slot_block(slot, block.into()).unwrap();
                }
            });
        }
    }
}
