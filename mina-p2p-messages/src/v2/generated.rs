use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};

use crate::pseq::PaddedSeq;

use super::manual::*;

/// **OCaml name**: `Mina_block__Block.Stable.V2`
///
/// Gid: `974`
/// Location: [src/lib/mina_block/block.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_block/block.ml#L8)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBlockBlockStableV2 {
    pub header: MinaBlockHeaderStableV2,
    pub body: StagedLedgerDiffBodyStableV1,
}

/// **OCaml name**: `Network_pool__Transaction_pool.Diff_versioned.Stable.V2`
///
/// Gid: `1001`
/// Location: [src/lib/network_pool/transaction_pool.ml:47:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_pool/transaction_pool.ml#L47)
///
///
/// Gid: `167`
/// Location: [src/std_internal.ml:131:2](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/std_internal.ml#L131)
/// Args: MinaBaseUserCommandStableV2
///
///
/// Gid: `50`
/// Location: [src/list0.ml:6:0](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/list0.ml#L6)
/// Args: MinaBaseUserCommandStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NetworkPoolTransactionPoolDiffVersionedStableV2(pub Vec<MinaBaseUserCommandStableV2>);

/// **OCaml name**: `Network_pool__Snark_pool.Diff_versioned.Stable.V2`
///
/// Gid: `1007`
/// Location: [src/lib/network_pool/snark_pool.ml:732:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_pool/snark_pool.ml#L732)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum NetworkPoolSnarkPoolDiffVersionedStableV2 {
    AddSolvedWork(
        Box<(
            TransactionSnarkWorkStatementStableV2,
            NetworkPoolSnarkPoolDiffVersionedStableV2AddSolvedWork1,
        )>,
    ),
    Empty,
}

/// **OCaml name**: `Mina_base__Sparse_ledger_base.Stable.V2`
///
/// Gid: `774`
/// Location: [src/lib/mina_base/sparse_ledger_base.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/sparse_ledger_base.ml#L8)
///
///
/// Gid: `598`
/// Location: [src/lib/sparse_ledger_lib/sparse_ledger.ml:38:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/sparse_ledger_lib/sparse_ledger.ml#L38)
/// Args: MinaBaseLedgerHash0StableV1 , MinaBaseAccountIdMakeStrStableV2 , MinaBaseAccountBinableArgStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSparseLedgerBaseStableV2 {
    pub indexes: Vec<(MinaBaseAccountIdMakeStrStableV2, crate::number::Int32)>,
    pub depth: crate::number::Int32,
    pub tree: MinaBaseSparseLedgerBaseStableV2Tree,
}

/// **OCaml name**: `Mina_base__Account.Binable_arg.Stable.V2`
///
/// Gid: `670`
/// Location: [src/lib/mina_base/account.ml:313:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account.ml#L313)
///
///
/// Gid: `667`
/// Location: [src/lib/mina_base/account.ml:226:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account.ml#L226)
/// Args: NonZeroCurvePointUncompressedStableV1 , MinaBaseAccountIdMakeStrDigestStableV1 , MinaBaseTokenPermissionsStableV1 , MinaBaseAccountTokenSymbolStableV1 , CurrencyMakeStrBalanceStableV1 , UnsignedExtendedUInt32StableV1 , MinaBaseReceiptChainHashStableV1 , Option < NonZeroCurvePointUncompressedStableV1 > , DataHashLibStateHashStableV1 , MinaBaseAccountTimingStableV1 , MinaBasePermissionsStableV2 , Option < MinaBaseZkappAccountStableV2 > , crate :: string :: ByteString
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountBinableArgStableV2 {
    pub public_key: NonZeroCurvePointUncompressedStableV1,
    pub token_id: MinaBaseAccountIdMakeStrDigestStableV1,
    pub token_permissions: MinaBaseTokenPermissionsStableV1,
    pub token_symbol: MinaBaseAccountTokenSymbolStableV1,
    pub balance: CurrencyMakeStrBalanceStableV1,
    pub nonce: UnsignedExtendedUInt32StableV1,
    pub receipt_chain_hash: MinaBaseReceiptChainHashStableV1,
    pub delegate: Option<NonZeroCurvePointUncompressedStableV1>,
    pub voting_for: DataHashLibStateHashStableV1,
    pub timing: MinaBaseAccountTimingStableV1,
    pub permissions: MinaBasePermissionsStableV2,
    pub zkapp: Option<MinaBaseZkappAccountStableV2>,
    pub zkapp_uri: crate::string::ByteString,
}

/// **OCaml name**: `Network_peer__Peer.Stable.V1`
///
/// Gid: `810`
/// Location: [src/lib/network_peer/peer.ml:28:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_peer/peer.ml#L28)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NetworkPeerPeerStableV1 {
    pub host: crate::string::ByteString,
    pub libp2p_port: crate::number::Int32,
    pub peer_id: NetworkPeerPeerIdStableV1,
}

/// **OCaml name**: `Transaction_snark_scan_state.Stable.V2`
///
/// Gid: `951`
/// Location: [src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml:153:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml#L153)
///
///
/// Gid: `948`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:803:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L803)
/// Args: TransactionSnarkScanStateLedgerProofWithSokMessageStableV2 , TransactionSnarkScanStateTransactionWithWitnessStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateStableV2 {
    pub trees: (
        TransactionSnarkScanStateStableV2TreesA,
        Vec<TransactionSnarkScanStateStableV2TreesA>,
    ),
    pub acc: Option<(
        TransactionSnarkScanStateLedgerProofWithSokMessageStableV2,
        Vec<TransactionSnarkScanStateTransactionWithWitnessStableV2>,
    )>,
    pub curr_job_seq_no: crate::number::Int32,
    pub max_base_jobs: crate::number::Int32,
    pub delay: crate::number::Int32,
}

/// **OCaml name**: `Mina_base__Pending_coinbase.Stable.V2`
///
/// Gid: `766`
/// Location: [src/lib/mina_base/pending_coinbase.ml:1238:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L1238)
///
///
/// Gid: `765`
/// Location: [src/lib/mina_base/pending_coinbase.ml:1226:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L1226)
/// Args: MinaBasePendingCoinbaseMerkleTreeVersionedStableV2 , MinaBasePendingCoinbaseStackIdStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStableV2 {
    pub tree: MinaBasePendingCoinbaseMerkleTreeVersionedStableV2,
    pub pos_list: Vec<MinaBasePendingCoinbaseStackIdStableV1>,
    pub new_pos: MinaBasePendingCoinbaseStackIdStableV1,
}

/// **OCaml name**: `Mina_state__Protocol_state.Value.Stable.V2`
///
/// Gid: `871`
/// Location: [src/lib/mina_state/protocol_state.ml:177:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L177)
///
///
/// Gid: `867`
/// Location: [src/lib/mina_state/protocol_state.ml:16:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L16)
/// Args: DataHashLibStateHashStableV1 , MinaStateProtocolStateBodyValueStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateProtocolStateValueStableV2 {
    pub previous_state_hash: DataHashLibStateHashStableV1,
    pub body: MinaStateProtocolStateBodyValueStableV2,
}

/// **OCaml name**: `Mina_ledger__Sync_ledger.Query.Stable.V1`
///
/// Gid: `828`
/// Location: [src/lib/mina_ledger/sync_ledger.ml:71:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_ledger/sync_ledger.ml#L71)
///
///
/// Gid: `817`
/// Location: [src/lib/syncable_ledger/syncable_ledger.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/syncable_ledger/syncable_ledger.ml#L17)
/// Args: MerkleAddressBinableArgStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaLedgerSyncLedgerQueryStableV1 {
    WhatChildHashes(MerkleAddressBinableArgStableV1),
    WhatContents(MerkleAddressBinableArgStableV1),
    NumAccounts,
}

/// **OCaml name**: `Mina_ledger__Sync_ledger.Answer.Stable.V2`
///
/// Gid: `827`
/// Location: [src/lib/mina_ledger/sync_ledger.ml:56:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_ledger/sync_ledger.ml#L56)
///
///
/// Gid: `818`
/// Location: [src/lib/syncable_ledger/syncable_ledger.ml:35:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/syncable_ledger/syncable_ledger.ml#L35)
/// Args: MinaBaseLedgerHash0StableV1 , MinaBaseAccountBinableArgStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaLedgerSyncLedgerAnswerStableV2 {
    ChildHashesAre(MinaBaseLedgerHash0StableV1, MinaBaseLedgerHash0StableV1),
    ContentsAre(Vec<MinaBaseAccountBinableArgStableV2>),
    NumAccounts(crate::number::Int32, MinaBaseLedgerHash0StableV1),
}

/// **OCaml name**: `Consensus__Proof_of_stake.Data.Consensus_state.Value.Stable.V1`
///
/// Gid: `859`
/// Location: [src/lib/consensus/proof_of_stake.ml:1722:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1722)
///
///
/// Gid: `858`
/// Location: [src/lib/consensus/proof_of_stake.ml:1687:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1687)
/// Args: UnsignedExtendedUInt32StableV1 , ConsensusVrfOutputTruncatedStableV1 , CurrencyMakeStrAmountMakeStrStableV1 , ConsensusGlobalSlotStableV1 , UnsignedExtendedUInt32StableV1 , ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1 , ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1 , bool , NonZeroCurvePointUncompressedStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataConsensusStateValueStableV1 {
    pub blockchain_length: UnsignedExtendedUInt32StableV1,
    pub epoch_count: UnsignedExtendedUInt32StableV1,
    pub min_window_density: UnsignedExtendedUInt32StableV1,
    pub sub_window_densities: Vec<UnsignedExtendedUInt32StableV1>,
    pub last_vrf_output: ConsensusVrfOutputTruncatedStableV1,
    pub total_currency: CurrencyMakeStrAmountMakeStrStableV1,
    pub curr_global_slot: ConsensusGlobalSlotStableV1,
    pub global_slot_since_genesis: UnsignedExtendedUInt32StableV1,
    pub staking_epoch_data: ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
    pub next_epoch_data: ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
    pub has_ancestor_in_same_checkpoint_window: bool,
    pub block_stake_winner: NonZeroCurvePointUncompressedStableV1,
    pub block_creator: NonZeroCurvePointUncompressedStableV1,
    pub coinbase_receiver: NonZeroCurvePointUncompressedStableV1,
    pub supercharge_coinbase: bool,
}

/// **OCaml name**: `Sync_status.T.Stable.V1`
///
/// Gid: `1031`
/// Location: [src/lib/sync_status/sync_status.ml:54:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/sync_status/sync_status.ml#L54)
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

/// **OCaml name**: `Trust_system__Peer_status.Stable.V1`
///
/// Gid: `815`
/// Location: [src/lib/trust_system/peer_status.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/trust_system/peer_status.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TrustSystemPeerStatusStableV1 {
    pub trust: crate::number::Float64,
    pub banned: TrustSystemBannedStatusStableV1,
}

/// **OCaml name**: `Mina_base__Account.Token_symbol.Stable.V1`
///
/// Gid: `73`
/// Location: [src/string.ml:14:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/string.ml#L14)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountTokenSymbolStableV1(pub crate::string::ByteString);

/// **OCaml name**: `Unsigned_extended.UInt32.Stable.V1`
///
/// Gid: `119`
/// Location: [src/int32.ml:6:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/int32.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct UnsignedExtendedUInt32StableV1(pub crate::number::Int32);

