use binprot::{BinProtRead, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};

use super::manual::*;

/// **Origin**: `Mina_block__External_transition.Raw_versioned__.Stable.V1.t`
///
/// **Location**: [src/lib/mina_block/external_transition.ml:31:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_block/external_transition.ml#L31)
///
/// **Gid**: 1647
pub type MinaBlockExternalTransitionRawVersionedStableV1Versioned =
    crate::versioned::Versioned<MinaBlockExternalTransitionRawVersionedStableV1VersionedV1, 1i32>;

/// **Origin**: `Network_pool__Transaction_pool.Diff_versioned.Stable.V1.t`
///
/// **Location**: [src/lib/network_pool/transaction_pool.ml:45:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_pool/transaction_pool.ml#L45)
///
/// **Gid**: 1729
pub type NetworkPoolTransactionPoolDiffVersionedStableV1Versioned =
    crate::versioned::Versioned<NetworkPoolTransactionPoolDiffVersionedStableV1VersionedV1, 1i32>;

/// **Origin**: `Network_pool__Snark_pool.Diff_versioned.Stable.V1.t`
///
/// **Location**: [src/lib/network_pool/snark_pool.ml:705:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_pool/snark_pool.ml#L705)
///
/// **Gid**: 1745
pub type NetworkPoolSnarkPoolDiffVersionedStableV1Versioned =
    crate::versioned::Versioned<NetworkPoolSnarkPoolDiffVersionedStableV1VersionedV1, 1i32>;

/// **Origin**: `Mina_base__Account.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/account.ml:188:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account.ml#L188)
///
/// **Gid**: 1005
pub type MinaBaseAccountStableV1Versioned =
    crate::versioned::Versioned<MinaBaseAccountStableV1VersionedV1, 1i32>;

/// **Origin**: `Network_peer__Peer.Stable.V1.t`
///
/// **Location**: [src/lib/network_peer/peer.ml:28:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_peer/peer.ml#L28)
///
/// **Gid**: 783
pub type NetworkPeerPeerStableV1Versioned =
    crate::versioned::Versioned<NetworkPeerPeerStableV1VersionedV1, 1i32>;

/// **Origin**: `Transaction_snark_scan_state.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml:151:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml#L151)
///
/// **Gid**: 1580
pub type TransactionSnarkScanStateStableV1Versioned =
    crate::versioned::Versioned<TransactionSnarkScanStateStableV1VersionedV1, 1i32>;

/// **Origin**: `Mina_base__Pending_coinbase.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/pending_coinbase.ml:1237:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L1237)
///
/// **Gid**: 1290
pub type MinaBasePendingCoinbaseStableV1Versioned =
    crate::versioned::Versioned<MinaBasePendingCoinbaseStableV1VersionedV1, 1i32>;

/// **Origin**: `Mina_base__Sync_ledger.Query.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/sync_ledger.ml:70:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/sync_ledger.ml#L70)
///
/// **Gid**: 1230
pub type MinaBaseSyncLedgerQueryStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSyncLedgerQueryStableV1VersionedV1, 1i32>;

/// **Origin**: `Mina_base__Sync_ledger.Answer.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/sync_ledger.ml:55:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/sync_ledger.ml#L55)
///
/// **Gid**: 1227
pub type MinaBaseSyncLedgerAnswerStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSyncLedgerAnswerStableV1VersionedV1, 1i32>;

/// **Origin**: `Sync_status.T.Stable.V1.t`
///
/// **Location**: [src/lib/sync_status/sync_status.ml:54:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/sync_status/sync_status.ml#L54)
///
/// **Gid**: 1783
pub type SyncStatusTStableV1Versioned =
    crate::versioned::Versioned<SyncStatusTStableV1VersionedV1, 1i32>;

/// **Origin**: `Trust_system__Peer_status.Stable.V1.t`
///
/// **Location**: [src/lib/trust_system/peer_status.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/trust_system/peer_status.ml#L6)
///
/// **Gid**: 804
pub type TrustSystemPeerStatusStableV1Versioned =
    crate::versioned::Versioned<TrustSystemPeerStatusStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_state/protocol_state.ml:16:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L16)
///
/// Gid: 1420
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateProtocolStateValueStableV1VersionedV1PolyV1<StateHash, Body> {
    pub previous_state_hash: StateHash,
    pub body: Body,
}

/// Location: [src/lib/mina_state/protocol_state.ml:16:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L16)
///
/// Gid: 1422
pub type MinaStateProtocolStateValueStableV1VersionedV1Poly<StateHash, Body> =
    crate::versioned::Versioned<
        MinaStateProtocolStateValueStableV1VersionedV1PolyV1<StateHash, Body>,
        1i32,
    >;

/// Location: [src/lib/data_hash_lib/state_hash.ml:43:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/data_hash_lib/state_hash.ml#L43)
///
/// Gid: 715
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, BinProtRead, BinProtWrite,
)]
pub struct MinaStateProtocolStateValueStableV1VersionedV1PolyArg0V1Poly(pub crate::bigint::BigInt);

#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, BinProtRead, BinProtWrite,
)]
/// Location: [src/lib/data_hash_lib/state_hash.ml:42:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/data_hash_lib/state_hash.ml#L42)
///
/// Gid: 718
pub struct MinaStateProtocolStateValueStableV1VersionedV1PolyArg0V1(
    pub MinaStateProtocolStateValueStableV1VersionedV1PolyArg0V1Poly,
);

/// Location: [src/lib/data_hash_lib/state_hash.ml:42:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/data_hash_lib/state_hash.ml#L42)
///
/// Gid: 719
pub type MinaStateProtocolStateValueStableV1VersionedV1PolyArg0 =
    crate::versioned::Versioned<MinaStateProtocolStateValueStableV1VersionedV1PolyArg0V1, 1i32>;

/// Location: [src/lib/mina_state/protocol_state.ml:38:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L38)
///
/// Gid: 1423
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateProtocolStateBodyValueStableV1VersionedV1PolyV1<
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

/// Location: [src/lib/mina_state/protocol_state.ml:38:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L38)
///
/// Gid: 1425
pub type MinaStateProtocolStateBodyValueStableV1VersionedV1Poly<
    StateHash,
    BlockchainState,
    ConsensusState,
    Constants,
> = crate::versioned::Versioned<
    MinaStateProtocolStateBodyValueStableV1VersionedV1PolyV1<
        StateHash,
        BlockchainState,
        ConsensusState,
        Constants,
    >,
    1i32,
>;

/// Location: [src/lib/mina_state/blockchain_state.ml:9:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/blockchain_state.ml#L9)
///
/// Gid: 1408
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateBlockchainStateValueStableV1VersionedV1PolyV1<
    StagedLedgerHash,
    SnarkedLedgerHash,
    TokenId,
    Time,
> {
    pub staged_ledger_hash: StagedLedgerHash,
    pub snarked_ledger_hash: SnarkedLedgerHash,
    pub genesis_ledger_hash: SnarkedLedgerHash,
    pub snarked_next_available_token: TokenId,
    pub timestamp: Time,
}

/// Location: [src/lib/mina_state/blockchain_state.ml:9:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/blockchain_state.ml#L9)
///
/// Gid: 1410
pub type MinaStateBlockchainStateValueStableV1VersionedV1Poly<
    StagedLedgerHash,
    SnarkedLedgerHash,
    TokenId,
    Time,
> = crate::versioned::Versioned<
    MinaStateBlockchainStateValueStableV1VersionedV1PolyV1<
        StagedLedgerHash,
        SnarkedLedgerHash,
        TokenId,
        Time,
    >,
    1i32,
>;

/// Location: [src/lib/mina_base/staged_ledger_hash.ml:174:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L174)
///
/// Gid: 1300
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashStableV1VersionedV1PolyV1<NonSnark, PendingCoinbaseHash> {
    pub non_snark: NonSnark,
    pub pending_coinbase_hash: PendingCoinbaseHash,
}

/// Location: [src/lib/mina_base/staged_ledger_hash.ml:174:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L174)
///
/// Gid: 1302
pub type MinaBaseStagedLedgerHashStableV1VersionedV1Poly<NonSnark, PendingCoinbaseHash> =
    crate::versioned::Versioned<
        MinaBaseStagedLedgerHashStableV1VersionedV1PolyV1<NonSnark, PendingCoinbaseHash>,
        1i32,
    >;

/// Location: [src/lib/mina_base/ledger_hash0.ml:18:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/ledger_hash0.ml#L18)
///
/// Gid: 893
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashNonSnarkStableV1VersionedV1LedgerHashV1Poly(
    pub crate::bigint::BigInt,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/ledger_hash0.ml:17:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/ledger_hash0.ml#L17)
///
/// Gid: 896
pub struct MinaBaseStagedLedgerHashNonSnarkStableV1VersionedV1LedgerHashV1(
    pub MinaBaseStagedLedgerHashNonSnarkStableV1VersionedV1LedgerHashV1Poly,
);

/// Location: [src/lib/mina_base/ledger_hash0.ml:17:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/ledger_hash0.ml#L17)
///
/// Gid: 897
pub type MinaBaseStagedLedgerHashNonSnarkStableV1VersionedV1LedgerHash =
    crate::versioned::Versioned<
        MinaBaseStagedLedgerHashNonSnarkStableV1VersionedV1LedgerHashV1,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/staged_ledger_hash.ml:15:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L15)
///
/// Gid: 1291
pub struct MinaBaseStagedLedgerHashAuxHashStableV1VersionedV1(pub crate::string::ByteString);

/// **Origin**: `Mina_base__Staged_ledger_hash.Aux_hash.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/staged_ledger_hash.ml:15:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L15)
///
/// **Gid**: 1293
pub type MinaBaseStagedLedgerHashAuxHashStableV1Versioned =
    crate::versioned::Versioned<MinaBaseStagedLedgerHashAuxHashStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/staged_ledger_hash.ml:59:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L59)
///
/// Gid: 1294
pub struct MinaBaseStagedLedgerHashPendingCoinbaseAuxStableV1VersionedV1(
    pub crate::string::ByteString,
);

/// **Origin**: `Mina_base__Staged_ledger_hash.Pending_coinbase_aux.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/staged_ledger_hash.ml:59:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L59)
///
/// **Gid**: 1296
pub type MinaBaseStagedLedgerHashPendingCoinbaseAuxStableV1Versioned = crate::versioned::Versioned<
    MinaBaseStagedLedgerHashPendingCoinbaseAuxStableV1VersionedV1,
    1i32,
>;

/// Location: [src/lib/mina_base/staged_ledger_hash.ml:95:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L95)
///
/// Gid: 1297
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashNonSnarkStableV1VersionedV1 {
    pub ledger_hash: MinaBaseStagedLedgerHashNonSnarkStableV1VersionedV1LedgerHash,
    pub aux_hash: MinaBaseStagedLedgerHashAuxHashStableV1Versioned,
    pub pending_coinbase_aux: MinaBaseStagedLedgerHashPendingCoinbaseAuxStableV1Versioned,
}

/// **Origin**: `Mina_base__Staged_ledger_hash.Non_snark.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/staged_ledger_hash.ml:95:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L95)
///
/// **Gid**: 1299
pub type MinaBaseStagedLedgerHashNonSnarkStableV1Versioned =
    crate::versioned::Versioned<MinaBaseStagedLedgerHashNonSnarkStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/pending_coinbase.ml:359:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L359)
///
/// Gid: 1259
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseHashVersionedStableV1VersionedV1PolyV1Poly(
    pub crate::bigint::BigInt,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/pending_coinbase.ml:358:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L358)
///
/// Gid: 1262
pub struct MinaBasePendingCoinbaseHashVersionedStableV1VersionedV1PolyV1(
    pub MinaBasePendingCoinbaseHashVersionedStableV1VersionedV1PolyV1Poly,
);

/// Location: [src/lib/mina_base/pending_coinbase.ml:358:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L358)
///
/// Gid: 1263
pub type MinaBasePendingCoinbaseHashVersionedStableV1VersionedV1Poly = crate::versioned::Versioned<
    MinaBasePendingCoinbaseHashVersionedStableV1VersionedV1PolyV1,
    1i32,
>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/pending_coinbase.ml:517:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L517)
///
/// Gid: 1279
pub struct MinaBasePendingCoinbaseHashVersionedStableV1VersionedV1(
    pub MinaBasePendingCoinbaseHashVersionedStableV1VersionedV1Poly,
);

/// **Origin**: `Mina_base__Pending_coinbase.Hash_versioned.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/pending_coinbase.ml:517:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L517)
///
/// **Gid**: 1281
pub type MinaBasePendingCoinbaseHashVersionedStableV1Versioned =
    crate::versioned::Versioned<MinaBasePendingCoinbaseHashVersionedStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/staged_ledger_hash.ml:191:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L191)
///
/// Gid: 1303
pub struct MinaBaseStagedLedgerHashStableV1VersionedV1(
    pub  MinaBaseStagedLedgerHashStableV1VersionedV1Poly<
        MinaBaseStagedLedgerHashNonSnarkStableV1Versioned,
        MinaBasePendingCoinbaseHashVersionedStableV1Versioned,
    >,
);

/// **Origin**: `Mina_base__Staged_ledger_hash.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/staged_ledger_hash.ml:191:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L191)
///
/// **Gid**: 1305
pub type MinaBaseStagedLedgerHashStableV1Versioned =
    crate::versioned::Versioned<MinaBaseStagedLedgerHashStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/unsigned_extended/unsigned_extended.ml:76:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/unsigned_extended/unsigned_extended.ml#L76)
///
/// Gid: 669
pub struct UnsignedExtendedUInt64StableV1VersionedV1(pub crate::number::Int64);

/// **Origin**: `Unsigned_extended.UInt64.Stable.V1.t`
///
/// **Location**: [src/lib/unsigned_extended/unsigned_extended.ml:76:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/unsigned_extended/unsigned_extended.ml#L76)
///
/// **Gid**: 670
pub type UnsignedExtendedUInt64StableV1Versioned =
    crate::versioned::Versioned<UnsignedExtendedUInt64StableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_numbers/nat.ml:220:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L220)
///
/// Gid: 814
pub struct MinaNumbersNatMake64StableV1VersionedV1(pub UnsignedExtendedUInt64StableV1Versioned);

/// **Origin**: `Mina_numbers__Nat.Make64.Stable.V1.t`
///
/// **Location**: [src/lib/mina_numbers/nat.ml:220:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L220)
///
/// **Gid**: 816
pub type MinaNumbersNatMake64StableV1Versioned =
    crate::versioned::Versioned<MinaNumbersNatMake64StableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/token_id.ml:49:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/token_id.ml#L49)
///
/// Gid: 817
pub struct MinaBaseTokenIdStableV1VersionedV1(pub MinaNumbersNatMake64StableV1Versioned);

/// **Origin**: `Mina_base__Token_id.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/token_id.ml:49:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/token_id.ml#L49)
///
/// **Gid**: 818
pub type MinaBaseTokenIdStableV1Versioned =
    crate::versioned::Versioned<MinaBaseTokenIdStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/block_time/block_time.ml:14:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/block_time/block_time.ml#L14)
///
/// Gid: 757
pub struct BlockTimeTimeStableV1VersionedV1(pub UnsignedExtendedUInt64StableV1Versioned);

/// **Origin**: `Block_time.Time.Stable.V1.t`
///
/// **Location**: [src/lib/block_time/block_time.ml:14:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/block_time/block_time.ml#L14)
///
/// **Gid**: 759
pub type BlockTimeTimeStableV1Versioned =
    crate::versioned::Versioned<BlockTimeTimeStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_state/blockchain_state.ml:35:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/blockchain_state.ml#L35)
///
/// Gid: 1411
pub struct MinaStateBlockchainStateValueStableV1VersionedV1(
    pub  MinaStateBlockchainStateValueStableV1VersionedV1Poly<
        MinaBaseStagedLedgerHashStableV1Versioned,
        MinaBaseStagedLedgerHashNonSnarkStableV1VersionedV1LedgerHash,
        MinaBaseTokenIdStableV1Versioned,
        BlockTimeTimeStableV1Versioned,
    >,
);

/// **Origin**: `Mina_state__Blockchain_state.Value.Stable.V1.t`
///
/// **Location**: [src/lib/mina_state/blockchain_state.ml:35:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/blockchain_state.ml#L35)
///
/// **Gid**: 1413
pub type MinaStateBlockchainStateValueStableV1Versioned =
    crate::versioned::Versioned<MinaStateBlockchainStateValueStableV1VersionedV1, 1i32>;

/// Location: [src/lib/consensus/proof_of_stake.ml:1681:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1681)
///
/// Gid: 1400
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyV1<
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

/// Location: [src/lib/consensus/proof_of_stake.ml:1681:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1681)
///
/// Gid: 1402
pub type ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1Poly<
    Length,
    VrfOutput,
    Amount,
    GlobalSlot,
    GlobalSlotSinceGenesis,
    StakingEpochData,
    NextEpochData,
    Bool,
    Pk,
> = crate::versioned::Versioned<
    ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyV1<
        Length,
        VrfOutput,
        Amount,
        GlobalSlot,
        GlobalSlotSinceGenesis,
        StakingEpochData,
        NextEpochData,
        Bool,
        Pk,
    >,
    1i32,
>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/unsigned_extended/unsigned_extended.ml:126:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/unsigned_extended/unsigned_extended.ml#L126)
///
/// Gid: 671
pub struct UnsignedExtendedUInt32StableV1VersionedV1(pub crate::number::Int32);

/// **Origin**: `Unsigned_extended.UInt32.Stable.V1.t`
///
/// **Location**: [src/lib/unsigned_extended/unsigned_extended.ml:126:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/unsigned_extended/unsigned_extended.ml#L126)
///
/// **Gid**: 672
pub type UnsignedExtendedUInt32StableV1Versioned =
    crate::versioned::Versioned<UnsignedExtendedUInt32StableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_numbers/nat.ml:186:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L186)
///
/// Gid: 686
pub struct ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg0V1(
    pub UnsignedExtendedUInt32StableV1Versioned,
);

/// Location: [src/lib/mina_numbers/nat.ml:186:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L186)
///
/// Gid: 688
pub type ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg0 =
    crate::versioned::Versioned<
        ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg0V1,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/consensus/vrf/consensus_vrf.ml:170:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/vrf/consensus_vrf.ml#L170)
///
/// Gid: 1338
pub struct ConsensusVrfOutputTruncatedStableV1VersionedV1(pub crate::string::ByteString);

/// **Origin**: `Consensus_vrf.Output.Truncated.Stable.V1.t`
///
/// **Location**: [src/lib/consensus/vrf/consensus_vrf.ml:170:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/vrf/consensus_vrf.ml#L170)
///
/// **Gid**: 1340
pub type ConsensusVrfOutputTruncatedStableV1Versioned =
    crate::versioned::Versioned<ConsensusVrfOutputTruncatedStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/currency/currency.ml:706:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/currency.ml#L706)
///
/// Gid: 700
pub struct CurrencyAmountMakeStrStableV1VersionedV1(pub UnsignedExtendedUInt64StableV1Versioned);

/// **Origin**: `Currency.Amount.Make_str.Stable.V1.t`
///
/// **Location**: [src/lib/currency/currency.ml:706:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/currency.ml#L706)
///
/// **Gid**: 702
pub type CurrencyAmountMakeStrStableV1Versioned =
    crate::versioned::Versioned<CurrencyAmountMakeStrStableV1VersionedV1, 1i32>;

/// Location: [src/lib/consensus/global_slot.ml:11:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/global_slot.ml#L11)
///
/// Gid: 1371
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusGlobalSlotStableV1VersionedV1PolyV1<SlotNumber, SlotsPerEpoch> {
    pub slot_number: SlotNumber,
    pub slots_per_epoch: SlotsPerEpoch,
}

/// Location: [src/lib/consensus/global_slot.ml:11:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/global_slot.ml#L11)
///
/// Gid: 1373
pub type ConsensusGlobalSlotStableV1VersionedV1Poly<SlotNumber, SlotsPerEpoch> =
    crate::versioned::Versioned<
        ConsensusGlobalSlotStableV1VersionedV1PolyV1<SlotNumber, SlotsPerEpoch>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_numbers/nat.ml:186:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L186)
///
/// Gid: 676
pub struct ConsensusGlobalSlotStableV1VersionedV1PolyArg0V1(
    pub UnsignedExtendedUInt32StableV1Versioned,
);

/// Location: [src/lib/mina_numbers/nat.ml:186:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L186)
///
/// Gid: 678
pub type ConsensusGlobalSlotStableV1VersionedV1PolyArg0 =
    crate::versioned::Versioned<ConsensusGlobalSlotStableV1VersionedV1PolyArg0V1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/consensus/global_slot.ml:21:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/global_slot.ml#L21)
///
/// Gid: 1374
pub struct ConsensusGlobalSlotStableV1VersionedV1(
    pub  ConsensusGlobalSlotStableV1VersionedV1Poly<
        ConsensusGlobalSlotStableV1VersionedV1PolyArg0,
        ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg0,
    >,
);

/// **Origin**: `Consensus__Global_slot.Stable.V1.t`
///
/// **Location**: [src/lib/consensus/global_slot.ml:21:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/global_slot.ml#L21)
///
/// **Gid**: 1376
pub type ConsensusGlobalSlotStableV1Versioned =
    crate::versioned::Versioned<ConsensusGlobalSlotStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/epoch_data.ml:8:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_data.ml#L8)
///
/// Gid: 1019
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1VersionedV1PolyV1<
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

/// Location: [src/lib/mina_base/epoch_data.ml:8:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_data.ml#L8)
///
/// Gid: 1021
pub type ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1VersionedV1Poly<
    EpochLedger,
    EpochSeed,
    StartCheckpoint,
    LockCheckpoint,
    Length,
> = crate::versioned::Versioned<
    ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1VersionedV1PolyV1<
        EpochLedger,
        EpochSeed,
        StartCheckpoint,
        LockCheckpoint,
        Length,
    >,
    1i32,
>;

/// Location: [src/lib/mina_base/epoch_ledger.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_ledger.ml#L10)
///
/// Gid: 1006
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseEpochLedgerValueStableV1VersionedV1PolyV1<LedgerHash, Amount> {
    pub hash: LedgerHash,
    pub total_currency: Amount,
}

/// Location: [src/lib/mina_base/epoch_ledger.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_ledger.ml#L10)
///
/// Gid: 1008
pub type MinaBaseEpochLedgerValueStableV1VersionedV1Poly<LedgerHash, Amount> =
    crate::versioned::Versioned<
        MinaBaseEpochLedgerValueStableV1VersionedV1PolyV1<LedgerHash, Amount>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/epoch_ledger.ml:21:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_ledger.ml#L21)
