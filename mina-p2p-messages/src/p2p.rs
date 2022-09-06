use binprot::{BinProtRead, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};

/// **Origin**: `Mina_block__External_transition.Raw_versioned__.Stable.V1.t`
///
/// **Location**: [src/lib/mina_block/external_transition.ml:31:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_block/external_transition.ml#L31)
pub type MinaBlockExternalTransitionRawVersionedStableV1Binable =
    crate::versioned::Versioned<MinaBlockExternalTransitionRawVersionedStableV1BinableV1, 1i32>;

/// **Origin**: `Network_pool__Transaction_pool.Diff_versioned.Stable.V1.t`
///
/// **Location**: [src/lib/network_pool/transaction_pool.ml:45:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_pool/transaction_pool.ml#L45)
pub type NetworkPoolTransactionPoolDiffVersionedStableV1Binable =
    crate::versioned::Versioned<NetworkPoolTransactionPoolDiffVersionedStableV1BinableV1, 1i32>;

/// **Origin**: `Network_pool__Snark_pool.Diff_versioned.Stable.V1.t`
///
/// **Location**: [src/lib/network_pool/snark_pool.ml:705:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_pool/snark_pool.ml#L705)
pub type NetworkPoolSnarkPoolDiffVersionedStableV1Binable =
    crate::versioned::Versioned<NetworkPoolSnarkPoolDiffVersionedStableV1BinableV1, 1i32>;

/// **Origin**: `Mina_base__Account.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/account.ml:188:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account.ml#L188)
pub type MinaBaseAccountStableV1Binable =
    crate::versioned::Versioned<MinaBaseAccountStableV1BinableV1, 1i32>;


/// Location: [src/lib/mina_state/protocol_state.ml:16:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L16)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateProtocolStateValueStableV1BinableV1PolyV1<StateHash, Body> {
    pub previous_state_hash: StateHash,
    pub body: Body,
}

/// Location: [src/lib/mina_state/protocol_state.ml:16:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L16)
pub type MinaStateProtocolStateValueStableV1BinableV1Poly<StateHash, Body> =
    crate::versioned::Versioned<
        MinaStateProtocolStateValueStableV1BinableV1PolyV1<StateHash, Body>,
        1i32,
    >;

/// Location: [src/lib/data_hash_lib/state_hash.ml:43:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/data_hash_lib/state_hash.ml#L43)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateProtocolStateValueStableV1BinableV1PolyArg0V1Poly(pub crate::bigint::BigInt);

/// Location: [src/lib/data_hash_lib/state_hash.ml:42:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/data_hash_lib/state_hash.ml#L42)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateProtocolStateValueStableV1BinableV1PolyArg0V1(
    pub MinaStateProtocolStateValueStableV1BinableV1PolyArg0V1Poly,
);

/// Location: [src/lib/data_hash_lib/state_hash.ml:42:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/data_hash_lib/state_hash.ml#L42)
pub type MinaStateProtocolStateValueStableV1BinableV1PolyArg0 =
    crate::versioned::Versioned<MinaStateProtocolStateValueStableV1BinableV1PolyArg0V1, 1i32>;

/// Location: [src/lib/mina_state/protocol_state.ml:38:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L38)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateProtocolStateBodyValueStableV1BinableV1PolyV1<
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
pub type MinaStateProtocolStateBodyValueStableV1BinableV1Poly<
    StateHash,
    BlockchainState,
    ConsensusState,
    Constants,
> = crate::versioned::Versioned<
    MinaStateProtocolStateBodyValueStableV1BinableV1PolyV1<
        StateHash,
        BlockchainState,
        ConsensusState,
        Constants,
    >,
    1i32,
>;

/// Location: [src/lib/mina_state/blockchain_state.ml:9:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/blockchain_state.ml#L9)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateBlockchainStateValueStableV1BinableV1PolyV1<
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
pub type MinaStateBlockchainStateValueStableV1BinableV1Poly<
    StagedLedgerHash,
    SnarkedLedgerHash,
    TokenId,
    Time,
> = crate::versioned::Versioned<
    MinaStateBlockchainStateValueStableV1BinableV1PolyV1<
        StagedLedgerHash,
        SnarkedLedgerHash,
        TokenId,
        Time,
    >,
    1i32,
>;

/// Location: [src/lib/mina_base/staged_ledger_hash.ml:174:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L174)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashStableV1BinableV1PolyV1<NonSnark, PendingCoinbaseHash> {
    pub non_snark: NonSnark,
    pub pending_coinbase_hash: PendingCoinbaseHash,
}

/// Location: [src/lib/mina_base/staged_ledger_hash.ml:174:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L174)
pub type MinaBaseStagedLedgerHashStableV1BinableV1Poly<NonSnark, PendingCoinbaseHash> =
    crate::versioned::Versioned<
        MinaBaseStagedLedgerHashStableV1BinableV1PolyV1<NonSnark, PendingCoinbaseHash>,
        1i32,
    >;

/// Location: [src/lib/mina_base/ledger_hash0.ml:18:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/ledger_hash0.ml#L18)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashNonSnarkStableV1BinableV1LedgerHashV1Poly(
    pub crate::bigint::BigInt,
);

/// Location: [src/lib/mina_base/ledger_hash0.ml:17:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/ledger_hash0.ml#L17)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashNonSnarkStableV1BinableV1LedgerHashV1(
    pub MinaBaseStagedLedgerHashNonSnarkStableV1BinableV1LedgerHashV1Poly,
);

/// Location: [src/lib/mina_base/ledger_hash0.ml:17:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/ledger_hash0.ml#L17)
pub type MinaBaseStagedLedgerHashNonSnarkStableV1BinableV1LedgerHash = crate::versioned::Versioned<
    MinaBaseStagedLedgerHashNonSnarkStableV1BinableV1LedgerHashV1,
    1i32,
>;

/// Location: [src/lib/mina_base/staged_ledger_hash.ml:15:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L15)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashAuxHashStableV1BinableV1(pub crate::string::String);

/// **Origin**: `Mina_base__Staged_ledger_hash.Aux_hash.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/staged_ledger_hash.ml:15:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L15)
pub type MinaBaseStagedLedgerHashAuxHashStableV1Binable =
    crate::versioned::Versioned<MinaBaseStagedLedgerHashAuxHashStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/staged_ledger_hash.ml:59:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L59)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashPendingCoinbaseAuxStableV1BinableV1(pub crate::string::String);

/// **Origin**: `Mina_base__Staged_ledger_hash.Pending_coinbase_aux.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/staged_ledger_hash.ml:59:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L59)
pub type MinaBaseStagedLedgerHashPendingCoinbaseAuxStableV1Binable =
    crate::versioned::Versioned<MinaBaseStagedLedgerHashPendingCoinbaseAuxStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/staged_ledger_hash.ml:95:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L95)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashNonSnarkStableV1BinableV1 {
    pub ledger_hash: MinaBaseStagedLedgerHashNonSnarkStableV1BinableV1LedgerHash,
    pub aux_hash: MinaBaseStagedLedgerHashAuxHashStableV1Binable,
    pub pending_coinbase_aux: MinaBaseStagedLedgerHashPendingCoinbaseAuxStableV1Binable,
}

/// **Origin**: `Mina_base__Staged_ledger_hash.Non_snark.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/staged_ledger_hash.ml:95:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L95)
pub type MinaBaseStagedLedgerHashNonSnarkStableV1Binable =
    crate::versioned::Versioned<MinaBaseStagedLedgerHashNonSnarkStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/pending_coinbase.ml:359:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L359)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseHashVersionedStableV1BinableV1PolyV1Poly(
    pub crate::bigint::BigInt,
);

/// Location: [src/lib/mina_base/pending_coinbase.ml:358:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L358)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseHashVersionedStableV1BinableV1PolyV1(
    pub MinaBasePendingCoinbaseHashVersionedStableV1BinableV1PolyV1Poly,
);

/// Location: [src/lib/mina_base/pending_coinbase.ml:358:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L358)
pub type MinaBasePendingCoinbaseHashVersionedStableV1BinableV1Poly =
    crate::versioned::Versioned<MinaBasePendingCoinbaseHashVersionedStableV1BinableV1PolyV1, 1i32>;

/// Location: [src/lib/mina_base/pending_coinbase.ml:517:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L517)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseHashVersionedStableV1BinableV1(
    pub MinaBasePendingCoinbaseHashVersionedStableV1BinableV1Poly,
);

/// **Origin**: `Mina_base__Pending_coinbase.Hash_versioned.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/pending_coinbase.ml:517:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L517)
pub type MinaBasePendingCoinbaseHashVersionedStableV1Binable =
    crate::versioned::Versioned<MinaBasePendingCoinbaseHashVersionedStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/staged_ledger_hash.ml:191:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L191)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashStableV1BinableV1(
    pub  MinaBaseStagedLedgerHashStableV1BinableV1Poly<
        MinaBaseStagedLedgerHashNonSnarkStableV1Binable,
        MinaBasePendingCoinbaseHashVersionedStableV1Binable,
    >,
);

/// **Origin**: `Mina_base__Staged_ledger_hash.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/staged_ledger_hash.ml:191:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L191)
pub type MinaBaseStagedLedgerHashStableV1Binable =
    crate::versioned::Versioned<MinaBaseStagedLedgerHashStableV1BinableV1, 1i32>;

/// Location: [src/lib/unsigned_extended/unsigned_extended.ml:76:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/unsigned_extended/unsigned_extended.ml#L76)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct UnsignedExtendedUInt64StableV1BinableV1(pub i64);

/// **Origin**: `Unsigned_extended.UInt64.Stable.V1.t`
///
/// **Location**: [src/lib/unsigned_extended/unsigned_extended.ml:76:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/unsigned_extended/unsigned_extended.ml#L76)
pub type UnsignedExtendedUInt64StableV1Binable =
    crate::versioned::Versioned<UnsignedExtendedUInt64StableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_numbers/nat.ml:220:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L220)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaNumbersNatMake64StableV1BinableV1(pub UnsignedExtendedUInt64StableV1Binable);

/// **Origin**: `Mina_numbers__Nat.Make64.Stable.V1.t`
///
/// **Location**: [src/lib/mina_numbers/nat.ml:220:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L220)
pub type MinaNumbersNatMake64StableV1Binable =
    crate::versioned::Versioned<MinaNumbersNatMake64StableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/token_id.ml:49:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/token_id.ml#L49)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseTokenIdStableV1BinableV1(pub MinaNumbersNatMake64StableV1Binable);

/// **Origin**: `Mina_base__Token_id.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/token_id.ml:49:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/token_id.ml#L49)
pub type MinaBaseTokenIdStableV1Binable =
    crate::versioned::Versioned<MinaBaseTokenIdStableV1BinableV1, 1i32>;

/// Location: [src/lib/block_time/block_time.ml:14:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/block_time/block_time.ml#L14)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct BlockTimeTimeStableV1BinableV1(pub UnsignedExtendedUInt64StableV1Binable);

/// **Origin**: `Block_time.Time.Stable.V1.t`
///
/// **Location**: [src/lib/block_time/block_time.ml:14:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/block_time/block_time.ml#L14)
pub type BlockTimeTimeStableV1Binable =
    crate::versioned::Versioned<BlockTimeTimeStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_state/blockchain_state.ml:35:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/blockchain_state.ml#L35)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateBlockchainStateValueStableV1BinableV1(
    pub  MinaStateBlockchainStateValueStableV1BinableV1Poly<
        MinaBaseStagedLedgerHashStableV1Binable,
        MinaBaseStagedLedgerHashNonSnarkStableV1BinableV1LedgerHash,
        MinaBaseTokenIdStableV1Binable,
        BlockTimeTimeStableV1Binable,
    >,
);

/// **Origin**: `Mina_state__Blockchain_state.Value.Stable.V1.t`
///
/// **Location**: [src/lib/mina_state/blockchain_state.ml:35:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/blockchain_state.ml#L35)
pub type MinaStateBlockchainStateValueStableV1Binable =
    crate::versioned::Versioned<MinaStateBlockchainStateValueStableV1BinableV1, 1i32>;

/// Location: [src/lib/consensus/proof_of_stake.ml:1681:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1681)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyV1<
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
pub type ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1Poly<
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
    ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyV1<
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
pub struct UnsignedExtendedUInt32StableV1BinableV1(pub i32);

/// **Origin**: `Unsigned_extended.UInt32.Stable.V1.t`
///
/// **Location**: [src/lib/unsigned_extended/unsigned_extended.ml:126:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/unsigned_extended/unsigned_extended.ml#L126)
pub type UnsignedExtendedUInt32StableV1Binable =
    crate::versioned::Versioned<UnsignedExtendedUInt32StableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_numbers/nat.ml:186:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L186)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg0V1(
    pub UnsignedExtendedUInt32StableV1Binable,
);

/// Location: [src/lib/mina_numbers/nat.ml:186:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L186)
pub type ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg0 =
    crate::versioned::Versioned<
        ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg0V1,
        1i32,
    >;

/// Location: [src/lib/consensus/vrf/consensus_vrf.ml:170:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/vrf/consensus_vrf.ml#L170)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusVrfOutputTruncatedStableV1BinableV1(pub crate::string::String);

/// **Origin**: `Consensus_vrf.Output.Truncated.Stable.V1.t`
///
/// **Location**: [src/lib/consensus/vrf/consensus_vrf.ml:170:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/vrf/consensus_vrf.ml#L170)
pub type ConsensusVrfOutputTruncatedStableV1Binable =
    crate::versioned::Versioned<ConsensusVrfOutputTruncatedStableV1BinableV1, 1i32>;

/// Location: [src/lib/currency/currency.ml:706:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/currency.ml#L706)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CurrencyAmountMakeStrStableV1BinableV1(pub UnsignedExtendedUInt64StableV1Binable);

/// **Origin**: `Currency.Amount.Make_str.Stable.V1.t`
///
/// **Location**: [src/lib/currency/currency.ml:706:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/currency.ml#L706)
pub type CurrencyAmountMakeStrStableV1Binable =
    crate::versioned::Versioned<CurrencyAmountMakeStrStableV1BinableV1, 1i32>;

/// Location: [src/lib/consensus/global_slot.ml:11:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/global_slot.ml#L11)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusGlobalSlotStableV1BinableV1PolyV1<SlotNumber, SlotsPerEpoch> {
    pub slot_number: SlotNumber,
    pub slots_per_epoch: SlotsPerEpoch,
}

/// Location: [src/lib/consensus/global_slot.ml:11:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/global_slot.ml#L11)
pub type ConsensusGlobalSlotStableV1BinableV1Poly<SlotNumber, SlotsPerEpoch> =
    crate::versioned::Versioned<
        ConsensusGlobalSlotStableV1BinableV1PolyV1<SlotNumber, SlotsPerEpoch>,
        1i32,
    >;

/// Location: [src/lib/mina_numbers/nat.ml:186:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L186)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusGlobalSlotStableV1BinableV1PolyArg0V1(
    pub UnsignedExtendedUInt32StableV1Binable,
);

/// Location: [src/lib/mina_numbers/nat.ml:186:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L186)
pub type ConsensusGlobalSlotStableV1BinableV1PolyArg0 =
    crate::versioned::Versioned<ConsensusGlobalSlotStableV1BinableV1PolyArg0V1, 1i32>;