/// **OCaml name**: `Unsigned_extended.UInt64.Stable.V1`
///
/// Gid: `125`
/// Location: [src/int64.ml:6:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/int64.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct UnsignedExtendedUInt64StableV1(pub crate::number::Int64);

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#statement#fp`
///
/// Gid: `458`
/// Location: [src/lib/pickles_types/shifted_value.ml:94:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/shifted_value.ml#L94)
/// Args: crate :: bigint :: BigInt
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum PicklesProofProofsVerified2ReprStableV2StatementFp {
    ShiftedValue(crate::bigint::BigInt),
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#prev_evals#evals#evals#lookup#a`
///
/// Gid: `461`
/// Location: [src/lib/pickles_types/plonk_types.ml:197:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_types.ml#L197)
/// Args: (Vec < crate :: bigint :: BigInt > , Vec < crate :: bigint :: BigInt > ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvalsLookupA {
    pub sorted: Vec<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>)>,
    pub aggreg: (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
    pub table: (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
    pub runtime: Option<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>)>,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#prev_evals#evals#evals`
///
/// Gid: `462`
/// Location: [src/lib/pickles_types/plonk_types.ml:266:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_types.ml#L266)
/// Args: (Vec < crate :: bigint :: BigInt > , Vec < crate :: bigint :: BigInt > ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals {
    pub w: (
        (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
        (
            (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
            (
                (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
                (
                    (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
                    (
                        (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
                        (
                            (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
                            (
                                (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
                                (
                                    (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
                                    (
                                        (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
                                        (
                                            (
                                                Vec<crate::bigint::BigInt>,
                                                Vec<crate::bigint::BigInt>,
                                            ),
                                            (
                                                (
                                                    Vec<crate::bigint::BigInt>,
                                                    Vec<crate::bigint::BigInt>,
                                                ),
                                                (
                                                    (
                                                        Vec<crate::bigint::BigInt>,
                                                        Vec<crate::bigint::BigInt>,
                                                    ),
                                                    (
                                                        (
                                                            Vec<crate::bigint::BigInt>,
                                                            Vec<crate::bigint::BigInt>,
                                                        ),
                                                        (
                                                            (
                                                                Vec<crate::bigint::BigInt>,
                                                                Vec<crate::bigint::BigInt>,
                                                            ),
                                                            (
                                                                (
                                                                    Vec<crate::bigint::BigInt>,
                                                                    Vec<crate::bigint::BigInt>,
                                                                ),
                                                                (),
                                                            ),
                                                        ),
                                                    ),
                                                ),
                                            ),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    ),
    pub z: (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
    pub s: (
        (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
        (
            (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
            (
                (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
                (
                    (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
                    (
                        (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
                        ((Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>), ()),
                    ),
                ),
            ),
        ),
    ),
    pub generic_selector: (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
    pub poseidon_selector: (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
    pub lookup: Option<PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvalsLookupA>,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#prev_evals#evals`
///
/// Gid: `463`
/// Location: [src/lib/pickles_types/plonk_types.ml:456:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_types.ml#L456)
/// Args: (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,) , (Vec < crate :: bigint :: BigInt > , Vec < crate :: bigint :: BigInt > ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PrevEvalsEvals {
    pub public_input: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub evals: PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#prev_evals`
///
/// Gid: `464`
/// Location: [src/lib/pickles_types/plonk_types.ml:489:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_types.ml#L489)
/// Args: crate :: bigint :: BigInt , Vec < crate :: bigint :: BigInt >
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PrevEvals {
    pub evals: PicklesProofProofsVerified2ReprStableV2PrevEvalsEvals,
    pub ft_eval1: crate::bigint::BigInt,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#proof#openings#proof`
///
/// Gid: `465`
/// Location: [src/lib/pickles_types/plonk_types.ml:536:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_types.ml#L536)
/// Args: (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,) , crate :: bigint :: BigInt
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2ProofOpeningsProof {
    pub lr: Vec<(
        (crate::bigint::BigInt, crate::bigint::BigInt),
        (crate::bigint::BigInt, crate::bigint::BigInt),
    )>,
    pub z_1: crate::bigint::BigInt,
    pub z_2: crate::bigint::BigInt,
    pub delta: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub challenge_polynomial_commitment: (crate::bigint::BigInt, crate::bigint::BigInt),
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#proof#openings`
///
/// Gid: `466`
/// Location: [src/lib/pickles_types/plonk_types.ml:558:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_types.ml#L558)
/// Args: (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,) , crate :: bigint :: BigInt , Vec < crate :: bigint :: BigInt >
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2ProofOpenings {
    pub proof: PicklesProofProofsVerified2ReprStableV2ProofOpeningsProof,
    pub evals: PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals,
    pub ft_eval1: crate::bigint::BigInt,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#proof#messages#lookup#a`
///
/// Gid: `469`
/// Location: [src/lib/pickles_types/plonk_types.ml:639:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_types.ml#L639)
/// Args: Vec < (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,) >
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2ProofMessagesLookupA {
    pub sorted: Vec<Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>>,
    pub aggreg: Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>,
    pub runtime: Option<Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>>,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#proof#messages`
///
/// Gid: `470`
/// Location: [src/lib/pickles_types/plonk_types.ml:689:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_types.ml#L689)
/// Args: (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2ProofMessages {
    pub w_comm: (
        Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>,
        (
            Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>,
            (
                Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>,
                (
                    Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>,
                    (
                        Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>,
                        (
                            Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>,
                            (
                                Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>,
                                (
                                    Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>,
                                    (
                                        Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>,
                                        (
                                            Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>,
                                            (
                                                Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>,
                                                (
                                                    Vec<(
                                                        crate::bigint::BigInt,
                                                        crate::bigint::BigInt,
                                                    )>,
                                                    (
                                                        Vec<(
                                                            crate::bigint::BigInt,
                                                            crate::bigint::BigInt,
                                                        )>,
                                                        (
                                                            Vec<(
                                                                crate::bigint::BigInt,
                                                                crate::bigint::BigInt,
                                                            )>,
                                                            (
                                                                Vec<(
                                                                    crate::bigint::BigInt,
                                                                    crate::bigint::BigInt,
                                                                )>,
                                                                (),
                                                            ),
                                                        ),
                                                    ),
                                                ),
                                            ),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    ),
    pub z_comm: Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>,
    pub t_comm: Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>,
    pub lookup: Option<PicklesProofProofsVerified2ReprStableV2ProofMessagesLookupA>,
}

/// Derived name: `Mina_base__Verification_key_wire.Stable.V1#wrap_index`
///
/// Gid: `473`
/// Location: [src/lib/pickles_types/plonk_verification_key_evals.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_verification_key_evals.ml#L9)
/// Args: (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseVerificationKeyWireStableV1WrapIndex {
    pub sigma_comm: (
        (crate::bigint::BigInt, crate::bigint::BigInt),
        (
            (crate::bigint::BigInt, crate::bigint::BigInt),
            (
                (crate::bigint::BigInt, crate::bigint::BigInt),
                (
                    (crate::bigint::BigInt, crate::bigint::BigInt),
                    (
                        (crate::bigint::BigInt, crate::bigint::BigInt),
                        (
                            (crate::bigint::BigInt, crate::bigint::BigInt),
                            ((crate::bigint::BigInt, crate::bigint::BigInt), ()),
                        ),
                    ),
                ),
            ),
        ),
    ),
    pub coefficients_comm: (
        (crate::bigint::BigInt, crate::bigint::BigInt),
        (
            (crate::bigint::BigInt, crate::bigint::BigInt),
            (
                (crate::bigint::BigInt, crate::bigint::BigInt),
                (
                    (crate::bigint::BigInt, crate::bigint::BigInt),
                    (
                        (crate::bigint::BigInt, crate::bigint::BigInt),
                        (
                            (crate::bigint::BigInt, crate::bigint::BigInt),
                            (
                                (crate::bigint::BigInt, crate::bigint::BigInt),
                                (
                                    (crate::bigint::BigInt, crate::bigint::BigInt),
                                    (
                                        (crate::bigint::BigInt, crate::bigint::BigInt),
                                        (
                                            (crate::bigint::BigInt, crate::bigint::BigInt),
                                            (
                                                (crate::bigint::BigInt, crate::bigint::BigInt),
                                                (
                                                    (crate::bigint::BigInt, crate::bigint::BigInt),
                                                    (
                                                        (
                                                            crate::bigint::BigInt,
                                                            crate::bigint::BigInt,
                                                        ),
                                                        (
                                                            (
                                                                crate::bigint::BigInt,
                                                                crate::bigint::BigInt,
                                                            ),
                                                            (
                                                                (
                                                                    crate::bigint::BigInt,
                                                                    crate::bigint::BigInt,
                                                                ),
                                                                (),
                                                            ),
                                                        ),
                                                    ),
                                                ),
                                            ),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    ),
    pub generic_comm: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub psm_comm: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub complete_add_comm: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub mul_comm: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub emul_comm: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub endomul_scalar_comm: (crate::bigint::BigInt, crate::bigint::BigInt),
}

/// Derived name: `Pickles__Reduced_messages_for_next_proof_over_same_field.Wrap.Challenges_vector.Stable.V2#a#challenge`
///
/// Gid: `478`
/// Location: [src/lib/crypto/kimchi_backend/common/scalar_challenge.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/crypto/kimchi_backend/common/scalar_challenge.ml#L6)
/// Args: (LimbVectorConstantHex64StableV1 , (LimbVectorConstantHex64StableV1 , () ,) ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge {
    pub inner: PaddedSeq<LimbVectorConstantHex64StableV1, 2>,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#proof`
///
/// Gid: `493`
/// Location: [src/lib/crypto/kimchi_backend/common/plonk_dlog_proof.ml:160:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/crypto/kimchi_backend/common/plonk_dlog_proof.ml#L160)
///
///
/// Gid: `471`
/// Location: [src/lib/pickles_types/plonk_types.ml:738:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_types/plonk_types.ml#L738)
/// Args: (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,) , crate :: bigint :: BigInt , Vec < crate :: bigint :: BigInt >
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2Proof {
    pub messages: PicklesProofProofsVerified2ReprStableV2ProofMessages,
    pub openings: PicklesProofProofsVerified2ReprStableV2ProofOpenings,
}

/// **OCaml name**: `Pickles_base__Proofs_verified.Stable.V1`
///
/// Gid: `505`
/// Location: [src/lib/pickles_base/proofs_verified.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/proofs_verified.ml#L7)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum PicklesBaseProofsVerifiedStableV1 {
    N0,
    N1,
    N2,
}

/// **OCaml name**: `Limb_vector__Constant.Hex64.Stable.V1`
///
/// Gid: `513`
/// Location: [src/lib/pickles/limb_vector/constant.ml:60:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/limb_vector/constant.ml#L60)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct LimbVectorConstantHex64StableV1(pub UnsignedExtendedUInt64StableV1);

/// **OCaml name**: `Composition_types__Branch_data.Make_str.Domain_log2.Stable.V1`
///
/// Gid: `514`
/// Location: [src/lib/pickles/composition_types/branch_data.ml:24:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/branch_data.ml#L24)
///
///
/// Gid: `161`
/// Location: [src/std_internal.ml:113:2](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/std_internal.ml#L113)
///
///
/// Gid: `89`
/// Location: [src/char.ml:8:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/char.ml#L8)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CompositionTypesBranchDataMakeStrDomainLog2StableV1(pub crate::char::Char);

/// **OCaml name**: `Composition_types__Branch_data.Make_str.Stable.V1`
///
/// Gid: `515`
/// Location: [src/lib/pickles/composition_types/branch_data.ml:51:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/branch_data.ml#L51)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CompositionTypesBranchDataMakeStrStableV1 {
    pub proofs_verified: PicklesBaseProofsVerifiedStableV1,
    pub domain_log2: CompositionTypesBranchDataMakeStrDomainLog2StableV1,
}

/// Derived name: `Pickles__Reduced_messages_for_next_proof_over_same_field.Wrap.Challenges_vector.Stable.V2#a`
///
/// Gid: `516`
/// Location: [src/lib/pickles/composition_types/bulletproof_challenge.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/bulletproof_challenge.ml#L6)
/// Args: PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A {
    pub prechallenge:
        PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge,
}

/// **OCaml name**: `Composition_types__Digest.Constant.Stable.V1`
///
/// Gid: `517`
/// Location: [src/lib/pickles/composition_types/digest.ml:13:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/digest.ml#L13)
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

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#statement#plonk`
///
/// Gid: `518`
/// Location: [src/lib/pickles/composition_types/composition_types.ml:45:14](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L45)
/// Args: (LimbVectorConstantHex64StableV1 , (LimbVectorConstantHex64StableV1 , () ,) ,) , PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2StatementPlonk {
    pub alpha:
        PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge,
    pub beta: (
        LimbVectorConstantHex64StableV1,
        (LimbVectorConstantHex64StableV1, ()),
    ),
    pub gamma: (
        LimbVectorConstantHex64StableV1,
        (LimbVectorConstantHex64StableV1, ()),
    ),
    pub zeta: PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge,
    pub joint_combiner: Option<
        PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge,
    >,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#statement#proof_state#deferred_values`
///
/// Gid: `519`
/// Location: [src/lib/pickles/composition_types/composition_types.ml:206:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L206)
/// Args: PicklesProofProofsVerified2ReprStableV2StatementPlonk , PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge , PicklesProofProofsVerified2ReprStableV2StatementFp , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , () ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) , CompositionTypesBranchDataMakeStrStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValues {
    pub plonk: PicklesProofProofsVerified2ReprStableV2StatementPlonk,
    pub combined_inner_product: PicklesProofProofsVerified2ReprStableV2StatementFp,
    pub b: PicklesProofProofsVerified2ReprStableV2StatementFp,
    pub xi: PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge,
    pub bulletproof_challenges:
        PaddedSeq<PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A, 16>,
    pub branch_data: CompositionTypesBranchDataMakeStrStableV1,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#messages_for_next_wrap_proof`
///
/// Gid: `520`
/// Location: [src/lib/pickles/composition_types/composition_types.ml:342:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L342)
/// Args: (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,) , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2 , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2 , () ,) ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof {
    pub challenge_polynomial_commitment: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub old_bulletproof_challenges: (
        PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2,
        (
            PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2,
            (),
        ),
    ),
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#statement#proof_state`
///
/// Gid: `521`
/// Location: [src/lib/pickles/composition_types/composition_types.ml:375:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L375)
/// Args: PicklesProofProofsVerified2ReprStableV2StatementPlonk , PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge , PicklesProofProofsVerified2ReprStableV2StatementFp , PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof , CompositionTypesDigestConstantStableV1 , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , () ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) , CompositionTypesBranchDataMakeStrStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2StatementProofState {
    pub deferred_values: PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValues,
    pub sponge_digest_before_evaluations: CompositionTypesDigestConstantStableV1,
    pub messages_for_next_wrap_proof:
        PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#statement`
///
/// Gid: `523`
/// Location: [src/lib/pickles/composition_types/composition_types.ml:625:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L625)
/// Args: (LimbVectorConstantHex64StableV1 , (LimbVectorConstantHex64StableV1 , () ,) ,) , PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge , PicklesProofProofsVerified2ReprStableV2StatementFp , PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof , CompositionTypesDigestConstantStableV1 , PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , () ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) , CompositionTypesBranchDataMakeStrStableV1
///
///
/// Gid: `522`
/// Location: [src/lib/pickles/composition_types/composition_types.ml:588:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/composition_types/composition_types.ml#L588)
/// Args: PicklesProofProofsVerified2ReprStableV2StatementPlonk , PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge , PicklesProofProofsVerified2ReprStableV2StatementFp , PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof , CompositionTypesDigestConstantStableV1 , PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , () ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) , CompositionTypesBranchDataMakeStrStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2Statement {
    pub proof_state: PicklesProofProofsVerified2ReprStableV2StatementProofState,
    pub messages_for_next_step_proof:
        PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#messages_for_next_step_proof`
///
/// Gid: `526`
/// Location: [src/lib/pickles/reduced_messages_for_next_proof_over_same_field.ml:16:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/reduced_messages_for_next_proof_over_same_field.ml#L16)
/// Args: () , Vec < (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,) > , Vec < (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , () ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) >
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof {
    pub app_state: (),
    pub challenge_polynomial_commitments: Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>,
    pub old_bulletproof_challenges: Vec<
        PaddedSeq<PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A, 16>,
    >,
}

/// **OCaml name**: `Pickles__Reduced_messages_for_next_proof_over_same_field.Wrap.Challenges_vector.Stable.V2`
///
/// Gid: `527`
/// Location: [src/lib/pickles/reduced_messages_for_next_proof_over_same_field.ml:53:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/reduced_messages_for_next_proof_over_same_field.ml#L53)
///
///
/// Gid: `484`
/// Location: [src/lib/crypto/kimchi_backend/pasta/basic.ml:32:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/crypto/kimchi_backend/pasta/basic.ml#L32)
/// Args: PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2(
    pub PaddedSeq<PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A, 15>,
);

/// **OCaml name**: `Mina_base__Verification_key_wire.Stable.V1`
///
/// Gid: `528`
/// Location: [src/lib/pickles/side_loaded_verification_key.ml:170:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/side_loaded_verification_key.ml#L170)
///
///
/// Gid: `511`
/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:150:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles_base/side_loaded_verification_key.ml#L150)
/// Args: (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseVerificationKeyWireStableV1 {
    pub max_proofs_verified: PicklesBaseProofsVerifiedStableV1,
    pub wrap_index: MinaBaseVerificationKeyWireStableV1WrapIndex,
}

/// **OCaml name**: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2`
///
/// Gid: `531`
/// Location: [src/lib/pickles/proof.ml:340:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L340)
///
///
/// Gid: `530`
/// Location: [src/lib/pickles/proof.ml:47:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L47)
/// Args: PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof , PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2 {
    pub statement: PicklesProofProofsVerified2ReprStableV2Statement,
    pub prev_evals: PicklesProofProofsVerified2ReprStableV2PrevEvals,
    pub proof: PicklesProofProofsVerified2ReprStableV2Proof,
}

/// **OCaml name**: `Pickles__Proof.Proofs_verified_max.Stable.V2`
///
/// Gid: `532`
/// Location: [src/lib/pickles/proof.ml:413:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L413)
///
///
/// Gid: `530`
/// Location: [src/lib/pickles/proof.ml:47:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/pickles/proof.ml#L47)
/// Args: PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof , PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerifiedMaxStableV2 {
    pub statement: PicklesProofProofsVerified2ReprStableV2Statement,
    pub prev_evals: PicklesProofProofsVerified2ReprStableV2PrevEvals,
    pub proof: PicklesProofProofsVerified2ReprStableV2Proof,
}

/// **OCaml name**: `Mina_numbers__Nat.Make32.Stable.V1`
///
/// Gid: `535`
/// Location: [src/lib/mina_numbers/nat.ml:258:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_numbers/nat.ml#L258)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaNumbersNatMake32StableV1(pub UnsignedExtendedUInt32StableV1);

/// **OCaml name**: `Sgn.Stable.V1`
///
/// Gid: `551`
/// Location: [src/lib/sgn/sgn.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/sgn/sgn.ml#L9)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum SgnStableV1 {
    Pos,
    Neg,
}

/// Derived name: `Mina_transaction_logic__Parties_logic.Local_state.Value.Stable.V1#excess`
///
/// Gid: `552`
/// Location: [src/lib/currency/signed_poly.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/signed_poly.ml#L6)
/// Args: CurrencyMakeStrAmountMakeStrStableV1 , SgnStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicPartiesLogicLocalStateValueStableV1Excess {
    pub magnitude: CurrencyMakeStrAmountMakeStrStableV1,
    pub sgn: SgnStableV1,
}

/// Derived name: `Mina_base__Fee_excess.Stable.V1#fee`
///
/// Gid: `552`
/// Location: [src/lib/currency/signed_poly.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/signed_poly.ml#L6)
/// Args: CurrencyMakeStrFeeStableV1 , SgnStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeExcessStableV1Fee {
    pub magnitude: CurrencyMakeStrFeeStableV1,
    pub sgn: SgnStableV1,
}

/// **OCaml name**: `Currency.Make_str.Fee.Stable.V1`
///
/// Gid: `553`
/// Location: [src/lib/currency/currency.ml:898:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/currency.ml#L898)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CurrencyMakeStrFeeStableV1(pub UnsignedExtendedUInt64StableV1);

/// **OCaml name**: `Currency.Make_str.Amount.Make_str.Stable.V1`
///
/// Gid: `554`
/// Location: [src/lib/currency/currency.ml:1030:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/currency.ml#L1030)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CurrencyMakeStrAmountMakeStrStableV1(pub UnsignedExtendedUInt64StableV1);

/// **OCaml name**: `Currency.Make_str.Balance.Stable.V1`
///
/// Gid: `555`
/// Location: [src/lib/currency/currency.ml:1072:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/currency/currency.ml#L1072)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CurrencyMakeStrBalanceStableV1(pub CurrencyMakeStrAmountMakeStrStableV1);

/// **OCaml name**: `Blake2.Make.Stable.V1`
///
/// Gid: `556`
/// Location: [src/binable0.ml:120:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/binable0.ml#L120)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct Blake2MakeStableV1(pub crate::string::ByteString);

/// **OCaml name**: `Non_zero_curve_point.Uncompressed.Stable.V1`
///
/// Gid: `565`
/// Location: [src/lib/non_zero_curve_point/non_zero_curve_point.ml:44:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/non_zero_curve_point.ml#L44)
///
///
/// Gid: `559`
/// Location: [src/lib/non_zero_curve_point/compressed_poly.ml:13:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/non_zero_curve_point/compressed_poly.ml#L13)
/// Args: crate :: bigint :: BigInt , bool
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NonZeroCurvePointUncompressedStableV1 {
    pub x: crate::bigint::BigInt,
    pub is_odd: bool,
}

/// **OCaml name**: `Data_hash_lib__State_hash.Stable.V1`
///
/// Gid: `586`
/// Location: [src/lib/data_hash_lib/state_hash.ml:44:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/data_hash_lib/state_hash.ml#L44)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct DataHashLibStateHashStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Block_time.Make_str.Time.Stable.V1`
///
/// Gid: `595`
/// Location: [src/lib/block_time/block_time.ml:22:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/block_time/block_time.ml#L22)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct BlockTimeMakeStrTimeStableV1(pub UnsignedExtendedUInt64StableV1);

/// Derived name: `Mina_base__Sparse_ledger_base.Stable.V2#tree`
///
/// Gid: `597`
/// Location: [src/lib/sparse_ledger_lib/sparse_ledger.ml:9:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/sparse_ledger_lib/sparse_ledger.ml#L9)
/// Args: MinaBaseLedgerHash0StableV1 , MinaBaseAccountBinableArgStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSparseLedgerBaseStableV2Tree {
    Account(Box<MinaBaseAccountBinableArgStableV2>),
    Hash(MinaBaseLedgerHash0StableV1),
    Node(
        MinaBaseLedgerHash0StableV1,
        Box<MinaBaseSparseLedgerBaseStableV2Tree>,
        Box<MinaBaseSparseLedgerBaseStableV2Tree>,
    ),
}

/// Derived name: `Mina_base__Pending_coinbase.Merkle_tree_versioned.Stable.V2#tree`
///
/// Gid: `597`
/// Location: [src/lib/sparse_ledger_lib/sparse_ledger.ml:9:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/sparse_ledger_lib/sparse_ledger.ml#L9)
/// Args: MinaBasePendingCoinbaseHashVersionedStableV1 , MinaBasePendingCoinbaseStackVersionedStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBasePendingCoinbaseMerkleTreeVersionedStableV2Tree {
    Account(MinaBasePendingCoinbaseStackVersionedStableV1),
    Hash(MinaBasePendingCoinbaseHashVersionedStableV1),
    Node(
        MinaBasePendingCoinbaseHashVersionedStableV1,
        Box<MinaBasePendingCoinbaseMerkleTreeVersionedStableV2Tree>,
        Box<MinaBasePendingCoinbaseMerkleTreeVersionedStableV2Tree>,
    ),
}

/// Derived name: `Transaction_snark_work.T.Stable.V2#proofs`
///
/// Gid: `599`
/// Location: [src/lib/one_or_two/one_or_two.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/one_or_two/one_or_two.ml#L7)
/// Args: LedgerProofProdStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[polymorphic_variant]
pub enum TransactionSnarkWorkTStableV2Proofs {
    One(LedgerProofProdStableV2),
    Two((LedgerProofProdStableV2, LedgerProofProdStableV2)),
}

/// **OCaml name**: `Mina_base__Account_id.Make_str.Digest.Stable.V1`
///
/// Gid: `601`
/// Location: [src/lib/mina_base/account_id.ml:64:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account_id.ml#L64)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountIdMakeStrDigestStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Account_id.Make_str.Stable.V2`
///
/// Gid: `606`
/// Location: [src/lib/mina_base/account_id.ml:147:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account_id.ml#L147)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountIdMakeStrStableV2(
    pub NonZeroCurvePointUncompressedStableV1,
    pub MinaBaseAccountIdMakeStrDigestStableV1,
);

/// **OCaml name**: `Mina_base__Account_timing.Stable.V1`
///
/// Gid: `612`
/// Location: [src/lib/mina_base/account_timing.ml:30:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account_timing.ml#L30)
///
///
/// Gid: `611`
/// Location: [src/lib/mina_base/account_timing.ml:13:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/account_timing.ml#L13)
/// Args: UnsignedExtendedUInt32StableV1 , CurrencyMakeStrBalanceStableV1 , CurrencyMakeStrAmountMakeStrStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountTimingStableV1 {
    Untimed,
    Timed {
        initial_minimum_balance: CurrencyMakeStrBalanceStableV1,
        cliff_time: UnsignedExtendedUInt32StableV1,
        cliff_amount: CurrencyMakeStrAmountMakeStrStableV1,
        vesting_period: UnsignedExtendedUInt32StableV1,
        vesting_increment: CurrencyMakeStrAmountMakeStrStableV1,
    },
}

/// **OCaml name**: `Mina_base__Signature.Stable.V1`
///
/// Gid: `615`
/// Location: [src/lib/mina_base/signature.ml:18:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signature.ml#L18)
///
///
/// Gid: `613`
/// Location: [src/lib/mina_base/signature_poly.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signature_poly.ml#L6)
/// Args: crate :: bigint :: BigInt , crate :: bigint :: BigInt
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignatureStableV1(pub crate::bigint::BigInt, pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Control.Stable.V2`
///
/// Gid: `616`
/// Location: [src/lib/mina_base/control.ml:11:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/control.ml#L11)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseControlStableV2 {
    Proof(Box<PicklesProofProofsVerifiedMaxStableV2>),
    Signature(MinaBaseSignatureStableV1),
    NoneGiven,
}

/// **OCaml name**: `Mina_base__Fee_excess.Stable.V1`
///
/// Gid: `618`
/// Location: [src/lib/mina_base/fee_excess.ml:123:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_excess.ml#L123)
///
///
/// Gid: `617`
/// Location: [src/lib/mina_base/fee_excess.ml:54:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_excess.ml#L54)
/// Args: MinaBaseAccountIdMakeStrDigestStableV1 , MinaBaseFeeExcessStableV1Fee
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeExcessStableV1 {
    pub fee_token_l: MinaBaseAccountIdMakeStrDigestStableV1,
    pub fee_excess_l: MinaBaseFeeExcessStableV1Fee,
    pub fee_token_r: MinaBaseAccountIdMakeStrDigestStableV1,
    pub fee_excess_r: MinaBaseFeeExcessStableV1Fee,
}

/// **OCaml name**: `Mina_base__Payment_payload.Stable.V2`
///
/// Gid: `620`
/// Location: [src/lib/mina_base/payment_payload.ml:27:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/payment_payload.ml#L27)
///
///
/// Gid: `619`
/// Location: [src/lib/mina_base/payment_payload.ml:14:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/payment_payload.ml#L14)
/// Args: NonZeroCurvePointUncompressedStableV1 , CurrencyMakeStrAmountMakeStrStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePaymentPayloadStableV2 {
    pub source_pk: NonZeroCurvePointUncompressedStableV1,
    pub receiver_pk: NonZeroCurvePointUncompressedStableV1,
    pub amount: CurrencyMakeStrAmountMakeStrStableV1,
}

/// **OCaml name**: `Mina_base__Ledger_hash0.Stable.V1`
///
/// Gid: `623`
/// Location: [src/lib/mina_base/ledger_hash0.ml:17:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/ledger_hash0.ml#L17)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseLedgerHash0StableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Permissions.Auth_required.Stable.V2`
///
/// Gid: `626`
/// Location: [src/lib/mina_base/permissions.ml:53:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/permissions.ml#L53)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBasePermissionsAuthRequiredStableV2 {
    None,
    Either,
    Proof,
    Signature,
    Impossible,
}

/// **OCaml name**: `Mina_base__Permissions.Stable.V2`
///
/// Gid: `628`
/// Location: [src/lib/mina_base/permissions.ml:352:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/permissions.ml#L352)
///
///
/// Gid: `627`
/// Location: [src/lib/mina_base/permissions.ml:319:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/permissions.ml#L319)
/// Args: MinaBasePermissionsAuthRequiredStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePermissionsStableV2 {
    pub edit_state: MinaBasePermissionsAuthRequiredStableV2,
    pub send: MinaBasePermissionsAuthRequiredStableV2,
    pub receive: MinaBasePermissionsAuthRequiredStableV2,
    pub set_delegate: MinaBasePermissionsAuthRequiredStableV2,
    pub set_permissions: MinaBasePermissionsAuthRequiredStableV2,
    pub set_verification_key: MinaBasePermissionsAuthRequiredStableV2,
    pub set_zkapp_uri: MinaBasePermissionsAuthRequiredStableV2,
    pub edit_sequence_state: MinaBasePermissionsAuthRequiredStableV2,
    pub set_token_symbol: MinaBasePermissionsAuthRequiredStableV2,
    pub increment_nonce: MinaBasePermissionsAuthRequiredStableV2,
    pub set_voting_for: MinaBasePermissionsAuthRequiredStableV2,
}

/// **OCaml name**: `Mina_base__Signed_command_memo.Make_str.Stable.V1`
///
/// Gid: `629`
/// Location: [src/lib/mina_base/signed_command_memo.ml:19:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_memo.ml#L19)
///
///
/// Gid: `170`
/// Location: [src/std_internal.ml:140:2](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/std_internal.ml#L140)
///
///
/// Gid: `83`
/// Location: [src/string.ml:44:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/string.ml#L44)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandMemoMakeStrStableV1(pub crate::string::ByteString);

/// **OCaml name**: `Mina_base__Stake_delegation.Stable.V1`
///
/// Gid: `630`
/// Location: [src/lib/mina_base/stake_delegation.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/stake_delegation.ml#L9)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseStakeDelegationStableV1 {
    SetDelegate {
        delegator: NonZeroCurvePointUncompressedStableV1,
        new_delegate: NonZeroCurvePointUncompressedStableV1,
    },
}

/// **OCaml name**: `Mina_base__Signed_command_payload.Common.Stable.V2`
///
/// Gid: `633`
/// Location: [src/lib/mina_base/signed_command_payload.ml:80:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L80)
///
///
/// Gid: `631`
/// Location: [src/lib/mina_base/signed_command_payload.ml:40:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L40)
/// Args: CurrencyMakeStrFeeStableV1 , NonZeroCurvePointUncompressedStableV1 , UnsignedExtendedUInt32StableV1 , UnsignedExtendedUInt32StableV1 , MinaBaseSignedCommandMemoMakeStrStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandPayloadCommonStableV2 {
    pub fee: CurrencyMakeStrFeeStableV1,
    pub fee_payer_pk: NonZeroCurvePointUncompressedStableV1,
    pub nonce: UnsignedExtendedUInt32StableV1,
    pub valid_until: UnsignedExtendedUInt32StableV1,
    pub memo: MinaBaseSignedCommandMemoMakeStrStableV1,
}

/// **OCaml name**: `Mina_base__Signed_command_payload.Body.Stable.V2`
///
/// Gid: `635`
/// Location: [src/lib/mina_base/signed_command_payload.ml:190:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L190)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSignedCommandPayloadBodyStableV2 {
    Payment(MinaBasePaymentPayloadStableV2),
    StakeDelegation(MinaBaseStakeDelegationStableV1),
}

/// **OCaml name**: `Mina_base__Signed_command_payload.Stable.V2`
///
/// Gid: `637`
/// Location: [src/lib/mina_base/signed_command_payload.ml:275:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L275)
///
///
/// Gid: `636`
/// Location: [src/lib/mina_base/signed_command_payload.ml:257:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command_payload.ml#L257)
/// Args: MinaBaseSignedCommandPayloadCommonStableV2 , MinaBaseSignedCommandPayloadBodyStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandPayloadStableV2 {
    pub common: MinaBaseSignedCommandPayloadCommonStableV2,
    pub body: MinaBaseSignedCommandPayloadBodyStableV2,
}

/// **OCaml name**: `Mina_base__Signed_command.Make_str.Stable.V2`
///
/// Gid: `639`
/// Location: [src/lib/mina_base/signed_command.ml:39:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command.ml#L39)
///
///
/// Gid: `638`
/// Location: [src/lib/mina_base/signed_command.ml:25:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/signed_command.ml#L25)
/// Args: MinaBaseSignedCommandPayloadStableV2 , NonZeroCurvePointUncompressedStableV1 , MinaBaseSignatureStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandMakeStrStableV2 {
    pub payload: MinaBaseSignedCommandPayloadStableV2,
    pub signer: NonZeroCurvePointUncompressedStableV1,
    pub signature: MinaBaseSignatureStableV1,
}

/// **OCaml name**: `Mina_base__Receipt.Chain_hash.Stable.V1`
///
/// Gid: `643`
/// Location: [src/lib/mina_base/receipt.ml:31:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/receipt.ml#L31)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseReceiptChainHashStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__State_body_hash.Stable.V1`
///
/// Gid: `648`
/// Location: [src/lib/mina_base/state_body_hash.ml:19:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/state_body_hash.ml#L19)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStateBodyHashStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Token_permissions.Stable.V1`
///
/// Gid: `653`
/// Location: [src/lib/mina_base/token_permissions.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/token_permissions.ml#L9)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseTokenPermissionsStableV1 {
    TokenOwned { disable_new_accounts: bool },
    NotOwned { account_disabled: bool },
}

/// Derived name: `Mina_base__Party.Update.Stable.V1#voting_for`
///
/// Gid: `655`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: DataHashLibStateHashStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBasePartyUpdateStableV1VotingFor {
    Set(DataHashLibStateHashStableV1),
    Keep,
}

/// Derived name: `Mina_base__Party.Update.Stable.V1#token_symbol`
///
/// Gid: `655`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: MinaBaseAccountTokenSymbolStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBasePartyUpdateStableV1TokenSymbol {
    Set(MinaBaseAccountTokenSymbolStableV1),
    Keep,
}

/// Derived name: `Mina_base__Party.Update.Stable.V1#timing`
///
/// Gid: `655`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: MinaBasePartyUpdateTimingInfoStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBasePartyUpdateStableV1Timing {
    Set(Box<MinaBasePartyUpdateTimingInfoStableV1>),
    Keep,
}

/// Derived name: `Mina_base__Party.Update.Stable.V1#permissions`
///
/// Gid: `655`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: MinaBasePermissionsStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBasePartyUpdateStableV1Permissions {
    Set(Box<MinaBasePermissionsStableV2>),
    Keep,
}

/// Derived name: `Mina_base__Party.Update.Stable.V1#verification_key`
///
/// Gid: `655`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: MinaBaseVerificationKeyWireStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBasePartyUpdateStableV1VerificationKey {
    Set(Box<MinaBaseVerificationKeyWireStableV1>),
    Keep,
}

/// Derived name: `Mina_base__Party.Update.Stable.V1#delegate`
///
/// Gid: `655`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: NonZeroCurvePointUncompressedStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBasePartyUpdateStableV1Delegate {
    Set(NonZeroCurvePointUncompressedStableV1),
    Keep,
}

/// Derived name: `Mina_base__Party.Update.Stable.V1#app_state#a`
///
/// Gid: `655`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: crate :: bigint :: BigInt
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBasePartyUpdateStableV1AppStateA {
    Set(crate::bigint::BigInt),
    Keep,
}

/// Derived name: `Mina_base__Party.Update.Stable.V1#zkapp_uri`
///
/// Gid: `655`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: crate :: string :: ByteString
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBasePartyUpdateStableV1ZkappUri {
    Set(crate::string::ByteString),
    Keep,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Epoch_data.Stable.V1#start_checkpoint`
///
/// Gid: `656`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: DataHashLibStateHashStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionProtocolStateEpochDataStableV1StartCheckpoint {
    Check(DataHashLibStateHashStableV1),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Epoch_data.Stable.V1#epoch_seed`
///
/// Gid: `656`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: MinaBaseEpochSeedStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochSeed {
    Check(MinaBaseEpochSeedStableV1),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1#snarked_ledger_hash`
///
/// Gid: `656`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: MinaBaseLedgerHash0StableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash {
    Check(MinaBaseLedgerHash0StableV1),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Account.Stable.V2#receipt_chain_hash`
///
/// Gid: `656`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: MinaBaseReceiptChainHashStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionAccountStableV2ReceiptChainHash {
    Check(MinaBaseReceiptChainHashStableV1),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Account.Stable.V2#delegate`
///
/// Gid: `656`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: NonZeroCurvePointUncompressedStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionAccountStableV2Delegate {
    Check(NonZeroCurvePointUncompressedStableV1),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Account.Stable.V2#proved_state`
///
/// Gid: `656`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: bool
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionAccountStableV2ProvedState {
    Check(bool),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Account.Stable.V2#state#a`
///
/// Gid: `656`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: crate :: bigint :: BigInt
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionAccountStableV2StateA {
    Check(crate::bigint::BigInt),
    Ignore,
}

/// **OCaml name**: `Mina_base__Zkapp_state.Value.Stable.V1`
///
/// Gid: `659`
/// Location: [src/lib/mina_base/zkapp_state.ml:50:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_state.ml#L50)
///
///
/// Gid: `658`
/// Location: [src/lib/mina_base/zkapp_state.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_state.ml#L17)
/// Args: crate :: bigint :: BigInt
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappStateValueStableV1(
    pub crate::bigint::BigInt,
    pub  (
        crate::bigint::BigInt,
        (
            crate::bigint::BigInt,
            (
                crate::bigint::BigInt,
                (
                    crate::bigint::BigInt,
                    (
                        crate::bigint::BigInt,
                        (crate::bigint::BigInt, (crate::bigint::BigInt, ())),
                    ),
                ),
            ),
        ),
    ),
);

/// **OCaml name**: `Mina_base__Zkapp_account.Stable.V2`
///
/// Gid: `661`
/// Location: [src/lib/mina_base/zkapp_account.ml:216:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_account.ml#L216)
///
///
/// Gid: `660`
/// Location: [src/lib/mina_base/zkapp_account.ml:188:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_account.ml#L188)
/// Args: MinaBaseZkappStateValueStableV1 , Option < MinaBaseVerificationKeyWireStableV1 > , MinaNumbersNatMake32StableV1 , crate :: bigint :: BigInt , UnsignedExtendedUInt32StableV1 , bool
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappAccountStableV2 {
    pub app_state: MinaBaseZkappStateValueStableV1,
    pub verification_key: Option<MinaBaseVerificationKeyWireStableV1>,
    pub zkapp_version: MinaNumbersNatMake32StableV1,
    pub sequence_state: (
        crate::bigint::BigInt,
        (
            crate::bigint::BigInt,
            (
                crate::bigint::BigInt,
                (crate::bigint::BigInt, (crate::bigint::BigInt, ())),
            ),
        ),
    ),
    pub last_sequence_slot: UnsignedExtendedUInt32StableV1,
    pub proved_state: bool,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Epoch_data.Stable.V1#epoch_ledger`
///
/// Gid: `671`
/// Location: [src/lib/mina_base/epoch_ledger.ml:9:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_ledger.ml#L9)
/// Args: MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash , MinaBaseZkappPreconditionProtocolStateStableV1Amount
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochLedger {
    pub hash: MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash,
    pub total_currency: MinaBaseZkappPreconditionProtocolStateStableV1Amount,
}

/// **OCaml name**: `Mina_base__Epoch_ledger.Value.Stable.V1`
///
/// Gid: `672`
/// Location: [src/lib/mina_base/epoch_ledger.ml:23:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_ledger.ml#L23)
///
///
/// Gid: `671`
/// Location: [src/lib/mina_base/epoch_ledger.ml:9:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_ledger.ml#L9)
/// Args: MinaBaseLedgerHash0StableV1 , CurrencyMakeStrAmountMakeStrStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseEpochLedgerValueStableV1 {
    pub hash: MinaBaseLedgerHash0StableV1,
    pub total_currency: CurrencyMakeStrAmountMakeStrStableV1,
}

/// **OCaml name**: `Mina_base__Epoch_seed.Stable.V1`
///
/// Gid: `675`
/// Location: [src/lib/mina_base/epoch_seed.ml:18:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_seed.ml#L18)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseEpochSeedStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Transaction_status.Failure.Stable.V2`
///
/// Gid: `680`
/// Location: [src/lib/mina_base/transaction_status.ml:9:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L9)
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

/// **OCaml name**: `Mina_base__Transaction_status.Failure.Collection.Stable.V1`
///
/// Gid: `682`
/// Location: [src/lib/mina_base/transaction_status.ml:71:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L71)
///
///
/// Gid: `167`
/// Location: [src/std_internal.ml:131:2](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/std_internal.ml#L131)
/// Args: Vec < MinaBaseTransactionStatusFailureStableV2 >
///
///
/// Gid: `50`
/// Location: [src/list0.ml:6:0](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/list0.ml#L6)
/// Args: Vec < MinaBaseTransactionStatusFailureStableV2 >
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseTransactionStatusFailureCollectionStableV1(
    pub Vec<Vec<MinaBaseTransactionStatusFailureStableV2>>,
);

/// **OCaml name**: `Mina_base__Transaction_status.Stable.V2`
///
/// Gid: `683`
/// Location: [src/lib/mina_base/transaction_status.ml:423:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/transaction_status.ml#L423)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseTransactionStatusStableV2 {
    Applied,
    Failed(MinaBaseTransactionStatusFailureCollectionStableV1),
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1#time#a`
///
/// Gid: `684`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:23:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_precondition.ml#L23)
/// Args: BlockTimeMakeStrTimeStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionProtocolStateStableV1TimeA {
    pub lower: BlockTimeMakeStrTimeStableV1,
    pub upper: BlockTimeMakeStrTimeStableV1,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1#amount#a`
///
/// Gid: `684`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:23:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_precondition.ml#L23)
/// Args: CurrencyMakeStrAmountMakeStrStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionProtocolStateStableV1AmountA {
    pub lower: CurrencyMakeStrAmountMakeStrStableV1,
    pub upper: CurrencyMakeStrAmountMakeStrStableV1,
}

/// Derived name: `Mina_base__Zkapp_precondition.Account.Stable.V2#balance#a`
///
/// Gid: `684`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:23:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_precondition.ml#L23)
/// Args: CurrencyMakeStrBalanceStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionAccountStableV2BalanceA {
    pub lower: CurrencyMakeStrBalanceStableV1,
    pub upper: CurrencyMakeStrBalanceStableV1,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1#length#a`
///
/// Gid: `684`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:23:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_precondition.ml#L23)
/// Args: UnsignedExtendedUInt32StableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionProtocolStateStableV1LengthA {
    pub lower: UnsignedExtendedUInt32StableV1,
    pub upper: UnsignedExtendedUInt32StableV1,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1#time`
///
/// Gid: `685`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:178:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_precondition.ml#L178)
/// Args: BlockTimeMakeStrTimeStableV1
///
///
/// Gid: `656`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: MinaBaseZkappPreconditionProtocolStateStableV1TimeA
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionProtocolStateStableV1Time {
    Check(MinaBaseZkappPreconditionProtocolStateStableV1TimeA),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1#amount`
///
/// Gid: `685`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:178:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_precondition.ml#L178)
/// Args: CurrencyMakeStrAmountMakeStrStableV1
///
///
/// Gid: `656`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: MinaBaseZkappPreconditionProtocolStateStableV1AmountA
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionProtocolStateStableV1Amount {
    Check(MinaBaseZkappPreconditionProtocolStateStableV1AmountA),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Account.Stable.V2#balance`
///
/// Gid: `685`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:178:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_precondition.ml#L178)
/// Args: CurrencyMakeStrBalanceStableV1
///
///
/// Gid: `656`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: MinaBaseZkappPreconditionAccountStableV2BalanceA
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionAccountStableV2Balance {
    Check(MinaBaseZkappPreconditionAccountStableV2BalanceA),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1#length`
///
/// Gid: `685`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:178:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_precondition.ml#L178)
/// Args: UnsignedExtendedUInt32StableV1
///
///
/// Gid: `656`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: MinaBaseZkappPreconditionProtocolStateStableV1LengthA
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionProtocolStateStableV1Length {
    Check(MinaBaseZkappPreconditionProtocolStateStableV1LengthA),
    Ignore,
}

/// **OCaml name**: `Mina_base__Zkapp_precondition.Account.Stable.V2`
///
/// Gid: `686`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:474:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_precondition.ml#L474)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionAccountStableV2 {
    pub balance: MinaBaseZkappPreconditionAccountStableV2Balance,
    pub nonce: MinaBaseZkappPreconditionProtocolStateStableV1Length,
    pub receipt_chain_hash: MinaBaseZkappPreconditionAccountStableV2ReceiptChainHash,
    pub delegate: MinaBaseZkappPreconditionAccountStableV2Delegate,
    pub state: (
        MinaBaseZkappPreconditionAccountStableV2StateA,
        (
            MinaBaseZkappPreconditionAccountStableV2StateA,
            (
                MinaBaseZkappPreconditionAccountStableV2StateA,
                (
                    MinaBaseZkappPreconditionAccountStableV2StateA,
                    (
                        MinaBaseZkappPreconditionAccountStableV2StateA,
                        (
                            MinaBaseZkappPreconditionAccountStableV2StateA,
                            (
                                MinaBaseZkappPreconditionAccountStableV2StateA,
                                (MinaBaseZkappPreconditionAccountStableV2StateA, ()),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    ),
    pub sequence_state: MinaBaseZkappPreconditionAccountStableV2StateA,
    pub proved_state: MinaBaseZkappPreconditionAccountStableV2ProvedState,
    pub is_new: MinaBaseZkappPreconditionAccountStableV2ProvedState,
}

/// **OCaml name**: `Mina_base__Zkapp_precondition.Protocol_state.Epoch_data.Stable.V1`
///
/// Gid: `687`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:790:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_precondition.ml#L790)
///
///
/// Gid: `678`
/// Location: [src/lib/mina_base/epoch_data.ml:8:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_data.ml#L8)
/// Args: MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochLedger , MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochSeed , MinaBaseZkappPreconditionProtocolStateEpochDataStableV1StartCheckpoint , MinaBaseZkappPreconditionProtocolStateEpochDataStableV1StartCheckpoint , MinaBaseZkappPreconditionProtocolStateStableV1Length
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionProtocolStateEpochDataStableV1 {
    pub ledger: MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochLedger,
    pub seed: MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochSeed,
    pub start_checkpoint: MinaBaseZkappPreconditionProtocolStateEpochDataStableV1StartCheckpoint,
    pub lock_checkpoint: MinaBaseZkappPreconditionProtocolStateEpochDataStableV1StartCheckpoint,
    pub epoch_length: MinaBaseZkappPreconditionProtocolStateStableV1Length,
}

/// **OCaml name**: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1`
///
/// Gid: `689`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:970:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_precondition.ml#L970)
///
///
/// Gid: `688`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:921:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/zkapp_precondition.ml#L921)
/// Args: MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash , MinaBaseZkappPreconditionProtocolStateStableV1Time , MinaBaseZkappPreconditionProtocolStateStableV1Length , () , MinaBaseZkappPreconditionProtocolStateStableV1Length , MinaBaseZkappPreconditionProtocolStateStableV1Amount , MinaBaseZkappPreconditionProtocolStateEpochDataStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionProtocolStateStableV1 {
    pub snarked_ledger_hash: MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash,
    pub timestamp: MinaBaseZkappPreconditionProtocolStateStableV1Time,
    pub blockchain_length: MinaBaseZkappPreconditionProtocolStateStableV1Length,
    pub min_window_density: MinaBaseZkappPreconditionProtocolStateStableV1Length,
    pub last_vrf_output: (),
    pub total_currency: MinaBaseZkappPreconditionProtocolStateStableV1Amount,
    pub global_slot_since_hard_fork: MinaBaseZkappPreconditionProtocolStateStableV1Length,
    pub global_slot_since_genesis: MinaBaseZkappPreconditionProtocolStateStableV1Length,
    pub staking_epoch_data: MinaBaseZkappPreconditionProtocolStateEpochDataStableV1,
    pub next_epoch_data: MinaBaseZkappPreconditionProtocolStateEpochDataStableV1,
}

/// **OCaml name**: `Mina_base__Party.Call_type.Stable.V1`
///
/// Gid: `696`
/// Location: [src/lib/mina_base/party.ml:27:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/party.ml#L27)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBasePartyCallTypeStableV1 {
    Call,
    DelegateCall,
}

/// **OCaml name**: `Mina_base__Party.Update.Timing_info.Stable.V1`
///
/// Gid: `697`
/// Location: [src/lib/mina_base/party.ml:64:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/party.ml#L64)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePartyUpdateTimingInfoStableV1 {
    pub initial_minimum_balance: CurrencyMakeStrBalanceStableV1,
    pub cliff_time: UnsignedExtendedUInt32StableV1,
    pub cliff_amount: CurrencyMakeStrAmountMakeStrStableV1,
    pub vesting_period: UnsignedExtendedUInt32StableV1,
    pub vesting_increment: CurrencyMakeStrAmountMakeStrStableV1,
}

/// **OCaml name**: `Mina_base__Party.Update.Stable.V1`
///
/// Gid: `698`
/// Location: [src/lib/mina_base/party.ml:219:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/party.ml#L219)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePartyUpdateStableV1 {
    pub app_state: (
        MinaBasePartyUpdateStableV1AppStateA,
        (
            MinaBasePartyUpdateStableV1AppStateA,
            (
                MinaBasePartyUpdateStableV1AppStateA,
                (
                    MinaBasePartyUpdateStableV1AppStateA,
                    (
                        MinaBasePartyUpdateStableV1AppStateA,
                        (
                            MinaBasePartyUpdateStableV1AppStateA,
                            (
                                MinaBasePartyUpdateStableV1AppStateA,
                                (MinaBasePartyUpdateStableV1AppStateA, ()),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    ),
    pub delegate: MinaBasePartyUpdateStableV1Delegate,
    pub verification_key: MinaBasePartyUpdateStableV1VerificationKey,
    pub permissions: MinaBasePartyUpdateStableV1Permissions,
    pub zkapp_uri: MinaBasePartyUpdateStableV1ZkappUri,
    pub token_symbol: MinaBasePartyUpdateStableV1TokenSymbol,
    pub timing: MinaBasePartyUpdateStableV1Timing,
    pub voting_for: MinaBasePartyUpdateStableV1VotingFor,
}

/// **OCaml name**: `Mina_base__Party.Account_precondition.Stable.V1`
///
/// Gid: `699`
/// Location: [src/lib/mina_base/party.ml:510:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/party.ml#L510)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBasePartyAccountPreconditionStableV1 {
    Full(Box<MinaBaseZkappPreconditionAccountStableV2>),
    Nonce(UnsignedExtendedUInt32StableV1),
    Accept,
}

/// **OCaml name**: `Mina_base__Party.Preconditions.Stable.V1`
///
/// Gid: `700`
/// Location: [src/lib/mina_base/party.ml:653:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/party.ml#L653)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePartyPreconditionsStableV1 {
    pub network: MinaBaseZkappPreconditionProtocolStateStableV1,
    pub account: MinaBasePartyAccountPreconditionStableV1,
}

/// **OCaml name**: `Mina_base__Party.Body.Events'.Stable.V1`
///
/// Gid: `701`
/// Location: [src/lib/mina_base/party.ml:729:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/party.ml#L729)
///
///
/// Gid: `167`
/// Location: [src/std_internal.ml:131:2](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/std_internal.ml#L131)
/// Args: Vec < crate :: bigint :: BigInt >
///
///
/// Gid: `50`
/// Location: [src/list0.ml:6:0](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/list0.ml#L6)
/// Args: Vec < crate :: bigint :: BigInt >
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePartyBodyEventsStableV1(pub Vec<Vec<crate::bigint::BigInt>>);

/// **OCaml name**: `Mina_base__Party.Body.Wire.Stable.V1`
///
/// Gid: `702`
/// Location: [src/lib/mina_base/party.ml:741:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/party.ml#L741)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePartyBodyWireStableV1 {
    pub public_key: NonZeroCurvePointUncompressedStableV1,
    pub token_id: MinaBaseAccountIdMakeStrDigestStableV1,
    pub update: MinaBasePartyUpdateStableV1,
    pub balance_change: MinaTransactionLogicPartiesLogicLocalStateValueStableV1Excess,
    pub increment_nonce: bool,
    pub events: MinaBasePartyBodyEventsStableV1,
    pub sequence_events: MinaBasePartyBodyEventsStableV1,
    pub call_data: crate::bigint::BigInt,
    pub preconditions: MinaBasePartyPreconditionsStableV1,
    pub use_full_commitment: bool,
    pub caller: MinaBasePartyCallTypeStableV1,
}

/// **OCaml name**: `Mina_base__Party.Body.Fee_payer.Stable.V1`
///
/// Gid: `706`
/// Location: [src/lib/mina_base/party.ml:963:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/party.ml#L963)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePartyBodyFeePayerStableV1 {
    pub public_key: NonZeroCurvePointUncompressedStableV1,
    pub fee: CurrencyMakeStrFeeStableV1,
    pub valid_until: Option<UnsignedExtendedUInt32StableV1>,
    pub nonce: UnsignedExtendedUInt32StableV1,
}

/// **OCaml name**: `Mina_base__Party.T.Wire.Stable.V1`
///
/// Gid: `709`
/// Location: [src/lib/mina_base/party.ml:1281:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/party.ml#L1281)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePartyTWireStableV1 {
    pub body: MinaBasePartyBodyWireStableV1,
    pub authorization: MinaBaseControlStableV2,
}

/// **OCaml name**: `Mina_base__Party.Fee_payer.Stable.V1`
///
/// Gid: `711`
/// Location: [src/lib/mina_base/party.ml:1355:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/party.ml#L1355)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePartyFeePayerStableV1 {
    pub body: MinaBasePartyBodyFeePayerStableV1,
    pub authorization: MinaBaseSignatureStableV1,
}

/// Derived name: `Mina_base__Parties.T.Stable.V1#other_parties#a#a#calls#a`
///
/// Gid: `712`
/// Location: [src/lib/mina_base/with_stack_hash.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/with_stack_hash.ml#L6)
/// Args: Box < MinaBasePartiesTStableV1OtherPartiesAA > , ()
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePartiesTStableV1OtherPartiesAACallsA {
    pub elt: Box<MinaBasePartiesTStableV1OtherPartiesAA>,
    pub stack_hash: (),
}

/// Derived name: `Mina_base__Parties.T.Stable.V1#other_parties#a`
///
/// Gid: `712`
/// Location: [src/lib/mina_base/with_stack_hash.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/with_stack_hash.ml#L6)
/// Args: MinaBasePartiesTStableV1OtherPartiesAA , ()
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePartiesTStableV1OtherPartiesA {
    pub elt: MinaBasePartiesTStableV1OtherPartiesAA,
    pub stack_hash: (),
}

/// Derived name: `Mina_base__Parties.T.Stable.V1#other_parties#a#a`
///
/// Gid: `713`
/// Location: [src/lib/mina_base/parties.ml:45:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/parties.ml#L45)
/// Args: MinaBasePartyTWireStableV1 , () , ()
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePartiesTStableV1OtherPartiesAA {
    pub party: MinaBasePartyTWireStableV1,
    pub party_digest: (),
    pub calls: Vec<MinaBasePartiesTStableV1OtherPartiesAACallsA>,
}

/// **OCaml name**: `Mina_base__Parties.T.Stable.V1`
///
/// Gid: `722`
/// Location: [src/lib/mina_base/parties.ml:876:12](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/parties.ml#L876)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePartiesTStableV1 {
    pub fee_payer: MinaBasePartyFeePayerStableV1,
    pub other_parties: Vec<MinaBasePartiesTStableV1OtherPartiesA>,
    pub memo: MinaBaseSignedCommandMemoMakeStrStableV1,
}

/// Derived name: `Mina_transaction_logic.Transaction_applied.Coinbase_applied.Stable.V2#coinbase`
///
/// Gid: `728`
/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/with_status.ml#L6)
/// Args: MinaBaseCoinbaseMakeStrStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedCoinbaseAppliedStableV2Coinbase {
    pub data: MinaBaseCoinbaseMakeStrStableV1,
    pub status: MinaBaseTransactionStatusStableV2,
}

/// Derived name: `Mina_transaction_logic.Transaction_applied.Fee_transfer_applied.Stable.V2#fee_transfer`
///
/// Gid: `728`
/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/with_status.ml#L6)
/// Args: MinaBaseFeeTransferMakeStrStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedFeeTransferAppliedStableV2FeeTransfer {
    pub data: MinaBaseFeeTransferMakeStrStableV2,
    pub status: MinaBaseTransactionStatusStableV2,
}

/// Derived name: `Mina_transaction_logic.Transaction_applied.Parties_applied.Stable.V1#command`
///
/// Gid: `728`
/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/with_status.ml#L6)
/// Args: MinaBasePartiesTStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedPartiesAppliedStableV1Command {
    pub data: MinaBasePartiesTStableV1,
    pub status: MinaBaseTransactionStatusStableV2,
}

/// Derived name: `Mina_transaction_logic.Transaction_applied.Signed_command_applied.Common.Stable.V2#user_command`
///
/// Gid: `728`
/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/with_status.ml#L6)
/// Args: MinaBaseSignedCommandMakeStrStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV2UserCommand {
    pub data: MinaBaseSignedCommandMakeStrStableV2,
    pub status: MinaBaseTransactionStatusStableV2,
}

/// Derived name: `Staged_ledger_diff__Diff.Pre_diff_with_at_most_two_coinbase.Stable.V2#b`
///
/// Gid: `728`
/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/with_status.ml#L6)
/// Args: MinaBaseUserCommandStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B {
    pub data: MinaBaseUserCommandStableV2,
    pub status: MinaBaseTransactionStatusStableV2,
}

/// **OCaml name**: `Mina_base__User_command.Stable.V2`
///
/// Gid: `731`
/// Location: [src/lib/mina_base/user_command.ml:67:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/user_command.ml#L67)
///
///
/// Gid: `729`
/// Location: [src/lib/mina_base/user_command.ml:7:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/user_command.ml#L7)
/// Args: MinaBaseSignedCommandMakeStrStableV2 , MinaBasePartiesTStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseUserCommandStableV2 {
    SignedCommand(MinaBaseSignedCommandMakeStrStableV2),
    Parties(MinaBasePartiesTStableV1),
}

/// **OCaml name**: `Mina_base__Fee_transfer.Make_str.Single.Stable.V2`
///
/// Gid: `735`
/// Location: [src/lib/mina_base/fee_transfer.ml:19:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_transfer.ml#L19)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeTransferMakeStrSingleStableV2 {
    pub receiver_pk: NonZeroCurvePointUncompressedStableV1,
    pub fee: CurrencyMakeStrFeeStableV1,
    pub fee_token: MinaBaseAccountIdMakeStrDigestStableV1,
}

/// **OCaml name**: `Mina_base__Fee_transfer.Make_str.Stable.V2`
///
/// Gid: `736`
/// Location: [src/lib/mina_base/fee_transfer.ml:68:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_transfer.ml#L68)
///
///
/// Gid: `599`
/// Location: [src/lib/one_or_two/one_or_two.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/one_or_two/one_or_two.ml#L7)
/// Args: MinaBaseFeeTransferMakeStrSingleStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[polymorphic_variant]
pub enum MinaBaseFeeTransferMakeStrStableV2 {
    One(MinaBaseFeeTransferMakeStrSingleStableV2),
    Two(
        (
            MinaBaseFeeTransferMakeStrSingleStableV2,
            MinaBaseFeeTransferMakeStrSingleStableV2,
        ),
    ),
}

/// **OCaml name**: `Mina_base__Coinbase_fee_transfer.Make_str.Stable.V1`
///
/// Gid: `737`
/// Location: [src/lib/mina_base/coinbase_fee_transfer.ml:15:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/coinbase_fee_transfer.ml#L15)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseCoinbaseFeeTransferMakeStrStableV1 {
    pub receiver_pk: NonZeroCurvePointUncompressedStableV1,
    pub fee: CurrencyMakeStrFeeStableV1,
}

/// **OCaml name**: `Mina_base__Coinbase.Make_str.Stable.V1`
///
/// Gid: `738`
/// Location: [src/lib/mina_base/coinbase.ml:17:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/coinbase.ml#L17)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseCoinbaseMakeStrStableV1 {
    pub receiver: NonZeroCurvePointUncompressedStableV1,
    pub amount: CurrencyMakeStrAmountMakeStrStableV1,
    pub fee_transfer: Option<MinaBaseCoinbaseFeeTransferMakeStrStableV1>,
}

/// **OCaml name**: `Mina_base__Pending_coinbase.Stack_id.Stable.V1`
///
/// Gid: `740`
/// Location: [src/lib/mina_base/pending_coinbase.ml:101:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L101)
///
///
/// Gid: `163`
/// Location: [src/std_internal.ml:119:2](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/std_internal.ml#L119)
///
///
/// Gid: `113`
/// Location: [src/int.ml:19:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/int.ml#L19)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStackIdStableV1(pub crate::number::Int32);

/// **OCaml name**: `Mina_base__Pending_coinbase.Coinbase_stack.Stable.V1`
///
/// Gid: `743`
/// Location: [src/lib/mina_base/pending_coinbase.ml:152:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L152)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseCoinbaseStackStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Pending_coinbase.Stack_hash.Stable.V1`
///
/// Gid: `748`
/// Location: [src/lib/mina_base/pending_coinbase.ml:212:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L212)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStackHashStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Pending_coinbase.State_stack.Stable.V1`
///
/// Gid: `752`
/// Location: [src/lib/mina_base/pending_coinbase.ml:247:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L247)
///
///
/// Gid: `751`
/// Location: [src/lib/mina_base/pending_coinbase.ml:238:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L238)
/// Args: MinaBasePendingCoinbaseStackHashStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStateStackStableV1 {
    pub init: MinaBasePendingCoinbaseStackHashStableV1,
    pub curr: MinaBasePendingCoinbaseStackHashStableV1,
}

/// **OCaml name**: `Mina_base__Pending_coinbase.Hash_builder.Stable.V1`
///
/// Gid: `755`
/// Location: [src/lib/mina_base/pending_coinbase.ml:358:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L358)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseHashBuilderStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Pending_coinbase.Stack_versioned.Stable.V1`
///
/// Gid: `762`
/// Location: [src/lib/mina_base/pending_coinbase.ml:504:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L504)
///
///
/// Gid: `761`
/// Location: [src/lib/mina_base/pending_coinbase.ml:494:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L494)
/// Args: MinaBasePendingCoinbaseCoinbaseStackStableV1 , MinaBasePendingCoinbaseStateStackStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStackVersionedStableV1 {
    pub data: MinaBasePendingCoinbaseCoinbaseStackStableV1,
    pub state: MinaBasePendingCoinbaseStateStackStableV1,
}

/// **OCaml name**: `Mina_base__Pending_coinbase.Hash_versioned.Stable.V1`
///
/// Gid: `763`
/// Location: [src/lib/mina_base/pending_coinbase.ml:517:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L517)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseHashVersionedStableV1(
    pub MinaBasePendingCoinbaseHashBuilderStableV1,
);

/// **OCaml name**: `Mina_base__Pending_coinbase.Merkle_tree_versioned.Stable.V2`
///
/// Gid: `764`
/// Location: [src/lib/mina_base/pending_coinbase.ml:529:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/pending_coinbase.ml#L529)
///
///
/// Gid: `598`
/// Location: [src/lib/sparse_ledger_lib/sparse_ledger.ml:38:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/sparse_ledger_lib/sparse_ledger.ml#L38)
/// Args: MinaBasePendingCoinbaseHashVersionedStableV1 , MinaBasePendingCoinbaseStackIdStableV1 , MinaBasePendingCoinbaseStackVersionedStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseMerkleTreeVersionedStableV2 {
    pub indexes: Vec<(MinaBasePendingCoinbaseStackIdStableV1, crate::number::Int32)>,
    pub depth: crate::number::Int32,
    pub tree: MinaBasePendingCoinbaseMerkleTreeVersionedStableV2Tree,
}

/// **OCaml name**: `Mina_base__Staged_ledger_hash.Aux_hash.Stable.V1`
///
/// Gid: `767`
/// Location: [src/lib/mina_base/staged_ledger_hash.ml:16:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L16)
///
///
/// Gid: `170`
/// Location: [src/std_internal.ml:140:2](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/std_internal.ml#L140)
///
///
/// Gid: `83`
/// Location: [src/string.ml:44:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/string.ml#L44)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashAuxHashStableV1(pub crate::string::ByteString);

/// **OCaml name**: `Mina_base__Staged_ledger_hash.Pending_coinbase_aux.Stable.V1`
///
/// Gid: `768`
/// Location: [src/lib/mina_base/staged_ledger_hash.ml:62:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L62)
///
///
/// Gid: `170`
/// Location: [src/std_internal.ml:140:2](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/std_internal.ml#L140)
///
///
/// Gid: `83`
/// Location: [src/string.ml:44:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/string.ml#L44)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashPendingCoinbaseAuxStableV1(pub crate::string::ByteString);

/// **OCaml name**: `Mina_base__Staged_ledger_hash.Non_snark.Stable.V1`
///
/// Gid: `769`
/// Location: [src/lib/mina_base/staged_ledger_hash.ml:98:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L98)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashNonSnarkStableV1 {
    pub ledger_hash: MinaBaseLedgerHash0StableV1,
    pub aux_hash: MinaBaseStagedLedgerHashAuxHashStableV1,
    pub pending_coinbase_aux: MinaBaseStagedLedgerHashPendingCoinbaseAuxStableV1,
}

/// **OCaml name**: `Mina_base__Staged_ledger_hash.Stable.V1`
///
/// Gid: `771`
/// Location: [src/lib/mina_base/staged_ledger_hash.ml:202:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L202)
///
///
/// Gid: `770`
/// Location: [src/lib/mina_base/staged_ledger_hash.ml:185:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/staged_ledger_hash.ml#L185)
/// Args: MinaBaseStagedLedgerHashNonSnarkStableV1 , MinaBasePendingCoinbaseHashVersionedStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashStableV1 {
    pub non_snark: MinaBaseStagedLedgerHashNonSnarkStableV1,
    pub pending_coinbase_hash: MinaBasePendingCoinbaseHashVersionedStableV1,
}

/// **OCaml name**: `Mina_base__Stack_frame.Digest.Stable.V1`
///
/// Gid: `773`
/// Location: [src/lib/mina_base/stack_frame.ml:55:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/stack_frame.ml#L55)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStackFrameDigestStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Sok_message.Stable.V1`
///
/// Gid: `775`
/// Location: [src/lib/mina_base/sok_message.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/sok_message.ml#L8)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSokMessageStableV1 {
    pub fee: CurrencyMakeStrFeeStableV1,
    pub prover: NonZeroCurvePointUncompressedStableV1,
}

/// **OCaml name**: `Mina_base__Protocol_constants_checked.Value.Stable.V1`
///
/// Gid: `776`
/// Location: [src/lib/mina_base/protocol_constants_checked.ml:22:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/protocol_constants_checked.ml#L22)
///
///
/// Gid: `592`
/// Location: [src/lib/genesis_constants/genesis_constants.ml:239:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/genesis_constants/genesis_constants.ml#L239)
/// Args: UnsignedExtendedUInt32StableV1 , UnsignedExtendedUInt32StableV1 , BlockTimeMakeStrTimeStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseProtocolConstantsCheckedValueStableV1 {
    pub k: UnsignedExtendedUInt32StableV1,
    pub slots_per_epoch: UnsignedExtendedUInt32StableV1,
    pub slots_per_sub_window: UnsignedExtendedUInt32StableV1,
    pub delta: UnsignedExtendedUInt32StableV1,
    pub genesis_state_timestamp: BlockTimeMakeStrTimeStableV1,
}

/// **OCaml name**: `Mina_base__Proof.Stable.V2`
///
/// Gid: `777`
/// Location: [src/lib/mina_base/proof.ml:12:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/proof.ml#L12)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseProofStableV2(pub PicklesProofProofsVerified2ReprStableV2);

/// **OCaml name**: `Mina_base__Call_stack_digest.Stable.V1`
///
/// Gid: `779`
/// Location: [src/lib/mina_base/call_stack_digest.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/call_stack_digest.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseCallStackDigestStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Fee_with_prover.Stable.V1`
///
/// Gid: `780`
/// Location: [src/lib/mina_base/fee_with_prover.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/fee_with_prover.ml#L7)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeWithProverStableV1 {
    pub fee: CurrencyMakeStrFeeStableV1,
    pub prover: NonZeroCurvePointUncompressedStableV1,
}

/// **OCaml name**: `Mina_transaction_logic__Parties_logic.Local_state.Value.Stable.V1`
///
/// Gid: `792`
/// Location: [src/lib/transaction_logic/parties_logic.ml:216:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_logic/parties_logic.ml#L216)
///
///
/// Gid: `791`
/// Location: [src/lib/transaction_logic/parties_logic.ml:170:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_logic/parties_logic.ml#L170)
/// Args: MinaBaseStackFrameDigestStableV1 , MinaBaseCallStackDigestStableV1 , MinaBaseAccountIdMakeStrDigestStableV1 , MinaTransactionLogicPartiesLogicLocalStateValueStableV1Excess , MinaBaseLedgerHash0StableV1 , bool , crate :: bigint :: BigInt , UnsignedExtendedUInt32StableV1 , MinaBaseTransactionStatusFailureCollectionStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicPartiesLogicLocalStateValueStableV1 {
    pub stack_frame: MinaBaseStackFrameDigestStableV1,
    pub call_stack: MinaBaseCallStackDigestStableV1,
    pub transaction_commitment: crate::bigint::BigInt,
    pub full_transaction_commitment: crate::bigint::BigInt,
    pub token_id: MinaBaseAccountIdMakeStrDigestStableV1,
    pub excess: MinaTransactionLogicPartiesLogicLocalStateValueStableV1Excess,
    pub ledger: MinaBaseLedgerHash0StableV1,
    pub success: bool,
    pub party_index: UnsignedExtendedUInt32StableV1,
    pub failure_status_tbl: MinaBaseTransactionStatusFailureCollectionStableV1,
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Signed_command_applied.Common.Stable.V2`
///
/// Gid: `794`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:17:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_logic/mina_transaction_logic.ml#L17)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV2 {
    pub user_command:
        MinaTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV2UserCommand,
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Signed_command_applied.Body.Stable.V2`
///
/// Gid: `795`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:31:10](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_logic/mina_transaction_logic.ml#L31)
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

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Signed_command_applied.Stable.V2`
///
/// Gid: `796`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:46:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_logic/mina_transaction_logic.ml#L46)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedSignedCommandAppliedStableV2 {
    pub common: MinaTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV2,
    pub body: MinaTransactionLogicTransactionAppliedSignedCommandAppliedBodyStableV2,
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Parties_applied.Stable.V1`
///
/// Gid: `797`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:58:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_logic/mina_transaction_logic.ml#L58)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedPartiesAppliedStableV1 {
    pub accounts: Vec<(
        MinaBaseAccountIdMakeStrStableV2,
        Option<MinaBaseAccountBinableArgStableV2>,
    )>,
    pub command: MinaTransactionLogicTransactionAppliedPartiesAppliedStableV1Command,
    pub new_accounts: Vec<MinaBaseAccountIdMakeStrStableV2>,
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Command_applied.Stable.V2`
///
/// Gid: `798`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:75:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_logic/mina_transaction_logic.ml#L75)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaTransactionLogicTransactionAppliedCommandAppliedStableV2 {
    SignedCommand(MinaTransactionLogicTransactionAppliedSignedCommandAppliedStableV2),
    Parties(MinaTransactionLogicTransactionAppliedPartiesAppliedStableV1),
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Fee_transfer_applied.Stable.V2`
///
/// Gid: `799`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:89:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_logic/mina_transaction_logic.ml#L89)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedFeeTransferAppliedStableV2 {
    pub fee_transfer: MinaTransactionLogicTransactionAppliedFeeTransferAppliedStableV2FeeTransfer,
    pub new_accounts: Vec<MinaBaseAccountIdMakeStrStableV2>,
    pub burned_tokens: CurrencyMakeStrAmountMakeStrStableV1,
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Coinbase_applied.Stable.V2`
///
/// Gid: `800`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:105:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_logic/mina_transaction_logic.ml#L105)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedCoinbaseAppliedStableV2 {
    pub coinbase: MinaTransactionLogicTransactionAppliedCoinbaseAppliedStableV2Coinbase,
    pub new_accounts: Vec<MinaBaseAccountIdMakeStrStableV2>,
    pub burned_tokens: CurrencyMakeStrAmountMakeStrStableV1,
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Varying.Stable.V2`
///
/// Gid: `801`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:121:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_logic/mina_transaction_logic.ml#L121)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaTransactionLogicTransactionAppliedVaryingStableV2 {
    Command(MinaTransactionLogicTransactionAppliedCommandAppliedStableV2),
    FeeTransfer(MinaTransactionLogicTransactionAppliedFeeTransferAppliedStableV2),
    Coinbase(MinaTransactionLogicTransactionAppliedCoinbaseAppliedStableV2),
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Stable.V2`
///
/// Gid: `802`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:135:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_logic/mina_transaction_logic.ml#L135)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedStableV2 {
    pub previous_hash: MinaBaseLedgerHash0StableV1,
    pub varying: MinaTransactionLogicTransactionAppliedVaryingStableV2,
}

/// **OCaml name**: `Merkle_address.Binable_arg.Stable.V1`
///
/// Gid: `803`
/// Location: [src/lib/merkle_address/merkle_address.ml:48:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/merkle_address/merkle_address.ml#L48)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MerkleAddressBinableArgStableV1(pub crate::number::Int32, pub crate::string::ByteString);

/// **OCaml name**: `Network_peer__Peer.Id.Stable.V1`
///
/// Gid: `809`
/// Location: [src/lib/network_peer/peer.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_peer/peer.ml#L10)
///
///
/// Gid: `170`
/// Location: [src/std_internal.ml:140:2](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/std_internal.ml#L140)
///
///
/// Gid: `83`
/// Location: [src/string.ml:44:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/string.ml#L44)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NetworkPeerPeerIdStableV1(pub crate::string::ByteString);

/// **OCaml name**: `Trust_system__Banned_status.Stable.V1`
///
/// Gid: `814`
/// Location: [src/lib/trust_system/banned_status.ml:6:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/trust_system/banned_status.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum TrustSystemBannedStatusStableV1 {
    Unbanned,
    BannedUntil(crate::number::Float64),
}

/// **OCaml name**: `Consensus_vrf.Output.Truncated.Stable.V1`
///
/// Gid: `829`
/// Location: [src/lib/consensus/vrf/consensus_vrf.ml:167:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/vrf/consensus_vrf.ml#L167)
///
///
/// Gid: `170`
/// Location: [src/std_internal.ml:140:2](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/std_internal.ml#L140)
///
///
/// Gid: `83`
/// Location: [src/string.ml:44:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/string.ml#L44)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusVrfOutputTruncatedStableV1(pub crate::string::ByteString);

/// **OCaml name**: `Consensus__Body_reference.Stable.V1`
///
/// Gid: `843`
/// Location: [src/lib/consensus/body_reference.ml:17:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/body_reference.ml#L17)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusBodyReferenceStableV1(pub Blake2MakeStableV1);

/// **OCaml name**: `Consensus__Global_slot.Stable.V1`
///
/// Gid: `848`
/// Location: [src/lib/consensus/global_slot.ml:21:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/global_slot.ml#L21)
///
///
/// Gid: `847`
/// Location: [src/lib/consensus/global_slot.ml:11:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/global_slot.ml#L11)
/// Args: UnsignedExtendedUInt32StableV1 , UnsignedExtendedUInt32StableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusGlobalSlotStableV1 {
    pub slot_number: UnsignedExtendedUInt32StableV1,
    pub slots_per_epoch: UnsignedExtendedUInt32StableV1,
}

/// **OCaml name**: `Consensus__Proof_of_stake.Data.Epoch_data.Staking_value_versioned.Value.Stable.V1`
///
/// Gid: `856`
/// Location: [src/lib/consensus/proof_of_stake.ml:1054:12](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1054)
///
///
/// Gid: `678`
/// Location: [src/lib/mina_base/epoch_data.ml:8:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_data.ml#L8)
/// Args: MinaBaseEpochLedgerValueStableV1 , MinaBaseEpochSeedStableV1 , DataHashLibStateHashStableV1 , DataHashLibStateHashStableV1 , UnsignedExtendedUInt32StableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1 {
    pub ledger: MinaBaseEpochLedgerValueStableV1,
    pub seed: MinaBaseEpochSeedStableV1,
    pub start_checkpoint: DataHashLibStateHashStableV1,
    pub lock_checkpoint: DataHashLibStateHashStableV1,
    pub epoch_length: UnsignedExtendedUInt32StableV1,
}

/// **OCaml name**: `Consensus__Proof_of_stake.Data.Epoch_data.Next_value_versioned.Value.Stable.V1`
///
/// Gid: `857`
/// Location: [src/lib/consensus/proof_of_stake.ml:1078:12](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/consensus/proof_of_stake.ml#L1078)
///
///
/// Gid: `678`
/// Location: [src/lib/mina_base/epoch_data.ml:8:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_base/epoch_data.ml#L8)
/// Args: MinaBaseEpochLedgerValueStableV1 , MinaBaseEpochSeedStableV1 , DataHashLibStateHashStableV1 , DataHashLibStateHashStableV1 , UnsignedExtendedUInt32StableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1 {
    pub ledger: MinaBaseEpochLedgerValueStableV1,
    pub seed: MinaBaseEpochSeedStableV1,
    pub start_checkpoint: DataHashLibStateHashStableV1,
    pub lock_checkpoint: DataHashLibStateHashStableV1,
    pub epoch_length: UnsignedExtendedUInt32StableV1,
}

/// Derived name: `Mina_state__Blockchain_state.Value.Stable.V2#registers`
///
/// Gid: `862`
/// Location: [src/lib/mina_state/registers.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/registers.ml#L8)
/// Args: MinaBaseLedgerHash0StableV1 , () , MinaTransactionLogicPartiesLogicLocalStateValueStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateBlockchainStateValueStableV2Registers {
    pub ledger: MinaBaseLedgerHash0StableV1,
    pub pending_coinbase_stack: (),
    pub local_state: MinaTransactionLogicPartiesLogicLocalStateValueStableV1,
}

/// Derived name: `Transaction_snark.Statement.With_sok.Stable.V2#source`
///
/// Gid: `862`
/// Location: [src/lib/mina_state/registers.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/registers.ml#L8)
/// Args: MinaBaseLedgerHash0StableV1 , MinaBasePendingCoinbaseStackVersionedStableV1 , MinaTransactionLogicPartiesLogicLocalStateValueStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkStatementWithSokStableV2Source {
    pub ledger: MinaBaseLedgerHash0StableV1,
    pub pending_coinbase_stack: MinaBasePendingCoinbaseStackVersionedStableV1,
    pub local_state: MinaTransactionLogicPartiesLogicLocalStateValueStableV1,
}

/// **OCaml name**: `Mina_state__Blockchain_state.Value.Stable.V2`
///
/// Gid: `864`
/// Location: [src/lib/mina_state/blockchain_state.ml:43:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/blockchain_state.ml#L43)
///
///
/// Gid: `863`
/// Location: [src/lib/mina_state/blockchain_state.ml:9:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/blockchain_state.ml#L9)
/// Args: MinaBaseStagedLedgerHashStableV1 , MinaBaseLedgerHash0StableV1 , MinaTransactionLogicPartiesLogicLocalStateValueStableV1 , BlockTimeMakeStrTimeStableV1 , ConsensusBodyReferenceStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateBlockchainStateValueStableV2 {
    pub staged_ledger_hash: MinaBaseStagedLedgerHashStableV1,
    pub genesis_ledger_hash: MinaBaseLedgerHash0StableV1,
    pub registers: MinaStateBlockchainStateValueStableV2Registers,
    pub timestamp: BlockTimeMakeStrTimeStableV1,
    pub body_reference: ConsensusBodyReferenceStableV1,
}

/// **OCaml name**: `Mina_state__Protocol_state.Body.Value.Stable.V2`
///
/// Gid: `870`
/// Location: [src/lib/mina_state/protocol_state.ml:53:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L53)
///
///
/// Gid: `868`
/// Location: [src/lib/mina_state/protocol_state.ml:38:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_state/protocol_state.ml#L38)
/// Args: DataHashLibStateHashStableV1 , MinaStateBlockchainStateValueStableV2 , ConsensusProofOfStakeDataConsensusStateValueStableV1 , MinaBaseProtocolConstantsCheckedValueStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateProtocolStateBodyValueStableV2 {
    pub genesis_state_hash: DataHashLibStateHashStableV1,
    pub blockchain_state: MinaStateBlockchainStateValueStableV2,
    pub consensus_state: ConsensusProofOfStakeDataConsensusStateValueStableV1,
    pub constants: MinaBaseProtocolConstantsCheckedValueStableV1,
}

/// **OCaml name**: `Transaction_snark.Pending_coinbase_stack_state.Init_stack.Stable.V1`
///
/// Gid: `899`
/// Location: [src/lib/transaction_snark/transaction_snark.ml:56:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L56)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum TransactionSnarkPendingCoinbaseStackStateInitStackStableV1 {
    Base(MinaBasePendingCoinbaseStackVersionedStableV1),
    Merge,
}

/// **OCaml name**: `Transaction_snark.Statement.Stable.V2`
///
/// Gid: `905`
/// Location: [src/lib/transaction_snark/transaction_snark.ml:205:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L205)
///
///
/// Gid: `904`
/// Location: [src/lib/transaction_snark/transaction_snark.ml:122:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L122)
/// Args: MinaBaseLedgerHash0StableV1 , MinaTransactionLogicPartiesLogicLocalStateValueStableV1Excess , MinaBasePendingCoinbaseStackVersionedStableV1 , MinaBaseFeeExcessStableV1 , () , MinaTransactionLogicPartiesLogicLocalStateValueStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkStatementStableV2 {
    pub source: TransactionSnarkStatementWithSokStableV2Source,
    pub target: TransactionSnarkStatementWithSokStableV2Source,
    pub supply_increase: MinaTransactionLogicPartiesLogicLocalStateValueStableV1Excess,
    pub fee_excess: MinaBaseFeeExcessStableV1,
    pub sok_digest: (),
}