///
/// Gid: 1009
pub struct MinaBaseEpochLedgerValueStableV1VersionedV1(
    pub  MinaBaseEpochLedgerValueStableV1VersionedV1Poly<
        MinaBaseStagedLedgerHashNonSnarkStableV1VersionedV1LedgerHash,
        CurrencyAmountMakeStrStableV1Versioned,
    >,
);

/// **Origin**: `Mina_base__Epoch_ledger.Value.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/epoch_ledger.ml:21:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_ledger.ml#L21)
///
/// **Gid**: 1011
pub type MinaBaseEpochLedgerValueStableV1Versioned =
    crate::versioned::Versioned<MinaBaseEpochLedgerValueStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/epoch_seed.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_seed.ml#L17)
///
/// Gid: 1014
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1VersionedV1PolyArg1V1Poly(
    pub crate::bigint::BigInt,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/epoch_seed.ml:16:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_seed.ml#L16)
///
/// Gid: 1017
pub struct ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1VersionedV1PolyArg1V1 (pub ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1VersionedV1PolyArg1V1Poly) ;

/// Location: [src/lib/mina_base/epoch_seed.ml:16:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_seed.ml#L16)
///
/// Gid: 1018
pub type ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1VersionedV1PolyArg1 =
    crate::versioned::Versioned<
        ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1VersionedV1PolyArg1V1,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/consensus/proof_of_stake.ml:1050:12](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1050)
///
/// Gid: 1394
pub struct ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1VersionedV1(
    pub  ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1VersionedV1Poly<
        MinaBaseEpochLedgerValueStableV1Versioned,
        ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1VersionedV1PolyArg1,
        MinaStateProtocolStateValueStableV1VersionedV1PolyArg0,
        MinaStateProtocolStateValueStableV1VersionedV1PolyArg0,
        ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg0,
    >,
);

/// **Origin**: `Consensus__Proof_of_stake.Data.Epoch_data.Staking_value_versioned.Value.Stable.V1.t`
///
/// **Location**: [src/lib/consensus/proof_of_stake.ml:1050:12](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1050)
///
/// **Gid**: 1396
pub type ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1Versioned =
    crate::versioned::Versioned<
        ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1VersionedV1,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/consensus/proof_of_stake.ml:1074:12](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1074)
///
/// Gid: 1397
pub struct ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1VersionedV1(
    pub  ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1VersionedV1Poly<
        MinaBaseEpochLedgerValueStableV1Versioned,
        ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1VersionedV1PolyArg1,
        MinaStateProtocolStateValueStableV1VersionedV1PolyArg0,
        MinaStateProtocolStateValueStableV1VersionedV1PolyArg0,
        ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg0,
    >,
);

/// **Origin**: `Consensus__Proof_of_stake.Data.Epoch_data.Next_value_versioned.Value.Stable.V1.t`
///
/// **Location**: [src/lib/consensus/proof_of_stake.ml:1074:12](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1074)
///
/// **Gid**: 1399
pub type ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1Versioned =
    crate::versioned::Versioned<
        ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1VersionedV1,
        1i32,
    >;

/// Location: [src/lib/non_zero_curve_point/compressed_poly.ml:11:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/compressed_poly.ml#L11)
///
/// Gid: 730
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8V1PolyPolyV1<
    Field,
    Boolean,
> {
    pub x: Field,
    pub is_odd: Boolean,
}

/// Location: [src/lib/non_zero_curve_point/compressed_poly.ml:11:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/compressed_poly.ml#L11)
///
/// Gid: 732
pub type ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8V1PolyPoly<
    Field,
    Boolean,
> = crate::versioned::Versioned<
    ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8V1PolyPolyV1<
        Field,
        Boolean,
    >,
    1i32,
>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/non_zero_curve_point/non_zero_curve_point.ml:52:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/non_zero_curve_point.ml#L52)
///
/// Gid: 736
pub struct ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8V1Poly(
    pub  ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8V1PolyPoly<
        crate::bigint::BigInt,
        bool,
    >,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/non_zero_curve_point/non_zero_curve_point.ml:51:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/non_zero_curve_point.ml#L51)
///
/// Gid: 739
pub struct ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8V1(
    pub ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8V1Poly,
);

/// Location: [src/lib/non_zero_curve_point/non_zero_curve_point.ml:51:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/non_zero_curve_point.ml#L51)
///
/// Gid: 740
pub type ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8 =
    crate::versioned::Versioned<
        ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8V1,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/consensus/proof_of_stake.ml:1716:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1716)
///
/// Gid: 1403
pub struct ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1(
    pub  ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1Poly<
        ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg0,
        ConsensusVrfOutputTruncatedStableV1Versioned,
        CurrencyAmountMakeStrStableV1Versioned,
        ConsensusGlobalSlotStableV1Versioned,
        ConsensusGlobalSlotStableV1VersionedV1PolyArg0,
        ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1Versioned,
        ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1Versioned,
        bool,
        ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8,
    >,
);

/// **Origin**: `Consensus__Proof_of_stake.Data.Consensus_state.Value.Stable.V1.t`
///
/// **Location**: [src/lib/consensus/proof_of_stake.ml:1716:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1716)
///
/// **Gid**: 1405
pub type ConsensusProofOfStakeDataConsensusStateValueStableV1Versioned =
    crate::versioned::Versioned<
        ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1,
        1i32,
    >;

/// Location: [src/lib/genesis_constants/genesis_constants.ml:239:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/genesis_constants/genesis_constants.ml#L239)
///
/// Gid: 723
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseProtocolConstantsCheckedValueStableV1VersionedV1PolyV1<
    Length,
    Delta,
    GenesisStateTimestamp,
> {
    pub k: Length,
    pub slots_per_epoch: Length,
    pub slots_per_sub_window: Length,
    pub delta: Delta,
    pub genesis_state_timestamp: GenesisStateTimestamp,
}

/// Location: [src/lib/genesis_constants/genesis_constants.ml:239:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/genesis_constants/genesis_constants.ml#L239)
///
/// Gid: 725
pub type MinaBaseProtocolConstantsCheckedValueStableV1VersionedV1Poly<
    Length,
    Delta,
    GenesisStateTimestamp,
> = crate::versioned::Versioned<
    MinaBaseProtocolConstantsCheckedValueStableV1VersionedV1PolyV1<
        Length,
        Delta,
        GenesisStateTimestamp,
    >,
    1i32,
>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/protocol_constants_checked.ml:22:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/protocol_constants_checked.ml#L22)
///
/// Gid: 1320
pub struct MinaBaseProtocolConstantsCheckedValueStableV1VersionedV1(
    pub  MinaBaseProtocolConstantsCheckedValueStableV1VersionedV1Poly<
        ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg0,
        ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg0,
        BlockTimeTimeStableV1Versioned,
    >,
);

/// **Origin**: `Mina_base__Protocol_constants_checked.Value.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/protocol_constants_checked.ml:22:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/protocol_constants_checked.ml#L22)
///
/// **Gid**: 1322
pub type MinaBaseProtocolConstantsCheckedValueStableV1Versioned =
    crate::versioned::Versioned<MinaBaseProtocolConstantsCheckedValueStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_state/protocol_state.ml:53:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L53)
///
/// Gid: 1427
pub struct MinaStateProtocolStateBodyValueStableV1VersionedV1(
    pub  MinaStateProtocolStateBodyValueStableV1VersionedV1Poly<
        MinaStateProtocolStateValueStableV1VersionedV1PolyArg0,
        MinaStateBlockchainStateValueStableV1Versioned,
        ConsensusProofOfStakeDataConsensusStateValueStableV1Versioned,
        MinaBaseProtocolConstantsCheckedValueStableV1Versioned,
    >,
);

/// **Origin**: `Mina_state__Protocol_state.Body.Value.Stable.V1.t`
///
/// **Location**: [src/lib/mina_state/protocol_state.ml:53:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L53)
///
/// **Gid**: 1429
pub type MinaStateProtocolStateBodyValueStableV1Versioned =
    crate::versioned::Versioned<MinaStateProtocolStateBodyValueStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_state/protocol_state.ml:177:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L177)
///
/// Gid: 1430
pub struct MinaStateProtocolStateValueStableV1VersionedV1(
    pub  MinaStateProtocolStateValueStableV1VersionedV1Poly<
        MinaStateProtocolStateValueStableV1VersionedV1PolyArg0,
        MinaStateProtocolStateBodyValueStableV1Versioned,
    >,
);

/// **Origin**: `Mina_state__Protocol_state.Value.Stable.V1.t`
///
/// **Location**: [src/lib/mina_state/protocol_state.ml:177:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L177)
///
/// **Gid**: 1432
pub type MinaStateProtocolStateValueStableV1Versioned =
    crate::versioned::Versioned<MinaStateProtocolStateValueStableV1VersionedV1, 1i32>;

/// Location: [src/lib/pickles/composition_types/composition_types.ml:155:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L155)
///
/// Gid: 605
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementV1PolyV1ProofStateV1DeferredValuesV1<
    Plonk,
    ScalarChallenge,
    Fp,
    Fq,
    BulletproofChallenges,
    Index,
> {
    pub plonk: Plonk,
    pub combined_inner_product: Fp,
    pub b: Fp,
    pub xi: ScalarChallenge,
    pub bulletproof_challenges: BulletproofChallenges,
    pub which_branch: Index,
    _phantom_data_0: crate::phantom::Phantom<Fq>,
}

/// Location: [src/lib/pickles/composition_types/composition_types.ml:155:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L155)
///
/// Gid: 607
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementV1PolyV1ProofStateV1DeferredValues < Plonk , ScalarChallenge , Fp , Fq , BulletproofChallenges , Index > = crate :: versioned :: Versioned < PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementV1PolyV1ProofStateV1DeferredValuesV1 < Plonk , ScalarChallenge , Fp , Fq , BulletproofChallenges , Index > , 1i32 > ;

/// Location: [src/lib/pickles/composition_types/composition_types.ml:299:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L299)
///
/// Gid: 611
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementV1PolyV1ProofStateV1 < Plonk , ScalarChallenge , Fp , Fq , MeOnly , Digest , BpChals , Index > { pub deferred_values : PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementV1PolyV1ProofStateV1DeferredValues < Plonk , ScalarChallenge , Fp , Fq , BpChals , Index > , pub sponge_digest_before_evaluations : Digest , pub me_only : MeOnly , }

/// Location: [src/lib/pickles/composition_types/composition_types.ml:299:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L299)
///
/// Gid: 613
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementV1PolyV1ProofState<
    Plonk,
    ScalarChallenge,
    Fp,
    Fq,
    MeOnly,
    Digest,
    BpChals,
    Index,
> = crate::versioned::Versioned<
    PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementV1PolyV1ProofStateV1<
        Plonk,
        ScalarChallenge,
        Fp,
        Fq,
        MeOnly,
        Digest,
        BpChals,
        Index,
    >,
    1i32,
>;

/// Location: [src/lib/pickles/composition_types/composition_types.ml:444:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L444)
///
/// Gid: 614
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementV1PolyV1<
    Plonk,
    ScalarChallenge,
    Fp,
    Fq,
    MeOnly,
    Digest,
    PassThrough,
    BpChals,
    Index,
> {
    pub proof_state: PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementV1PolyV1ProofState<
        Plonk,
        ScalarChallenge,
        Fp,
        Fq,
        MeOnly,
        Digest,
        BpChals,
        Index,
    >,
    pub pass_through: PassThrough,
}

/// Location: [src/lib/pickles/composition_types/composition_types.ml:444:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L444)
///
/// Gid: 616
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementV1Poly<
    Plonk,
    ScalarChallenge,
    Fp,
    Fq,
    MeOnly,
    Digest,
    PassThrough,
    BpChals,
    Index,
> = crate::versioned::Versioned<
    PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementV1PolyV1<
        Plonk,
        ScalarChallenge,
        Fp,
        Fq,
        MeOnly,
        Digest,
        PassThrough,
        BpChals,
        Index,
    >,
    1i32,
>;

/// Location: [src/lib/pickles/composition_types/composition_types.ml:62:14](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L62)
///
/// Gid: 602
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementV1PolyArg0V1<
    Challenge,
    ScalarChallenge,
> {
    pub alpha: ScalarChallenge,
    pub beta: Challenge,
    pub gamma: Challenge,
    pub zeta: ScalarChallenge,
}

/// Location: [src/lib/pickles/composition_types/composition_types.ml:62:14](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L62)
///
/// Gid: 604
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementV1PolyArg0<
    Challenge,
    ScalarChallenge,
> = crate::versioned::Versioned<
    PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementV1PolyArg0V1<
        Challenge,
        ScalarChallenge,
    >,
    1i32,
>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/pickles/composition_types/composition_types.ml:474:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L474)
///
/// Gid: 617
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementV1<
    Challenge,
    ScalarChallenge,
    Fp,
    Fq,
    MeOnly,
    Digest,
    PassThrough,
    BpChals,
    Index,
>(
    pub  PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementV1Poly<
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementV1PolyArg0<
            Challenge,
            ScalarChallenge,
        >,
        ScalarChallenge,
        Fp,
        Fq,
        MeOnly,
        Digest,
        PassThrough,
        BpChals,
        Index,
    >,
);

/// Location: [src/lib/pickles/composition_types/composition_types.ml:474:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L474)
///
/// Gid: 619
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyV1Statement<
    Challenge,
    ScalarChallenge,
    Fp,
    Fq,
    MeOnly,
    Digest,
    PassThrough,
    BpChals,
    Index,
> = crate::versioned::Versioned<
    PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementV1<
        Challenge,
        ScalarChallenge,
        Fp,
        Fq,
        MeOnly,
        Digest,
        PassThrough,
        BpChals,
        Index,
    >,
    1i32,
>;

/// Location: [src/lib/pickles_types/vector.ml:445:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L445)
///
/// Gid: 467
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg0V1<A>(
    pub A,
    pub (A, ()),
);

/// Location: [src/lib/pickles_types/vector.ml:445:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L445)
///
/// Gid: 468
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg0<A> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg0V1<A>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/pickles/limb_vector/constant.ml:61:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/limb_vector/constant.ml#L61)
///
/// Gid: 590
pub struct LimbVectorConstantHex64StableV1VersionedV1(pub crate::number::Int64);

/// **Origin**: `Limb_vector__Constant.Hex64.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/limb_vector/constant.ml:61:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/limb_vector/constant.ml#L61)
///
/// **Gid**: 592
pub type LimbVectorConstantHex64StableV1Versioned =
    crate::versioned::Versioned<LimbVectorConstantHex64StableV1VersionedV1, 1i32>;

/// Location: [src/lib/pickles_types/scalar_challenge.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/scalar_challenge.ml#L6)
///
/// Gid: 480
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg1V1<F> {
    ScalarChallenge(F),
}

/// Location: [src/lib/pickles_types/scalar_challenge.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/scalar_challenge.ml#L6)
///
/// Gid: 482
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg1<F> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg1V1<F>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/shifted_value.ml:31:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/shifted_value.ml#L31)
///
/// Gid: 477
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg2V1<F> {
    ShiftedValue(F),
}

/// Location: [src/lib/pickles_types/shifted_value.ml:31:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/shifted_value.ml#L31)
///
/// Gid: 479
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg2<F> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg2V1<F>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/vector.ml:474:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L474)
///
/// Gid: 469
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CompositionTypesDigestConstantStableV1VersionedV1PolyV1<A>(pub A, pub (A, (A, (A, ()))));

/// Location: [src/lib/pickles_types/vector.ml:474:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L474)
///
/// Gid: 470
pub type CompositionTypesDigestConstantStableV1VersionedV1Poly<A> =
    crate::versioned::Versioned<CompositionTypesDigestConstantStableV1VersionedV1PolyV1<A>, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/pickles/composition_types/digest.ml:13:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/digest.ml#L13)
///
/// Gid: 596
pub struct CompositionTypesDigestConstantStableV1VersionedV1(
    pub  CompositionTypesDigestConstantStableV1VersionedV1Poly<
        LimbVectorConstantHex64StableV1Versioned,
    >,
);

/// **Origin**: `Composition_types__Digest.Constant.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/composition_types/digest.ml:13:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/digest.ml#L13)
///
/// **Gid**: 598
pub type CompositionTypesDigestConstantStableV1Versioned =
    crate::versioned::Versioned<CompositionTypesDigestConstantStableV1VersionedV1, 1i32>;

/// Location: [src/lib/pickles_types/vector.ml:561:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L561)
///
/// Gid: 475
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg7V1PolyV1<A>(
    pub A,
    pub  (
        A,
        (
            A,
            (
                A,
                (
                    A,
                    (
                        A,
                        (A, (A, (A, (A, (A, (A, (A, (A, (A, (A, (A, (A, ())))))))))))),
                    ),
                ),
            ),
        ),
    ),
);

/// Location: [src/lib/pickles_types/vector.ml:561:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L561)
///
/// Gid: 476
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg7V1Poly<A> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg7V1PolyV1<A>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/zexe_backend/pasta/basic.ml:54:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/zexe_backend/pasta/basic.ml#L54)
///
/// Gid: 537
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg7V1<A>(
    pub PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg7V1Poly<A>,
);

/// Location: [src/lib/zexe_backend/pasta/basic.ml:54:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/zexe_backend/pasta/basic.ml#L54)
///
/// Gid: 539
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg7<A> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg7V1<A>,
        1i32,
    >;

/// Location: [src/lib/pickles/composition_types/bulletproof_challenge.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/bulletproof_challenge.ml#L6)
///
/// Gid: 593
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg7Arg0V1<Challenge> {
    pub prechallenge: Challenge,
}

/// Location: [src/lib/pickles/composition_types/bulletproof_challenge.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/bulletproof_challenge.ml#L6)
///
/// Gid: 595
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg7Arg0<Challenge> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg7Arg0V1<Challenge>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/pickles/composition_types/index.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/index.ml#L7)
///
/// Gid: 599
pub struct CompositionTypesIndexStableV1VersionedV1(pub crate::char::Char);

/// **Origin**: `Composition_types__Index.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/composition_types/index.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/index.ml#L7)
///
/// **Gid**: 601
pub type CompositionTypesIndexStableV1Versioned =
    crate::versioned::Versioned<CompositionTypesIndexStableV1VersionedV1, 1i32>;

/// Location: [src/lib/pickles/proof.ml:34:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L34)
///
/// Gid: 653
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyV1PrevEvalsV1<A>(pub A, pub A);

/// Location: [src/lib/pickles/proof.ml:34:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L34)
///
/// Gid: 655
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyV1PrevEvals<A> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1PrevEvalsV1<A>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:30:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L30)
///
/// Gid: 492
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyV1PrevEvalsArg0V1<A> {
    pub l: A,
    pub r: A,
    pub o: A,
    pub z: A,
    pub t: A,
    pub f: A,
    pub sigma1: A,
    pub sigma2: A,
}

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:30:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L30)
///
/// Gid: 494
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyV1PrevEvalsArg0<A> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1PrevEvalsArg0V1<A>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L17)
///
/// Gid: 489
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyV1PrevEvalsArg0Arg0V1<A>(pub Vec<A>);

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L17)
///
/// Gid: 491
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyV1PrevEvalsArg0Arg0<A> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1PrevEvalsArg0Arg0V1<A>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:203:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L203)
///
/// Gid: 504
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1MessagesV1LCommV1<G>(
    pub PicklesProofBranching2ReprStableV1VersionedV1PolyV1PrevEvalsArg0Arg0<G>,
);

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:203:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L203)
///
/// Gid: 506
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1MessagesV1LComm<G> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1MessagesV1LCommV1<G>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:149:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L149)
///
/// Gid: 501
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1MessagesV1TCommV1<GOpt> {
    pub unshifted: PicklesProofBranching2ReprStableV1VersionedV1PolyV1PrevEvalsArg0Arg0<GOpt>,
    pub shifted: GOpt,
}

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:149:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L149)
///
/// Gid: 503
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1MessagesV1TComm<GOpt> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1MessagesV1TCommV1<GOpt>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:218:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L218)
///
/// Gid: 507
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1MessagesV1<G, GOpt> {
    pub l_comm: PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1MessagesV1LComm<G>,
    pub r_comm: PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1MessagesV1LComm<G>,
    pub o_comm: PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1MessagesV1LComm<G>,
    pub z_comm: PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1MessagesV1LComm<G>,
    pub t_comm:
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1MessagesV1TComm<GOpt>,
}

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:218:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L218)
///
/// Gid: 509
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1Messages<G, GOpt> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1MessagesV1<G, GOpt>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:102:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L102)
///
/// Gid: 495
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1OpeningsV1ProofV1<G, Fq>
{
    pub lr: PicklesProofBranching2ReprStableV1VersionedV1PolyV1PrevEvalsArg0Arg0<(G, G)>,
    pub z_1: Fq,
    pub z_2: Fq,
    pub delta: G,
    pub sg: G,
}

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:102:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L102)
///
/// Gid: 497
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1OpeningsV1Proof<G, Fq> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1OpeningsV1ProofV1<G, Fq>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:124:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L124)
///
/// Gid: 498
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1OpeningsV1<G, Fq, Fqv> {
    pub proof:
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1OpeningsV1Proof<G, Fq>,
    pub evals: (
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1PrevEvalsArg0<Fqv>,
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1PrevEvalsArg0<Fqv>,
    ),
}

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:124:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L124)
///
/// Gid: 500
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1Openings<G, Fq, Fqv> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1OpeningsV1<G, Fq, Fqv>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:250:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L250)
///
/// Gid: 510
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1<G, GOpt, Fq, Fqv> {
    pub messages: PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1Messages<G, GOpt>,
    pub openings:
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1Openings<G, Fq, Fqv>,
}

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:250:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L250)
///
/// Gid: 512
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1Poly<G, GOpt, Fq, Fqv> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyV1<G, GOpt, Fq, Fqv>,
        1i32,
    >;

/// Location: [src/lib/zexe_backend/zexe_backend_common/curve.ml:97:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/zexe_backend/zexe_backend_common/curve.ml#L97)
///
/// Gid: 541
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyArg0(
    pub crate::bigint::BigInt,
    pub crate::bigint::BigInt,
);

/// Location: [src/lib/pickles_types/or_infinity.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/or_infinity.ml#L6)
///
/// Gid: 486
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyArg1V1<A> {
    Infinity,
    Finite(A),
}