/// Location: [src/lib/consensus/global_slot.ml:21:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/global_slot.ml#L21)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusGlobalSlotStableV1BinableV1(
    pub  ConsensusGlobalSlotStableV1BinableV1Poly<
        ConsensusGlobalSlotStableV1BinableV1PolyArg0,
        ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg0,
    >,
);

/// **Origin**: `Consensus__Global_slot.Stable.V1.t`
///
/// **Location**: [src/lib/consensus/global_slot.ml:21:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/global_slot.ml#L21)
pub type ConsensusGlobalSlotStableV1Binable =
    crate::versioned::Versioned<ConsensusGlobalSlotStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/epoch_data.ml:8:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_data.ml#L8)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1BinableV1PolyV1<
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
pub type ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1BinableV1Poly<
    EpochLedger,
    EpochSeed,
    StartCheckpoint,
    LockCheckpoint,
    Length,
> = crate::versioned::Versioned<
    ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1BinableV1PolyV1<
        EpochLedger,
        EpochSeed,
        StartCheckpoint,
        LockCheckpoint,
        Length,
    >,
    1i32,
>;

/// Location: [src/lib/mina_base/epoch_ledger.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_ledger.ml#L10)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseEpochLedgerValueStableV1BinableV1PolyV1<LedgerHash, Amount> {
    pub hash: LedgerHash,
    pub total_currency: Amount,
}

/// Location: [src/lib/mina_base/epoch_ledger.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_ledger.ml#L10)
pub type MinaBaseEpochLedgerValueStableV1BinableV1Poly<LedgerHash, Amount> =
    crate::versioned::Versioned<
        MinaBaseEpochLedgerValueStableV1BinableV1PolyV1<LedgerHash, Amount>,
        1i32,
    >;

/// Location: [src/lib/mina_base/epoch_ledger.ml:21:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_ledger.ml#L21)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseEpochLedgerValueStableV1BinableV1(
    pub  MinaBaseEpochLedgerValueStableV1BinableV1Poly<
        MinaBaseStagedLedgerHashNonSnarkStableV1BinableV1LedgerHash,
        CurrencyAmountMakeStrStableV1Binable,
    >,
);

/// **Origin**: `Mina_base__Epoch_ledger.Value.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/epoch_ledger.ml:21:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_ledger.ml#L21)
pub type MinaBaseEpochLedgerValueStableV1Binable =
    crate::versioned::Versioned<MinaBaseEpochLedgerValueStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/epoch_seed.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_seed.ml#L17)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1BinableV1PolyArg1V1Poly(
    pub crate::bigint::BigInt,
);

/// Location: [src/lib/mina_base/epoch_seed.ml:16:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_seed.ml#L16)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1BinableV1PolyArg1V1(
    pub ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1BinableV1PolyArg1V1Poly,
);

/// Location: [src/lib/mina_base/epoch_seed.ml:16:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_seed.ml#L16)
pub type ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1BinableV1PolyArg1 =
    crate::versioned::Versioned<
        ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1BinableV1PolyArg1V1,
        1i32,
    >;

/// Location: [src/lib/consensus/proof_of_stake.ml:1050:12](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1050)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1BinableV1(
    pub  ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1BinableV1Poly<
        MinaBaseEpochLedgerValueStableV1Binable,
        ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1BinableV1PolyArg1,
        MinaStateProtocolStateValueStableV1BinableV1PolyArg0,
        MinaStateProtocolStateValueStableV1BinableV1PolyArg0,
        ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg0,
    >,
);

/// **Origin**: `Consensus__Proof_of_stake.Data.Epoch_data.Staking_value_versioned.Value.Stable.V1.t`
///
/// **Location**: [src/lib/consensus/proof_of_stake.ml:1050:12](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1050)
pub type ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1Binable =
    crate::versioned::Versioned<
        ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1BinableV1,
        1i32,
    >;

/// Location: [src/lib/consensus/proof_of_stake.ml:1074:12](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1074)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1BinableV1(
    pub  ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1BinableV1Poly<
        MinaBaseEpochLedgerValueStableV1Binable,
        ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1BinableV1PolyArg1,
        MinaStateProtocolStateValueStableV1BinableV1PolyArg0,
        MinaStateProtocolStateValueStableV1BinableV1PolyArg0,
        ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg0,
    >,
);

/// **Origin**: `Consensus__Proof_of_stake.Data.Epoch_data.Next_value_versioned.Value.Stable.V1.t`
///
/// **Location**: [src/lib/consensus/proof_of_stake.ml:1074:12](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1074)
pub type ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1Binable =
    crate::versioned::Versioned<
        ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1BinableV1,
        1i32,
    >;

/// Location: [src/lib/non_zero_curve_point/compressed_poly.ml:11:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/compressed_poly.ml#L11)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8V1PolyPolyV1<
    Field,
    Boolean,
> {
    pub x: Field,
    pub is_odd: Boolean,
}

/// Location: [src/lib/non_zero_curve_point/compressed_poly.ml:11:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/compressed_poly.ml#L11)
pub type ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8V1PolyPoly<
    Field,
    Boolean,
> = crate::versioned::Versioned<
    ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8V1PolyPolyV1<
        Field,
        Boolean,
    >,
    1i32,
>;

/// Location: [src/lib/non_zero_curve_point/non_zero_curve_point.ml:52:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/non_zero_curve_point.ml#L52)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8V1Poly(
    pub  ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8V1PolyPoly<
        crate::bigint::BigInt,
        bool,
    >,
);

/// Location: [src/lib/non_zero_curve_point/non_zero_curve_point.ml:51:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/non_zero_curve_point.ml#L51)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8V1(
    pub ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8V1Poly,
);

/// Location: [src/lib/non_zero_curve_point/non_zero_curve_point.ml:51:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/non_zero_curve_point.ml#L51)
pub type ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8 =
    crate::versioned::Versioned<
        ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8V1,
        1i32,
    >;

/// Location: [src/lib/consensus/proof_of_stake.ml:1716:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1716)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1(
    pub  ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1Poly<
        ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg0,
        ConsensusVrfOutputTruncatedStableV1Binable,
        CurrencyAmountMakeStrStableV1Binable,
        ConsensusGlobalSlotStableV1Binable,
        ConsensusGlobalSlotStableV1BinableV1PolyArg0,
        ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1Binable,
        ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1Binable,
        bool,
        ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8,
    >,
);

/// **Origin**: `Consensus__Proof_of_stake.Data.Consensus_state.Value.Stable.V1.t`
///
/// **Location**: [src/lib/consensus/proof_of_stake.ml:1716:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1716)
pub type ConsensusProofOfStakeDataConsensusStateValueStableV1Binable = crate::versioned::Versioned<
    ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1,
    1i32,
>;

/// Location: [src/lib/genesis_constants/genesis_constants.ml:239:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/genesis_constants/genesis_constants.ml#L239)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseProtocolConstantsCheckedValueStableV1BinableV1PolyV1<
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
pub type MinaBaseProtocolConstantsCheckedValueStableV1BinableV1Poly<
    Length,
    Delta,
    GenesisStateTimestamp,
> = crate::versioned::Versioned<
    MinaBaseProtocolConstantsCheckedValueStableV1BinableV1PolyV1<
        Length,
        Delta,
        GenesisStateTimestamp,
    >,
    1i32,
>;

/// Location: [src/lib/mina_base/protocol_constants_checked.ml:22:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/protocol_constants_checked.ml#L22)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseProtocolConstantsCheckedValueStableV1BinableV1(
    pub  MinaBaseProtocolConstantsCheckedValueStableV1BinableV1Poly<
        ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg0,
        ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg0,
        BlockTimeTimeStableV1Binable,
    >,
);

/// **Origin**: `Mina_base__Protocol_constants_checked.Value.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/protocol_constants_checked.ml:22:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/protocol_constants_checked.ml#L22)
pub type MinaBaseProtocolConstantsCheckedValueStableV1Binable =
    crate::versioned::Versioned<MinaBaseProtocolConstantsCheckedValueStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_state/protocol_state.ml:53:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L53)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateProtocolStateBodyValueStableV1BinableV1(
    pub  MinaStateProtocolStateBodyValueStableV1BinableV1Poly<
        MinaStateProtocolStateValueStableV1BinableV1PolyArg0,
        MinaStateBlockchainStateValueStableV1Binable,
        ConsensusProofOfStakeDataConsensusStateValueStableV1Binable,
        MinaBaseProtocolConstantsCheckedValueStableV1Binable,
    >,
);

/// **Origin**: `Mina_state__Protocol_state.Body.Value.Stable.V1.t`
///
/// **Location**: [src/lib/mina_state/protocol_state.ml:53:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L53)
pub type MinaStateProtocolStateBodyValueStableV1Binable =
    crate::versioned::Versioned<MinaStateProtocolStateBodyValueStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_state/protocol_state.ml:177:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L177)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateProtocolStateValueStableV1BinableV1(
    pub  MinaStateProtocolStateValueStableV1BinableV1Poly<
        MinaStateProtocolStateValueStableV1BinableV1PolyArg0,
        MinaStateProtocolStateBodyValueStableV1Binable,
    >,
);

/// **Origin**: `Mina_state__Protocol_state.Value.Stable.V1.t`
///
/// **Location**: [src/lib/mina_state/protocol_state.ml:177:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L177)
pub type MinaStateProtocolStateValueStableV1Binable =
    crate::versioned::Versioned<MinaStateProtocolStateValueStableV1BinableV1, 1i32>;

/// Location: [src/lib/pickles/composition_types/composition_types.ml:155:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L155)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementV1PolyV1ProofStateV1DeferredValuesV1<
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
pub type PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementV1PolyV1ProofStateV1DeferredValues<
    Plonk,
    ScalarChallenge,
    Fp,
    Fq,
    BulletproofChallenges,
    Index,
> = crate::versioned::Versioned<
    PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementV1PolyV1ProofStateV1DeferredValuesV1<
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
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementV1PolyV1ProofStateV1 < Plonk , ScalarChallenge , Fp , Fq , MeOnly , Digest , BpChals , Index > { pub deferred_values : PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementV1PolyV1ProofStateV1DeferredValues < Plonk , ScalarChallenge , Fp , Fq , BpChals , Index > , pub sponge_digest_before_evaluations : Digest , pub me_only : MeOnly , }

/// Location: [src/lib/pickles/composition_types/composition_types.ml:299:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L299)
pub type PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementV1PolyV1ProofState<
    Plonk,
    ScalarChallenge,
    Fp,
    Fq,
    MeOnly,
    Digest,
    BpChals,
    Index,
> = crate::versioned::Versioned<
    PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementV1PolyV1ProofStateV1<
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
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementV1PolyV1<
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
    pub proof_state: PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementV1PolyV1ProofState<
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
pub type PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementV1Poly<
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
    PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementV1PolyV1<
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
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementV1PolyArg0V1<
    Challenge,
    ScalarChallenge,
> {
    pub alpha: ScalarChallenge,
    pub beta: Challenge,
    pub gamma: Challenge,
    pub zeta: ScalarChallenge,
}

/// Location: [src/lib/pickles/composition_types/composition_types.ml:62:14](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L62)
pub type PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementV1PolyArg0<
    Challenge,
    ScalarChallenge,
> = crate::versioned::Versioned<
    PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementV1PolyArg0V1<
        Challenge,
        ScalarChallenge,
    >,
    1i32,
>;

/// Location: [src/lib/pickles/composition_types/composition_types.ml:474:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L474)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementV1<
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
    pub  PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementV1Poly<
        PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementV1PolyArg0<
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
pub type PicklesProofBranching2ReprStableV1BinableV1PolyV1Statement<
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
    PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementV1<
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
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg0V1<A>(pub A, pub (A, ()));

/// Location: [src/lib/pickles_types/vector.ml:445:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L445)
pub type PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg0<A> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg0V1<A>,
        1i32,
    >;

/// Location: [src/lib/pickles/limb_vector/constant.ml:61:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/limb_vector/constant.ml#L61)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct LimbVectorConstantHex64StableV1BinableV1(pub i64);

/// **Origin**: `Limb_vector__Constant.Hex64.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/limb_vector/constant.ml:61:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/limb_vector/constant.ml#L61)
pub type LimbVectorConstantHex64StableV1Binable =
    crate::versioned::Versioned<LimbVectorConstantHex64StableV1BinableV1, 1i32>;

/// Location: [src/lib/pickles_types/scalar_challenge.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/scalar_challenge.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg1V1<F> {
    ScalarChallenge(F),
}

/// Location: [src/lib/pickles_types/scalar_challenge.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/scalar_challenge.ml#L6)
pub type PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg1<F> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg1V1<F>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/shifted_value.ml:31:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/shifted_value.ml#L31)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg2V1<F> {
    ShiftedValue(F),
}

/// Location: [src/lib/pickles_types/shifted_value.ml:31:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/shifted_value.ml#L31)
pub type PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg2<F> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg2V1<F>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/vector.ml:474:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L474)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CompositionTypesDigestConstantStableV1BinableV1PolyV1<A>(pub A, pub (A, (A, (A, ()))));

/// Location: [src/lib/pickles_types/vector.ml:474:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L474)
pub type CompositionTypesDigestConstantStableV1BinableV1Poly<A> =
    crate::versioned::Versioned<CompositionTypesDigestConstantStableV1BinableV1PolyV1<A>, 1i32>;

/// Location: [src/lib/pickles/composition_types/digest.ml:13:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/digest.ml#L13)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CompositionTypesDigestConstantStableV1BinableV1(
    pub CompositionTypesDigestConstantStableV1BinableV1Poly<LimbVectorConstantHex64StableV1Binable>,
);

/// **Origin**: `Composition_types__Digest.Constant.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/composition_types/digest.ml:13:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/digest.ml#L13)
pub type CompositionTypesDigestConstantStableV1Binable =
    crate::versioned::Versioned<CompositionTypesDigestConstantStableV1BinableV1, 1i32>;

/// Location: [src/lib/pickles_types/vector.ml:561:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L561)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg7V1PolyV1<A>(
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
pub type PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg7V1Poly<A> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg7V1PolyV1<A>,
        1i32,
    >;

/// Location: [src/lib/zexe_backend/pasta/basic.ml:54:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/zexe_backend/pasta/basic.ml#L54)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg7V1<A>(
    pub PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg7V1Poly<A>,
);

/// Location: [src/lib/zexe_backend/pasta/basic.ml:54:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/zexe_backend/pasta/basic.ml#L54)
pub type PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg7<A> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg7V1<A>,
        1i32,
    >;

/// Location: [src/lib/pickles/composition_types/bulletproof_challenge.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/bulletproof_challenge.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg7Arg0V1<Challenge> {
    pub prechallenge: Challenge,
}

/// Location: [src/lib/pickles/composition_types/bulletproof_challenge.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/bulletproof_challenge.ml#L6)
pub type PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg7Arg0<Challenge> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg7Arg0V1<Challenge>,
        1i32,
    >;

/// Location: [src/lib/pickles/composition_types/index.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/index.ml#L7)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CompositionTypesIndexStableV1BinableV1(pub crate::char_::Char);

