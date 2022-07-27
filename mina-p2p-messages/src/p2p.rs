use binprot::{BinProtRead, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};

/// **Origin**: `Mina_block__External_transition.Raw_versioned__.Stable.V1.t`
///
/// **Location**: [src/lib/mina_block/external_transition.ml:31:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_block/external_transition.ml#L31)
pub type MinaBlockExternalTransitionRawVersionedStable =
    crate::versioned::Versioned<MinaBlockExternalTransitionRawVersionedStableV1, 1i32>;

/// **Origin**: `Network_pool__Transaction_pool.Diff_versioned.Stable.V1.t`
///
/// **Location**: [src/lib/network_pool/transaction_pool.ml:45:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_pool/transaction_pool.ml#L45)
pub type NetworkPoolTransactionPoolDiffVersionedStable =
    crate::versioned::Versioned<NetworkPoolTransactionPoolDiffVersionedStableV1, 1i32>;

/// **Origin**: `Network_pool__Snark_pool.Diff_versioned.Stable.V1.t`
///
/// **Location**: [src/lib/network_pool/snark_pool.ml:705:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_pool/snark_pool.ml#L705)
pub type NetworkPoolSnarkPoolDiffVersionedStable =
    crate::versioned::Versioned<NetworkPoolSnarkPoolDiffVersionedStableV1, 1i32>;

/// **Origin**: `Mina_base__Account.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/account.ml:188:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account.ml#L188)
pub type MinaBaseAccountStable = crate::versioned::Versioned<MinaBaseAccountStableV1, 1i32>;


/// Location: [src/lib/mina_state/protocol_state.ml:16:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L16)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateProtocolStatePolyStableV1<StateHash, Body> {
    pub previous_state_hash: StateHash,
    pub body: Body,
}

/// **Origin**: `Mina_state__Protocol_state.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_state/protocol_state.ml:16:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L16)
pub type MinaStateProtocolStatePolyStable<StateHash, Body> =
    crate::versioned::Versioned<MinaStateProtocolStatePolyStableV1<StateHash, Body>, 1i32>;

/// Location: [src/lib/data_hash_lib/state_hash.ml:43:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/data_hash_lib/state_hash.ml#L43)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateProtocolStatePolyStableArg0V1Poly(pub crate::bigint::BigInt);

/// Location: [src/lib/data_hash_lib/state_hash.ml:42:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/data_hash_lib/state_hash.ml#L42)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateProtocolStatePolyStableArg0V1(pub MinaStateProtocolStatePolyStableArg0V1Poly);

/// Location: [src/lib/data_hash_lib/state_hash.ml:42:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/data_hash_lib/state_hash.ml#L42)
pub type MinaStateProtocolStatePolyStableArg0 =
    crate::versioned::Versioned<MinaStateProtocolStatePolyStableArg0V1, 1i32>;

/// Location: [src/lib/mina_state/protocol_state.ml:38:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L38)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateProtocolStateBodyPolyStableV1<
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

/// **Origin**: `Mina_state__Protocol_state.Body.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_state/protocol_state.ml:38:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L38)
pub type MinaStateProtocolStateBodyPolyStable<
    StateHash,
    BlockchainState,
    ConsensusState,
    Constants,
> = crate::versioned::Versioned<
    MinaStateProtocolStateBodyPolyStableV1<StateHash, BlockchainState, ConsensusState, Constants>,
    1i32,
>;

/// Location: [src/lib/mina_state/blockchain_state.ml:9:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/blockchain_state.ml#L9)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateBlockchainStatePolyStableV1<StagedLedgerHash, SnarkedLedgerHash, TokenId, Time>
{
    pub staged_ledger_hash: StagedLedgerHash,
    pub snarked_ledger_hash: SnarkedLedgerHash,
    pub genesis_ledger_hash: SnarkedLedgerHash,
    pub snarked_next_available_token: TokenId,
    pub timestamp: Time,
}

/// **Origin**: `Mina_state__Blockchain_state.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_state/blockchain_state.ml:9:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/blockchain_state.ml#L9)
pub type MinaStateBlockchainStatePolyStable<StagedLedgerHash, SnarkedLedgerHash, TokenId, Time> =
    crate::versioned::Versioned<
        MinaStateBlockchainStatePolyStableV1<StagedLedgerHash, SnarkedLedgerHash, TokenId, Time>,
        1i32,
    >;

/// Location: [src/lib/mina_base/staged_ledger_hash.ml:174:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L174)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashPolyStableV1<NonSnark, PendingCoinbaseHash> {
    pub non_snark: NonSnark,
    pub pending_coinbase_hash: PendingCoinbaseHash,
}

/// **Origin**: `Mina_base__Staged_ledger_hash.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/staged_ledger_hash.ml:174:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L174)
pub type MinaBaseStagedLedgerHashPolyStable<NonSnark, PendingCoinbaseHash> =
    crate::versioned::Versioned<
        MinaBaseStagedLedgerHashPolyStableV1<NonSnark, PendingCoinbaseHash>,
        1i32,
    >;

/// Location: [src/lib/mina_base/ledger_hash0.ml:18:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/ledger_hash0.ml#L18)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashNonSnarkStableV1LedgerHashV1Poly(pub crate::bigint::BigInt);

/// Location: [src/lib/mina_base/ledger_hash0.ml:17:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/ledger_hash0.ml#L17)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashNonSnarkStableV1LedgerHashV1(
    pub MinaBaseStagedLedgerHashNonSnarkStableV1LedgerHashV1Poly,
);

/// Location: [src/lib/mina_base/ledger_hash0.ml:17:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/ledger_hash0.ml#L17)
pub type MinaBaseStagedLedgerHashNonSnarkStableV1LedgerHash =
    crate::versioned::Versioned<MinaBaseStagedLedgerHashNonSnarkStableV1LedgerHashV1, 1i32>;

/// Location: [src/lib/mina_base/staged_ledger_hash.ml:15:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L15)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashAuxHashStableV1(pub crate::string::String);

/// **Origin**: `Mina_base__Staged_ledger_hash.Aux_hash.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/staged_ledger_hash.ml:15:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L15)
pub type MinaBaseStagedLedgerHashAuxHashStable =
    crate::versioned::Versioned<MinaBaseStagedLedgerHashAuxHashStableV1, 1i32>;

/// Location: [src/lib/mina_base/staged_ledger_hash.ml:59:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L59)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashPendingCoinbaseAuxStableV1(pub crate::string::String);

/// **Origin**: `Mina_base__Staged_ledger_hash.Pending_coinbase_aux.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/staged_ledger_hash.ml:59:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L59)
pub type MinaBaseStagedLedgerHashPendingCoinbaseAuxStable =
    crate::versioned::Versioned<MinaBaseStagedLedgerHashPendingCoinbaseAuxStableV1, 1i32>;

/// Location: [src/lib/mina_base/staged_ledger_hash.ml:95:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L95)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashNonSnarkStableV1 {
    pub ledger_hash: MinaBaseStagedLedgerHashNonSnarkStableV1LedgerHash,
    pub aux_hash: MinaBaseStagedLedgerHashAuxHashStable,
    pub pending_coinbase_aux: MinaBaseStagedLedgerHashPendingCoinbaseAuxStable,
}

/// **Origin**: `Mina_base__Staged_ledger_hash.Non_snark.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/staged_ledger_hash.ml:95:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L95)
pub type MinaBaseStagedLedgerHashNonSnarkStable =
    crate::versioned::Versioned<MinaBaseStagedLedgerHashNonSnarkStableV1, 1i32>;

/// Location: [src/lib/mina_base/pending_coinbase.ml:359:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L359)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseHashVersionedStableV1PolyV1Poly(pub crate::bigint::BigInt);

/// Location: [src/lib/mina_base/pending_coinbase.ml:358:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L358)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseHashVersionedStableV1PolyV1(
    pub MinaBasePendingCoinbaseHashVersionedStableV1PolyV1Poly,
);

/// Location: [src/lib/mina_base/pending_coinbase.ml:358:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L358)
pub type MinaBasePendingCoinbaseHashVersionedStableV1Poly =
    crate::versioned::Versioned<MinaBasePendingCoinbaseHashVersionedStableV1PolyV1, 1i32>;

/// Location: [src/lib/mina_base/pending_coinbase.ml:517:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L517)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseHashVersionedStableV1(
    pub MinaBasePendingCoinbaseHashVersionedStableV1Poly,
);

/// **Origin**: `Mina_base__Pending_coinbase.Hash_versioned.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/pending_coinbase.ml:517:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L517)
pub type MinaBasePendingCoinbaseHashVersionedStable =
    crate::versioned::Versioned<MinaBasePendingCoinbaseHashVersionedStableV1, 1i32>;

/// Location: [src/lib/mina_base/staged_ledger_hash.ml:191:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L191)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashStableV1(
    pub  MinaBaseStagedLedgerHashPolyStable<
        MinaBaseStagedLedgerHashNonSnarkStable,
        MinaBasePendingCoinbaseHashVersionedStable,
    >,
);

/// **Origin**: `Mina_base__Staged_ledger_hash.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/staged_ledger_hash.ml:191:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L191)
pub type MinaBaseStagedLedgerHashStable =
    crate::versioned::Versioned<MinaBaseStagedLedgerHashStableV1, 1i32>;

/// Location: [src/lib/unsigned_extended/unsigned_extended.ml:76:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/unsigned_extended/unsigned_extended.ml#L76)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct UnsignedExtendedUInt64StableV1(pub i64);

/// **Origin**: `Unsigned_extended.UInt64.Stable.V1.t`
///
/// **Location**: [src/lib/unsigned_extended/unsigned_extended.ml:76:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/unsigned_extended/unsigned_extended.ml#L76)
pub type UnsignedExtendedUInt64Stable =
    crate::versioned::Versioned<UnsignedExtendedUInt64StableV1, 1i32>;

/// Location: [src/lib/mina_numbers/nat.ml:220:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L220)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaNumbersNatMake64StableV1(pub UnsignedExtendedUInt64Stable);

/// **Origin**: `Mina_numbers__Nat.Make64.Stable.V1.t`
///
/// **Location**: [src/lib/mina_numbers/nat.ml:220:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L220)
pub type MinaNumbersNatMake64Stable =
    crate::versioned::Versioned<MinaNumbersNatMake64StableV1, 1i32>;

/// Location: [src/lib/mina_base/token_id.ml:49:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/token_id.ml#L49)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseTokenIdStableV1(pub MinaNumbersNatMake64Stable);

/// **Origin**: `Mina_base__Token_id.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/token_id.ml:49:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/token_id.ml#L49)
pub type MinaBaseTokenIdStable = crate::versioned::Versioned<MinaBaseTokenIdStableV1, 1i32>;

/// Location: [src/lib/block_time/block_time.ml:14:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/block_time/block_time.ml#L14)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct BlockTimeTimeStableV1(pub UnsignedExtendedUInt64Stable);

/// **Origin**: `Block_time.Time.Stable.V1.t`
///
/// **Location**: [src/lib/block_time/block_time.ml:14:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/block_time/block_time.ml#L14)
pub type BlockTimeTimeStable = crate::versioned::Versioned<BlockTimeTimeStableV1, 1i32>;

/// Location: [src/lib/mina_state/blockchain_state.ml:35:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/blockchain_state.ml#L35)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateBlockchainStateValueStableV1(
    pub  MinaStateBlockchainStatePolyStable<
        MinaBaseStagedLedgerHashStable,
        MinaBaseStagedLedgerHashNonSnarkStableV1LedgerHash,
        MinaBaseTokenIdStable,
        BlockTimeTimeStable,
    >,
);

/// **Origin**: `Mina_state__Blockchain_state.Value.Stable.V1.t`
///
/// **Location**: [src/lib/mina_state/blockchain_state.ml:35:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/blockchain_state.ml#L35)
pub type MinaStateBlockchainStateValueStable =
    crate::versioned::Versioned<MinaStateBlockchainStateValueStableV1, 1i32>;

/// Location: [src/lib/consensus/proof_of_stake.ml:1681:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1681)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataConsensusStatePolyStableV1<
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

/// **Origin**: `Consensus__Proof_of_stake.Data.Consensus_state.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/consensus/proof_of_stake.ml:1681:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1681)
pub type ConsensusProofOfStakeDataConsensusStatePolyStable<
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
    ConsensusProofOfStakeDataConsensusStatePolyStableV1<
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

/// Location: [src/lib/unsigned_extended/unsigned_extended.ml:126:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/unsigned_extended/unsigned_extended.ml#L126)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct UnsignedExtendedUInt32StableV1(pub i32);

/// **Origin**: `Unsigned_extended.UInt32.Stable.V1.t`
///
/// **Location**: [src/lib/unsigned_extended/unsigned_extended.ml:126:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/unsigned_extended/unsigned_extended.ml#L126)
pub type UnsignedExtendedUInt32Stable =
    crate::versioned::Versioned<UnsignedExtendedUInt32StableV1, 1i32>;

/// Location: [src/lib/mina_numbers/nat.ml:186:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L186)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataConsensusStatePolyStableArg0V1(
    pub UnsignedExtendedUInt32Stable,
);

/// Location: [src/lib/mina_numbers/nat.ml:186:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L186)
pub type ConsensusProofOfStakeDataConsensusStatePolyStableArg0 =
    crate::versioned::Versioned<ConsensusProofOfStakeDataConsensusStatePolyStableArg0V1, 1i32>;

/// Location: [src/lib/consensus/vrf/consensus_vrf.ml:170:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/vrf/consensus_vrf.ml#L170)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusVrfOutputTruncatedStableV1(pub crate::string::String);

/// **Origin**: `Consensus_vrf.Output.Truncated.Stable.V1.t`
///
/// **Location**: [src/lib/consensus/vrf/consensus_vrf.ml:170:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/vrf/consensus_vrf.ml#L170)
pub type ConsensusVrfOutputTruncatedStable =
    crate::versioned::Versioned<ConsensusVrfOutputTruncatedStableV1, 1i32>;

/// Location: [src/lib/currency/currency.ml:712:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/currency.ml#L712)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CurrencyAmountMakeStrStableV1(pub UnsignedExtendedUInt64Stable);

/// **Origin**: `Currency.Amount.Make_str.Stable.V1.t`
///
/// **Location**: [src/lib/currency/currency.ml:712:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/currency.ml#L712)
pub type CurrencyAmountMakeStrStable =
    crate::versioned::Versioned<CurrencyAmountMakeStrStableV1, 1i32>;

/// Location: [src/lib/consensus/global_slot.ml:11:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/global_slot.ml#L11)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusGlobalSlotPolyStableV1<SlotNumber, SlotsPerEpoch> {
    pub slot_number: SlotNumber,
    pub slots_per_epoch: SlotsPerEpoch,
}

/// **Origin**: `Consensus__Global_slot.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/consensus/global_slot.ml:11:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/global_slot.ml#L11)
pub type ConsensusGlobalSlotPolyStable<SlotNumber, SlotsPerEpoch> =
    crate::versioned::Versioned<ConsensusGlobalSlotPolyStableV1<SlotNumber, SlotsPerEpoch>, 1i32>;

/// Location: [src/lib/mina_numbers/nat.ml:186:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L186)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusGlobalSlotPolyStableArg0V1(pub UnsignedExtendedUInt32Stable);

/// Location: [src/lib/mina_numbers/nat.ml:186:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L186)
pub type ConsensusGlobalSlotPolyStableArg0 =
    crate::versioned::Versioned<ConsensusGlobalSlotPolyStableArg0V1, 1i32>;