/// Location: [src/lib/pickles_types/or_infinity.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/or_infinity.ml#L6)
///
/// Gid: 488
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyArg1<A> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyArg1V1<A>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/zexe_backend/zexe_backend_common/plonk_dlog_proof.ml:155:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/zexe_backend/zexe_backend_common/plonk_dlog_proof.ml#L155)
///
/// Gid: 553
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1(
    pub  PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1Poly<
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyArg0,
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyArg1<
            PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyArg0,
        >,
        crate::bigint::BigInt,
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1PrevEvalsArg0Arg0<crate::bigint::BigInt>,
    >,
);

/// Location: [src/lib/zexe_backend/zexe_backend_common/plonk_dlog_proof.ml:155:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/zexe_backend/zexe_backend_common/plonk_dlog_proof.ml#L155)
///
/// Gid: 555
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyV1Proof =
    crate::versioned::Versioned<PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1, 1i32>;

/// Location: [src/lib/pickles/proof.ml:45:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L45)
///
/// Gid: 656
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyV1<DlogMeOnly, PairingMeOnly> {
    pub statement: PicklesProofBranching2ReprStableV1VersionedV1PolyV1Statement<
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg0<
            LimbVectorConstantHex64StableV1Versioned,
        >,
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg1<
            PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg0<
                LimbVectorConstantHex64StableV1Versioned,
            >,
        >,
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg2<crate::bigint::BigInt>,
        crate::bigint::BigInt,
        DlogMeOnly,
        CompositionTypesDigestConstantStableV1Versioned,
        PairingMeOnly,
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg7<
            PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg7Arg0<
                PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg1<
                    PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg0<
                        LimbVectorConstantHex64StableV1Versioned,
                    >,
                >,
            >,
        >,
        CompositionTypesIndexStableV1Versioned,
    >,
    pub prev_evals: PicklesProofBranching2ReprStableV1VersionedV1PolyV1PrevEvals<
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1PrevEvalsArg0<
            PicklesProofBranching2ReprStableV1VersionedV1PolyV1PrevEvalsArg0Arg0<
                crate::bigint::BigInt,
            >,
        >,
    >,
    pub prev_x_hat:
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1PrevEvals<crate::bigint::BigInt>,
    pub proof: PicklesProofBranching2ReprStableV1VersionedV1PolyV1Proof,
}

/// Location: [src/lib/pickles/proof.ml:45:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L45)
///
/// Gid: 658
pub type PicklesProofBranching2ReprStableV1VersionedV1Poly<DlogMeOnly, PairingMeOnly> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1<DlogMeOnly, PairingMeOnly>,
        1i32,
    >;

/// Location: [src/lib/pickles/composition_types/composition_types.ml:275:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L275)
///
/// Gid: 608
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyArg0V1<G1, BulletproofChallenges> {
    pub sg: G1,
    pub old_bulletproof_challenges: BulletproofChallenges,
}

/// Location: [src/lib/pickles/composition_types/composition_types.ml:275:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L275)
///
/// Gid: 610
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyArg0<G1, BulletproofChallenges> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1VersionedV1PolyArg0V1<G1, BulletproofChallenges>,
        1i32,
    >;

/// Location: [src/lib/zexe_backend/zexe_backend_common/curve.ml:97:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/zexe_backend/zexe_backend_common/curve.ml#L97)
///
/// Gid: 540
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyArg0Arg0(
    pub crate::bigint::BigInt,
    pub crate::bigint::BigInt,
);

/// Location: [src/lib/pickles_types/vector.ml:532:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L532)
///
/// Gid: 473
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1VersionedV1PolyV1PolyV1<A>(
    pub A,
    pub  (
        A,
        (
            A,
            (
                A,
                (
                    A,
                    (A, (A, (A, (A, (A, (A, (A, (A, (A, (A, (A, (A, ())))))))))))),
                ),
            ),
        ),
    ),
);

/// Location: [src/lib/pickles_types/vector.ml:532:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L532)
///
/// Gid: 474
pub type PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1VersionedV1PolyV1Poly<A> =
    crate::versioned::Versioned<
        PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1VersionedV1PolyV1PolyV1<A>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/zexe_backend/pasta/basic.ml:33:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/zexe_backend/pasta/basic.ml#L33)
///
/// Gid: 534
pub struct PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1VersionedV1PolyV1<A>(
    pub PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1VersionedV1PolyV1Poly<A>,
);

/// Location: [src/lib/zexe_backend/pasta/basic.ml:33:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/zexe_backend/pasta/basic.ml#L33)
///
/// Gid: 536
pub type PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1VersionedV1Poly<A> =
    crate::versioned::Versioned<
        PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1VersionedV1PolyV1<A>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/pickles/reduced_me_only.ml:38:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/reduced_me_only.ml#L38)
///
/// Gid: 650
pub struct PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1VersionedV1(
    pub  PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1VersionedV1Poly<
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg7Arg0<
            PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg1<
                PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg0<
                    LimbVectorConstantHex64StableV1Versioned,
                >,
            >,
        >,
    >,
);

/// **Origin**: `Pickles__Reduced_me_only.Dlog_based.Challenges_vector.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/reduced_me_only.ml:38:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/reduced_me_only.ml#L38)
///
/// **Gid**: 652
pub type PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1Versioned =
    crate::versioned::Versioned<
        PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1VersionedV1,
        1i32,
    >;

/// Location: [src/lib/pickles/reduced_me_only.ml:16:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/reduced_me_only.ml#L16)
///
/// Gid: 647
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyArg1V1<S, Sgs, Bpcs> {
    pub app_state: S,
    pub sg: Sgs,
    pub old_bulletproof_challenges: Bpcs,
}

/// Location: [src/lib/pickles/reduced_me_only.ml:16:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/reduced_me_only.ml#L16)
///
/// Gid: 649
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyArg1<S, Sgs, Bpcs> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1VersionedV1PolyArg1V1<S, Sgs, Bpcs>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/pickles_types/at_most.ml:106:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/at_most.ml#L106)
///
/// Gid: 519
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyArg1Arg1V1<A>(pub Vec<A>);

/// Location: [src/lib/pickles_types/at_most.ml:106:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/at_most.ml#L106)
///
/// Gid: 520
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyArg1Arg1<A> = crate::versioned::Versioned<
    PicklesProofBranching2ReprStableV1VersionedV1PolyArg1Arg1V1<A>,
    1i32,
>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/pickles/proof.ml:283:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L283)
///
/// Gid: 659
pub struct PicklesProofBranching2ReprStableV1VersionedV1(
    pub  PicklesProofBranching2ReprStableV1VersionedV1Poly<
        PicklesProofBranching2ReprStableV1VersionedV1PolyArg0<
            PicklesProofBranching2ReprStableV1VersionedV1PolyArg0Arg0,
            PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg0<
                PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1Versioned,
            >,
        >,
        PicklesProofBranching2ReprStableV1VersionedV1PolyArg1<
            (),
            PicklesProofBranching2ReprStableV1VersionedV1PolyArg1Arg1<
                PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyArg0,
            >,
            PicklesProofBranching2ReprStableV1VersionedV1PolyArg1Arg1<
                PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg7<
                    PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg7Arg0<
                        PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg1<
                            PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg0<
                                LimbVectorConstantHex64StableV1Versioned,
                            >,
                        >,
                    >,
                >,
            >,
        >,
    >,
);

/// **Origin**: `Pickles__Proof.Branching_2.Repr.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/proof.ml:283:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L283)
///
/// **Gid**: 661
pub type PicklesProofBranching2ReprStableV1Versioned =
    crate::versioned::Versioned<PicklesProofBranching2ReprStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/pickles/proof.ml:318:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L318)
///
/// Gid: 662
pub struct PicklesProofBranching2StableV1VersionedV1(
    pub PicklesProofBranching2ReprStableV1Versioned,
);

/// **Origin**: `Pickles__Proof.Branching_2.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/proof.ml:318:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L318)
///
/// **Gid**: 663
pub type PicklesProofBranching2StableV1Versioned =
    crate::versioned::Versioned<PicklesProofBranching2StableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/proof.ml:12:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/proof.ml#L12)
///
/// Gid: 1323
pub struct MinaBaseProofStableV1VersionedV1(pub PicklesProofBranching2StableV1Versioned);

/// **Origin**: `Mina_base__Proof.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/proof.ml:12:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/proof.ml#L12)
///
/// **Gid**: 1325
pub type MinaBaseProofStableV1Versioned =
    crate::versioned::Versioned<MinaBaseProofStableV1VersionedV1, 1i32>;

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L10)
///
/// Gid: 1596
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1VersionedV1PolyV1CoinbaseV1<A> {
    Zero,
    One(Option<A>),
    Two(Option<(A, Option<A>)>),
}

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L10)
///
/// Gid: 1598
pub type StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1VersionedV1PolyV1Coinbase<A> =
    crate::versioned::Versioned<
        StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1VersionedV1PolyV1CoinbaseV1<A>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/currency/currency.ml:586:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/currency.ml#L586)
///
/// Gid: 697
pub struct CurrencyFeeStableV1VersionedV1(pub UnsignedExtendedUInt64StableV1Versioned);

/// **Origin**: `Currency.Fee.Stable.V1.t`
///
/// **Location**: [src/lib/currency/currency.ml:586:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/currency.ml#L586)
///
/// **Gid**: 699
pub type CurrencyFeeStableV1Versioned =
    crate::versioned::Versioned<CurrencyFeeStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/coinbase_fee_transfer.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/coinbase_fee_transfer.ml#L7)
///
/// Gid: 1159
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseCoinbaseFeeTransferStableV1VersionedV1 {
    pub receiver_pk: ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8,
    pub fee: CurrencyFeeStableV1Versioned,
}

/// **Origin**: `Mina_base__Coinbase_fee_transfer.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/coinbase_fee_transfer.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/coinbase_fee_transfer.ml#L7)
///
/// **Gid**: 1161
pub type MinaBaseCoinbaseFeeTransferStableV1Versioned =
    crate::versioned::Versioned<MinaBaseCoinbaseFeeTransferStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:66:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L66)
///
/// Gid: 1602
pub struct StagedLedgerDiffFtStableV1VersionedV1(pub MinaBaseCoinbaseFeeTransferStableV1Versioned);

/// **Origin**: `Staged_ledger_diff.Ft.Stable.V1.t`
///
/// **Location**: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:66:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L66)
///
/// **Gid**: 1604
pub type StagedLedgerDiffFtStableV1Versioned =
    crate::versioned::Versioned<StagedLedgerDiffFtStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/currency/currency.ml:744:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/currency.ml#L744)
///
/// Gid: 703
pub struct CurrencyBalanceStableV1VersionedV1(pub CurrencyAmountMakeStrStableV1Versioned);

/// **Origin**: `Currency.Balance.Stable.V1.t`
///
/// **Location**: [src/lib/currency/currency.ml:744:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/currency.ml#L744)
///
/// **Gid**: 705
pub type CurrencyBalanceStableV1Versioned =
    crate::versioned::Versioned<CurrencyBalanceStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/transaction_status.ml:711:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L711)
///
/// Gid: 829
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseTransactionStatusCoinbaseBalanceDataStableV1VersionedV1 {
    pub coinbase_receiver_balance: CurrencyBalanceStableV1Versioned,
    pub fee_transfer_receiver_balance: Option<CurrencyBalanceStableV1Versioned>,
}

/// **Origin**: `Mina_base__Transaction_status.Coinbase_balance_data.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_status.ml:711:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L711)
///
/// **Gid**: 831
pub type MinaBaseTransactionStatusCoinbaseBalanceDataStableV1Versioned =
    crate::versioned::Versioned<
        MinaBaseTransactionStatusCoinbaseBalanceDataStableV1VersionedV1,
        1i32,
    >;

/// Location: [src/lib/mina_base/transaction_status.ml:754:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L754)
///
/// Gid: 832
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseTransactionStatusFeeTransferBalanceDataStableV1VersionedV1 {
    pub receiver1_balance: CurrencyBalanceStableV1Versioned,
    pub receiver2_balance: Option<CurrencyBalanceStableV1Versioned>,
}

/// **Origin**: `Mina_base__Transaction_status.Fee_transfer_balance_data.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_status.ml:754:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L754)
///
/// **Gid**: 834
pub type MinaBaseTransactionStatusFeeTransferBalanceDataStableV1Versioned =
    crate::versioned::Versioned<
        MinaBaseTransactionStatusFeeTransferBalanceDataStableV1VersionedV1,
        1i32,
    >;

/// Location: [src/lib/mina_base/transaction_status.ml:795:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L795)
///
/// Gid: 835
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseTransactionStatusInternalCommandBalanceDataStableV1VersionedV1 {
    Coinbase(MinaBaseTransactionStatusCoinbaseBalanceDataStableV1Versioned),
    FeeTransfer(MinaBaseTransactionStatusFeeTransferBalanceDataStableV1Versioned),
}

/// **Origin**: `Mina_base__Transaction_status.Internal_command_balance_data.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_status.ml:795:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L795)
///
/// **Gid**: 837
pub type MinaBaseTransactionStatusInternalCommandBalanceDataStableV1Versioned =
    crate::versioned::Versioned<
        MinaBaseTransactionStatusInternalCommandBalanceDataStableV1VersionedV1,
        1i32,
    >;

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:82:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L82)
///
/// Gid: 1605
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1VersionedV1PolyV1<A, B> {
    pub completed_works: Vec<A>,
    pub commands: Vec<B>,
    pub coinbase: StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1VersionedV1PolyV1Coinbase<
        StagedLedgerDiffFtStableV1Versioned,
    >,
    pub internal_command_balances:
        Vec<MinaBaseTransactionStatusInternalCommandBalanceDataStableV1Versioned>,
}

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:82:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L82)
///
/// Gid: 1607
pub type StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1VersionedV1Poly<A, B> =
    crate::versioned::Versioned<
        StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1VersionedV1PolyV1<A, B>,
        1i32,
    >;

/// Location: [src/lib/one_or_two/one_or_two.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/one_or_two/one_or_two.ml#L7)
///
/// Gid: 769
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[polymorphic_variant]
pub enum TransactionSnarkWorkTStableV1VersionedV1ProofsV1<A> {
    One(A),
    Two((A, A)),
}

/// Location: [src/lib/one_or_two/one_or_two.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/one_or_two/one_or_two.ml#L7)
///
/// Gid: 771
pub type TransactionSnarkWorkTStableV1VersionedV1Proofs<A> =
    crate::versioned::Versioned<TransactionSnarkWorkTStableV1VersionedV1ProofsV1<A>, 1i32>;

/// Location: [src/lib/transaction_snark/transaction_snark.ml:117:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L117)
///
/// Gid: 1478
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkStatementWithSokStableV1VersionedV1PolyV1<
    LedgerHash,
    Amount,
    PendingCoinbase,
    FeeExcess,
    TokenId,
    SokDigest,
> {
    pub source: LedgerHash,
    pub target: LedgerHash,
    pub supply_increase: Amount,
    pub pending_coinbase_stack_state: PendingCoinbase,
    pub fee_excess: FeeExcess,
    pub next_available_token_before: TokenId,
    pub next_available_token_after: TokenId,
    pub sok_digest: SokDigest,
}

/// Location: [src/lib/transaction_snark/transaction_snark.ml:117:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L117)
///
/// Gid: 1480
pub type TransactionSnarkStatementWithSokStableV1VersionedV1Poly<
    LedgerHash,
    Amount,
    PendingCoinbase,
    FeeExcess,
    TokenId,
    SokDigest,
> = crate::versioned::Versioned<
    TransactionSnarkStatementWithSokStableV1VersionedV1PolyV1<
        LedgerHash,
        Amount,
        PendingCoinbase,
        FeeExcess,
        TokenId,
        SokDigest,
    >,
    1i32,
>;

/// Location: [src/lib/transaction_snark/transaction_snark.ml:63:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L63)
///
/// Gid: 1470
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkPendingCoinbaseStackStateStableV1VersionedV1PolyV1<PendingCoinbase> {
    pub source: PendingCoinbase,
    pub target: PendingCoinbase,
}

/// Location: [src/lib/transaction_snark/transaction_snark.ml:63:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L63)
///
/// Gid: 1472
pub type TransactionSnarkPendingCoinbaseStackStateStableV1VersionedV1Poly<PendingCoinbase> =
    crate::versioned::Versioned<
        TransactionSnarkPendingCoinbaseStackStateStableV1VersionedV1PolyV1<PendingCoinbase>,
        1i32,
    >;

/// Location: [src/lib/mina_base/pending_coinbase.ml:494:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L494)
///
/// Gid: 1273
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStackVersionedStableV1VersionedV1PolyV1<DataStack, StateStack> {
    pub data: DataStack,
    pub state: StateStack,
}

/// Location: [src/lib/mina_base/pending_coinbase.ml:494:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L494)
///
/// Gid: 1275
pub type MinaBasePendingCoinbaseStackVersionedStableV1VersionedV1Poly<DataStack, StateStack> =
    crate::versioned::Versioned<
        MinaBasePendingCoinbaseStackVersionedStableV1VersionedV1PolyV1<DataStack, StateStack>,
        1i32,
    >;

/// Location: [src/lib/mina_base/pending_coinbase.ml:154:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L154)
///
/// Gid: 1239
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStackVersionedStableV1VersionedV1PolyArg0V1Poly(
    pub crate::bigint::BigInt,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/pending_coinbase.ml:153:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L153)
///
/// Gid: 1242
pub struct MinaBasePendingCoinbaseStackVersionedStableV1VersionedV1PolyArg0V1(
    pub MinaBasePendingCoinbaseStackVersionedStableV1VersionedV1PolyArg0V1Poly,
);

/// Location: [src/lib/mina_base/pending_coinbase.ml:153:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L153)
///
/// Gid: 1243
pub type MinaBasePendingCoinbaseStackVersionedStableV1VersionedV1PolyArg0 =
    crate::versioned::Versioned<
        MinaBasePendingCoinbaseStackVersionedStableV1VersionedV1PolyArg0V1,
        1i32,
    >;

/// Location: [src/lib/mina_base/pending_coinbase.ml:238:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L238)
///
/// Gid: 1251
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStateStackStableV1VersionedV1PolyV1<StackHash> {
    pub init: StackHash,
    pub curr: StackHash,
}

/// Location: [src/lib/mina_base/pending_coinbase.ml:238:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L238)
///
/// Gid: 1253
pub type MinaBasePendingCoinbaseStateStackStableV1VersionedV1Poly<StackHash> =
    crate::versioned::Versioned<
        MinaBasePendingCoinbaseStateStackStableV1VersionedV1PolyV1<StackHash>,
        1i32,
    >;

/// Location: [src/lib/mina_base/pending_coinbase.ml:213:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L213)
///
/// Gid: 1246
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStateStackStableV1VersionedV1PolyArg0V1Poly(
    pub crate::bigint::BigInt,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/pending_coinbase.ml:212:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L212)
///
/// Gid: 1249
pub struct MinaBasePendingCoinbaseStateStackStableV1VersionedV1PolyArg0V1(
    pub MinaBasePendingCoinbaseStateStackStableV1VersionedV1PolyArg0V1Poly,
);

/// Location: [src/lib/mina_base/pending_coinbase.ml:212:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L212)
///
/// Gid: 1250
pub type MinaBasePendingCoinbaseStateStackStableV1VersionedV1PolyArg0 = crate::versioned::Versioned<
    MinaBasePendingCoinbaseStateStackStableV1VersionedV1PolyArg0V1,
    1i32,
>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/pending_coinbase.ml:247:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L247)
///
/// Gid: 1254
pub struct MinaBasePendingCoinbaseStateStackStableV1VersionedV1(
    pub  MinaBasePendingCoinbaseStateStackStableV1VersionedV1Poly<
        MinaBasePendingCoinbaseStateStackStableV1VersionedV1PolyArg0,
    >,
);

/// **Origin**: `Mina_base__Pending_coinbase.State_stack.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/pending_coinbase.ml:247:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L247)
///
/// **Gid**: 1256
pub type MinaBasePendingCoinbaseStateStackStableV1Versioned =
    crate::versioned::Versioned<MinaBasePendingCoinbaseStateStackStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/pending_coinbase.ml:504:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L504)
///
/// Gid: 1276
pub struct MinaBasePendingCoinbaseStackVersionedStableV1VersionedV1(
    pub  MinaBasePendingCoinbaseStackVersionedStableV1VersionedV1Poly<
        MinaBasePendingCoinbaseStackVersionedStableV1VersionedV1PolyArg0,
        MinaBasePendingCoinbaseStateStackStableV1Versioned,
    >,
);

/// **Origin**: `Mina_base__Pending_coinbase.Stack_versioned.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/pending_coinbase.ml:504:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L504)
///
/// **Gid**: 1278
pub type MinaBasePendingCoinbaseStackVersionedStableV1Versioned =
    crate::versioned::Versioned<MinaBasePendingCoinbaseStackVersionedStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/transaction_snark/transaction_snark.ml:87:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L87)
///
/// Gid: 1473
pub struct TransactionSnarkPendingCoinbaseStackStateStableV1VersionedV1(
    pub  TransactionSnarkPendingCoinbaseStackStateStableV1VersionedV1Poly<
        MinaBasePendingCoinbaseStackVersionedStableV1Versioned,
    >,
);

/// **Origin**: `Transaction_snark.Pending_coinbase_stack_state.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark/transaction_snark.ml:87:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L87)
///
/// **Gid**: 1475
pub type TransactionSnarkPendingCoinbaseStackStateStableV1Versioned =
    crate::versioned::Versioned<TransactionSnarkPendingCoinbaseStackStateStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/fee_excess.ml:54:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_excess.ml#L54)
///
/// Gid: 870
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeExcessStableV1VersionedV1PolyV1<Token, Fee> {
    pub fee_token_l: Token,
    pub fee_excess_l: Fee,
    pub fee_token_r: Token,
    pub fee_excess_r: Fee,
}

/// Location: [src/lib/mina_base/fee_excess.ml:54:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_excess.ml#L54)
///
/// Gid: 872
pub type MinaBaseFeeExcessStableV1VersionedV1Poly<Token, Fee> =
    crate::versioned::Versioned<MinaBaseFeeExcessStableV1VersionedV1PolyV1<Token, Fee>, 1i32>;

/// Location: [src/lib/currency/signed_poly.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/signed_poly.ml#L6)
///
/// Gid: 694
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeExcessStableV1VersionedV1PolyArg1V1<Magnitude, Sgn> {
    pub magnitude: Magnitude,
    pub sgn: Sgn,
}