/// **OCaml name**: `Transaction_snark.Statement.With_sok.Stable.V2`
///
/// Gid: `906`
/// Location: [src/lib/transaction_snark/transaction_snark.ml:223:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L223)
///
///
/// Gid: `904`
/// Location: [src/lib/transaction_snark/transaction_snark.ml:122:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L122)
/// Args: MinaBaseLedgerHash0StableV1 , MinaTransactionLogicPartiesLogicLocalStateValueStableV1Excess , MinaBasePendingCoinbaseStackVersionedStableV1 , MinaBaseFeeExcessStableV1 , MinaBaseAccountTokenSymbolStableV1 , MinaTransactionLogicPartiesLogicLocalStateValueStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkStatementWithSokStableV2 {
    pub source: TransactionSnarkStatementWithSokStableV2Source,
    pub target: TransactionSnarkStatementWithSokStableV2Source,
    pub supply_increase: MinaTransactionLogicPartiesLogicLocalStateValueStableV1Excess,
    pub fee_excess: MinaBaseFeeExcessStableV1,
    pub sok_digest: MinaBaseAccountTokenSymbolStableV1,
}

/// **OCaml name**: `Transaction_snark.Proof.Stable.V2`
///
/// Gid: `909`
/// Location: [src/lib/transaction_snark/transaction_snark.ml:378:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L378)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkProofStableV2(pub PicklesProofProofsVerified2ReprStableV2);

