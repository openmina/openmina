use mina_hasher::Fp;
use mina_signer::CompressedPubKey;

pub struct StagedLedgerHashNonSnark {
    pub ledger_hash: Fp,
    pub aux_hash: Fp,             // TODO: In binprot it's a string ?
    pub pending_coinbase_aux: Fp, // TODO: In binprot it's a string ?
}

pub struct StagedLedgerHash {
    pub non_snark: StagedLedgerHashNonSnark,
    pub pending_coinbase_hash: Fp,
}

pub enum Sgn {
    Pos,
    Neg,
}

pub struct Excess {
    pub magnitude: i64,
    pub sgn: Sgn,
}

pub enum TransactionFailure {
    Predicate,
    SourceNotPresent,
    ReceiverNotPresent,
    AmountInsufficientToCreateAccount,
    CannotPayCreationFeeInToken,
    SourceInsufficientBalance,
    SourceMinimumBalanceViolation,
    ReceiverAlreadyExists,
    TokenOwnerNotCaller,
    Overflow,
    GlobalExcessOverflow,
    LocalExcessOverflow,
    SignedCommandOnZkappAccount,
    ZkappAccountNotPresent,
    UpdateNotPermittedBalance,
    UpdateNotPermittedTimingExistingAccount,
    UpdateNotPermittedDelegate,
    UpdateNotPermittedAppState,
    UpdateNotPermittedVerificationKey,
    UpdateNotPermittedSequenceState,
    UpdateNotPermittedZkappUri,
    UpdateNotPermittedTokenSymbol,
    UpdateNotPermittedPermissions,
    UpdateNotPermittedNonce,
    UpdateNotPermittedVotingFor,
    PartiesReplayCheckFailed,
    FeePayerNonceMustIncrease,
    FeePayerMustBeSigned,
    AccountBalancePreconditionUnsatisfied,
    AccountNoncePreconditionUnsatisfied,
    AccountReceiptChainHashPreconditionUnsatisfied,
    AccountDelegatePreconditionUnsatisfied,
    AccountSequenceStatePreconditionUnsatisfied,
    AccountAppStatePreconditionUnsatisfied(i32),
    AccountProvedStatePreconditionUnsatisfied,
    AccountIsNewPreconditionUnsatisfied,
    ProtocolStatePreconditionUnsatisfied,
    IncorrectNonce,
    InvalidFeeExcess,
}

pub struct LocalState {
    pub stack_frame: Fp,
    pub call_stack: Fp,
    pub transaction_commitment: Fp,
    pub full_transaction_commitment: Fp,
    pub token_id: Fp,
    pub excess: Excess,
    pub ledger: Fp,
    pub success: bool,
    pub party_index: i32,
    pub failure_status_tbl: Vec<Vec<TransactionFailure>>,
}

pub struct BlockchainStateRegisters {
    pub ledger: Fp,
    pub pending_coinbase_stack: (),
    pub local_state: LocalState,
}

pub struct ConsensusGlobalSlot {
    pub slot_number: u32,
    pub slots_per_epoch: u32,
}

pub struct EpochLedger {
    pub hash: Fp,
    pub total_currency: i64,
}

pub struct DataStaking {
    pub ledger: EpochLedger,
    pub seed: Fp,
    pub start_checkpoint: Fp,
    pub lock_checkpoint: Fp,
    pub epoch_length: u32,
}

pub struct ConsensusState {
    pub blockchain_length: u32,
    pub epoch_count: u32,
    pub min_window_density: u32,
    pub sub_window_densities: Vec<u32>,
    pub last_vrf_output: Fp, // TODO: In binprot it's a string ?
    pub total_currency: i64,
    pub curr_global_slot: ConsensusGlobalSlot,
    pub global_slot_since_genesis: u32,
    pub staking_epoch_data: DataStaking,
    pub next_epoch_data: DataStaking,
    pub has_ancestor_in_same_checkpoint_window: bool,
    pub block_stake_winner: CompressedPubKey,
    pub block_creator: CompressedPubKey,
    pub coinbase_receiver: CompressedPubKey,
    pub supercharge_coinbase: bool,
}

pub struct BlockchainState {
    pub staged_ledger_hash: StagedLedgerHash,
    pub genesis_ledger_hash: Fp,
    pub registers: BlockchainStateRegisters,
    pub timestamp: u64,
    pub body_reference: Fp, // TODO: In binprot it's a string ?
}

pub struct ProtocolConstants {
    pub k: u32,
    pub slots_per_epoch: u32,
    pub slots_per_sub_window: u32,
    pub delta: u32,
    pub genesis_state_timestamp: u64,
}

pub struct ProtocolStateBody {
    pub genesis_state_hash: Fp,
    pub blockchain_state: BlockchainState,
    pub consensus_state: ConsensusState,
    pub constants: ProtocolConstants,
}

pub struct ProtocolState {
    pub previous_state_hash: Fp,
    pub body: ProtocolStateBody,
}