/// Location: [src/lib/consensus/global_slot.ml:21:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/global_slot.ml#L21)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusGlobalSlotStableV1(
    pub  ConsensusGlobalSlotPolyStable<
        ConsensusGlobalSlotPolyStableArg0,
        ConsensusProofOfStakeDataConsensusStatePolyStableArg0,
    >,
);

/// **Origin**: `Consensus__Global_slot.Stable.V1.t`
///
/// **Location**: [src/lib/consensus/global_slot.ml:21:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/global_slot.ml#L21)
pub type ConsensusGlobalSlotStable = crate::versioned::Versioned<ConsensusGlobalSlotStableV1, 1i32>;

/// Location: [src/lib/mina_base/epoch_data.ml:8:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_data.ml#L8)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseEpochDataPolyStableV1<
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

/// **Origin**: `Mina_base__Epoch_data.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/epoch_data.ml:8:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_data.ml#L8)
pub type MinaBaseEpochDataPolyStable<
    EpochLedger,
    EpochSeed,
    StartCheckpoint,
    LockCheckpoint,
    Length,
> = crate::versioned::Versioned<
    MinaBaseEpochDataPolyStableV1<EpochLedger, EpochSeed, StartCheckpoint, LockCheckpoint, Length>,
    1i32,
>;

/// Location: [src/lib/mina_base/epoch_ledger.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_ledger.ml#L10)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseEpochLedgerPolyStableV1<LedgerHash, Amount> {
    pub hash: LedgerHash,
    pub total_currency: Amount,
}

/// **Origin**: `Mina_base__Epoch_ledger.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/epoch_ledger.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_ledger.ml#L10)
pub type MinaBaseEpochLedgerPolyStable<LedgerHash, Amount> =
    crate::versioned::Versioned<MinaBaseEpochLedgerPolyStableV1<LedgerHash, Amount>, 1i32>;

/// Location: [src/lib/mina_base/epoch_ledger.ml:21:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_ledger.ml#L21)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseEpochLedgerValueStableV1(
    pub  MinaBaseEpochLedgerPolyStable<
        MinaBaseStagedLedgerHashNonSnarkStableV1LedgerHash,
        CurrencyAmountMakeStrStable,
    >,
);

/// **Origin**: `Mina_base__Epoch_ledger.Value.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/epoch_ledger.ml:21:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_ledger.ml#L21)
pub type MinaBaseEpochLedgerValueStable =
    crate::versioned::Versioned<MinaBaseEpochLedgerValueStableV1, 1i32>;

/// Location: [src/lib/mina_base/epoch_seed.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_seed.ml#L17)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseEpochDataPolyStableArg1V1Poly(pub crate::bigint::BigInt);

/// Location: [src/lib/mina_base/epoch_seed.ml:16:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_seed.ml#L16)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseEpochDataPolyStableArg1V1(pub MinaBaseEpochDataPolyStableArg1V1Poly);

/// Location: [src/lib/mina_base/epoch_seed.ml:16:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_seed.ml#L16)
pub type MinaBaseEpochDataPolyStableArg1 =
    crate::versioned::Versioned<MinaBaseEpochDataPolyStableArg1V1, 1i32>;

/// Location: [src/lib/consensus/proof_of_stake.ml:1050:12](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1050)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1(
    pub  MinaBaseEpochDataPolyStable<
        MinaBaseEpochLedgerValueStable,
        MinaBaseEpochDataPolyStableArg1,
        MinaStateProtocolStatePolyStableArg0,
        MinaStateProtocolStatePolyStableArg0,
        ConsensusProofOfStakeDataConsensusStatePolyStableArg0,
    >,
);

/// **Origin**: `Consensus__Proof_of_stake.Data.Epoch_data.Staking_value_versioned.Value.Stable.V1.t`
///
/// **Location**: [src/lib/consensus/proof_of_stake.ml:1050:12](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1050)
pub type ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStable =
    crate::versioned::Versioned<
        ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
        1i32,
    >;

/// Location: [src/lib/consensus/proof_of_stake.ml:1074:12](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1074)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1(
    pub  MinaBaseEpochDataPolyStable<
        MinaBaseEpochLedgerValueStable,
        MinaBaseEpochDataPolyStableArg1,
        MinaStateProtocolStatePolyStableArg0,
        MinaStateProtocolStatePolyStableArg0,
        ConsensusProofOfStakeDataConsensusStatePolyStableArg0,
    >,
);

/// **Origin**: `Consensus__Proof_of_stake.Data.Epoch_data.Next_value_versioned.Value.Stable.V1.t`
///
/// **Location**: [src/lib/consensus/proof_of_stake.ml:1074:12](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1074)
pub type ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStable =
    crate::versioned::Versioned<
        ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
        1i32,
    >;

/// Location: [src/lib/non_zero_curve_point/compressed_poly.ml:11:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/compressed_poly.ml#L11)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NonZeroCurvePointCompressedPolyPolyStableV1<Field, Boolean> {
    pub x: Field,
    pub is_odd: Boolean,
}

/// **Origin**: `Non_zero_curve_point__Compressed_poly.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/non_zero_curve_point/compressed_poly.ml:11:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/compressed_poly.ml#L11)
pub type NonZeroCurvePointCompressedPolyPolyStable<Field, Boolean> =
    crate::versioned::Versioned<NonZeroCurvePointCompressedPolyPolyStableV1<Field, Boolean>, 1i32>;

/// Location: [src/lib/non_zero_curve_point/non_zero_curve_point.ml:52:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/non_zero_curve_point.ml#L52)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataConsensusStatePolyStableArg8V1Poly(
    pub NonZeroCurvePointCompressedPolyPolyStable<crate::bigint::BigInt, bool>,
);

/// Location: [src/lib/non_zero_curve_point/non_zero_curve_point.ml:51:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/non_zero_curve_point.ml#L51)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataConsensusStatePolyStableArg8V1(
    pub ConsensusProofOfStakeDataConsensusStatePolyStableArg8V1Poly,
);

/// Location: [src/lib/non_zero_curve_point/non_zero_curve_point.ml:51:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/non_zero_curve_point.ml#L51)
pub type ConsensusProofOfStakeDataConsensusStatePolyStableArg8 =
    crate::versioned::Versioned<ConsensusProofOfStakeDataConsensusStatePolyStableArg8V1, 1i32>;

/// Location: [src/lib/consensus/proof_of_stake.ml:1716:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1716)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataConsensusStateValueStableV1(
    pub  ConsensusProofOfStakeDataConsensusStatePolyStable<
        ConsensusProofOfStakeDataConsensusStatePolyStableArg0,
        ConsensusVrfOutputTruncatedStable,
        CurrencyAmountMakeStrStable,
        ConsensusGlobalSlotStable,
        ConsensusGlobalSlotPolyStableArg0,
        ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStable,
        ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStable,
        bool,
        ConsensusProofOfStakeDataConsensusStatePolyStableArg8,
    >,
);

/// **Origin**: `Consensus__Proof_of_stake.Data.Consensus_state.Value.Stable.V1.t`
///
/// **Location**: [src/lib/consensus/proof_of_stake.ml:1716:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1716)
pub type ConsensusProofOfStakeDataConsensusStateValueStable =
    crate::versioned::Versioned<ConsensusProofOfStakeDataConsensusStateValueStableV1, 1i32>;

/// Location: [src/lib/genesis_constants/genesis_constants.ml:239:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/genesis_constants/genesis_constants.ml#L239)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct GenesisConstantsProtocolPolyStableV1<Length, Delta, GenesisStateTimestamp> {
    pub k: Length,
    pub slots_per_epoch: Length,
    pub slots_per_sub_window: Length,
    pub delta: Delta,
    pub genesis_state_timestamp: GenesisStateTimestamp,
}

/// **Origin**: `Genesis_constants.Protocol.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/genesis_constants/genesis_constants.ml:239:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/genesis_constants/genesis_constants.ml#L239)
pub type GenesisConstantsProtocolPolyStable<Length, Delta, GenesisStateTimestamp> =
    crate::versioned::Versioned<
        GenesisConstantsProtocolPolyStableV1<Length, Delta, GenesisStateTimestamp>,
        1i32,
    >;

/// Location: [src/lib/mina_base/protocol_constants_checked.ml:22:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/protocol_constants_checked.ml#L22)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseProtocolConstantsCheckedValueStableV1(
    pub  GenesisConstantsProtocolPolyStable<
        ConsensusProofOfStakeDataConsensusStatePolyStableArg0,
        ConsensusProofOfStakeDataConsensusStatePolyStableArg0,
        BlockTimeTimeStable,
    >,
);

/// **Origin**: `Mina_base__Protocol_constants_checked.Value.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/protocol_constants_checked.ml:22:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/protocol_constants_checked.ml#L22)
pub type MinaBaseProtocolConstantsCheckedValueStable =
    crate::versioned::Versioned<MinaBaseProtocolConstantsCheckedValueStableV1, 1i32>;

/// Location: [src/lib/mina_state/protocol_state.ml:53:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L53)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateProtocolStateBodyValueStableV1(
    pub  MinaStateProtocolStateBodyPolyStable<
        MinaStateProtocolStatePolyStableArg0,
        MinaStateBlockchainStateValueStable,
        ConsensusProofOfStakeDataConsensusStateValueStable,
        MinaBaseProtocolConstantsCheckedValueStable,
    >,
);

/// **Origin**: `Mina_state__Protocol_state.Body.Value.Stable.V1.t`
///
/// **Location**: [src/lib/mina_state/protocol_state.ml:53:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L53)
pub type MinaStateProtocolStateBodyValueStable =
    crate::versioned::Versioned<MinaStateProtocolStateBodyValueStableV1, 1i32>;

/// Location: [src/lib/mina_state/protocol_state.ml:177:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L177)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateProtocolStateValueStableV1(
    pub  MinaStateProtocolStatePolyStable<
        MinaStateProtocolStatePolyStableArg0,
        MinaStateProtocolStateBodyValueStable,
    >,
);

/// **Origin**: `Mina_state__Protocol_state.Value.Stable.V1.t`
///
/// **Location**: [src/lib/mina_state/protocol_state.ml:177:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L177)
pub type MinaStateProtocolStateValueStable =
    crate::versioned::Versioned<MinaStateProtocolStateValueStableV1, 1i32>;

/// Location: [src/lib/pickles/composition_types/composition_types.ml:155:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L155)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CompositionTypesDlogBasedProofStateDeferredValuesStableV1<
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

/// **Origin**: `Composition_types.Dlog_based.Proof_state.Deferred_values.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/composition_types/composition_types.ml:155:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L155)
pub type CompositionTypesDlogBasedProofStateDeferredValuesStable<
    Plonk,
    ScalarChallenge,
    Fp,
    Fq,
    BulletproofChallenges,
    Index,
> = crate::versioned::Versioned<
    CompositionTypesDlogBasedProofStateDeferredValuesStableV1<
        Plonk,
        ScalarChallenge,
        Fp,
        Fq,
        BulletproofChallenges,
        Index,
    >,
    1i32,
>;

/// Location: [src/lib/pickles/composition_types/composition_types.ml:299:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L299)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CompositionTypesDlogBasedProofStateStableV1<
    Plonk,
    ScalarChallenge,
    Fp,
    Fq,
    MeOnly,
    Digest,
    BpChals,
    Index,
> {
    pub deferred_values: CompositionTypesDlogBasedProofStateDeferredValuesStable<
        Plonk,
        ScalarChallenge,
        Fp,
        Fq,
        BpChals,
        Index,
    >,
    pub sponge_digest_before_evaluations: Digest,
    pub me_only: MeOnly,
}

/// **Origin**: `Composition_types.Dlog_based.Proof_state.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/composition_types/composition_types.ml:299:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L299)
pub type CompositionTypesDlogBasedProofStateStable<
    Plonk,
    ScalarChallenge,
    Fp,
    Fq,
    MeOnly,
    Digest,
    BpChals,
    Index,
> = crate::versioned::Versioned<
    CompositionTypesDlogBasedProofStateStableV1<
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CompositionTypesDlogBasedStatementStableV1<
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
    pub proof_state: CompositionTypesDlogBasedProofStateStable<
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

/// **Origin**: `Composition_types.Dlog_based.Statement.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/composition_types/composition_types.ml:444:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L444)
pub type CompositionTypesDlogBasedStatementStable<
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
    CompositionTypesDlogBasedStatementStableV1<
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CompositionTypesDlogBasedProofStateDeferredValuesPlonkMinimalStableV1<
    Challenge,
    ScalarChallenge,
> {
    pub alpha: ScalarChallenge,
    pub beta: Challenge,
    pub gamma: Challenge,
    pub zeta: ScalarChallenge,
}

/// **Origin**: `Composition_types.Dlog_based.Proof_state.Deferred_values.Plonk.Minimal.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/composition_types/composition_types.ml:62:14](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L62)
pub type CompositionTypesDlogBasedProofStateDeferredValuesPlonkMinimalStable<
    Challenge,
    ScalarChallenge,
> = crate::versioned::Versioned<
    CompositionTypesDlogBasedProofStateDeferredValuesPlonkMinimalStableV1<
        Challenge,
        ScalarChallenge,
    >,
    1i32,
>;

/// Location: [src/lib/pickles/composition_types/composition_types.ml:474:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L474)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CompositionTypesDlogBasedStatementMinimalStableV1<
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
    pub  CompositionTypesDlogBasedStatementStable<
        CompositionTypesDlogBasedProofStateDeferredValuesPlonkMinimalStable<
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

/// **Origin**: `Composition_types.Dlog_based.Statement.Minimal.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/composition_types/composition_types.ml:474:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L474)
pub type CompositionTypesDlogBasedStatementMinimalStable<
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
    CompositionTypesDlogBasedStatementMinimalStableV1<
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesTypesVectorVector2StableV1<A>(pub A, pub (A, ()));

/// **Origin**: `Pickles_types__Vector.Vector_2.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_types/vector.ml:445:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L445)
pub type PicklesTypesVectorVector2Stable<A> =
    crate::versioned::Versioned<PicklesTypesVectorVector2StableV1<A>, 1i32>;

/// Location: [src/lib/pickles/limb_vector/constant.ml:61:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/limb_vector/constant.ml#L61)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct LimbVectorConstantHex64StableV1(pub i64);

/// **Origin**: `Limb_vector__Constant.Hex64.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/limb_vector/constant.ml:61:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/limb_vector/constant.ml#L61)
pub type LimbVectorConstantHex64Stable =
    crate::versioned::Versioned<LimbVectorConstantHex64StableV1, 1i32>;

/// Location: [src/lib/pickles_types/scalar_challenge.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/scalar_challenge.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum PicklesTypesScalarChallengeStableV1<F> {
    ScalarChallenge(F),
}

/// **Origin**: `Pickles_types__Scalar_challenge.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_types/scalar_challenge.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/scalar_challenge.ml#L6)
pub type PicklesTypesScalarChallengeStable<F> =
    crate::versioned::Versioned<PicklesTypesScalarChallengeStableV1<F>, 1i32>;

/// Location: [src/lib/pickles_types/shifted_value.ml:31:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/shifted_value.ml#L31)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum PicklesTypesShiftedValueStableV1<F> {
    ShiftedValue(F),
}

/// **Origin**: `Pickles_types__Shifted_value.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_types/shifted_value.ml:31:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/shifted_value.ml#L31)
pub type PicklesTypesShiftedValueStable<F> =
    crate::versioned::Versioned<PicklesTypesShiftedValueStableV1<F>, 1i32>;

/// Location: [src/lib/pickles_types/vector.ml:474:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L474)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesTypesVectorVector4StableV1<A>(pub A, pub (A, (A, (A, ()))));