/// **OCaml name**: `Transaction_snark.Stable.V2`
///
/// Gid: `910`
/// Location: [src/lib/transaction_snark/transaction_snark.ml:389:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark/transaction_snark.ml#L389)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkStableV2 {
    pub statement: TransactionSnarkStatementWithSokStableV2,
    pub proof: TransactionSnarkProofStableV2,
}

/// **OCaml name**: `Ledger_proof.Prod.Stable.V2`
///
/// Gid: `913`
/// Location: [src/lib/ledger_proof/ledger_proof.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/ledger_proof/ledger_proof.ml#L10)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct LedgerProofProdStableV2(pub TransactionSnarkStableV2);

/// **OCaml name**: `Transaction_snark_work.Statement.Stable.V2`
///
/// Gid: `915`
/// Location: [src/lib/transaction_snark_work/transaction_snark_work.ml:23:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_work/transaction_snark_work.ml#L23)
///
///
/// Gid: `599`
/// Location: [src/lib/one_or_two/one_or_two.ml:7:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/one_or_two/one_or_two.ml#L7)
/// Args: TransactionSnarkStatementStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[polymorphic_variant]
pub enum TransactionSnarkWorkStatementStableV2 {
    One(TransactionSnarkStatementStableV2),
    Two(
        (
            TransactionSnarkStatementStableV2,
            TransactionSnarkStatementStableV2,
        ),
    ),
}