/// **Origin**: `Composition_types__Index.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/composition_types/index.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/index.ml#L7)
pub type CompositionTypesIndexStableV1Binable =
    crate::versioned::Versioned<CompositionTypesIndexStableV1BinableV1, 1i32>;

/// Location: [src/lib/pickles/proof.ml:34:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L34)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyV1PrevEvalsV1<A>(pub A, pub A);

/// Location: [src/lib/pickles/proof.ml:34:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L34)
pub type PicklesProofBranching2ReprStableV1BinableV1PolyV1PrevEvals<A> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1BinableV1PolyV1PrevEvalsV1<A>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:30:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L30)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyV1PrevEvalsArg0V1<A> {
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
pub type PicklesProofBranching2ReprStableV1BinableV1PolyV1PrevEvalsArg0<A> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1BinableV1PolyV1PrevEvalsArg0V1<A>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L17)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyV1PrevEvalsArg0Arg0V1<A>(pub Vec<A>);

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L17)
pub type PicklesProofBranching2ReprStableV1BinableV1PolyV1PrevEvalsArg0Arg0<A> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1BinableV1PolyV1PrevEvalsArg0Arg0V1<A>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:203:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L203)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1MessagesV1LCommV1<G>(
    pub PicklesProofBranching2ReprStableV1BinableV1PolyV1PrevEvalsArg0Arg0<G>,
);

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:203:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L203)
pub type PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1MessagesV1LComm<G> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1MessagesV1LCommV1<G>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:149:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L149)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1MessagesV1TCommV1<GOpt> {
    pub unshifted: PicklesProofBranching2ReprStableV1BinableV1PolyV1PrevEvalsArg0Arg0<GOpt>,
    pub shifted: GOpt,
}

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:149:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L149)
pub type PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1MessagesV1TComm<GOpt> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1MessagesV1TCommV1<GOpt>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:218:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L218)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1MessagesV1<G, GOpt> {
    pub l_comm: PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1MessagesV1LComm<G>,
    pub r_comm: PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1MessagesV1LComm<G>,
    pub o_comm: PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1MessagesV1LComm<G>,
    pub z_comm: PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1MessagesV1LComm<G>,
    pub t_comm: PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1MessagesV1TComm<GOpt>,
}

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:218:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L218)
pub type PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1Messages<G, GOpt> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1MessagesV1<G, GOpt>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:102:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L102)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1OpeningsV1ProofV1<G, Fq> {
    pub lr: PicklesProofBranching2ReprStableV1BinableV1PolyV1PrevEvalsArg0Arg0<(G, G)>,
    pub z_1: Fq,
    pub z_2: Fq,
    pub delta: G,
    pub sg: G,
}

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:102:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L102)
pub type PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1OpeningsV1Proof<G, Fq> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1OpeningsV1ProofV1<G, Fq>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:124:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L124)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1OpeningsV1<G, Fq, Fqv> {
    pub proof: PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1OpeningsV1Proof<G, Fq>,
    pub evals: (
        PicklesProofBranching2ReprStableV1BinableV1PolyV1PrevEvalsArg0<Fqv>,
        PicklesProofBranching2ReprStableV1BinableV1PolyV1PrevEvalsArg0<Fqv>,
    ),
}

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:124:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L124)
pub type PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1Openings<G, Fq, Fqv> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1OpeningsV1<G, Fq, Fqv>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:250:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L250)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1<G, GOpt, Fq, Fqv> {
    pub messages: PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1Messages<G, GOpt>,
    pub openings:
        PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1Openings<G, Fq, Fqv>,
}

/// Location: [src/lib/pickles_types/dlog_plonk_types.ml:250:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/dlog_plonk_types.ml#L250)
pub type PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1Poly<G, GOpt, Fq, Fqv> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyV1<G, GOpt, Fq, Fqv>,
        1i32,
    >;

/// Location: [src/lib/zexe_backend/zexe_backend_common/curve.ml:97:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/zexe_backend/zexe_backend_common/curve.ml#L97)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyArg0(
    pub crate::bigint::BigInt,
    pub crate::bigint::BigInt,
);

/// Location: [src/lib/pickles_types/or_infinity.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/or_infinity.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyArg1V1<A> {
    Infinity,
    Finite(A),
}

/// Location: [src/lib/pickles_types/or_infinity.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/or_infinity.ml#L6)
pub type PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyArg1<A> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyArg1V1<A>,
        1i32,
    >;

/// Location: [src/lib/zexe_backend/zexe_backend_common/plonk_dlog_proof.ml:155:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/zexe_backend/zexe_backend_common/plonk_dlog_proof.ml#L155)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1(
    pub  PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1Poly<
        PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyArg0,
        PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyArg1<
            PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyArg0,
        >,
        crate::bigint::BigInt,
        PicklesProofBranching2ReprStableV1BinableV1PolyV1PrevEvalsArg0Arg0<crate::bigint::BigInt>,
    >,
);

/// Location: [src/lib/zexe_backend/zexe_backend_common/plonk_dlog_proof.ml:155:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/zexe_backend/zexe_backend_common/plonk_dlog_proof.ml#L155)
pub type PicklesProofBranching2ReprStableV1BinableV1PolyV1Proof =
    crate::versioned::Versioned<PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1, 1i32>;

/// Location: [src/lib/pickles/proof.ml:45:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L45)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyV1<DlogMeOnly, PairingMeOnly> {
    pub statement: PicklesProofBranching2ReprStableV1BinableV1PolyV1Statement<
        PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg0<
            LimbVectorConstantHex64StableV1Binable,
        >,
        PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg1<
            PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg0<
                LimbVectorConstantHex64StableV1Binable,
            >,
        >,
        PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg2<crate::bigint::BigInt>,
        crate::bigint::BigInt,
        DlogMeOnly,
        CompositionTypesDigestConstantStableV1Binable,
        PairingMeOnly,
        PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg7<
            PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg7Arg0<
                PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg1<
                    PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg0<
                        LimbVectorConstantHex64StableV1Binable,
                    >,
                >,
            >,
        >,
        CompositionTypesIndexStableV1Binable,
    >,
    pub prev_evals: PicklesProofBranching2ReprStableV1BinableV1PolyV1PrevEvals<
        PicklesProofBranching2ReprStableV1BinableV1PolyV1PrevEvalsArg0<
            PicklesProofBranching2ReprStableV1BinableV1PolyV1PrevEvalsArg0Arg0<
                crate::bigint::BigInt,
            >,
        >,
    >,
    pub prev_x_hat:
        PicklesProofBranching2ReprStableV1BinableV1PolyV1PrevEvals<crate::bigint::BigInt>,
    pub proof: PicklesProofBranching2ReprStableV1BinableV1PolyV1Proof,
}

/// Location: [src/lib/pickles/proof.ml:45:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L45)
pub type PicklesProofBranching2ReprStableV1BinableV1Poly<DlogMeOnly, PairingMeOnly> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1BinableV1PolyV1<DlogMeOnly, PairingMeOnly>,
        1i32,
    >;

/// Location: [src/lib/pickles/composition_types/composition_types.ml:275:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L275)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyArg0V1<G1, BulletproofChallenges> {
    pub sg: G1,
    pub old_bulletproof_challenges: BulletproofChallenges,
}

/// Location: [src/lib/pickles/composition_types/composition_types.ml:275:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L275)
pub type PicklesProofBranching2ReprStableV1BinableV1PolyArg0<G1, BulletproofChallenges> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1BinableV1PolyArg0V1<G1, BulletproofChallenges>,
        1i32,
    >;

/// Location: [src/lib/zexe_backend/zexe_backend_common/curve.ml:97:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/zexe_backend/zexe_backend_common/curve.ml#L97)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyArg0Arg0(
    pub crate::bigint::BigInt,
    pub crate::bigint::BigInt,
);

/// Location: [src/lib/pickles_types/vector.ml:532:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L532)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1BinableV1PolyV1PolyV1<A>(
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
pub type PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1BinableV1PolyV1Poly<A> =
    crate::versioned::Versioned<
        PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1BinableV1PolyV1PolyV1<A>,
        1i32,
    >;

/// Location: [src/lib/zexe_backend/pasta/basic.ml:33:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/zexe_backend/pasta/basic.ml#L33)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1BinableV1PolyV1<A>(
    pub PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1BinableV1PolyV1Poly<A>,
);

/// Location: [src/lib/zexe_backend/pasta/basic.ml:33:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/zexe_backend/pasta/basic.ml#L33)
pub type PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1BinableV1Poly<A> =
    crate::versioned::Versioned<
        PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1BinableV1PolyV1<A>,
        1i32,
    >;

/// Location: [src/lib/pickles/reduced_me_only.ml:38:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/reduced_me_only.ml#L38)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1BinableV1(
    pub  PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1BinableV1Poly<
        PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg7Arg0<
            PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg1<
                PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg0<
                    LimbVectorConstantHex64StableV1Binable,
                >,
            >,
        >,
    >,
);

/// **Origin**: `Pickles__Reduced_me_only.Dlog_based.Challenges_vector.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/reduced_me_only.ml:38:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/reduced_me_only.ml#L38)
pub type PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1Binable = crate::versioned::Versioned<
    PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1BinableV1,
    1i32,
>;

/// Location: [src/lib/pickles/reduced_me_only.ml:16:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/reduced_me_only.ml#L16)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyArg1V1<S, Sgs, Bpcs> {
    pub app_state: S,
    pub sg: Sgs,
    pub old_bulletproof_challenges: Bpcs,
}

/// Location: [src/lib/pickles/reduced_me_only.ml:16:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/reduced_me_only.ml#L16)
pub type PicklesProofBranching2ReprStableV1BinableV1PolyArg1<S, Sgs, Bpcs> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1BinableV1PolyArg1V1<S, Sgs, Bpcs>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/at_most.ml:106:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/at_most.ml#L106)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyArg1Arg1V1<A>(pub Vec<A>);

/// Location: [src/lib/pickles_types/at_most.ml:106:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/at_most.ml#L106)
pub type PicklesProofBranching2ReprStableV1BinableV1PolyArg1Arg1<A> =
    crate::versioned::Versioned<PicklesProofBranching2ReprStableV1BinableV1PolyArg1Arg1V1<A>, 1i32>;

/// Location: [src/lib/pickles/proof.ml:283:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L283)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1BinableV1(
    pub  PicklesProofBranching2ReprStableV1BinableV1Poly<
        PicklesProofBranching2ReprStableV1BinableV1PolyArg0<
            PicklesProofBranching2ReprStableV1BinableV1PolyArg0Arg0,
            PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg0<
                PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1Binable,
            >,
        >,
        PicklesProofBranching2ReprStableV1BinableV1PolyArg1<
            (),
            PicklesProofBranching2ReprStableV1BinableV1PolyArg1Arg1<
                PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyArg0,
            >,
            PicklesProofBranching2ReprStableV1BinableV1PolyArg1Arg1<
                PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg7<
                    PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg7Arg0<
                        PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg1<
                            PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg0<
                                LimbVectorConstantHex64StableV1Binable,
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
pub type PicklesProofBranching2ReprStableV1Binable =
    crate::versioned::Versioned<PicklesProofBranching2ReprStableV1BinableV1, 1i32>;

/// Location: [src/lib/pickles/proof.ml:318:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L318)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2StableV1BinableV1(pub PicklesProofBranching2ReprStableV1Binable);

/// **Origin**: `Pickles__Proof.Branching_2.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/proof.ml:318:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L318)
pub type PicklesProofBranching2StableV1Binable =
    crate::versioned::Versioned<PicklesProofBranching2StableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/proof.ml:12:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/proof.ml#L12)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseProofStableV1BinableV1(pub PicklesProofBranching2StableV1Binable);

/// **Origin**: `Mina_base__Proof.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/proof.ml:12:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/proof.ml#L12)
pub type MinaBaseProofStableV1Binable =
    crate::versioned::Versioned<MinaBaseProofStableV1BinableV1, 1i32>;

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L10)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1BinableV1PolyV1CoinbaseV1<A> {
    Zero,
    One(Option<A>),
    Two(Option<(A, Option<A>)>),
}

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L10)
pub type StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1BinableV1PolyV1Coinbase<A> =
    crate::versioned::Versioned<
        StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1BinableV1PolyV1CoinbaseV1<A>,
        1i32,
    >;

/// Location: [src/lib/currency/currency.ml:586:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/currency.ml#L586)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CurrencyFeeStableV1BinableV1(pub UnsignedExtendedUInt64StableV1Binable);

/// **Origin**: `Currency.Fee.Stable.V1.t`
///
/// **Location**: [src/lib/currency/currency.ml:586:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/currency.ml#L586)
pub type CurrencyFeeStableV1Binable =
    crate::versioned::Versioned<CurrencyFeeStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/coinbase_fee_transfer.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/coinbase_fee_transfer.ml#L7)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseCoinbaseFeeTransferStableV1BinableV1 {
    pub receiver_pk: ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8,
    pub fee: CurrencyFeeStableV1Binable,
}

/// **Origin**: `Mina_base__Coinbase_fee_transfer.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/coinbase_fee_transfer.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/coinbase_fee_transfer.ml#L7)
pub type MinaBaseCoinbaseFeeTransferStableV1Binable =
    crate::versioned::Versioned<MinaBaseCoinbaseFeeTransferStableV1BinableV1, 1i32>;

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:66:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L66)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffFtStableV1BinableV1(pub MinaBaseCoinbaseFeeTransferStableV1Binable);

/// **Origin**: `Staged_ledger_diff.Ft.Stable.V1.t`
///
/// **Location**: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:66:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L66)
pub type StagedLedgerDiffFtStableV1Binable =
    crate::versioned::Versioned<StagedLedgerDiffFtStableV1BinableV1, 1i32>;

/// Location: [src/lib/currency/currency.ml:744:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/currency.ml#L744)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CurrencyBalanceStableV1BinableV1(pub CurrencyAmountMakeStrStableV1Binable);

/// **Origin**: `Currency.Balance.Stable.V1.t`
///
/// **Location**: [src/lib/currency/currency.ml:744:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/currency.ml#L744)
pub type CurrencyBalanceStableV1Binable =
    crate::versioned::Versioned<CurrencyBalanceStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/transaction_status.ml:711:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L711)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseTransactionStatusCoinbaseBalanceDataStableV1BinableV1 {
    pub coinbase_receiver_balance: CurrencyBalanceStableV1Binable,
    pub fee_transfer_receiver_balance: Option<CurrencyBalanceStableV1Binable>,
}

/// **Origin**: `Mina_base__Transaction_status.Coinbase_balance_data.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_status.ml:711:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L711)
pub type MinaBaseTransactionStatusCoinbaseBalanceDataStableV1Binable = crate::versioned::Versioned<
    MinaBaseTransactionStatusCoinbaseBalanceDataStableV1BinableV1,
    1i32,
>;

/// Location: [src/lib/mina_base/transaction_status.ml:754:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L754)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseTransactionStatusFeeTransferBalanceDataStableV1BinableV1 {
    pub receiver1_balance: CurrencyBalanceStableV1Binable,
    pub receiver2_balance: Option<CurrencyBalanceStableV1Binable>,
}

/// **Origin**: `Mina_base__Transaction_status.Fee_transfer_balance_data.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_status.ml:754:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L754)
pub type MinaBaseTransactionStatusFeeTransferBalanceDataStableV1Binable =
    crate::versioned::Versioned<
        MinaBaseTransactionStatusFeeTransferBalanceDataStableV1BinableV1,
        1i32,
    >;