/// Location: [src/lib/currency/signed_poly.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/signed_poly.ml#L6)
///
/// Gid: 696
pub type MinaBaseFeeExcessStableV1VersionedV1PolyArg1<Magnitude, Sgn> = crate::versioned::Versioned<
    MinaBaseFeeExcessStableV1VersionedV1PolyArg1V1<Magnitude, Sgn>,
    1i32,
>;

/// Location: [src/lib/sgn/sgn.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/sgn/sgn.ml#L9)
///
/// Gid: 691
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum SgnStableV1VersionedV1 {
    Pos,
    Neg,
}

/// **Origin**: `Sgn.Stable.V1.t`
///
/// **Location**: [src/lib/sgn/sgn.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/sgn/sgn.ml#L9)
///
/// **Gid**: 693
pub type SgnStableV1Versioned = crate::versioned::Versioned<SgnStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/fee_excess.ml:123:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_excess.ml#L123)
///
/// Gid: 873
pub struct MinaBaseFeeExcessStableV1VersionedV1(
    pub  MinaBaseFeeExcessStableV1VersionedV1Poly<
        MinaBaseTokenIdStableV1Versioned,
        MinaBaseFeeExcessStableV1VersionedV1PolyArg1<
            CurrencyFeeStableV1Versioned,
            SgnStableV1Versioned,
        >,
    >,
);

/// **Origin**: `Mina_base__Fee_excess.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/fee_excess.ml:123:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_excess.ml#L123)
///
/// **Gid**: 875
pub type MinaBaseFeeExcessStableV1Versioned =
    crate::versioned::Versioned<MinaBaseFeeExcessStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/sok_message.ml:25:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/sok_message.ml#L25)
///
/// Gid: 1312
pub struct MinaBaseSokMessageDigestStableV1VersionedV1(pub crate::string::ByteString);

/// **Origin**: `Mina_base__Sok_message.Digest.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/sok_message.ml:25:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/sok_message.ml#L25)
///
/// **Gid**: 1313
pub type MinaBaseSokMessageDigestStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSokMessageDigestStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/transaction_snark/transaction_snark.ml:220:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L220)
///
/// Gid: 1484
pub struct TransactionSnarkStatementWithSokStableV1VersionedV1(
    pub  TransactionSnarkStatementWithSokStableV1VersionedV1Poly<
        MinaBaseStagedLedgerHashNonSnarkStableV1VersionedV1LedgerHash,
        CurrencyAmountMakeStrStableV1Versioned,
        TransactionSnarkPendingCoinbaseStackStateStableV1Versioned,
        MinaBaseFeeExcessStableV1Versioned,
        MinaBaseTokenIdStableV1Versioned,
        MinaBaseSokMessageDigestStableV1Versioned,
    >,
);

/// **Origin**: `Transaction_snark.Statement.With_sok.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark/transaction_snark.ml:220:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L220)
///
/// **Gid**: 1486
pub type TransactionSnarkStatementWithSokStableV1Versioned =
    crate::versioned::Versioned<TransactionSnarkStatementWithSokStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/transaction_snark/transaction_snark.ml:420:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L420)
///
/// Gid: 1490
pub struct TransactionSnarkProofStableV1VersionedV1(pub PicklesProofBranching2StableV1Versioned);

/// **Origin**: `Transaction_snark.Proof.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark/transaction_snark.ml:420:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L420)
///
/// **Gid**: 1492
pub type TransactionSnarkProofStableV1Versioned =
    crate::versioned::Versioned<TransactionSnarkProofStableV1VersionedV1, 1i32>;

/// Location: [src/lib/transaction_snark/transaction_snark.ml:432:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L432)
///
/// Gid: 1493
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkStableV1VersionedV1 {
    pub statement: TransactionSnarkStatementWithSokStableV1Versioned,
    pub proof: TransactionSnarkProofStableV1Versioned,
}

/// **Origin**: `Transaction_snark.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark/transaction_snark.ml:432:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L432)
///
/// **Gid**: 1495
pub type TransactionSnarkStableV1Versioned =
    crate::versioned::Versioned<TransactionSnarkStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/ledger_proof/ledger_proof.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/ledger_proof/ledger_proof.ml#L10)
///
/// Gid: 1499
pub struct LedgerProofProdStableV1VersionedV1(pub TransactionSnarkStableV1Versioned);

/// **Origin**: `Ledger_proof.Prod.Stable.V1.t`
///
/// **Location**: [src/lib/ledger_proof/ledger_proof.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/ledger_proof/ledger_proof.ml#L10)
///
/// **Gid**: 1501
pub type LedgerProofProdStableV1Versioned =
    crate::versioned::Versioned<LedgerProofProdStableV1VersionedV1, 1i32>;

/// Location: [src/lib/transaction_snark_work/transaction_snark_work.ml:82:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_work/transaction_snark_work.ml#L82)
///
/// Gid: 1513
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkWorkTStableV1VersionedV1 {
    pub fee: CurrencyFeeStableV1Versioned,
    pub proofs: TransactionSnarkWorkTStableV1VersionedV1Proofs<LedgerProofProdStableV1Versioned>,
    pub prover: ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8,
}

/// **Origin**: `Transaction_snark_work.T.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark_work/transaction_snark_work.ml:82:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_work/transaction_snark_work.ml#L82)
///
/// **Gid**: 1515
pub type TransactionSnarkWorkTStableV1Versioned =
    crate::versioned::Versioned<TransactionSnarkWorkTStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/transaction_status.ml:809:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L809)
///
/// Gid: 838
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseTransactionStatusAuxiliaryDataStableV1VersionedV1 {
    pub fee_payer_account_creation_fee_paid: Option<CurrencyAmountMakeStrStableV1Versioned>,
    pub receiver_account_creation_fee_paid: Option<CurrencyAmountMakeStrStableV1Versioned>,
    pub created_token: Option<MinaBaseTokenIdStableV1Versioned>,
}

/// **Origin**: `Mina_base__Transaction_status.Auxiliary_data.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_status.ml:809:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L809)
///
/// **Gid**: 840
pub type MinaBaseTransactionStatusAuxiliaryDataStableV1Versioned =
    crate::versioned::Versioned<MinaBaseTransactionStatusAuxiliaryDataStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/transaction_status.ml:692:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L692)
///
/// Gid: 826
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseTransactionStatusBalanceDataStableV1VersionedV1 {
    pub fee_payer_balance: Option<CurrencyBalanceStableV1Versioned>,
    pub source_balance: Option<CurrencyBalanceStableV1Versioned>,
    pub receiver_balance: Option<CurrencyBalanceStableV1Versioned>,
}

/// **Origin**: `Mina_base__Transaction_status.Balance_data.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_status.ml:692:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L692)
///
/// **Gid**: 828
pub type MinaBaseTransactionStatusBalanceDataStableV1Versioned =
    crate::versioned::Versioned<MinaBaseTransactionStatusBalanceDataStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/transaction_status.ml:13:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L13)
///
/// Gid: 823
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseTransactionStatusFailureStableV1VersionedV1 {
    Predicate,
    SourceNotPresent,
    ReceiverNotPresent,
    AmountInsufficientToCreateAccount,
    CannotPayCreationFeeInToken,
    SourceInsufficientBalance,
    SourceMinimumBalanceViolation,
    ReceiverAlreadyExists,
    NotTokenOwner,
    MismatchedTokenPermissions,
    Overflow,
    SignedCommandOnSnappAccount,
    SnappAccountNotPresent,
    UpdateNotPermitted,
    IncorrectNonce,
}

/// **Origin**: `Mina_base__Transaction_status.Failure.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_status.ml:13:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L13)
///
/// **Gid**: 825
pub type MinaBaseTransactionStatusFailureStableV1Versioned =
    crate::versioned::Versioned<MinaBaseTransactionStatusFailureStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/transaction_status.ml:832:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L832)
///
/// Gid: 841
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseTransactionStatusStableV1VersionedV1 {
    Applied(
        MinaBaseTransactionStatusAuxiliaryDataStableV1Versioned,
        MinaBaseTransactionStatusBalanceDataStableV1Versioned,
    ),
    Failed(
        MinaBaseTransactionStatusFailureStableV1Versioned,
        MinaBaseTransactionStatusBalanceDataStableV1Versioned,
    ),
}

/// **Origin**: `Mina_base__Transaction_status.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_status.ml:832:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L832)
///
/// **Gid**: 843
pub type MinaBaseTransactionStatusStableV1Versioned =
    crate::versioned::Versioned<MinaBaseTransactionStatusStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/with_status.ml#L6)
///
/// Gid: 844
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1VersionedV1PolyArg1V1<A> {
    pub data: A,
    pub status: MinaBaseTransactionStatusStableV1Versioned,
}

/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/with_status.ml#L6)
///
/// Gid: 846
pub type StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1VersionedV1PolyArg1<A> =
    crate::versioned::Versioned<
        StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1VersionedV1PolyArg1V1<A>,
        1i32,
    >;

/// Location: [src/lib/mina_base/user_command.ml:7:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/user_command.ml#L7)
///
/// Gid: 1138
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseUserCommandStableV1VersionedV1PolyV1<U, S> {
    SignedCommand(U),
    SnappCommand(S),
}

/// Location: [src/lib/mina_base/user_command.ml:7:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/user_command.ml#L7)
///
/// Gid: 1140
pub type MinaBaseUserCommandStableV1VersionedV1Poly<U, S> =
    crate::versioned::Versioned<MinaBaseUserCommandStableV1VersionedV1PolyV1<U, S>, 1i32>;

/// Location: [src/lib/mina_base/signed_command.ml:13:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command.ml#L13)
///
/// Gid: 932
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandStableV1VersionedV1PolyV1<Payload, Pk, Signature> {
    pub payload: Payload,
    pub signer: Pk,
    pub signature: Signature,
}

/// Location: [src/lib/mina_base/signed_command.ml:13:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command.ml#L13)
///
/// Gid: 934
pub type MinaBaseSignedCommandStableV1VersionedV1Poly<Payload, Pk, Signature> =
    crate::versioned::Versioned<
        MinaBaseSignedCommandStableV1VersionedV1PolyV1<Payload, Pk, Signature>,
        1i32,
    >;

/// Location: [src/lib/mina_base/signed_command_payload.ml:402:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L402)
///
/// Gid: 926
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandPayloadStableV1VersionedV1PolyV1<Common, Body> {
    pub common: Common,
    pub body: Body,
}

/// Location: [src/lib/mina_base/signed_command_payload.ml:402:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L402)
///
/// Gid: 928
pub type MinaBaseSignedCommandPayloadStableV1VersionedV1Poly<Common, Body> =
    crate::versioned::Versioned<
        MinaBaseSignedCommandPayloadStableV1VersionedV1PolyV1<Common, Body>,
        1i32,
    >;

/// Location: [src/lib/mina_base/signed_command_payload.ml:23:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L23)
///
/// Gid: 913
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandPayloadCommonBinableArgStableV1VersionedV1PolyV1<
    Fee,
    PublicKey,
    TokenId,
    Nonce,
    GlobalSlot,
    Memo,
> {
    pub fee: Fee,
    pub fee_token: TokenId,
    pub fee_payer_pk: PublicKey,
    pub nonce: Nonce,
    pub valid_until: GlobalSlot,
    pub memo: Memo,
}

/// Location: [src/lib/mina_base/signed_command_payload.ml:23:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L23)
///
/// Gid: 915
pub type MinaBaseSignedCommandPayloadCommonBinableArgStableV1VersionedV1Poly<
    Fee,
    PublicKey,
    TokenId,
    Nonce,
    GlobalSlot,
    Memo,
> = crate::versioned::Versioned<
    MinaBaseSignedCommandPayloadCommonBinableArgStableV1VersionedV1PolyV1<
        Fee,
        PublicKey,
        TokenId,
        Nonce,
        GlobalSlot,
        Memo,
    >,
    1i32,
>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_numbers/nat.ml:186:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L186)
///
/// Gid: 673
pub struct MinaNumbersNatMake32StableV1VersionedV1(pub UnsignedExtendedUInt32StableV1Versioned);

/// **Origin**: `Mina_numbers__Nat.Make32.Stable.V1.t`
///
/// **Location**: [src/lib/mina_numbers/nat.ml:186:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L186)
///
/// **Gid**: 675
pub type MinaNumbersNatMake32StableV1Versioned =
    crate::versioned::Versioned<MinaNumbersNatMake32StableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/signed_command_memo.ml:16:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_memo.ml#L16)
///
/// Gid: 907
pub struct MinaBaseSignedCommandMemoStableV1VersionedV1(pub crate::string::ByteString);

/// **Origin**: `Mina_base__Signed_command_memo.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command_memo.ml:16:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_memo.ml#L16)
///
/// **Gid**: 909
pub type MinaBaseSignedCommandMemoStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSignedCommandMemoStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/signed_command_payload.ml:61:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L61)
///
/// Gid: 916
pub struct MinaBaseSignedCommandPayloadCommonBinableArgStableV1VersionedV1(
    pub  MinaBaseSignedCommandPayloadCommonBinableArgStableV1VersionedV1Poly<
        CurrencyFeeStableV1Versioned,
        ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8,
        MinaBaseTokenIdStableV1Versioned,
        MinaNumbersNatMake32StableV1Versioned,
        ConsensusGlobalSlotStableV1VersionedV1PolyArg0,
        MinaBaseSignedCommandMemoStableV1Versioned,
    >,
);

/// **Origin**: `Mina_base__Signed_command_payload.Common.Binable_arg.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command_payload.ml:61:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L61)
///
/// **Gid**: 918
pub type MinaBaseSignedCommandPayloadCommonBinableArgStableV1Versioned =
    crate::versioned::Versioned<
        MinaBaseSignedCommandPayloadCommonBinableArgStableV1VersionedV1,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/signed_command_payload.ml:83:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L83)
///
/// Gid: 919
pub struct MinaBaseSignedCommandPayloadCommonStableV1VersionedV1(
    pub MinaBaseSignedCommandPayloadCommonBinableArgStableV1Versioned,
);

/// **Origin**: `Mina_base__Signed_command_payload.Common.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command_payload.ml:83:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L83)
///
/// **Gid**: 920
pub type MinaBaseSignedCommandPayloadCommonStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSignedCommandPayloadCommonStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/payment_payload.ml:21:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/payment_payload.ml#L21)
///
/// Gid: 885
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePaymentPayloadStableV1VersionedV1PolyV1<PublicKey, TokenId, Amount> {
    pub source_pk: PublicKey,
    pub receiver_pk: PublicKey,
    pub token_id: TokenId,
    pub amount: Amount,
}

/// Location: [src/lib/mina_base/payment_payload.ml:21:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/payment_payload.ml#L21)
///
/// Gid: 887
pub type MinaBasePaymentPayloadStableV1VersionedV1Poly<PublicKey, TokenId, Amount> =
    crate::versioned::Versioned<
        MinaBasePaymentPayloadStableV1VersionedV1PolyV1<PublicKey, TokenId, Amount>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/payment_payload.ml:35:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/payment_payload.ml#L35)
///
/// Gid: 888
pub struct MinaBasePaymentPayloadStableV1VersionedV1(
    pub  MinaBasePaymentPayloadStableV1VersionedV1Poly<
        ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8,
        MinaBaseTokenIdStableV1Versioned,
        CurrencyAmountMakeStrStableV1Versioned,
    >,
);

/// **Origin**: `Mina_base__Payment_payload.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/payment_payload.ml:35:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/payment_payload.ml#L35)
///
/// **Gid**: 890
pub type MinaBasePaymentPayloadStableV1Versioned =
    crate::versioned::Versioned<MinaBasePaymentPayloadStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/stake_delegation.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/stake_delegation.ml#L9)
///
/// Gid: 910
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseStakeDelegationStableV1VersionedV1 {
    SetDelegate {
        delegator: ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8,
        new_delegate: ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8,
    },
}

/// **Origin**: `Mina_base__Stake_delegation.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/stake_delegation.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/stake_delegation.ml#L9)
///
/// **Gid**: 912
pub type MinaBaseStakeDelegationStableV1Versioned =
    crate::versioned::Versioned<MinaBaseStakeDelegationStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/new_token_payload.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/new_token_payload.ml#L7)
///
/// Gid: 882
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseNewTokenPayloadStableV1VersionedV1 {
    pub token_owner_pk: ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8,
    pub disable_new_accounts: bool,
}

/// **Origin**: `Mina_base__New_token_payload.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/new_token_payload.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/new_token_payload.ml#L7)
///
/// **Gid**: 884
pub type MinaBaseNewTokenPayloadStableV1Versioned =
    crate::versioned::Versioned<MinaBaseNewTokenPayloadStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/new_account_payload.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/new_account_payload.ml#L7)
///
/// Gid: 879
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseNewAccountPayloadStableV1VersionedV1 {
    pub token_id: MinaBaseTokenIdStableV1Versioned,
    pub token_owner_pk: ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8,
    pub receiver_pk: ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8,
    pub account_disabled: bool,
}

/// **Origin**: `Mina_base__New_account_payload.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/new_account_payload.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/new_account_payload.ml#L7)
///
/// **Gid**: 881
pub type MinaBaseNewAccountPayloadStableV1Versioned =
    crate::versioned::Versioned<MinaBaseNewAccountPayloadStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/minting_payload.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/minting_payload.ml#L7)
///
/// Gid: 876
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseMintingPayloadStableV1VersionedV1 {
    pub token_id: MinaBaseTokenIdStableV1Versioned,
    pub token_owner_pk: ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8,
    pub receiver_pk: ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8,
    pub amount: CurrencyAmountMakeStrStableV1Versioned,
}

/// **Origin**: `Mina_base__Minting_payload.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/minting_payload.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/minting_payload.ml#L7)
///
/// **Gid**: 878
pub type MinaBaseMintingPayloadStableV1Versioned =
    crate::versioned::Versioned<MinaBaseMintingPayloadStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/signed_command_payload.ml:197:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L197)
///
/// Gid: 921
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSignedCommandPayloadBodyBinableArgStableV1VersionedV1 {
    Payment(MinaBasePaymentPayloadStableV1Versioned),
    StakeDelegation(MinaBaseStakeDelegationStableV1Versioned),
    CreateNewToken(MinaBaseNewTokenPayloadStableV1Versioned),
    CreateTokenAccount(MinaBaseNewAccountPayloadStableV1Versioned),
    MintTokens(MinaBaseMintingPayloadStableV1Versioned),
}

/// **Origin**: `Mina_base__Signed_command_payload.Body.Binable_arg.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command_payload.ml:197:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L197)
///
/// **Gid**: 923
pub type MinaBaseSignedCommandPayloadBodyBinableArgStableV1Versioned = crate::versioned::Versioned<
    MinaBaseSignedCommandPayloadBodyBinableArgStableV1VersionedV1,
    1i32,
>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/signed_command_payload.ml:244:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L244)
///
/// Gid: 924
pub struct MinaBaseSignedCommandPayloadBodyStableV1VersionedV1(
    pub MinaBaseSignedCommandPayloadBodyBinableArgStableV1Versioned,
);

/// **Origin**: `Mina_base__Signed_command_payload.Body.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command_payload.ml:244:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L244)
///
/// **Gid**: 925
pub type MinaBaseSignedCommandPayloadBodyStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSignedCommandPayloadBodyStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/signed_command_payload.ml:416:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L416)
///
/// Gid: 929
pub struct MinaBaseSignedCommandPayloadStableV1VersionedV1(
    pub  MinaBaseSignedCommandPayloadStableV1VersionedV1Poly<
        MinaBaseSignedCommandPayloadCommonStableV1Versioned,
        MinaBaseSignedCommandPayloadBodyStableV1Versioned,
    >,
);

/// **Origin**: `Mina_base__Signed_command_payload.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command_payload.ml:416:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L416)
///
/// **Gid**: 931
pub type MinaBaseSignedCommandPayloadStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSignedCommandPayloadStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/non_zero_curve_point/non_zero_curve_point.ml:194:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/non_zero_curve_point.ml#L194)
///
/// Gid: 745
pub struct NonZeroCurvePointUncompressedStableV1VersionedV1(
    pub ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8,
);

/// **Origin**: `Non_zero_curve_point.Uncompressed.Stable.V1.t`
///
/// **Location**: [src/lib/non_zero_curve_point/non_zero_curve_point.ml:194:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/non_zero_curve_point.ml#L194)
///
/// **Gid**: 746
pub type NonZeroCurvePointUncompressedStableV1Versioned =
    crate::versioned::Versioned<NonZeroCurvePointUncompressedStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/signature_poly.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signature_poly.ml#L6)
///
/// Gid: 860
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignatureStableV1VersionedV1PolyV1<Field, Scalar>(pub Field, pub Scalar);

/// Location: [src/lib/mina_base/signature_poly.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signature_poly.ml#L6)
///
/// Gid: 862
pub type MinaBaseSignatureStableV1VersionedV1Poly<Field, Scalar> =
    crate::versioned::Versioned<MinaBaseSignatureStableV1VersionedV1PolyV1<Field, Scalar>, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/signature.ml:18:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signature.ml#L18)
///
/// Gid: 864
pub struct MinaBaseSignatureStableV1VersionedV1(
    pub MinaBaseSignatureStableV1VersionedV1Poly<crate::bigint::BigInt, crate::bigint::BigInt>,
);

/// **Origin**: `Mina_base__Signature.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signature.ml:18:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signature.ml#L18)
///
/// **Gid**: 866
pub type MinaBaseSignatureStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSignatureStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/signed_command.ml:23:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command.ml#L23)
///
/// Gid: 935
pub struct MinaBaseSignedCommandStableV1VersionedV1(
    pub  MinaBaseSignedCommandStableV1VersionedV1Poly<
        MinaBaseSignedCommandPayloadStableV1Versioned,
        NonZeroCurvePointUncompressedStableV1Versioned,
        MinaBaseSignatureStableV1Versioned,
    >,
);

/// **Origin**: `Mina_base__Signed_command.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command.ml:23:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command.ml#L23)
///
/// **Gid**: 937
pub type MinaBaseSignedCommandStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSignedCommandStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/other_fee_payer.ml:10:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/other_fee_payer.ml#L10)
///
/// Gid: 1025
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseOtherFeePayerPayloadStableV1VersionedV1PolyV1<Pk, TokenId, Nonce, Fee> {
    pub pk: Pk,
    pub token_id: TokenId,
    pub nonce: Nonce,
    pub fee: Fee,
}