/// **OCaml name**: `Transaction_snark_work.T.Stable.V2`
///
/// Gid: `919`
/// Location: [src/lib/transaction_snark_work/transaction_snark_work.ml:82:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_work/transaction_snark_work.ml#L82)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkWorkTStableV2 {
    pub fee: CurrencyMakeStrFeeStableV1,
    pub proofs: TransactionSnarkWorkTStableV2Proofs,
    pub prover: NonZeroCurvePointUncompressedStableV1,
}

/// Derived name: `Staged_ledger_diff__Diff.Pre_diff_with_at_most_two_coinbase.Stable.V2#coinbase`
///
/// Gid: `920`
/// Location: [src/lib/staged_ledger_diff/diff.ml:10:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/diff.ml#L10)
/// Args: StagedLedgerDiffDiffFtStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2Coinbase {
    Zero,
    One(Option<StagedLedgerDiffDiffFtStableV1>),
    Two(
        Option<(
            StagedLedgerDiffDiffFtStableV1,
            Option<StagedLedgerDiffDiffFtStableV1>,
        )>,
    ),
}

/// Derived name: `Staged_ledger_diff__Diff.Pre_diff_with_at_most_one_coinbase.Stable.V2#coinbase`
///
/// Gid: `921`
/// Location: [src/lib/staged_ledger_diff/diff.ml:43:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/diff.ml#L43)
/// Args: StagedLedgerDiffDiffFtStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2Coinbase {
    Zero,
    One(Option<StagedLedgerDiffDiffFtStableV1>),
}

