use binprot::{BinProtRead, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};

use super::manual::*;

/// **Origin**: `Mina_block__Block.Stable.V2.t`
///
/// **Location**: [src/lib/mina_block/block.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_block/block.ml#L8)
///
/// **Gid**: 974
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBlockBlockStableV2 {
    pub header: MinaBlockHeaderStableV2,
    pub body: StagedLedgerDiffBodyStableV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Network_pool__Transaction_pool.Diff_versioned.Stable.V2.t`
///
/// **Location**: [src/lib/network_pool/transaction_pool.ml:47:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_pool/transaction_pool.ml#L47)
///
/// **Gid**: 1001
pub struct NetworkPoolTransactionPoolDiffVersionedStableV2(pub Vec<MinaBaseUserCommandStableV2>);

/// **Origin**: `Network_pool__Snark_pool.Diff_versioned.Stable.V2.t`
///
/// **Location**: [src/lib/network_pool/snark_pool.ml:732:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_pool/snark_pool.ml#L732)
///
/// **Gid**: 1007
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum NetworkPoolSnarkPoolDiffVersionedStableV2 {
    AddSolvedWork(
        TransactionSnarkWorkStatementStableV2,
        NetworkPoolSnarkPoolDiffVersionedStableV2AddSolvedWork1<
            TransactionSnarkWorkTStableV2Proofs<LedgerProofProdStableV2>,
        >,
    ),
    Empty,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Sparse_ledger_base.Stable.V2.t`
///
/// **Location**: [src/lib/mina_base/sparse_ledger_base.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/sparse_ledger_base.ml#L8)
///
/// **Gid**: 774
pub struct MinaBaseSparseLedgerBaseStableV2(
    pub  MinaBaseSparseLedgerBaseStableV2Poly<
        MinaBaseLedgerHash0StableV1,
        MinaBaseAccountIdMakeStrStableV2,
        MinaBaseAccountBinableArgStableV2,
    >,
);

/// **Origin**: `Network_peer__Peer.Stable.V1.t`
///
/// **Location**: [src/lib/network_peer/peer.ml:28:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_peer/peer.ml#L28)
///
/// **Gid**: 810
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NetworkPeerPeerStableV1 {
    pub host: crate::string::ByteString,
    pub libp2p_port: crate::number::Int32,
    pub peer_id: NetworkPeerPeerIdStableV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Transaction_snark_scan_state.Stable.V2.t`
///
/// **Location**: [src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml:153:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml#L153)
///
/// **Gid**: 951
pub struct TransactionSnarkScanStateStableV2(
    pub  TransactionSnarkScanStateStableV2Poly<
        TransactionSnarkScanStateLedgerProofWithSokMessageStableV2,
        TransactionSnarkScanStateTransactionWithWitnessStableV2,
    >,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Pending_coinbase.Stable.V2.t`
///
/// **Location**: [src/lib/mina_base/pending_coinbase.ml:1238:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L1238)
///
/// **Gid**: 766
pub struct MinaBasePendingCoinbaseStableV2(
    pub  MinaBasePendingCoinbaseStableV2Poly<
        MinaBasePendingCoinbaseMerkleTreeVersionedStableV2,
        MinaBasePendingCoinbaseStackIdStableV1,
    >,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_ledger__Sync_ledger.Query.Stable.V1.t`
///
/// **Location**: [src/lib/mina_ledger/sync_ledger.ml:71:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_ledger/sync_ledger.ml#L71)
///
/// **Gid**: 828
pub struct MinaLedgerSyncLedgerQueryStableV1(
    pub MinaLedgerSyncLedgerQueryStableV1Poly<MerkleAddressBinableArgStableV1>,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_ledger__Sync_ledger.Answer.Stable.V2.t`
///
/// **Location**: [src/lib/mina_ledger/sync_ledger.ml:56:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_ledger/sync_ledger.ml#L56)
///
/// **Gid**: 827
pub struct MinaLedgerSyncLedgerAnswerStableV2(
    pub  MinaLedgerSyncLedgerAnswerStableV2Poly<
        MinaBaseLedgerHash0StableV1,
        MinaBaseAccountBinableArgStableV2,
    >,
);

/// **Origin**: `Sync_status.T.Stable.V1.t`
///
/// **Location**: [src/lib/sync_status/sync_status.ml:54:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/sync_status/sync_status.ml#L54)
///
/// **Gid**: 1031
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[polymorphic_variant]
pub enum SyncStatusTStableV1 {
    Connecting,
    Listening,
    Offline,
    Bootstrap,
    Synced,
    Catchup,
}

/// **Origin**: `Trust_system__Peer_status.Stable.V1.t`
///
/// **Location**: [src/lib/trust_system/peer_status.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/trust_system/peer_status.ml#L6)
///
/// **Gid**: 815
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TrustSystemPeerStatusStableV1 {
    pub trust: crate::number::Float64,
    pub banned: TrustSystemBannedStatusStableV1,
}

/// Location: [src/lib/mina_state/protocol_state.ml:16:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L16)
///
/// Gid: 867
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateProtocolStateValueStableV2Poly<StateHash, Body> {
    pub previous_state_hash: StateHash,
    pub body: Body,
}

/// **Origin**: `Data_hash_lib__State_hash.Stable.V1.t`
///
/// **Location**: [src/lib/data_hash_lib/state_hash.ml:44:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/data_hash_lib/state_hash.ml#L44)
///
/// **Gid**: 586
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct DataHashLibStateHashStableV1(pub crate::bigint::BigInt);

/// Location: [src/lib/mina_state/protocol_state.ml:38:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L38)
///
/// Gid: 868
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateProtocolStateBodyValueStableV2Poly<
    StateHash,
    BlockchainState,
    ConsensusState,
    Constants,
> {
    pub genesis_state_hash: StateHash,
    pub blockchain_state: BlockchainState,
    pub consensus_state: ConsensusState,
    pub constants: Constants,
}

/// Location: [src/lib/mina_state/registers.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/registers.ml#L8)
///
/// Gid: 862
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateBlockchainStateValueStableV2PolyRegisters<
    Ledger,
    PendingCoinbaseStack,
    LocalState,
> {
    pub ledger: Ledger,
    pub pending_coinbase_stack: PendingCoinbaseStack,
    pub local_state: LocalState,
}

/// Location: [src/lib/mina_state/blockchain_state.ml:9:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/blockchain_state.ml#L9)
///
/// Gid: 863
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateBlockchainStateValueStableV2Poly<
    StagedLedgerHash,
    SnarkedLedgerHash,
    LocalState,
    Time,
    BodyReference,
> {
    pub staged_ledger_hash: StagedLedgerHash,
    pub genesis_ledger_hash: SnarkedLedgerHash,
    pub registers:
        MinaStateBlockchainStateValueStableV2PolyRegisters<SnarkedLedgerHash, (), LocalState>,
    pub timestamp: Time,
    pub body_reference: BodyReference,
}

/// Location: [src/lib/mina_base/staged_ledger_hash.ml:185:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L185)
///
/// Gid: 770
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashStableV1Poly<NonSnark, PendingCoinbaseHash> {
    pub non_snark: NonSnark,
    pub pending_coinbase_hash: PendingCoinbaseHash,
}

/// **Origin**: `Mina_base__Ledger_hash0.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/ledger_hash0.ml:17:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/ledger_hash0.ml#L17)
///
/// **Gid**: 623
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseLedgerHash0StableV1(pub crate::bigint::BigInt);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Staged_ledger_hash.Aux_hash.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/staged_ledger_hash.ml:16:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L16)
///
/// **Gid**: 767
pub struct MinaBaseStagedLedgerHashAuxHashStableV1(pub crate::string::ByteString);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Staged_ledger_hash.Pending_coinbase_aux.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/staged_ledger_hash.ml:62:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L62)
///
/// **Gid**: 768
pub struct MinaBaseStagedLedgerHashPendingCoinbaseAuxStableV1(pub crate::string::ByteString);

/// **Origin**: `Mina_base__Staged_ledger_hash.Non_snark.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/staged_ledger_hash.ml:98:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L98)
///
/// **Gid**: 769
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashNonSnarkStableV1 {
    pub ledger_hash: MinaBaseLedgerHash0StableV1,
    pub aux_hash: MinaBaseStagedLedgerHashAuxHashStableV1,
    pub pending_coinbase_aux: MinaBaseStagedLedgerHashPendingCoinbaseAuxStableV1,
}

/// **Origin**: `Mina_base__Pending_coinbase.Hash_builder.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/pending_coinbase.ml:358:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L358)
///
/// **Gid**: 755
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseHashBuilderStableV1(pub crate::bigint::BigInt);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Pending_coinbase.Hash_versioned.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/pending_coinbase.ml:517:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L517)
///
/// **Gid**: 763
pub struct MinaBasePendingCoinbaseHashVersionedStableV1(
    pub MinaBasePendingCoinbaseHashBuilderStableV1,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Staged_ledger_hash.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/staged_ledger_hash.ml:202:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L202)
///
/// **Gid**: 771
pub struct MinaBaseStagedLedgerHashStableV1(
    pub  MinaBaseStagedLedgerHashStableV1Poly<
        MinaBaseStagedLedgerHashNonSnarkStableV1,
        MinaBasePendingCoinbaseHashVersionedStableV1,
    >,
);

/// Location: [src/lib/transaction_logic/parties_logic.ml:170:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_logic/parties_logic.ml#L170)
///
/// Gid: 791
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicPartiesLogicLocalStateValueStableV1Poly<
    StackFrame,
    CallStack,
    TokenId,
    Excess,
    Ledger,
    Bool,
    Comm,
    Length,
    FailureStatusTbl,
> {
    pub stack_frame: StackFrame,
    pub call_stack: CallStack,
    pub transaction_commitment: Comm,
    pub full_transaction_commitment: Comm,
    pub token_id: TokenId,
    pub excess: Excess,
    pub ledger: Ledger,
    pub success: Bool,
    pub party_index: Length,
    pub failure_status_tbl: FailureStatusTbl,
}

/// **Origin**: `Mina_base__Stack_frame.Digest.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/stack_frame.ml:55:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/stack_frame.ml#L55)
///
/// **Gid**: 773
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStackFrameDigestStableV1(pub crate::bigint::BigInt);

/// **Origin**: `Mina_base__Call_stack_digest.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/call_stack_digest.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/call_stack_digest.ml#L6)
///
/// **Gid**: 779
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseCallStackDigestStableV1(pub crate::bigint::BigInt);

/// **Origin**: `Mina_base__Account_id.Make_str.Digest.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/account_id.ml:64:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account_id.ml#L64)
///
/// **Gid**: 601
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountIdMakeStrDigestStableV1(pub crate::bigint::BigInt);

