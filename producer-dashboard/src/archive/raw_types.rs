use serde::{Deserialize, Serialize};

use super::{
    postgres_types::{InternalCommandType, TransactionStatus},
    ArchiveConnector, ArchiveUrl, ChainStatus,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawBlock {
    id: i32,
    state_hash: String,
    parent_id: Option<i32>,
    parent_hash: String,
    creator_id: i32,
    block_winner_id: i32,
    last_vrf_output: String,
    snarked_ledger_hash_id: i32,
    staking_epoch_data_id: i32,
    next_epoch_data_id: i32,
    min_window_density: i64,
    sub_window_densities: Vec<i64>,
    total_currency: String,
    ledger_hash: String,
    height: i64,
    global_slot_since_hard_fork: i64,
    global_slot_since_genesis: i64,
    protocol_version_id: i32,
    proposed_protocol_version_id: Option<i32>,
    timestamp: String,
    chain_status: ChainStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawAccountIdentifier {
    id: i32,
    public_key_id: i32,
    token_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawAccountsAccessed {
    ledger_index: i32,
    block_id: i32,
    account_identifier_id: i32,
    token_symbol_id: i32,
    balance: String,
    nonce: i64,
    receipt_chain_hash: String,
    delegate_id: Option<i32>,
    voting_for_id: i32,
    timing_id: Option<i32>,
    permissions_id: i32,
    zkapp_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawAccountsCreated {
    block_id: i32,
    account_identifier_id: i32,
    creation_fee: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawBlocksInternalCommands {
    block_id: i32,
    internal_command_id: i32,
    sequence_no: i32,
    secondary_sequence_no: i32,
    status: TransactionStatus,
    failure_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawBlocksUserCommands {
    block_id: i32,
    user_command_id: i32,
    sequence_no: i32,
    status: TransactionStatus,
    failure_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawBlocksZkappCommands {
    block_id: i32,
    zkapp_command_id: i32,
    sequence_no: i32,
    status: TransactionStatus,
    failure_reasons_ids: Option<Vec<i32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawEpochData {
    id: i32,
    seed: String,
    ledger_hash_id: i32,
    total_currency: String,
    start_checkpoint: String,
    lock_checkpoint: String,
    epoch_length: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawInternalCommands {
    id: i32,
    command_type: InternalCommandType,
    receiver_id: i32,
    fee: String,
    hash: String,
}

pub struct ArchiveConnectorForTest {
    inner: ArchiveConnector,
}

impl ArchiveConnectorForTest {
    pub async fn new(url: ArchiveUrl) -> Self {
        Self {
            inner: ArchiveConnector::connect(url).await,
        }
    }

    pub async fn all_blocks(&self) -> Result<Vec<RawBlock>, sqlx::Error> {
        sqlx::query_file_as!(RawBlock, "src/archive/sql/test/all_blocks.sql")
            .fetch_all(&self.inner.pool)
            .await
    }

    pub async fn all_account_identifiers(&self) -> Result<Vec<RawAccountIdentifier>, sqlx::Error> {
        sqlx::query_file_as!(
            RawAccountIdentifier,
            "src/archive/sql/test/all_account_identifiers.sql"
        )
        .fetch_all(&self.inner.pool)
        .await
    }

    pub async fn all_accounts_accessed(&self) -> Result<Vec<RawAccountsAccessed>, sqlx::Error> {
        sqlx::query_file_as!(
            RawAccountsAccessed,
            "src/archive/sql/test/all_accounts_accessed.sql"
        )
        .fetch_all(&self.inner.pool)
        .await
    }

    pub async fn all_accounts_created(&self) -> Result<Vec<RawAccountsCreated>, sqlx::Error> {
        sqlx::query_file_as!(
            RawAccountsCreated,
            "src/archive/sql/test/all_accounts_created.sql"
        )
        .fetch_all(&self.inner.pool)
        .await
    }

    pub async fn all_blocks_internal_commands(
        &self,
    ) -> Result<Vec<RawBlocksInternalCommands>, sqlx::Error> {
        sqlx::query_file_as!(
            RawBlocksInternalCommands,
            "src/archive/sql/test/all_blocks_internal_commands.sql"
        )
        .fetch_all(&self.inner.pool)
        .await
    }

    pub async fn all_blocks_user_commands(
        &self,
    ) -> Result<Vec<RawBlocksUserCommands>, sqlx::Error> {
        sqlx::query_file_as!(
            RawBlocksUserCommands,
            "src/archive/sql/test/all_blocks_user_commands.sql"
        )
        .fetch_all(&self.inner.pool)
        .await
    }

    pub async fn all_blocks_zkapp_commands(
        &self,
    ) -> Result<Vec<RawBlocksZkappCommands>, sqlx::Error> {
        sqlx::query_file_as!(
            RawBlocksZkappCommands,
            "src/archive/sql/test/all_blocks_zkapp_commands.sql"
        )
        .fetch_all(&self.inner.pool)
        .await
    }

    pub async fn all_epoch_data(&self) -> Result<Vec<RawEpochData>, sqlx::Error> {
        sqlx::query_file_as!(RawEpochData, "src/archive/sql/test/all_epoch_data.sql")
            .fetch_all(&self.inner.pool)
            .await
    }

    pub async fn all_internal_commands(&self) -> Result<Vec<RawInternalCommands>, sqlx::Error> {
        sqlx::query_file_as!(
            RawInternalCommands,
            "src/archive/sql/test/all_internal_commands.sql"
        )
        .fetch_all(&self.inner.pool)
        .await
    }
}