/// **OCaml name**: `Staged_ledger_diff__Diff.Ft.Stable.V1`
///
/// Gid: `922`
/// Location: [src/lib/staged_ledger_diff/diff.ml:67:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/diff.ml#L67)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffFtStableV1(pub MinaBaseCoinbaseFeeTransferMakeStrStableV1);

/// **OCaml name**: `Staged_ledger_diff__Diff.Pre_diff_with_at_most_two_coinbase.Stable.V2`
///
/// Gid: `925`
/// Location: [src/lib/staged_ledger_diff/diff.ml:147:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/diff.ml#L147)
///
///
/// Gid: `923`
/// Location: [src/lib/staged_ledger_diff/diff.ml:83:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/diff.ml#L83)
/// Args: TransactionSnarkWorkTStableV2 , StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2 {
    pub completed_works: Vec<TransactionSnarkWorkTStableV2>,
    pub commands: Vec<StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B>,
    pub coinbase: StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2Coinbase,
    pub internal_command_statuses: Vec<MinaBaseTransactionStatusStableV2>,
}

/// **OCaml name**: `Staged_ledger_diff__Diff.Pre_diff_with_at_most_one_coinbase.Stable.V2`
///
/// Gid: `926`
/// Location: [src/lib/staged_ledger_diff/diff.ml:166:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/diff.ml#L166)
///
///
/// Gid: `924`
/// Location: [src/lib/staged_ledger_diff/diff.ml:115:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/diff.ml#L115)
/// Args: TransactionSnarkWorkTStableV2 , StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2 {
    pub completed_works: Vec<TransactionSnarkWorkTStableV2>,
    pub commands: Vec<StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B>,
    pub coinbase: StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2Coinbase,
    pub internal_command_statuses: Vec<MinaBaseTransactionStatusStableV2>,
}