/// **Origin**: `Pickles_types__Vector.Vector_4.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_types/vector.ml:474:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L474)
pub type PicklesTypesVectorVector4Stable<A> =
    crate::versioned::Versioned<PicklesTypesVectorVector4StableV1<A>, 1i32>;

/// Location: [src/lib/pickles/composition_types/digest.ml:13:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/digest.ml#L13)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CompositionTypesDigestConstantStableV1(
    pub PicklesTypesVectorVector4Stable<LimbVectorConstantHex64Stable>,
);

/// **Origin**: `Composition_types__Digest.Constant.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/composition_types/digest.ml:13:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/digest.ml#L13)
pub type CompositionTypesDigestConstantStable =
    crate::versioned::Versioned<CompositionTypesDigestConstantStableV1, 1i32>;

/// Location: [src/lib/pickles_types/vector.ml:561:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L561)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesTypesVectorVector18StableV1<A>(
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

/// **Origin**: `Pickles_types__Vector.Vector_18.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_types/vector.ml:561:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L561)
pub type PicklesTypesVectorVector18Stable<A> =
    crate::versioned::Versioned<PicklesTypesVectorVector18StableV1<A>, 1i32>;

/// Location: [src/lib/zexe_backend/pasta/basic.ml:54:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/zexe_backend/pasta/basic.ml#L54)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PastaBasicRoundsStepVectorStableV1<A>(pub PicklesTypesVectorVector18Stable<A>);

/// **Origin**: `Pasta__Basic.Rounds.Step_vector.Stable.V1.t`
///
/// **Location**: [src/lib/zexe_backend/pasta/basic.ml:54:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/zexe_backend/pasta/basic.ml#L54)
pub type PastaBasicRoundsStepVectorStable<A> =
    crate::versioned::Versioned<PastaBasicRoundsStepVectorStableV1<A>, 1i32>;

/// Location: [src/lib/pickles/composition_types/bulletproof_challenge.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/bulletproof_challenge.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CompositionTypesBulletproofChallengeStableV1<Challenge> {
    pub prechallenge: Challenge,
}

/// **Origin**: `Composition_types__Bulletproof_challenge.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/composition_types/bulletproof_challenge.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/bulletproof_challenge.ml#L6)
pub type CompositionTypesBulletproofChallengeStable<Challenge> =
    crate::versioned::Versioned<CompositionTypesBulletproofChallengeStableV1<Challenge>, 1i32>;

/// Location: [src/lib/pickles/composition_types/index.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/index.ml#L7)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CompositionTypesIndexStableV1(pub crate::char_::Char);

/// **Origin**: `Composition_types__Index.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/composition_types/index.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/index.ml#L7)
pub type CompositionTypesIndexStable =
    crate::versioned::Versioned<CompositionTypesIndexStableV1, 1i32>;

/// Location: [src/lib/pickles/proof.ml:34:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L34)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBaseDoubleStableV1<A>(pub A, pub A);

/// **Origin**: `Pickles__Proof.Base.Double.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/proof.ml:34:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L34)
pub type PicklesProofBaseDoubleStable<A> =
    crate::versioned::Versioned<PicklesProofBaseDoubleStableV1<A>, 1i32>;

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:30:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L30)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesTypesDlogPlonkTypesEvalsStableV1<A> {
    pub l: A,
    pub r: A,
    pub o: A,
    pub z: A,
    pub t: A,
    pub f: A,
    pub sigma1: A,
    pub sigma2: A,
}

/// **Origin**: `Pickles_types__Dlog_plonk_types.Evals.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_types/dlog_plonk_types.ml:30:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L30)
pub type PicklesTypesDlogPlonkTypesEvalsStable<A> =
    crate::versioned::Versioned<PicklesTypesDlogPlonkTypesEvalsStableV1<A>, 1i32>;

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L17)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesTypesDlogPlonkTypesPcArrayStableV1<A>(pub Vec<A>);

/// **Origin**: `Pickles_types__Dlog_plonk_types.Pc_array.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_types/dlog_plonk_types.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L17)
pub type PicklesTypesDlogPlonkTypesPcArrayStable<A> =
    crate::versioned::Versioned<PicklesTypesDlogPlonkTypesPcArrayStableV1<A>, 1i32>;

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:203:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L203)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesTypesDlogPlonkTypesPolyCommWithoutDegreeBoundStableV1<G>(
    pub PicklesTypesDlogPlonkTypesPcArrayStable<G>,
);

/// **Origin**: `Pickles_types__Dlog_plonk_types.Poly_comm.Without_degree_bound.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_types/dlog_plonk_types.ml:203:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L203)
pub type PicklesTypesDlogPlonkTypesPolyCommWithoutDegreeBoundStable<G> =
    crate::versioned::Versioned<
        PicklesTypesDlogPlonkTypesPolyCommWithoutDegreeBoundStableV1<G>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:149:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L149)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesTypesDlogPlonkTypesPolyCommWithDegreeBoundStableV1<GOpt> {
    pub unshifted: PicklesTypesDlogPlonkTypesPcArrayStable<GOpt>,
    pub shifted: GOpt,
}

/// **Origin**: `Pickles_types__Dlog_plonk_types.Poly_comm.With_degree_bound.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_types/dlog_plonk_types.ml:149:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L149)
pub type PicklesTypesDlogPlonkTypesPolyCommWithDegreeBoundStable<GOpt> =
    crate::versioned::Versioned<
        PicklesTypesDlogPlonkTypesPolyCommWithDegreeBoundStableV1<GOpt>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:218:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L218)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesTypesDlogPlonkTypesMessagesStableV1<G, GOpt> {
    pub l_comm: PicklesTypesDlogPlonkTypesPolyCommWithoutDegreeBoundStable<G>,
    pub r_comm: PicklesTypesDlogPlonkTypesPolyCommWithoutDegreeBoundStable<G>,
    pub o_comm: PicklesTypesDlogPlonkTypesPolyCommWithoutDegreeBoundStable<G>,
    pub z_comm: PicklesTypesDlogPlonkTypesPolyCommWithoutDegreeBoundStable<G>,
    pub t_comm: PicklesTypesDlogPlonkTypesPolyCommWithDegreeBoundStable<GOpt>,
}

/// **Origin**: `Pickles_types__Dlog_plonk_types.Messages.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_types/dlog_plonk_types.ml:218:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L218)
pub type PicklesTypesDlogPlonkTypesMessagesStable<G, GOpt> =
    crate::versioned::Versioned<PicklesTypesDlogPlonkTypesMessagesStableV1<G, GOpt>, 1i32>;

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:102:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L102)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesTypesDlogPlonkTypesOpeningsBulletproofStableV1<G, Fq> {
    pub lr: PicklesTypesDlogPlonkTypesPcArrayStable<(G, G)>,
    pub z_1: Fq,
    pub z_2: Fq,
    pub delta: G,
    pub sg: G,
}

/// **Origin**: `Pickles_types__Dlog_plonk_types.Openings.Bulletproof.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_types/dlog_plonk_types.ml:102:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L102)
pub type PicklesTypesDlogPlonkTypesOpeningsBulletproofStable<G, Fq> =
    crate::versioned::Versioned<PicklesTypesDlogPlonkTypesOpeningsBulletproofStableV1<G, Fq>, 1i32>;

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:124:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L124)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesTypesDlogPlonkTypesOpeningsStableV1<G, Fq, Fqv> {
    pub proof: PicklesTypesDlogPlonkTypesOpeningsBulletproofStable<G, Fq>,
    pub evals: (
        PicklesTypesDlogPlonkTypesEvalsStable<Fqv>,
        PicklesTypesDlogPlonkTypesEvalsStable<Fqv>,
    ),
}

/// **Origin**: `Pickles_types__Dlog_plonk_types.Openings.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_types/dlog_plonk_types.ml:124:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L124)
pub type PicklesTypesDlogPlonkTypesOpeningsStable<G, Fq, Fqv> =
    crate::versioned::Versioned<PicklesTypesDlogPlonkTypesOpeningsStableV1<G, Fq, Fqv>, 1i32>;

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:250:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L250)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesTypesDlogPlonkTypesProofStableV1<G, GOpt, Fq, Fqv> {
    pub messages: PicklesTypesDlogPlonkTypesMessagesStable<G, GOpt>,
    pub openings: PicklesTypesDlogPlonkTypesOpeningsStable<G, Fq, Fqv>,
}

/// **Origin**: `Pickles_types__Dlog_plonk_types.Proof.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_types/dlog_plonk_types.ml:250:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L250)
pub type PicklesTypesDlogPlonkTypesProofStable<G, GOpt, Fq, Fqv> =
    crate::versioned::Versioned<PicklesTypesDlogPlonkTypesProofStableV1<G, GOpt, Fq, Fqv>, 1i32>;

/// Location: [src/lib/zexe_backend/zexe_backend_common/curve.ml:97:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/zexe_backend/zexe_backend_common/curve.ml#L97)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesTypesDlogPlonkTypesProofStableArg0(
    pub crate::bigint::BigInt,
    pub crate::bigint::BigInt,
);

/// Location: [src/lib/pickles_types/or_infinity.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/or_infinity.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum PicklesTypesOrInfinityStableV1<A> {
    Infinity,
    Finite(A),
}

/// **Origin**: `Pickles_types__Or_infinity.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_types/or_infinity.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/or_infinity.ml#L6)
pub type PicklesTypesOrInfinityStable<A> =
    crate::versioned::Versioned<PicklesTypesOrInfinityStableV1<A>, 1i32>;

/// Location: [src/lib/zexe_backend/zexe_backend_common/plonk_dlog_proof.ml:155:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/zexe_backend/zexe_backend_common/plonk_dlog_proof.ml#L155)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBaseDlogBasedStableV1ProofV1(
    pub  PicklesTypesDlogPlonkTypesProofStable<
        PicklesTypesDlogPlonkTypesProofStableArg0,
        PicklesTypesOrInfinityStable<PicklesTypesDlogPlonkTypesProofStableArg0>,
        crate::bigint::BigInt,
        PicklesTypesDlogPlonkTypesPcArrayStable<crate::bigint::BigInt>,
    >,
);

/// Location: [src/lib/zexe_backend/zexe_backend_common/plonk_dlog_proof.ml:155:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/zexe_backend/zexe_backend_common/plonk_dlog_proof.ml#L155)
pub type PicklesProofBaseDlogBasedStableV1Proof =
    crate::versioned::Versioned<PicklesProofBaseDlogBasedStableV1ProofV1, 1i32>;

/// Location: [src/lib/pickles/proof.ml:45:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L45)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBaseDlogBasedStableV1<DlogMeOnly, PairingMeOnly> {
    pub statement: CompositionTypesDlogBasedStatementMinimalStable<
        PicklesTypesVectorVector2Stable<LimbVectorConstantHex64Stable>,
        PicklesTypesScalarChallengeStable<
            PicklesTypesVectorVector2Stable<LimbVectorConstantHex64Stable>,
        >,
        PicklesTypesShiftedValueStable<crate::bigint::BigInt>,
        crate::bigint::BigInt,
        DlogMeOnly,
        CompositionTypesDigestConstantStable,
        PairingMeOnly,
        PastaBasicRoundsStepVectorStable<
            CompositionTypesBulletproofChallengeStable<
                PicklesTypesScalarChallengeStable<
                    PicklesTypesVectorVector2Stable<LimbVectorConstantHex64Stable>,
                >,
            >,
        >,
        CompositionTypesIndexStable,
    >,
    pub prev_evals: PicklesProofBaseDoubleStable<
        PicklesTypesDlogPlonkTypesEvalsStable<
            PicklesTypesDlogPlonkTypesPcArrayStable<crate::bigint::BigInt>,
        >,
    >,
    pub prev_x_hat: PicklesProofBaseDoubleStable<crate::bigint::BigInt>,
    pub proof: PicklesProofBaseDlogBasedStableV1Proof,
}

/// **Origin**: `Pickles__Proof.Base.Dlog_based.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/proof.ml:45:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L45)
pub type PicklesProofBaseDlogBasedStable<DlogMeOnly, PairingMeOnly> =
    crate::versioned::Versioned<PicklesProofBaseDlogBasedStableV1<DlogMeOnly, PairingMeOnly>, 1i32>;

/// Location: [src/lib/pickles/composition_types/composition_types.ml:275:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L275)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CompositionTypesDlogBasedProofStateMeOnlyStableV1<G1, BulletproofChallenges> {
    pub sg: G1,
    pub old_bulletproof_challenges: BulletproofChallenges,
}

/// **Origin**: `Composition_types.Dlog_based.Proof_state.Me_only.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/composition_types/composition_types.ml:275:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L275)
pub type CompositionTypesDlogBasedProofStateMeOnlyStable<G1, BulletproofChallenges> =
    crate::versioned::Versioned<
        CompositionTypesDlogBasedProofStateMeOnlyStableV1<G1, BulletproofChallenges>,
        1i32,
    >;

/// Location: [src/lib/zexe_backend/zexe_backend_common/curve.ml:97:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/zexe_backend/zexe_backend_common/curve.ml#L97)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CompositionTypesDlogBasedProofStateMeOnlyStableArg0(
    pub crate::bigint::BigInt,
    pub crate::bigint::BigInt,
);

/// Location: [src/lib/pickles_types/vector.ml:532:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L532)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesTypesVectorVector17StableV1<A>(
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

/// **Origin**: `Pickles_types__Vector.Vector_17.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_types/vector.ml:532:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L532)
pub type PicklesTypesVectorVector17Stable<A> =
    crate::versioned::Versioned<PicklesTypesVectorVector17StableV1<A>, 1i32>;

/// Location: [src/lib/zexe_backend/pasta/basic.ml:33:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/zexe_backend/pasta/basic.ml#L33)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PastaBasicRoundsWrapVectorStableV1<A>(pub PicklesTypesVectorVector17Stable<A>);

/// **Origin**: `Pasta__Basic.Rounds.Wrap_vector.Stable.V1.t`
///
/// **Location**: [src/lib/zexe_backend/pasta/basic.ml:33:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/zexe_backend/pasta/basic.ml#L33)
pub type PastaBasicRoundsWrapVectorStable<A> =
    crate::versioned::Versioned<PastaBasicRoundsWrapVectorStableV1<A>, 1i32>;

/// Location: [src/lib/pickles/reduced_me_only.ml:38:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/reduced_me_only.ml#L38)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1(
    pub  PastaBasicRoundsWrapVectorStable<
        CompositionTypesBulletproofChallengeStable<
            PicklesTypesScalarChallengeStable<
                PicklesTypesVectorVector2Stable<LimbVectorConstantHex64Stable>,
            >,
        >,
    >,
);

/// **Origin**: `Pickles__Reduced_me_only.Dlog_based.Challenges_vector.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/reduced_me_only.ml:38:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/reduced_me_only.ml#L38)
pub type PicklesReducedMeOnlyDlogBasedChallengesVectorStable =
    crate::versioned::Versioned<PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1, 1i32>;

/// Location: [src/lib/pickles/reduced_me_only.ml:16:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/reduced_me_only.ml#L16)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesReducedMeOnlyPairingBasedStableV1<S, Sgs, Bpcs> {
    pub app_state: S,
    pub sg: Sgs,
    pub old_bulletproof_challenges: Bpcs,
}

/// **Origin**: `Pickles__Reduced_me_only.Pairing_based.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/reduced_me_only.ml:16:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/reduced_me_only.ml#L16)
pub type PicklesReducedMeOnlyPairingBasedStable<S, Sgs, Bpcs> =
    crate::versioned::Versioned<PicklesReducedMeOnlyPairingBasedStableV1<S, Sgs, Bpcs>, 1i32>;