/// Location: [src/lib/mina_base/other_fee_payer.ml:10:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/other_fee_payer.ml#L10)
///
/// Gid: 1027
pub type MinaBaseOtherFeePayerPayloadStableV1VersionedV1Poly<Pk, TokenId, Nonce, Fee> =
    crate::versioned::Versioned<
        MinaBaseOtherFeePayerPayloadStableV1VersionedV1PolyV1<Pk, TokenId, Nonce, Fee>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/other_fee_payer.ml:20:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/other_fee_payer.ml#L20)
///
/// Gid: 1028
pub struct MinaBaseOtherFeePayerPayloadStableV1VersionedV1(
    pub  MinaBaseOtherFeePayerPayloadStableV1VersionedV1Poly<
        ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8,
        MinaBaseTokenIdStableV1Versioned,
        MinaNumbersNatMake32StableV1Versioned,
        CurrencyFeeStableV1Versioned,
    >,
);

/// **Origin**: `Mina_base__Other_fee_payer.Payload.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/other_fee_payer.ml:20:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/other_fee_payer.ml#L20)
///
/// **Gid**: 1030
pub type MinaBaseOtherFeePayerPayloadStableV1Versioned =
    crate::versioned::Versioned<MinaBaseOtherFeePayerPayloadStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/other_fee_payer.ml:84:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/other_fee_payer.ml#L84)
///
/// Gid: 1031
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseOtherFeePayerStableV1VersionedV1 {
    pub payload: MinaBaseOtherFeePayerPayloadStableV1Versioned,
    pub signature: MinaBaseSignatureStableV1Versioned,
}

/// **Origin**: `Mina_base__Other_fee_payer.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/other_fee_payer.ml:84:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/other_fee_payer.ml#L84)
///
/// **Gid**: 1033
pub type MinaBaseOtherFeePayerStableV1Versioned =
    crate::versioned::Versioned<MinaBaseOtherFeePayerStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:352:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L352)
///
/// Gid: 1109
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandBinableArgStableV1VersionedV1ProvedEmpty0V1<One, Two> {
    pub token_id: MinaBaseTokenIdStableV1Versioned,
    pub fee_payment: Option<MinaBaseOtherFeePayerStableV1Versioned>,
    pub one: One,
    pub two: Two,
}

/// Location: [src/lib/mina_base/snapp_command.ml:352:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L352)
///
/// Gid: 1111
pub type MinaBaseSnappCommandBinableArgStableV1VersionedV1ProvedEmpty0<One, Two> =
    crate::versioned::Versioned<
        MinaBaseSnappCommandBinableArgStableV1VersionedV1ProvedEmpty0V1<One, Two>,
        1i32,
    >;

/// Location: [src/lib/mina_base/snapp_command.ml:298:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L298)
///
/// Gid: 1097
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyAuthorizedProvedStableV1VersionedV1PolyV1<Data, Auth> {
    pub data: Data,
    pub authorization: Auth,
}

/// Location: [src/lib/mina_base/snapp_command.ml:298:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L298)
///
/// Gid: 1099
pub type MinaBaseSnappCommandPartyAuthorizedProvedStableV1VersionedV1Poly<Data, Auth> =
    crate::versioned::Versioned<
        MinaBaseSnappCommandPartyAuthorizedProvedStableV1VersionedV1PolyV1<Data, Auth>,
        1i32,
    >;

/// Location: [src/lib/mina_base/snapp_command.ml:211:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L211)
///
/// Gid: 1085
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyPredicatedProvedStableV1VersionedV1PolyV1<Body, Predicate> {
    pub body: Body,
    pub predicate: Predicate,
}

/// Location: [src/lib/mina_base/snapp_command.ml:211:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L211)
///
/// Gid: 1087
pub type MinaBaseSnappCommandPartyPredicatedProvedStableV1VersionedV1Poly<Body, Predicate> =
    crate::versioned::Versioned<
        MinaBaseSnappCommandPartyPredicatedProvedStableV1VersionedV1PolyV1<Body, Predicate>,
        1i32,
    >;

/// Location: [src/lib/mina_base/snapp_command.ml:136:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L136)
///
/// Gid: 1079
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyBodyStableV1VersionedV1PolyV1<Pk, Update, SignedAmount> {
    pub pk: Pk,
    pub update: Update,
    pub delta: SignedAmount,
}

/// Location: [src/lib/mina_base/snapp_command.ml:136:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L136)
///
/// Gid: 1081
pub type MinaBaseSnappCommandPartyBodyStableV1VersionedV1Poly<Pk, Update, SignedAmount> =
    crate::versioned::Versioned<
        MinaBaseSnappCommandPartyBodyStableV1VersionedV1PolyV1<Pk, Update, SignedAmount>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/vector.ml:503:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L503)
///
/// Gid: 471
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyUpdateStableV1VersionedV1PolyV1AppStateV1PolyV1<A>(
    pub A,
    pub (A, (A, (A, (A, (A, (A, (A, ()))))))),
);

/// Location: [src/lib/pickles_types/vector.ml:503:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L503)
///
/// Gid: 472
pub type MinaBaseSnappCommandPartyUpdateStableV1VersionedV1PolyV1AppStateV1Poly<A> =
    crate::versioned::Versioned<
        MinaBaseSnappCommandPartyUpdateStableV1VersionedV1PolyV1AppStateV1PolyV1<A>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/snapp_state.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_state.ml#L17)
///
/// Gid: 960
pub struct MinaBaseSnappCommandPartyUpdateStableV1VersionedV1PolyV1AppStateV1<A>(
    pub MinaBaseSnappCommandPartyUpdateStableV1VersionedV1PolyV1AppStateV1Poly<A>,
);

/// Location: [src/lib/mina_base/snapp_state.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_state.ml#L17)
///
/// Gid: 962
pub type MinaBaseSnappCommandPartyUpdateStableV1VersionedV1PolyV1AppState<A> =
    crate::versioned::Versioned<
        MinaBaseSnappCommandPartyUpdateStableV1VersionedV1PolyV1AppStateV1<A>,
        1i32,
    >;

/// Location: [src/lib/mina_base/snapp_command.ml:35:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L35)
///
/// Gid: 1073
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyUpdateStableV1VersionedV1PolyV1<StateElement, Pk, Vk, Perms> {
    pub app_state: MinaBaseSnappCommandPartyUpdateStableV1VersionedV1PolyV1AppState<StateElement>,
    pub delegate: Pk,
    pub verification_key: Vk,
    pub permissions: Perms,
}

/// Location: [src/lib/mina_base/snapp_command.ml:35:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L35)
///
/// Gid: 1075
pub type MinaBaseSnappCommandPartyUpdateStableV1VersionedV1Poly<StateElement, Pk, Vk, Perms> =
    crate::versioned::Versioned<
        MinaBaseSnappCommandPartyUpdateStableV1VersionedV1PolyV1<StateElement, Pk, Vk, Perms>,
        1i32,
    >;

/// Location: [src/lib/mina_base/snapp_basic.ml:87:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_basic.ml#L87)
///
/// Gid: 951
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSnappCommandPartyUpdateStableV1VersionedV1PolyArg0V1<A> {
    Set(A),
    Keep,
}

/// Location: [src/lib/mina_base/snapp_basic.ml:87:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_basic.ml#L87)
///
/// Gid: 953
pub type MinaBaseSnappCommandPartyUpdateStableV1VersionedV1PolyArg0<A> =
    crate::versioned::Versioned<
        MinaBaseSnappCommandPartyUpdateStableV1VersionedV1PolyArg0V1<A>,
        1i32,
    >;

/// Location: [src/lib/with_hash/with_hash.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/with_hash/with_hash.ml#L8)
///
/// Gid: 775
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyUpdateStableV1VersionedV1PolyArg0Arg0V1<A, H> {
    pub data: A,
    pub hash: H,
}

/// Location: [src/lib/with_hash/with_hash.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/with_hash/with_hash.ml#L8)
///
/// Gid: 777
pub type MinaBaseSnappCommandPartyUpdateStableV1VersionedV1PolyArg0Arg0<A, H> =
    crate::versioned::Versioned<
        MinaBaseSnappCommandPartyUpdateStableV1VersionedV1PolyArg0Arg0V1<A, H>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/pickles_types/at_most.ml:135:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/at_most.ml#L135)
///
/// Gid: 521
pub struct PicklesSideLoadedVerificationKeyRStableV1VersionedV1PolyV1StepDataV1PolyV1<A>(
    pub Vec<A>,
);

/// Location: [src/lib/pickles_types/at_most.ml:135:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/at_most.ml#L135)
///
/// Gid: 522
pub type PicklesSideLoadedVerificationKeyRStableV1VersionedV1PolyV1StepDataV1Poly<A> =
    crate::versioned::Versioned<
        PicklesSideLoadedVerificationKeyRStableV1VersionedV1PolyV1StepDataV1PolyV1<A>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:120:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L120)
///
/// Gid: 578
pub struct PicklesSideLoadedVerificationKeyRStableV1VersionedV1PolyV1StepDataV1<A>(
    pub PicklesSideLoadedVerificationKeyRStableV1VersionedV1PolyV1StepDataV1Poly<A>,
);

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:120:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L120)
///
/// Gid: 580
pub type PicklesSideLoadedVerificationKeyRStableV1VersionedV1PolyV1StepData<A> =
    crate::versioned::Versioned<
        PicklesSideLoadedVerificationKeyRStableV1VersionedV1PolyV1StepDataV1<A>,
        1i32,
    >;

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:136:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L136)
///
/// Gid: 581
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesSideLoadedVerificationKeyRStableV1VersionedV1PolyV1StepDataArg0V1<A> {
    pub h: A,
}

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:136:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L136)
///
/// Gid: 583
pub type PicklesSideLoadedVerificationKeyRStableV1VersionedV1PolyV1StepDataArg0<A> =
    crate::versioned::Versioned<
        PicklesSideLoadedVerificationKeyRStableV1VersionedV1PolyV1StepDataArg0V1<A>,
        1i32,
    >;

/// Location: [src/lib/pickles_base/domain.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/domain.ml#L6)
///
/// Gid: 563
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum PicklesBaseDomainStableV1VersionedV1 {
    Pow2RootsOfUnity(crate::number::Int32),
}

/// **Origin**: `Pickles_base__Domain.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_base/domain.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/domain.ml#L6)
///
/// **Gid**: 565
pub type PicklesBaseDomainStableV1Versioned =
    crate::versioned::Versioned<PicklesBaseDomainStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:44:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L44)
///
/// Gid: 569
pub struct PicklesBaseSideLoadedVerificationKeyWidthStableV1VersionedV1(pub crate::char::Char);

/// **Origin**: `Pickles_base__Side_loaded_verification_key.Width.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_base/side_loaded_verification_key.ml:44:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L44)
///
/// **Gid**: 571
pub type PicklesBaseSideLoadedVerificationKeyWidthStableV1Versioned =
    crate::versioned::Versioned<PicklesBaseSideLoadedVerificationKeyWidthStableV1VersionedV1, 1i32>;

/// Location: [src/lib/pickles_types/plonk_verification_key_evals.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_verification_key_evals.ml#L7)
///
/// Gid: 483
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesSideLoadedVerificationKeyRStableV1VersionedV1PolyV1WrapIndexV1<Comm> {
    pub sigma_comm_0: Comm,
    pub sigma_comm_1: Comm,
    pub sigma_comm_2: Comm,
    pub ql_comm: Comm,
    pub qr_comm: Comm,
    pub qo_comm: Comm,
    pub qm_comm: Comm,
    pub qc_comm: Comm,
    pub rcm_comm_0: Comm,
    pub rcm_comm_1: Comm,
    pub rcm_comm_2: Comm,
    pub psm_comm: Comm,
    pub add_comm: Comm,
    pub mul1_comm: Comm,
    pub mul2_comm: Comm,
    pub emul1_comm: Comm,
    pub emul2_comm: Comm,
    pub emul3_comm: Comm,
}

/// Location: [src/lib/pickles_types/plonk_verification_key_evals.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_verification_key_evals.ml#L7)
///
/// Gid: 485
pub type PicklesSideLoadedVerificationKeyRStableV1VersionedV1PolyV1WrapIndex<Comm> =
    crate::versioned::Versioned<
        PicklesSideLoadedVerificationKeyRStableV1VersionedV1PolyV1WrapIndexV1<Comm>,
        1i32,
    >;

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:150:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L150)
///
/// Gid: 584
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesSideLoadedVerificationKeyRStableV1VersionedV1PolyV1<G> {
    pub step_data: PicklesSideLoadedVerificationKeyRStableV1VersionedV1PolyV1StepData<(
        PicklesSideLoadedVerificationKeyRStableV1VersionedV1PolyV1StepDataArg0<
            PicklesBaseDomainStableV1Versioned,
        >,
        PicklesBaseSideLoadedVerificationKeyWidthStableV1Versioned,
    )>,
    pub max_width: PicklesBaseSideLoadedVerificationKeyWidthStableV1Versioned,
    pub wrap_index: PicklesSideLoadedVerificationKeyRStableV1VersionedV1PolyV1WrapIndex<Vec<G>>,
}

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:150:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L150)
///
/// Gid: 586
pub type PicklesSideLoadedVerificationKeyRStableV1VersionedV1Poly<G> = crate::versioned::Versioned<
    PicklesSideLoadedVerificationKeyRStableV1VersionedV1PolyV1<G>,
    1i32,
>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/pickles/side_loaded_verification_key.ml:156:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/side_loaded_verification_key.ml#L156)
///
/// Gid: 634
pub struct PicklesSideLoadedVerificationKeyRStableV1VersionedV1(
    pub  PicklesSideLoadedVerificationKeyRStableV1VersionedV1Poly<
        PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyArg0,
    >,
);

/// **Origin**: `Pickles__Side_loaded_verification_key.R.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/side_loaded_verification_key.ml:156:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/side_loaded_verification_key.ml#L156)
///
/// **Gid**: 636
pub type PicklesSideLoadedVerificationKeyRStableV1Versioned =
    crate::versioned::Versioned<PicklesSideLoadedVerificationKeyRStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/pickles/side_loaded_verification_key.ml:166:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/side_loaded_verification_key.ml#L166)
///
/// Gid: 637
pub struct PicklesSideLoadedVerificationKeyStableV1VersionedV1(
    pub PicklesSideLoadedVerificationKeyRStableV1Versioned,
);

/// **Origin**: `Pickles__Side_loaded_verification_key.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/side_loaded_verification_key.ml:166:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/side_loaded_verification_key.ml#L166)
///
/// **Gid**: 638
pub type PicklesSideLoadedVerificationKeyStableV1Versioned =
    crate::versioned::Versioned<PicklesSideLoadedVerificationKeyStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/permissions.ml:287:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/permissions.ml#L287)
///
/// Gid: 901
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePermissionsStableV1VersionedV1PolyV1<Bool, Controller> {
    pub stake: Bool,
    pub edit_state: Controller,
    pub send: Controller,
    pub receive: Controller,
    pub set_delegate: Controller,
    pub set_permissions: Controller,
    pub set_verification_key: Controller,
}

/// Location: [src/lib/mina_base/permissions.ml:287:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/permissions.ml#L287)
///
/// Gid: 903
pub type MinaBasePermissionsStableV1VersionedV1Poly<Bool, Controller> = crate::versioned::Versioned<
    MinaBasePermissionsStableV1VersionedV1PolyV1<Bool, Controller>,
    1i32,
>;

/// Location: [src/lib/mina_base/permissions.ml:52:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/permissions.ml#L52)
///
/// Gid: 898
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBasePermissionsAuthRequiredStableV1VersionedV1 {
    None,
    Either,
    Proof,
    Signature,
    Both,
    Impossible,
}

/// **Origin**: `Mina_base__Permissions.Auth_required.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/permissions.ml:52:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/permissions.ml#L52)
///
/// **Gid**: 900
pub type MinaBasePermissionsAuthRequiredStableV1Versioned =
    crate::versioned::Versioned<MinaBasePermissionsAuthRequiredStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/permissions.ml:313:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/permissions.ml#L313)
///
/// Gid: 904
pub struct MinaBasePermissionsStableV1VersionedV1(
    pub  MinaBasePermissionsStableV1VersionedV1Poly<
        bool,
        MinaBasePermissionsAuthRequiredStableV1Versioned,
    >,
);

/// **Origin**: `Mina_base__Permissions.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/permissions.ml:313:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/permissions.ml#L313)
///
/// **Gid**: 906
pub type MinaBasePermissionsStableV1Versioned =
    crate::versioned::Versioned<MinaBasePermissionsStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/snapp_command.ml:51:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L51)
///
/// Gid: 1076
pub struct MinaBaseSnappCommandPartyUpdateStableV1VersionedV1(
    pub  MinaBaseSnappCommandPartyUpdateStableV1VersionedV1Poly<
        MinaBaseSnappCommandPartyUpdateStableV1VersionedV1PolyArg0<crate::bigint::BigInt>,
        MinaBaseSnappCommandPartyUpdateStableV1VersionedV1PolyArg0<
            ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8,
        >,
        MinaBaseSnappCommandPartyUpdateStableV1VersionedV1PolyArg0<
            MinaBaseSnappCommandPartyUpdateStableV1VersionedV1PolyArg0Arg0<
                PicklesSideLoadedVerificationKeyStableV1Versioned,
                crate::bigint::BigInt,
            >,
        >,
        MinaBaseSnappCommandPartyUpdateStableV1VersionedV1PolyArg0<
            MinaBasePermissionsStableV1Versioned,
        >,
    >,
);

/// **Origin**: `Mina_base__Snapp_command.Party.Update.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:51:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L51)
///
/// **Gid**: 1078
pub type MinaBaseSnappCommandPartyUpdateStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyUpdateStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/snapp_command.ml:146:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L146)
///
/// Gid: 1082
pub struct MinaBaseSnappCommandPartyBodyStableV1VersionedV1(
    pub  MinaBaseSnappCommandPartyBodyStableV1VersionedV1Poly<
        ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8,
        MinaBaseSnappCommandPartyUpdateStableV1Versioned,
        MinaBaseFeeExcessStableV1VersionedV1PolyArg1<
            CurrencyAmountMakeStrStableV1Versioned,
            SgnStableV1Versioned,
        >,
    >,
);

/// **Origin**: `Mina_base__Snapp_command.Party.Body.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:146:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L146)
///
/// **Gid**: 1084
pub type MinaBaseSnappCommandPartyBodyStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyBodyStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_predicate.ml:1167:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L1167)
///
/// Gid: 1067
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateStableV1VersionedV1PolyV1<Account, ProtocolState, Other, Pk> {
    pub self_predicate: Account,
    pub other: Other,
    pub fee_payer: Pk,
    pub protocol_state_predicate: ProtocolState,
}

/// Location: [src/lib/mina_base/snapp_predicate.ml:1167:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L1167)
///
/// Gid: 1069
pub type MinaBaseSnappPredicateStableV1VersionedV1Poly<Account, ProtocolState, Other, Pk> =
    crate::versioned::Versioned<
        MinaBaseSnappPredicateStableV1VersionedV1PolyV1<Account, ProtocolState, Other, Pk>,
        1i32,
    >;

/// Location: [src/lib/mina_base/snapp_predicate.ml:353:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L353)
///
/// Gid: 1040
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateAccountStableV1VersionedV1PolyV1<
    Balance,
    Nonce,
    ReceiptChainHash,
    Pk,
    Field,
> {
    pub balance: Balance,
    pub nonce: Nonce,
    pub receipt_chain_hash: ReceiptChainHash,
    pub public_key: Pk,
    pub delegate: Pk,
    pub state: MinaBaseSnappCommandPartyUpdateStableV1VersionedV1PolyV1AppState<Field>,
}

/// Location: [src/lib/mina_base/snapp_predicate.ml:353:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L353)
///
/// Gid: 1042
pub type MinaBaseSnappPredicateAccountStableV1VersionedV1Poly<
    Balance,
    Nonce,
    ReceiptChainHash,
    Pk,
    Field,
> = crate::versioned::Versioned<
    MinaBaseSnappPredicateAccountStableV1VersionedV1PolyV1<
        Balance,
        Nonce,
        ReceiptChainHash,
        Pk,
        Field,
    >,
    1i32,
>;

/// Location: [src/lib/mina_base/snapp_basic.ml:158:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_basic.ml#L158)
///
/// Gid: 954
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1PolyV1<A> {
    Check(A),
    Ignore,
}

/// Location: [src/lib/mina_base/snapp_basic.ml:158:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_basic.ml#L158)
///
/// Gid: 956
pub type MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1Poly<A> =
    crate::versioned::Versioned<
        MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1PolyV1<A>,
        1i32,
    >;

/// Location: [src/lib/mina_base/snapp_predicate.ml:23:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L23)
///
/// Gid: 1034
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1PolyArg0V1<A> {
    pub lower: A,
    pub upper: A,
}

/// Location: [src/lib/mina_base/snapp_predicate.ml:23:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L23)
///
/// Gid: 1036
pub type MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1PolyArg0<A> =
    crate::versioned::Versioned<
        MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1PolyArg0V1<A>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/snapp_predicate.ml:150:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L150)
///
/// Gid: 1037
pub struct MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1<A>(
    pub  MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1Poly<
        MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1PolyArg0<A>,
    >,
);

/// Location: [src/lib/mina_base/snapp_predicate.ml:150:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L150)
///
/// Gid: 1039
pub type MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0<A> = crate::versioned::Versioned<
    MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1<A>,
    1i32,
>;

/// Location: [src/lib/mina_base/receipt.ml:30:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/receipt.ml#L30)
///
/// Gid: 943
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1PolyArg0Dup1V1Poly(
    pub crate::bigint::BigInt,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/receipt.ml:29:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/receipt.ml#L29)
///
/// Gid: 946
pub struct MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1PolyArg0Dup1V1(
    pub MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1PolyArg0Dup1V1Poly,
);

/// Location: [src/lib/mina_base/receipt.ml:29:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/receipt.ml#L29)
///
/// Gid: 947
pub type MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1PolyArg0Dup1 =
    crate::versioned::Versioned<
        MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1PolyArg0Dup1V1,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/snapp_predicate.ml:369:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L369)
///
/// Gid: 1043
pub struct MinaBaseSnappPredicateAccountStableV1VersionedV1(
    pub  MinaBaseSnappPredicateAccountStableV1VersionedV1Poly<
        MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0<CurrencyBalanceStableV1Versioned>,
        MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0<
            MinaNumbersNatMake32StableV1Versioned,
        >,
        MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1Poly<
            MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1PolyArg0Dup1,
        >,
        MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1Poly<
            ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8,
        >,
        MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1Poly<crate::bigint::BigInt>,
    >,
);