/// **OCaml name**: `Staged_ledger_diff__Diff.Diff.Stable.V2`
///
/// Gid: `927`
/// Location: [src/lib/staged_ledger_diff/diff.ml:185:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/diff.ml#L185)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffDiffStableV2(
    pub StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2,
    pub Option<StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2>,
);

/// **OCaml name**: `Staged_ledger_diff__Diff.Stable.V2`
///
/// Gid: `928`
/// Location: [src/lib/staged_ledger_diff/diff.ml:202:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/diff.ml#L202)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffStableV2 {
    pub diff: StagedLedgerDiffDiffDiffStableV2,
}

/// **OCaml name**: `Staged_ledger_diff__Body.Stable.V1`
///
/// Gid: `929`
/// Location: [src/lib/staged_ledger_diff/body.ml:12:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/staged_ledger_diff/body.ml#L12)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffBodyStableV1 {
    pub staged_ledger_diff: StagedLedgerDiffDiffStableV2,
}

/// **OCaml name**: `Parallel_scan.Sequence_number.Stable.V1`
///
/// Gid: `934`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:22:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L22)
///
///
/// Gid: `163`
/// Location: [src/std_internal.ml:119:2](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/std_internal.ml#L119)
///
///
/// Gid: `113`
/// Location: [src/int.ml:19:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/int.ml#L19)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ParallelScanSequenceNumberStableV1(pub crate::number::Int32);

