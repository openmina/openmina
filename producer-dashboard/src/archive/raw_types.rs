use serde::{Deserialize, Serialize};

use super::{
    postgres_types::{
        AuthorizationKind, InternalCommandType, MayUseToken, TransactionStatus, UserCommandType,
        ZkappAuthRequiredType,
    },
    ArchiveConnector, ArchiveUrl, ChainStatus,
};

// macro_rules! define_fetch_all {
//     ($fn_name:ident, $struct_name:ty, $sql_file:expr) => {
//         pub async fn $fn_name(&self) -> Result<Vec<$struct_name>, sqlx::Error> {
//             sqlx::query_file_as!($struct_name, $sql_file)
//                 .fetch_all(&self.inner.pool)
//                 .await
//         }
//     };
// }

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawProtocolVersion {
    id: i32,
    transaction: i32,
    network: i32,
    patch: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawPublicKeys {
    id: i32,
    value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawSnarkedLedgerHashes {
    id: i32,
    value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawTimingInfo {
    id: i32,
    account_identifier_id: i32,
    initial_minimum_balance: String,
    cliff_time: i64,
    cliff_amount: String,
    vesting_period: i64,
    vesting_increment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawTokenSymbols {
    id: i32,
    value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawTokens {
    id: i32,
    value: String,
    owner_public_key_id: Option<i32>,
    owner_token_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawUserCommands {
    id: i32,
    command_type: UserCommandType,
    fee_payer_id: i32,
    source_id: i32,
    receiver_id: i32,
    nonce: i64,
    amount: Option<String>,
    fee: String,
    valid_until: Option<i64>,
    memo: String,
    hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawVotingFor {
    id: i32,
    value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappAccountPrecondition {
    id: i32,
    balance_id: Option<i32>,
    nonce_id: Option<i32>,
    receipt_chain_hash: Option<String>,
    delegate_id: Option<i32>,
    state_id: i32,
    action_state_id: Option<i32>,
    proved_state: Option<bool>,
    is_new: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappAccountUpdate {
    id: i32,
    body_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappAccountUpdateBody {
    id: i32,
    account_identifier_id: i32,
    update_id: i32,
    balance_change: String,
    increment_nonce: bool,
    events_id: i32,
    actions_id: i32,
    call_data_id: i32,
    call_depth: i32,
    zkapp_network_precondition_id: i32,
    zkapp_account_precondition_id: i32,
    zkapp_valid_while_precondition_id: Option<i32>,
    use_full_commitment: bool,
    implicit_account_creation_fee: bool,
    may_use_token: MayUseToken,
    authorization_kind: AuthorizationKind,
    verification_key_hash_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappAccountUpdateFailure {
    id: i32,
    index: i32,
    failures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappAccount {
    id: i32,
    app_state_id: i32,
    verification_key_id: Option<i32>,
    zkapp_version: i64,
    action_state_id: i32,
    last_action_slot: i64,
    proved_state: bool,
    zkapp_uri_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappActionState {
    id: i32,
    element0: i32,
    element1: i32,
    element2: i32,
    element3: i32,
    element4: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappAmountBounds {
    id: i32,
    amount_lower_bound: String,
    amount_upper_bound: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappBalanceBounds {
    id: i32,
    balance_lower_bound: String,
    balance_upper_bound: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappCommands {
    id: i32,
    zkapp_fee_payer_body_id: i32,
    zkapp_account_updates_ids: Vec<i32>,
    memo: String,
    hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappEpochData {
    id: i32,
    epoch_ledger_id: Option<i32>,
    epoch_seed: Option<String>,
    start_checkpoint: Option<String>,
    lock_checkpoint: Option<String>,
    epoch_length_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappEpochLedger {
    id: i32,
    hash_id: Option<i32>,
    total_currency_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappEvents {
    id: i32,
    element_ids: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappFeePayerBody {
    id: i32,
    public_key_id: i32,
    fee: String,
    valid_until: Option<i64>,
    nonce: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappField {
    id: i32,
    field: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappFieldArray {
    id: i32,
    element_ids: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappGlobalSlotBounds {
    id: i32,
    global_slot_lower_bound: i64,
    global_slot_upper_bound: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappLengthBounds {
    id: i32,
    length_lower_bound: i64,
    length_upper_bound: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappNetworkPrecondition {
    id: i32,
    snarked_ledger_hash_id: Option<i32>,
    blockchain_length_id: Option<i32>,
    min_window_density_id: Option<i32>,
    total_currency_id: Option<i32>,
    global_slot_since_genesis: Option<i32>,
    staking_epoch_data_id: Option<i32>,
    next_epoch_data_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappNonceBounds {
    id: i32,
    nonce_lower_bound: i64,
    nonce_upper_bound: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappPermissions {
    id: i32,
    edit_state: ZkappAuthRequiredType,
    send: ZkappAuthRequiredType,
    receive: ZkappAuthRequiredType,
    access: ZkappAuthRequiredType,
    set_delegate: ZkappAuthRequiredType,
    set_permissions: ZkappAuthRequiredType,
    set_verification_key_auth: ZkappAuthRequiredType,
    set_verification_key_txn_version: i32,
    set_zkapp_uri: ZkappAuthRequiredType,
    edit_action_state: ZkappAuthRequiredType,
    set_token_symbol: ZkappAuthRequiredType,
    increment_nonce: ZkappAuthRequiredType,
    set_voting_for: ZkappAuthRequiredType,
    set_timing: ZkappAuthRequiredType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappStates {
    id: i32,
    element0: i32,
    element1: i32,
    element2: i32,
    element3: i32,
    element4: i32,
    element5: i32,
    element6: i32,
    element7: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappStatesNullable {
    id: i32,
    element0: Option<i32>,
    element1: Option<i32>,
    element2: Option<i32>,
    element3: Option<i32>,
    element4: Option<i32>,
    element5: Option<i32>,
    element6: Option<i32>,
    element7: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappTimingInfo {
    id: i32,
    initial_minimum_balance: String,
    cliff_time: i64,
    cliff_amount: String,
    vesting_period: i64,
    vesting_increment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappTokenIdBounds {
    id: i32,
    token_id_lower_bound: String,
    token_id_upper_bound: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappUpdates {
    id: i32,
    app_state_id: i32,
    delegate_id: Option<i32>,
    verification_key_id: Option<i32>,
    permissions_id: Option<i32>,
    zkapp_uri_id: Option<i32>,
    token_symbol_id: Option<i32>,
    timing_id: Option<i32>,
    voting_for_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappUris {
    id: i32,
    value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappVerificationKeyHashes {
    id: i32,
    value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawZkappVerificationKeys {
    id: i32,
    verification_key: String,
    hash_id: i32,
}

#[allow(dead_code)]
pub struct ArchiveConnectorForTest {
    inner: ArchiveConnector,
}

impl ArchiveConnectorForTest {
    pub async fn new(url: ArchiveUrl) -> Self {
        Self {
            inner: ArchiveConnector::connect(url).await,
        }
    }
}
