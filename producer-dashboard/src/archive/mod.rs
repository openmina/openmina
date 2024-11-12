use postgres_types::ChainStatus;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub mod postgres_types;
pub mod raw_types;
pub mod watchdog;

use raw_types::*;

#[derive(Debug, Clone)]
pub struct ArchiveConnector {
    pool: PgPool,
}

pub enum ArchiveUrl {
    Url(String),
    Env,
}

impl ArchiveConnector {
    pub async fn connect(postgres_url: ArchiveUrl) -> Self {
        let db_url = match postgres_url {
            ArchiveUrl::Url(url) => url,
            ArchiveUrl::Env => {
                if let Ok(url) = dotenvy::var("DATABASE_URL") {
                    url
                } else {
                    std::env::var("DATABASE_URL")
                        .expect("No db url found, check env var DATABASE_URL")
                }
            }
        };
        // TODO(adonagy): unwrap
        let pool = PgPool::connect(&db_url).await.unwrap();

        Self { pool }
    }

    pub async fn _get_producer_blocks(&self, producer_pk: &str) -> Result<Vec<Block>, sqlx::Error> {
        sqlx::query_file_as!(
            Block,
            "src/archive/sql/query_producer_blocks.sql",
            producer_pk
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_blocks_in_slot_range(
        &self,
        start_slot: i64,
        finish_slot: i64,
    ) -> Result<Vec<Block>, sqlx::Error> {
        sqlx::query_file_as!(
            Block,
            "src/archive/sql/query_blocks_in_slot_range.sql",
            start_slot,
            finish_slot
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_canonical_chain(
        &self,
        start_slot: i64,
        finish_slot: i64,
        best_tip_hash: String,
    ) -> Result<Vec<Block>, sqlx::Error> {
        sqlx::query_file_as!(
            Block,
            "src/archive/sql/query_canonical_chain.sql",
            best_tip_hash,
            start_slot,
            finish_slot
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_last_canonical_blocks(
        &self,
        best_tip_hash: String,
        limit: i64,
    ) -> Result<Vec<Block>, sqlx::Error> {
        sqlx::query_file_as!(
            Block,
            "src/archive/sql/query_last_canonical_blocks.sql",
            best_tip_hash,
            limit
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_latest_block(&self) -> Result<StateHash, sqlx::Error> {
        let block = sqlx::query_file_as!(LatestBlock, "src/archive/sql/query_latest_block.sql")
            .fetch_one(&self.pool)
            .await?;

        Ok(block.state_hash)
    }
}

pub type StateHash = String;
struct LatestBlock {
    state_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    id: i32,
    pub state_hash: String,
    pub height: i64,
    timestamp: String,
    pub chain_status: ChainStatus,
    pub creator_key: String,
    winner_key: String,
    global_slot_since_hard_fork: i64,
    global_slot_since_genesis: i64,
    parent_id: Option<i32>,
}

impl Block {
    pub fn global_slot(&self) -> u32 {
        self.global_slot_since_hard_fork as u32
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test() {
        let db = ArchiveConnector::connect(ArchiveUrl::Env).await;

        let blocks = db
            ._get_producer_blocks("B62qkPpK6z4ktWjxcmFzM4cFWjWLzrjNh6USjUMiYGcF3YAVbdo2p4H")
            .await
            .unwrap();

        let canonical = blocks
            .iter()
            .filter(|block| block.chain_status == ChainStatus::Pending)
            .collect::<Vec<_>>();

        println!("Canonical blocks: {}", canonical.len());
    }
}