/// **Origin**: `Mina_base__Snapp_predicate.Account.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_predicate.ml:369:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L369)
///
/// **Gid**: 1045
pub type MinaBaseSnappPredicateAccountStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSnappPredicateAccountStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_predicate.ml:603:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L603)
///
/// Gid: 1049
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateProtocolStateStableV1VersionedV1PolyV1<
    SnarkedLedgerHash,
    TokenId,
    Time,
    Length,
    VrfOutput,
    GlobalSlot,
    Amount,
    EpochData,
> {
    pub snarked_ledger_hash: SnarkedLedgerHash,
    pub snarked_next_available_token: TokenId,
    pub timestamp: Time,
    pub blockchain_length: Length,
    pub min_window_density: Length,
    pub last_vrf_output: VrfOutput,
    pub total_currency: Amount,
    pub curr_global_slot: GlobalSlot,
    pub global_slot_since_genesis: GlobalSlot,
    pub staking_epoch_data: EpochData,
    pub next_epoch_data: EpochData,
}

/// Location: [src/lib/mina_base/snapp_predicate.ml:603:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L603)
///
/// Gid: 1051
pub type MinaBaseSnappPredicateProtocolStateStableV1VersionedV1Poly<
    SnarkedLedgerHash,
    TokenId,
    Time,
    Length,
    VrfOutput,
    GlobalSlot,
    Amount,
    EpochData,
> = crate::versioned::Versioned<
    MinaBaseSnappPredicateProtocolStateStableV1VersionedV1PolyV1<
        SnarkedLedgerHash,
        TokenId,
        Time,
        Length,
        VrfOutput,
        GlobalSlot,
        Amount,
        EpochData,
    >,
    1i32,
>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/snapp_predicate.ml:535:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L535)
///
/// Gid: 1046
pub struct MinaBaseSnappPredicateProtocolStateEpochDataStableV1VersionedV1(
    pub  ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1VersionedV1Poly<
        MinaBaseEpochLedgerValueStableV1VersionedV1Poly<
            MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1Poly<
                MinaBaseStagedLedgerHashNonSnarkStableV1VersionedV1LedgerHash,
            >,
            MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0<
                CurrencyAmountMakeStrStableV1Versioned,
            >,
        >,
        MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1Poly<
            ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1VersionedV1PolyArg1,
        >,
        MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1Poly<
            MinaStateProtocolStateValueStableV1VersionedV1PolyArg0,
        >,
        MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1Poly<
            MinaStateProtocolStateValueStableV1VersionedV1PolyArg0,
        >,
        MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0<
            ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg0,
        >,
    >,
);

/// **Origin**: `Mina_base__Snapp_predicate.Protocol_state.Epoch_data.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_predicate.ml:535:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L535)
///
/// **Gid**: 1048
pub type MinaBaseSnappPredicateProtocolStateEpochDataStableV1Versioned =
    crate::versioned::Versioned<
        MinaBaseSnappPredicateProtocolStateEpochDataStableV1VersionedV1,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/snapp_predicate.ml:644:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L644)
///
/// Gid: 1052
pub struct MinaBaseSnappPredicateProtocolStateStableV1VersionedV1(
    pub  MinaBaseSnappPredicateProtocolStateStableV1VersionedV1Poly<
        MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1Poly<
            MinaBaseStagedLedgerHashNonSnarkStableV1VersionedV1LedgerHash,
        >,
        MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0<MinaBaseTokenIdStableV1Versioned>,
        MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0<BlockTimeTimeStableV1Versioned>,
        MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0<
            ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg0,
        >,
        (),
        MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0<
            ConsensusGlobalSlotStableV1VersionedV1PolyArg0,
        >,
        MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0<
            CurrencyAmountMakeStrStableV1Versioned,
        >,
        MinaBaseSnappPredicateProtocolStateEpochDataStableV1Versioned,
    >,
);

/// **Origin**: `Mina_base__Snapp_predicate.Protocol_state.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_predicate.ml:644:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L644)
///
/// **Gid**: 1054
pub type MinaBaseSnappPredicateProtocolStateStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSnappPredicateProtocolStateStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_predicate.ml:1100:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L1100)
///
/// Gid: 1061
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateOtherStableV1VersionedV1PolyV1<Account, AccountTransition, Vk> {
    pub predicate: Account,
    pub account_transition: AccountTransition,
    pub account_vk: Vk,
}

/// Location: [src/lib/mina_base/snapp_predicate.ml:1100:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L1100)
///
/// Gid: 1063
pub type MinaBaseSnappPredicateOtherStableV1VersionedV1Poly<Account, AccountTransition, Vk> =
    crate::versioned::Versioned<
        MinaBaseSnappPredicateOtherStableV1VersionedV1PolyV1<Account, AccountTransition, Vk>,
        1i32,
    >;

/// Location: [src/lib/mina_base/snapp_basic.ml:21:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_basic.ml#L21)
///
/// Gid: 948
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateOtherStableV1VersionedV1PolyArg1V1<A> {
    pub prev: A,
    pub next: A,
}

/// Location: [src/lib/mina_base/snapp_basic.ml:21:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_basic.ml#L21)
///
/// Gid: 950
pub type MinaBaseSnappPredicateOtherStableV1VersionedV1PolyArg1<A> =
    crate::versioned::Versioned<MinaBaseSnappPredicateOtherStableV1VersionedV1PolyArg1V1<A>, 1i32>;

/// Location: [src/lib/mina_base/snapp_basic.ml:234:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_basic.ml#L234)
///
/// Gid: 957
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSnappBasicAccountStateStableV1VersionedV1 {
    Empty,
    NonEmpty,
    Any,
}

/// **Origin**: `Mina_base__Snapp_basic.Account_state.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_basic.ml:234:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_basic.ml#L234)
///
/// **Gid**: 959
pub type MinaBaseSnappBasicAccountStateStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSnappBasicAccountStateStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/snapp_predicate.ml:1113:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L1113)
///
/// Gid: 1064
pub struct MinaBaseSnappPredicateOtherStableV1VersionedV1(
    pub  MinaBaseSnappPredicateOtherStableV1VersionedV1Poly<
        MinaBaseSnappPredicateAccountStableV1Versioned,
        MinaBaseSnappPredicateOtherStableV1VersionedV1PolyArg1<
            MinaBaseSnappBasicAccountStateStableV1Versioned,
        >,
        MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1Poly<crate::bigint::BigInt>,
    >,
);

/// **Origin**: `Mina_base__Snapp_predicate.Other.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_predicate.ml:1113:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L1113)
///
/// **Gid**: 1066
pub type MinaBaseSnappPredicateOtherStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSnappPredicateOtherStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/snapp_predicate.ml:1188:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L1188)
///
/// Gid: 1070
pub struct MinaBaseSnappPredicateStableV1VersionedV1(
    pub  MinaBaseSnappPredicateStableV1VersionedV1Poly<
        MinaBaseSnappPredicateAccountStableV1Versioned,
        MinaBaseSnappPredicateProtocolStateStableV1Versioned,
        MinaBaseSnappPredicateOtherStableV1Versioned,
        MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1Poly<
            ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8,
        >,
    >,
);

/// **Origin**: `Mina_base__Snapp_predicate.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_predicate.ml:1188:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L1188)
///
/// **Gid**: 1072
pub type MinaBaseSnappPredicateStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSnappPredicateStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/snapp_command.ml:225:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L225)
///
/// Gid: 1088
pub struct MinaBaseSnappCommandPartyPredicatedProvedStableV1VersionedV1(
    pub  MinaBaseSnappCommandPartyPredicatedProvedStableV1VersionedV1Poly<
        MinaBaseSnappCommandPartyBodyStableV1Versioned,
        MinaBaseSnappPredicateStableV1Versioned,
    >,
);

/// **Origin**: `Mina_base__Snapp_command.Party.Predicated.Proved.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:225:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L225)
///
/// **Gid**: 1090
pub type MinaBaseSnappCommandPartyPredicatedProvedStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyPredicatedProvedStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:66:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L66)
///
/// Gid: 572
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyArg0Arg1V1<A>(
    pub PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg0<A>,
);

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:66:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L66)
///
/// Gid: 574
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyArg0Arg1<A> = crate::versioned::Versioned<
    PicklesProofBranching2ReprStableV1VersionedV1PolyArg0Arg1V1<A>,
    1i32,
>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:87:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L87)
///
/// Gid: 575
pub struct PicklesProofBranching2ReprStableV1VersionedV1PolyArg1Arg1Dup1V1<A>(
    pub PicklesProofBranching2ReprStableV1VersionedV1PolyArg1Arg1<A>,
);

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:87:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L87)
///
/// Gid: 577
pub type PicklesProofBranching2ReprStableV1VersionedV1PolyArg1Arg1Dup1<A> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1VersionedV1PolyArg1Arg1Dup1V1<A>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/pickles/proof.ml:352:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L352)
///
/// Gid: 664
pub struct PicklesProofBranchingMaxReprStableV1VersionedV1(
    pub  PicklesProofBranching2ReprStableV1VersionedV1Poly<
        PicklesProofBranching2ReprStableV1VersionedV1PolyArg0<
            PicklesProofBranching2ReprStableV1VersionedV1PolyArg0Arg0,
            PicklesProofBranching2ReprStableV1VersionedV1PolyArg0Arg1<
                PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1Versioned,
            >,
        >,
        PicklesProofBranching2ReprStableV1VersionedV1PolyArg1<
            (),
            PicklesProofBranching2ReprStableV1VersionedV1PolyArg1Arg1Dup1<
                PicklesProofBranching2ReprStableV1VersionedV1PolyV1ProofV1PolyArg0,
            >,
            PicklesProofBranching2ReprStableV1VersionedV1PolyArg1Arg1Dup1<
                PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg7<
                    PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg7Arg0<
                        PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg1<
                            PicklesProofBranching2ReprStableV1VersionedV1PolyV1StatementArg0<
                                LimbVectorConstantHex64StableV1Versioned,
                            >,
                        >,
                    >,
                >,
            >,
        >,
    >,
);

/// **Origin**: `Pickles__Proof.Branching_max.Repr.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/proof.ml:352:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L352)
///
/// **Gid**: 666
pub type PicklesProofBranchingMaxReprStableV1Versioned =
    crate::versioned::Versioned<PicklesProofBranchingMaxReprStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/pickles/proof.ml:388:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L388)
///
/// Gid: 667
pub struct PicklesProofBranchingMaxStableV1VersionedV1(
    pub PicklesProofBranchingMaxReprStableV1Versioned,
);

/// **Origin**: `Pickles__Proof.Branching_max.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/proof.ml:388:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L388)
///
/// **Gid**: 668
pub type PicklesProofBranchingMaxStableV1Versioned =
    crate::versioned::Versioned<PicklesProofBranchingMaxStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/control.ml:11:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/control.ml#L11)
///
/// Gid: 867
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseControlStableV1VersionedV1 {
    Proof(PicklesProofBranchingMaxStableV1Versioned),
    Signature(MinaBaseSignatureStableV1Versioned),
    Both {
        signature: MinaBaseSignatureStableV1Versioned,
        proof: PicklesProofBranchingMaxStableV1Versioned,
    },
    NoneGiven,
}

/// **Origin**: `Mina_base__Control.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/control.ml:11:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/control.ml#L11)
///
/// **Gid**: 869
pub type MinaBaseControlStableV1Versioned =
    crate::versioned::Versioned<MinaBaseControlStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/snapp_command.ml:308:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L308)
///
/// Gid: 1100
pub struct MinaBaseSnappCommandPartyAuthorizedProvedStableV1VersionedV1(
    pub  MinaBaseSnappCommandPartyAuthorizedProvedStableV1VersionedV1Poly<
        MinaBaseSnappCommandPartyPredicatedProvedStableV1Versioned,
        MinaBaseControlStableV1Versioned,
    >,
);

/// **Origin**: `Mina_base__Snapp_command.Party.Authorized.Proved.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:308:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L308)
///
/// **Gid**: 1102
pub type MinaBaseSnappCommandPartyAuthorizedProvedStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyAuthorizedProvedStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/snapp_command.ml:280:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L280)
///
/// Gid: 1094
pub struct MinaBaseSnappCommandPartyPredicatedEmptyStableV1VersionedV1(
    pub  MinaBaseSnappCommandPartyPredicatedProvedStableV1VersionedV1Poly<
        MinaBaseSnappCommandPartyBodyStableV1Versioned,
        (),
    >,
);

/// **Origin**: `Mina_base__Snapp_command.Party.Predicated.Empty.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:280:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L280)
///
/// **Gid**: 1096
pub type MinaBaseSnappCommandPartyPredicatedEmptyStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyPredicatedEmptyStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/snapp_command.ml:338:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L338)
///
/// Gid: 1106
pub struct MinaBaseSnappCommandPartyAuthorizedEmptyStableV1VersionedV1(
    pub  MinaBaseSnappCommandPartyAuthorizedProvedStableV1VersionedV1Poly<
        MinaBaseSnappCommandPartyPredicatedEmptyStableV1Versioned,
        (),
    >,
);

/// **Origin**: `Mina_base__Snapp_command.Party.Authorized.Empty.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:338:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L338)
///
/// **Gid**: 1108
pub type MinaBaseSnappCommandPartyAuthorizedEmptyStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyAuthorizedEmptyStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/snapp_command.ml:246:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L246)
///
/// Gid: 1091
pub struct MinaBaseSnappCommandPartyPredicatedSignedStableV1VersionedV1(
    pub  MinaBaseSnappCommandPartyPredicatedProvedStableV1VersionedV1Poly<
        MinaBaseSnappCommandPartyBodyStableV1Versioned,
        MinaNumbersNatMake32StableV1Versioned,
    >,
);

/// **Origin**: `Mina_base__Snapp_command.Party.Predicated.Signed.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:246:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L246)
///
/// **Gid**: 1093
pub type MinaBaseSnappCommandPartyPredicatedSignedStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyPredicatedSignedStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/snapp_command.ml:323:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L323)
///
/// Gid: 1103
pub struct MinaBaseSnappCommandPartyAuthorizedSignedStableV1VersionedV1(
    pub  MinaBaseSnappCommandPartyAuthorizedProvedStableV1VersionedV1Poly<
        MinaBaseSnappCommandPartyPredicatedSignedStableV1Versioned,
        MinaBaseSignatureStableV1Versioned,
    >,
);

/// **Origin**: `Mina_base__Snapp_command.Party.Authorized.Signed.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:323:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L323)
///
/// **Gid**: 1105
pub type MinaBaseSnappCommandPartyAuthorizedSignedStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyAuthorizedSignedStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:367:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L367)
///
/// Gid: 1112
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSnappCommandBinableArgStableV1VersionedV1 {
    ProvedEmpty(
        MinaBaseSnappCommandBinableArgStableV1VersionedV1ProvedEmpty0<
            MinaBaseSnappCommandPartyAuthorizedProvedStableV1Versioned,
            Option<MinaBaseSnappCommandPartyAuthorizedEmptyStableV1Versioned>,
        >,
    ),
    ProvedSigned(
        MinaBaseSnappCommandBinableArgStableV1VersionedV1ProvedEmpty0<
            MinaBaseSnappCommandPartyAuthorizedProvedStableV1Versioned,
            MinaBaseSnappCommandPartyAuthorizedSignedStableV1Versioned,
        >,
    ),
    ProvedProved(
        MinaBaseSnappCommandBinableArgStableV1VersionedV1ProvedEmpty0<
            MinaBaseSnappCommandPartyAuthorizedProvedStableV1Versioned,
            MinaBaseSnappCommandPartyAuthorizedProvedStableV1Versioned,
        >,
    ),
    SignedSigned(
        MinaBaseSnappCommandBinableArgStableV1VersionedV1ProvedEmpty0<
            MinaBaseSnappCommandPartyAuthorizedSignedStableV1Versioned,
            MinaBaseSnappCommandPartyAuthorizedSignedStableV1Versioned,
        >,
    ),
    SignedEmpty(
        MinaBaseSnappCommandBinableArgStableV1VersionedV1ProvedEmpty0<
            MinaBaseSnappCommandPartyAuthorizedSignedStableV1Versioned,
            Option<MinaBaseSnappCommandPartyAuthorizedEmptyStableV1Versioned>,
        >,
    ),
}

/// **Origin**: `Mina_base__Snapp_command.Binable_arg.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:367:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L367)
///
/// **Gid**: 1114
pub type MinaBaseSnappCommandBinableArgStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSnappCommandBinableArgStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/snapp_command.ml:408:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L408)
///
/// Gid: 1115
pub struct MinaBaseSnappCommandStableV1VersionedV1(
    pub MinaBaseSnappCommandBinableArgStableV1Versioned,
);

/// **Origin**: `Mina_base__Snapp_command.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:408:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L408)
///
/// **Gid**: 1116
pub type MinaBaseSnappCommandStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSnappCommandStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/user_command.ml:74:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/user_command.ml#L74)
///
/// Gid: 1144
pub struct MinaBaseUserCommandStableV1VersionedV1(
    pub  MinaBaseUserCommandStableV1VersionedV1Poly<
        MinaBaseSignedCommandStableV1Versioned,
        MinaBaseSnappCommandStableV1Versioned,
    >,
);

/// **Origin**: `Mina_base__User_command.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/user_command.ml:74:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/user_command.ml#L74)
///
/// **Gid**: 1146
pub type MinaBaseUserCommandStableV1Versioned =
    crate::versioned::Versioned<MinaBaseUserCommandStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:136:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L136)
///
/// Gid: 1611
pub struct StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1VersionedV1(
    pub  StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1VersionedV1Poly<
        TransactionSnarkWorkTStableV1Versioned,
        StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1VersionedV1PolyArg1<
            MinaBaseUserCommandStableV1Versioned,
        >,
    >,
);

/// **Origin**: `Staged_ledger_diff.Pre_diff_with_at_most_two_coinbase.Stable.V1.t`
///
/// **Location**: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:136:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L136)
///
/// **Gid**: 1613
pub type StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1Versioned =
    crate::versioned::Versioned<
        StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1VersionedV1,
        1i32,
    >;

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:43:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L43)
///
/// Gid: 1599
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1VersionedV1PolyV1CoinbaseV1<A> {
    Zero,
    One(Option<A>),
}

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:43:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L43)
///
/// Gid: 1601
pub type StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1VersionedV1PolyV1Coinbase<A> =
    crate::versioned::Versioned<
        StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1VersionedV1PolyV1CoinbaseV1<A>,
        1i32,
    >;

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:109:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L109)
///
/// Gid: 1608
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1VersionedV1PolyV1<A, B> {
    pub completed_works: Vec<A>,
    pub commands: Vec<B>,
    pub coinbase: StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1VersionedV1PolyV1Coinbase<
        StagedLedgerDiffFtStableV1Versioned,
    >,
    pub internal_command_balances:
        Vec<MinaBaseTransactionStatusInternalCommandBalanceDataStableV1Versioned>,
}

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:109:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L109)
///
/// Gid: 1610
pub type StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1VersionedV1Poly<A, B> =
    crate::versioned::Versioned<
        StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1VersionedV1PolyV1<A, B>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:155:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L155)
///
/// Gid: 1614
pub struct StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1VersionedV1(
    pub  StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1VersionedV1Poly<
        TransactionSnarkWorkTStableV1Versioned,
        StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1VersionedV1PolyArg1<
            MinaBaseUserCommandStableV1Versioned,
        >,
    >,
);

/// **Origin**: `Staged_ledger_diff.Pre_diff_with_at_most_one_coinbase.Stable.V1.t`
///
/// **Location**: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:155:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L155)
///
/// **Gid**: 1616
pub type StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1Versioned =
    crate::versioned::Versioned<
        StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1VersionedV1,
        1i32,
    >;

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:174:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L174)
///
/// Gid: 1617
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffStableV1VersionedV1(
    pub StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1Versioned,
    pub Option<StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1Versioned>,
);

/// **Origin**: `Staged_ledger_diff.Diff.Stable.V1.t`
///
/// **Location**: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:174:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L174)
///
/// **Gid**: 1619
pub type StagedLedgerDiffDiffStableV1Versioned =
    crate::versioned::Versioned<StagedLedgerDiffDiffStableV1VersionedV1, 1i32>;

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:191:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L191)
///
/// Gid: 1620
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffStableV1VersionedV1 {
    pub diff: StagedLedgerDiffDiffStableV1Versioned,
}

/// **Origin**: `Staged_ledger_diff.Stable.V1.t`
///
/// **Location**: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:191:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L191)
///
/// **Gid**: 1622
pub type StagedLedgerDiffStableV1Versioned =
    crate::versioned::Versioned<StagedLedgerDiffStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/state_body_hash.ml:20:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/state_body_hash.ml#L20)