/// Location: [src/lib/mina_base/transaction_status.ml:795:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L795)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseTransactionStatusInternalCommandBalanceDataStableV1BinableV1 {
    Coinbase(MinaBaseTransactionStatusCoinbaseBalanceDataStableV1Binable),
    FeeTransfer(MinaBaseTransactionStatusFeeTransferBalanceDataStableV1Binable),
}

/// **Origin**: `Mina_base__Transaction_status.Internal_command_balance_data.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_status.ml:795:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L795)
pub type MinaBaseTransactionStatusInternalCommandBalanceDataStableV1Binable =
    crate::versioned::Versioned<
        MinaBaseTransactionStatusInternalCommandBalanceDataStableV1BinableV1,
        1i32,
    >;

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:82:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L82)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1BinableV1PolyV1<A, B> {
    pub completed_works: Vec<A>,
    pub commands: Vec<B>,
    pub coinbase: StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1BinableV1PolyV1Coinbase<
        StagedLedgerDiffFtStableV1Binable,
    >,
    pub internal_command_balances:
        Vec<MinaBaseTransactionStatusInternalCommandBalanceDataStableV1Binable>,
}

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:82:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L82)
pub type StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1BinableV1Poly<A, B> =
    crate::versioned::Versioned<
        StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1BinableV1PolyV1<A, B>,
        1i32,
    >;

/// Location: [src/lib/one_or_two/one_or_two.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/one_or_two/one_or_two.ml#L7)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[polymorphic_variant]
pub enum TransactionSnarkWorkTStableV1BinableV1ProofsV1<A> {
    One(A),
    Two((A, A)),
}

/// Location: [src/lib/one_or_two/one_or_two.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/one_or_two/one_or_two.ml#L7)
pub type TransactionSnarkWorkTStableV1BinableV1Proofs<A> =
    crate::versioned::Versioned<TransactionSnarkWorkTStableV1BinableV1ProofsV1<A>, 1i32>;

/// Location: [src/lib/transaction_snark/transaction_snark.ml:117:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L117)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkStatementWithSokStableV1BinableV1PolyV1<
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
pub type TransactionSnarkStatementWithSokStableV1BinableV1Poly<
    LedgerHash,
    Amount,
    PendingCoinbase,
    FeeExcess,
    TokenId,
    SokDigest,
> = crate::versioned::Versioned<
    TransactionSnarkStatementWithSokStableV1BinableV1PolyV1<
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
pub struct TransactionSnarkPendingCoinbaseStackStateStableV1BinableV1PolyV1<PendingCoinbase> {
    pub source: PendingCoinbase,
    pub target: PendingCoinbase,
}

/// Location: [src/lib/transaction_snark/transaction_snark.ml:63:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L63)
pub type TransactionSnarkPendingCoinbaseStackStateStableV1BinableV1Poly<PendingCoinbase> =
    crate::versioned::Versioned<
        TransactionSnarkPendingCoinbaseStackStateStableV1BinableV1PolyV1<PendingCoinbase>,
        1i32,
    >;

/// Location: [src/lib/mina_base/pending_coinbase.ml:494:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L494)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStackVersionedStableV1BinableV1PolyV1<DataStack, StateStack> {
    pub data: DataStack,
    pub state: StateStack,
}

/// Location: [src/lib/mina_base/pending_coinbase.ml:494:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L494)
pub type MinaBasePendingCoinbaseStackVersionedStableV1BinableV1Poly<DataStack, StateStack> =
    crate::versioned::Versioned<
        MinaBasePendingCoinbaseStackVersionedStableV1BinableV1PolyV1<DataStack, StateStack>,
        1i32,
    >;

/// Location: [src/lib/mina_base/pending_coinbase.ml:154:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L154)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStackVersionedStableV1BinableV1PolyArg0V1Poly(
    pub crate::bigint::BigInt,
);

/// Location: [src/lib/mina_base/pending_coinbase.ml:153:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L153)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStackVersionedStableV1BinableV1PolyArg0V1(
    pub MinaBasePendingCoinbaseStackVersionedStableV1BinableV1PolyArg0V1Poly,
);

/// Location: [src/lib/mina_base/pending_coinbase.ml:153:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L153)
pub type MinaBasePendingCoinbaseStackVersionedStableV1BinableV1PolyArg0 =
    crate::versioned::Versioned<
        MinaBasePendingCoinbaseStackVersionedStableV1BinableV1PolyArg0V1,
        1i32,
    >;

/// Location: [src/lib/mina_base/pending_coinbase.ml:238:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L238)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStateStackStableV1BinableV1PolyV1<StackHash> {
    pub init: StackHash,
    pub curr: StackHash,
}

/// Location: [src/lib/mina_base/pending_coinbase.ml:238:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L238)
pub type MinaBasePendingCoinbaseStateStackStableV1BinableV1Poly<StackHash> =
    crate::versioned::Versioned<
        MinaBasePendingCoinbaseStateStackStableV1BinableV1PolyV1<StackHash>,
        1i32,
    >;

/// Location: [src/lib/mina_base/pending_coinbase.ml:213:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L213)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStateStackStableV1BinableV1PolyArg0V1Poly(
    pub crate::bigint::BigInt,
);

/// Location: [src/lib/mina_base/pending_coinbase.ml:212:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L212)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStateStackStableV1BinableV1PolyArg0V1(
    pub MinaBasePendingCoinbaseStateStackStableV1BinableV1PolyArg0V1Poly,
);

/// Location: [src/lib/mina_base/pending_coinbase.ml:212:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L212)
pub type MinaBasePendingCoinbaseStateStackStableV1BinableV1PolyArg0 =
    crate::versioned::Versioned<MinaBasePendingCoinbaseStateStackStableV1BinableV1PolyArg0V1, 1i32>;

/// Location: [src/lib/mina_base/pending_coinbase.ml:247:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L247)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStateStackStableV1BinableV1(
    pub  MinaBasePendingCoinbaseStateStackStableV1BinableV1Poly<
        MinaBasePendingCoinbaseStateStackStableV1BinableV1PolyArg0,
    >,
);

/// **Origin**: `Mina_base__Pending_coinbase.State_stack.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/pending_coinbase.ml:247:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L247)
pub type MinaBasePendingCoinbaseStateStackStableV1Binable =
    crate::versioned::Versioned<MinaBasePendingCoinbaseStateStackStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/pending_coinbase.ml:504:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L504)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStackVersionedStableV1BinableV1(
    pub  MinaBasePendingCoinbaseStackVersionedStableV1BinableV1Poly<
        MinaBasePendingCoinbaseStackVersionedStableV1BinableV1PolyArg0,
        MinaBasePendingCoinbaseStateStackStableV1Binable,
    >,
);

/// **Origin**: `Mina_base__Pending_coinbase.Stack_versioned.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/pending_coinbase.ml:504:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L504)
pub type MinaBasePendingCoinbaseStackVersionedStableV1Binable =
    crate::versioned::Versioned<MinaBasePendingCoinbaseStackVersionedStableV1BinableV1, 1i32>;

/// Location: [src/lib/transaction_snark/transaction_snark.ml:87:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L87)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkPendingCoinbaseStackStateStableV1BinableV1(
    pub  TransactionSnarkPendingCoinbaseStackStateStableV1BinableV1Poly<
        MinaBasePendingCoinbaseStackVersionedStableV1Binable,
    >,
);

/// **Origin**: `Transaction_snark.Pending_coinbase_stack_state.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark/transaction_snark.ml:87:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L87)
pub type TransactionSnarkPendingCoinbaseStackStateStableV1Binable =
    crate::versioned::Versioned<TransactionSnarkPendingCoinbaseStackStateStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/fee_excess.ml:54:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_excess.ml#L54)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeExcessStableV1BinableV1PolyV1<Token, Fee> {
    pub fee_token_l: Token,
    pub fee_excess_l: Fee,
    pub fee_token_r: Token,
    pub fee_excess_r: Fee,
}

/// Location: [src/lib/mina_base/fee_excess.ml:54:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_excess.ml#L54)
pub type MinaBaseFeeExcessStableV1BinableV1Poly<Token, Fee> =
    crate::versioned::Versioned<MinaBaseFeeExcessStableV1BinableV1PolyV1<Token, Fee>, 1i32>;

/// Location: [src/lib/currency/signed_poly.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/signed_poly.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeExcessStableV1BinableV1PolyArg1V1<Magnitude, Sgn> {
    pub magnitude: Magnitude,
    pub sgn: Sgn,
}

/// Location: [src/lib/currency/signed_poly.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/signed_poly.ml#L6)
pub type MinaBaseFeeExcessStableV1BinableV1PolyArg1<Magnitude, Sgn> =
    crate::versioned::Versioned<MinaBaseFeeExcessStableV1BinableV1PolyArg1V1<Magnitude, Sgn>, 1i32>;

/// Location: [src/lib/sgn/sgn.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/sgn/sgn.ml#L9)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum SgnStableV1BinableV1 {
    Pos,
    Neg,
}

/// **Origin**: `Sgn.Stable.V1.t`
///
/// **Location**: [src/lib/sgn/sgn.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/sgn/sgn.ml#L9)
pub type SgnStableV1Binable = crate::versioned::Versioned<SgnStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/fee_excess.ml:123:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_excess.ml#L123)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeExcessStableV1BinableV1(
    pub  MinaBaseFeeExcessStableV1BinableV1Poly<
        MinaBaseTokenIdStableV1Binable,
        MinaBaseFeeExcessStableV1BinableV1PolyArg1<CurrencyFeeStableV1Binable, SgnStableV1Binable>,
    >,
);

/// **Origin**: `Mina_base__Fee_excess.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/fee_excess.ml:123:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_excess.ml#L123)
pub type MinaBaseFeeExcessStableV1Binable =
    crate::versioned::Versioned<MinaBaseFeeExcessStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/sok_message.ml:25:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/sok_message.ml#L25)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSokMessageDigestStableV1BinableV1(pub crate::string::String);

/// **Origin**: `Mina_base__Sok_message.Digest.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/sok_message.ml:25:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/sok_message.ml#L25)
pub type MinaBaseSokMessageDigestStableV1Binable =
    crate::versioned::Versioned<MinaBaseSokMessageDigestStableV1BinableV1, 1i32>;

/// Location: [src/lib/transaction_snark/transaction_snark.ml:220:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L220)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkStatementWithSokStableV1BinableV1(
    pub  TransactionSnarkStatementWithSokStableV1BinableV1Poly<
        MinaBaseStagedLedgerHashNonSnarkStableV1BinableV1LedgerHash,
        CurrencyAmountMakeStrStableV1Binable,
        TransactionSnarkPendingCoinbaseStackStateStableV1Binable,
        MinaBaseFeeExcessStableV1Binable,
        MinaBaseTokenIdStableV1Binable,
        MinaBaseSokMessageDigestStableV1Binable,
    >,
);

/// **Origin**: `Transaction_snark.Statement.With_sok.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark/transaction_snark.ml:220:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L220)
pub type TransactionSnarkStatementWithSokStableV1Binable =
    crate::versioned::Versioned<TransactionSnarkStatementWithSokStableV1BinableV1, 1i32>;

/// Location: [src/lib/transaction_snark/transaction_snark.ml:420:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L420)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkProofStableV1BinableV1(pub PicklesProofBranching2StableV1Binable);

/// **Origin**: `Transaction_snark.Proof.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark/transaction_snark.ml:420:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L420)
pub type TransactionSnarkProofStableV1Binable =
    crate::versioned::Versioned<TransactionSnarkProofStableV1BinableV1, 1i32>;

/// Location: [src/lib/transaction_snark/transaction_snark.ml:432:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L432)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkStableV1BinableV1 {
    pub statement: TransactionSnarkStatementWithSokStableV1Binable,
    pub proof: TransactionSnarkProofStableV1Binable,
}

/// **Origin**: `Transaction_snark.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark/transaction_snark.ml:432:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L432)
pub type TransactionSnarkStableV1Binable =
    crate::versioned::Versioned<TransactionSnarkStableV1BinableV1, 1i32>;

/// Location: [src/lib/ledger_proof/ledger_proof.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/ledger_proof/ledger_proof.ml#L10)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct LedgerProofProdStableV1BinableV1(pub TransactionSnarkStableV1Binable);

/// **Origin**: `Ledger_proof.Prod.Stable.V1.t`
///
/// **Location**: [src/lib/ledger_proof/ledger_proof.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/ledger_proof/ledger_proof.ml#L10)
pub type LedgerProofProdStableV1Binable =
    crate::versioned::Versioned<LedgerProofProdStableV1BinableV1, 1i32>;

/// Location: [src/lib/transaction_snark_work/transaction_snark_work.ml:82:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_work/transaction_snark_work.ml#L82)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkWorkTStableV1BinableV1 {
    pub fee: CurrencyFeeStableV1Binable,
    pub proofs: TransactionSnarkWorkTStableV1BinableV1Proofs<LedgerProofProdStableV1Binable>,
    pub prover: ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8,
}

/// **Origin**: `Transaction_snark_work.T.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark_work/transaction_snark_work.ml:82:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_work/transaction_snark_work.ml#L82)
pub type TransactionSnarkWorkTStableV1Binable =
    crate::versioned::Versioned<TransactionSnarkWorkTStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/transaction_status.ml:809:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L809)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseTransactionStatusAuxiliaryDataStableV1BinableV1 {
    pub fee_payer_account_creation_fee_paid: Option<CurrencyAmountMakeStrStableV1Binable>,
    pub receiver_account_creation_fee_paid: Option<CurrencyAmountMakeStrStableV1Binable>,
    pub created_token: Option<MinaBaseTokenIdStableV1Binable>,
}

/// **Origin**: `Mina_base__Transaction_status.Auxiliary_data.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_status.ml:809:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L809)
pub type MinaBaseTransactionStatusAuxiliaryDataStableV1Binable =
    crate::versioned::Versioned<MinaBaseTransactionStatusAuxiliaryDataStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/transaction_status.ml:692:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L692)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseTransactionStatusBalanceDataStableV1BinableV1 {
    pub fee_payer_balance: Option<CurrencyBalanceStableV1Binable>,
    pub source_balance: Option<CurrencyBalanceStableV1Binable>,
    pub receiver_balance: Option<CurrencyBalanceStableV1Binable>,
}

/// **Origin**: `Mina_base__Transaction_status.Balance_data.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_status.ml:692:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L692)
pub type MinaBaseTransactionStatusBalanceDataStableV1Binable =
    crate::versioned::Versioned<MinaBaseTransactionStatusBalanceDataStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/transaction_status.ml:13:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L13)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseTransactionStatusFailureStableV1BinableV1 {
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
pub type MinaBaseTransactionStatusFailureStableV1Binable =
    crate::versioned::Versioned<MinaBaseTransactionStatusFailureStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/transaction_status.ml:832:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L832)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseTransactionStatusStableV1BinableV1 {
    Applied(
        MinaBaseTransactionStatusAuxiliaryDataStableV1Binable,
        MinaBaseTransactionStatusBalanceDataStableV1Binable,
    ),
    Failed(
        MinaBaseTransactionStatusFailureStableV1Binable,
        MinaBaseTransactionStatusBalanceDataStableV1Binable,
    ),
}