/// Location: [src/lib/pickles_types/at_most.ml:106:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/at_most.ml#L106)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesTypesAtMostAtMost2StableV1<A>(pub Vec<A>);

/// **Origin**: `Pickles_types__At_most.At_most_2.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_types/at_most.ml:106:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/at_most.ml#L106)
pub type PicklesTypesAtMostAtMost2Stable<A> =
    crate::versioned::Versioned<PicklesTypesAtMostAtMost2StableV1<A>, 1i32>;

/// Location: [src/lib/pickles/proof.ml:283:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L283)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1(
    pub  PicklesProofBaseDlogBasedStable<
        CompositionTypesDlogBasedProofStateMeOnlyStable<
            CompositionTypesDlogBasedProofStateMeOnlyStableArg0,
            PicklesTypesVectorVector2Stable<PicklesReducedMeOnlyDlogBasedChallengesVectorStable>,
        >,
        PicklesReducedMeOnlyPairingBasedStable<
            (),
            PicklesTypesAtMostAtMost2Stable<PicklesTypesDlogPlonkTypesProofStableArg0>,
            PicklesTypesAtMostAtMost2Stable<
                PastaBasicRoundsStepVectorStable<
                    CompositionTypesBulletproofChallengeStable<
                        PicklesTypesScalarChallengeStable<
                            PicklesTypesVectorVector2Stable<LimbVectorConstantHex64Stable>,
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
pub type PicklesProofBranching2ReprStable =
    crate::versioned::Versioned<PicklesProofBranching2ReprStableV1, 1i32>;

/// Location: [src/lib/pickles/proof.ml:318:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L318)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2StableV1(pub PicklesProofBranching2ReprStable);

/// **Origin**: `Pickles__Proof.Branching_2.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/proof.ml:318:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L318)
pub type PicklesProofBranching2Stable =
    crate::versioned::Versioned<PicklesProofBranching2StableV1, 1i32>;

/// Location: [src/lib/mina_base/proof.ml:12:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/proof.ml#L12)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseProofStableV1(pub PicklesProofBranching2Stable);

/// **Origin**: `Mina_base__Proof.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/proof.ml:12:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/proof.ml#L12)
pub type MinaBaseProofStable = crate::versioned::Versioned<MinaBaseProofStableV1, 1i32>;

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L10)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum StagedLedgerDiffAtMostTwoStableV1<A> {
    Zero,
    One(Option<A>),
    Two(Option<(A, Option<A>)>),
}

/// **Origin**: `Staged_ledger_diff.At_most_two.Stable.V1.t`
///
/// **Location**: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L10)
pub type StagedLedgerDiffAtMostTwoStable<A> =
    crate::versioned::Versioned<StagedLedgerDiffAtMostTwoStableV1<A>, 1i32>;

/// Location: [src/lib/currency/currency.ml:586:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/currency.ml#L586)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CurrencyFeeStableV1(pub UnsignedExtendedUInt64Stable);

/// **Origin**: `Currency.Fee.Stable.V1.t`
///
/// **Location**: [src/lib/currency/currency.ml:586:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/currency.ml#L586)
pub type CurrencyFeeStable = crate::versioned::Versioned<CurrencyFeeStableV1, 1i32>;

/// Location: [src/lib/mina_base/coinbase_fee_transfer.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/coinbase_fee_transfer.ml#L7)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseCoinbaseFeeTransferStableV1 {
    pub receiver_pk: ConsensusProofOfStakeDataConsensusStatePolyStableArg8,
    pub fee: CurrencyFeeStable,
}

/// **Origin**: `Mina_base__Coinbase_fee_transfer.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/coinbase_fee_transfer.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/coinbase_fee_transfer.ml#L7)
pub type MinaBaseCoinbaseFeeTransferStable =
    crate::versioned::Versioned<MinaBaseCoinbaseFeeTransferStableV1, 1i32>;

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:66:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L66)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffFtStableV1(pub MinaBaseCoinbaseFeeTransferStable);

/// **Origin**: `Staged_ledger_diff.Ft.Stable.V1.t`
///
/// **Location**: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:66:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L66)
pub type StagedLedgerDiffFtStable = crate::versioned::Versioned<StagedLedgerDiffFtStableV1, 1i32>;

/// Location: [src/lib/currency/currency.ml:750:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/currency.ml#L750)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CurrencyBalanceStableV1(pub CurrencyAmountMakeStrStable);

/// **Origin**: `Currency.Balance.Stable.V1.t`
///
/// **Location**: [src/lib/currency/currency.ml:750:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/currency.ml#L750)
pub type CurrencyBalanceStable = crate::versioned::Versioned<CurrencyBalanceStableV1, 1i32>;

/// Location: [src/lib/mina_base/transaction_status.ml:711:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L711)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseTransactionStatusCoinbaseBalanceDataStableV1 {
    pub coinbase_receiver_balance: CurrencyBalanceStable,
    pub fee_transfer_receiver_balance: Option<CurrencyBalanceStable>,
}

/// **Origin**: `Mina_base__Transaction_status.Coinbase_balance_data.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_status.ml:711:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L711)
pub type MinaBaseTransactionStatusCoinbaseBalanceDataStable =
    crate::versioned::Versioned<MinaBaseTransactionStatusCoinbaseBalanceDataStableV1, 1i32>;

/// Location: [src/lib/mina_base/transaction_status.ml:754:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L754)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseTransactionStatusFeeTransferBalanceDataStableV1 {
    pub receiver1_balance: CurrencyBalanceStable,
    pub receiver2_balance: Option<CurrencyBalanceStable>,
}

/// **Origin**: `Mina_base__Transaction_status.Fee_transfer_balance_data.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_status.ml:754:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L754)
pub type MinaBaseTransactionStatusFeeTransferBalanceDataStable =
    crate::versioned::Versioned<MinaBaseTransactionStatusFeeTransferBalanceDataStableV1, 1i32>;

/// Location: [src/lib/mina_base/transaction_status.ml:795:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L795)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseTransactionStatusInternalCommandBalanceDataStableV1 {
    Coinbase(MinaBaseTransactionStatusCoinbaseBalanceDataStable),
    FeeTransfer(MinaBaseTransactionStatusFeeTransferBalanceDataStable),
}

/// **Origin**: `Mina_base__Transaction_status.Internal_command_balance_data.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_status.ml:795:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L795)
pub type MinaBaseTransactionStatusInternalCommandBalanceDataStable =
    crate::versioned::Versioned<MinaBaseTransactionStatusInternalCommandBalanceDataStableV1, 1i32>;

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:82:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L82)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffPreDiffTwoStableV1<A, B> {
    pub completed_works: Vec<A>,
    pub commands: Vec<B>,
    pub coinbase: StagedLedgerDiffAtMostTwoStable<StagedLedgerDiffFtStable>,
    pub internal_command_balances: Vec<MinaBaseTransactionStatusInternalCommandBalanceDataStable>,
}

/// **Origin**: `Staged_ledger_diff.Pre_diff_two.Stable.V1.t`
///
/// **Location**: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:82:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L82)
pub type StagedLedgerDiffPreDiffTwoStable<A, B> =
    crate::versioned::Versioned<StagedLedgerDiffPreDiffTwoStableV1<A, B>, 1i32>;

/// Location: [src/lib/one_or_two/one_or_two.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/one_or_two/one_or_two.ml#L7)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[polymorphic_variant]
pub enum OneOrTwoStableV1<A> {
    One(A),
    Two((A, A)),
}

/// **Origin**: `One_or_two.Stable.V1.t`
///
/// **Location**: [src/lib/one_or_two/one_or_two.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/one_or_two/one_or_two.ml#L7)
pub type OneOrTwoStable<A> = crate::versioned::Versioned<OneOrTwoStableV1<A>, 1i32>;

/// Location: [src/lib/transaction_snark/transaction_snark.ml:117:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L117)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkStatementPolyStableV1<
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

/// **Origin**: `Transaction_snark.Statement.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark/transaction_snark.ml:117:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L117)
pub type TransactionSnarkStatementPolyStable<
    LedgerHash,
    Amount,
    PendingCoinbase,
    FeeExcess,
    TokenId,
    SokDigest,
> = crate::versioned::Versioned<
    TransactionSnarkStatementPolyStableV1<
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkPendingCoinbaseStackStatePolyStableV1<PendingCoinbase> {
    pub source: PendingCoinbase,
    pub target: PendingCoinbase,
}

/// **Origin**: `Transaction_snark.Pending_coinbase_stack_state.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark/transaction_snark.ml:63:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L63)
pub type TransactionSnarkPendingCoinbaseStackStatePolyStable<PendingCoinbase> =
    crate::versioned::Versioned<
        TransactionSnarkPendingCoinbaseStackStatePolyStableV1<PendingCoinbase>,
        1i32,
    >;

/// Location: [src/lib/mina_base/pending_coinbase.ml:494:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L494)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStackVersionedPolyStableV1<DataStack, StateStack> {
    pub data: DataStack,
    pub state: StateStack,
}

/// **Origin**: `Mina_base__Pending_coinbase.Stack_versioned.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/pending_coinbase.ml:494:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L494)
pub type MinaBasePendingCoinbaseStackVersionedPolyStable<DataStack, StateStack> =
    crate::versioned::Versioned<
        MinaBasePendingCoinbaseStackVersionedPolyStableV1<DataStack, StateStack>,
        1i32,
    >;

/// Location: [src/lib/mina_base/pending_coinbase.ml:154:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L154)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStackVersionedPolyStableArg0V1Poly(pub crate::bigint::BigInt);

/// Location: [src/lib/mina_base/pending_coinbase.ml:153:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L153)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStackVersionedPolyStableArg0V1(
    pub MinaBasePendingCoinbaseStackVersionedPolyStableArg0V1Poly,
);

/// Location: [src/lib/mina_base/pending_coinbase.ml:153:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L153)
pub type MinaBasePendingCoinbaseStackVersionedPolyStableArg0 =
    crate::versioned::Versioned<MinaBasePendingCoinbaseStackVersionedPolyStableArg0V1, 1i32>;

/// Location: [src/lib/mina_base/pending_coinbase.ml:238:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L238)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStateStackPolyStableV1<StackHash> {
    pub init: StackHash,
    pub curr: StackHash,
}

/// **Origin**: `Mina_base__Pending_coinbase.State_stack.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/pending_coinbase.ml:238:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L238)
pub type MinaBasePendingCoinbaseStateStackPolyStable<StackHash> =
    crate::versioned::Versioned<MinaBasePendingCoinbaseStateStackPolyStableV1<StackHash>, 1i32>;

/// Location: [src/lib/mina_base/pending_coinbase.ml:213:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L213)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStateStackPolyStableArg0V1Poly(pub crate::bigint::BigInt);

/// Location: [src/lib/mina_base/pending_coinbase.ml:212:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L212)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStateStackPolyStableArg0V1(
    pub MinaBasePendingCoinbaseStateStackPolyStableArg0V1Poly,
);

/// Location: [src/lib/mina_base/pending_coinbase.ml:212:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L212)
pub type MinaBasePendingCoinbaseStateStackPolyStableArg0 =
    crate::versioned::Versioned<MinaBasePendingCoinbaseStateStackPolyStableArg0V1, 1i32>;

/// Location: [src/lib/mina_base/pending_coinbase.ml:247:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L247)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStateStackStableV1(
    pub MinaBasePendingCoinbaseStateStackPolyStable<MinaBasePendingCoinbaseStateStackPolyStableArg0>,
);

/// **Origin**: `Mina_base__Pending_coinbase.State_stack.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/pending_coinbase.ml:247:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L247)
pub type MinaBasePendingCoinbaseStateStackStable =
    crate::versioned::Versioned<MinaBasePendingCoinbaseStateStackStableV1, 1i32>;

/// Location: [src/lib/mina_base/pending_coinbase.ml:504:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L504)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStackVersionedStableV1(
    pub  MinaBasePendingCoinbaseStackVersionedPolyStable<
        MinaBasePendingCoinbaseStackVersionedPolyStableArg0,
        MinaBasePendingCoinbaseStateStackStable,
    >,
);

/// **Origin**: `Mina_base__Pending_coinbase.Stack_versioned.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/pending_coinbase.ml:504:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L504)
pub type MinaBasePendingCoinbaseStackVersionedStable =
    crate::versioned::Versioned<MinaBasePendingCoinbaseStackVersionedStableV1, 1i32>;

/// Location: [src/lib/transaction_snark/transaction_snark.ml:87:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L87)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkPendingCoinbaseStackStateStableV1(
    pub  TransactionSnarkPendingCoinbaseStackStatePolyStable<
        MinaBasePendingCoinbaseStackVersionedStable,
    >,
);

/// **Origin**: `Transaction_snark.Pending_coinbase_stack_state.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark/transaction_snark.ml:87:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L87)
pub type TransactionSnarkPendingCoinbaseStackStateStable =
    crate::versioned::Versioned<TransactionSnarkPendingCoinbaseStackStateStableV1, 1i32>;

/// Location: [src/lib/mina_base/fee_excess.ml:54:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_excess.ml#L54)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeExcessPolyStableV1<Token, Fee> {
    pub fee_token_l: Token,
    pub fee_excess_l: Fee,
    pub fee_token_r: Token,
    pub fee_excess_r: Fee,
}

/// **Origin**: `Mina_base__Fee_excess.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/fee_excess.ml:54:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_excess.ml#L54)
pub type MinaBaseFeeExcessPolyStable<Token, Fee> =
    crate::versioned::Versioned<MinaBaseFeeExcessPolyStableV1<Token, Fee>, 1i32>;

/// Location: [src/lib/currency/signed_poly.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/signed_poly.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CurrencySignedPolyStableV1<Magnitude, Sgn> {
    pub magnitude: Magnitude,
    pub sgn: Sgn,
}

/// **Origin**: `Currency__Signed_poly.Stable.V1.t`
///
/// **Location**: [src/lib/currency/signed_poly.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/signed_poly.ml#L6)
pub type CurrencySignedPolyStable<Magnitude, Sgn> =
    crate::versioned::Versioned<CurrencySignedPolyStableV1<Magnitude, Sgn>, 1i32>;

/// Location: [src/lib/sgn/sgn.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/sgn/sgn.ml#L9)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum SgnStableV1 {
    Pos,
    Neg,
}

/// **Origin**: `Sgn.Stable.V1.t`
///
/// **Location**: [src/lib/sgn/sgn.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/sgn/sgn.ml#L9)
pub type SgnStable = crate::versioned::Versioned<SgnStableV1, 1i32>;

/// Location: [src/lib/mina_base/fee_excess.ml:123:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_excess.ml#L123)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeExcessStableV1(
    pub  MinaBaseFeeExcessPolyStable<
        MinaBaseTokenIdStable,
        CurrencySignedPolyStable<CurrencyFeeStable, SgnStable>,
    >,
);

/// **Origin**: `Mina_base__Fee_excess.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/fee_excess.ml:123:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_excess.ml#L123)
pub type MinaBaseFeeExcessStable = crate::versioned::Versioned<MinaBaseFeeExcessStableV1, 1i32>;

/// Location: [src/lib/mina_base/sok_message.ml:25:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/sok_message.ml#L25)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSokMessageDigestStableV1(pub crate::string::String);

/// **Origin**: `Mina_base__Sok_message.Digest.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/sok_message.ml:25:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/sok_message.ml#L25)
pub type MinaBaseSokMessageDigestStable =
    crate::versioned::Versioned<MinaBaseSokMessageDigestStableV1, 1i32>;