/// **OCaml name**: `Parallel_scan.Job_status.Stable.V1`
///
/// Gid: `935`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:35:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L35)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum ParallelScanJobStatusStableV1 {
    Todo,
    Done,
}

/// **OCaml name**: `Parallel_scan.Weight.Stable.V1`
///
/// Gid: `936`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:53:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L53)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ParallelScanWeightStableV1 {
    pub base: crate::number::Int32,
    pub merge: crate::number::Int32,
}

/// Derived name: `Transaction_snark_scan_state.Stable.V2#trees#a#base_t#1#Full`
///
/// Gid: `937`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:68:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L68)
/// Args: TransactionSnarkScanStateTransactionWithWitnessStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateStableV2TreesABaseT1Full {
    pub job: TransactionSnarkScanStateTransactionWithWitnessStableV2,
    pub seq_no: ParallelScanSequenceNumberStableV1,
    pub status: ParallelScanJobStatusStableV1,
}

/// Derived name: `Transaction_snark_scan_state.Stable.V2#trees#a#base_t#1`
///
/// Gid: `938`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:84:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L84)
/// Args: TransactionSnarkScanStateTransactionWithWitnessStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum TransactionSnarkScanStateStableV2TreesABaseT1 {
    Empty,
    Full(Box<TransactionSnarkScanStateStableV2TreesABaseT1Full>),
}

/// Derived name: `Transaction_snark_scan_state.Stable.V2#trees#a#merge_t#1#Full`
///
/// Gid: `940`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:112:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L112)
/// Args: TransactionSnarkScanStateLedgerProofWithSokMessageStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateStableV2TreesAMergeT1Full {
    pub left: TransactionSnarkScanStateLedgerProofWithSokMessageStableV2,
    pub right: TransactionSnarkScanStateLedgerProofWithSokMessageStableV2,
    pub seq_no: ParallelScanSequenceNumberStableV1,
    pub status: ParallelScanJobStatusStableV1,
}

/// Derived name: `Transaction_snark_scan_state.Stable.V2#trees#a#merge_t#1`
///
/// Gid: `941`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:130:8](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L130)
/// Args: TransactionSnarkScanStateLedgerProofWithSokMessageStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum TransactionSnarkScanStateStableV2TreesAMergeT1 {
    Empty,
    Part(Box<TransactionSnarkScanStateLedgerProofWithSokMessageStableV2>),
    Full(Box<TransactionSnarkScanStateStableV2TreesAMergeT1Full>),
}

/// **OCaml name**: `Transaction_snark_scan_state.Transaction_with_witness.Stable.V2`
///
/// Gid: `949`
/// Location: [src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml:40:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml#L40)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateTransactionWithWitnessStableV2 {
    pub transaction_with_info: MinaTransactionLogicTransactionAppliedStableV2,
    pub state_hash: (DataHashLibStateHashStableV1, MinaBaseStateBodyHashStableV1),
    pub statement: TransactionSnarkStatementStableV2,
    pub init_stack: TransactionSnarkPendingCoinbaseStackStateInitStackStableV1,
    pub ledger_witness: MinaBaseSparseLedgerBaseStableV2,
}

/// **OCaml name**: `Transaction_snark_scan_state.Ledger_proof_with_sok_message.Stable.V2`
///
/// Gid: `950`
/// Location: [src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml:61:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml#L61)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateLedgerProofWithSokMessageStableV2(
    pub LedgerProofProdStableV2,
    pub MinaBaseSokMessageStableV1,
);

/// **OCaml name**: `Protocol_version.Stable.V1`
///
/// Gid: `970`
/// Location: [src/lib/protocol_version/protocol_version.ml:8:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/protocol_version/protocol_version.ml#L8)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ProtocolVersionStableV1 {
    pub major: crate::number::Int32,
    pub minor: crate::number::Int32,
    pub patch: crate::number::Int32,
}

/// **OCaml name**: `Mina_block__Header.Stable.V2`
///
/// Gid: `973`
/// Location: [src/lib/mina_block/header.ml:14:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/mina_block/header.ml#L14)
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

/// Derived name: `Network_pool__Snark_pool.Diff_versioned.Stable.V2#Add_solved_work#1`
///
/// Gid: `1000`
/// Location: [src/lib/network_pool/priced_proof.ml:9:4](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/network_pool/priced_proof.ml#L9)
/// Args: TransactionSnarkWorkTStableV2Proofs
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NetworkPoolSnarkPoolDiffVersionedStableV2AddSolvedWork1 {
    pub proof: TransactionSnarkWorkTStableV2Proofs,
    pub fee: MinaBaseFeeWithProverStableV1,
}