/// **Origin**: `Mina_base__Transaction_status.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/transaction_status.ml:832:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L832)
pub type MinaBaseTransactionStatusStableV1Binable =
    crate::versioned::Versioned<MinaBaseTransactionStatusStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/with_status.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1BinableV1PolyArg1V1<A> {
    pub data: A,
    pub status: MinaBaseTransactionStatusStableV1Binable,
}

/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/with_status.ml#L6)
pub type StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1BinableV1PolyArg1<A> =
    crate::versioned::Versioned<
        StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1BinableV1PolyArg1V1<A>,
        1i32,
    >;

/// Location: [src/lib/mina_base/user_command.ml:7:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/user_command.ml#L7)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseUserCommandStableV1BinableV1PolyV1<U, S> {
    SignedCommand(U),
    SnappCommand(S),
}

/// Location: [src/lib/mina_base/user_command.ml:7:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/user_command.ml#L7)
pub type MinaBaseUserCommandStableV1BinableV1Poly<U, S> =
    crate::versioned::Versioned<MinaBaseUserCommandStableV1BinableV1PolyV1<U, S>, 1i32>;

/// Location: [src/lib/mina_base/signed_command.ml:13:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command.ml#L13)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandStableV1BinableV1PolyV1<Payload, Pk, Signature> {
    pub payload: Payload,
    pub signer: Pk,
    pub signature: Signature,
}

/// Location: [src/lib/mina_base/signed_command.ml:13:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command.ml#L13)
pub type MinaBaseSignedCommandStableV1BinableV1Poly<Payload, Pk, Signature> =
    crate::versioned::Versioned<
        MinaBaseSignedCommandStableV1BinableV1PolyV1<Payload, Pk, Signature>,
        1i32,
    >;

/// Location: [src/lib/mina_base/signed_command_payload.ml:402:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L402)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandPayloadStableV1BinableV1PolyV1<Common, Body> {
    pub common: Common,
    pub body: Body,
}

/// Location: [src/lib/mina_base/signed_command_payload.ml:402:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L402)
pub type MinaBaseSignedCommandPayloadStableV1BinableV1Poly<Common, Body> =
    crate::versioned::Versioned<
        MinaBaseSignedCommandPayloadStableV1BinableV1PolyV1<Common, Body>,
        1i32,
    >;

/// Location: [src/lib/mina_base/signed_command_payload.ml:23:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L23)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandPayloadCommonBinableArgStableV1BinableV1PolyV1<
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
pub type MinaBaseSignedCommandPayloadCommonBinableArgStableV1BinableV1Poly<
    Fee,
    PublicKey,
    TokenId,
    Nonce,
    GlobalSlot,
    Memo,
> = crate::versioned::Versioned<
    MinaBaseSignedCommandPayloadCommonBinableArgStableV1BinableV1PolyV1<
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
pub struct MinaNumbersNatMake32StableV1BinableV1(pub UnsignedExtendedUInt32StableV1Binable);

/// **Origin**: `Mina_numbers__Nat.Make32.Stable.V1.t`
///
/// **Location**: [src/lib/mina_numbers/nat.ml:186:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L186)
pub type MinaNumbersNatMake32StableV1Binable =
    crate::versioned::Versioned<MinaNumbersNatMake32StableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/signed_command_memo.ml:16:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_memo.ml#L16)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandMemoStableV1BinableV1(pub crate::string::String);

/// **Origin**: `Mina_base__Signed_command_memo.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command_memo.ml:16:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_memo.ml#L16)
pub type MinaBaseSignedCommandMemoStableV1Binable =
    crate::versioned::Versioned<MinaBaseSignedCommandMemoStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/signed_command_payload.ml:61:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L61)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandPayloadCommonBinableArgStableV1BinableV1(
    pub  MinaBaseSignedCommandPayloadCommonBinableArgStableV1BinableV1Poly<
        CurrencyFeeStableV1Binable,
        ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8,
        MinaBaseTokenIdStableV1Binable,
        MinaNumbersNatMake32StableV1Binable,
        ConsensusGlobalSlotStableV1BinableV1PolyArg0,
        MinaBaseSignedCommandMemoStableV1Binable,
    >,
);

/// **Origin**: `Mina_base__Signed_command_payload.Common.Binable_arg.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command_payload.ml:61:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L61)
pub type MinaBaseSignedCommandPayloadCommonBinableArgStableV1Binable = crate::versioned::Versioned<
    MinaBaseSignedCommandPayloadCommonBinableArgStableV1BinableV1,
    1i32,
>;

/// Location: [src/lib/mina_base/signed_command_payload.ml:83:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L83)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandPayloadCommonStableV1BinableV1(
    pub MinaBaseSignedCommandPayloadCommonBinableArgStableV1Binable,
);

/// **Origin**: `Mina_base__Signed_command_payload.Common.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command_payload.ml:83:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L83)
pub type MinaBaseSignedCommandPayloadCommonStableV1Binable =
    crate::versioned::Versioned<MinaBaseSignedCommandPayloadCommonStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/payment_payload.ml:21:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/payment_payload.ml#L21)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePaymentPayloadStableV1BinableV1PolyV1<PublicKey, TokenId, Amount> {
    pub source_pk: PublicKey,
    pub receiver_pk: PublicKey,
    pub token_id: TokenId,
    pub amount: Amount,
}

/// Location: [src/lib/mina_base/payment_payload.ml:21:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/payment_payload.ml#L21)
pub type MinaBasePaymentPayloadStableV1BinableV1Poly<PublicKey, TokenId, Amount> =
    crate::versioned::Versioned<
        MinaBasePaymentPayloadStableV1BinableV1PolyV1<PublicKey, TokenId, Amount>,
        1i32,
    >;

/// Location: [src/lib/mina_base/payment_payload.ml:35:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/payment_payload.ml#L35)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePaymentPayloadStableV1BinableV1(
    pub  MinaBasePaymentPayloadStableV1BinableV1Poly<
        ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8,
        MinaBaseTokenIdStableV1Binable,
        CurrencyAmountMakeStrStableV1Binable,
    >,
);

/// **Origin**: `Mina_base__Payment_payload.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/payment_payload.ml:35:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/payment_payload.ml#L35)
pub type MinaBasePaymentPayloadStableV1Binable =
    crate::versioned::Versioned<MinaBasePaymentPayloadStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/stake_delegation.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/stake_delegation.ml#L9)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseStakeDelegationStableV1BinableV1 {
    SetDelegate {
        delegator: ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8,
        new_delegate: ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8,
    },
}

/// **Origin**: `Mina_base__Stake_delegation.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/stake_delegation.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/stake_delegation.ml#L9)
pub type MinaBaseStakeDelegationStableV1Binable =
    crate::versioned::Versioned<MinaBaseStakeDelegationStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/new_token_payload.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/new_token_payload.ml#L7)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseNewTokenPayloadStableV1BinableV1 {
    pub token_owner_pk: ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8,
    pub disable_new_accounts: bool,
}

/// **Origin**: `Mina_base__New_token_payload.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/new_token_payload.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/new_token_payload.ml#L7)
pub type MinaBaseNewTokenPayloadStableV1Binable =
    crate::versioned::Versioned<MinaBaseNewTokenPayloadStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/new_account_payload.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/new_account_payload.ml#L7)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseNewAccountPayloadStableV1BinableV1 {
    pub token_id: MinaBaseTokenIdStableV1Binable,
    pub token_owner_pk: ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8,
    pub receiver_pk: ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8,
    pub account_disabled: bool,
}

/// **Origin**: `Mina_base__New_account_payload.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/new_account_payload.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/new_account_payload.ml#L7)
pub type MinaBaseNewAccountPayloadStableV1Binable =
    crate::versioned::Versioned<MinaBaseNewAccountPayloadStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/minting_payload.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/minting_payload.ml#L7)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseMintingPayloadStableV1BinableV1 {
    pub token_id: MinaBaseTokenIdStableV1Binable,
    pub token_owner_pk: ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8,
    pub receiver_pk: ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8,
    pub amount: CurrencyAmountMakeStrStableV1Binable,
}

/// **Origin**: `Mina_base__Minting_payload.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/minting_payload.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/minting_payload.ml#L7)
pub type MinaBaseMintingPayloadStableV1Binable =
    crate::versioned::Versioned<MinaBaseMintingPayloadStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/signed_command_payload.ml:197:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L197)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSignedCommandPayloadBodyBinableArgStableV1BinableV1 {
    Payment(MinaBasePaymentPayloadStableV1Binable),
    StakeDelegation(MinaBaseStakeDelegationStableV1Binable),
    CreateNewToken(MinaBaseNewTokenPayloadStableV1Binable),
    CreateTokenAccount(MinaBaseNewAccountPayloadStableV1Binable),
    MintTokens(MinaBaseMintingPayloadStableV1Binable),
}

/// **Origin**: `Mina_base__Signed_command_payload.Body.Binable_arg.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command_payload.ml:197:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L197)
pub type MinaBaseSignedCommandPayloadBodyBinableArgStableV1Binable =
    crate::versioned::Versioned<MinaBaseSignedCommandPayloadBodyBinableArgStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/signed_command_payload.ml:244:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L244)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandPayloadBodyStableV1BinableV1(
    pub MinaBaseSignedCommandPayloadBodyBinableArgStableV1Binable,
);

/// **Origin**: `Mina_base__Signed_command_payload.Body.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command_payload.ml:244:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L244)
pub type MinaBaseSignedCommandPayloadBodyStableV1Binable =
    crate::versioned::Versioned<MinaBaseSignedCommandPayloadBodyStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/signed_command_payload.ml:416:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L416)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandPayloadStableV1BinableV1(
    pub  MinaBaseSignedCommandPayloadStableV1BinableV1Poly<
        MinaBaseSignedCommandPayloadCommonStableV1Binable,
        MinaBaseSignedCommandPayloadBodyStableV1Binable,
    >,
);

/// **Origin**: `Mina_base__Signed_command_payload.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command_payload.ml:416:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L416)
pub type MinaBaseSignedCommandPayloadStableV1Binable =
    crate::versioned::Versioned<MinaBaseSignedCommandPayloadStableV1BinableV1, 1i32>;

/// Location: [src/lib/non_zero_curve_point/non_zero_curve_point.ml:194:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/non_zero_curve_point.ml#L194)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NonZeroCurvePointUncompressedStableV1BinableV1(
    pub ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8,
);

/// **Origin**: `Non_zero_curve_point.Uncompressed.Stable.V1.t`
///
/// **Location**: [src/lib/non_zero_curve_point/non_zero_curve_point.ml:194:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/non_zero_curve_point.ml#L194)
pub type NonZeroCurvePointUncompressedStableV1Binable =
    crate::versioned::Versioned<NonZeroCurvePointUncompressedStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/signature_poly.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signature_poly.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignatureStableV1BinableV1PolyV1<Field, Scalar>(pub Field, pub Scalar);

/// Location: [src/lib/mina_base/signature_poly.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signature_poly.ml#L6)
pub type MinaBaseSignatureStableV1BinableV1Poly<Field, Scalar> =
    crate::versioned::Versioned<MinaBaseSignatureStableV1BinableV1PolyV1<Field, Scalar>, 1i32>;

/// Location: [src/lib/mina_base/signature.ml:18:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signature.ml#L18)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignatureStableV1BinableV1(
    pub MinaBaseSignatureStableV1BinableV1Poly<crate::bigint::BigInt, crate::bigint::BigInt>,
);

/// **Origin**: `Mina_base__Signature.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signature.ml:18:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signature.ml#L18)
pub type MinaBaseSignatureStableV1Binable =
    crate::versioned::Versioned<MinaBaseSignatureStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/signed_command.ml:23:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command.ml#L23)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandStableV1BinableV1(
    pub  MinaBaseSignedCommandStableV1BinableV1Poly<
        MinaBaseSignedCommandPayloadStableV1Binable,
        NonZeroCurvePointUncompressedStableV1Binable,
        MinaBaseSignatureStableV1Binable,
    >,
);

/// **Origin**: `Mina_base__Signed_command.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/signed_command.ml:23:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command.ml#L23)
pub type MinaBaseSignedCommandStableV1Binable =
    crate::versioned::Versioned<MinaBaseSignedCommandStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/other_fee_payer.ml:10:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/other_fee_payer.ml#L10)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseOtherFeePayerPayloadStableV1BinableV1PolyV1<Pk, TokenId, Nonce, Fee> {
    pub pk: Pk,
    pub token_id: TokenId,
    pub nonce: Nonce,
    pub fee: Fee,
}

/// Location: [src/lib/mina_base/other_fee_payer.ml:10:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/other_fee_payer.ml#L10)
pub type MinaBaseOtherFeePayerPayloadStableV1BinableV1Poly<Pk, TokenId, Nonce, Fee> =
    crate::versioned::Versioned<
        MinaBaseOtherFeePayerPayloadStableV1BinableV1PolyV1<Pk, TokenId, Nonce, Fee>,
        1i32,
    >;

/// Location: [src/lib/mina_base/other_fee_payer.ml:20:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/other_fee_payer.ml#L20)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseOtherFeePayerPayloadStableV1BinableV1(
    pub  MinaBaseOtherFeePayerPayloadStableV1BinableV1Poly<
        ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8,
        MinaBaseTokenIdStableV1Binable,
        MinaNumbersNatMake32StableV1Binable,
        CurrencyFeeStableV1Binable,
    >,
);

/// **Origin**: `Mina_base__Other_fee_payer.Payload.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/other_fee_payer.ml:20:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/other_fee_payer.ml#L20)
pub type MinaBaseOtherFeePayerPayloadStableV1Binable =
    crate::versioned::Versioned<MinaBaseOtherFeePayerPayloadStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/other_fee_payer.ml:84:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/other_fee_payer.ml#L84)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseOtherFeePayerStableV1BinableV1 {
    pub payload: MinaBaseOtherFeePayerPayloadStableV1Binable,
    pub signature: MinaBaseSignatureStableV1Binable,
}

/// **Origin**: `Mina_base__Other_fee_payer.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/other_fee_payer.ml:84:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/other_fee_payer.ml#L84)
pub type MinaBaseOtherFeePayerStableV1Binable =
    crate::versioned::Versioned<MinaBaseOtherFeePayerStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:352:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L352)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandBinableArgStableV1BinableV1ProvedEmpty0V1<One, Two> {
    pub token_id: MinaBaseTokenIdStableV1Binable,
    pub fee_payment: Option<MinaBaseOtherFeePayerStableV1Binable>,
    pub one: One,
    pub two: Two,
}

/// Location: [src/lib/mina_base/snapp_command.ml:352:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L352)
pub type MinaBaseSnappCommandBinableArgStableV1BinableV1ProvedEmpty0<One, Two> =
    crate::versioned::Versioned<
        MinaBaseSnappCommandBinableArgStableV1BinableV1ProvedEmpty0V1<One, Two>,
        1i32,
    >;

/// Location: [src/lib/mina_base/snapp_command.ml:298:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L298)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyAuthorizedProvedStableV1BinableV1PolyV1<Data, Auth> {
    pub data: Data,
    pub authorization: Auth,
}