/// Location: [src/lib/currency/signed_poly.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/signed_poly.ml#L6)
///
/// Gid: 552
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicPartiesLogicLocalStateValueStableV1PolyArg3<Magnitude, Sgn> {
    pub magnitude: Magnitude,
    pub sgn: Sgn,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Currency.Make_str.Amount.Make_str.Stable.V1.t`
///
/// **Location**: [src/lib/currency/currency.ml:1030:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/currency.ml#L1030)
///
/// **Gid**: 554
pub struct CurrencyMakeStrAmountMakeStrStableV1(pub crate::number::Int64);

/// **Origin**: `Sgn.Stable.V1.t`
///
/// **Location**: [src/lib/sgn/sgn.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/sgn/sgn.ml#L9)
///
/// **Gid**: 551
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum SgnStableV1 {
    Pos,
    Neg,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_numbers/nat.ml:258:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L258)
///
/// Gid: 545
pub struct MinaTransactionLogicPartiesLogicLocalStateValueStableV1PolyArg7(
    pub crate::number::Int32,
);

/// **Origin**: `Mina_base__Transaction_status.Failure.Stable.V2.t`
///
/// **Location**: [src/lib/mina_base/transaction_status.ml:9:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L9)
///
/// **Gid**: 680
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseTransactionStatusFailureStableV2 {
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
    AccountAppStatePreconditionUnsatisfied(crate::number::Int32),
    AccountProvedStatePreconditionUnsatisfied,
    AccountIsNewPreconditionUnsatisfied,
    ProtocolStatePreconditionUnsatisfied,
    IncorrectNonce,
    InvalidFeeExcess,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Transaction_status.Failure.Collection.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_status.ml:71:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L71)
///
/// **Gid**: 682
pub struct MinaBaseTransactionStatusFailureCollectionStableV1(
    pub Vec<Vec<MinaBaseTransactionStatusFailureStableV2>>,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_transaction_logic__Parties_logic.Local_state.Value.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_logic/parties_logic.ml:216:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_logic/parties_logic.ml#L216)
///
/// **Gid**: 792
pub struct MinaTransactionLogicPartiesLogicLocalStateValueStableV1(
    pub  MinaTransactionLogicPartiesLogicLocalStateValueStableV1Poly<
        MinaBaseStackFrameDigestStableV1,
        MinaBaseCallStackDigestStableV1,
        MinaBaseAccountIdMakeStrDigestStableV1,
        MinaTransactionLogicPartiesLogicLocalStateValueStableV1PolyArg3<
            CurrencyMakeStrAmountMakeStrStableV1,
            SgnStableV1,
        >,
        MinaBaseLedgerHash0StableV1,
        bool,
        crate::bigint::BigInt,
        MinaTransactionLogicPartiesLogicLocalStateValueStableV1PolyArg7,
        MinaBaseTransactionStatusFailureCollectionStableV1,
    >,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Block_time.Make_str.Time.Stable.V1.t`
///
/// **Location**: [src/lib/block_time/block_time.ml:22:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/block_time/block_time.ml#L22)
///
/// **Gid**: 595
pub struct BlockTimeMakeStrTimeStableV1(pub crate::number::Int64);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Consensus__Body_reference.Stable.V1.t`
///
/// **Location**: [src/lib/consensus/body_reference.ml:17:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/body_reference.ml#L17)
///
/// **Gid**: 843
pub struct ConsensusBodyReferenceStableV1(pub crate::string::ByteString);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_state__Blockchain_state.Value.Stable.V2.t`
///
/// **Location**: [src/lib/mina_state/blockchain_state.ml:43:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/blockchain_state.ml#L43)
///
/// **Gid**: 864
pub struct MinaStateBlockchainStateValueStableV2(
    pub  MinaStateBlockchainStateValueStableV2Poly<
        MinaBaseStagedLedgerHashStableV1,
        MinaBaseLedgerHash0StableV1,
        MinaTransactionLogicPartiesLogicLocalStateValueStableV1,
        BlockTimeMakeStrTimeStableV1,
        ConsensusBodyReferenceStableV1,
    >,
);

/// Location: [src/lib/consensus/proof_of_stake.ml:1687:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1687)
///
/// Gid: 858
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataConsensusStateValueStableV1Poly<
    Length,
    VrfOutput,
    Amount,
    GlobalSlot,
    GlobalSlotSinceGenesis,
    StakingEpochData,
    NextEpochData,
    Bool,
    Pk,
> {
    pub blockchain_length: Length,
    pub epoch_count: Length,
    pub min_window_density: Length,
    pub sub_window_densities: Vec<Length>,
    pub last_vrf_output: VrfOutput,
    pub total_currency: Amount,
    pub curr_global_slot: GlobalSlot,
    pub global_slot_since_genesis: GlobalSlotSinceGenesis,
    pub staking_epoch_data: StakingEpochData,
    pub next_epoch_data: NextEpochData,
    pub has_ancestor_in_same_checkpoint_window: Bool,
    pub block_stake_winner: Pk,
    pub block_creator: Pk,
    pub coinbase_receiver: Pk,
    pub supercharge_coinbase: Bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_numbers/nat.ml:258:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L258)
///
/// Gid: 548
pub struct ConsensusProofOfStakeDataConsensusStateValueStableV1PolyArg0(pub crate::number::Int32);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Consensus_vrf.Output.Truncated.Stable.V1.t`
///
/// **Location**: [src/lib/consensus/vrf/consensus_vrf.ml:167:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/vrf/consensus_vrf.ml#L167)
///
/// **Gid**: 829
pub struct ConsensusVrfOutputTruncatedStableV1(pub crate::string::ByteString);

/// Location: [src/lib/consensus/global_slot.ml:11:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/global_slot.ml#L11)
///
/// Gid: 847
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusGlobalSlotStableV1Poly<SlotNumber, SlotsPerEpoch> {
    pub slot_number: SlotNumber,
    pub slots_per_epoch: SlotsPerEpoch,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_numbers/nat.ml:258:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L258)
///
/// Gid: 539
pub struct ConsensusGlobalSlotStableV1PolyArg0(pub crate::number::Int32);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Consensus__Global_slot.Stable.V1.t`
///
/// **Location**: [src/lib/consensus/global_slot.ml:21:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/global_slot.ml#L21)
///
/// **Gid**: 848
pub struct ConsensusGlobalSlotStableV1(
    pub  ConsensusGlobalSlotStableV1Poly<
        ConsensusGlobalSlotStableV1PolyArg0,
        ConsensusProofOfStakeDataConsensusStateValueStableV1PolyArg0,
    >,
);

/// Location: [src/lib/mina_base/epoch_data.ml:8:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_data.ml#L8)
///
/// Gid: 678
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1Poly<
    EpochLedger,
    EpochSeed,
    StartCheckpoint,
    LockCheckpoint,
    Length,
> {
    pub ledger: EpochLedger,
    pub seed: EpochSeed,
    pub start_checkpoint: StartCheckpoint,
    pub lock_checkpoint: LockCheckpoint,
    pub epoch_length: Length,
}

/// Location: [src/lib/mina_base/epoch_ledger.ml:9:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_ledger.ml#L9)
///
/// Gid: 671
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseEpochLedgerValueStableV1Poly<LedgerHash, Amount> {
    pub hash: LedgerHash,
    pub total_currency: Amount,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Epoch_ledger.Value.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/epoch_ledger.ml:23:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_ledger.ml#L23)
///
/// **Gid**: 672
pub struct MinaBaseEpochLedgerValueStableV1(
    pub  MinaBaseEpochLedgerValueStableV1Poly<
        MinaBaseLedgerHash0StableV1,
        CurrencyMakeStrAmountMakeStrStableV1,
    >,
);

/// **Origin**: `Mina_base__Epoch_seed.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/epoch_seed.ml:18:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_seed.ml#L18)
///
/// **Gid**: 675
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseEpochSeedStableV1(pub crate::bigint::BigInt);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Consensus__Proof_of_stake.Data.Epoch_data.Staking_value_versioned.Value.Stable.V1.t`
///
/// **Location**: [src/lib/consensus/proof_of_stake.ml:1054:12](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1054)
///
/// **Gid**: 856
pub struct ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1(
    pub  ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1Poly<
        MinaBaseEpochLedgerValueStableV1,
        MinaBaseEpochSeedStableV1,
        DataHashLibStateHashStableV1,
        DataHashLibStateHashStableV1,
        ConsensusProofOfStakeDataConsensusStateValueStableV1PolyArg0,
    >,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Consensus__Proof_of_stake.Data.Epoch_data.Next_value_versioned.Value.Stable.V1.t`
///
/// **Location**: [src/lib/consensus/proof_of_stake.ml:1078:12](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1078)
///
/// **Gid**: 857
pub struct ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1(
    pub  ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1Poly<
        MinaBaseEpochLedgerValueStableV1,
        MinaBaseEpochSeedStableV1,
        DataHashLibStateHashStableV1,
        DataHashLibStateHashStableV1,
        ConsensusProofOfStakeDataConsensusStateValueStableV1PolyArg0,
    >,
);

/// Location: [src/lib/non_zero_curve_point/compressed_poly.ml:13:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/compressed_poly.ml#L13)
///
/// Gid: 559
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NonZeroCurvePointUncompressedStableV1Poly<Field, Boolean> {
    pub x: Field,
    pub is_odd: Boolean,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Non_zero_curve_point.Uncompressed.Stable.V1.t`
///
/// **Location**: [src/lib/non_zero_curve_point/non_zero_curve_point.ml:44:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/non_zero_curve_point.ml#L44)
///
/// **Gid**: 565
pub struct NonZeroCurvePointUncompressedStableV1(
    pub NonZeroCurvePointUncompressedStableV1Poly<crate::bigint::BigInt, bool>,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Consensus__Proof_of_stake.Data.Consensus_state.Value.Stable.V1.t`
///
/// **Location**: [src/lib/consensus/proof_of_stake.ml:1722:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1722)
///
/// **Gid**: 859
pub struct ConsensusProofOfStakeDataConsensusStateValueStableV1(
    pub  ConsensusProofOfStakeDataConsensusStateValueStableV1Poly<
        ConsensusProofOfStakeDataConsensusStateValueStableV1PolyArg0,
        ConsensusVrfOutputTruncatedStableV1,
        CurrencyMakeStrAmountMakeStrStableV1,
        ConsensusGlobalSlotStableV1,
        ConsensusGlobalSlotStableV1PolyArg0,
        ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
        ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
        bool,
        NonZeroCurvePointUncompressedStableV1,
    >,
);

/// Location: [src/lib/genesis_constants/genesis_constants.ml:239:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/genesis_constants/genesis_constants.ml#L239)
///
/// Gid: 592
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseProtocolConstantsCheckedValueStableV1Poly<Length, Delta, GenesisStateTimestamp> {
    pub k: Length,
    pub slots_per_epoch: Length,
    pub slots_per_sub_window: Length,
    pub delta: Delta,
    pub genesis_state_timestamp: GenesisStateTimestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Protocol_constants_checked.Value.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/protocol_constants_checked.ml:22:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/protocol_constants_checked.ml#L22)
///
/// **Gid**: 776
pub struct MinaBaseProtocolConstantsCheckedValueStableV1(
    pub  MinaBaseProtocolConstantsCheckedValueStableV1Poly<
        ConsensusProofOfStakeDataConsensusStateValueStableV1PolyArg0,
        ConsensusProofOfStakeDataConsensusStateValueStableV1PolyArg0,
        BlockTimeMakeStrTimeStableV1,
    >,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_state__Protocol_state.Body.Value.Stable.V2.t`
///
/// **Location**: [src/lib/mina_state/protocol_state.ml:53:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L53)
///
/// **Gid**: 870
pub struct MinaStateProtocolStateBodyValueStableV2(
    pub  MinaStateProtocolStateBodyValueStableV2Poly<
        DataHashLibStateHashStableV1,
        MinaStateBlockchainStateValueStableV2,
        ConsensusProofOfStakeDataConsensusStateValueStableV1,
        MinaBaseProtocolConstantsCheckedValueStableV1,
    >,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_state__Protocol_state.Value.Stable.V2.t`
///
/// **Location**: [src/lib/mina_state/protocol_state.ml:177:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L177)
///
/// **Gid**: 871
pub struct MinaStateProtocolStateValueStableV2(
    pub  MinaStateProtocolStateValueStableV2Poly<
        DataHashLibStateHashStableV1,
        MinaStateProtocolStateBodyValueStableV2,
    >,
);

/// Location: [src/lib/pickles/composition_types/composition_types.ml:206:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L206)
///
/// Gid: 519
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PolyStatementPolyProofStateDeferredValues<
    Plonk,
    ScalarChallenge,
    Fp,
    BulletproofChallenges,
    BranchData,
> {
    pub plonk: Plonk,
    pub combined_inner_product: Fp,
    pub b: Fp,
    pub xi: ScalarChallenge,
    pub bulletproof_challenges: BulletproofChallenges,
    pub branch_data: BranchData,
}

/// Location: [src/lib/pickles/composition_types/composition_types.ml:375:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L375)
///
/// Gid: 521
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PolyStatementPolyProofState<
    Plonk,
    ScalarChallenge,
    Fp,
    MessagesForNextWrapProof,
    Digest,
    BpChals,
    Index,
> {
    pub deferred_values:
        PicklesProofProofsVerified2ReprStableV2PolyStatementPolyProofStateDeferredValues<
            Plonk,
            ScalarChallenge,
            Fp,
            BpChals,
            Index,
        >,
    pub sponge_digest_before_evaluations: Digest,
    pub messages_for_next_wrap_proof: MessagesForNextWrapProof,
}

/// Location: [src/lib/pickles/composition_types/composition_types.ml:588:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L588)
///
/// Gid: 522
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PolyStatementPoly<
    Plonk,
    ScalarChallenge,
    Fp,
    MessagesForNextWrapProof,
    Digest,
    MessagesForNextStepProof,
    BpChals,
    Index,
> {
    pub proof_state: PicklesProofProofsVerified2ReprStableV2PolyStatementPolyProofState<
        Plonk,
        ScalarChallenge,
        Fp,
        MessagesForNextWrapProof,
        Digest,
        BpChals,
        Index,
    >,
    pub messages_for_next_step_proof: MessagesForNextStepProof,
}

/// Location: [src/lib/pickles/composition_types/composition_types.ml:45:14](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L45)
///
/// Gid: 518
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PolyStatementPolyArg0<Challenge, ScalarChallenge>
{
    pub alpha: ScalarChallenge,
    pub beta: Challenge,
    pub gamma: Challenge,
    pub zeta: ScalarChallenge,
    pub joint_combiner: Option<ScalarChallenge>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/pickles/composition_types/composition_types.ml:625:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L625)
///
/// Gid: 523
pub struct PicklesProofProofsVerified2ReprStableV2PolyStatement<
    Challenge,
    ScalarChallenge,
    Fp,
    MessagesForNextWrapProof,
    Digest,
    MessagesForNextStepProof,
    BpChals,
    Index,
>(
    pub  PicklesProofProofsVerified2ReprStableV2PolyStatementPoly<
        PicklesProofProofsVerified2ReprStableV2PolyStatementPolyArg0<Challenge, ScalarChallenge>,
        ScalarChallenge,
        Fp,
        MessagesForNextWrapProof,
        Digest,
        MessagesForNextStepProof,
        BpChals,
        Index,
    >,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Limb_vector__Constant.Hex64.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/limb_vector/constant.ml:60:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/limb_vector/constant.ml#L60)
///
/// **Gid**: 513
pub struct LimbVectorConstantHex64StableV1(pub crate::number::Int64);

/// Location: [src/lib/crypto/kimchi_backend/common/scalar_challenge.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/crypto/kimchi_backend/common/scalar_challenge.ml#L6)
///
/// Gid: 478
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PolyStatementArg1<F> {
    pub inner: F,
}

/// Location: [src/lib/pickles_types/shifted_value.ml:94:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/shifted_value.ml#L94)
///
/// Gid: 458
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum PicklesProofProofsVerified2ReprStableV2PolyStatementArg2<F> {
    ShiftedValue(F),
}

/// **Origin**: `Composition_types__Digest.Constant.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/composition_types/digest.ml:13:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/digest.ml#L13)
///
/// **Gid**: 517
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CompositionTypesDigestConstantStableV1(
    pub LimbVectorConstantHex64StableV1,
    pub  (
        LimbVectorConstantHex64StableV1,
        (
            LimbVectorConstantHex64StableV1,
            (LimbVectorConstantHex64StableV1, ()),
        ),
    ),
);

/// Location: [src/lib/crypto/kimchi_backend/pasta/basic.ml:53:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/crypto/kimchi_backend/pasta/basic.ml#L53)
///
/// Gid: 485
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PolyStatementArg6<A>(
    pub A,
    pub  (
        A,
        (
            A,
            (
                A,
                (A, (A, (A, (A, (A, (A, (A, (A, (A, (A, (A, (A, ())))))))))))),
            ),
        ),
    ),
);

/// Location: [src/lib/pickles/composition_types/bulletproof_challenge.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/bulletproof_challenge.ml#L6)
///
/// Gid: 516
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PolyStatementArg6Arg0<Challenge> {
    pub prechallenge: Challenge,
}

/// **Origin**: `Pickles_base__Proofs_verified.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_base/proofs_verified.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/proofs_verified.ml#L7)
///
/// **Gid**: 505
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum PicklesBaseProofsVerifiedStableV1 {
    N0,
    N1,
    N2,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Composition_types__Branch_data.Make_str.Domain_log2.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/composition_types/branch_data.ml:24:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/branch_data.ml#L24)
///
/// **Gid**: 514
pub struct CompositionTypesBranchDataMakeStrDomainLog2StableV1(pub crate::char::Char);

/// **Origin**: `Composition_types__Branch_data.Make_str.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/composition_types/branch_data.ml:51:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/branch_data.ml#L51)
///
/// **Gid**: 515
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CompositionTypesBranchDataMakeStrStableV1 {
    pub proofs_verified: PicklesBaseProofsVerifiedStableV1,
    pub domain_log2: CompositionTypesBranchDataMakeStrDomainLog2StableV1,
}

/// Location: [src/lib/pickles_types/plonk_types.ml:197:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_types.ml#L197)
///
/// Gid: 461
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PolyPrevEvalsEvalsEvalsLookupArg0<F> {
    pub sorted: Vec<F>,
    pub aggreg: F,
    pub table: F,
    pub runtime: Option<F>,
}

/// Location: [src/lib/pickles_types/plonk_types.ml:266:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_types.ml#L266)
///
/// Gid: 462
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PolyPrevEvalsEvalsEvals<A> {
    pub w: (
        A,
        (
            A,
            (
                A,
                (A, (A, (A, (A, (A, (A, (A, (A, (A, (A, (A, (A, ())))))))))))),
            ),
        ),
    ),
    pub z: A,
    pub s: (A, (A, (A, (A, (A, (A, ())))))),
    pub generic_selector: A,
    pub poseidon_selector: A,
    pub lookup: Option<PicklesProofProofsVerified2ReprStableV2PolyPrevEvalsEvalsEvalsLookupArg0<A>>,
}

/// Location: [src/lib/pickles_types/plonk_types.ml:456:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_types.ml#L456)
///
/// Gid: 463
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PolyPrevEvalsEvals<F, FMulti> {
    pub public_input: F,
    pub evals: PicklesProofProofsVerified2ReprStableV2PolyPrevEvalsEvalsEvals<FMulti>,
}

/// Location: [src/lib/pickles_types/plonk_types.ml:489:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_types.ml#L489)
///
/// Gid: 464
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PolyPrevEvals<F, FMulti> {
    pub evals: PicklesProofProofsVerified2ReprStableV2PolyPrevEvalsEvals<(F, F), (FMulti, FMulti)>,
    pub ft_eval1: F,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/pickles_types/plonk_types.ml:619:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_types.ml#L619)
///
/// Gid: 468
pub struct PicklesProofProofsVerified2ReprStableV2PolyProofPolyMessagesWComm<G>(pub Vec<G>);

/// Location: [src/lib/pickles_types/plonk_types.ml:639:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_types.ml#L639)
///
/// Gid: 469
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PolyProofPolyMessagesLookupArg0<G> {
    pub sorted: Vec<G>,
    pub aggreg: G,
    pub runtime: Option<G>,
}

/// Location: [src/lib/pickles_types/plonk_types.ml:689:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_types.ml#L689)
///
/// Gid: 470
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PolyProofPolyMessages < G > { pub w_comm : (PicklesProofProofsVerified2ReprStableV2PolyProofPolyMessagesWComm < G > , (PicklesProofProofsVerified2ReprStableV2PolyProofPolyMessagesWComm < G > , (PicklesProofProofsVerified2ReprStableV2PolyProofPolyMessagesWComm < G > , (PicklesProofProofsVerified2ReprStableV2PolyProofPolyMessagesWComm < G > , (PicklesProofProofsVerified2ReprStableV2PolyProofPolyMessagesWComm < G > , (PicklesProofProofsVerified2ReprStableV2PolyProofPolyMessagesWComm < G > , (PicklesProofProofsVerified2ReprStableV2PolyProofPolyMessagesWComm < G > , (PicklesProofProofsVerified2ReprStableV2PolyProofPolyMessagesWComm < G > , (PicklesProofProofsVerified2ReprStableV2PolyProofPolyMessagesWComm < G > , (PicklesProofProofsVerified2ReprStableV2PolyProofPolyMessagesWComm < G > , (PicklesProofProofsVerified2ReprStableV2PolyProofPolyMessagesWComm < G > , (PicklesProofProofsVerified2ReprStableV2PolyProofPolyMessagesWComm < G > , (PicklesProofProofsVerified2ReprStableV2PolyProofPolyMessagesWComm < G > , (PicklesProofProofsVerified2ReprStableV2PolyProofPolyMessagesWComm < G > , (PicklesProofProofsVerified2ReprStableV2PolyProofPolyMessagesWComm < G > , ()))))))))))))))) , pub z_comm : PicklesProofProofsVerified2ReprStableV2PolyProofPolyMessagesWComm < G > , pub t_comm : PicklesProofProofsVerified2ReprStableV2PolyProofPolyMessagesWComm < G > , pub lookup : Option < PicklesProofProofsVerified2ReprStableV2PolyProofPolyMessagesLookupArg0 < PicklesProofProofsVerified2ReprStableV2PolyProofPolyMessagesWComm < G > > > , }

/// Location: [src/lib/pickles_types/plonk_types.ml:536:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_types.ml#L536)
///
/// Gid: 465
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PolyProofPolyOpeningsProof<G, Fq> {
    pub lr: Vec<(G, G)>,
    pub z_1: Fq,
    pub z_2: Fq,
    pub delta: G,
    pub challenge_polynomial_commitment: G,
}

/// Location: [src/lib/pickles_types/plonk_types.ml:558:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_types.ml#L558)
///
/// Gid: 466
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PolyProofPolyOpenings<G, Fq, Fqv> {
    pub proof: PicklesProofProofsVerified2ReprStableV2PolyProofPolyOpeningsProof<G, Fq>,
    pub evals: PicklesProofProofsVerified2ReprStableV2PolyPrevEvalsEvalsEvals<(Fqv, Fqv)>,
    pub ft_eval1: Fq,
}

/// Location: [src/lib/pickles_types/plonk_types.ml:738:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_types.ml#L738)
///
/// Gid: 471
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PolyProofPoly<G, Fq, Fqv> {
    pub messages: PicklesProofProofsVerified2ReprStableV2PolyProofPolyMessages<G>,
    pub openings: PicklesProofProofsVerified2ReprStableV2PolyProofPolyOpenings<G, Fq, Fqv>,
}

/// Location: [src/lib/crypto/kimchi_backend/common/curve.ml:99:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/crypto/kimchi_backend/common/curve.ml#L99)
///
/// Gid: 487
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PolyProofPolyArg0(
    pub crate::bigint::BigInt,
    pub crate::bigint::BigInt,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/crypto/kimchi_backend/common/plonk_dlog_proof.ml:160:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/crypto/kimchi_backend/common/plonk_dlog_proof.ml#L160)
///
/// Gid: 493
pub struct PicklesProofProofsVerified2ReprStableV2PolyProof(
    pub  PicklesProofProofsVerified2ReprStableV2PolyProofPoly<
        PicklesProofProofsVerified2ReprStableV2PolyProofPolyArg0,
        crate::bigint::BigInt,
        Vec<crate::bigint::BigInt>,
    >,
);

/// Location: [src/lib/pickles/proof.ml:47:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L47)
///
/// Gid: 530
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2Poly<
    MessagesForNextWrapProof,
    MessagesForNextStepProof,
> {
    pub statement: PicklesProofProofsVerified2ReprStableV2PolyStatement<
        (
            LimbVectorConstantHex64StableV1,
            (LimbVectorConstantHex64StableV1, ()),
        ),
        PicklesProofProofsVerified2ReprStableV2PolyStatementArg1<(
            LimbVectorConstantHex64StableV1,
            (LimbVectorConstantHex64StableV1, ()),
        )>,
        PicklesProofProofsVerified2ReprStableV2PolyStatementArg2<crate::bigint::BigInt>,
        MessagesForNextWrapProof,
        CompositionTypesDigestConstantStableV1,
        MessagesForNextStepProof,
        PicklesProofProofsVerified2ReprStableV2PolyStatementArg6<
            PicklesProofProofsVerified2ReprStableV2PolyStatementArg6Arg0<
                PicklesProofProofsVerified2ReprStableV2PolyStatementArg1<(
                    LimbVectorConstantHex64StableV1,
                    (LimbVectorConstantHex64StableV1, ()),
                )>,
            >,
        >,
        CompositionTypesBranchDataMakeStrStableV1,
    >,
    pub prev_evals: PicklesProofProofsVerified2ReprStableV2PolyPrevEvals<
        crate::bigint::BigInt,
        Vec<crate::bigint::BigInt>,
    >,
    pub proof: PicklesProofProofsVerified2ReprStableV2PolyProof,
}

/// Location: [src/lib/pickles/composition_types/composition_types.ml:342:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L342)
///
/// Gid: 520
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PolyArg0<G1, BulletproofChallenges> {
    pub challenge_polynomial_commitment: G1,
    pub old_bulletproof_challenges: BulletproofChallenges,
}

/// Location: [src/lib/crypto/kimchi_backend/common/curve.ml:99:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/crypto/kimchi_backend/common/curve.ml#L99)
///
/// Gid: 486
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PolyArg0Arg0(
    pub crate::bigint::BigInt,
    pub crate::bigint::BigInt,
);

/// Location: [src/lib/crypto/kimchi_backend/pasta/basic.ml:32:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/crypto/kimchi_backend/pasta/basic.ml#L32)
///
/// Gid: 484
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2Poly<A>(
    pub A,
    pub  (
        A,
        (
            A,
            (A, (A, (A, (A, (A, (A, (A, (A, (A, (A, (A, (A, ())))))))))))),
        ),
    ),
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Pickles__Reduced_messages_for_next_proof_over_same_field.Wrap.Challenges_vector.Stable.V2.t`
///
/// **Location**: [src/lib/pickles/reduced_messages_for_next_proof_over_same_field.ml:53:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/reduced_messages_for_next_proof_over_same_field.ml#L53)
///
/// **Gid**: 527
pub struct PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2(
    pub  PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2Poly<
        PicklesProofProofsVerified2ReprStableV2PolyStatementArg6Arg0<
            PicklesProofProofsVerified2ReprStableV2PolyStatementArg1<(
                LimbVectorConstantHex64StableV1,
                (LimbVectorConstantHex64StableV1, ()),
            )>,
        >,
    >,
);

/// Location: [src/lib/pickles/reduced_messages_for_next_proof_over_same_field.ml:16:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/reduced_messages_for_next_proof_over_same_field.ml#L16)
///
/// Gid: 526
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PolyArg1<S, ChallengePolynomialCommitments, Bpcs>
{
    pub app_state: S,
    pub challenge_polynomial_commitments: ChallengePolynomialCommitments,
    pub old_bulletproof_challenges: Bpcs,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.t`
///
/// **Location**: [src/lib/pickles/proof.ml:340:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L340)
///
/// **Gid**: 531
pub struct PicklesProofProofsVerified2ReprStableV2(
    pub  PicklesProofProofsVerified2ReprStableV2Poly<
        PicklesProofProofsVerified2ReprStableV2PolyArg0<
            PicklesProofProofsVerified2ReprStableV2PolyArg0Arg0,
            (
                PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2,
                (
                    PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2,
                    (),
                ),
            ),
        >,
        PicklesProofProofsVerified2ReprStableV2PolyArg1<
            (),
            Vec<PicklesProofProofsVerified2ReprStableV2PolyProofPolyArg0>,
            Vec<
                PicklesProofProofsVerified2ReprStableV2PolyStatementArg6<
                    PicklesProofProofsVerified2ReprStableV2PolyStatementArg6Arg0<
                        PicklesProofProofsVerified2ReprStableV2PolyStatementArg1<(
                            LimbVectorConstantHex64StableV1,
                            (LimbVectorConstantHex64StableV1, ()),
                        )>,
                    >,
                >,
            >,
        >,
    >,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Proof.Stable.V2.t`
///
/// **Location**: [src/lib/mina_base/proof.ml:12:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/proof.ml#L12)
///
/// **Gid**: 777
pub struct MinaBaseProofStableV2(pub PicklesProofProofsVerified2ReprStableV2);

/// **Origin**: `Mina_base__State_body_hash.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/state_body_hash.ml:19:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/state_body_hash.ml#L19)
///
/// **Gid**: 648
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStateBodyHashStableV1(pub crate::bigint::BigInt);

/// **Origin**: `Protocol_version.Stable.V1.t`
///
/// **Location**: [src/lib/protocol_version/protocol_version.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/protocol_version/protocol_version.ml#L8)
///
/// **Gid**: 970
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ProtocolVersionStableV1 {
    pub major: crate::number::Int32,
    pub minor: crate::number::Int32,
    pub patch: crate::number::Int32,
}

/// **Origin**: `Mina_block__Header.Stable.V2.t`
///
/// **Location**: [src/lib/mina_block/header.ml:14:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_block/header.ml#L14)
///
/// **Gid**: 973
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBlockHeaderStableV2 {
    pub protocol_state: MinaStateProtocolStateValueStableV2,
    pub protocol_state_proof: MinaBaseProofStableV2,
    pub delta_block_chain_proof: (
        DataHashLibStateHashStableV1,
        Vec<MinaBaseStateBodyHashStableV1>,
    ),
    pub current_protocol_version: ProtocolVersionStableV1,
    pub proposed_protocol_version_opt: Option<ProtocolVersionStableV1>,
}

/// Location: [src/lib/staged_ledger_diff/diff.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/diff.ml#L10)
///
/// Gid: 920
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2PolyCoinbase<A> {
    Zero,
    One(Option<A>),
    Two(Option<(A, Option<A>)>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Currency.Make_str.Fee.Stable.V1.t`
///
/// **Location**: [src/lib/currency/currency.ml:898:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/currency.ml#L898)
///
/// **Gid**: 553
pub struct CurrencyMakeStrFeeStableV1(pub crate::number::Int64);

/// **Origin**: `Mina_base__Coinbase_fee_transfer.Make_str.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/coinbase_fee_transfer.ml:15:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/coinbase_fee_transfer.ml#L15)
///
/// **Gid**: 737
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseCoinbaseFeeTransferMakeStrStableV1 {
    pub receiver_pk: NonZeroCurvePointUncompressedStableV1,
    pub fee: CurrencyMakeStrFeeStableV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Staged_ledger_diff__Diff.Ft.Stable.V1.t`
///
/// **Location**: [src/lib/staged_ledger_diff/diff.ml:67:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/diff.ml#L67)
///
/// **Gid**: 922
pub struct StagedLedgerDiffDiffFtStableV1(pub MinaBaseCoinbaseFeeTransferMakeStrStableV1);

/// **Origin**: `Mina_base__Transaction_status.Stable.V2.t`
///
/// **Location**: [src/lib/mina_base/transaction_status.ml:423:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L423)
///
/// **Gid**: 683
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseTransactionStatusStableV2 {
    Applied,
    Failed(MinaBaseTransactionStatusFailureCollectionStableV1),
}

/// Location: [src/lib/staged_ledger_diff/diff.ml:83:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/diff.ml#L83)
///
/// Gid: 923
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2Poly<A, B> {
    pub completed_works: Vec<A>,
    pub commands: Vec<B>,
    pub coinbase: StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2PolyCoinbase<
        StagedLedgerDiffDiffFtStableV1,
    >,
    pub internal_command_statuses: Vec<MinaBaseTransactionStatusStableV2>,
}

/// Location: [src/lib/one_or_two/one_or_two.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/one_or_two/one_or_two.ml#L7)
///
/// Gid: 599
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[polymorphic_variant]
pub enum TransactionSnarkWorkTStableV2Proofs<A> {
    One(A),
    Two((A, A)),
}

/// Location: [src/lib/transaction_snark/transaction_snark.ml:122:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L122)
///
/// Gid: 904
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkStatementWithSokStableV2Poly<
    LedgerHash,
    Amount,
    PendingCoinbase,
    FeeExcess,
    SokDigest,
    LocalState,
> {
    pub source:
        MinaStateBlockchainStateValueStableV2PolyRegisters<LedgerHash, PendingCoinbase, LocalState>,
    pub target:
        MinaStateBlockchainStateValueStableV2PolyRegisters<LedgerHash, PendingCoinbase, LocalState>,
    pub supply_increase: Amount,
    pub fee_excess: FeeExcess,
    pub sok_digest: SokDigest,
}

/// Location: [src/lib/mina_base/pending_coinbase.ml:494:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L494)
///
/// Gid: 761
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStackVersionedStableV1Poly<DataStack, StateStack> {
    pub data: DataStack,
    pub state: StateStack,
}

/// **Origin**: `Mina_base__Pending_coinbase.Coinbase_stack.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/pending_coinbase.ml:152:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L152)
///
/// **Gid**: 743
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseCoinbaseStackStableV1(pub crate::bigint::BigInt);

/// Location: [src/lib/mina_base/pending_coinbase.ml:238:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L238)
///
/// Gid: 751
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStateStackStableV1Poly<StackHash> {
    pub init: StackHash,
    pub curr: StackHash,
}

/// **Origin**: `Mina_base__Pending_coinbase.Stack_hash.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/pending_coinbase.ml:212:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L212)
///
/// **Gid**: 748
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStackHashStableV1(pub crate::bigint::BigInt);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Pending_coinbase.State_stack.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/pending_coinbase.ml:247:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L247)
///
/// **Gid**: 752
pub struct MinaBasePendingCoinbaseStateStackStableV1(
    pub MinaBasePendingCoinbaseStateStackStableV1Poly<MinaBasePendingCoinbaseStackHashStableV1>,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Pending_coinbase.Stack_versioned.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/pending_coinbase.ml:504:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L504)
///
/// **Gid**: 762
pub struct MinaBasePendingCoinbaseStackVersionedStableV1(
    pub  MinaBasePendingCoinbaseStackVersionedStableV1Poly<
        MinaBasePendingCoinbaseCoinbaseStackStableV1,
        MinaBasePendingCoinbaseStateStackStableV1,
    >,
);

/// Location: [src/lib/mina_base/fee_excess.ml:54:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_excess.ml#L54)
///
/// Gid: 617
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeExcessStableV1Poly<Token, Fee> {
    pub fee_token_l: Token,
    pub fee_excess_l: Fee,
    pub fee_token_r: Token,
    pub fee_excess_r: Fee,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Fee_excess.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/fee_excess.ml:123:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_excess.ml#L123)
///
/// **Gid**: 618
pub struct MinaBaseFeeExcessStableV1(
    pub  MinaBaseFeeExcessStableV1Poly<
        MinaBaseAccountIdMakeStrDigestStableV1,
        MinaTransactionLogicPartiesLogicLocalStateValueStableV1PolyArg3<
            CurrencyMakeStrFeeStableV1,
            SgnStableV1,
        >,
    >,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Transaction_snark.Statement.With_sok.Stable.V2.t`
///
/// **Location**: [src/lib/transaction_snark/transaction_snark.ml:223:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L223)
///
/// **Gid**: 906
pub struct TransactionSnarkStatementWithSokStableV2(
    pub  TransactionSnarkStatementWithSokStableV2Poly<
        MinaBaseLedgerHash0StableV1,
        MinaTransactionLogicPartiesLogicLocalStateValueStableV1PolyArg3<
            CurrencyMakeStrAmountMakeStrStableV1,
            SgnStableV1,
        >,
        MinaBasePendingCoinbaseStackVersionedStableV1,
        MinaBaseFeeExcessStableV1,
        crate::string::ByteString,
        MinaTransactionLogicPartiesLogicLocalStateValueStableV1,
    >,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Transaction_snark.Proof.Stable.V2.t`
///
/// **Location**: [src/lib/transaction_snark/transaction_snark.ml:378:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L378)
///
/// **Gid**: 909
pub struct TransactionSnarkProofStableV2(pub PicklesProofProofsVerified2ReprStableV2);

/// **Origin**: `Transaction_snark.Stable.V2.t`
///
/// **Location**: [src/lib/transaction_snark/transaction_snark.ml:389:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L389)
///
/// **Gid**: 910
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkStableV2 {
    pub statement: TransactionSnarkStatementWithSokStableV2,
    pub proof: TransactionSnarkProofStableV2,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Ledger_proof.Prod.Stable.V2.t`
///
/// **Location**: [src/lib/ledger_proof/ledger_proof.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/ledger_proof/ledger_proof.ml#L10)
///
/// **Gid**: 913
pub struct LedgerProofProdStableV2(pub TransactionSnarkStableV2);

/// **Origin**: `Transaction_snark_work.T.Stable.V2.t`
///
/// **Location**: [src/lib/transaction_snark_work/transaction_snark_work.ml:82:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_work/transaction_snark_work.ml#L82)
///
/// **Gid**: 919
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkWorkTStableV2 {
    pub fee: CurrencyMakeStrFeeStableV1,
    pub proofs: TransactionSnarkWorkTStableV2Proofs<LedgerProofProdStableV2>,
    pub prover: NonZeroCurvePointUncompressedStableV1,
}

/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/with_status.ml#L6)
///
/// Gid: 728
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2PolyArg1<A> {
    pub data: A,
    pub status: MinaBaseTransactionStatusStableV2,
}

/// Location: [src/lib/mina_base/user_command.ml:7:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/user_command.ml#L7)
///
/// Gid: 729
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseUserCommandStableV2Poly<U, S> {
    SignedCommand(U),
    Parties(S),
}

/// Location: [src/lib/mina_base/signed_command.ml:25:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command.ml#L25)
///
/// Gid: 638
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandMakeStrStableV2Poly<Payload, Pk, Signature> {
    pub payload: Payload,
    pub signer: Pk,
    pub signature: Signature,
}

/// Location: [src/lib/mina_base/signed_command_payload.ml:257:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L257)
///
/// Gid: 636
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandPayloadStableV2Poly<Common, Body> {
    pub common: Common,
    pub body: Body,
}

/// Location: [src/lib/mina_base/signed_command_payload.ml:40:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L40)
///
/// Gid: 631
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandPayloadCommonStableV2Poly<Fee, PublicKey, Nonce, GlobalSlot, Memo> {
    pub fee: Fee,
    pub fee_payer_pk: PublicKey,
    pub nonce: Nonce,
    pub valid_until: GlobalSlot,
    pub memo: Memo,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_numbers/nat.ml:258:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L258)
///
/// Gid: 536
pub struct MinaBaseSignedCommandPayloadCommonStableV2PolyArg2(pub crate::number::Int32);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Signed_command_memo.Make_str.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command_memo.ml:19:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_memo.ml#L19)
///
/// **Gid**: 629
pub struct MinaBaseSignedCommandMemoMakeStrStableV1(pub crate::string::ByteString);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Signed_command_payload.Common.Stable.V2.t`
///
/// **Location**: [src/lib/mina_base/signed_command_payload.ml:80:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L80)
///
/// **Gid**: 633
pub struct MinaBaseSignedCommandPayloadCommonStableV2(
    pub  MinaBaseSignedCommandPayloadCommonStableV2Poly<
        CurrencyMakeStrFeeStableV1,
        NonZeroCurvePointUncompressedStableV1,
        MinaBaseSignedCommandPayloadCommonStableV2PolyArg2,
        ConsensusGlobalSlotStableV1PolyArg0,
        MinaBaseSignedCommandMemoMakeStrStableV1,
    >,
);

/// Location: [src/lib/mina_base/payment_payload.ml:14:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/payment_payload.ml#L14)
///
/// Gid: 619
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePaymentPayloadStableV2Poly<PublicKey, Amount> {
    pub source_pk: PublicKey,
    pub receiver_pk: PublicKey,
    pub amount: Amount,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Payment_payload.Stable.V2.t`
///
/// **Location**: [src/lib/mina_base/payment_payload.ml:27:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/payment_payload.ml#L27)
///
/// **Gid**: 620
pub struct MinaBasePaymentPayloadStableV2(
    pub  MinaBasePaymentPayloadStableV2Poly<
        NonZeroCurvePointUncompressedStableV1,
        CurrencyMakeStrAmountMakeStrStableV1,
    >,
);

/// **Origin**: `Mina_base__Stake_delegation.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/stake_delegation.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/stake_delegation.ml#L9)
///
/// **Gid**: 630
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseStakeDelegationStableV1 {
    SetDelegate {
        delegator: NonZeroCurvePointUncompressedStableV1,
        new_delegate: NonZeroCurvePointUncompressedStableV1,
    },
}

/// **Origin**: `Mina_base__Signed_command_payload.Body.Stable.V2.t`
///
/// **Location**: [src/lib/mina_base/signed_command_payload.ml:190:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L190)
///
/// **Gid**: 635
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSignedCommandPayloadBodyStableV2 {
    Payment(MinaBasePaymentPayloadStableV2),
    StakeDelegation(MinaBaseStakeDelegationStableV1),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Signed_command_payload.Stable.V2.t`
///
/// **Location**: [src/lib/mina_base/signed_command_payload.ml:275:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L275)
///
/// **Gid**: 637
pub struct MinaBaseSignedCommandPayloadStableV2(
    pub  MinaBaseSignedCommandPayloadStableV2Poly<
        MinaBaseSignedCommandPayloadCommonStableV2,
        MinaBaseSignedCommandPayloadBodyStableV2,
    >,
);

/// Location: [src/lib/mina_base/signature_poly.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signature_poly.ml#L6)
///
/// Gid: 613
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignatureStableV1Poly<Field, Scalar>(pub Field, pub Scalar);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Signature.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signature.ml:18:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signature.ml#L18)
///
/// **Gid**: 615
pub struct MinaBaseSignatureStableV1(
    pub MinaBaseSignatureStableV1Poly<crate::bigint::BigInt, crate::bigint::BigInt>,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Signed_command.Make_str.Stable.V2.t`
///
/// **Location**: [src/lib/mina_base/signed_command.ml:39:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command.ml#L39)
///
/// **Gid**: 639
pub struct MinaBaseSignedCommandMakeStrStableV2(
    pub  MinaBaseSignedCommandMakeStrStableV2Poly<
        MinaBaseSignedCommandPayloadStableV2,
        NonZeroCurvePointUncompressedStableV1,
        MinaBaseSignatureStableV1,
    >,
);

/// **Origin**: `Mina_base__Party.Body.Fee_payer.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/party.ml:963:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/party.ml#L963)
///
/// **Gid**: 706
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePartyBodyFeePayerStableV1 {
    pub public_key: NonZeroCurvePointUncompressedStableV1,
    pub fee: CurrencyMakeStrFeeStableV1,
    pub valid_until: Option<ConsensusGlobalSlotStableV1PolyArg0>,
    pub nonce: MinaBaseSignedCommandPayloadCommonStableV2PolyArg2,
}

/// **Origin**: `Mina_base__Party.Fee_payer.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/party.ml:1355:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/party.ml#L1355)
///
/// **Gid**: 711
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePartyFeePayerStableV1 {
    pub body: MinaBasePartyBodyFeePayerStableV1,
    pub authorization: MinaBaseSignatureStableV1,
}

/// Location: [src/lib/mina_base/with_stack_hash.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/with_stack_hash.ml#L6)
///
/// Gid: 712
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePartiesTStableV1OtherPartiesPolyArg0<A, Field> {
    pub elt: A,
    pub stack_hash: Field,
}

/// Location: [src/lib/mina_base/parties.ml:45:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/parties.ml#L45)
///
/// Gid: 713
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePartiesTStableV1OtherPartiesPolyArg0Arg0<Party, PartyDigest, Digest> {
    pub party: Party,
    pub party_digest: PartyDigest,
    pub calls: Vec<
        MinaBasePartiesTStableV1OtherPartiesPolyArg0<
            Box<MinaBasePartiesTStableV1OtherPartiesPolyArg0Arg0<Party, PartyDigest, Digest>>,
            Digest,
        >,
    >,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/parties.ml:315:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/parties.ml#L315)
///
/// Gid: 717
pub struct MinaBasePartiesTStableV1OtherParties<Party, PartyDigest, Digest>(
    pub  Vec<
        MinaBasePartiesTStableV1OtherPartiesPolyArg0<
            MinaBasePartiesTStableV1OtherPartiesPolyArg0Arg0<Party, PartyDigest, Digest>,
            Digest,
        >,
    >,
);

/// Location: [src/lib/mina_base/zkapp_state.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_state.ml#L17)
///
/// Gid: 658
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePartyUpdateStableV1AppState<A>(pub A, pub (A, (A, (A, (A, (A, (A, (A, ()))))))));

/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_basic.ml#L100)
///
/// Gid: 655
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBasePartyUpdateStableV1AppStateArg0<A> {
    Set(A),
    Keep,
}

/// Location: [src/lib/pickles_types/plonk_verification_key_evals.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_verification_key_evals.ml#L9)
///
/// Gid: 473
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseVerificationKeyWireStableV1PolyWrapIndex<Comm> {
    pub sigma_comm: (Comm, (Comm, (Comm, (Comm, (Comm, (Comm, (Comm, ()))))))),
    pub coefficients_comm: (
        Comm,
        (
            Comm,
            (
                Comm,
                (
                    Comm,
                    (
                        Comm,
                        (
                            Comm,
                            (
                                Comm,
                                (
                                    Comm,
                                    (Comm, (Comm, (Comm, (Comm, (Comm, (Comm, (Comm, ()))))))),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    ),
    pub generic_comm: Comm,
    pub psm_comm: Comm,
    pub complete_add_comm: Comm,
    pub mul_comm: Comm,
    pub emul_comm: Comm,
    pub endomul_scalar_comm: Comm,
}

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:150:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L150)
///
/// Gid: 511
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseVerificationKeyWireStableV1Poly<G> {
    pub max_proofs_verified: PicklesBaseProofsVerifiedStableV1,
    pub wrap_index: MinaBaseVerificationKeyWireStableV1PolyWrapIndex<G>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Verification_key_wire.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/side_loaded_verification_key.ml:170:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/side_loaded_verification_key.ml#L170)
///
/// **Gid**: 528
pub struct MinaBaseVerificationKeyWireStableV1(
    pub  MinaBaseVerificationKeyWireStableV1Poly<
        PicklesProofProofsVerified2ReprStableV2PolyProofPolyArg0,
    >,
);

/// Location: [src/lib/mina_base/permissions.ml:319:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/permissions.ml#L319)
///
/// Gid: 627
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePermissionsStableV2Poly<Controller> {
    pub edit_state: Controller,
    pub send: Controller,
    pub receive: Controller,
    pub set_delegate: Controller,
    pub set_permissions: Controller,
    pub set_verification_key: Controller,
    pub set_zkapp_uri: Controller,
    pub edit_sequence_state: Controller,
    pub set_token_symbol: Controller,
    pub increment_nonce: Controller,
    pub set_voting_for: Controller,
}

/// **Origin**: `Mina_base__Permissions.Auth_required.Stable.V2.t`
///
/// **Location**: [src/lib/mina_base/permissions.ml:53:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/permissions.ml#L53)
///
/// **Gid**: 626
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBasePermissionsAuthRequiredStableV2 {
    None,
    Either,
    Proof,
    Signature,
    Impossible,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Permissions.Stable.V2.t`
///
/// **Location**: [src/lib/mina_base/permissions.ml:352:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/permissions.ml#L352)
///
/// **Gid**: 628
pub struct MinaBasePermissionsStableV2(
    pub MinaBasePermissionsStableV2Poly<MinaBasePermissionsAuthRequiredStableV2>,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Currency.Make_str.Balance.Stable.V1.t`
///
/// **Location**: [src/lib/currency/currency.ml:1072:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/currency.ml#L1072)
///
/// **Gid**: 555
pub struct CurrencyMakeStrBalanceStableV1(pub CurrencyMakeStrAmountMakeStrStableV1);

/// **Origin**: `Mina_base__Party.Update.Timing_info.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/party.ml:64:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/party.ml#L64)
///
/// **Gid**: 697
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePartyUpdateTimingInfoStableV1 {
    pub initial_minimum_balance: CurrencyMakeStrBalanceStableV1,
    pub cliff_time: ConsensusGlobalSlotStableV1PolyArg0,
    pub cliff_amount: CurrencyMakeStrAmountMakeStrStableV1,
    pub vesting_period: ConsensusGlobalSlotStableV1PolyArg0,
    pub vesting_increment: CurrencyMakeStrAmountMakeStrStableV1,
}

/// **Origin**: `Mina_base__Party.Update.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/party.ml:219:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/party.ml#L219)
///
/// **Gid**: 698
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePartyUpdateStableV1 {
    pub app_state: MinaBasePartyUpdateStableV1AppState<
        MinaBasePartyUpdateStableV1AppStateArg0<crate::bigint::BigInt>,
    >,
    pub delegate: MinaBasePartyUpdateStableV1AppStateArg0<NonZeroCurvePointUncompressedStableV1>,
    pub verification_key:
        MinaBasePartyUpdateStableV1AppStateArg0<MinaBaseVerificationKeyWireStableV1>,
    pub permissions: MinaBasePartyUpdateStableV1AppStateArg0<MinaBasePermissionsStableV2>,
    pub zkapp_uri: MinaBasePartyUpdateStableV1AppStateArg0<crate::string::ByteString>,
    pub token_symbol: MinaBasePartyUpdateStableV1AppStateArg0<crate::string::ByteString>,
    pub timing: MinaBasePartyUpdateStableV1AppStateArg0<MinaBasePartyUpdateTimingInfoStableV1>,
    pub voting_for: MinaBasePartyUpdateStableV1AppStateArg0<DataHashLibStateHashStableV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Party.Body.Events'.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/party.ml:729:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/party.ml#L729)
///
/// **Gid**: 701
pub struct MinaBasePartyBodyEventsStableV1(pub Vec<Vec<crate::bigint::BigInt>>);

/// Location: [src/lib/mina_base/zkapp_precondition.ml:921:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_precondition.ml#L921)
///
/// Gid: 688
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionProtocolStateStableV1Poly<
    SnarkedLedgerHash,
    Time,
    Length,
    VrfOutput,
    GlobalSlot,
    Amount,
    EpochData,
> {
    pub snarked_ledger_hash: SnarkedLedgerHash,
    pub timestamp: Time,
    pub blockchain_length: Length,
    pub min_window_density: Length,
    pub last_vrf_output: VrfOutput,
    pub total_currency: Amount,
    pub global_slot_since_hard_fork: GlobalSlot,
    pub global_slot_since_genesis: GlobalSlot,
    pub staking_epoch_data: EpochData,
    pub next_epoch_data: EpochData,
}

/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_basic.ml#L232)
///
/// Gid: 656
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionProtocolStateStableV1PolyArg0<A> {
    Check(A),
    Ignore,
}

/// Location: [src/lib/mina_base/zkapp_precondition.ml:23:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_precondition.ml#L23)
///
/// Gid: 684
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionProtocolStateStableV1PolyArg0Arg0<A> {
    pub lower: A,
    pub upper: A,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/zkapp_precondition.ml:178:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_precondition.ml#L178)
///
/// Gid: 685
pub struct MinaBaseZkappPreconditionProtocolStateStableV1PolyArg1<A>(
    pub  MinaBaseZkappPreconditionProtocolStateStableV1PolyArg0<
        MinaBaseZkappPreconditionProtocolStateStableV1PolyArg0Arg0<A>,
    >,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Zkapp_precondition.Protocol_state.Epoch_data.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/zkapp_precondition.ml:790:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_precondition.ml#L790)
///
/// **Gid**: 687
pub struct MinaBaseZkappPreconditionProtocolStateEpochDataStableV1(
    pub  ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1Poly<
        MinaBaseEpochLedgerValueStableV1Poly<
            MinaBaseZkappPreconditionProtocolStateStableV1PolyArg0<MinaBaseLedgerHash0StableV1>,
            MinaBaseZkappPreconditionProtocolStateStableV1PolyArg1<
                CurrencyMakeStrAmountMakeStrStableV1,
            >,
        >,
        MinaBaseZkappPreconditionProtocolStateStableV1PolyArg0<MinaBaseEpochSeedStableV1>,
        MinaBaseZkappPreconditionProtocolStateStableV1PolyArg0<DataHashLibStateHashStableV1>,
        MinaBaseZkappPreconditionProtocolStateStableV1PolyArg0<DataHashLibStateHashStableV1>,
        MinaBaseZkappPreconditionProtocolStateStableV1PolyArg1<
            ConsensusProofOfStakeDataConsensusStateValueStableV1PolyArg0,
        >,
    >,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/zkapp_precondition.ml:970:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_precondition.ml#L970)
///
/// **Gid**: 689
pub struct MinaBaseZkappPreconditionProtocolStateStableV1(
    pub  MinaBaseZkappPreconditionProtocolStateStableV1Poly<
        MinaBaseZkappPreconditionProtocolStateStableV1PolyArg0<MinaBaseLedgerHash0StableV1>,
        MinaBaseZkappPreconditionProtocolStateStableV1PolyArg1<BlockTimeMakeStrTimeStableV1>,
        MinaBaseZkappPreconditionProtocolStateStableV1PolyArg1<
            ConsensusProofOfStakeDataConsensusStateValueStableV1PolyArg0,
        >,
        (),
        MinaBaseZkappPreconditionProtocolStateStableV1PolyArg1<ConsensusGlobalSlotStableV1PolyArg0>,
        MinaBaseZkappPreconditionProtocolStateStableV1PolyArg1<
            CurrencyMakeStrAmountMakeStrStableV1,
        >,
        MinaBaseZkappPreconditionProtocolStateEpochDataStableV1,
    >,
);

/// **Origin**: `Mina_base__Receipt.Chain_hash.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/receipt.ml:31:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/receipt.ml#L31)
///
/// **Gid**: 643
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseReceiptChainHashStableV1(pub crate::bigint::BigInt);

/// **Origin**: `Mina_base__Zkapp_precondition.Account.Stable.V2.t`
///
/// **Location**: [src/lib/mina_base/zkapp_precondition.ml:474:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_precondition.ml#L474)
///
/// **Gid**: 686
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionAccountStableV2 {
    pub balance:
        MinaBaseZkappPreconditionProtocolStateStableV1PolyArg1<CurrencyMakeStrBalanceStableV1>,
    pub nonce: MinaBaseZkappPreconditionProtocolStateStableV1PolyArg1<
        MinaBaseSignedCommandPayloadCommonStableV2PolyArg2,
    >,
    pub receipt_chain_hash:
        MinaBaseZkappPreconditionProtocolStateStableV1PolyArg0<MinaBaseReceiptChainHashStableV1>,
    pub delegate: MinaBaseZkappPreconditionProtocolStateStableV1PolyArg0<
        NonZeroCurvePointUncompressedStableV1,
    >,
    pub state: MinaBasePartyUpdateStableV1AppState<
        MinaBaseZkappPreconditionProtocolStateStableV1PolyArg0<crate::bigint::BigInt>,
    >,
    pub sequence_state:
        MinaBaseZkappPreconditionProtocolStateStableV1PolyArg0<crate::bigint::BigInt>,
    pub proved_state: MinaBaseZkappPreconditionProtocolStateStableV1PolyArg0<bool>,
    pub is_new: MinaBaseZkappPreconditionProtocolStateStableV1PolyArg0<bool>,
}

/// **Origin**: `Mina_base__Party.Account_precondition.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/party.ml:510:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/party.ml#L510)
///
/// **Gid**: 699
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBasePartyAccountPreconditionStableV1 {
    Full(MinaBaseZkappPreconditionAccountStableV2),
    Nonce(MinaBaseSignedCommandPayloadCommonStableV2PolyArg2),
    Accept,
}

/// **Origin**: `Mina_base__Party.Preconditions.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/party.ml:653:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/party.ml#L653)
///
/// **Gid**: 700
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePartyPreconditionsStableV1 {
    pub network: MinaBaseZkappPreconditionProtocolStateStableV1,
    pub account: MinaBasePartyAccountPreconditionStableV1,
}

/// **Origin**: `Mina_base__Party.Call_type.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/party.ml:27:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/party.ml#L27)
///
/// **Gid**: 696
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBasePartyCallTypeStableV1 {
    Call,
    DelegateCall,
}

/// **Origin**: `Mina_base__Party.Body.Wire.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/party.ml:741:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/party.ml#L741)
///
/// **Gid**: 702
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePartyBodyWireStableV1 {
    pub public_key: NonZeroCurvePointUncompressedStableV1,
    pub token_id: MinaBaseAccountIdMakeStrDigestStableV1,
    pub update: MinaBasePartyUpdateStableV1,
    pub balance_change: MinaTransactionLogicPartiesLogicLocalStateValueStableV1PolyArg3<
        CurrencyMakeStrAmountMakeStrStableV1,
        SgnStableV1,
    >,
    pub increment_nonce: bool,
    pub events: MinaBasePartyBodyEventsStableV1,
    pub sequence_events: MinaBasePartyBodyEventsStableV1,
    pub call_data: crate::bigint::BigInt,
    pub preconditions: MinaBasePartyPreconditionsStableV1,
    pub use_full_commitment: bool,
    pub caller: MinaBasePartyCallTypeStableV1,
}

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:66:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L66)
///
/// Gid: 507
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PolyArg0Arg1<A>(pub A, pub (A, ()));

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:87:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L87)
///
/// Gid: 508
pub struct PicklesProofProofsVerified2ReprStableV2PolyArg1Arg1<A>(pub Vec<A>);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Pickles__Proof.Proofs_verified_max.Stable.V2.t`
///
/// **Location**: [src/lib/pickles/proof.ml:413:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L413)
///
/// **Gid**: 532
pub struct PicklesProofProofsVerifiedMaxStableV2(
    pub  PicklesProofProofsVerified2ReprStableV2Poly<
        PicklesProofProofsVerified2ReprStableV2PolyArg0<
            PicklesProofProofsVerified2ReprStableV2PolyArg0Arg0,
            PicklesProofProofsVerified2ReprStableV2PolyArg0Arg1<
                PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2,
            >,
        >,
        PicklesProofProofsVerified2ReprStableV2PolyArg1<
            (),
            PicklesProofProofsVerified2ReprStableV2PolyArg1Arg1<
                PicklesProofProofsVerified2ReprStableV2PolyProofPolyArg0,
            >,
            PicklesProofProofsVerified2ReprStableV2PolyArg1Arg1<
                PicklesProofProofsVerified2ReprStableV2PolyStatementArg6<
                    PicklesProofProofsVerified2ReprStableV2PolyStatementArg6Arg0<
                        PicklesProofProofsVerified2ReprStableV2PolyStatementArg1<(
                            LimbVectorConstantHex64StableV1,
                            (LimbVectorConstantHex64StableV1, ()),
                        )>,
                    >,
                >,
            >,
        >,
    >,
);

/// **Origin**: `Mina_base__Control.Stable.V2.t`
///
/// **Location**: [src/lib/mina_base/control.ml:11:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/control.ml#L11)
///
/// **Gid**: 616
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseControlStableV2 {
    Proof(PicklesProofProofsVerifiedMaxStableV2),
    Signature(MinaBaseSignatureStableV1),
    NoneGiven,
}

/// **Origin**: `Mina_base__Party.T.Wire.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/party.ml:1281:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/party.ml#L1281)
///
/// **Gid**: 709
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePartyTWireStableV1 {
    pub body: MinaBasePartyBodyWireStableV1,
    pub authorization: MinaBaseControlStableV2,
}

/// **Origin**: `Mina_base__Parties.T.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/parties.ml:876:12](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/parties.ml#L876)
///
/// **Gid**: 722
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePartiesTStableV1 {
    pub fee_payer: MinaBasePartyFeePayerStableV1,
    pub other_parties: MinaBasePartiesTStableV1OtherParties<MinaBasePartyTWireStableV1, (), ()>,
    pub memo: MinaBaseSignedCommandMemoMakeStrStableV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__User_command.Stable.V2.t`
///
/// **Location**: [src/lib/mina_base/user_command.ml:67:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/user_command.ml#L67)
///
/// **Gid**: 731
pub struct MinaBaseUserCommandStableV2(
    pub  MinaBaseUserCommandStableV2Poly<
        MinaBaseSignedCommandMakeStrStableV2,
        MinaBasePartiesTStableV1,
    >,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Staged_ledger_diff__Diff.Pre_diff_with_at_most_two_coinbase.Stable.V2.t`
///
/// **Location**: [src/lib/staged_ledger_diff/diff.ml:147:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/diff.ml#L147)
///
/// **Gid**: 925
pub struct StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2(
    pub  StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2Poly<
        TransactionSnarkWorkTStableV2,
        StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2PolyArg1<
            MinaBaseUserCommandStableV2,
        >,
    >,
);

/// Location: [src/lib/staged_ledger_diff/diff.ml:43:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/diff.ml#L43)
///
/// Gid: 921
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2PolyCoinbase<A> {
    Zero,
    One(Option<A>),
}

/// Location: [src/lib/staged_ledger_diff/diff.ml:115:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/diff.ml#L115)
///
/// Gid: 924
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2Poly<A, B> {
    pub completed_works: Vec<A>,
    pub commands: Vec<B>,
    pub coinbase: StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2PolyCoinbase<
        StagedLedgerDiffDiffFtStableV1,
    >,
    pub internal_command_statuses: Vec<MinaBaseTransactionStatusStableV2>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Staged_ledger_diff__Diff.Pre_diff_with_at_most_one_coinbase.Stable.V2.t`
///
/// **Location**: [src/lib/staged_ledger_diff/diff.ml:166:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/diff.ml#L166)
///
/// **Gid**: 926
pub struct StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2(
    pub  StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2Poly<
        TransactionSnarkWorkTStableV2,
        StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2PolyArg1<
            MinaBaseUserCommandStableV2,
        >,
    >,
);

/// **Origin**: `Staged_ledger_diff__Diff.Diff.Stable.V2.t`
///
/// **Location**: [src/lib/staged_ledger_diff/diff.ml:185:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/diff.ml#L185)
///
/// **Gid**: 927
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffDiffStableV2(
    pub StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2,
    pub Option<StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2>,
);

/// **Origin**: `Staged_ledger_diff__Diff.Stable.V2.t`
///
/// **Location**: [src/lib/staged_ledger_diff/diff.ml:202:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/diff.ml#L202)
///
/// **Gid**: 928
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffStableV2 {
    pub diff: StagedLedgerDiffDiffDiffStableV2,
}

/// **Origin**: `Staged_ledger_diff__Body.Stable.V1.t`
///
/// **Location**: [src/lib/staged_ledger_diff/body.ml:12:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/body.ml#L12)
///
/// **Gid**: 929
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffBodyStableV1 {
    pub staged_ledger_diff: StagedLedgerDiffDiffStableV2,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Transaction_snark.Statement.Stable.V2.t`
///
/// **Location**: [src/lib/transaction_snark/transaction_snark.ml:205:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L205)
///
/// **Gid**: 905
pub struct TransactionSnarkStatementStableV2(
    pub  TransactionSnarkStatementWithSokStableV2Poly<
        MinaBaseLedgerHash0StableV1,
        MinaTransactionLogicPartiesLogicLocalStateValueStableV1PolyArg3<
            CurrencyMakeStrAmountMakeStrStableV1,
            SgnStableV1,
        >,
        MinaBasePendingCoinbaseStackVersionedStableV1,
        MinaBaseFeeExcessStableV1,
        (),
        MinaTransactionLogicPartiesLogicLocalStateValueStableV1,
    >,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Transaction_snark_work.Statement.Stable.V2.t`
///
/// **Location**: [src/lib/transaction_snark_work/transaction_snark_work.ml:23:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_work/transaction_snark_work.ml#L23)
///
/// **Gid**: 915
pub struct TransactionSnarkWorkStatementStableV2(
    pub TransactionSnarkWorkTStableV2Proofs<TransactionSnarkStatementStableV2>,
);

/// **Origin**: `Mina_base__Fee_with_prover.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/fee_with_prover.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_with_prover.ml#L7)
///
/// **Gid**: 780
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeWithProverStableV1 {
    pub fee: CurrencyMakeStrFeeStableV1,
    pub prover: NonZeroCurvePointUncompressedStableV1,
}

/// Location: [src/lib/network_pool/priced_proof.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_pool/priced_proof.ml#L9)
///
/// Gid: 1000
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NetworkPoolSnarkPoolDiffVersionedStableV2AddSolvedWork1<Proof> {
    pub proof: Proof,
    pub fee: MinaBaseFeeWithProverStableV1,
}

/// Location: [src/lib/sparse_ledger_lib/sparse_ledger.ml:9:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/sparse_ledger_lib/sparse_ledger.ml#L9)
///
/// Gid: 597
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSparseLedgerBaseStableV2PolyTree<Hash, Account> {
    Account(Account),
    Hash(Hash),
    Node(
        Hash,
        Box<MinaBaseSparseLedgerBaseStableV2PolyTree<Hash, Account>>,
        Box<MinaBaseSparseLedgerBaseStableV2PolyTree<Hash, Account>>,
    ),
}

/// Location: [src/lib/sparse_ledger_lib/sparse_ledger.ml:38:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/sparse_ledger_lib/sparse_ledger.ml#L38)
///
/// Gid: 598
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSparseLedgerBaseStableV2Poly<Hash, Key, Account> {
    pub indexes: Vec<(Key, crate::number::Int32)>,
    pub depth: crate::number::Int32,
    pub tree: MinaBaseSparseLedgerBaseStableV2PolyTree<Hash, Account>,
}

/// **Origin**: `Mina_base__Account_id.Make_str.Stable.V2.t`
///
/// **Location**: [src/lib/mina_base/account_id.ml:147:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account_id.ml#L147)
///
/// **Gid**: 606
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountIdMakeStrStableV2(
    pub NonZeroCurvePointUncompressedStableV1,
    pub MinaBaseAccountIdMakeStrDigestStableV1,
);

/// Location: [src/lib/mina_base/account.ml:226:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account.ml#L226)
///
/// Gid: 667
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountBinableArgStableV2Poly<
    Pk,
    Id,
    TokenPermissions,
    TokenSymbol,
    Amount,
    Nonce,
    ReceiptChainHash,
    Delegate,
    StateHash,
    Timing,
    Permissions,
    ZkappOpt,
    ZkappUri,
> {
    pub public_key: Pk,
    pub token_id: Id,
    pub token_permissions: TokenPermissions,
    pub token_symbol: TokenSymbol,
    pub balance: Amount,
    pub nonce: Nonce,
    pub receipt_chain_hash: ReceiptChainHash,
    pub delegate: Delegate,
    pub voting_for: StateHash,
    pub timing: Timing,
    pub permissions: Permissions,
    pub zkapp: ZkappOpt,
    pub zkapp_uri: ZkappUri,
}

/// **Origin**: `Mina_base__Token_permissions.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/token_permissions.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/token_permissions.ml#L9)
///
/// **Gid**: 653
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseTokenPermissionsStableV1 {
    TokenOwned { disable_new_accounts: bool },
    NotOwned { account_disabled: bool },
}

/// Location: [src/lib/mina_base/account_timing.ml:13:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account_timing.ml#L13)
///
/// Gid: 611
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountTimingStableV1Poly<Slot, Balance, Amount> {
    Untimed,
    Timed {
        initial_minimum_balance: Balance,
        cliff_time: Slot,
        cliff_amount: Amount,
        vesting_period: Slot,
        vesting_increment: Amount,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Account_timing.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/account_timing.ml:30:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account_timing.ml#L30)
///
/// **Gid**: 612
pub struct MinaBaseAccountTimingStableV1(
    pub  MinaBaseAccountTimingStableV1Poly<
        ConsensusGlobalSlotStableV1PolyArg0,
        CurrencyMakeStrBalanceStableV1,
        CurrencyMakeStrAmountMakeStrStableV1,
    >,
);

/// Location: [src/lib/mina_base/zkapp_account.ml:188:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_account.ml#L188)
///
/// Gid: 660
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappAccountStableV2Poly<AppState, Vk, ZkappVersion, Field, Slot, Bool> {
    pub app_state: AppState,
    pub verification_key: Vk,
    pub zkapp_version: ZkappVersion,
    pub sequence_state: (Field, (Field, (Field, (Field, (Field, ()))))),
    pub last_sequence_slot: Slot,
    pub proved_state: Bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Zkapp_state.Value.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/zkapp_state.ml:50:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_state.ml#L50)
///
/// **Gid**: 659
pub struct MinaBaseZkappStateValueStableV1(
    pub MinaBasePartyUpdateStableV1AppState<crate::bigint::BigInt>,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_numbers__Nat.Make32.Stable.V1.t`
///
/// **Location**: [src/lib/mina_numbers/nat.ml:258:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L258)
///
/// **Gid**: 535
pub struct MinaNumbersNatMake32StableV1(pub crate::number::Int32);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Zkapp_account.Stable.V2.t`
///
/// **Location**: [src/lib/mina_base/zkapp_account.ml:216:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_account.ml#L216)
///
/// **Gid**: 661
pub struct MinaBaseZkappAccountStableV2(
    pub  MinaBaseZkappAccountStableV2Poly<
        MinaBaseZkappStateValueStableV1,
        Option<MinaBaseVerificationKeyWireStableV1>,
        MinaNumbersNatMake32StableV1,
        crate::bigint::BigInt,
        ConsensusGlobalSlotStableV1PolyArg0,
        bool,
    >,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Account.Binable_arg.Stable.V2.t`
///
/// **Location**: [src/lib/mina_base/account.ml:313:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account.ml#L313)
///
/// **Gid**: 670
pub struct MinaBaseAccountBinableArgStableV2(
    pub  MinaBaseAccountBinableArgStableV2Poly<
        NonZeroCurvePointUncompressedStableV1,
        MinaBaseAccountIdMakeStrDigestStableV1,
        MinaBaseTokenPermissionsStableV1,
        crate::string::ByteString,
        CurrencyMakeStrBalanceStableV1,
        MinaBaseSignedCommandPayloadCommonStableV2PolyArg2,
        MinaBaseReceiptChainHashStableV1,
        Option<NonZeroCurvePointUncompressedStableV1>,
        DataHashLibStateHashStableV1,
        MinaBaseAccountTimingStableV1,
        MinaBasePermissionsStableV2,
        Option<MinaBaseZkappAccountStableV2>,
        crate::string::ByteString,
    >,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Network_peer__Peer.Id.Stable.V1.t`
///
/// **Location**: [src/lib/network_peer/peer.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_peer/peer.ml#L10)
///
/// **Gid**: 809
pub struct NetworkPeerPeerIdStableV1(pub crate::string::ByteString);

/// Location: [src/lib/non_empty_list/non_empty_list.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_empty_list/non_empty_list.ml#L7)
///
/// Gid: 600
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateStableV2PolyTrees<A>(pub A, pub Vec<A>);

//  The type `TransactionSnarkScanStateStableV2PolyTreesArg0` is skipped
//
//  Location: [src/lib/parallel_scan/parallel_scan.ml:247:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L247)
//
//  Gid: 947

/// **Origin**: `Parallel_scan.Weight.Stable.V1.t`
///
/// **Location**: [src/lib/parallel_scan/parallel_scan.ml:53:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L53)
///
/// **Gid**: 936
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ParallelScanWeightStableV1 {
    pub base: crate::number::Int32,
    pub merge: crate::number::Int32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Parallel_scan.Sequence_number.Stable.V1.t`
///
/// **Location**: [src/lib/parallel_scan/parallel_scan.ml:22:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L22)
///
/// **Gid**: 934
pub struct ParallelScanSequenceNumberStableV1(pub crate::number::Int32);

/// **Origin**: `Parallel_scan.Job_status.Stable.V1.t`
///
/// **Location**: [src/lib/parallel_scan/parallel_scan.ml:35:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L35)
///
/// **Gid**: 935
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum ParallelScanJobStatusStableV1 {
    Todo,
    Done,
}

/// Location: [src/lib/parallel_scan/parallel_scan.ml:112:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L112)
///
/// Gid: 940
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateStableV2PolyTreesArg0Arg01Full0<Merge> {
    pub left: Merge,
    pub right: Merge,
    pub seq_no: ParallelScanSequenceNumberStableV1,
    pub status: ParallelScanJobStatusStableV1,
}

/// Location: [src/lib/parallel_scan/parallel_scan.ml:130:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L130)
///
/// Gid: 941
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum TransactionSnarkScanStateStableV2PolyTreesArg0Arg01<Merge> {
    Empty,
    Part(Merge),
    Full(TransactionSnarkScanStateStableV2PolyTreesArg0Arg01Full0<Merge>),
}

/// Location: [src/lib/parallel_scan/parallel_scan.ml:159:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L159)
///
/// Gid: 942
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateStableV2PolyTreesArg0Arg0<Merge>(
    pub (ParallelScanWeightStableV1, ParallelScanWeightStableV1),
    pub TransactionSnarkScanStateStableV2PolyTreesArg0Arg01<Merge>,
);

/// Location: [src/lib/parallel_scan/parallel_scan.ml:68:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L68)
///
/// Gid: 937
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateStableV2PolyTreesArg0Arg11Full0<Base> {
    pub job: Base,
    pub seq_no: ParallelScanSequenceNumberStableV1,
    pub status: ParallelScanJobStatusStableV1,
}

/// Location: [src/lib/parallel_scan/parallel_scan.ml:84:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L84)
///
/// Gid: 938
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum TransactionSnarkScanStateStableV2PolyTreesArg0Arg11<Base> {
    Empty,
    Full(TransactionSnarkScanStateStableV2PolyTreesArg0Arg11Full0<Base>),
}

/// Location: [src/lib/parallel_scan/parallel_scan.ml:98:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L98)
///
/// Gid: 939
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateStableV2PolyTreesArg0Arg1<Base>(
    pub ParallelScanWeightStableV1,
    pub TransactionSnarkScanStateStableV2PolyTreesArg0Arg11<Base>,
);

/// Location: [src/lib/parallel_scan/parallel_scan.ml:803:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L803)
///
/// Gid: 948
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateStableV2Poly<Merge, Base> {
    pub trees: TransactionSnarkScanStateStableV2PolyTrees<
        TransactionSnarkScanStateStableV2PolyTreesArg0<
            TransactionSnarkScanStateStableV2PolyTreesArg0Arg0<Merge>,
            TransactionSnarkScanStateStableV2PolyTreesArg0Arg1<Base>,
        >,
    >,
    pub acc: Option<(Merge, Vec<Base>)>,
    pub curr_job_seq_no: crate::number::Int32,
    pub max_base_jobs: crate::number::Int32,
    pub delay: crate::number::Int32,
}

/// **Origin**: `Mina_base__Sok_message.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/sok_message.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/sok_message.ml#L8)
///
/// **Gid**: 775
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSokMessageStableV1 {
    pub fee: CurrencyMakeStrFeeStableV1,
    pub prover: NonZeroCurvePointUncompressedStableV1,
}

/// **Origin**: `Transaction_snark_scan_state.Ledger_proof_with_sok_message.Stable.V2.t`
///
/// **Location**: [src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml:61:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml#L61)
///
/// **Gid**: 950
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateLedgerProofWithSokMessageStableV2(
    pub LedgerProofProdStableV2,
    pub MinaBaseSokMessageStableV1,
);

/// **Origin**: `Mina_transaction_logic.Transaction_applied.Signed_command_applied.Common.Stable.V2.t`
///
/// **Location**: [src/lib/transaction_logic/mina_transaction_logic.ml:17:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_logic/mina_transaction_logic.ml#L17)
///
/// **Gid**: 794
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV2 {
    pub user_command: StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2PolyArg1<
        MinaBaseSignedCommandMakeStrStableV2,
    >,
}

/// **Origin**: `Mina_transaction_logic.Transaction_applied.Signed_command_applied.Body.Stable.V2.t`
///
/// **Location**: [src/lib/transaction_logic/mina_transaction_logic.ml:31:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_logic/mina_transaction_logic.ml#L31)
///
/// **Gid**: 795
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaTransactionLogicTransactionAppliedSignedCommandAppliedBodyStableV2 {
    Payment {
        new_accounts: Vec<MinaBaseAccountIdMakeStrStableV2>,
    },
    StakeDelegation {
        previous_delegate: Option<NonZeroCurvePointUncompressedStableV1>,
    },
    Failed,
}

/// **Origin**: `Mina_transaction_logic.Transaction_applied.Signed_command_applied.Stable.V2.t`
///
/// **Location**: [src/lib/transaction_logic/mina_transaction_logic.ml:46:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_logic/mina_transaction_logic.ml#L46)
///
/// **Gid**: 796
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedSignedCommandAppliedStableV2 {
    pub common: MinaTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV2,
    pub body: MinaTransactionLogicTransactionAppliedSignedCommandAppliedBodyStableV2,
}

/// **Origin**: `Mina_transaction_logic.Transaction_applied.Parties_applied.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_logic/mina_transaction_logic.ml:58:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_logic/mina_transaction_logic.ml#L58)
///
/// **Gid**: 797
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedPartiesAppliedStableV1 {
    pub accounts: Vec<(
        MinaBaseAccountIdMakeStrStableV2,
        Option<MinaBaseAccountBinableArgStableV2>,
    )>,
    pub command:
        StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2PolyArg1<MinaBasePartiesTStableV1>,
    pub new_accounts: Vec<MinaBaseAccountIdMakeStrStableV2>,
}

/// **Origin**: `Mina_transaction_logic.Transaction_applied.Command_applied.Stable.V2.t`
///
/// **Location**: [src/lib/transaction_logic/mina_transaction_logic.ml:75:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_logic/mina_transaction_logic.ml#L75)
///
/// **Gid**: 798
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaTransactionLogicTransactionAppliedCommandAppliedStableV2 {
    SignedCommand(MinaTransactionLogicTransactionAppliedSignedCommandAppliedStableV2),
    Parties(MinaTransactionLogicTransactionAppliedPartiesAppliedStableV1),
}

/// **Origin**: `Mina_base__Fee_transfer.Make_str.Single.Stable.V2.t`
///
/// **Location**: [src/lib/mina_base/fee_transfer.ml:19:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_transfer.ml#L19)
///
/// **Gid**: 735
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeTransferMakeStrSingleStableV2 {
    pub receiver_pk: NonZeroCurvePointUncompressedStableV1,
    pub fee: CurrencyMakeStrFeeStableV1,
    pub fee_token: MinaBaseAccountIdMakeStrDigestStableV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Fee_transfer.Make_str.Stable.V2.t`
///
/// **Location**: [src/lib/mina_base/fee_transfer.ml:68:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_transfer.ml#L68)
///
/// **Gid**: 736
pub struct MinaBaseFeeTransferMakeStrStableV2(
    pub TransactionSnarkWorkTStableV2Proofs<MinaBaseFeeTransferMakeStrSingleStableV2>,
);

/// **Origin**: `Mina_transaction_logic.Transaction_applied.Fee_transfer_applied.Stable.V2.t`
///
/// **Location**: [src/lib/transaction_logic/mina_transaction_logic.ml:89:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_logic/mina_transaction_logic.ml#L89)
///
/// **Gid**: 799
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedFeeTransferAppliedStableV2 {
    pub fee_transfer: StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2PolyArg1<
        MinaBaseFeeTransferMakeStrStableV2,
    >,
    pub new_accounts: Vec<MinaBaseAccountIdMakeStrStableV2>,
    pub burned_tokens: CurrencyMakeStrAmountMakeStrStableV1,
}

/// **Origin**: `Mina_base__Coinbase.Make_str.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/coinbase.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/coinbase.ml#L17)
///
/// **Gid**: 738
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseCoinbaseMakeStrStableV1 {
    pub receiver: NonZeroCurvePointUncompressedStableV1,
    pub amount: CurrencyMakeStrAmountMakeStrStableV1,
    pub fee_transfer: Option<MinaBaseCoinbaseFeeTransferMakeStrStableV1>,
}

/// **Origin**: `Mina_transaction_logic.Transaction_applied.Coinbase_applied.Stable.V2.t`
///
/// **Location**: [src/lib/transaction_logic/mina_transaction_logic.ml:105:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_logic/mina_transaction_logic.ml#L105)
///
/// **Gid**: 800
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedCoinbaseAppliedStableV2 {
    pub coinbase: StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2PolyArg1<
        MinaBaseCoinbaseMakeStrStableV1,
    >,
    pub new_accounts: Vec<MinaBaseAccountIdMakeStrStableV2>,
    pub burned_tokens: CurrencyMakeStrAmountMakeStrStableV1,
}

/// **Origin**: `Mina_transaction_logic.Transaction_applied.Varying.Stable.V2.t`
///
/// **Location**: [src/lib/transaction_logic/mina_transaction_logic.ml:121:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_logic/mina_transaction_logic.ml#L121)
///
/// **Gid**: 801
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaTransactionLogicTransactionAppliedVaryingStableV2 {
    Command(MinaTransactionLogicTransactionAppliedCommandAppliedStableV2),
    FeeTransfer(MinaTransactionLogicTransactionAppliedFeeTransferAppliedStableV2),
    Coinbase(MinaTransactionLogicTransactionAppliedCoinbaseAppliedStableV2),
}

/// **Origin**: `Mina_transaction_logic.Transaction_applied.Stable.V2.t`
///
/// **Location**: [src/lib/transaction_logic/mina_transaction_logic.ml:135:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_logic/mina_transaction_logic.ml#L135)
///
/// **Gid**: 802
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedStableV2 {
    pub previous_hash: MinaBaseLedgerHash0StableV1,
    pub varying: MinaTransactionLogicTransactionAppliedVaryingStableV2,
}

/// **Origin**: `Transaction_snark.Pending_coinbase_stack_state.Init_stack.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark/transaction_snark.ml:56:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L56)
///
/// **Gid**: 899
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum TransactionSnarkPendingCoinbaseStackStateInitStackStableV1 {
    Base(MinaBasePendingCoinbaseStackVersionedStableV1),
    Merge,
}

/// **Origin**: `Transaction_snark_scan_state.Transaction_with_witness.Stable.V2.t`
///
/// **Location**: [src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml:40:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml#L40)
///
/// **Gid**: 949
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateTransactionWithWitnessStableV2 {
    pub transaction_with_info: MinaTransactionLogicTransactionAppliedStableV2,
    pub state_hash: (DataHashLibStateHashStableV1, MinaBaseStateBodyHashStableV1),
    pub statement: TransactionSnarkStatementStableV2,
    pub init_stack: TransactionSnarkPendingCoinbaseStackStateInitStackStableV1,
    pub ledger_witness: MinaBaseSparseLedgerBaseStableV2,
}

/// Location: [src/lib/mina_base/pending_coinbase.ml:1226:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L1226)
///
/// Gid: 765
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStableV2Poly<Tree, StackId> {
    pub tree: Tree,
    pub pos_list: Vec<StackId>,
    pub new_pos: StackId,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Pending_coinbase.Stack_id.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/pending_coinbase.ml:101:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L101)
///
/// **Gid**: 740
pub struct MinaBasePendingCoinbaseStackIdStableV1(pub crate::number::Int32);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// **Origin**: `Mina_base__Pending_coinbase.Merkle_tree_versioned.Stable.V2.t`
///
/// **Location**: [src/lib/mina_base/pending_coinbase.ml:529:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L529)
///
/// **Gid**: 764
pub struct MinaBasePendingCoinbaseMerkleTreeVersionedStableV2(
    pub  MinaBaseSparseLedgerBaseStableV2Poly<
        MinaBasePendingCoinbaseHashVersionedStableV1,
        MinaBasePendingCoinbaseStackIdStableV1,
        MinaBasePendingCoinbaseStackVersionedStableV1,
    >,
);

/// Location: [src/lib/syncable_ledger/syncable_ledger.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/syncable_ledger/syncable_ledger.ml#L17)
///
/// Gid: 817
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaLedgerSyncLedgerQueryStableV1Poly<Addr> {
    WhatChildHashes(Addr),
    WhatContents(Addr),
    NumAccounts,
}

/// **Origin**: `Merkle_address.Binable_arg.Stable.V1.t`
///
/// **Location**: [src/lib/merkle_address/merkle_address.ml:48:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/merkle_address/merkle_address.ml#L48)
///
/// **Gid**: 803
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MerkleAddressBinableArgStableV1(pub crate::number::Int32, pub crate::string::ByteString);

/// Location: [src/lib/syncable_ledger/syncable_ledger.ml:35:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/syncable_ledger/syncable_ledger.ml#L35)
///
/// Gid: 818
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaLedgerSyncLedgerAnswerStableV2Poly<Hash, Account> {
    ChildHashesAre(Hash, Hash),
    ContentsAre(Vec<Account>),
    NumAccounts(crate::number::Int32, Hash),
}

/// **Origin**: `Trust_system__Banned_status.Stable.V1.t`
///
/// **Location**: [src/lib/trust_system/banned_status.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/trust_system/banned_status.ml#L6)
///
/// **Gid**: 814
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum TrustSystemBannedStatusStableV1 {
    Unbanned,
    BannedUntil(crate::number::Float64),
}
