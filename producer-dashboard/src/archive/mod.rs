use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct ArchiveConnector {
    pool: PgPool,
}

impl ArchiveConnector {
    pub async fn connect() -> Self {
        // TODO(adonagy): unwrap
        let pool = PgPool::connect(&dotenvy::var("DATABASE_URL").unwrap())
            .await
            .unwrap();

        Self { pool }
    }

    // pub fn pool_ref(&self) -> &PgPool {
    //     &self.pool
    // }

    pub async fn get_producer_blocks(&self, producer_pk: &str) -> Result<Vec<Block>, sqlx::Error> {
        sqlx::query_as!(
            Block,
            r#"SELECT 
                    b.id, 
                    b.state_hash, 
                    b.height, 
                    b.timestamp, 
                    b.chain_status AS "chain_status: ChainStatus",
                    pk_creator.value AS "creator_key",
                    pk_winner.value AS "winner_key"
                FROM 
                    blocks b
                JOIN 
                    public_keys pk_creator ON b.creator_id = pk_creator.id
                JOIN 
                    public_keys pk_winner ON b.block_winner_id = pk_winner.id
                WHERE 
                    pk_creator.value = $1"#,
            producer_pk
        )
        .fetch_all(&self.pool)
        .await

        // Other logic
    }
}

#[derive(sqlx::Type, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "chain_status_type", rename_all = "lowercase")]
enum ChainStatus {
    Canonical,
    Orphaned,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    id: i32,
    state_hash: String,
    height: i64,
    timestamp: String,
    chain_status: ChainStatus,
    creator_key: String,
    winner_key: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test() {
        let db = ArchiveConnector::connect().await;

        let blocks = db
            .get_producer_blocks("B62qkPpK6z4ktWjxcmFzM4cFWjWLzrjNh6USjUMiYGcF3YAVbdo2p4H")
            .await
            .unwrap();

        let canonical = blocks
            .iter()
            .filter(|block| block.chain_status == ChainStatus::Pending)
            .collect::<Vec<_>>();

        println!("Canonical blocks: {}", canonical.len());
    }
}