/// Location: [src/lib/mina_base/snapp_command.ml:298:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L298)
pub type MinaBaseSnappCommandPartyAuthorizedProvedStableV1BinableV1Poly<Data, Auth> =
    crate::versioned::Versioned<
        MinaBaseSnappCommandPartyAuthorizedProvedStableV1BinableV1PolyV1<Data, Auth>,
        1i32,
    >;

/// Location: [src/lib/mina_base/snapp_command.ml:211:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L211)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyPredicatedProvedStableV1BinableV1PolyV1<Body, Predicate> {
    pub body: Body,
    pub predicate: Predicate,
}

/// Location: [src/lib/mina_base/snapp_command.ml:211:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L211)
pub type MinaBaseSnappCommandPartyPredicatedProvedStableV1BinableV1Poly<Body, Predicate> =
    crate::versioned::Versioned<
        MinaBaseSnappCommandPartyPredicatedProvedStableV1BinableV1PolyV1<Body, Predicate>,
        1i32,
    >;

/// Location: [src/lib/mina_base/snapp_command.ml:136:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L136)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyBodyStableV1BinableV1PolyV1<Pk, Update, SignedAmount> {
    pub pk: Pk,
    pub update: Update,
    pub delta: SignedAmount,
}

/// Location: [src/lib/mina_base/snapp_command.ml:136:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L136)
pub type MinaBaseSnappCommandPartyBodyStableV1BinableV1Poly<Pk, Update, SignedAmount> =
    crate::versioned::Versioned<
        MinaBaseSnappCommandPartyBodyStableV1BinableV1PolyV1<Pk, Update, SignedAmount>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/vector.ml:503:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L503)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyUpdateStableV1BinableV1PolyV1AppStateV1PolyV1<A>(
    pub A,
    pub (A, (A, (A, (A, (A, (A, (A, ()))))))),
);

/// Location: [src/lib/pickles_types/vector.ml:503:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/vector.ml#L503)
pub type MinaBaseSnappCommandPartyUpdateStableV1BinableV1PolyV1AppStateV1Poly<A> =
    crate::versioned::Versioned<
        MinaBaseSnappCommandPartyUpdateStableV1BinableV1PolyV1AppStateV1PolyV1<A>,
        1i32,
    >;

/// Location: [src/lib/mina_base/snapp_state.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_state.ml#L17)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyUpdateStableV1BinableV1PolyV1AppStateV1<A>(
    pub MinaBaseSnappCommandPartyUpdateStableV1BinableV1PolyV1AppStateV1Poly<A>,
);

/// Location: [src/lib/mina_base/snapp_state.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_state.ml#L17)
pub type MinaBaseSnappCommandPartyUpdateStableV1BinableV1PolyV1AppState<A> =
    crate::versioned::Versioned<
        MinaBaseSnappCommandPartyUpdateStableV1BinableV1PolyV1AppStateV1<A>,
        1i32,
    >;

/// Location: [src/lib/mina_base/snapp_command.ml:35:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L35)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyUpdateStableV1BinableV1PolyV1<StateElement, Pk, Vk, Perms> {
    pub app_state: MinaBaseSnappCommandPartyUpdateStableV1BinableV1PolyV1AppState<StateElement>,
    pub delegate: Pk,
    pub verification_key: Vk,
    pub permissions: Perms,
}

/// Location: [src/lib/mina_base/snapp_command.ml:35:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L35)
pub type MinaBaseSnappCommandPartyUpdateStableV1BinableV1Poly<StateElement, Pk, Vk, Perms> =
    crate::versioned::Versioned<
        MinaBaseSnappCommandPartyUpdateStableV1BinableV1PolyV1<StateElement, Pk, Vk, Perms>,
        1i32,
    >;

/// Location: [src/lib/mina_base/snapp_basic.ml:87:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_basic.ml#L87)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSnappCommandPartyUpdateStableV1BinableV1PolyArg0V1<A> {
    Set(A),
    Keep,
}

/// Location: [src/lib/mina_base/snapp_basic.ml:87:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_basic.ml#L87)
pub type MinaBaseSnappCommandPartyUpdateStableV1BinableV1PolyArg0<A> = crate::versioned::Versioned<
    MinaBaseSnappCommandPartyUpdateStableV1BinableV1PolyArg0V1<A>,
    1i32,
>;

/// Location: [src/lib/with_hash/with_hash.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/with_hash/with_hash.ml#L8)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyUpdateStableV1BinableV1PolyArg0Arg0V1<A, H> {
    pub data: A,
    pub hash: H,
}

/// Location: [src/lib/with_hash/with_hash.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/with_hash/with_hash.ml#L8)
pub type MinaBaseSnappCommandPartyUpdateStableV1BinableV1PolyArg0Arg0<A, H> =
    crate::versioned::Versioned<
        MinaBaseSnappCommandPartyUpdateStableV1BinableV1PolyArg0Arg0V1<A, H>,
        1i32,
    >;

/// Location: [src/lib/pickles_types/at_most.ml:135:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/at_most.ml#L135)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesSideLoadedVerificationKeyRStableV1BinableV1PolyV1StepDataV1PolyV1<A>(pub Vec<A>);

/// Location: [src/lib/pickles_types/at_most.ml:135:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/at_most.ml#L135)
pub type PicklesSideLoadedVerificationKeyRStableV1BinableV1PolyV1StepDataV1Poly<A> =
    crate::versioned::Versioned<
        PicklesSideLoadedVerificationKeyRStableV1BinableV1PolyV1StepDataV1PolyV1<A>,
        1i32,
    >;

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:120:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L120)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesSideLoadedVerificationKeyRStableV1BinableV1PolyV1StepDataV1<A>(
    pub PicklesSideLoadedVerificationKeyRStableV1BinableV1PolyV1StepDataV1Poly<A>,
);

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:120:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L120)
pub type PicklesSideLoadedVerificationKeyRStableV1BinableV1PolyV1StepData<A> =
    crate::versioned::Versioned<
        PicklesSideLoadedVerificationKeyRStableV1BinableV1PolyV1StepDataV1<A>,
        1i32,
    >;

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:136:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L136)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesSideLoadedVerificationKeyRStableV1BinableV1PolyV1StepDataArg0V1<A> {
    pub h: A,
}

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:136:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L136)
pub type PicklesSideLoadedVerificationKeyRStableV1BinableV1PolyV1StepDataArg0<A> =
    crate::versioned::Versioned<
        PicklesSideLoadedVerificationKeyRStableV1BinableV1PolyV1StepDataArg0V1<A>,
        1i32,
    >;

/// Location: [src/lib/pickles_base/domain.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/domain.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum PicklesBaseDomainStableV1BinableV1 {
    Pow2RootsOfUnity(i32),
}

/// **Origin**: `Pickles_base__Domain.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_base/domain.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/domain.ml#L6)
pub type PicklesBaseDomainStableV1Binable =
    crate::versioned::Versioned<PicklesBaseDomainStableV1BinableV1, 1i32>;

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:44:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L44)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesBaseSideLoadedVerificationKeyWidthStableV1BinableV1(pub crate::char_::Char);

/// **Origin**: `Pickles_base__Side_loaded_verification_key.Width.Stable.V1.t`
///
/// **Location**: [src/lib/pickles_base/side_loaded_verification_key.ml:44:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L44)
pub type PicklesBaseSideLoadedVerificationKeyWidthStableV1Binable =
    crate::versioned::Versioned<PicklesBaseSideLoadedVerificationKeyWidthStableV1BinableV1, 1i32>;

/// Location: [src/lib/pickles_types/plonk_verification_key_evals.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_verification_key_evals.ml#L7)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesSideLoadedVerificationKeyRStableV1BinableV1PolyV1WrapIndexV1<Comm> {
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
pub type PicklesSideLoadedVerificationKeyRStableV1BinableV1PolyV1WrapIndex<Comm> =
    crate::versioned::Versioned<
        PicklesSideLoadedVerificationKeyRStableV1BinableV1PolyV1WrapIndexV1<Comm>,
        1i32,
    >;

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:150:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L150)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesSideLoadedVerificationKeyRStableV1BinableV1PolyV1<G> {
    pub step_data: PicklesSideLoadedVerificationKeyRStableV1BinableV1PolyV1StepData<(
        PicklesSideLoadedVerificationKeyRStableV1BinableV1PolyV1StepDataArg0<
            PicklesBaseDomainStableV1Binable,
        >,
        PicklesBaseSideLoadedVerificationKeyWidthStableV1Binable,
    )>,
    pub max_width: PicklesBaseSideLoadedVerificationKeyWidthStableV1Binable,
    pub wrap_index: PicklesSideLoadedVerificationKeyRStableV1BinableV1PolyV1WrapIndex<Vec<G>>,
}

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:150:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L150)
pub type PicklesSideLoadedVerificationKeyRStableV1BinableV1Poly<G> =
    crate::versioned::Versioned<PicklesSideLoadedVerificationKeyRStableV1BinableV1PolyV1<G>, 1i32>;

/// Location: [src/lib/pickles/side_loaded_verification_key.ml:156:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/side_loaded_verification_key.ml#L156)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesSideLoadedVerificationKeyRStableV1BinableV1(
    pub  PicklesSideLoadedVerificationKeyRStableV1BinableV1Poly<
        PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyArg0,
    >,
);

/// **Origin**: `Pickles__Side_loaded_verification_key.R.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/side_loaded_verification_key.ml:156:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/side_loaded_verification_key.ml#L156)
pub type PicklesSideLoadedVerificationKeyRStableV1Binable =
    crate::versioned::Versioned<PicklesSideLoadedVerificationKeyRStableV1BinableV1, 1i32>;

/// Location: [src/lib/pickles/side_loaded_verification_key.ml:166:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/side_loaded_verification_key.ml#L166)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesSideLoadedVerificationKeyStableV1BinableV1(
    pub PicklesSideLoadedVerificationKeyRStableV1Binable,
);

/// **Origin**: `Pickles__Side_loaded_verification_key.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/side_loaded_verification_key.ml:166:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/side_loaded_verification_key.ml#L166)
pub type PicklesSideLoadedVerificationKeyStableV1Binable =
    crate::versioned::Versioned<PicklesSideLoadedVerificationKeyStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/permissions.ml:287:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/permissions.ml#L287)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePermissionsStableV1BinableV1PolyV1<Bool, Controller> {
    pub stake: Bool,
    pub edit_state: Controller,
    pub send: Controller,
    pub receive: Controller,
    pub set_delegate: Controller,
    pub set_permissions: Controller,
    pub set_verification_key: Controller,
}

/// Location: [src/lib/mina_base/permissions.ml:287:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/permissions.ml#L287)
pub type MinaBasePermissionsStableV1BinableV1Poly<Bool, Controller> =
    crate::versioned::Versioned<MinaBasePermissionsStableV1BinableV1PolyV1<Bool, Controller>, 1i32>;

/// Location: [src/lib/mina_base/permissions.ml:52:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/permissions.ml#L52)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBasePermissionsAuthRequiredStableV1BinableV1 {
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
pub type MinaBasePermissionsAuthRequiredStableV1Binable =
    crate::versioned::Versioned<MinaBasePermissionsAuthRequiredStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/permissions.ml:313:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/permissions.ml#L313)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePermissionsStableV1BinableV1(
    pub  MinaBasePermissionsStableV1BinableV1Poly<
        bool,
        MinaBasePermissionsAuthRequiredStableV1Binable,
    >,
);

/// **Origin**: `Mina_base__Permissions.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/permissions.ml:313:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/permissions.ml#L313)
pub type MinaBasePermissionsStableV1Binable =
    crate::versioned::Versioned<MinaBasePermissionsStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:51:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L51)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyUpdateStableV1BinableV1(
    pub  MinaBaseSnappCommandPartyUpdateStableV1BinableV1Poly<
        MinaBaseSnappCommandPartyUpdateStableV1BinableV1PolyArg0<crate::bigint::BigInt>,
        MinaBaseSnappCommandPartyUpdateStableV1BinableV1PolyArg0<
            ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8,
        >,
        MinaBaseSnappCommandPartyUpdateStableV1BinableV1PolyArg0<
            MinaBaseSnappCommandPartyUpdateStableV1BinableV1PolyArg0Arg0<
                PicklesSideLoadedVerificationKeyStableV1Binable,
                crate::bigint::BigInt,
            >,
        >,
        MinaBaseSnappCommandPartyUpdateStableV1BinableV1PolyArg0<
            MinaBasePermissionsStableV1Binable,
        >,
    >,
);

/// **Origin**: `Mina_base__Snapp_command.Party.Update.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:51:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L51)
pub type MinaBaseSnappCommandPartyUpdateStableV1Binable =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyUpdateStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:146:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L146)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyBodyStableV1BinableV1(
    pub  MinaBaseSnappCommandPartyBodyStableV1BinableV1Poly<
        ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8,
        MinaBaseSnappCommandPartyUpdateStableV1Binable,
        MinaBaseFeeExcessStableV1BinableV1PolyArg1<
            CurrencyAmountMakeStrStableV1Binable,
            SgnStableV1Binable,
        >,
    >,
);

/// **Origin**: `Mina_base__Snapp_command.Party.Body.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:146:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L146)
pub type MinaBaseSnappCommandPartyBodyStableV1Binable =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyBodyStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_predicate.ml:1167:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L1167)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateStableV1BinableV1PolyV1<Account, ProtocolState, Other, Pk> {
    pub self_predicate: Account,
    pub other: Other,
    pub fee_payer: Pk,
    pub protocol_state_predicate: ProtocolState,
}

/// Location: [src/lib/mina_base/snapp_predicate.ml:1167:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L1167)
pub type MinaBaseSnappPredicateStableV1BinableV1Poly<Account, ProtocolState, Other, Pk> =
    crate::versioned::Versioned<
        MinaBaseSnappPredicateStableV1BinableV1PolyV1<Account, ProtocolState, Other, Pk>,
        1i32,
    >;

/// Location: [src/lib/mina_base/snapp_predicate.ml:353:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L353)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateAccountStableV1BinableV1PolyV1<
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
    pub state: MinaBaseSnappCommandPartyUpdateStableV1BinableV1PolyV1AppState<Field>,
}

/// Location: [src/lib/mina_base/snapp_predicate.ml:353:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L353)
pub type MinaBaseSnappPredicateAccountStableV1BinableV1Poly<
    Balance,
    Nonce,
    ReceiptChainHash,
    Pk,
    Field,
> = crate::versioned::Versioned<
    MinaBaseSnappPredicateAccountStableV1BinableV1PolyV1<
        Balance,
        Nonce,
        ReceiptChainHash,
        Pk,
        Field,
    >,
    1i32,
>;

/// Location: [src/lib/mina_base/snapp_basic.ml:158:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_basic.ml#L158)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1PolyV1<A> {
    Check(A),
    Ignore,
}

/// Location: [src/lib/mina_base/snapp_basic.ml:158:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_basic.ml#L158)
pub type MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1Poly<A> =
    crate::versioned::Versioned<
        MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1PolyV1<A>,
        1i32,
    >;

/// Location: [src/lib/mina_base/snapp_predicate.ml:23:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L23)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1PolyArg0V1<A> {
    pub lower: A,
    pub upper: A,
}