/// Location: [src/lib/transaction_snark/transaction_snark.ml:220:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L220)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkStatementWithSokStableV1(
    pub  TransactionSnarkStatementPolyStable<
        MinaBaseStagedLedgerHashNonSnarkStableV1LedgerHash,
        CurrencyAmountMakeStrStable,
        TransactionSnarkPendingCoinbaseStackStateStable,
        MinaBaseFeeExcessStable,
        MinaBaseTokenIdStable,
        MinaBaseSokMessageDigestStable,
    >,
);

/// **Origin**: `Transaction_snark.Statement.With_sok.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark/transaction_snark.ml:220:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L220)
pub type TransactionSnarkStatementWithSokStable =
    crate::versioned::Versioned<TransactionSnarkStatementWithSokStableV1, 1i32>;

/// Location: [src/lib/transaction_snark/transaction_snark.ml:420:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L420)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkProofStableV1(pub PicklesProofBranching2Stable);

/// **Origin**: `Transaction_snark.Proof.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark/transaction_snark.ml:420:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L420)
pub type TransactionSnarkProofStable =
    crate::versioned::Versioned<TransactionSnarkProofStableV1, 1i32>;

/// Location: [src/lib/transaction_snark/transaction_snark.ml:432:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L432)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkStableV1 {
    pub statement: TransactionSnarkStatementWithSokStable,
    pub proof: TransactionSnarkProofStable,
}

/// **Origin**: `Transaction_snark.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark/transaction_snark.ml:432:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L432)
pub type TransactionSnarkStable = crate::versioned::Versioned<TransactionSnarkStableV1, 1i32>;

/// Location: [src/lib/ledger_proof/ledger_proof.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/ledger_proof/ledger_proof.ml#L10)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct LedgerProofProdStableV1(pub TransactionSnarkStable);

/// **Origin**: `Ledger_proof.Prod.Stable.V1.t`
///
/// **Location**: [src/lib/ledger_proof/ledger_proof.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/ledger_proof/ledger_proof.ml#L10)
pub type LedgerProofProdStable = crate::versioned::Versioned<LedgerProofProdStableV1, 1i32>;

/// Location: [src/lib/transaction_snark_work/transaction_snark_work.ml:82:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_work/transaction_snark_work.ml#L82)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkWorkTStableV1 {
    pub fee: CurrencyFeeStable,
    pub proofs: OneOrTwoStable<LedgerProofProdStable>,
    pub prover: ConsensusProofOfStakeDataConsensusStatePolyStableArg8,
}

/// **Origin**: `Transaction_snark_work.T.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark_work/transaction_snark_work.ml:82:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_work/transaction_snark_work.ml#L82)
pub type TransactionSnarkWorkTStable =
    crate::versioned::Versioned<TransactionSnarkWorkTStableV1, 1i32>;

/// Location: [src/lib/mina_base/transaction_status.ml:809:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L809)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseTransactionStatusAuxiliaryDataStableV1 {
    pub fee_payer_account_creation_fee_paid: Option<CurrencyAmountMakeStrStable>,
    pub receiver_account_creation_fee_paid: Option<CurrencyAmountMakeStrStable>,
    pub created_token: Option<MinaBaseTokenIdStable>,
}

/// **Origin**: `Mina_base__Transaction_status.Auxiliary_data.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_status.ml:809:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L809)
pub type MinaBaseTransactionStatusAuxiliaryDataStable =
    crate::versioned::Versioned<MinaBaseTransactionStatusAuxiliaryDataStableV1, 1i32>;

/// Location: [src/lib/mina_base/transaction_status.ml:692:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L692)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseTransactionStatusBalanceDataStableV1 {
    pub fee_payer_balance: Option<CurrencyBalanceStable>,
    pub source_balance: Option<CurrencyBalanceStable>,
    pub receiver_balance: Option<CurrencyBalanceStable>,
}

/// **Origin**: `Mina_base__Transaction_status.Balance_data.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_status.ml:692:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L692)
pub type MinaBaseTransactionStatusBalanceDataStable =
    crate::versioned::Versioned<MinaBaseTransactionStatusBalanceDataStableV1, 1i32>;

/// Location: [src/lib/mina_base/transaction_status.ml:13:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L13)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseTransactionStatusFailureStableV1 {
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
pub type MinaBaseTransactionStatusFailureStable =
    crate::versioned::Versioned<MinaBaseTransactionStatusFailureStableV1, 1i32>;

/// Location: [src/lib/mina_base/transaction_status.ml:832:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L832)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseTransactionStatusStableV1 {
    Applied(
        MinaBaseTransactionStatusAuxiliaryDataStable,
        MinaBaseTransactionStatusBalanceDataStable,
    ),
    Failed(
        MinaBaseTransactionStatusFailureStable,
        MinaBaseTransactionStatusBalanceDataStable,
    ),
}

/// **Origin**: `Mina_base__Transaction_status.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_status.ml:832:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L832)
pub type MinaBaseTransactionStatusStable =
    crate::versioned::Versioned<MinaBaseTransactionStatusStableV1, 1i32>;

/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/with_status.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseWithStatusStableV1<A> {
    pub data: A,
    pub status: MinaBaseTransactionStatusStable,
}

/// **Origin**: `Mina_base__With_status.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/with_status.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/with_status.ml#L6)
pub type MinaBaseWithStatusStable<A> =
    crate::versioned::Versioned<MinaBaseWithStatusStableV1<A>, 1i32>;

/// Location: [src/lib/mina_base/user_command.ml:7:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/user_command.ml#L7)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseUserCommandPolyStableV1<U, S> {
    SignedCommand(U),
    SnappCommand(S),
}

/// **Origin**: `Mina_base__User_command.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/user_command.ml:7:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/user_command.ml#L7)
pub type MinaBaseUserCommandPolyStable<U, S> =
    crate::versioned::Versioned<MinaBaseUserCommandPolyStableV1<U, S>, 1i32>;

/// Location: [src/lib/mina_base/signed_command.ml:13:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command.ml#L13)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandPolyStableV1<Payload, Pk, Signature> {
    pub payload: Payload,
    pub signer: Pk,
    pub signature: Signature,
}

/// **Origin**: `Mina_base__Signed_command.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command.ml:13:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command.ml#L13)
pub type MinaBaseSignedCommandPolyStable<Payload, Pk, Signature> =
    crate::versioned::Versioned<MinaBaseSignedCommandPolyStableV1<Payload, Pk, Signature>, 1i32>;

/// Location: [src/lib/mina_base/signed_command_payload.ml:410:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L410)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandPayloadPolyStableV1<Common, Body> {
    pub common: Common,
    pub body: Body,
}

/// **Origin**: `Mina_base__Signed_command_payload.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command_payload.ml:410:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L410)
pub type MinaBaseSignedCommandPayloadPolyStable<Common, Body> =
    crate::versioned::Versioned<MinaBaseSignedCommandPayloadPolyStableV1<Common, Body>, 1i32>;

/// Location: [src/lib/mina_base/signed_command_payload.ml:23:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L23)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandPayloadCommonPolyStableV1<
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

/// **Origin**: `Mina_base__Signed_command_payload.Common.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command_payload.ml:23:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L23)
pub type MinaBaseSignedCommandPayloadCommonPolyStable<
    Fee,
    PublicKey,
    TokenId,
    Nonce,
    GlobalSlot,
    Memo,
> = crate::versioned::Versioned<
    MinaBaseSignedCommandPayloadCommonPolyStableV1<
        Fee,
        PublicKey,
        TokenId,
        Nonce,
        GlobalSlot,
        Memo,
    >,
    1i32,
>;

/// Location: [src/lib/mina_numbers/nat.ml:186:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L186)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaNumbersNatMake32StableV1(pub UnsignedExtendedUInt32Stable);

/// **Origin**: `Mina_numbers__Nat.Make32.Stable.V1.t`
///
/// **Location**: [src/lib/mina_numbers/nat.ml:186:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L186)
pub type MinaNumbersNatMake32Stable =
    crate::versioned::Versioned<MinaNumbersNatMake32StableV1, 1i32>;

/// Location: [src/lib/mina_base/signed_command_memo.ml:16:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_memo.ml#L16)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandMemoStableV1(pub crate::string::String);

/// **Origin**: `Mina_base__Signed_command_memo.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command_memo.ml:16:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_memo.ml#L16)
pub type MinaBaseSignedCommandMemoStable =
    crate::versioned::Versioned<MinaBaseSignedCommandMemoStableV1, 1i32>;

/// Location: [src/lib/mina_base/signed_command_payload.ml:61:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L61)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandPayloadCommonBinableArgStableV1(
    pub  MinaBaseSignedCommandPayloadCommonPolyStable<
        CurrencyFeeStable,
        ConsensusProofOfStakeDataConsensusStatePolyStableArg8,
        MinaBaseTokenIdStable,
        MinaNumbersNatMake32Stable,
        ConsensusGlobalSlotPolyStableArg0,
        MinaBaseSignedCommandMemoStable,
    >,
);

/// **Origin**: `Mina_base__Signed_command_payload.Common.Binable_arg.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command_payload.ml:61:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L61)
pub type MinaBaseSignedCommandPayloadCommonBinableArgStable =
    crate::versioned::Versioned<MinaBaseSignedCommandPayloadCommonBinableArgStableV1, 1i32>;

/// Location: [src/lib/mina_base/signed_command_payload.ml:87:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L87)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandPayloadCommonStableV1(
    pub MinaBaseSignedCommandPayloadCommonBinableArgStable,
);

/// **Origin**: `Mina_base__Signed_command_payload.Common.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command_payload.ml:87:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L87)
pub type MinaBaseSignedCommandPayloadCommonStable =
    crate::versioned::Versioned<MinaBaseSignedCommandPayloadCommonStableV1, 1i32>;

/// Location: [src/lib/mina_base/payment_payload.ml:21:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/payment_payload.ml#L21)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePaymentPayloadPolyStableV1<PublicKey, TokenId, Amount> {
    pub source_pk: PublicKey,
    pub receiver_pk: PublicKey,
    pub token_id: TokenId,
    pub amount: Amount,
}

/// **Origin**: `Mina_base__Payment_payload.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/payment_payload.ml:21:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/payment_payload.ml#L21)
pub type MinaBasePaymentPayloadPolyStable<PublicKey, TokenId, Amount> = crate::versioned::Versioned<
    MinaBasePaymentPayloadPolyStableV1<PublicKey, TokenId, Amount>,
    1i32,
>;

/// Location: [src/lib/mina_base/payment_payload.ml:35:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/payment_payload.ml#L35)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePaymentPayloadStableV1(
    pub  MinaBasePaymentPayloadPolyStable<
        ConsensusProofOfStakeDataConsensusStatePolyStableArg8,
        MinaBaseTokenIdStable,
        CurrencyAmountMakeStrStable,
    >,
);

/// **Origin**: `Mina_base__Payment_payload.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/payment_payload.ml:35:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/payment_payload.ml#L35)
pub type MinaBasePaymentPayloadStable =
    crate::versioned::Versioned<MinaBasePaymentPayloadStableV1, 1i32>;

/// Location: [src/lib/mina_base/stake_delegation.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/stake_delegation.ml#L9)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseStakeDelegationStableV1 {
    SetDelegate {
        delegator: ConsensusProofOfStakeDataConsensusStatePolyStableArg8,
        new_delegate: ConsensusProofOfStakeDataConsensusStatePolyStableArg8,
    },
}

/// **Origin**: `Mina_base__Stake_delegation.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/stake_delegation.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/stake_delegation.ml#L9)
pub type MinaBaseStakeDelegationStable =
    crate::versioned::Versioned<MinaBaseStakeDelegationStableV1, 1i32>;

/// Location: [src/lib/mina_base/new_token_payload.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/new_token_payload.ml#L7)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseNewTokenPayloadStableV1 {
    pub token_owner_pk: ConsensusProofOfStakeDataConsensusStatePolyStableArg8,
    pub disable_new_accounts: bool,
}

/// **Origin**: `Mina_base__New_token_payload.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/new_token_payload.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/new_token_payload.ml#L7)
pub type MinaBaseNewTokenPayloadStable =
    crate::versioned::Versioned<MinaBaseNewTokenPayloadStableV1, 1i32>;

/// Location: [src/lib/mina_base/new_account_payload.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/new_account_payload.ml#L7)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseNewAccountPayloadStableV1 {
    pub token_id: MinaBaseTokenIdStable,
    pub token_owner_pk: ConsensusProofOfStakeDataConsensusStatePolyStableArg8,
    pub receiver_pk: ConsensusProofOfStakeDataConsensusStatePolyStableArg8,
    pub account_disabled: bool,
}

/// **Origin**: `Mina_base__New_account_payload.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/new_account_payload.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/new_account_payload.ml#L7)
pub type MinaBaseNewAccountPayloadStable =
    crate::versioned::Versioned<MinaBaseNewAccountPayloadStableV1, 1i32>;

/// Location: [src/lib/mina_base/minting_payload.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/minting_payload.ml#L7)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseMintingPayloadStableV1 {
    pub token_id: MinaBaseTokenIdStable,
    pub token_owner_pk: ConsensusProofOfStakeDataConsensusStatePolyStableArg8,
    pub receiver_pk: ConsensusProofOfStakeDataConsensusStatePolyStableArg8,
    pub amount: CurrencyAmountMakeStrStable,
}

/// **Origin**: `Mina_base__Minting_payload.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/minting_payload.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/minting_payload.ml#L7)
pub type MinaBaseMintingPayloadStable =
    crate::versioned::Versioned<MinaBaseMintingPayloadStableV1, 1i32>;

/// Location: [src/lib/mina_base/signed_command_payload.ml:205:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L205)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSignedCommandPayloadBodyBinableArgStableV1 {
    Payment(MinaBasePaymentPayloadStable),
    StakeDelegation(MinaBaseStakeDelegationStable),
    CreateNewToken(MinaBaseNewTokenPayloadStable),
    CreateTokenAccount(MinaBaseNewAccountPayloadStable),
    MintTokens(MinaBaseMintingPayloadStable),
}

/// **Origin**: `Mina_base__Signed_command_payload.Body.Binable_arg.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command_payload.ml:205:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L205)
pub type MinaBaseSignedCommandPayloadBodyBinableArgStable =
    crate::versioned::Versioned<MinaBaseSignedCommandPayloadBodyBinableArgStableV1, 1i32>;

/// Location: [src/lib/mina_base/signed_command_payload.ml:252:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L252)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandPayloadBodyStableV1(
    pub MinaBaseSignedCommandPayloadBodyBinableArgStable,
);

/// **Origin**: `Mina_base__Signed_command_payload.Body.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command_payload.ml:252:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L252)
pub type MinaBaseSignedCommandPayloadBodyStable =
    crate::versioned::Versioned<MinaBaseSignedCommandPayloadBodyStableV1, 1i32>;

/// Location: [src/lib/mina_base/signed_command_payload.ml:424:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L424)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandPayloadStableV1(
    pub  MinaBaseSignedCommandPayloadPolyStable<
        MinaBaseSignedCommandPayloadCommonStable,
        MinaBaseSignedCommandPayloadBodyStable,
    >,
);

/// **Origin**: `Mina_base__Signed_command_payload.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command_payload.ml:424:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L424)
pub type MinaBaseSignedCommandPayloadStable =
    crate::versioned::Versioned<MinaBaseSignedCommandPayloadStableV1, 1i32>;

/// Location: [src/lib/non_zero_curve_point/non_zero_curve_point.ml:194:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/non_zero_curve_point.ml#L194)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NonZeroCurvePointUncompressedStableV1(
    pub ConsensusProofOfStakeDataConsensusStatePolyStableArg8,
);