///
/// Gid: 974
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBlockExternalTransitionRawVersionedStableV1VersionedV1DeltaTransitionChainProofArg0V1Poly(
    pub crate::bigint::BigInt,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/state_body_hash.ml:19:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/state_body_hash.ml#L19)
///
/// Gid: 977
pub struct MinaBlockExternalTransitionRawVersionedStableV1VersionedV1DeltaTransitionChainProofArg0V1 (pub MinaBlockExternalTransitionRawVersionedStableV1VersionedV1DeltaTransitionChainProofArg0V1Poly) ;

/// Location: [src/lib/mina_base/state_body_hash.ml:19:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/state_body_hash.ml#L19)
///
/// Gid: 978
pub type MinaBlockExternalTransitionRawVersionedStableV1VersionedV1DeltaTransitionChainProofArg0 =
    crate::versioned::Versioned<
        MinaBlockExternalTransitionRawVersionedStableV1VersionedV1DeltaTransitionChainProofArg0V1,
        1i32,
    >;

/// Location: [src/lib/protocol_version/protocol_version.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/protocol_version/protocol_version.ml#L8)
///
/// Gid: 1623
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ProtocolVersionStableV1VersionedV1 {
    pub major: crate::number::Int32,
    pub minor: crate::number::Int32,
    pub patch: crate::number::Int32,
}

/// **Origin**: `Protocol_version.Stable.V1.t`
///
/// **Location**: [src/lib/protocol_version/protocol_version.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/protocol_version/protocol_version.ml#L8)
///
/// **Gid**: 1625
pub type ProtocolVersionStableV1Versioned =
    crate::versioned::Versioned<ProtocolVersionStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_block/external_transition.ml:31:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_block/external_transition.ml#L31)
///
/// Gid: 1645
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBlockExternalTransitionRawVersionedStableV1VersionedV1 {
    pub protocol_state: MinaStateProtocolStateValueStableV1Versioned,
    pub protocol_state_proof: MinaBaseProofStableV1Versioned,
    pub staged_ledger_diff: StagedLedgerDiffStableV1Versioned,
    pub delta_transition_chain_proof: (
        MinaStateProtocolStateValueStableV1VersionedV1PolyArg0,
        Vec<
            MinaBlockExternalTransitionRawVersionedStableV1VersionedV1DeltaTransitionChainProofArg0,
        >,
    ),
    pub current_protocol_version: ProtocolVersionStableV1Versioned,
    pub proposed_protocol_version_opt: Option<ProtocolVersionStableV1Versioned>,
    pub validation_callback: (),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/network_pool/transaction_pool.ml:45:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_pool/transaction_pool.ml#L45)
///
/// Gid: 1727
pub struct NetworkPoolTransactionPoolDiffVersionedStableV1VersionedV1(
    pub Vec<MinaBaseUserCommandStableV1Versioned>,
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/transaction_snark/transaction_snark.ml:202:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L202)
///
/// Gid: 1481
pub struct TransactionSnarkStatementStableV1VersionedV1(
    pub  TransactionSnarkStatementWithSokStableV1VersionedV1Poly<
        MinaBaseStagedLedgerHashNonSnarkStableV1VersionedV1LedgerHash,
        CurrencyAmountMakeStrStableV1Versioned,
        TransactionSnarkPendingCoinbaseStackStateStableV1Versioned,
        MinaBaseFeeExcessStableV1Versioned,
        MinaBaseTokenIdStableV1Versioned,
        (),
    >,
);

/// **Origin**: `Transaction_snark.Statement.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark/transaction_snark.ml:202:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L202)
///
/// **Gid**: 1483
pub type TransactionSnarkStatementStableV1Versioned =
    crate::versioned::Versioned<TransactionSnarkStatementStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/transaction_snark_work/transaction_snark_work.ml:23:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_work/transaction_snark_work.ml#L23)
///
/// Gid: 1505
pub struct TransactionSnarkWorkStatementStableV1VersionedV1(
    pub TransactionSnarkWorkTStableV1VersionedV1Proofs<TransactionSnarkStatementStableV1Versioned>,
);

/// **Origin**: `Transaction_snark_work.Statement.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark_work/transaction_snark_work.ml:23:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_work/transaction_snark_work.ml#L23)
///
/// **Gid**: 1509
pub type TransactionSnarkWorkStatementStableV1Versioned =
    crate::versioned::Versioned<TransactionSnarkWorkStatementStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/fee_with_prover.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_with_prover.ml#L7)
///
/// Gid: 1329
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeWithProverStableV1VersionedV1 {
    pub fee: CurrencyFeeStableV1Versioned,
    pub prover: ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8,
}

/// **Origin**: `Mina_base__Fee_with_prover.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/fee_with_prover.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_with_prover.ml#L7)
///
/// **Gid**: 1331
pub type MinaBaseFeeWithProverStableV1Versioned =
    crate::versioned::Versioned<MinaBaseFeeWithProverStableV1VersionedV1, 1i32>;

/// Location: [src/lib/network_pool/priced_proof.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_pool/priced_proof.ml#L9)
///
/// Gid: 1724
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NetworkPoolSnarkPoolDiffVersionedStableV1VersionedV1AddSolvedWork1V1<Proof> {
    pub proof: Proof,
    pub fee: MinaBaseFeeWithProverStableV1Versioned,
}

/// Location: [src/lib/network_pool/priced_proof.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_pool/priced_proof.ml#L9)
///
/// Gid: 1726
pub type NetworkPoolSnarkPoolDiffVersionedStableV1VersionedV1AddSolvedWork1<Proof> =
    crate::versioned::Versioned<
        NetworkPoolSnarkPoolDiffVersionedStableV1VersionedV1AddSolvedWork1V1<Proof>,
        1i32,
    >;

/// Location: [src/lib/network_pool/snark_pool.ml:705:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_pool/snark_pool.ml#L705)
///
/// Gid: 1743
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum NetworkPoolSnarkPoolDiffVersionedStableV1VersionedV1 {
    AddSolvedWork(
        TransactionSnarkWorkStatementStableV1Versioned,
        NetworkPoolSnarkPoolDiffVersionedStableV1VersionedV1AddSolvedWork1<
            TransactionSnarkWorkTStableV1VersionedV1Proofs<LedgerProofProdStableV1Versioned>,
        >,
    ),
    Empty,
}

/// Location: [src/lib/mina_base/account.ml:89:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account.ml#L89)
///
/// Gid: 995
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountBinableArgStableV1VersionedV1PolyV1<
    Pk,
    Tid,
    TokenPermissions,
    Amount,
    Nonce,
    ReceiptChainHash,
    Delegate,
    StateHash,
    Timing,
    Permissions,
    SnappOpt,
> {
    pub public_key: Pk,
    pub token_id: Tid,
    pub token_permissions: TokenPermissions,
    pub balance: Amount,
    pub nonce: Nonce,
    pub receipt_chain_hash: ReceiptChainHash,
    pub delegate: Delegate,
    pub voting_for: StateHash,
    pub timing: Timing,
    pub permissions: Permissions,
    pub snapp: SnappOpt,
}

/// Location: [src/lib/mina_base/account.ml:89:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account.ml#L89)
///
/// Gid: 997
pub type MinaBaseAccountBinableArgStableV1VersionedV1Poly<
    Pk,
    Tid,
    TokenPermissions,
    Amount,
    Nonce,
    ReceiptChainHash,
    Delegate,
    StateHash,
    Timing,
    Permissions,
    SnappOpt,
> = crate::versioned::Versioned<
    MinaBaseAccountBinableArgStableV1VersionedV1PolyV1<
        Pk,
        Tid,
        TokenPermissions,
        Amount,
        Nonce,
        ReceiptChainHash,
        Delegate,
        StateHash,
        Timing,
        Permissions,
        SnappOpt,
    >,
    1i32,
>;

/// Location: [src/lib/mina_base/token_permissions.ml:14:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/token_permissions.ml#L14)
///
/// Gid: 985
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseTokenPermissionsStableV1VersionedV1 {
    TokenOwned { disable_new_accounts: bool },
    NotOwned { account_disabled: bool },
}

/// **Origin**: `Mina_base__Token_permissions.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/token_permissions.ml:14:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/token_permissions.ml#L14)
///
/// **Gid**: 987
pub type MinaBaseTokenPermissionsStableV1Versioned =
    crate::versioned::Versioned<MinaBaseTokenPermissionsStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/account_timing.ml:19:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account_timing.ml#L19)
///
/// Gid: 854
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountTimingStableV1VersionedV1PolyV1<Slot, Balance, Amount> {
    Untimed,
    Timed {
        initial_minimum_balance: Balance,
        cliff_time: Slot,
        cliff_amount: Amount,
        vesting_period: Slot,
        vesting_increment: Amount,
    },
}

/// Location: [src/lib/mina_base/account_timing.ml:19:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account_timing.ml#L19)
///
/// Gid: 856
pub type MinaBaseAccountTimingStableV1VersionedV1Poly<Slot, Balance, Amount> =
    crate::versioned::Versioned<
        MinaBaseAccountTimingStableV1VersionedV1PolyV1<Slot, Balance, Amount>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/account_timing.ml:36:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account_timing.ml#L36)
///
/// Gid: 857
pub struct MinaBaseAccountTimingStableV1VersionedV1(
    pub  MinaBaseAccountTimingStableV1VersionedV1Poly<
        ConsensusGlobalSlotStableV1VersionedV1PolyArg0,
        CurrencyBalanceStableV1Versioned,
        CurrencyAmountMakeStrStableV1Versioned,
    >,
);

/// **Origin**: `Mina_base__Account_timing.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/account_timing.ml:36:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account_timing.ml#L36)
///
/// **Gid**: 859
pub type MinaBaseAccountTimingStableV1Versioned =
    crate::versioned::Versioned<MinaBaseAccountTimingStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_account.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_account.ml#L17)
///
/// Gid: 966
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappAccountStableV1VersionedV1PolyV1<AppState, Vk> {
    pub app_state: AppState,
    pub verification_key: Vk,
}

/// Location: [src/lib/mina_base/snapp_account.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_account.ml#L17)
///
/// Gid: 968
pub type MinaBaseSnappAccountStableV1VersionedV1Poly<AppState, Vk> =
    crate::versioned::Versioned<MinaBaseSnappAccountStableV1VersionedV1PolyV1<AppState, Vk>, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/snapp_state.ml:50:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_state.ml#L50)
///
/// Gid: 963
pub struct MinaBaseSnappStateValueStableV1VersionedV1(
    pub MinaBaseSnappCommandPartyUpdateStableV1VersionedV1PolyV1AppState<crate::bigint::BigInt>,
);

/// **Origin**: `Mina_base__Snapp_state.Value.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_state.ml:50:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_state.ml#L50)
///
/// **Gid**: 965
pub type MinaBaseSnappStateValueStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSnappStateValueStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/snapp_account.ml:30:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_account.ml#L30)
///
/// Gid: 969
pub struct MinaBaseSnappAccountStableV1VersionedV1(
    pub  MinaBaseSnappAccountStableV1VersionedV1Poly<
        MinaBaseSnappStateValueStableV1Versioned,
        Option<
            MinaBaseSnappCommandPartyUpdateStableV1VersionedV1PolyArg0Arg0<
                PicklesSideLoadedVerificationKeyStableV1Versioned,
                crate::bigint::BigInt,
            >,
        >,
    >,
);

/// **Origin**: `Mina_base__Snapp_account.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_account.ml:30:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_account.ml#L30)
///
/// **Gid**: 971
pub type MinaBaseSnappAccountStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSnappAccountStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/account.ml:140:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account.ml#L140)
///
/// Gid: 1001
pub struct MinaBaseAccountBinableArgStableV1VersionedV1(
    pub  MinaBaseAccountBinableArgStableV1VersionedV1Poly<
        ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8,
        MinaBaseTokenIdStableV1Versioned,
        MinaBaseTokenPermissionsStableV1Versioned,
        CurrencyBalanceStableV1Versioned,
        MinaNumbersNatMake32StableV1Versioned,
        MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1PolyArg0Dup1,
        Option<ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8>,
        MinaStateProtocolStateValueStableV1VersionedV1PolyArg0,
        MinaBaseAccountTimingStableV1Versioned,
        MinaBasePermissionsStableV1Versioned,
        Option<MinaBaseSnappAccountStableV1Versioned>,
    >,
);

/// **Origin**: `Mina_base__Account.Binable_arg.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/account.ml:140:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account.ml#L140)
///
/// **Gid**: 1003
pub type MinaBaseAccountBinableArgStableV1Versioned =
    crate::versioned::Versioned<MinaBaseAccountBinableArgStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/account.ml:188:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account.ml#L188)
///
/// Gid: 1004
pub struct MinaBaseAccountStableV1VersionedV1(pub MinaBaseAccountBinableArgStableV1Versioned);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/network_peer/peer.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_peer/peer.ml#L10)
///
/// Gid: 778
pub struct NetworkPeerPeerIdStableV1VersionedV1(pub crate::string::ByteString);

/// **Origin**: `Network_peer__Peer.Id.Stable.V1.t`
///
/// **Location**: [src/lib/network_peer/peer.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_peer/peer.ml#L10)
///
/// **Gid**: 780
pub type NetworkPeerPeerIdStableV1Versioned =
    crate::versioned::Versioned<NetworkPeerPeerIdStableV1VersionedV1, 1i32>;

/// Location: [src/lib/network_peer/peer.ml:28:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_peer/peer.ml#L28)
///
/// Gid: 781
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NetworkPeerPeerStableV1VersionedV1 {
    pub host: crate::string::ByteString,
    pub libp2p_port: crate::number::Int32,
    pub peer_id: NetworkPeerPeerIdStableV1Versioned,
}

/// Location: [src/lib/non_empty_list/non_empty_list.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_empty_list/non_empty_list.ml#L7)
///
/// Gid: 772
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesV1<A>(pub A, pub Vec<A>);

/// Location: [src/lib/non_empty_list/non_empty_list.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_empty_list/non_empty_list.ml#L7)
///
/// Gid: 774
pub type TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1Trees<A> =
    crate::versioned::Versioned<
        TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesV1<A>,
        1i32,
    >;

//  The type `TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0V1` is skipped
//
//  Location: [src/lib/parallel_scan/parallel_scan.ml:226:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L226)
//
//  Gid: 1564

/// Location: [src/lib/parallel_scan/parallel_scan.ml:226:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L226)
///
/// Gid: 1566
pub type TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0<MergeT, BaseT> =
    crate::versioned::Versioned<
        TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0V1<MergeT, BaseT>,
        1i32,
    >;

/// Location: [src/lib/parallel_scan/parallel_scan.ml:53:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L53)
///
/// Gid: 1531
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ParallelScanWeightStableV1VersionedV1 {
    pub base: crate::number::Int32,
    pub merge: crate::number::Int32,
}

/// **Origin**: `Parallel_scan.Weight.Stable.V1.t`
///
/// **Location**: [src/lib/parallel_scan/parallel_scan.ml:53:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L53)
///
/// **Gid**: 1533
pub type ParallelScanWeightStableV1Versioned =
    crate::versioned::Versioned<ParallelScanWeightStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/parallel_scan/parallel_scan.ml:22:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L22)
///
/// Gid: 1525
pub struct ParallelScanSequenceNumberStableV1VersionedV1(pub crate::number::Int32);

/// **Origin**: `Parallel_scan.Sequence_number.Stable.V1.t`
///
/// **Location**: [src/lib/parallel_scan/parallel_scan.ml:22:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L22)
///
/// **Gid**: 1527
pub type ParallelScanSequenceNumberStableV1Versioned =
    crate::versioned::Versioned<ParallelScanSequenceNumberStableV1VersionedV1, 1i32>;

/// Location: [src/lib/parallel_scan/parallel_scan.ml:35:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L35)
///
/// Gid: 1528
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum ParallelScanJobStatusStableV1VersionedV1 {
    Todo,
    Done,
}

/// **Origin**: `Parallel_scan.Job_status.Stable.V1.t`
///
/// **Location**: [src/lib/parallel_scan/parallel_scan.ml:35:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L35)
///
/// **Gid**: 1530
pub type ParallelScanJobStatusStableV1Versioned =
    crate::versioned::Versioned<ParallelScanJobStatusStableV1VersionedV1, 1i32>;

/// Location: [src/lib/parallel_scan/parallel_scan.ml:105:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L105)
///
/// Gid: 1543
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0Arg0V11V1Full0V1<Merge>
{
    pub left: Merge,
    pub right: Merge,
    pub seq_no: ParallelScanSequenceNumberStableV1Versioned,
    pub status: ParallelScanJobStatusStableV1Versioned,
}

/// Location: [src/lib/parallel_scan/parallel_scan.ml:105:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L105)
///
/// Gid: 1545
pub type TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0Arg0V11V1Full0<Merge> =
    crate::versioned::Versioned<
        TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0Arg0V11V1Full0V1<Merge>,
        1i32,
    >;

/// Location: [src/lib/parallel_scan/parallel_scan.ml:120:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L120)
///
/// Gid: 1546
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0Arg0V11V1<Merge> {
    Empty,
    Part(Merge),
    Full(TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0Arg0V11V1Full0<Merge>),
}

/// Location: [src/lib/parallel_scan/parallel_scan.ml:120:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L120)
///
/// Gid: 1548
pub type TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0Arg0V11<Merge> =
    crate::versioned::Versioned<
        TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0Arg0V11V1<Merge>,
        1i32,
    >;

/// Location: [src/lib/parallel_scan/parallel_scan.ml:140:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L140)
///
/// Gid: 1549
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0Arg0V1<Merge>(
    pub  (
        ParallelScanWeightStableV1Versioned,
        ParallelScanWeightStableV1Versioned,
    ),
    pub TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0Arg0V11<Merge>,
);

/// Location: [src/lib/parallel_scan/parallel_scan.ml:140:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L140)
///
/// Gid: 1551
pub type TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0Arg0<Merge> =
    crate::versioned::Versioned<
        TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0Arg0V1<Merge>,
        1i32,
    >;

/// Location: [src/lib/parallel_scan/parallel_scan.ml:68:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L68)
///
/// Gid: 1534
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0Arg1V11V1Full0V1<Base> {
    pub job: Base,
    pub seq_no: ParallelScanSequenceNumberStableV1Versioned,
    pub status: ParallelScanJobStatusStableV1Versioned,
}

/// Location: [src/lib/parallel_scan/parallel_scan.ml:68:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L68)
///
/// Gid: 1536
pub type TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0Arg1V11V1Full0<Base> =
    crate::versioned::Versioned<
        TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0Arg1V11V1Full0V1<Base>,
        1i32,
    >;

/// Location: [src/lib/parallel_scan/parallel_scan.ml:82:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L82)
///
/// Gid: 1537
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0Arg1V11V1<Base> {
    Empty,
    Full(TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0Arg1V11V1Full0<Base>),
}

/// Location: [src/lib/parallel_scan/parallel_scan.ml:82:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L82)
///
/// Gid: 1539
pub type TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0Arg1V11<Base> =
    crate::versioned::Versioned<
        TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0Arg1V11V1<Base>,
        1i32,
    >;

/// Location: [src/lib/parallel_scan/parallel_scan.ml:93:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L93)
///
/// Gid: 1540
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0Arg1V1<Base>(
    pub ParallelScanWeightStableV1Versioned,
    pub TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0Arg1V11<Base>,
);

/// Location: [src/lib/parallel_scan/parallel_scan.ml:93:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L93)
///
/// Gid: 1542
pub type TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0Arg1<Base> =
    crate::versioned::Versioned<
        TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0Arg1V1<Base>,
        1i32,
    >;

/// Location: [src/lib/parallel_scan/parallel_scan.ml:782:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L782)
///
/// Gid: 1567
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1<Merge, Base> {
    pub trees: TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1Trees<
        TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0<
            TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0Arg0<Merge>,
            TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0Arg1<Base>,
        >,
    >,
    pub acc: Option<(Merge, Vec<Base>)>,
    pub curr_job_seq_no: crate::number::Int32,
    pub max_base_jobs: crate::number::Int32,
    pub delay: crate::number::Int32,
}

/// Location: [src/lib/parallel_scan/parallel_scan.ml:782:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L782)
///
/// Gid: 1569
pub type TransactionSnarkScanStateStableV1VersionedV1PolyV1Poly<Merge, Base> =
    crate::versioned::Versioned<
        TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1<Merge, Base>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/parallel_scan/parallel_scan.ml:800:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L800)
///
/// Gid: 1570
pub struct TransactionSnarkScanStateStableV1VersionedV1PolyV1<Merge, Base>(
    pub TransactionSnarkScanStateStableV1VersionedV1PolyV1Poly<Merge, Base>,
);

/// Location: [src/lib/parallel_scan/parallel_scan.ml:800:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L800)
///
/// Gid: 1571
pub type TransactionSnarkScanStateStableV1VersionedV1Poly<Merge, Base> =
    crate::versioned::Versioned<
        TransactionSnarkScanStateStableV1VersionedV1PolyV1<Merge, Base>,
        1i32,
    >;

/// Location: [src/lib/mina_base/sok_message.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/sok_message.ml#L7)
///
/// Gid: 1309
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSokMessageStableV1VersionedV1 {
    pub fee: CurrencyFeeStableV1Versioned,
    pub prover: ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8,
}

/// **Origin**: `Mina_base__Sok_message.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/sok_message.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/sok_message.ml#L7)
///
/// **Gid**: 1311
pub type MinaBaseSokMessageStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSokMessageStableV1VersionedV1, 1i32>;

/// Location: [src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml:63:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml#L63)
///
/// Gid: 1575
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateLedgerProofWithSokMessageStableV1VersionedV1(
    pub LedgerProofProdStableV1Versioned,
    pub MinaBaseSokMessageStableV1Versioned,
);

/// **Origin**: `Transaction_snark_scan_state.Ledger_proof_with_sok_message.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml:63:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml#L63)
///
/// **Gid**: 1577
pub type TransactionSnarkScanStateLedgerProofWithSokMessageStableV1Versioned =
    crate::versioned::Versioned<
        TransactionSnarkScanStateLedgerProofWithSokMessageStableV1VersionedV1,
        1i32,
    >;

/// Location: [src/lib/mina_base/transaction_logic.ml:44:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_logic.ml#L44)
///
/// Gid: 1174
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV1VersionedV1 {
    pub user_command: StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1VersionedV1PolyArg1<
        MinaBaseSignedCommandStableV1Versioned,
    >,
    pub previous_receipt_chain_hash:
        MinaBaseSnappPredicateAccountStableV1VersionedV1PolyArg0V1PolyArg0Dup1,
    pub fee_payer_timing: MinaBaseAccountTimingStableV1Versioned,
    pub source_timing: Option<MinaBaseAccountTimingStableV1Versioned>,
}

/// **Origin**: `Mina_base__Transaction_logic.Transaction_applied.Signed_command_applied.Common.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_logic.ml:44:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_logic.ml#L44)
///
/// **Gid**: 1176
pub type MinaBaseTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV1Versioned =
    crate::versioned::Versioned<
        MinaBaseTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV1VersionedV1,
        1i32,
    >;

/// Location: [src/lib/mina_base/account_id.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account_id.ml#L9)
///
/// Gid: 847
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountIdStableV1VersionedV1(
    pub ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8,
    pub MinaBaseTokenIdStableV1Versioned,
);

/// **Origin**: `Mina_base__Account_id.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/account_id.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account_id.ml#L9)
///
/// **Gid**: 849
pub type MinaBaseAccountIdStableV1Versioned =
    crate::versioned::Versioned<MinaBaseAccountIdStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/transaction_logic.ml:61:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_logic.ml#L61)
///
/// Gid: 1177
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseTransactionLogicTransactionAppliedSignedCommandAppliedBodyStableV1VersionedV1 {
    Payment {
        previous_empty_accounts: Vec<MinaBaseAccountIdStableV1Versioned>,
    },
    StakeDelegation {
        previous_delegate:
            Option<ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8>,
    },
    CreateNewToken {
        created_token: MinaBaseTokenIdStableV1Versioned,
    },
    CreateTokenAccount,
    MintTokens,
    Failed,
}

/// **Origin**: `Mina_base__Transaction_logic.Transaction_applied.Signed_command_applied.Body.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_logic.ml:61:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_logic.ml#L61)
///
/// **Gid**: 1179
pub type MinaBaseTransactionLogicTransactionAppliedSignedCommandAppliedBodyStableV1Versioned =
    crate::versioned::Versioned<
        MinaBaseTransactionLogicTransactionAppliedSignedCommandAppliedBodyStableV1VersionedV1,
        1i32,
    >;