/// Location: [src/lib/mina_base/snapp_predicate.ml:23:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L23)
pub type MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1PolyArg0<A> =
    crate::versioned::Versioned<
        MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1PolyArg0V1<A>,
        1i32,
    >;

/// Location: [src/lib/mina_base/snapp_predicate.ml:150:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L150)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1<A>(
    pub  MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1Poly<
        MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1PolyArg0<A>,
    >,
);

/// Location: [src/lib/mina_base/snapp_predicate.ml:150:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L150)
pub type MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0<A> =
    crate::versioned::Versioned<MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1<A>, 1i32>;

/// Location: [src/lib/mina_base/receipt.ml:30:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/receipt.ml#L30)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1PolyArg0Dup1V1Poly(
    pub crate::bigint::BigInt,
);

/// Location: [src/lib/mina_base/receipt.ml:29:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/receipt.ml#L29)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1PolyArg0Dup1V1(
    pub MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1PolyArg0Dup1V1Poly,
);

/// Location: [src/lib/mina_base/receipt.ml:29:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/receipt.ml#L29)
pub type MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1PolyArg0Dup1 =
    crate::versioned::Versioned<
        MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1PolyArg0Dup1V1,
        1i32,
    >;

/// Location: [src/lib/mina_base/snapp_predicate.ml:369:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L369)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateAccountStableV1BinableV1(
    pub  MinaBaseSnappPredicateAccountStableV1BinableV1Poly<
        MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0<CurrencyBalanceStableV1Binable>,
        MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0<MinaNumbersNatMake32StableV1Binable>,
        MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1Poly<
            MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1PolyArg0Dup1,
        >,
        MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1Poly<
            ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8,
        >,
        MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1Poly<crate::bigint::BigInt>,
    >,
);

/// **Origin**: `Mina_base__Snapp_predicate.Account.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_predicate.ml:369:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L369)
pub type MinaBaseSnappPredicateAccountStableV1Binable =
    crate::versioned::Versioned<MinaBaseSnappPredicateAccountStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_predicate.ml:603:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L603)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateProtocolStateStableV1BinableV1PolyV1<
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
pub type MinaBaseSnappPredicateProtocolStateStableV1BinableV1Poly<
    SnarkedLedgerHash,
    TokenId,
    Time,
    Length,
    VrfOutput,
    GlobalSlot,
    Amount,
    EpochData,
> = crate::versioned::Versioned<
    MinaBaseSnappPredicateProtocolStateStableV1BinableV1PolyV1<
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
pub struct MinaBaseSnappPredicateProtocolStateEpochDataStableV1BinableV1(
    pub  ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1BinableV1Poly<
        MinaBaseEpochLedgerValueStableV1BinableV1Poly<
            MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1Poly<
                MinaBaseStagedLedgerHashNonSnarkStableV1BinableV1LedgerHash,
            >,
            MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0<
                CurrencyAmountMakeStrStableV1Binable,
            >,
        >,
        MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1Poly<
            ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1BinableV1PolyArg1,
        >,
        MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1Poly<
            MinaStateProtocolStateValueStableV1BinableV1PolyArg0,
        >,
        MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1Poly<
            MinaStateProtocolStateValueStableV1BinableV1PolyArg0,
        >,
        MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0<
            ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg0,
        >,
    >,
);

/// **Origin**: `Mina_base__Snapp_predicate.Protocol_state.Epoch_data.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_predicate.ml:535:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L535)
pub type MinaBaseSnappPredicateProtocolStateEpochDataStableV1Binable = crate::versioned::Versioned<
    MinaBaseSnappPredicateProtocolStateEpochDataStableV1BinableV1,
    1i32,
>;

/// Location: [src/lib/mina_base/snapp_predicate.ml:644:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L644)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateProtocolStateStableV1BinableV1(
    pub  MinaBaseSnappPredicateProtocolStateStableV1BinableV1Poly<
        MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1Poly<
            MinaBaseStagedLedgerHashNonSnarkStableV1BinableV1LedgerHash,
        >,
        MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0<MinaBaseTokenIdStableV1Binable>,
        MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0<BlockTimeTimeStableV1Binable>,
        MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0<
            ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg0,
        >,
        (),
        MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0<
            ConsensusGlobalSlotStableV1BinableV1PolyArg0,
        >,
        MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0<
            CurrencyAmountMakeStrStableV1Binable,
        >,
        MinaBaseSnappPredicateProtocolStateEpochDataStableV1Binable,
    >,
);

/// **Origin**: `Mina_base__Snapp_predicate.Protocol_state.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_predicate.ml:644:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L644)
pub type MinaBaseSnappPredicateProtocolStateStableV1Binable =
    crate::versioned::Versioned<MinaBaseSnappPredicateProtocolStateStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_predicate.ml:1100:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L1100)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateOtherStableV1BinableV1PolyV1<Account, AccountTransition, Vk> {
    pub predicate: Account,
    pub account_transition: AccountTransition,
    pub account_vk: Vk,
}

/// Location: [src/lib/mina_base/snapp_predicate.ml:1100:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L1100)
pub type MinaBaseSnappPredicateOtherStableV1BinableV1Poly<Account, AccountTransition, Vk> =
    crate::versioned::Versioned<
        MinaBaseSnappPredicateOtherStableV1BinableV1PolyV1<Account, AccountTransition, Vk>,
        1i32,
    >;

/// Location: [src/lib/mina_base/snapp_basic.ml:21:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_basic.ml#L21)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateOtherStableV1BinableV1PolyArg1V1<A> {
    pub prev: A,
    pub next: A,
}

/// Location: [src/lib/mina_base/snapp_basic.ml:21:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_basic.ml#L21)
pub type MinaBaseSnappPredicateOtherStableV1BinableV1PolyArg1<A> =
    crate::versioned::Versioned<MinaBaseSnappPredicateOtherStableV1BinableV1PolyArg1V1<A>, 1i32>;

/// Location: [src/lib/mina_base/snapp_basic.ml:234:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_basic.ml#L234)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSnappBasicAccountStateStableV1BinableV1 {
    Empty,
    NonEmpty,
    Any,
}

/// **Origin**: `Mina_base__Snapp_basic.Account_state.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_basic.ml:234:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_basic.ml#L234)
pub type MinaBaseSnappBasicAccountStateStableV1Binable =
    crate::versioned::Versioned<MinaBaseSnappBasicAccountStateStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_predicate.ml:1113:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L1113)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateOtherStableV1BinableV1(
    pub  MinaBaseSnappPredicateOtherStableV1BinableV1Poly<
        MinaBaseSnappPredicateAccountStableV1Binable,
        MinaBaseSnappPredicateOtherStableV1BinableV1PolyArg1<
            MinaBaseSnappBasicAccountStateStableV1Binable,
        >,
        MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1Poly<crate::bigint::BigInt>,
    >,
);

/// **Origin**: `Mina_base__Snapp_predicate.Other.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_predicate.ml:1113:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L1113)
pub type MinaBaseSnappPredicateOtherStableV1Binable =
    crate::versioned::Versioned<MinaBaseSnappPredicateOtherStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_predicate.ml:1188:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L1188)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappPredicateStableV1BinableV1(
    pub  MinaBaseSnappPredicateStableV1BinableV1Poly<
        MinaBaseSnappPredicateAccountStableV1Binable,
        MinaBaseSnappPredicateProtocolStateStableV1Binable,
        MinaBaseSnappPredicateOtherStableV1Binable,
        MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1Poly<
            ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8,
        >,
    >,
);

/// **Origin**: `Mina_base__Snapp_predicate.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_predicate.ml:1188:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_predicate.ml#L1188)
pub type MinaBaseSnappPredicateStableV1Binable =
    crate::versioned::Versioned<MinaBaseSnappPredicateStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:225:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L225)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyPredicatedProvedStableV1BinableV1(
    pub  MinaBaseSnappCommandPartyPredicatedProvedStableV1BinableV1Poly<
        MinaBaseSnappCommandPartyBodyStableV1Binable,
        MinaBaseSnappPredicateStableV1Binable,
    >,
);

/// **Origin**: `Mina_base__Snapp_command.Party.Predicated.Proved.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:225:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L225)
pub type MinaBaseSnappCommandPartyPredicatedProvedStableV1Binable =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyPredicatedProvedStableV1BinableV1, 1i32>;

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:66:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L66)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyArg0Arg1V1<A>(
    pub PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg0<A>,
);

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:66:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L66)
pub type PicklesProofBranching2ReprStableV1BinableV1PolyArg0Arg1<A> =
    crate::versioned::Versioned<PicklesProofBranching2ReprStableV1BinableV1PolyArg0Arg1V1<A>, 1i32>;

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:87:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L87)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranching2ReprStableV1BinableV1PolyArg1Arg1Dup1V1<A>(
    pub PicklesProofBranching2ReprStableV1BinableV1PolyArg1Arg1<A>,
);

/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:87:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L87)
pub type PicklesProofBranching2ReprStableV1BinableV1PolyArg1Arg1Dup1<A> =
    crate::versioned::Versioned<
        PicklesProofBranching2ReprStableV1BinableV1PolyArg1Arg1Dup1V1<A>,
        1i32,
    >;

/// Location: [src/lib/pickles/proof.ml:352:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L352)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranchingMaxReprStableV1BinableV1(
    pub  PicklesProofBranching2ReprStableV1BinableV1Poly<
        PicklesProofBranching2ReprStableV1BinableV1PolyArg0<
            PicklesProofBranching2ReprStableV1BinableV1PolyArg0Arg0,
            PicklesProofBranching2ReprStableV1BinableV1PolyArg0Arg1<
                PicklesReducedMeOnlyDlogBasedChallengesVectorStableV1Binable,
            >,
        >,
        PicklesProofBranching2ReprStableV1BinableV1PolyArg1<
            (),
            PicklesProofBranching2ReprStableV1BinableV1PolyArg1Arg1Dup1<
                PicklesProofBranching2ReprStableV1BinableV1PolyV1ProofV1PolyArg0,
            >,
            PicklesProofBranching2ReprStableV1BinableV1PolyArg1Arg1Dup1<
                PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg7<
                    PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg7Arg0<
                        PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg1<
                            PicklesProofBranching2ReprStableV1BinableV1PolyV1StatementArg0<
                                LimbVectorConstantHex64StableV1Binable,
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
pub type PicklesProofBranchingMaxReprStableV1Binable =
    crate::versioned::Versioned<PicklesProofBranchingMaxReprStableV1BinableV1, 1i32>;

/// Location: [src/lib/pickles/proof.ml:388:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L388)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofBranchingMaxStableV1BinableV1(
    pub PicklesProofBranchingMaxReprStableV1Binable,
);

/// **Origin**: `Pickles__Proof.Branching_max.Stable.V1.t`
///
/// **Location**: [src/lib/pickles/proof.ml:388:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L388)
pub type PicklesProofBranchingMaxStableV1Binable =
    crate::versioned::Versioned<PicklesProofBranchingMaxStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/control.ml:11:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/control.ml#L11)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseControlStableV1BinableV1 {
    Proof(PicklesProofBranchingMaxStableV1Binable),
    Signature(MinaBaseSignatureStableV1Binable),
    Both {
        signature: MinaBaseSignatureStableV1Binable,
        proof: PicklesProofBranchingMaxStableV1Binable,
    },
    NoneGiven,
}

/// **Origin**: `Mina_base__Control.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/control.ml:11:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/control.ml#L11)
pub type MinaBaseControlStableV1Binable =
    crate::versioned::Versioned<MinaBaseControlStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:308:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L308)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyAuthorizedProvedStableV1BinableV1(
    pub  MinaBaseSnappCommandPartyAuthorizedProvedStableV1BinableV1Poly<
        MinaBaseSnappCommandPartyPredicatedProvedStableV1Binable,
        MinaBaseControlStableV1Binable,
    >,
);

/// **Origin**: `Mina_base__Snapp_command.Party.Authorized.Proved.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:308:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L308)
pub type MinaBaseSnappCommandPartyAuthorizedProvedStableV1Binable =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyAuthorizedProvedStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:280:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L280)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyPredicatedEmptyStableV1BinableV1(
    pub  MinaBaseSnappCommandPartyPredicatedProvedStableV1BinableV1Poly<
        MinaBaseSnappCommandPartyBodyStableV1Binable,
        (),
    >,
);

/// **Origin**: `Mina_base__Snapp_command.Party.Predicated.Empty.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:280:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L280)
pub type MinaBaseSnappCommandPartyPredicatedEmptyStableV1Binable =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyPredicatedEmptyStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:338:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L338)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyAuthorizedEmptyStableV1BinableV1(
    pub  MinaBaseSnappCommandPartyAuthorizedProvedStableV1BinableV1Poly<
        MinaBaseSnappCommandPartyPredicatedEmptyStableV1Binable,
        (),
    >,
);

/// **Origin**: `Mina_base__Snapp_command.Party.Authorized.Empty.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:338:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L338)
pub type MinaBaseSnappCommandPartyAuthorizedEmptyStableV1Binable =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyAuthorizedEmptyStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:246:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L246)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyPredicatedSignedStableV1BinableV1(
    pub  MinaBaseSnappCommandPartyPredicatedProvedStableV1BinableV1Poly<
        MinaBaseSnappCommandPartyBodyStableV1Binable,
        MinaNumbersNatMake32StableV1Binable,
    >,
);

/// **Origin**: `Mina_base__Snapp_command.Party.Predicated.Signed.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:246:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L246)
pub type MinaBaseSnappCommandPartyPredicatedSignedStableV1Binable =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyPredicatedSignedStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:323:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L323)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandPartyAuthorizedSignedStableV1BinableV1(
    pub  MinaBaseSnappCommandPartyAuthorizedProvedStableV1BinableV1Poly<
        MinaBaseSnappCommandPartyPredicatedSignedStableV1Binable,
        MinaBaseSignatureStableV1Binable,
    >,
);

/// **Origin**: `Mina_base__Snapp_command.Party.Authorized.Signed.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:323:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L323)
pub type MinaBaseSnappCommandPartyAuthorizedSignedStableV1Binable =
    crate::versioned::Versioned<MinaBaseSnappCommandPartyAuthorizedSignedStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:367:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L367)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSnappCommandBinableArgStableV1BinableV1 {
    ProvedEmpty(
        MinaBaseSnappCommandBinableArgStableV1BinableV1ProvedEmpty0<
            MinaBaseSnappCommandPartyAuthorizedProvedStableV1Binable,
            Option<MinaBaseSnappCommandPartyAuthorizedEmptyStableV1Binable>,
        >,
    ),
    ProvedSigned(
        MinaBaseSnappCommandBinableArgStableV1BinableV1ProvedEmpty0<
            MinaBaseSnappCommandPartyAuthorizedProvedStableV1Binable,
            MinaBaseSnappCommandPartyAuthorizedSignedStableV1Binable,
        >,
    ),
    ProvedProved(
        MinaBaseSnappCommandBinableArgStableV1BinableV1ProvedEmpty0<
            MinaBaseSnappCommandPartyAuthorizedProvedStableV1Binable,
            MinaBaseSnappCommandPartyAuthorizedProvedStableV1Binable,
        >,
    ),
    SignedSigned(
        MinaBaseSnappCommandBinableArgStableV1BinableV1ProvedEmpty0<
            MinaBaseSnappCommandPartyAuthorizedSignedStableV1Binable,
            MinaBaseSnappCommandPartyAuthorizedSignedStableV1Binable,
        >,
    ),
    SignedEmpty(
        MinaBaseSnappCommandBinableArgStableV1BinableV1ProvedEmpty0<
            MinaBaseSnappCommandPartyAuthorizedSignedStableV1Binable,
            Option<MinaBaseSnappCommandPartyAuthorizedEmptyStableV1Binable>,
        >,
    ),
}