/// **Origin**: `Non_zero_curve_point.Uncompressed.Stable.V1.t`
///
/// **Location**: [src/lib/non_zero_curve_point/non_zero_curve_point.ml:194:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/non_zero_curve_point.ml#L194)
pub type NonZeroCurvePointUncompressedStable =
    crate::versioned::Versioned<NonZeroCurvePointUncompressedStableV1, 1i32>;

/// Location: [src/lib/mina_base/signature_poly.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signature_poly.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignaturePolyStableV1<Field, Scalar>(pub Field, pub Scalar);

/// **Origin**: `Mina_base__Signature_poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signature_poly.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signature_poly.ml#L6)
pub type MinaBaseSignaturePolyStable<Field, Scalar> =
    crate::versioned::Versioned<MinaBaseSignaturePolyStableV1<Field, Scalar>, 1i32>;

/// Location: [src/lib/mina_base/signature.ml:18:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signature.ml#L18)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignatureStableV1(
    pub MinaBaseSignaturePolyStable<crate::bigint::BigInt, crate::bigint::BigInt>,
);

/// **Origin**: `Mina_base__Signature.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signature.ml:18:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signature.ml#L18)
pub type MinaBaseSignatureStable = crate::versioned::Versioned<MinaBaseSignatureStableV1, 1i32>;

/// Location: [src/lib/mina_base/signed_command.ml:23:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command.ml#L23)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandStableV1(
    pub  MinaBaseSignedCommandPolyStable<
        MinaBaseSignedCommandPayloadStable,
        NonZeroCurvePointUncompressedStable,
        MinaBaseSignatureStable,
    >,
);

/// **Origin**: `Mina_base__Signed_command.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command.ml:23:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command.ml#L23)
pub type MinaBaseSignedCommandStable =
    crate::versioned::Versioned<MinaBaseSignedCommandStableV1, 1i32>;

/// Location: [src/lib/mina_base/other_fee_payer.ml:10:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/other_fee_payer.ml#L10)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseOtherFeePayerPayloadPolyStableV1<Pk, TokenId, Nonce, Fee> {
    pub pk: Pk,
    pub token_id: TokenId,
    pub nonce: Nonce,
    pub fee: Fee,
}

/// **Origin**: `Mina_base__Other_fee_payer.Payload.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/other_fee_payer.ml:10:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/other_fee_payer.ml#L10)
pub type MinaBaseOtherFeePayerPayloadPolyStable<Pk, TokenId, Nonce, Fee> =
    crate::versioned::Versioned<
        MinaBaseOtherFeePayerPayloadPolyStableV1<Pk, TokenId, Nonce, Fee>,
        1i32,
    >;

/// Location: [src/lib/mina_base/other_fee_payer.ml:20:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/other_fee_payer.ml#L20)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseOtherFeePayerPayloadStableV1(
    pub  MinaBaseOtherFeePayerPayloadPolyStable<
        ConsensusProofOfStakeDataConsensusStatePolyStableArg8,
        MinaBaseTokenIdStable,
        MinaNumbersNatMake32Stable,
        CurrencyFeeStable,
    >,
);

/// **Origin**: `Mina_base__Other_fee_payer.Payload.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/other_fee_payer.ml:20:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/other_fee_payer.ml#L20)
pub type MinaBaseOtherFeePayerPayloadStable =
    crate::versioned::Versioned<MinaBaseOtherFeePayerPayloadStableV1, 1i32>;

/// Location: [src/lib/mina_base/other_fee_payer.ml:84:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/other_fee_payer.ml#L84)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseOtherFeePayerStableV1 {
    pub payload: MinaBaseOtherFeePayerPayloadStable,
    pub signature: MinaBaseSignatureStable,
}

/// **Origin**: `Mina_base__Other_fee_payer.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/other_fee_payer.ml:84:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/other_fee_payer.ml#L84)
pub type MinaBaseOtherFeePayerStable =
    crate::versioned::Versioned<MinaBaseOtherFeePayerStableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:352:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L352)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandInnerStableV1<One, Two> {
    pub token_id: MinaBaseTokenIdStable,
    pub fee_payment: Option<MinaBaseOtherFeePayerStable>,
    pub one: One,
    pub two: Two,
}

/// **Origin**: `Mina_base__Snapp_command.Inner.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:352:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L352)
pub type MinaBaseSnappCommandInnerStable<One, Two> =
    crate::versioned::Versioned<MinaBaseSnappCommandInnerStableV1<One, Two>, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:298:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L298)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyAuthorizedPolyStableV1<Data, Auth> {
    pub data: Data,
    pub authorization: Auth,
}

/// **Origin**: `Mina_base__Snapp_command.Party.Authorized.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:298:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L298)
pub type MinaBaseSnappCommandPartyAuthorizedPolyStable<Data, Auth> =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyAuthorizedPolyStableV1<Data, Auth>, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:211:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L211)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyPredicatedPolyStableV1<Body, Predicate> {
    pub body: Body,
    pub predicate: Predicate,
}

/// **Origin**: `Mina_base__Snapp_command.Party.Predicated.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:211:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L211)
pub type MinaBaseSnappCommandPartyPredicatedPolyStable<Body, Predicate> =
    crate::versioned::Versioned<
        MinaBaseSnappCommandPartyPredicatedPolyStableV1<Body, Predicate>,
        1i32,
    >;

/// Location: [src/lib/mina_base/snapp_command.ml:136:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L136)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyBodyPolyStableV1<Pk, Update, SignedAmount> {
    pub pk: Pk,
    pub update: Update,
    pub delta: SignedAmount,
}

/// **Origin**: `Mina_base__Snapp_command.Party.Body.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:136:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L136)
pub type MinaBaseSnappCommandPartyBodyPolyStable<Pk, Update, SignedAmount> =
    crate::versioned::Versioned<
        MinaBaseSnappCommandPartyBodyPolyStableV1<Pk, Update, SignedAmount>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/vector.ml:503:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L503)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesTypesVectorVector8StableV1<A>(pub A, pub (A, (A, (A, (A, (A, (A, (A, ()))))))));

/// **Origin**: `Pickles_types__Vector.Vector_8.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_types/vector.ml:503:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L503)
pub type PicklesTypesVectorVector8Stable<A> =
    crate::versioned::Versioned<PicklesTypesVectorVector8StableV1<A>, 1i32>;

/// Location: [src/lib/mina_base/snapp_state.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_state.ml#L17)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappStateVStableV1<A>(pub PicklesTypesVectorVector8Stable<A>);

/// **Origin**: `Mina_base__Snapp_state.V.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_state.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_state.ml#L17)
pub type MinaBaseSnappStateVStable<A> =
    crate::versioned::Versioned<MinaBaseSnappStateVStableV1<A>, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:35:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L35)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyUpdatePolyStableV1<StateElement, Pk, Vk, Perms> {
    pub app_state: MinaBaseSnappStateVStable<StateElement>,
    pub delegate: Pk,
    pub verification_key: Vk,
    pub permissions: Perms,
}

/// **Origin**: `Mina_base__Snapp_command.Party.Update.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:35:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L35)
pub type MinaBaseSnappCommandPartyUpdatePolyStable<StateElement, Pk, Vk, Perms> =
    crate::versioned::Versioned<
        MinaBaseSnappCommandPartyUpdatePolyStableV1<StateElement, Pk, Vk, Perms>,
        1i32,
    >;

/// Location: [src/lib/mina_base/snapp_basic.ml:87:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_basic.ml#L87)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSnappBasicSetOrKeepStableV1<A> {
    Set(A),
    Keep,
}

/// **Origin**: `Mina_base__Snapp_basic.Set_or_keep.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_basic.ml:87:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_basic.ml#L87)
pub type MinaBaseSnappBasicSetOrKeepStable<A> =
    crate::versioned::Versioned<MinaBaseSnappBasicSetOrKeepStableV1<A>, 1i32>;

/// Location: [src/lib/with_hash/with_hash.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/with_hash/with_hash.ml#L8)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct WithHashStableV1<A, H> {
    pub data: A,
    pub hash: H,
}

/// **Origin**: `With_hash.Stable.V1.t`
///
/// **Location**: [src/lib/with_hash/with_hash.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/with_hash/with_hash.ml#L8)
pub type WithHashStable<A, H> = crate::versioned::Versioned<WithHashStableV1<A, H>, 1i32>;

/// Location: [src/lib/pickles_types/at_most.ml:135:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/at_most.ml#L135)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesTypesAtMostAtMost8StableV1<A>(pub Vec<A>);

/// **Origin**: `Pickles_types__At_most.At_most_8.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_types/at_most.ml:135:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/at_most.ml#L135)
pub type PicklesTypesAtMostAtMost8Stable<A> =
    crate::versioned::Versioned<PicklesTypesAtMostAtMost8StableV1<A>, 1i32>;

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:120:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L120)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesBaseSideLoadedVerificationKeyMaxBranchesVecStableV1<A>(
    pub PicklesTypesAtMostAtMost8Stable<A>,
);

/// **Origin**: `Pickles_base__Side_loaded_verification_key.Max_branches_vec.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_base/side_loaded_verification_key.ml:120:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L120)
pub type PicklesBaseSideLoadedVerificationKeyMaxBranchesVecStable<A> = crate::versioned::Versioned<
    PicklesBaseSideLoadedVerificationKeyMaxBranchesVecStableV1<A>,
    1i32,
>;

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:136:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L136)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesBaseSideLoadedVerificationKeyDomainsStableV1<A> {
    pub h: A,
}

/// **Origin**: `Pickles_base__Side_loaded_verification_key.Domains.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_base/side_loaded_verification_key.ml:136:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L136)
pub type PicklesBaseSideLoadedVerificationKeyDomainsStable<A> =
    crate::versioned::Versioned<PicklesBaseSideLoadedVerificationKeyDomainsStableV1<A>, 1i32>;

/// Location: [src/lib/pickles_base/domain.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/domain.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum PicklesBaseDomainStableV1 {
    Pow2RootsOfUnity(i32),
}

/// **Origin**: `Pickles_base__Domain.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_base/domain.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/domain.ml#L6)
pub type PicklesBaseDomainStable = crate::versioned::Versioned<PicklesBaseDomainStableV1, 1i32>;

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:44:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L44)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesBaseSideLoadedVerificationKeyWidthStableV1(pub crate::char_::Char);

/// **Origin**: `Pickles_base__Side_loaded_verification_key.Width.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_base/side_loaded_verification_key.ml:44:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L44)
pub type PicklesBaseSideLoadedVerificationKeyWidthStable =
    crate::versioned::Versioned<PicklesBaseSideLoadedVerificationKeyWidthStableV1, 1i32>;

/// Location: [src/lib/pickles_types/plonk_verification_key_evals.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_verification_key_evals.ml#L7)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesTypesPlonkVerificationKeyEvalsStableV1<Comm> {
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

/// **Origin**: `Pickles_types__Plonk_verification_key_evals.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_types/plonk_verification_key_evals.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_verification_key_evals.ml#L7)
pub type PicklesTypesPlonkVerificationKeyEvalsStable<Comm> =
    crate::versioned::Versioned<PicklesTypesPlonkVerificationKeyEvalsStableV1<Comm>, 1i32>;

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:150:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L150)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesBaseSideLoadedVerificationKeyReprStableV1<G> {
    pub step_data: PicklesBaseSideLoadedVerificationKeyMaxBranchesVecStable<(
        PicklesBaseSideLoadedVerificationKeyDomainsStable<PicklesBaseDomainStable>,
        PicklesBaseSideLoadedVerificationKeyWidthStable,
    )>,
    pub max_width: PicklesBaseSideLoadedVerificationKeyWidthStable,
    pub wrap_index: PicklesTypesPlonkVerificationKeyEvalsStable<Vec<G>>,
}

/// **Origin**: `Pickles_base__Side_loaded_verification_key.Repr.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_base/side_loaded_verification_key.ml:150:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L150)
pub type PicklesBaseSideLoadedVerificationKeyReprStable<G> =
    crate::versioned::Versioned<PicklesBaseSideLoadedVerificationKeyReprStableV1<G>, 1i32>;

/// Location: [src/lib/pickles/side_loaded_verification_key.ml:156:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/side_loaded_verification_key.ml#L156)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesSideLoadedVerificationKeyRStableV1(
    pub PicklesBaseSideLoadedVerificationKeyReprStable<PicklesTypesDlogPlonkTypesProofStableArg0>,
);

/// **Origin**: `Pickles__Side_loaded_verification_key.R.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/side_loaded_verification_key.ml:156:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/side_loaded_verification_key.ml#L156)
pub type PicklesSideLoadedVerificationKeyRStable =
    crate::versioned::Versioned<PicklesSideLoadedVerificationKeyRStableV1, 1i32>;

/// Location: [src/lib/pickles/side_loaded_verification_key.ml:166:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/side_loaded_verification_key.ml#L166)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesSideLoadedVerificationKeyStableV1(pub PicklesSideLoadedVerificationKeyRStable);

/// **Origin**: `Pickles__Side_loaded_verification_key.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/side_loaded_verification_key.ml:166:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/side_loaded_verification_key.ml#L166)
pub type PicklesSideLoadedVerificationKeyStable =
    crate::versioned::Versioned<PicklesSideLoadedVerificationKeyStableV1, 1i32>;

/// Location: [src/lib/mina_base/permissions.ml:287:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/permissions.ml#L287)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePermissionsPolyStableV1<Bool, Controller> {
    pub stake: Bool,
    pub edit_state: Controller,
    pub send: Controller,
    pub receive: Controller,
    pub set_delegate: Controller,
    pub set_permissions: Controller,
    pub set_verification_key: Controller,
}

/// **Origin**: `Mina_base__Permissions.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/permissions.ml:287:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/permissions.ml#L287)
pub type MinaBasePermissionsPolyStable<Bool, Controller> =
    crate::versioned::Versioned<MinaBasePermissionsPolyStableV1<Bool, Controller>, 1i32>;

/// Location: [src/lib/mina_base/permissions.ml:52:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/permissions.ml#L52)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBasePermissionsAuthRequiredStableV1 {
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
pub type MinaBasePermissionsAuthRequiredStable =
    crate::versioned::Versioned<MinaBasePermissionsAuthRequiredStableV1, 1i32>;

/// Location: [src/lib/mina_base/permissions.ml:313:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/permissions.ml#L313)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePermissionsStableV1(
    pub MinaBasePermissionsPolyStable<bool, MinaBasePermissionsAuthRequiredStable>,
);

/// **Origin**: `Mina_base__Permissions.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/permissions.ml:313:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/permissions.ml#L313)
pub type MinaBasePermissionsStable = crate::versioned::Versioned<MinaBasePermissionsStableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:51:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L51)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyUpdateStableV1(
    pub  MinaBaseSnappCommandPartyUpdatePolyStable<
        MinaBaseSnappBasicSetOrKeepStable<crate::bigint::BigInt>,
        MinaBaseSnappBasicSetOrKeepStable<ConsensusProofOfStakeDataConsensusStatePolyStableArg8>,
        MinaBaseSnappBasicSetOrKeepStable<
            WithHashStable<PicklesSideLoadedVerificationKeyStable, crate::bigint::BigInt>,
        >,
        MinaBaseSnappBasicSetOrKeepStable<MinaBasePermissionsStable>,
    >,
);

/// **Origin**: `Mina_base__Snapp_command.Party.Update.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:51:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L51)
pub type MinaBaseSnappCommandPartyUpdateStable =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyUpdateStableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:146:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L146)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyBodyStableV1(
    pub  MinaBaseSnappCommandPartyBodyPolyStable<
        ConsensusProofOfStakeDataConsensusStatePolyStableArg8,
        MinaBaseSnappCommandPartyUpdateStable,
        CurrencySignedPolyStable<CurrencyAmountMakeStrStable, SgnStable>,
    >,
);