/// Location: [src/lib/mina_base/transaction_logic.ml:80:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_logic.ml#L80)
///
/// Gid: 1180
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseTransactionLogicTransactionAppliedSignedCommandAppliedStableV1VersionedV1 {
    pub common:
        MinaBaseTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV1Versioned,
    pub body: MinaBaseTransactionLogicTransactionAppliedSignedCommandAppliedBodyStableV1Versioned,
}

/// **Origin**: `Mina_base__Transaction_logic.Transaction_applied.Signed_command_applied.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_logic.ml:80:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_logic.ml#L80)
///
/// **Gid**: 1182
pub type MinaBaseTransactionLogicTransactionAppliedSignedCommandAppliedStableV1Versioned =
    crate::versioned::Versioned<
        MinaBaseTransactionLogicTransactionAppliedSignedCommandAppliedStableV1VersionedV1,
        1i32,
    >;

/// Location: [src/lib/mina_base/transaction_logic.ml:92:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_logic.ml#L92)
///
/// Gid: 1183
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseTransactionLogicTransactionAppliedSnappCommandAppliedStableV1VersionedV1 {
    pub accounts: Vec<(
        MinaBaseAccountIdStableV1Versioned,
        Option<MinaBaseAccountStableV1Versioned>,
    )>,
    pub command: StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1VersionedV1PolyArg1<
        MinaBaseSnappCommandStableV1Versioned,
    >,
}

/// **Origin**: `Mina_base__Transaction_logic.Transaction_applied.Snapp_command_applied.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_logic.ml:92:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_logic.ml#L92)
///
/// **Gid**: 1185
pub type MinaBaseTransactionLogicTransactionAppliedSnappCommandAppliedStableV1Versioned =
    crate::versioned::Versioned<
        MinaBaseTransactionLogicTransactionAppliedSnappCommandAppliedStableV1VersionedV1,
        1i32,
    >;

/// Location: [src/lib/mina_base/transaction_logic.ml:108:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_logic.ml#L108)
///
/// Gid: 1186
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseTransactionLogicTransactionAppliedCommandAppliedStableV1VersionedV1 {
    SignedCommand(MinaBaseTransactionLogicTransactionAppliedSignedCommandAppliedStableV1Versioned),
    SnappCommand(MinaBaseTransactionLogicTransactionAppliedSnappCommandAppliedStableV1Versioned),
}

/// **Origin**: `Mina_base__Transaction_logic.Transaction_applied.Command_applied.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_logic.ml:108:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_logic.ml#L108)
///
/// **Gid**: 1188
pub type MinaBaseTransactionLogicTransactionAppliedCommandAppliedStableV1Versioned =
    crate::versioned::Versioned<
        MinaBaseTransactionLogicTransactionAppliedCommandAppliedStableV1VersionedV1,
        1i32,
    >;

/// Location: [src/lib/mina_base/fee_transfer.ml:8:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_transfer.ml#L8)
///
/// Gid: 1153
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeTransferSingleStableV1VersionedV1 {
    pub receiver_pk: ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8,
    pub fee: CurrencyFeeStableV1Versioned,
    pub fee_token: MinaBaseTokenIdStableV1Versioned,
}

/// **Origin**: `Mina_base__Fee_transfer.Single.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/fee_transfer.ml:8:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_transfer.ml#L8)
///
/// **Gid**: 1155
pub type MinaBaseFeeTransferSingleStableV1Versioned =
    crate::versioned::Versioned<MinaBaseFeeTransferSingleStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/fee_transfer.ml:57:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_transfer.ml#L57)
///
/// Gid: 1156
pub struct MinaBaseFeeTransferStableV1VersionedV1(
    pub TransactionSnarkWorkTStableV1VersionedV1Proofs<MinaBaseFeeTransferSingleStableV1Versioned>,
);

/// **Origin**: `Mina_base__Fee_transfer.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/fee_transfer.ml:57:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_transfer.ml#L57)
///
/// **Gid**: 1158
pub type MinaBaseFeeTransferStableV1Versioned =
    crate::versioned::Versioned<MinaBaseFeeTransferStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/transaction_logic.ml:122:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_logic.ml#L122)
///
/// Gid: 1189
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseTransactionLogicTransactionAppliedFeeTransferAppliedStableV1VersionedV1 {
    pub fee_transfer: MinaBaseFeeTransferStableV1Versioned,
    pub previous_empty_accounts: Vec<MinaBaseAccountIdStableV1Versioned>,
    pub receiver_timing: MinaBaseAccountTimingStableV1Versioned,
    pub balances: MinaBaseTransactionStatusFeeTransferBalanceDataStableV1Versioned,
}

/// **Origin**: `Mina_base__Transaction_logic.Transaction_applied.Fee_transfer_applied.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_logic.ml:122:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_logic.ml#L122)
///
/// **Gid**: 1191
pub type MinaBaseTransactionLogicTransactionAppliedFeeTransferAppliedStableV1Versioned =
    crate::versioned::Versioned<
        MinaBaseTransactionLogicTransactionAppliedFeeTransferAppliedStableV1VersionedV1,
        1i32,
    >;

/// Location: [src/lib/mina_base/coinbase.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/coinbase.ml#L8)
///
/// Gid: 1162
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseCoinbaseStableV1VersionedV1 {
    pub receiver: ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg8,
    pub amount: CurrencyAmountMakeStrStableV1Versioned,
    pub fee_transfer: Option<MinaBaseCoinbaseFeeTransferStableV1Versioned>,
}

/// **Origin**: `Mina_base__Coinbase.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/coinbase.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/coinbase.ml#L8)
///
/// **Gid**: 1164
pub type MinaBaseCoinbaseStableV1Versioned =
    crate::versioned::Versioned<MinaBaseCoinbaseStableV1VersionedV1, 1i32>;

/// Location: [src/lib/mina_base/transaction_logic.ml:139:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_logic.ml#L139)
///
/// Gid: 1192
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseTransactionLogicTransactionAppliedCoinbaseAppliedStableV1VersionedV1 {
    pub coinbase: MinaBaseCoinbaseStableV1Versioned,
    pub previous_empty_accounts: Vec<MinaBaseAccountIdStableV1Versioned>,
    pub receiver_timing: MinaBaseAccountTimingStableV1Versioned,
    pub balances: MinaBaseTransactionStatusCoinbaseBalanceDataStableV1Versioned,
}

/// **Origin**: `Mina_base__Transaction_logic.Transaction_applied.Coinbase_applied.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_logic.ml:139:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_logic.ml#L139)
///
/// **Gid**: 1194
pub type MinaBaseTransactionLogicTransactionAppliedCoinbaseAppliedStableV1Versioned =
    crate::versioned::Versioned<
        MinaBaseTransactionLogicTransactionAppliedCoinbaseAppliedStableV1VersionedV1,
        1i32,
    >;

/// Location: [src/lib/mina_base/transaction_logic.ml:156:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_logic.ml#L156)
///
/// Gid: 1195
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseTransactionLogicTransactionAppliedVaryingStableV1VersionedV1 {
    Command(MinaBaseTransactionLogicTransactionAppliedCommandAppliedStableV1Versioned),
    FeeTransfer(MinaBaseTransactionLogicTransactionAppliedFeeTransferAppliedStableV1Versioned),
    Coinbase(MinaBaseTransactionLogicTransactionAppliedCoinbaseAppliedStableV1Versioned),
}

/// **Origin**: `Mina_base__Transaction_logic.Transaction_applied.Varying.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_logic.ml:156:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_logic.ml#L156)
///
/// **Gid**: 1197
pub type MinaBaseTransactionLogicTransactionAppliedVaryingStableV1Versioned =
    crate::versioned::Versioned<
        MinaBaseTransactionLogicTransactionAppliedVaryingStableV1VersionedV1,
        1i32,
    >;

/// Location: [src/lib/mina_base/transaction_logic.ml:170:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_logic.ml#L170)
///
/// Gid: 1198
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseTransactionLogicTransactionAppliedStableV1VersionedV1 {
    pub previous_hash: MinaBaseStagedLedgerHashNonSnarkStableV1VersionedV1LedgerHash,
    pub varying: MinaBaseTransactionLogicTransactionAppliedVaryingStableV1Versioned,
}

/// **Origin**: `Mina_base__Transaction_logic.Transaction_applied.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_logic.ml:170:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_logic.ml#L170)
///
/// **Gid**: 1200
pub type MinaBaseTransactionLogicTransactionAppliedStableV1Versioned = crate::versioned::Versioned<
    MinaBaseTransactionLogicTransactionAppliedStableV1VersionedV1,
    1i32,
>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/snapp_predicate.ml:699:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L699)
///
/// Gid: 1055
pub struct MinaBaseSnappPredicateProtocolStateViewStableV1VersionedV1(
    pub  MinaBaseSnappPredicateProtocolStateStableV1VersionedV1Poly<
        MinaBaseStagedLedgerHashNonSnarkStableV1VersionedV1LedgerHash,
        MinaBaseTokenIdStableV1Versioned,
        BlockTimeTimeStableV1Versioned,
        ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg0,
        (),
        ConsensusGlobalSlotStableV1VersionedV1PolyArg0,
        CurrencyAmountMakeStrStableV1Versioned,
        ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1VersionedV1Poly<
            MinaBaseEpochLedgerValueStableV1VersionedV1Poly<
                MinaBaseStagedLedgerHashNonSnarkStableV1VersionedV1LedgerHash,
                CurrencyAmountMakeStrStableV1Versioned,
            >,
            ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1VersionedV1PolyArg1,
            MinaStateProtocolStateValueStableV1VersionedV1PolyArg0,
            MinaStateProtocolStateValueStableV1VersionedV1PolyArg0,
            ConsensusProofOfStakeDataConsensusStateValueStableV1VersionedV1PolyArg0,
        >,
    >,
);

/// **Origin**: `Mina_base__Snapp_predicate.Protocol_state.View.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_predicate.ml:699:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L699)
///
/// **Gid**: 1057
pub type MinaBaseSnappPredicateProtocolStateViewStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSnappPredicateProtocolStateViewStableV1VersionedV1, 1i32>;

/// Location: [src/lib/transaction_snark/transaction_snark.ml:51:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L51)
///
/// Gid: 1467
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum TransactionSnarkPendingCoinbaseStackStateInitStackStableV1VersionedV1 {
    Base(MinaBasePendingCoinbaseStackVersionedStableV1Versioned),
    Merge,
}

/// **Origin**: `Transaction_snark.Pending_coinbase_stack_state.Init_stack.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark/transaction_snark.ml:51:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L51)
///
/// **Gid**: 1469
pub type TransactionSnarkPendingCoinbaseStackStateInitStackStableV1Versioned =
    crate::versioned::Versioned<
        TransactionSnarkPendingCoinbaseStackStateInitStackStableV1VersionedV1,
        1i32,
    >;

/// Location: [src/lib/sparse_ledger_lib/sparse_ledger.ml:9:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/sparse_ledger_lib/sparse_ledger.ml#L9)
///
/// Gid: 763
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSparseLedgerStableV1VersionedV1PolyV1TreeV1<Hash, Account> {
    Account(Account),
    Hash(Hash),
    Node(
        Hash,
        Box<MinaBaseSparseLedgerStableV1VersionedV1PolyV1TreeV1<Hash, Account>>,
        Box<MinaBaseSparseLedgerStableV1VersionedV1PolyV1TreeV1<Hash, Account>>,
    ),
}

/// Location: [src/lib/sparse_ledger_lib/sparse_ledger.ml:9:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/sparse_ledger_lib/sparse_ledger.ml#L9)
///
/// Gid: 765
pub type MinaBaseSparseLedgerStableV1VersionedV1PolyV1Tree<Hash, Account> =
    crate::versioned::Versioned<
        MinaBaseSparseLedgerStableV1VersionedV1PolyV1TreeV1<Hash, Account>,
        1i32,
    >;

/// Location: [src/lib/sparse_ledger_lib/sparse_ledger.ml:30:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/sparse_ledger_lib/sparse_ledger.ml#L30)
///
/// Gid: 766
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSparseLedgerStableV1VersionedV1PolyV1<Hash, Key, Account, TokenId> {
    pub indexes: Vec<(Key, crate::number::Int32)>,
    pub depth: crate::number::Int32,
    pub tree: MinaBaseSparseLedgerStableV1VersionedV1PolyV1Tree<Hash, Account>,
    pub next_available_token: TokenId,
}

/// Location: [src/lib/sparse_ledger_lib/sparse_ledger.ml:30:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/sparse_ledger_lib/sparse_ledger.ml#L30)
///
/// Gid: 768
pub type MinaBaseSparseLedgerStableV1VersionedV1Poly<Hash, Key, Account, TokenId> =
    crate::versioned::Versioned<
        MinaBaseSparseLedgerStableV1VersionedV1PolyV1<Hash, Key, Account, TokenId>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/sparse_ledger.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/sparse_ledger.ml#L8)
///
/// Gid: 1306
pub struct MinaBaseSparseLedgerStableV1VersionedV1(
    pub  MinaBaseSparseLedgerStableV1VersionedV1Poly<
        MinaBaseStagedLedgerHashNonSnarkStableV1VersionedV1LedgerHash,
        MinaBaseAccountIdStableV1Versioned,
        MinaBaseAccountStableV1Versioned,
        MinaBaseTokenIdStableV1Versioned,
    >,
);

/// **Origin**: `Mina_base__Sparse_ledger.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/sparse_ledger.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/sparse_ledger.ml#L8)
///
/// **Gid**: 1308
pub type MinaBaseSparseLedgerStableV1Versioned =
    crate::versioned::Versioned<MinaBaseSparseLedgerStableV1VersionedV1, 1i32>;

/// Location: [src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml:40:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml#L40)
///
/// Gid: 1572
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateTransactionWithWitnessStableV1VersionedV1 {
    pub transaction_with_info: MinaBaseTransactionLogicTransactionAppliedStableV1Versioned,
    pub state_hash: (
        MinaStateProtocolStateValueStableV1VersionedV1PolyArg0,
        MinaBlockExternalTransitionRawVersionedStableV1VersionedV1DeltaTransitionChainProofArg0,
    ),
    pub state_view: MinaBaseSnappPredicateProtocolStateViewStableV1Versioned,
    pub statement: TransactionSnarkStatementStableV1Versioned,
    pub init_stack: TransactionSnarkPendingCoinbaseStackStateInitStackStableV1Versioned,
    pub ledger_witness: MinaBaseSparseLedgerStableV1Versioned,
}

/// **Origin**: `Transaction_snark_scan_state.Transaction_with_witness.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml:40:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml#L40)
///
/// **Gid**: 1574
pub type TransactionSnarkScanStateTransactionWithWitnessStableV1Versioned =
    crate::versioned::Versioned<
        TransactionSnarkScanStateTransactionWithWitnessStableV1VersionedV1,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml:151:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml#L151)
///
/// Gid: 1578
pub struct TransactionSnarkScanStateStableV1VersionedV1(
    pub  TransactionSnarkScanStateStableV1VersionedV1Poly<
        TransactionSnarkScanStateLedgerProofWithSokMessageStableV1Versioned,
        TransactionSnarkScanStateTransactionWithWitnessStableV1Versioned,
    >,
);

/// Location: [src/lib/mina_base/pending_coinbase.ml:1225:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L1225)
///
/// Gid: 1285
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStableV1VersionedV1PolyV1<Tree, StackId> {
    pub tree: Tree,
    pub pos_list: Vec<StackId>,
    pub new_pos: StackId,
}

/// Location: [src/lib/mina_base/pending_coinbase.ml:1225:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L1225)
///
/// Gid: 1287
pub type MinaBasePendingCoinbaseStableV1VersionedV1Poly<Tree, StackId> =
    crate::versioned::Versioned<
        MinaBasePendingCoinbaseStableV1VersionedV1PolyV1<Tree, StackId>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/pending_coinbase.ml:104:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L104)
///
/// Gid: 1234
pub struct MinaBasePendingCoinbaseStackIdStableV1VersionedV1(pub crate::number::Int32);

/// **Origin**: `Mina_base__Pending_coinbase.Stack_id.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/pending_coinbase.ml:104:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L104)
///
/// **Gid**: 1236
pub type MinaBasePendingCoinbaseStackIdStableV1Versioned =
    crate::versioned::Versioned<MinaBasePendingCoinbaseStackIdStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/pending_coinbase.ml:529:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L529)
///
/// Gid: 1282
pub struct MinaBasePendingCoinbaseMerkleTreeVersionedStableV1VersionedV1(
    pub  MinaBaseSparseLedgerStableV1VersionedV1Poly<
        MinaBasePendingCoinbaseHashVersionedStableV1Versioned,
        MinaBasePendingCoinbaseStackIdStableV1Versioned,
        MinaBasePendingCoinbaseStackVersionedStableV1Versioned,
        (),
    >,
);

/// **Origin**: `Mina_base__Pending_coinbase.Merkle_tree_versioned.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/pending_coinbase.ml:529:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L529)
///
/// **Gid**: 1284
pub type MinaBasePendingCoinbaseMerkleTreeVersionedStableV1Versioned = crate::versioned::Versioned<
    MinaBasePendingCoinbaseMerkleTreeVersionedStableV1VersionedV1,
    1i32,
>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/pending_coinbase.ml:1237:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L1237)
///
/// Gid: 1288
pub struct MinaBasePendingCoinbaseStableV1VersionedV1(
    pub  MinaBasePendingCoinbaseStableV1VersionedV1Poly<
        MinaBasePendingCoinbaseMerkleTreeVersionedStableV1Versioned,
        MinaBasePendingCoinbaseStackIdStableV1Versioned,
    >,
);

/// Location: [src/lib/syncable_ledger/syncable_ledger.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/syncable_ledger/syncable_ledger.ml#L17)
///
/// Gid: 808
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSyncLedgerQueryStableV1VersionedV1PolyV1<Addr> {
    WhatChildHashes(Addr),
    WhatContents(Addr),
    NumAccounts,
}

/// Location: [src/lib/syncable_ledger/syncable_ledger.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/syncable_ledger/syncable_ledger.ml#L17)
///
/// Gid: 810
pub type MinaBaseSyncLedgerQueryStableV1VersionedV1Poly<Addr> =
    crate::versioned::Versioned<MinaBaseSyncLedgerQueryStableV1VersionedV1PolyV1<Addr>, 1i32>;

/// Location: [src/lib/merkle_address/merkle_address.ml:48:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/merkle_address/merkle_address.ml#L48)
///
/// Gid: 789
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MerkleAddressBinableArgStableV1VersionedV1(
    pub crate::number::Int32,
    pub crate::string::ByteString,
);

/// **Origin**: `Merkle_address.Binable_arg.Stable.V1.t`
///
/// **Location**: [src/lib/merkle_address/merkle_address.ml:48:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/merkle_address/merkle_address.ml#L48)
///
/// **Gid**: 791
pub type MerkleAddressBinableArgStableV1Versioned =
    crate::versioned::Versioned<MerkleAddressBinableArgStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/merkle_address/merkle_address.ml:58:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/merkle_address/merkle_address.ml#L58)
///
/// Gid: 792
pub struct MerkleAddressStableV1VersionedV1(pub MerkleAddressBinableArgStableV1Versioned);

/// **Origin**: `Merkle_address.Stable.V1.t`
///
/// **Location**: [src/lib/merkle_address/merkle_address.ml:58:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/merkle_address/merkle_address.ml#L58)
///
/// **Gid**: 793
pub type MerkleAddressStableV1Versioned =
    crate::versioned::Versioned<MerkleAddressStableV1VersionedV1, 1i32>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/sync_ledger.ml:70:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/sync_ledger.ml#L70)
///
/// Gid: 1228
pub struct MinaBaseSyncLedgerQueryStableV1VersionedV1(
    pub MinaBaseSyncLedgerQueryStableV1VersionedV1Poly<MerkleAddressStableV1Versioned>,
);

/// Location: [src/lib/syncable_ledger/syncable_ledger.ml:35:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/syncable_ledger/syncable_ledger.ml#L35)
///
/// Gid: 811
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSyncLedgerAnswerStableV1VersionedV1PolyV1<Hash, Account> {
    ChildHashesAre(Hash, Hash),
    ContentsAre(Vec<Account>),
    NumAccounts(crate::number::Int32, Hash),
}

/// Location: [src/lib/syncable_ledger/syncable_ledger.ml:35:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/syncable_ledger/syncable_ledger.ml#L35)
///
/// Gid: 813
pub type MinaBaseSyncLedgerAnswerStableV1VersionedV1Poly<Hash, Account> =
    crate::versioned::Versioned<
        MinaBaseSyncLedgerAnswerStableV1VersionedV1PolyV1<Hash, Account>,
        1i32,
    >;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
/// Location: [src/lib/mina_base/sync_ledger.ml:55:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/sync_ledger.ml#L55)
///
/// Gid: 1225
pub struct MinaBaseSyncLedgerAnswerStableV1VersionedV1(
    pub  MinaBaseSyncLedgerAnswerStableV1VersionedV1Poly<
        MinaBaseStagedLedgerHashNonSnarkStableV1VersionedV1LedgerHash,
        MinaBaseAccountStableV1Versioned,
    >,
);

/// Location: [src/lib/sync_status/sync_status.ml:54:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/sync_status/sync_status.ml#L54)
///
/// Gid: 1781
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[polymorphic_variant]
pub enum SyncStatusTStableV1VersionedV1 {
    Connecting,
    Listening,
    Offline,
    Bootstrap,
    Synced,
    Catchup,
}

/// Location: [src/lib/trust_system/banned_status.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/trust_system/banned_status.ml#L7)
///
/// Gid: 799
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum TrustSystemBannedStatusStableV1VersionedV1 {
    Unbanned,
    BannedUntil(crate::number::Float64),
}

/// **Origin**: `Trust_system__Banned_status.Stable.V1.t`
///
/// **Location**: [src/lib/trust_system/banned_status.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/trust_system/banned_status.ml#L7)
///
/// **Gid**: 801
pub type TrustSystemBannedStatusStableV1Versioned =
    crate::versioned::Versioned<TrustSystemBannedStatusStableV1VersionedV1, 1i32>;

/// Location: [src/lib/trust_system/peer_status.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/trust_system/peer_status.ml#L6)
///
/// Gid: 802
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TrustSystemPeerStatusStableV1VersionedV1 {
    pub trust: crate::number::Float64,
    pub banned: TrustSystemBannedStatusStableV1Versioned,
}