/// **Origin**: `Mina_base__Snapp_command.Binable_arg.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:367:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L367)
pub type MinaBaseSnappCommandBinableArgStableV1Binable =
    crate::versioned::Versioned<MinaBaseSnappCommandBinableArgStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_command.ml:408:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L408)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappCommandStableV1BinableV1(pub MinaBaseSnappCommandBinableArgStableV1Binable);

/// **Origin**: `Mina_base__Snapp_command.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_command.ml:408:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_command.ml#L408)
pub type MinaBaseSnappCommandStableV1Binable =
    crate::versioned::Versioned<MinaBaseSnappCommandStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/user_command.ml:74:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/user_command.ml#L74)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseUserCommandStableV1BinableV1(
    pub  MinaBaseUserCommandStableV1BinableV1Poly<
        MinaBaseSignedCommandStableV1Binable,
        MinaBaseSnappCommandStableV1Binable,
    >,
);

/// **Origin**: `Mina_base__User_command.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/user_command.ml:74:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/user_command.ml#L74)
pub type MinaBaseUserCommandStableV1Binable =
    crate::versioned::Versioned<MinaBaseUserCommandStableV1BinableV1, 1i32>;

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:136:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L136)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1BinableV1(
    pub  StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1BinableV1Poly<
        TransactionSnarkWorkTStableV1Binable,
        StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1BinableV1PolyArg1<
            MinaBaseUserCommandStableV1Binable,
        >,
    >,
);

/// **Origin**: `Staged_ledger_diff.Pre_diff_with_at_most_two_coinbase.Stable.V1.t`
///
/// **Location**: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:136:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L136)
pub type StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1Binable = crate::versioned::Versioned<
    StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1BinableV1,
    1i32,
>;

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:43:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L43)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1BinableV1PolyV1CoinbaseV1<A> {
    Zero,
    One(Option<A>),
}

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:43:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L43)
pub type StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1BinableV1PolyV1Coinbase<A> =
    crate::versioned::Versioned<
        StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1BinableV1PolyV1CoinbaseV1<A>,
        1i32,
    >;

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:109:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L109)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1BinableV1PolyV1<A, B> {
    pub completed_works: Vec<A>,
    pub commands: Vec<B>,
    pub coinbase: StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1BinableV1PolyV1Coinbase<
        StagedLedgerDiffFtStableV1Binable,
    >,
    pub internal_command_balances:
        Vec<MinaBaseTransactionStatusInternalCommandBalanceDataStableV1Binable>,
}

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:109:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L109)
pub type StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1BinableV1Poly<A, B> =
    crate::versioned::Versioned<
        StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1BinableV1PolyV1<A, B>,
        1i32,
    >;

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:155:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L155)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1BinableV1(
    pub  StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1BinableV1Poly<
        TransactionSnarkWorkTStableV1Binable,
        StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1BinableV1PolyArg1<
            MinaBaseUserCommandStableV1Binable,
        >,
    >,
);

/// **Origin**: `Staged_ledger_diff.Pre_diff_with_at_most_one_coinbase.Stable.V1.t`
///
/// **Location**: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:155:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L155)
pub type StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1Binable = crate::versioned::Versioned<
    StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1BinableV1,
    1i32,
>;

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:174:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L174)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffStableV1BinableV1(
    pub StagedLedgerDiffPreDiffWithAtMostTwoCoinbaseStableV1Binable,
    pub Option<StagedLedgerDiffPreDiffWithAtMostOneCoinbaseStableV1Binable>,
);

/// **Origin**: `Staged_ledger_diff.Diff.Stable.V1.t`
///
/// **Location**: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:174:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L174)
pub type StagedLedgerDiffDiffStableV1Binable =
    crate::versioned::Versioned<StagedLedgerDiffDiffStableV1BinableV1, 1i32>;

/// Location: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:191:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L191)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffStableV1BinableV1 {
    pub diff: StagedLedgerDiffDiffStableV1Binable,
}

/// **Origin**: `Staged_ledger_diff.Stable.V1.t`
///
/// **Location**: [src/lib/staged_ledger_diff/staged_ledger_diff.ml:191:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/staged_ledger_diff.ml#L191)
pub type StagedLedgerDiffStableV1Binable =
    crate::versioned::Versioned<StagedLedgerDiffStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/state_body_hash.ml:20:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/state_body_hash.ml#L20)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBlockExternalTransitionRawVersionedStableV1BinableV1DeltaTransitionChainProofArg0V1Poly(
    pub crate::bigint::BigInt,
);

/// Location: [src/lib/mina_base/state_body_hash.ml:19:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/state_body_hash.ml#L19)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBlockExternalTransitionRawVersionedStableV1BinableV1DeltaTransitionChainProofArg0V1(
    pub MinaBlockExternalTransitionRawVersionedStableV1BinableV1DeltaTransitionChainProofArg0V1Poly,
);

/// Location: [src/lib/mina_base/state_body_hash.ml:19:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/state_body_hash.ml#L19)
pub type MinaBlockExternalTransitionRawVersionedStableV1BinableV1DeltaTransitionChainProofArg0 =
    crate::versioned::Versioned<
        MinaBlockExternalTransitionRawVersionedStableV1BinableV1DeltaTransitionChainProofArg0V1,
        1i32,
    >;

/// Location: [src/lib/protocol_version/protocol_version.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/protocol_version/protocol_version.ml#L8)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ProtocolVersionStableV1BinableV1 {
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
}

/// **Origin**: `Protocol_version.Stable.V1.t`
///
/// **Location**: [src/lib/protocol_version/protocol_version.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/protocol_version/protocol_version.ml#L8)
pub type ProtocolVersionStableV1Binable =
    crate::versioned::Versioned<ProtocolVersionStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_block/external_transition.ml:31:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_block/external_transition.ml#L31)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBlockExternalTransitionRawVersionedStableV1BinableV1 {
    pub protocol_state: MinaStateProtocolStateValueStableV1Binable,
    pub protocol_state_proof: MinaBaseProofStableV1Binable,
    pub staged_ledger_diff: StagedLedgerDiffStableV1Binable,
    pub delta_transition_chain_proof: (
        MinaStateProtocolStateValueStableV1BinableV1PolyArg0,
        Vec<MinaBlockExternalTransitionRawVersionedStableV1BinableV1DeltaTransitionChainProofArg0>,
    ),
    pub current_protocol_version: ProtocolVersionStableV1Binable,
    pub proposed_protocol_version_opt: Option<ProtocolVersionStableV1Binable>,
    pub validation_callback: (),
}

/// Location: [src/lib/network_pool/transaction_pool.ml:45:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_pool/transaction_pool.ml#L45)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NetworkPoolTransactionPoolDiffVersionedStableV1BinableV1(
    pub Vec<MinaBaseUserCommandStableV1Binable>,
);

/// Location: [src/lib/transaction_snark/transaction_snark.ml:202:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L202)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkStatementStableV1BinableV1(
    pub  TransactionSnarkStatementWithSokStableV1BinableV1Poly<
        MinaBaseStagedLedgerHashNonSnarkStableV1BinableV1LedgerHash,
        CurrencyAmountMakeStrStableV1Binable,
        TransactionSnarkPendingCoinbaseStackStateStableV1Binable,
        MinaBaseFeeExcessStableV1Binable,
        MinaBaseTokenIdStableV1Binable,
        (),
    >,
);

/// **Origin**: `Transaction_snark.Statement.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark/transaction_snark.ml:202:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L202)
pub type TransactionSnarkStatementStableV1Binable =
    crate::versioned::Versioned<TransactionSnarkStatementStableV1BinableV1, 1i32>;

/// Location: [src/lib/transaction_snark_work/transaction_snark_work.ml:23:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_work/transaction_snark_work.ml#L23)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkWorkStatementStableV1BinableV1(
    pub TransactionSnarkWorkTStableV1BinableV1Proofs<TransactionSnarkStatementStableV1Binable>,
);

/// **Origin**: `Transaction_snark_work.Statement.Stable.V1.t`
///
/// **Location**: [src/lib/transaction_snark_work/transaction_snark_work.ml:23:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_work/transaction_snark_work.ml#L23)
pub type TransactionSnarkWorkStatementStableV1Binable =
    crate::versioned::Versioned<TransactionSnarkWorkStatementStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/fee_with_prover.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_with_prover.ml#L7)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeWithProverStableV1BinableV1 {
    pub fee: CurrencyFeeStableV1Binable,
    pub prover: ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8,
}

/// **Origin**: `Mina_base__Fee_with_prover.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/fee_with_prover.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_with_prover.ml#L7)
pub type MinaBaseFeeWithProverStableV1Binable =
    crate::versioned::Versioned<MinaBaseFeeWithProverStableV1BinableV1, 1i32>;

/// Location: [src/lib/network_pool/priced_proof.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_pool/priced_proof.ml#L9)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NetworkPoolSnarkPoolDiffVersionedStableV1BinableV1AddSolvedWork1V1<Proof> {
    pub proof: Proof,
    pub fee: MinaBaseFeeWithProverStableV1Binable,
}

/// Location: [src/lib/network_pool/priced_proof.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_pool/priced_proof.ml#L9)
pub type NetworkPoolSnarkPoolDiffVersionedStableV1BinableV1AddSolvedWork1<Proof> =
    crate::versioned::Versioned<
        NetworkPoolSnarkPoolDiffVersionedStableV1BinableV1AddSolvedWork1V1<Proof>,
        1i32,
    >;

/// Location: [src/lib/network_pool/snark_pool.ml:705:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_pool/snark_pool.ml#L705)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum NetworkPoolSnarkPoolDiffVersionedStableV1BinableV1 {
    AddSolvedWork(
        TransactionSnarkWorkStatementStableV1Binable,
        NetworkPoolSnarkPoolDiffVersionedStableV1BinableV1AddSolvedWork1<
            TransactionSnarkWorkTStableV1BinableV1Proofs<LedgerProofProdStableV1Binable>,
        >,
    ),
    Empty,
}

/// Location: [src/lib/mina_base/account.ml:89:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account.ml#L89)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountBinableArgStableV1BinableV1PolyV1<
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
pub type MinaBaseAccountBinableArgStableV1BinableV1Poly<
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
    MinaBaseAccountBinableArgStableV1BinableV1PolyV1<
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
pub enum MinaBaseTokenPermissionsStableV1BinableV1 {
    TokenOwned { disable_new_accounts: bool },
    NotOwned { account_disabled: bool },
}

/// **Origin**: `Mina_base__Token_permissions.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/token_permissions.ml:14:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/token_permissions.ml#L14)
pub type MinaBaseTokenPermissionsStableV1Binable =
    crate::versioned::Versioned<MinaBaseTokenPermissionsStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/account_timing.ml:19:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account_timing.ml#L19)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountTimingStableV1BinableV1PolyV1<Slot, Balance, Amount> {
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
pub type MinaBaseAccountTimingStableV1BinableV1Poly<Slot, Balance, Amount> =
    crate::versioned::Versioned<
        MinaBaseAccountTimingStableV1BinableV1PolyV1<Slot, Balance, Amount>,
        1i32,
    >;

/// Location: [src/lib/mina_base/account_timing.ml:36:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account_timing.ml#L36)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountTimingStableV1BinableV1(
    pub  MinaBaseAccountTimingStableV1BinableV1Poly<
        ConsensusGlobalSlotStableV1BinableV1PolyArg0,
        CurrencyBalanceStableV1Binable,
        CurrencyAmountMakeStrStableV1Binable,
    >,
);

/// **Origin**: `Mina_base__Account_timing.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/account_timing.ml:36:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account_timing.ml#L36)
pub type MinaBaseAccountTimingStableV1Binable =
    crate::versioned::Versioned<MinaBaseAccountTimingStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_account.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_account.ml#L17)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappAccountStableV1BinableV1PolyV1<AppState, Vk> {
    pub app_state: AppState,
    pub verification_key: Vk,
}

/// Location: [src/lib/mina_base/snapp_account.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_account.ml#L17)
pub type MinaBaseSnappAccountStableV1BinableV1Poly<AppState, Vk> =
    crate::versioned::Versioned<MinaBaseSnappAccountStableV1BinableV1PolyV1<AppState, Vk>, 1i32>;

/// Location: [src/lib/mina_base/snapp_state.ml:50:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_state.ml#L50)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappStateValueStableV1BinableV1(
    pub MinaBaseSnappCommandPartyUpdateStableV1BinableV1PolyV1AppState<crate::bigint::BigInt>,
);

/// **Origin**: `Mina_base__Snapp_state.Value.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_state.ml:50:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_state.ml#L50)
pub type MinaBaseSnappStateValueStableV1Binable =
    crate::versioned::Versioned<MinaBaseSnappStateValueStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/snapp_account.ml:30:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_account.ml#L30)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSnappAccountStableV1BinableV1(
    pub  MinaBaseSnappAccountStableV1BinableV1Poly<
        MinaBaseSnappStateValueStableV1Binable,
        Option<
            MinaBaseSnappCommandPartyUpdateStableV1BinableV1PolyArg0Arg0<
                PicklesSideLoadedVerificationKeyStableV1Binable,
                crate::bigint::BigInt,
            >,
        >,
    >,
);

/// **Origin**: `Mina_base__Snapp_account.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/snapp_account.ml:30:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/snapp_account.ml#L30)
pub type MinaBaseSnappAccountStableV1Binable =
    crate::versioned::Versioned<MinaBaseSnappAccountStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/account.ml:140:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account.ml#L140)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountBinableArgStableV1BinableV1(
    pub  MinaBaseAccountBinableArgStableV1BinableV1Poly<
        ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8,
        MinaBaseTokenIdStableV1Binable,
        MinaBaseTokenPermissionsStableV1Binable,
        CurrencyBalanceStableV1Binable,
        MinaNumbersNatMake32StableV1Binable,
        MinaBaseSnappPredicateAccountStableV1BinableV1PolyArg0V1PolyArg0Dup1,
        Option<ConsensusProofOfStakeDataConsensusStateValueStableV1BinableV1PolyArg8>,
        MinaStateProtocolStateValueStableV1BinableV1PolyArg0,
        MinaBaseAccountTimingStableV1Binable,
        MinaBasePermissionsStableV1Binable,
        Option<MinaBaseSnappAccountStableV1Binable>,
    >,
);

/// **Origin**: `Mina_base__Account.Binable_arg.Stable.V1.t`
///
/// **Location**: [src/lib/mina_base/account.ml:140:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account.ml#L140)
pub type MinaBaseAccountBinableArgStableV1Binable =
    crate::versioned::Versioned<MinaBaseAccountBinableArgStableV1BinableV1, 1i32>;

/// Location: [src/lib/mina_base/account.ml:188:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account.ml#L188)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountStableV1BinableV1(pub MinaBaseAccountBinableArgStableV1Binable);