/// **Origin**: `Mina_base__Snapp_command.Party.Body.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:146:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L146)
pub type MinaBaseSnappCommandPartyBodyStable =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyBodyStableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_predicate.ml:1167:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L1167)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicatePolyStableV1<Account, ProtocolState, Other, Pk> {
    pub self_predicate: Account,
    pub other: Other,
    pub fee_payer: Pk,
    pub protocol_state_predicate: ProtocolState,
}

/// **Origin**: `Mina_base__Snapp_predicate.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_predicate.ml:1167:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L1167)
pub type MinaBaseSnappPredicatePolyStable<Account, ProtocolState, Other, Pk> =
    crate::versioned::Versioned<
        MinaBaseSnappPredicatePolyStableV1<Account, ProtocolState, Other, Pk>,
        1i32,
    >;

/// Location: [src/lib/mina_base/snapp_predicate.ml:353:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L353)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateAccountPolyStableV1<Balance, Nonce, ReceiptChainHash, Pk, Field> {
    pub balance: Balance,
    pub nonce: Nonce,
    pub receipt_chain_hash: ReceiptChainHash,
    pub public_key: Pk,
    pub delegate: Pk,
    pub state: MinaBaseSnappStateVStable<Field>,
}

/// **Origin**: `Mina_base__Snapp_predicate.Account.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_predicate.ml:353:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L353)
pub type MinaBaseSnappPredicateAccountPolyStable<Balance, Nonce, ReceiptChainHash, Pk, Field> =
    crate::versioned::Versioned<
        MinaBaseSnappPredicateAccountPolyStableV1<Balance, Nonce, ReceiptChainHash, Pk, Field>,
        1i32,
    >;

/// Location: [src/lib/mina_base/snapp_basic.ml:158:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_basic.ml#L158)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSnappBasicOrIgnoreStableV1<A> {
    Check(A),
    Ignore,
}

/// **Origin**: `Mina_base__Snapp_basic.Or_ignore.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_basic.ml:158:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_basic.ml#L158)
pub type MinaBaseSnappBasicOrIgnoreStable<A> =
    crate::versioned::Versioned<MinaBaseSnappBasicOrIgnoreStableV1<A>, 1i32>;

/// Location: [src/lib/mina_base/snapp_predicate.ml:23:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L23)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateClosedIntervalStableV1<A> {
    pub lower: A,
    pub upper: A,
}

/// **Origin**: `Mina_base__Snapp_predicate.Closed_interval.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_predicate.ml:23:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L23)
pub type MinaBaseSnappPredicateClosedIntervalStable<A> =
    crate::versioned::Versioned<MinaBaseSnappPredicateClosedIntervalStableV1<A>, 1i32>;

/// Location: [src/lib/mina_base/snapp_predicate.ml:150:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L150)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateNumericStableV1<A>(
    pub MinaBaseSnappBasicOrIgnoreStable<MinaBaseSnappPredicateClosedIntervalStable<A>>,
);

/// **Origin**: `Mina_base__Snapp_predicate.Numeric.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_predicate.ml:150:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L150)
pub type MinaBaseSnappPredicateNumericStable<A> =
    crate::versioned::Versioned<MinaBaseSnappPredicateNumericStableV1<A>, 1i32>;

/// Location: [src/lib/mina_base/receipt.ml:30:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/receipt.ml#L30)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappBasicOrIgnoreStableArg0V1Poly(pub crate::bigint::BigInt);

/// Location: [src/lib/mina_base/receipt.ml:29:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/receipt.ml#L29)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappBasicOrIgnoreStableArg0V1(pub MinaBaseSnappBasicOrIgnoreStableArg0V1Poly);

/// Location: [src/lib/mina_base/receipt.ml:29:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/receipt.ml#L29)
pub type MinaBaseSnappBasicOrIgnoreStableArg0 =
    crate::versioned::Versioned<MinaBaseSnappBasicOrIgnoreStableArg0V1, 1i32>;

/// Location: [src/lib/mina_base/snapp_predicate.ml:369:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L369)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateAccountStableV1(
    pub  MinaBaseSnappPredicateAccountPolyStable<
        MinaBaseSnappPredicateNumericStable<CurrencyBalanceStable>,
        MinaBaseSnappPredicateNumericStable<MinaNumbersNatMake32Stable>,
        MinaBaseSnappBasicOrIgnoreStable<MinaBaseSnappBasicOrIgnoreStableArg0>,
        MinaBaseSnappBasicOrIgnoreStable<ConsensusProofOfStakeDataConsensusStatePolyStableArg8>,
        MinaBaseSnappBasicOrIgnoreStable<crate::bigint::BigInt>,
    >,
);

/// **Origin**: `Mina_base__Snapp_predicate.Account.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_predicate.ml:369:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L369)
pub type MinaBaseSnappPredicateAccountStable =
    crate::versioned::Versioned<MinaBaseSnappPredicateAccountStableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_predicate.ml:603:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L603)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateProtocolStatePolyStableV1<
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

/// **Origin**: `Mina_base__Snapp_predicate.Protocol_state.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_predicate.ml:603:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L603)
pub type MinaBaseSnappPredicateProtocolStatePolyStable<
    SnarkedLedgerHash,
    TokenId,
    Time,
    Length,
    VrfOutput,
    GlobalSlot,
    Amount,
    EpochData,
> = crate::versioned::Versioned<
    MinaBaseSnappPredicateProtocolStatePolyStableV1<
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

/// Location: [src/lib/mina_base/snapp_predicate.ml:535:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L535)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateProtocolStateEpochDataStableV1(
    pub  MinaBaseEpochDataPolyStable<
        MinaBaseEpochLedgerPolyStable<
            MinaBaseSnappBasicOrIgnoreStable<MinaBaseStagedLedgerHashNonSnarkStableV1LedgerHash>,
            MinaBaseSnappPredicateNumericStable<CurrencyAmountMakeStrStable>,
        >,
        MinaBaseSnappBasicOrIgnoreStable<MinaBaseEpochDataPolyStableArg1>,
        MinaBaseSnappBasicOrIgnoreStable<MinaStateProtocolStatePolyStableArg0>,
        MinaBaseSnappBasicOrIgnoreStable<MinaStateProtocolStatePolyStableArg0>,
        MinaBaseSnappPredicateNumericStable<ConsensusProofOfStakeDataConsensusStatePolyStableArg0>,
    >,
);

/// **Origin**: `Mina_base__Snapp_predicate.Protocol_state.Epoch_data.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_predicate.ml:535:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L535)
pub type MinaBaseSnappPredicateProtocolStateEpochDataStable =
    crate::versioned::Versioned<MinaBaseSnappPredicateProtocolStateEpochDataStableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_predicate.ml:644:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L644)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateProtocolStateStableV1(
    pub  MinaBaseSnappPredicateProtocolStatePolyStable<
        MinaBaseSnappBasicOrIgnoreStable<MinaBaseStagedLedgerHashNonSnarkStableV1LedgerHash>,
        MinaBaseSnappPredicateNumericStable<MinaBaseTokenIdStable>,
        MinaBaseSnappPredicateNumericStable<BlockTimeTimeStable>,
        MinaBaseSnappPredicateNumericStable<ConsensusProofOfStakeDataConsensusStatePolyStableArg0>,
        (),
        MinaBaseSnappPredicateNumericStable<ConsensusGlobalSlotPolyStableArg0>,
        MinaBaseSnappPredicateNumericStable<CurrencyAmountMakeStrStable>,
        MinaBaseSnappPredicateProtocolStateEpochDataStable,
    >,
);

/// **Origin**: `Mina_base__Snapp_predicate.Protocol_state.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_predicate.ml:644:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L644)
pub type MinaBaseSnappPredicateProtocolStateStable =
    crate::versioned::Versioned<MinaBaseSnappPredicateProtocolStateStableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_predicate.ml:1100:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L1100)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateOtherPolyStableV1<Account, AccountTransition, Vk> {
    pub predicate: Account,
    pub account_transition: AccountTransition,
    pub account_vk: Vk,
}

/// **Origin**: `Mina_base__Snapp_predicate.Other.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_predicate.ml:1100:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L1100)
pub type MinaBaseSnappPredicateOtherPolyStable<Account, AccountTransition, Vk> =
    crate::versioned::Versioned<
        MinaBaseSnappPredicateOtherPolyStableV1<Account, AccountTransition, Vk>,
        1i32,
    >;

/// Location: [src/lib/mina_base/snapp_basic.ml:21:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_basic.ml#L21)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappBasicTransitionStableV1<A> {
    pub prev: A,
    pub next: A,
}

/// **Origin**: `Mina_base__Snapp_basic.Transition.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_basic.ml:21:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_basic.ml#L21)
pub type MinaBaseSnappBasicTransitionStable<A> =
    crate::versioned::Versioned<MinaBaseSnappBasicTransitionStableV1<A>, 1i32>;

/// Location: [src/lib/mina_base/snapp_basic.ml:234:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_basic.ml#L234)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSnappBasicAccountStateStableV1 {
    Empty,
    NonEmpty,
    Any,
}

/// **Origin**: `Mina_base__Snapp_basic.Account_state.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_basic.ml:234:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_basic.ml#L234)
pub type MinaBaseSnappBasicAccountStateStable =
    crate::versioned::Versioned<MinaBaseSnappBasicAccountStateStableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_predicate.ml:1113:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L1113)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateOtherStableV1(
    pub  MinaBaseSnappPredicateOtherPolyStable<
        MinaBaseSnappPredicateAccountStable,
        MinaBaseSnappBasicTransitionStable<MinaBaseSnappBasicAccountStateStable>,
        MinaBaseSnappBasicOrIgnoreStable<crate::bigint::BigInt>,
    >,
);

/// **Origin**: `Mina_base__Snapp_predicate.Other.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_predicate.ml:1113:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L1113)
pub type MinaBaseSnappPredicateOtherStable =
    crate::versioned::Versioned<MinaBaseSnappPredicateOtherStableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_predicate.ml:1188:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L1188)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateStableV1(
    pub  MinaBaseSnappPredicatePolyStable<
        MinaBaseSnappPredicateAccountStable,
        MinaBaseSnappPredicateProtocolStateStable,
        MinaBaseSnappPredicateOtherStable,
        MinaBaseSnappBasicOrIgnoreStable<ConsensusProofOfStakeDataConsensusStatePolyStableArg8>,
    >,
);

/// **Origin**: `Mina_base__Snapp_predicate.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_predicate.ml:1188:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L1188)
pub type MinaBaseSnappPredicateStable =
    crate::versioned::Versioned<MinaBaseSnappPredicateStableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:225:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L225)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyPredicatedProvedStableV1(
    pub  MinaBaseSnappCommandPartyPredicatedPolyStable<
        MinaBaseSnappCommandPartyBodyStable,
        MinaBaseSnappPredicateStable,
    >,
);

/// **Origin**: `Mina_base__Snapp_command.Party.Predicated.Proved.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:225:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L225)
pub type MinaBaseSnappCommandPartyPredicatedProvedStable =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyPredicatedProvedStableV1, 1i32>;

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:66:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L66)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesBaseSideLoadedVerificationKeyWidthMaxVectorStableV1<A>(
    pub PicklesTypesVectorVector2Stable<A>,
);

/// **Origin**: `Pickles_base__Side_loaded_verification_key.Width.Max_vector.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_base/side_loaded_verification_key.ml:66:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L66)
pub type PicklesBaseSideLoadedVerificationKeyWidthMaxVectorStable<A> = crate::versioned::Versioned<
    PicklesBaseSideLoadedVerificationKeyWidthMaxVectorStableV1<A>,
    1i32,
>;

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:87:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L87)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesBaseSideLoadedVerificationKeyWidthMaxAtMostStableV1<A>(
    pub PicklesTypesAtMostAtMost2Stable<A>,
);

/// **Origin**: `Pickles_base__Side_loaded_verification_key.Width.Max_at_most.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_base/side_loaded_verification_key.ml:87:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L87)
pub type PicklesBaseSideLoadedVerificationKeyWidthMaxAtMostStable<A> = crate::versioned::Versioned<
    PicklesBaseSideLoadedVerificationKeyWidthMaxAtMostStableV1<A>,
    1i32,
>;

/// Location: [src/lib/pickles/proof.ml:352:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L352)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranchingMaxReprStableV1(
    pub  PicklesProofBaseDlogBasedStable<
        CompositionTypesDlogBasedProofStateMeOnlyStable<
            CompositionTypesDlogBasedProofStateMeOnlyStableArg0,
            PicklesBaseSideLoadedVerificationKeyWidthMaxVectorStable<
                PicklesReducedMeOnlyDlogBasedChallengesVectorStable,
            >,
        >,
        PicklesReducedMeOnlyPairingBasedStable<
            (),
            PicklesBaseSideLoadedVerificationKeyWidthMaxAtMostStable<
                PicklesTypesDlogPlonkTypesProofStableArg0,
            >,
            PicklesBaseSideLoadedVerificationKeyWidthMaxAtMostStable<
                PastaBasicRoundsStepVectorStable<
                    CompositionTypesBulletproofChallengeStable<
                        PicklesTypesScalarChallengeStable<
                            PicklesTypesVectorVector2Stable<LimbVectorConstantHex64Stable>,
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
pub type PicklesProofBranchingMaxReprStable =
    crate::versioned::Versioned<PicklesProofBranchingMaxReprStableV1, 1i32>;

/// Location: [src/lib/pickles/proof.ml:388:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L388)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranchingMaxStableV1(pub PicklesProofBranchingMaxReprStable);

/// **Origin**: `Pickles__Proof.Branching_max.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/proof.ml:388:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L388)
pub type PicklesProofBranchingMaxStable =
    crate::versioned::Versioned<PicklesProofBranchingMaxStableV1, 1i32>;

/// Location: [src/lib/mina_base/control.ml:11:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/control.ml#L11)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseControlStableV1 {
    Proof(PicklesProofBranchingMaxStable),
    Signature(MinaBaseSignatureStable),
    Both {
        signature: MinaBaseSignatureStable,
        proof: PicklesProofBranchingMaxStable,
    },
    NoneGiven,
}

/// **Origin**: `Mina_base__Control.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/control.ml:11:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/control.ml#L11)
pub type MinaBaseControlStable = crate::versioned::Versioned<MinaBaseControlStableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:308:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L308)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyAuthorizedProvedStableV1(
    pub  MinaBaseSnappCommandPartyAuthorizedPolyStable<
        MinaBaseSnappCommandPartyPredicatedProvedStable,
        MinaBaseControlStable,
    >,
);

/// **Origin**: `Mina_base__Snapp_command.Party.Authorized.Proved.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:308:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L308)
pub type MinaBaseSnappCommandPartyAuthorizedProvedStable =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyAuthorizedProvedStableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:280:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L280)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyPredicatedEmptyStableV1(
    pub MinaBaseSnappCommandPartyPredicatedPolyStable<MinaBaseSnappCommandPartyBodyStable, ()>,
);

/// **Origin**: `Mina_base__Snapp_command.Party.Predicated.Empty.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:280:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L280)
pub type MinaBaseSnappCommandPartyPredicatedEmptyStable =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyPredicatedEmptyStableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:338:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L338)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyAuthorizedEmptyStableV1(
    pub  MinaBaseSnappCommandPartyAuthorizedPolyStable<
        MinaBaseSnappCommandPartyPredicatedEmptyStable,
        (),
    >,
);

/// **Origin**: `Mina_base__Snapp_command.Party.Authorized.Empty.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:338:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L338)
pub type MinaBaseSnappCommandPartyAuthorizedEmptyStable =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyAuthorizedEmptyStableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:246:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L246)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyPredicatedSignedStableV1(
    pub  MinaBaseSnappCommandPartyPredicatedPolyStable<
        MinaBaseSnappCommandPartyBodyStable,
        MinaNumbersNatMake32Stable,
    >,
);

/// **Origin**: `Mina_base__Snapp_command.Party.Predicated.Signed.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:246:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L246)
pub type MinaBaseSnappCommandPartyPredicatedSignedStable =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyPredicatedSignedStableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:323:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L323)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyAuthorizedSignedStableV1(
    pub  MinaBaseSnappCommandPartyAuthorizedPolyStable<
        MinaBaseSnappCommandPartyPredicatedSignedStable,
        MinaBaseSignatureStable,
    >,
);

/// **Origin**: `Mina_base__Snapp_command.Party.Authorized.Signed.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:323:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L323)
pub type MinaBaseSnappCommandPartyAuthorizedSignedStable =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyAuthorizedSignedStableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:367:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L367)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSnappCommandBinableArgStableV1 {
    ProvedEmpty(
        MinaBaseSnappCommandInnerStable<
            MinaBaseSnappCommandPartyAuthorizedProvedStable,
            Option<MinaBaseSnappCommandPartyAuthorizedEmptyStable>,
        >,
    ),
    ProvedSigned(
        MinaBaseSnappCommandInnerStable<
            MinaBaseSnappCommandPartyAuthorizedProvedStable,
            MinaBaseSnappCommandPartyAuthorizedSignedStable,
        >,
    ),
    ProvedProved(
        MinaBaseSnappCommandInnerStable<
            MinaBaseSnappCommandPartyAuthorizedProvedStable,
            MinaBaseSnappCommandPartyAuthorizedProvedStable,
        >,
    ),
    SignedSigned(
        MinaBaseSnappCommandInnerStable<
            MinaBaseSnappCommandPartyAuthorizedSignedStable,
            MinaBaseSnappCommandPartyAuthorizedSignedStable,
        >,
    ),
    SignedEmpty(
        MinaBaseSnappCommandInnerStable<
            MinaBaseSnappCommandPartyAuthorizedSignedStable,
            Option<MinaBaseSnappCommandPartyAuthorizedEmptyStable>,
        >,
    ),
}

/// **Origin**: `Mina_base__Snapp_command.Binable_arg.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:367:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L367)
pub type MinaBaseSnappCommandBinableArgStable =
    crate::versioned::Versioned<MinaBaseSnappCommandBinableArgStableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:408:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L408)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandStableV1(pub MinaBaseSnappCommandBinableArgStable);

/// **Origin**: `Mina_base__Snapp_command.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:408:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L408)
pub type MinaBaseSnappCommandStable =
    crate::versioned::Versioned<MinaBaseSnappCommandStableV1, 1i32>;

/// Location: [src/lib/mina_base/user_command.ml:74:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/user_command.ml#L74)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseUserCommandStableV1(
    pub MinaBaseUserCommandPolyStable<MinaBaseSignedCommandStable, MinaBaseSnappCommandStable>,
);

/// **Origin**: `Mina_base__User_command.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/user_command.ml:74:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/user_command.ml#L74)
pub type MinaBaseUserCommandStable = crate::versioned::Versioned<MinaBaseUserCommandStableV1, 1i32>;

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:136:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L136)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1(
    pub  StagedLedgerDiffPreDiffTwoStable<
        TransactionSnarkWorkTStable,
        MinaBaseWithStatusStable<MinaBaseUserCommandStable>,
    >,
);

/// **Origin**: `Staged_ledger_diff.Pre_diff_with_at_most_two_coinbase.Stable.V1.t`
///
/// **Location**: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:136:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L136)
pub type StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStable =
    crate::versioned::Versioned<StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1, 1i32>;

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:43:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L43)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum StagedLedgerDiffAtMostOneStableV1<A> {
    Zero,
    One(Option<A>),
}

/// **Origin**: `Staged_ledger_diff.At_most_one.Stable.V1.t`
///
/// **Location**: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:43:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L43)
pub type StagedLedgerDiffAtMostOneStable<A> =
    crate::versioned::Versioned<StagedLedgerDiffAtMostOneStableV1<A>, 1i32>;

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:109:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L109)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffPreDiffOneStableV1<A, B> {
    pub completed_works: Vec<A>,
    pub commands: Vec<B>,
    pub coinbase: StagedLedgerDiffAtMostOneStable<StagedLedgerDiffFtStable>,
    pub internal_command_balances: Vec<MinaBaseTransactionStatusInternalCommandBalanceDataStable>,
}

/// **Origin**: `Staged_ledger_diff.Pre_diff_one.Stable.V1.t`
///
/// **Location**: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:109:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L109)
pub type StagedLedgerDiffPreDiffOneStable<A, B> =
    crate::versioned::Versioned<StagedLedgerDiffPreDiffOneStableV1<A, B>, 1i32>;

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:155:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L155)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1(
    pub  StagedLedgerDiffPreDiffOneStable<
        TransactionSnarkWorkTStable,
        MinaBaseWithStatusStable<MinaBaseUserCommandStable>,
    >,
);

/// **Origin**: `Staged_ledger_diff.Pre_diff_with_at_most_one_coinbase.Stable.V1.t`
///
/// **Location**: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:155:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L155)
pub type StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStable =
    crate::versioned::Versioned<StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1, 1i32>;

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:174:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L174)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffStableV1(
    pub StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStable,
    pub Option<StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStable>,
);

/// **Origin**: `Staged_ledger_diff.Diff.Stable.V1.t`
///
/// **Location**: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:174:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L174)
pub type StagedLedgerDiffDiffStable =
    crate::versioned::Versioned<StagedLedgerDiffDiffStableV1, 1i32>;

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:191:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L191)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffStableV1 {
    pub diff: StagedLedgerDiffDiffStable,
}

/// **Origin**: `Staged_ledger_diff.Stable.V1.t`
///
/// **Location**: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:191:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L191)
pub type StagedLedgerDiffStable = crate::versioned::Versioned<StagedLedgerDiffStableV1, 1i32>;

/// Location: [src/lib/mina_base/state_body_hash.ml:20:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/state_body_hash.ml#L20)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBlockExternalTransitionRawVersionedStableV1DeltaTransitionChainProofArg0V1Poly(
    pub crate::bigint::BigInt,
);

/// Location: [src/lib/mina_base/state_body_hash.ml:19:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/state_body_hash.ml#L19)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBlockExternalTransitionRawVersionedStableV1DeltaTransitionChainProofArg0V1(
    pub MinaBlockExternalTransitionRawVersionedStableV1DeltaTransitionChainProofArg0V1Poly,
);

/// Location: [src/lib/mina_base/state_body_hash.ml:19:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/state_body_hash.ml#L19)
pub type MinaBlockExternalTransitionRawVersionedStableV1DeltaTransitionChainProofArg0 =
    crate::versioned::Versioned<
        MinaBlockExternalTransitionRawVersionedStableV1DeltaTransitionChainProofArg0V1,
        1i32,
    >;

/// Location: [src/lib/protocol_version/protocol_version.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/protocol_version/protocol_version.ml#L8)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ProtocolVersionStableV1 {
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
}

/// **Origin**: `Protocol_version.Stable.V1.t`
///
/// **Location**: [src/lib/protocol_version/protocol_version.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/protocol_version/protocol_version.ml#L8)
pub type ProtocolVersionStable = crate::versioned::Versioned<ProtocolVersionStableV1, 1i32>;

/// Location: [src/lib/mina_block/external_transition.ml:31:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_block/external_transition.ml#L31)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBlockExternalTransitionRawVersionedStableV1 {
    pub protocol_state: MinaStateProtocolStateValueStable,
    pub protocol_state_proof: MinaBaseProofStable,
    pub staged_ledger_diff: StagedLedgerDiffStable,
    pub delta_transition_chain_proof: (
        MinaStateProtocolStatePolyStableArg0,
        Vec<MinaBlockExternalTransitionRawVersionedStableV1DeltaTransitionChainProofArg0>,
    ),
    pub current_protocol_version: ProtocolVersionStable,
    pub proposed_protocol_version_opt: Option<ProtocolVersionStable>,
    pub validation_callback: (),
}

/// Location: [src/lib/network_pool/transaction_pool.ml:45:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_pool/transaction_pool.ml#L45)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NetworkPoolTransactionPoolDiffVersionedStableV1(pub Vec<MinaBaseUserCommandStable>);

/// Location: [src/lib/transaction_snark/transaction_snark.ml:202:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L202)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkStatementStableV1(
    pub  TransactionSnarkStatementPolyStable<
        MinaBaseStagedLedgerHashNonSnarkStableV1LedgerHash,
        CurrencyAmountMakeStrStable,
        TransactionSnarkPendingCoinbaseStackStateStable,
        MinaBaseFeeExcessStable,
        MinaBaseTokenIdStable,
        (),
    >,
);

/// **Origin**: `Transaction_snark.Statement.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark/transaction_snark.ml:202:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L202)
pub type TransactionSnarkStatementStable =
    crate::versioned::Versioned<TransactionSnarkStatementStableV1, 1i32>;

/// Location: [src/lib/transaction_snark_work/transaction_snark_work.ml:23:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_work/transaction_snark_work.ml#L23)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkWorkStatementStableV1(
    pub OneOrTwoStable<TransactionSnarkStatementStable>,
);

/// **Origin**: `Transaction_snark_work.Statement.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark_work/transaction_snark_work.ml:23:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_work/transaction_snark_work.ml#L23)
pub type TransactionSnarkWorkStatementStable =
    crate::versioned::Versioned<TransactionSnarkWorkStatementStableV1, 1i32>;

/// Location: [src/lib/mina_base/fee_with_prover.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_with_prover.ml#L7)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeWithProverStableV1 {
    pub fee: CurrencyFeeStable,
    pub prover: ConsensusProofOfStakeDataConsensusStatePolyStableArg8,
}

/// **Origin**: `Mina_base__Fee_with_prover.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/fee_with_prover.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_with_prover.ml#L7)
pub type MinaBaseFeeWithProverStable =
    crate::versioned::Versioned<MinaBaseFeeWithProverStableV1, 1i32>;

/// Location: [src/lib/network_pool/priced_proof.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_pool/priced_proof.ml#L9)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NetworkPoolPricedProofStableV1<Proof> {
    pub proof: Proof,
    pub fee: MinaBaseFeeWithProverStable,
}

/// **Origin**: `Network_pool__Priced_proof.Stable.V1.t`
///
/// **Location**: [src/lib/network_pool/priced_proof.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_pool/priced_proof.ml#L9)
pub type NetworkPoolPricedProofStable<Proof> =
    crate::versioned::Versioned<NetworkPoolPricedProofStableV1<Proof>, 1i32>;

/// Location: [src/lib/network_pool/snark_pool.ml:705:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_pool/snark_pool.ml#L705)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum NetworkPoolSnarkPoolDiffVersionedStableV1 {
    AddSolvedWork(
        TransactionSnarkWorkStatementStable,
        NetworkPoolPricedProofStable<OneOrTwoStable<LedgerProofProdStable>>,
    ),
    Empty,
}

/// Location: [src/lib/mina_base/account.ml:89:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account.ml#L89)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountPolyStableV1<
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

/// **Origin**: `Mina_base__Account.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/account.ml:89:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account.ml#L89)
pub type MinaBaseAccountPolyStable<
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
    MinaBaseAccountPolyStableV1<
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseTokenPermissionsStableV1 {
    TokenOwned { disable_new_accounts: bool },
    NotOwned { account_disabled: bool },
}

/// **Origin**: `Mina_base__Token_permissions.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/token_permissions.ml:14:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/token_permissions.ml#L14)
pub type MinaBaseTokenPermissionsStable =
    crate::versioned::Versioned<MinaBaseTokenPermissionsStableV1, 1i32>;

/// Location: [src/lib/mina_base/account_timing.ml:19:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account_timing.ml#L19)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountTimingPolyStableV1<Slot, Balance, Amount> {
    Untimed,
    Timed {
        initial_minimum_balance: Balance,
        cliff_time: Slot,
        cliff_amount: Amount,
        vesting_period: Slot,
        vesting_increment: Amount,
    },
}

/// **Origin**: `Mina_base__Account_timing.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/account_timing.ml:19:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account_timing.ml#L19)
pub type MinaBaseAccountTimingPolyStable<Slot, Balance, Amount> =
    crate::versioned::Versioned<MinaBaseAccountTimingPolyStableV1<Slot, Balance, Amount>, 1i32>;

/// Location: [src/lib/mina_base/account_timing.ml:36:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account_timing.ml#L36)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountTimingStableV1(
    pub  MinaBaseAccountTimingPolyStable<
        ConsensusGlobalSlotPolyStableArg0,
        CurrencyBalanceStable,
        CurrencyAmountMakeStrStable,
    >,
);

/// **Origin**: `Mina_base__Account_timing.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/account_timing.ml:36:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account_timing.ml#L36)
pub type MinaBaseAccountTimingStable =
    crate::versioned::Versioned<MinaBaseAccountTimingStableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_account.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_account.ml#L17)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappAccountPolyStableV1<AppState, Vk> {
    pub app_state: AppState,
    pub verification_key: Vk,
}

/// **Origin**: `Mina_base__Snapp_account.Poly.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_account.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_account.ml#L17)
pub type MinaBaseSnappAccountPolyStable<AppState, Vk> =
    crate::versioned::Versioned<MinaBaseSnappAccountPolyStableV1<AppState, Vk>, 1i32>;

/// Location: [src/lib/mina_base/snapp_state.ml:50:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_state.ml#L50)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappStateValueStableV1(pub MinaBaseSnappStateVStable<crate::bigint::BigInt>);

/// **Origin**: `Mina_base__Snapp_state.Value.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_state.ml:50:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_state.ml#L50)
pub type MinaBaseSnappStateValueStable =
    crate::versioned::Versioned<MinaBaseSnappStateValueStableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_account.ml:30:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_account.ml#L30)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappAccountStableV1(
    pub  MinaBaseSnappAccountPolyStable<
        MinaBaseSnappStateValueStable,
        Option<WithHashStable<PicklesSideLoadedVerificationKeyStable, crate::bigint::BigInt>>,
    >,
);

/// **Origin**: `Mina_base__Snapp_account.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_account.ml:30:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_account.ml#L30)
pub type MinaBaseSnappAccountStable =
    crate::versioned::Versioned<MinaBaseSnappAccountStableV1, 1i32>;

/// Location: [src/lib/mina_base/account.ml:140:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account.ml#L140)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountBinableArgStableV1(
    pub  MinaBaseAccountPolyStable<
        ConsensusProofOfStakeDataConsensusStatePolyStableArg8,
        MinaBaseTokenIdStable,
        MinaBaseTokenPermissionsStable,
        CurrencyBalanceStable,
        MinaNumbersNatMake32Stable,
        MinaBaseSnappBasicOrIgnoreStableArg0,
        Option<ConsensusProofOfStakeDataConsensusStatePolyStableArg8>,
        MinaStateProtocolStatePolyStableArg0,
        MinaBaseAccountTimingStable,
        MinaBasePermissionsStable,
        Option<MinaBaseSnappAccountStable>,
    >,
);

/// **Origin**: `Mina_base__Account.Binable_arg.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/account.ml:140:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account.ml#L140)
pub type MinaBaseAccountBinableArgStable =
    crate::versioned::Versioned<MinaBaseAccountBinableArgStableV1, 1i32>;

/// Location: [src/lib/mina_base/account.ml:188:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account.ml#L188)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountStableV1(pub MinaBaseAccountBinableArgStable);
