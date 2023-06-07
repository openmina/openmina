use binprot_derive::{BinProtRead, BinProtWrite};
use derive_more::Deref;
use serde::{Deserialize, Serialize};

use crate::pseq::PaddedSeq;

use super::manual::*;

/// **OCaml name**: `Mina_block__Block.Stable.V2`
///
/// Gid: `1048`
/// Location: [src/lib/mina_block/block.ml:8:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_block/block.ml#L8)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBlockBlockStableV2 {
    pub header: MinaBlockHeaderStableV2,
    pub body: StagedLedgerDiffBodyStableV1,
}

/// **OCaml name**: `Network_pool__Transaction_pool.Diff_versioned.Stable.V2`
///
/// Gid: `1067`
/// Location: [src/lib/network_pool/transaction_pool.ml:47:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/network_pool/transaction_pool.ml#L47)
///
///
/// Gid: `167`
/// Location: [src/std_internal.ml:131:2](https://github.com/Minaprotocol/mina/blob/32a9161/src/std_internal.ml#L131)
/// Args: MinaBaseUserCommandStableV2
///
///
/// Gid: `50`
/// Location: [src/list0.ml:6:0](https://github.com/Minaprotocol/mina/blob/32a9161/src/list0.ml#L6)
/// Args: MinaBaseUserCommandStableV2
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NetworkPoolTransactionPoolDiffVersionedStableV2(pub Vec<MinaBaseUserCommandStableV2>);

/// **OCaml name**: `Network_pool__Snark_pool.Diff_versioned.Stable.V2`
///
/// Gid: `1073`
/// Location: [src/lib/network_pool/snark_pool.ml:735:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/network_pool/snark_pool.ml#L735)
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
/// Gid: `844`
/// Location: [src/lib/mina_base/sparse_ledger_base.ml:8:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/sparse_ledger_base.ml#L8)
///
///
/// Gid: `626`
/// Location: [src/lib/sparse_ledger_lib/sparse_ledger.ml:38:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/sparse_ledger_lib/sparse_ledger.ml#L38)
/// Args: LedgerHash , MinaBaseAccountIdStableV2 , MinaBaseAccountBinableArgStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSparseLedgerBaseStableV2 {
    pub indexes: Vec<(MinaBaseAccountIdStableV2, crate::number::Int32)>,
    pub depth: crate::number::Int32,
    pub tree: MinaBaseSparseLedgerBaseStableV2Tree,
}

/// **OCaml name**: `Mina_base__Account.Binable_arg.Stable.V2`
///
/// Gid: `742`
/// Location: [src/lib/mina_base/account.ml:284:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account.ml#L284)
///
///
/// Gid: `740`
/// Location: [src/lib/mina_base/account.ml:229:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account.ml#L229)
/// Args: NonZeroCurvePoint , TokenIdKeyHash , MinaBaseZkappAccountZkappUriStableV1 , CurrencyBalanceStableV1 , UnsignedExtendedUInt32StableV1 , MinaBaseReceiptChainHashStableV1 , Option < NonZeroCurvePoint > , StateHash , MinaBaseAccountTimingStableV1 , MinaBasePermissionsStableV2 , Option < MinaBaseZkappAccountStableV2 >
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountBinableArgStableV2 {
    pub public_key: NonZeroCurvePoint,
    pub token_id: TokenIdKeyHash,
    pub token_symbol: MinaBaseZkappAccountZkappUriStableV1,
    pub balance: CurrencyBalanceStableV1,
    pub nonce: UnsignedExtendedUInt32StableV1,
    pub receipt_chain_hash: MinaBaseReceiptChainHashStableV1,
    pub delegate: Option<NonZeroCurvePoint>,
    pub voting_for: StateHash,
    pub timing: MinaBaseAccountTimingStableV1,
    pub permissions: MinaBasePermissionsStableV2,
    pub zkapp: Option<MinaBaseZkappAccountStableV2>,
}

/// **OCaml name**: `Network_peer__Peer.Stable.V1`
///
/// Gid: `852`
/// Location: [src/lib/network_peer/peer.ml:28:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/network_peer/peer.ml#L28)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NetworkPeerPeerStableV1 {
    pub host: crate::string::ByteString,
    pub libp2p_port: crate::number::Int32,
    pub peer_id: NetworkPeerPeerIdStableV1,
}

/// **OCaml name**: `Transaction_snark_scan_state.Stable.V2`
///
/// Gid: `1046`
/// Location: [src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml:160:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml#L160)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateStableV2 {
    pub scan_state: TransactionSnarkScanStateStableV2ScanState,
    pub previous_incomplete_zkapp_updates: (
        Vec<TransactionSnarkScanStateTransactionWithWitnessStableV2>,
        TransactionSnarkScanStateStableV2PreviousIncompleteZkappUpdates1,
    ),
}

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.Stable.V2`
///
/// Gid: `836`
/// Location: [src/lib/mina_base/pending_coinbase.ml:1279:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/pending_coinbase.ml#L1279)
///
///
/// Gid: `835`
/// Location: [src/lib/mina_base/pending_coinbase.ml:1267:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/pending_coinbase.ml#L1267)
/// Args: MinaBasePendingCoinbaseMerkleTreeVersionedStableV2 , MinaBasePendingCoinbaseStackIdStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStableV2 {
    pub tree: MinaBasePendingCoinbaseMerkleTreeVersionedStableV2,
    pub pos_list: Vec<MinaBasePendingCoinbaseStackIdStableV1>,
    pub new_pos: MinaBasePendingCoinbaseStackIdStableV1,
}

/// **OCaml name**: `Mina_state__Protocol_state.Make_str.Value.Stable.V2`
///
/// Gid: `956`
/// Location: [src/lib/mina_state/protocol_state.ml:205:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_state/protocol_state.ml#L205)
///
///
/// Gid: `952`
/// Location: [src/lib/mina_state/protocol_state.ml:38:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_state/protocol_state.ml#L38)
/// Args: StateHash , MinaStateProtocolStateBodyValueStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateProtocolStateValueStableV2 {
    pub previous_state_hash: StateHash,
    pub body: MinaStateProtocolStateBodyValueStableV2,
}

/// **OCaml name**: `Mina_ledger__Sync_ledger.Query.Stable.V1`
///
/// Gid: `897`
/// Location: [src/lib/mina_ledger/sync_ledger.ml:83:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_ledger/sync_ledger.ml#L83)
///
///
/// Gid: `886`
/// Location: [src/lib/syncable_ledger/syncable_ledger.ml:17:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/syncable_ledger/syncable_ledger.ml#L17)
/// Args: MerkleAddressBinableArgStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaLedgerSyncLedgerQueryStableV1 {
    WhatChildHashes(MerkleAddressBinableArgStableV1),
    WhatContents(MerkleAddressBinableArgStableV1),
    NumAccounts,
}

/// **OCaml name**: `Mina_ledger__Sync_ledger.Answer.Stable.V2`
///
/// Gid: `896`
/// Location: [src/lib/mina_ledger/sync_ledger.ml:58:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_ledger/sync_ledger.ml#L58)
///
///
/// Gid: `887`
/// Location: [src/lib/syncable_ledger/syncable_ledger.ml:35:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/syncable_ledger/syncable_ledger.ml#L35)
/// Args: LedgerHash , MinaBaseAccountBinableArgStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaLedgerSyncLedgerAnswerStableV2 {
    ChildHashesAre(LedgerHash, LedgerHash),
    ContentsAre(Vec<MinaBaseAccountBinableArgStableV2>),
    NumAccounts(crate::number::Int32, LedgerHash),
}

/// **OCaml name**: `Consensus__Proof_of_stake.Make_str.Data.Consensus_state.Value.Stable.V1`
///
/// Gid: `936`
/// Location: [src/lib/consensus/proof_of_stake.ml:1785:12](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/consensus/proof_of_stake.ml#L1785)
///
///
/// Gid: `935`
/// Location: [src/lib/consensus/proof_of_stake.ml:1740:12](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/consensus/proof_of_stake.ml#L1740)
/// Args: UnsignedExtendedUInt32StableV1 , ConsensusVrfOutputTruncatedStableV1 , CurrencyAmountStableV1 , ConsensusGlobalSlotStableV1 , UnsignedExtendedUInt32StableV1 , ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1 , ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1 , bool , NonZeroCurvePoint
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataConsensusStateValueStableV1 {
    pub blockchain_length: UnsignedExtendedUInt32StableV1,
    pub epoch_count: UnsignedExtendedUInt32StableV1,
    pub min_window_density: UnsignedExtendedUInt32StableV1,
    pub sub_window_densities: Vec<UnsignedExtendedUInt32StableV1>,
    pub last_vrf_output: ConsensusVrfOutputTruncatedStableV1,
    pub total_currency: CurrencyAmountStableV1,
    pub curr_global_slot: ConsensusGlobalSlotStableV1,
    pub global_slot_since_genesis: UnsignedExtendedUInt32StableV1,
    pub staking_epoch_data: ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
    pub next_epoch_data: ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
    pub has_ancestor_in_same_checkpoint_window: bool,
    pub block_stake_winner: NonZeroCurvePoint,
    pub block_creator: NonZeroCurvePoint,
    pub coinbase_receiver: NonZeroCurvePoint,
    pub supercharge_coinbase: bool,
}

/// **OCaml name**: `Sync_status.T.Stable.V1`
///
/// Gid: `1105`
/// Location: [src/lib/sync_status/sync_status.ml:54:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/sync_status/sync_status.ml#L54)
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
/// Gid: `884`
/// Location: [src/lib/trust_system/peer_status.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/trust_system/peer_status.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TrustSystemPeerStatusStableV1 {
    pub trust: crate::number::Float64,
    pub banned: TrustSystemBannedStatusStableV1,
}

/// **OCaml name**: `Blockchain_snark__Blockchain.Stable.V2`
///
/// Gid: `1005`
/// Location: [src/lib/blockchain_snark/blockchain.ml:8:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/blockchain_snark/blockchain.ml#L8)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct BlockchainSnarkBlockchainStableV2 {
    pub state: MinaStateProtocolStateValueStableV2,
    pub proof: MinaBaseProofStableV2,
}

/// **OCaml name**: `Mina_base__Zkapp_account.Zkapp_uri.Stable.V1`
///
/// Gid: `73`
/// Location: [src/string.ml:14:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/string.ml#L14)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappAccountZkappUriStableV1(pub crate::string::ByteString);

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.statement.fp`
///
/// Gid: `458`
/// Location: [src/lib/pickles_types/shifted_value.ml:98:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_types/shifted_value.ml#L98)
/// Args: crate :: bigint :: BigInt
#[derive(Clone, Debug, PartialEq, BinProtRead, BinProtWrite)]
pub enum PicklesProofProofsVerified2ReprStableV2StatementFp {
    ShiftedValue(crate::bigint::BigInt),
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.statement.plonk.feature_flags`
///
/// Gid: `461`
/// Location: [src/lib/pickles_types/plonk_types.ml:184:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_types/plonk_types.ml#L184)
/// Args: bool
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2StatementPlonkFeatureFlags {
    pub range_check0: bool,
    pub range_check1: bool,
    pub foreign_field_add: bool,
    pub foreign_field_mul: bool,
    pub xor: bool,
    pub rot: bool,
    pub lookup: bool,
    pub runtime_tables: bool,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.prev_evals.evals.evals.lookup.a`
///
/// Gid: `462`
/// Location: [src/lib/pickles_types/plonk_types.ml:354:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_types/plonk_types.ml#L354)
/// Args: (Vec < crate :: bigint :: BigInt > , Vec < crate :: bigint :: BigInt > ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvalsLookupA {
    pub sorted: Vec<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>)>,
    pub aggreg: (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
    pub table: (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
    pub runtime: Option<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>)>,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.prev_evals.evals.evals`
///
/// Gid: `463`
/// Location: [src/lib/pickles_types/plonk_types.ml:424:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_types/plonk_types.ml#L424)
/// Args: (Vec < crate :: bigint :: BigInt > , Vec < crate :: bigint :: BigInt > ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals {
    pub w: PaddedSeq<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>), 15>,
    pub coefficients: PaddedSeq<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>), 15>,
    pub z: (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
    pub s: PaddedSeq<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>), 6>,
    pub generic_selector: (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
    pub poseidon_selector: (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
    pub lookup: Option<PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvalsLookupA>,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.prev_evals.evals`
///
/// Gid: `464`
/// Location: [src/lib/pickles_types/plonk_types.ml:630:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_types/plonk_types.ml#L630)
/// Args: (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,) , (Vec < crate :: bigint :: BigInt > , Vec < crate :: bigint :: BigInt > ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PrevEvalsEvals {
    pub public_input: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub evals: PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.prev_evals`
///
/// Gid: `465`
/// Location: [src/lib/pickles_types/plonk_types.ml:665:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_types/plonk_types.ml#L665)
/// Args: crate :: bigint :: BigInt , Vec < crate :: bigint :: BigInt >
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PrevEvals {
    pub evals: PicklesProofProofsVerified2ReprStableV2PrevEvalsEvals,
    pub ft_eval1: crate::bigint::BigInt,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.proof.openings.proof`
///
/// Gid: `466`
/// Location: [src/lib/pickles_types/plonk_types.ml:714:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_types/plonk_types.ml#L714)
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

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.proof.openings`
///
/// Gid: `467`
/// Location: [src/lib/pickles_types/plonk_types.ml:736:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_types/plonk_types.ml#L736)
/// Args: (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,) , crate :: bigint :: BigInt , Vec < crate :: bigint :: BigInt >
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2ProofOpenings {
    pub proof: PicklesProofProofsVerified2ReprStableV2ProofOpeningsProof,
    pub evals: PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals,
    pub ft_eval1: crate::bigint::BigInt,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.proof.messages.lookup.a`
///
/// Gid: `470`
/// Location: [src/lib/pickles_types/plonk_types.ml:800:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_types/plonk_types.ml#L800)
/// Args: Vec < (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,) >
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2ProofMessagesLookupA {
    pub sorted: Vec<Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>>,
    pub aggreg: Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>,
    pub runtime: Option<Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>>,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.proof.messages`
///
/// Gid: `471`
/// Location: [src/lib/pickles_types/plonk_types.ml:839:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_types/plonk_types.ml#L839)
/// Args: (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2ProofMessages {
    pub w_comm: PaddedSeq<Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>, 15>,
    pub z_comm: Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>,
    pub t_comm: Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>,
    pub lookup: Option<PicklesProofProofsVerified2ReprStableV2ProofMessagesLookupA>,
}

/// Derived name: `Mina_base__Verification_key_wire.Stable.V1.wrap_index`
///
/// Gid: `474`
/// Location: [src/lib/pickles_types/plonk_verification_key_evals.ml:7:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_types/plonk_verification_key_evals.ml#L7)
/// Args: (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseVerificationKeyWireStableV1WrapIndex {
    pub sigma_comm: PaddedSeq<(crate::bigint::BigInt, crate::bigint::BigInt), 7>,
    pub coefficients_comm: PaddedSeq<(crate::bigint::BigInt, crate::bigint::BigInt), 15>,
    pub generic_comm: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub psm_comm: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub complete_add_comm: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub mul_comm: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub emul_comm: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub endomul_scalar_comm: (crate::bigint::BigInt, crate::bigint::BigInt),
}

/// Derived name: `Pickles__Reduced_messages_for_next_proof_over_same_field.Wrap.Challenges_vector.Stable.V2.a.challenge`
///
/// Gid: `478`
/// Location: [src/lib/crypto/kimchi_backend/common/scalar_challenge.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/crypto/kimchi_backend/common/scalar_challenge.ml#L6)
/// Args: PaddedSeq < LimbVectorConstantHex64StableV1 , 2 >
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge {
    pub inner: PaddedSeq<LimbVectorConstantHex64StableV1, 2>,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.proof`
///
/// Gid: `497`
/// Location: [src/lib/crypto/kimchi_backend/common/plonk_dlog_proof.ml:160:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/crypto/kimchi_backend/common/plonk_dlog_proof.ml#L160)
///
///
/// Gid: `472`
/// Location: [src/lib/pickles_types/plonk_types.ml:888:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_types/plonk_types.ml#L888)
/// Args: (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,) , crate :: bigint :: BigInt , Vec < crate :: bigint :: BigInt >
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2Proof {
    pub messages: PicklesProofProofsVerified2ReprStableV2ProofMessages,
    pub openings: PicklesProofProofsVerified2ReprStableV2ProofOpenings,
}

/// **OCaml name**: `Blake2.Make.Stable.V1`
///
/// Gid: `500`
/// Location: [src/binable0.ml:120:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/binable0.ml#L120)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct Blake2MakeStableV1(pub crate::string::ByteString);

/// Derived name: `Transaction_snark_work.T.Stable.V2.proofs`
///
/// Gid: `503`
/// Location: [src/lib/one_or_two/one_or_two.ml:7:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/one_or_two/one_or_two.ml#L7)
/// Args: LedgerProofProdStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[polymorphic_variant]
pub enum TransactionSnarkWorkTStableV2Proofs {
    One(LedgerProofProdStableV2),
    Two((LedgerProofProdStableV2, LedgerProofProdStableV2)),
}

/// **OCaml name**: `Pickles_base__Proofs_verified.Stable.V1`
///
/// Gid: `509`
/// Location: [src/lib/pickles_base/proofs_verified.ml:8:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_base/proofs_verified.ml#L8)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum PicklesBaseProofsVerifiedStableV1 {
    N0,
    N1,
    N2,
}

/// **OCaml name**: `Limb_vector__Constant.Hex64.Stable.V1`
///
/// Gid: `518`
/// Location: [src/lib/pickles/limb_vector/constant.ml:60:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/limb_vector/constant.ml#L60)
///
///
/// Gid: `125`
/// Location: [src/int64.ml:6:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/int64.ml#L6)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct LimbVectorConstantHex64StableV1(pub crate::number::Int64);

/// **OCaml name**: `Composition_types__Branch_data.Make_str.Domain_log2.Stable.V1`
///
/// Gid: `519`
/// Location: [src/lib/pickles/composition_types/branch_data.ml:24:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/composition_types/branch_data.ml#L24)
///
///
/// Gid: `161`
/// Location: [src/std_internal.ml:113:2](https://github.com/Minaprotocol/mina/blob/32a9161/src/std_internal.ml#L113)
///
///
/// Gid: `89`
/// Location: [src/char.ml:8:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/char.ml#L8)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CompositionTypesBranchDataDomainLog2StableV1(pub crate::char::Char);

/// **OCaml name**: `Composition_types__Branch_data.Make_str.Stable.V1`
///
/// Gid: `520`
/// Location: [src/lib/pickles/composition_types/branch_data.ml:51:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/composition_types/branch_data.ml#L51)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CompositionTypesBranchDataStableV1 {
    pub proofs_verified: PicklesBaseProofsVerifiedStableV1,
    pub domain_log2: CompositionTypesBranchDataDomainLog2StableV1,
}

/// Derived name: `Pickles__Reduced_messages_for_next_proof_over_same_field.Wrap.Challenges_vector.Stable.V2.a`
///
/// Gid: `521`
/// Location: [src/lib/pickles/composition_types/bulletproof_challenge.ml:4:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/composition_types/bulletproof_challenge.ml#L4)
/// Args: PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A {
    pub prechallenge:
        PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge,
}

/// **OCaml name**: `Composition_types__Digest.Constant.Stable.V1`
///
/// Gid: `522`
/// Location: [src/lib/pickles/composition_types/digest.ml:13:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/composition_types/digest.ml#L13)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CompositionTypesDigestConstantStableV1(
    pub PaddedSeq<LimbVectorConstantHex64StableV1, 4>,
);

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.statement.plonk`
///
/// Gid: `523`
/// Location: [src/lib/pickles/composition_types/composition_types.ml:45:14](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/composition_types/composition_types.ml#L45)
/// Args: PaddedSeq < LimbVectorConstantHex64StableV1 , 2 > , PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge , bool
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2StatementPlonk {
    pub alpha:
        PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge,
    pub beta: PaddedSeq<LimbVectorConstantHex64StableV1, 2>,
    pub gamma: PaddedSeq<LimbVectorConstantHex64StableV1, 2>,
    pub zeta: PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge,
    pub joint_combiner: Option<
        PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge,
    >,
    pub feature_flags: PicklesProofProofsVerified2ReprStableV2StatementPlonkFeatureFlags,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.statement.proof_state.deferred_values`
///
/// Gid: `524`
/// Location: [src/lib/pickles/composition_types/composition_types.ml:404:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/composition_types/composition_types.ml#L404)
/// Args: PicklesProofProofsVerified2ReprStableV2StatementPlonk , PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge , PicklesProofProofsVerified2ReprStableV2StatementFp , PaddedSeq < PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , 16 > , CompositionTypesBranchDataStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValues {
    pub plonk: PicklesProofProofsVerified2ReprStableV2StatementPlonk,
    pub combined_inner_product: PicklesProofProofsVerified2ReprStableV2StatementFp,
    pub b: PicklesProofProofsVerified2ReprStableV2StatementFp,
    pub xi: PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge,
    pub bulletproof_challenges:
        PaddedSeq<PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A, 16>,
    pub branch_data: CompositionTypesBranchDataStableV1,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.messages_for_next_wrap_proof`
///
/// Gid: `525`
/// Location: [src/lib/pickles/composition_types/composition_types.ml:550:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/composition_types/composition_types.ml#L550)
/// Args: (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,) , PaddedSeq < PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2 , 2 >
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof {
    pub challenge_polynomial_commitment: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub old_bulletproof_challenges:
        PaddedSeq<PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2, 2>,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.statement.proof_state`
///
/// Gid: `526`
/// Location: [src/lib/pickles/composition_types/composition_types.ml:583:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/composition_types/composition_types.ml#L583)
/// Args: PicklesProofProofsVerified2ReprStableV2StatementPlonk , PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge , PicklesProofProofsVerified2ReprStableV2StatementFp , PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof , CompositionTypesDigestConstantStableV1 , PaddedSeq < PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , 16 > , CompositionTypesBranchDataStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2StatementProofState {
    pub deferred_values: PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValues,
    pub sponge_digest_before_evaluations: CompositionTypesDigestConstantStableV1,
    pub messages_for_next_wrap_proof:
        PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.statement`
///
/// Gid: `528`
/// Location: [src/lib/pickles/composition_types/composition_types.ml:840:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/composition_types/composition_types.ml#L840)
/// Args: PaddedSeq < LimbVectorConstantHex64StableV1 , 2 > , PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge , PicklesProofProofsVerified2ReprStableV2StatementFp , bool , PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof , CompositionTypesDigestConstantStableV1 , PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof , PaddedSeq < PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , 16 > , CompositionTypesBranchDataStableV1
///
///
/// Gid: `527`
/// Location: [src/lib/pickles/composition_types/composition_types.ml:803:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/composition_types/composition_types.ml#L803)
/// Args: PicklesProofProofsVerified2ReprStableV2StatementPlonk , PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge , PicklesProofProofsVerified2ReprStableV2StatementFp , PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof , CompositionTypesDigestConstantStableV1 , PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof , PaddedSeq < PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , 16 > , CompositionTypesBranchDataStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2Statement {
    pub proof_state: PicklesProofProofsVerified2ReprStableV2StatementProofState,
    pub messages_for_next_step_proof:
        PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.messages_for_next_step_proof`
///
/// Gid: `531`
/// Location: [src/lib/pickles/reduced_messages_for_next_proof_over_same_field.ml:16:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/reduced_messages_for_next_proof_over_same_field.ml#L16)
/// Args: () , Vec < (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,) > , Vec < PaddedSeq < PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , 16 > >
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
/// Gid: `532`
/// Location: [src/lib/pickles/reduced_messages_for_next_proof_over_same_field.ml:57:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/reduced_messages_for_next_proof_over_same_field.ml#L57)
///
///
/// Gid: `484`
/// Location: [src/lib/crypto/kimchi_backend/pasta/basic/kimchi_pasta_basic.ml:32:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/crypto/kimchi_backend/pasta/basic/kimchi_pasta_basic.ml#L32)
/// Args: PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2(
    pub PaddedSeq<PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A, 15>,
);

/// **OCaml name**: `Mina_base__Verification_key_wire.Stable.V1`
///
/// Gid: `533`
/// Location: [src/lib/pickles/side_loaded_verification_key.ml:170:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/side_loaded_verification_key.ml#L170)
///
///
/// Gid: `514`
/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:132:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_base/side_loaded_verification_key.ml#L132)
/// Args: (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseVerificationKeyWireStableV1 {
    pub max_proofs_verified: PicklesBaseProofsVerifiedStableV1,
    pub actual_wrap_domain_size: PicklesBaseProofsVerifiedStableV1,
    pub wrap_index: MinaBaseVerificationKeyWireStableV1WrapIndex,
}

/// **OCaml name**: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2`
///
/// Gid: `536`
/// Location: [src/lib/pickles/proof.ml:343:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/proof.ml#L343)
///
///
/// Gid: `535`
/// Location: [src/lib/pickles/proof.ml:47:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/proof.ml#L47)
/// Args: PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof , PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2 {
    pub statement: PicklesProofProofsVerified2ReprStableV2Statement,
    pub prev_evals: PicklesProofProofsVerified2ReprStableV2PrevEvals,
    pub proof: PicklesProofProofsVerified2ReprStableV2Proof,
}

/// **OCaml name**: `Pickles__Proof.Proofs_verified_max.Stable.V2`
///
/// Gid: `537`
/// Location: [src/lib/pickles/proof.ml:412:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/proof.ml#L412)
///
///
/// Gid: `535`
/// Location: [src/lib/pickles/proof.ml:47:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/proof.ml#L47)
/// Args: PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof , PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerifiedMaxStableV2 {
    pub statement: PicklesProofProofsVerified2ReprStableV2Statement,
    pub prev_evals: PicklesProofProofsVerified2ReprStableV2PrevEvals,
    pub proof: PicklesProofProofsVerified2ReprStableV2Proof,
}

/// **OCaml name**: `Non_zero_curve_point.Uncompressed.Stable.V1`
///
/// Gid: `547`
/// Location: [src/lib/non_zero_curve_point/non_zero_curve_point.ml:46:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/non_zero_curve_point/non_zero_curve_point.ml#L46)
///
///
/// Gid: `541`
/// Location: [src/lib/non_zero_curve_point/compressed_poly.ml:13:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/non_zero_curve_point/compressed_poly.ml#L13)
/// Args: crate :: bigint :: BigInt , bool
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, BinProtRead, BinProtWrite,
)]
pub struct NonZeroCurvePointUncompressedStableV1 {
    pub x: crate::bigint::BigInt,
    pub is_odd: bool,
}

/// **OCaml name**: `Unsigned_extended.UInt64.Int64_for_version_tags.Stable.V1`
///
/// Gid: `565`
/// Location: [src/lib/unsigned_extended/unsigned_extended.ml:81:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/unsigned_extended/unsigned_extended.ml#L81)
///
///
/// Gid: `125`
/// Location: [src/int64.ml:6:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/int64.ml#L6)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct UnsignedExtendedUInt64Int64ForVersionTagsStableV1(pub crate::number::Int64);

/// **OCaml name**: `Unsigned_extended.UInt32.Stable.V1`
///
/// Gid: `569`
/// Location: [src/lib/unsigned_extended/unsigned_extended.ml:156:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/unsigned_extended/unsigned_extended.ml#L156)
///
///
/// Gid: `119`
/// Location: [src/int32.ml:6:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/int32.ml#L6)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct UnsignedExtendedUInt32StableV1(pub crate::number::Int32);

/// **OCaml name**: `Mina_numbers__Nat.Make32.Stable.V1`
///
/// Gid: `573`
/// Location: [src/lib/mina_numbers/nat.ml:260:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_numbers/nat.ml#L260)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaNumbersNatMake32StableV1(pub UnsignedExtendedUInt32StableV1);

/// **OCaml name**: `Sgn.Stable.V1`
///
/// Gid: `601`
/// Location: [src/lib/sgn/sgn.ml:9:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/sgn/sgn.ml#L9)
#[derive(Clone, Debug, PartialEq, BinProtRead, BinProtWrite)]
pub enum SgnStableV1 {
    Pos,
    Neg,
}

/// Derived name: `Mina_transaction_logic__Zkapp_command_logic.Local_state.Value.Stable.V1.signed_amount`
///
/// Gid: `602`
/// Location: [src/lib/currency/signed_poly.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/currency/signed_poly.ml#L6)
/// Args: CurrencyAmountStableV1 , SgnStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount {
    pub magnitude: CurrencyAmountStableV1,
    pub sgn: SgnStableV1,
}

/// Derived name: `Mina_base__Fee_excess.Stable.V1.fee`
///
/// Gid: `602`
/// Location: [src/lib/currency/signed_poly.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/currency/signed_poly.ml#L6)
/// Args: CurrencyFeeStableV1 , SgnStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeExcessStableV1Fee {
    pub magnitude: CurrencyFeeStableV1,
    pub sgn: SgnStableV1,
}

/// **OCaml name**: `Currency.Make_str.Fee.Stable.V1`
///
/// Gid: `603`
/// Location: [src/lib/currency/currency.ml:945:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/currency/currency.ml#L945)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CurrencyFeeStableV1(pub UnsignedExtendedUInt64Int64ForVersionTagsStableV1);

/// **OCaml name**: `Currency.Make_str.Amount.Make_str.Stable.V1`
///
/// Gid: `606`
/// Location: [src/lib/currency/currency.ml:1083:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/currency/currency.ml#L1083)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CurrencyAmountStableV1(pub UnsignedExtendedUInt64Int64ForVersionTagsStableV1);

/// **OCaml name**: `Currency.Make_str.Balance.Stable.V1`
///
/// Gid: `609`
/// Location: [src/lib/currency/currency.ml:1127:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/currency/currency.ml#L1127)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CurrencyBalanceStableV1(pub CurrencyAmountStableV1);

/// **OCaml name**: `Data_hash_lib__State_hash.Stable.V1`
///
/// Gid: `616`
/// Location: [src/lib/data_hash_lib/state_hash.ml:44:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/data_hash_lib/state_hash.ml#L44)
#[derive(
    Clone,
    Debug,
    Deref,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    BinProtRead,
    BinProtWrite,
)]
pub struct DataHashLibStateHashStableV1(pub crate::bigint::BigInt);

/// Derived name: `Mina_base__Sparse_ledger_base.Stable.V2.tree`
///
/// Gid: `625`
/// Location: [src/lib/sparse_ledger_lib/sparse_ledger.ml:9:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/sparse_ledger_lib/sparse_ledger.ml#L9)
/// Args: LedgerHash , MinaBaseAccountBinableArgStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSparseLedgerBaseStableV2Tree {
    Account(Box<MinaBaseAccountBinableArgStableV2>),
    Hash(LedgerHash),
    Node(
        LedgerHash,
        Box<MinaBaseSparseLedgerBaseStableV2Tree>,
        Box<MinaBaseSparseLedgerBaseStableV2Tree>,
    ),
}

/// Derived name: `Mina_base__Pending_coinbase.Make_str.Merkle_tree_versioned.Stable.V2.tree`
///
/// Gid: `625`
/// Location: [src/lib/sparse_ledger_lib/sparse_ledger.ml:9:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/sparse_ledger_lib/sparse_ledger.ml#L9)
/// Args: PendingCoinbaseHash , MinaBasePendingCoinbaseStackVersionedStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBasePendingCoinbaseMerkleTreeVersionedStableV2Tree {
    Account(MinaBasePendingCoinbaseStackVersionedStableV1),
    Hash(PendingCoinbaseHash),
    Node(
        PendingCoinbaseHash,
        Box<MinaBasePendingCoinbaseMerkleTreeVersionedStableV2Tree>,
        Box<MinaBasePendingCoinbaseMerkleTreeVersionedStableV2Tree>,
    ),
}

/// **OCaml name**: `Block_time.Make_str.Time.Stable.V1`
///
/// Gid: `627`
/// Location: [src/lib/block_time/block_time.ml:22:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/block_time/block_time.ml#L22)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct BlockTimeTimeStableV1(pub UnsignedExtendedUInt64Int64ForVersionTagsStableV1);

/// **OCaml name**: `Mina_base__Account_id.Make_str.Digest.Stable.V1`
///
/// Gid: `629`
/// Location: [src/lib/mina_base/account_id.ml:64:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_id.ml#L64)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountIdDigestStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Account_id.Make_str.Stable.V2`
///
/// Gid: `634`
/// Location: [src/lib/mina_base/account_id.ml:151:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_id.ml#L151)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountIdStableV2(pub NonZeroCurvePoint, pub MinaBaseAccountIdDigestStableV1);

/// **OCaml name**: `Mina_base__Account_timing.Stable.V1`
///
/// Gid: `640`
/// Location: [src/lib/mina_base/account_timing.ml:39:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_timing.ml#L39)
///
///
/// Gid: `639`
/// Location: [src/lib/mina_base/account_timing.ml:22:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_timing.ml#L22)
/// Args: UnsignedExtendedUInt32StableV1 , CurrencyBalanceStableV1 , CurrencyAmountStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountTimingStableV1 {
    Untimed,
    Timed {
        initial_minimum_balance: CurrencyBalanceStableV1,
        cliff_time: UnsignedExtendedUInt32StableV1,
        cliff_amount: CurrencyAmountStableV1,
        vesting_period: UnsignedExtendedUInt32StableV1,
        vesting_increment: CurrencyAmountStableV1,
    },
}

/// **OCaml name**: `Mina_base__Signature.Stable.V1`
///
/// Gid: `644`
/// Location: [src/lib/mina_base/signature.ml:23:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/signature.ml#L23)
///
///
/// Gid: `641`
/// Location: [src/lib/mina_base/signature.ml:12:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/signature.ml#L12)
/// Args: crate :: bigint :: BigInt , crate :: bigint :: BigInt
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignatureStableV1(pub crate::bigint::BigInt, pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Control.Stable.V2`
///
/// Gid: `648`
/// Location: [src/lib/mina_base/control.ml:11:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/control.ml#L11)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseControlStableV2 {
    Proof(Box<PicklesProofProofsVerifiedMaxStableV2>),
    Signature(MinaBaseSignatureStableV1),
    NoneGiven,
}

/// **OCaml name**: `Mina_base__Token_id.Stable.V2`
///
/// Gid: `652`
/// Location: [src/lib/mina_base/token_id.ml:8:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/token_id.ml#L8)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseTokenIdStableV2(pub MinaBaseAccountIdDigestStableV1);

/// **OCaml name**: `Mina_base__Fee_excess.Stable.V1`
///
/// Gid: `657`
/// Location: [src/lib/mina_base/fee_excess.ml:124:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/fee_excess.ml#L124)
///
///
/// Gid: `656`
/// Location: [src/lib/mina_base/fee_excess.ml:54:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/fee_excess.ml#L54)
/// Args: TokenIdKeyHash , MinaBaseFeeExcessStableV1Fee
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeExcessStableV1 {
    pub fee_token_l: TokenIdKeyHash,
    pub fee_excess_l: MinaBaseFeeExcessStableV1Fee,
    pub fee_token_r: TokenIdKeyHash,
    pub fee_excess_r: MinaBaseFeeExcessStableV1Fee,
}

/// **OCaml name**: `Mina_base__Payment_payload.Stable.V2`
///
/// Gid: `662`
/// Location: [src/lib/mina_base/payment_payload.ml:39:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/payment_payload.ml#L39)
///
///
/// Gid: `658`
/// Location: [src/lib/mina_base/payment_payload.ml:14:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/payment_payload.ml#L14)
/// Args: NonZeroCurvePoint , CurrencyAmountStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePaymentPayloadStableV2 {
    pub source_pk: NonZeroCurvePoint,
    pub receiver_pk: NonZeroCurvePoint,
    pub amount: CurrencyAmountStableV1,
}

/// **OCaml name**: `Mina_base__Ledger_hash0.Stable.V1`
///
/// Gid: `668`
/// Location: [src/lib/mina_base/ledger_hash0.ml:17:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/ledger_hash0.ml#L17)
#[derive(
    Clone,
    Debug,
    Deref,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    BinProtRead,
    BinProtWrite,
)]
pub struct MinaBaseLedgerHash0StableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Permissions.Auth_required.Stable.V2`
///
/// Gid: `671`
/// Location: [src/lib/mina_base/permissions.ml:53:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/permissions.ml#L53)
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
/// Gid: `673`
/// Location: [src/lib/mina_base/permissions.ml:381:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/permissions.ml#L381)
///
///
/// Gid: `672`
/// Location: [src/lib/mina_base/permissions.ml:345:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/permissions.ml#L345)
/// Args: MinaBasePermissionsAuthRequiredStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePermissionsStableV2 {
    pub edit_state: MinaBasePermissionsAuthRequiredStableV2,
    pub access: MinaBasePermissionsAuthRequiredStableV2,
    pub send: MinaBasePermissionsAuthRequiredStableV2,
    pub receive: MinaBasePermissionsAuthRequiredStableV2,
    pub set_delegate: MinaBasePermissionsAuthRequiredStableV2,
    pub set_permissions: MinaBasePermissionsAuthRequiredStableV2,
    pub set_verification_key: MinaBasePermissionsAuthRequiredStableV2,
    pub set_zkapp_uri: MinaBasePermissionsAuthRequiredStableV2,
    pub edit_action_state: MinaBasePermissionsAuthRequiredStableV2,
    pub set_token_symbol: MinaBasePermissionsAuthRequiredStableV2,
    pub increment_nonce: MinaBasePermissionsAuthRequiredStableV2,
    pub set_voting_for: MinaBasePermissionsAuthRequiredStableV2,
    pub set_timing: MinaBasePermissionsAuthRequiredStableV2,
}

/// **OCaml name**: `Mina_base__Signed_command_memo.Make_str.Stable.V1`
///
/// Gid: `674`
/// Location: [src/lib/mina_base/signed_command_memo.ml:21:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/signed_command_memo.ml#L21)
///
///
/// Gid: `170`
/// Location: [src/std_internal.ml:140:2](https://github.com/Minaprotocol/mina/blob/32a9161/src/std_internal.ml#L140)
///
///
/// Gid: `83`
/// Location: [src/string.ml:44:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/string.ml#L44)
#[derive(
    Clone, Debug, derive_more::Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite,
)]
pub struct MinaBaseSignedCommandMemoStableV1(pub crate::string::CharString);

/// **OCaml name**: `Mina_base__Stake_delegation.Stable.V1`
///
/// Gid: `677`
/// Location: [src/lib/mina_base/stake_delegation.ml:11:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/stake_delegation.ml#L11)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseStakeDelegationStableV1 {
    SetDelegate {
        delegator: NonZeroCurvePoint,
        new_delegate: NonZeroCurvePoint,
    },
}

/// **OCaml name**: `Mina_base__Transaction_status.Failure.Stable.V2`
///
/// Gid: `680`
/// Location: [src/lib/mina_base/transaction_status.ml:9:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/transaction_status.ml#L9)
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
    LocalSupplyIncreaseOverflow,
    GlobalSupplyIncreaseOverflow,
    SignedCommandOnZkappAccount,
    ZkappAccountNotPresent,
    UpdateNotPermittedBalance,
    UpdateNotPermittedAccess,
    UpdateNotPermittedTiming,
    UpdateNotPermittedDelegate,
    UpdateNotPermittedAppState,
    UpdateNotPermittedVerificationKey,
    UpdateNotPermittedActionState,
    UpdateNotPermittedZkappUri,
    UpdateNotPermittedTokenSymbol,
    UpdateNotPermittedPermissions,
    UpdateNotPermittedNonce,
    UpdateNotPermittedVotingFor,
    ZkappCommandReplayCheckFailed,
    FeePayerNonceMustIncrease,
    FeePayerMustBeSigned,
    AccountBalancePreconditionUnsatisfied,
    AccountNoncePreconditionUnsatisfied,
    AccountReceiptChainHashPreconditionUnsatisfied,
    AccountDelegatePreconditionUnsatisfied,
    AccountActionStatePreconditionUnsatisfied,
    AccountAppStatePreconditionUnsatisfied(crate::number::Int32),
    AccountProvedStatePreconditionUnsatisfied,
    AccountIsNewPreconditionUnsatisfied,
    ProtocolStatePreconditionUnsatisfied,
    UnexpectedVerificationKeyHash,
    ValidWhilePreconditionUnsatisfied,
    IncorrectNonce,
    InvalidFeeExcess,
    Cancelled,
}

/// **OCaml name**: `Mina_base__Transaction_status.Failure.Collection.Stable.V1`
///
/// Gid: `682`
/// Location: [src/lib/mina_base/transaction_status.ml:77:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/transaction_status.ml#L77)
///
///
/// Gid: `167`
/// Location: [src/std_internal.ml:131:2](https://github.com/Minaprotocol/mina/blob/32a9161/src/std_internal.ml#L131)
/// Args: Vec < MinaBaseTransactionStatusFailureStableV2 >
///
///
/// Gid: `50`
/// Location: [src/list0.ml:6:0](https://github.com/Minaprotocol/mina/blob/32a9161/src/list0.ml#L6)
/// Args: Vec < MinaBaseTransactionStatusFailureStableV2 >
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseTransactionStatusFailureCollectionStableV1(
    pub Vec<Vec<MinaBaseTransactionStatusFailureStableV2>>,
);

/// **OCaml name**: `Mina_base__Transaction_status.Stable.V2`
///
/// Gid: `683`
/// Location: [src/lib/mina_base/transaction_status.ml:476:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/transaction_status.ml#L476)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseTransactionStatusStableV2 {
    Applied,
    Failed(MinaBaseTransactionStatusFailureCollectionStableV1),
}

/// **OCaml name**: `Mina_base__Signed_command_payload.Common.Stable.V2`
///
/// Gid: `688`
/// Location: [src/lib/mina_base/signed_command_payload.ml:75:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/signed_command_payload.ml#L75)
///
///
/// Gid: `684`
/// Location: [src/lib/mina_base/signed_command_payload.ml:40:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/signed_command_payload.ml#L40)
/// Args: CurrencyFeeStableV1 , NonZeroCurvePoint , UnsignedExtendedUInt32StableV1 , UnsignedExtendedUInt32StableV1 , MinaBaseSignedCommandMemoStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandPayloadCommonStableV2 {
    pub fee: CurrencyFeeStableV1,
    pub fee_payer_pk: NonZeroCurvePoint,
    pub nonce: UnsignedExtendedUInt32StableV1,
    pub valid_until: UnsignedExtendedUInt32StableV1,
    pub memo: MinaBaseSignedCommandMemoStableV1,
}

/// **OCaml name**: `Mina_base__Signed_command_payload.Body.Stable.V2`
///
/// Gid: `692`
/// Location: [src/lib/mina_base/signed_command_payload.ml:187:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/signed_command_payload.ml#L187)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSignedCommandPayloadBodyStableV2 {
    Payment(MinaBasePaymentPayloadStableV2),
    StakeDelegation(MinaBaseStakeDelegationStableV1),
}

/// **OCaml name**: `Mina_base__Signed_command_payload.Stable.V2`
///
/// Gid: `699`
/// Location: [src/lib/mina_base/signed_command_payload.ml:288:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/signed_command_payload.ml#L288)
///
///
/// Gid: `696`
/// Location: [src/lib/mina_base/signed_command_payload.ml:270:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/signed_command_payload.ml#L270)
/// Args: MinaBaseSignedCommandPayloadCommonStableV2 , MinaBaseSignedCommandPayloadBodyStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandPayloadStableV2 {
    pub common: MinaBaseSignedCommandPayloadCommonStableV2,
    pub body: MinaBaseSignedCommandPayloadBodyStableV2,
}

/// **OCaml name**: `Mina_base__Signed_command.Make_str.Stable.V2`
///
/// Gid: `706`
/// Location: [src/lib/mina_base/signed_command.ml:52:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/signed_command.ml#L52)
///
///
/// Gid: `703`
/// Location: [src/lib/mina_base/signed_command.ml:27:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/signed_command.ml#L27)
/// Args: MinaBaseSignedCommandPayloadStableV2 , NonZeroCurvePoint , MinaBaseSignatureStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandStableV2 {
    pub payload: MinaBaseSignedCommandPayloadStableV2,
    pub signer: NonZeroCurvePoint,
    pub signature: MinaBaseSignatureStableV1,
}

/// **OCaml name**: `Mina_base__Receipt.Chain_hash.Stable.V1`
///
/// Gid: `717`
/// Location: [src/lib/mina_base/receipt.ml:31:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/receipt.ml#L31)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseReceiptChainHashStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__State_body_hash.Stable.V1`
///
/// Gid: `722`
/// Location: [src/lib/mina_base/state_body_hash.ml:19:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/state_body_hash.ml#L19)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStateBodyHashStableV1(pub crate::bigint::BigInt);

/// Derived name: `Mina_base__Account_update.Update.Stable.V1.timing`
///
/// Gid: `728`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: MinaBaseAccountUpdateUpdateTimingInfoStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateUpdateStableV1Timing {
    Set(Box<MinaBaseAccountUpdateUpdateTimingInfoStableV1>),
    Keep,
}

/// Derived name: `Mina_base__Account_update.Update.Stable.V1.permissions`
///
/// Gid: `728`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: MinaBasePermissionsStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateUpdateStableV1Permissions {
    Set(Box<MinaBasePermissionsStableV2>),
    Keep,
}

/// Derived name: `Mina_base__Account_update.Update.Stable.V1.verification_key`
///
/// Gid: `728`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: MinaBaseVerificationKeyWireStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateUpdateStableV1VerificationKey {
    Set(Box<MinaBaseVerificationKeyWireStableV1>),
    Keep,
}

/// Derived name: `Mina_base__Account_update.Update.Stable.V1.token_symbol`
///
/// Gid: `728`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: MinaBaseZkappAccountZkappUriStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateUpdateStableV1TokenSymbol {
    Set(MinaBaseZkappAccountZkappUriStableV1),
    Keep,
}

/// Derived name: `Mina_base__Account_update.Update.Stable.V1.delegate`
///
/// Gid: `728`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: NonZeroCurvePoint
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateUpdateStableV1Delegate {
    Set(NonZeroCurvePoint),
    Keep,
}

/// Derived name: `Mina_base__Account_update.Update.Stable.V1.voting_for`
///
/// Gid: `728`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: StateHash
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateUpdateStableV1VotingFor {
    Set(StateHash),
    Keep,
}

/// Derived name: `Mina_base__Account_update.Update.Stable.V1.app_state.a`
///
/// Gid: `728`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: crate :: bigint :: BigInt
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateUpdateStableV1AppStateA {
    Set(crate::bigint::BigInt),
    Keep,
}

/// Derived name: `Mina_base__Account_update.Update.Stable.V1.zkapp_uri`
///
/// Gid: `728`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: crate :: string :: ByteString
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateUpdateStableV1ZkappUri {
    Set(crate::string::ByteString),
    Keep,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Epoch_data.Stable.V1.epoch_seed`
///
/// Gid: `729`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: EpochSeed
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochSeed {
    Check(EpochSeed),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1.snarked_ledger_hash`
///
/// Gid: `729`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: LedgerHash
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash {
    Check(LedgerHash),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Account.Stable.V2.receipt_chain_hash`
///
/// Gid: `729`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: MinaBaseReceiptChainHashStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionAccountStableV2ReceiptChainHash {
    Check(MinaBaseReceiptChainHashStableV1),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Account.Stable.V2.delegate`
///
/// Gid: `729`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: NonZeroCurvePoint
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionAccountStableV2Delegate {
    Check(NonZeroCurvePoint),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Epoch_data.Stable.V1.start_checkpoint`
///
/// Gid: `729`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: StateHash
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionProtocolStateEpochDataStableV1StartCheckpoint {
    Check(StateHash),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Account.Stable.V2.proved_state`
///
/// Gid: `729`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: bool
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionAccountStableV2ProvedState {
    Check(bool),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Account.Stable.V2.state.a`
///
/// Gid: `729`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: crate :: bigint :: BigInt
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionAccountStableV2StateA {
    Check(crate::bigint::BigInt),
    Ignore,
}

/// **OCaml name**: `Mina_base__Zkapp_state.Value.Stable.V1`
///
/// Gid: `732`
/// Location: [src/lib/mina_base/zkapp_state.ml:46:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_state.ml#L46)
///
///
/// Gid: `731`
/// Location: [src/lib/mina_base/zkapp_state.ml:17:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_state.ml#L17)
/// Args: crate :: bigint :: BigInt
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappStateValueStableV1(pub PaddedSeq<crate::bigint::BigInt, 8>);

/// **OCaml name**: `Mina_base__Zkapp_account.Stable.V2`
///
/// Gid: `734`
/// Location: [src/lib/mina_base/zkapp_account.ml:264:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_account.ml#L264)
///
///
/// Gid: `733`
/// Location: [src/lib/mina_base/zkapp_account.ml:234:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_account.ml#L234)
/// Args: MinaBaseZkappStateValueStableV1 , Option < MinaBaseVerificationKeyWireStableV1 > , MinaNumbersNatMake32StableV1 , crate :: bigint :: BigInt , UnsignedExtendedUInt32StableV1 , bool , MinaBaseZkappAccountZkappUriStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappAccountStableV2 {
    pub app_state: MinaBaseZkappStateValueStableV1,
    pub verification_key: Option<MinaBaseVerificationKeyWireStableV1>,
    pub zkapp_version: MinaNumbersNatMake32StableV1,
    pub action_state: PaddedSeq<crate::bigint::BigInt, 5>,
    pub last_action_slot: UnsignedExtendedUInt32StableV1,
    pub proved_state: bool,
    pub zkapp_uri: MinaBaseZkappAccountZkappUriStableV1,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Epoch_data.Stable.V1.epoch_ledger`
///
/// Gid: `743`
/// Location: [src/lib/mina_base/epoch_ledger.ml:9:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/epoch_ledger.ml#L9)
/// Args: MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash , MinaBaseZkappPreconditionProtocolStateStableV1Amount
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochLedger {
    pub hash: MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash,
    pub total_currency: MinaBaseZkappPreconditionProtocolStateStableV1Amount,
}

/// **OCaml name**: `Mina_base__Epoch_ledger.Value.Stable.V1`
///
/// Gid: `744`
/// Location: [src/lib/mina_base/epoch_ledger.ml:23:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/epoch_ledger.ml#L23)
///
///
/// Gid: `743`
/// Location: [src/lib/mina_base/epoch_ledger.ml:9:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/epoch_ledger.ml#L9)
/// Args: LedgerHash , CurrencyAmountStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseEpochLedgerValueStableV1 {
    pub hash: LedgerHash,
    pub total_currency: CurrencyAmountStableV1,
}

/// **OCaml name**: `Mina_base__Epoch_seed.Stable.V1`
///
/// Gid: `747`
/// Location: [src/lib/mina_base/epoch_seed.ml:14:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/epoch_seed.ml#L14)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseEpochSeedStableV1(pub crate::bigint::BigInt);

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1.amount.a`
///
/// Gid: `752`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:23:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_precondition.ml#L23)
/// Args: CurrencyAmountStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionProtocolStateStableV1AmountA {
    pub lower: CurrencyAmountStableV1,
    pub upper: CurrencyAmountStableV1,
}

/// Derived name: `Mina_base__Zkapp_precondition.Account.Stable.V2.balance.a`
///
/// Gid: `752`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:23:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_precondition.ml#L23)
/// Args: CurrencyBalanceStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionAccountStableV2BalanceA {
    pub lower: CurrencyBalanceStableV1,
    pub upper: CurrencyBalanceStableV1,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1.length.a`
///
/// Gid: `752`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:23:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_precondition.ml#L23)
/// Args: UnsignedExtendedUInt32StableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionProtocolStateStableV1LengthA {
    pub lower: UnsignedExtendedUInt32StableV1,
    pub upper: UnsignedExtendedUInt32StableV1,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1.amount`
///
/// Gid: `753`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:165:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_precondition.ml#L165)
/// Args: CurrencyAmountStableV1
///
///
/// Gid: `729`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: MinaBaseZkappPreconditionProtocolStateStableV1AmountA
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionProtocolStateStableV1Amount {
    Check(MinaBaseZkappPreconditionProtocolStateStableV1AmountA),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Account.Stable.V2.balance`
///
/// Gid: `753`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:165:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_precondition.ml#L165)
/// Args: CurrencyBalanceStableV1
///
///
/// Gid: `729`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: MinaBaseZkappPreconditionAccountStableV2BalanceA
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionAccountStableV2Balance {
    Check(MinaBaseZkappPreconditionAccountStableV2BalanceA),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1.length`
///
/// Gid: `753`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:165:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_precondition.ml#L165)
/// Args: UnsignedExtendedUInt32StableV1
///
///
/// Gid: `729`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: MinaBaseZkappPreconditionProtocolStateStableV1LengthA
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionProtocolStateStableV1Length {
    Check(MinaBaseZkappPreconditionProtocolStateStableV1LengthA),
    Ignore,
}

/// **OCaml name**: `Mina_base__Zkapp_precondition.Account.Stable.V2`
///
/// Gid: `754`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:463:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_precondition.ml#L463)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionAccountStableV2 {
    pub balance: MinaBaseZkappPreconditionAccountStableV2Balance,
    pub nonce: MinaBaseZkappPreconditionProtocolStateStableV1Length,
    pub receipt_chain_hash: MinaBaseZkappPreconditionAccountStableV2ReceiptChainHash,
    pub delegate: MinaBaseZkappPreconditionAccountStableV2Delegate,
    pub state: PaddedSeq<MinaBaseZkappPreconditionAccountStableV2StateA, 8>,
    pub action_state: MinaBaseZkappPreconditionAccountStableV2StateA,
    pub proved_state: MinaBaseZkappPreconditionAccountStableV2ProvedState,
    pub is_new: MinaBaseZkappPreconditionAccountStableV2ProvedState,
}

/// **OCaml name**: `Mina_base__Zkapp_precondition.Protocol_state.Epoch_data.Stable.V1`
///
/// Gid: `755`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:777:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_precondition.ml#L777)
///
///
/// Gid: `750`
/// Location: [src/lib/mina_base/epoch_data.ml:8:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/epoch_data.ml#L8)
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
/// Gid: `757`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:955:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_precondition.ml#L955)
///
///
/// Gid: `756`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:908:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_precondition.ml#L908)
/// Args: MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash , MinaBaseZkappPreconditionProtocolStateStableV1Length , () , MinaBaseZkappPreconditionProtocolStateStableV1Length , MinaBaseZkappPreconditionProtocolStateStableV1Amount , MinaBaseZkappPreconditionProtocolStateEpochDataStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionProtocolStateStableV1 {
    pub snarked_ledger_hash: MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash,
    pub blockchain_length: MinaBaseZkappPreconditionProtocolStateStableV1Length,
    pub min_window_density: MinaBaseZkappPreconditionProtocolStateStableV1Length,
    pub last_vrf_output: (),
    pub total_currency: MinaBaseZkappPreconditionProtocolStateStableV1Amount,
    pub global_slot_since_genesis: MinaBaseZkappPreconditionProtocolStateStableV1Length,
    pub staking_epoch_data: MinaBaseZkappPreconditionProtocolStateEpochDataStableV1,
    pub next_epoch_data: MinaBaseZkappPreconditionProtocolStateEpochDataStableV1,
}

/// **OCaml name**: `Mina_base__Account_update.Authorization_kind.Stable.V1`
///
/// Gid: `765`
/// Location: [src/lib/mina_base/account_update.ml:28:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_update.ml#L28)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateAuthorizationKindStableV1 {
    Signature,
    Proof(crate::bigint::BigInt),
    NoneGiven,
}

/// **OCaml name**: `Mina_base__Account_update.May_use_token.Stable.V1`
///
/// Gid: `766`
/// Location: [src/lib/mina_base/account_update.ml:158:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_update.ml#L158)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateMayUseTokenStableV1 {
    No,
    ParentsOwnToken,
    InheritFromParent,
}

/// **OCaml name**: `Mina_base__Account_update.Update.Timing_info.Stable.V1`
///
/// Gid: `767`
/// Location: [src/lib/mina_base/account_update.ml:529:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_update.ml#L529)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountUpdateUpdateTimingInfoStableV1 {
    pub initial_minimum_balance: CurrencyBalanceStableV1,
    pub cliff_time: UnsignedExtendedUInt32StableV1,
    pub cliff_amount: CurrencyAmountStableV1,
    pub vesting_period: UnsignedExtendedUInt32StableV1,
    pub vesting_increment: CurrencyAmountStableV1,
}

/// **OCaml name**: `Mina_base__Account_update.Update.Stable.V1`
///
/// Gid: `768`
/// Location: [src/lib/mina_base/account_update.ml:685:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_update.ml#L685)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountUpdateUpdateStableV1 {
    pub app_state: PaddedSeq<MinaBaseAccountUpdateUpdateStableV1AppStateA, 8>,
    pub delegate: MinaBaseAccountUpdateUpdateStableV1Delegate,
    pub verification_key: MinaBaseAccountUpdateUpdateStableV1VerificationKey,
    pub permissions: MinaBaseAccountUpdateUpdateStableV1Permissions,
    pub zkapp_uri: MinaBaseAccountUpdateUpdateStableV1ZkappUri,
    pub token_symbol: MinaBaseAccountUpdateUpdateStableV1TokenSymbol,
    pub timing: MinaBaseAccountUpdateUpdateStableV1Timing,
    pub voting_for: MinaBaseAccountUpdateUpdateStableV1VotingFor,
}

/// **OCaml name**: `Mina_base__Account_update.Account_precondition.Stable.V1`
///
/// Gid: `769`
/// Location: [src/lib/mina_base/account_update.ml:981:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_update.ml#L981)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateAccountPreconditionStableV1 {
    Full(Box<MinaBaseZkappPreconditionAccountStableV2>),
    Nonce(UnsignedExtendedUInt32StableV1),
    Accept,
}

/// **OCaml name**: `Mina_base__Account_update.Preconditions.Stable.V1`
///
/// Gid: `770`
/// Location: [src/lib/mina_base/account_update.ml:1150:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_update.ml#L1150)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountUpdatePreconditionsStableV1 {
    pub network: MinaBaseZkappPreconditionProtocolStateStableV1,
    pub account: MinaBaseAccountUpdateAccountPreconditionStableV1,
    pub valid_while: MinaBaseZkappPreconditionProtocolStateStableV1Length,
}

/// **OCaml name**: `Mina_base__Account_update.Body.Events'.Stable.V1`
///
/// Gid: `771`
/// Location: [src/lib/mina_base/account_update.ml:1238:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_update.ml#L1238)
///
///
/// Gid: `167`
/// Location: [src/std_internal.ml:131:2](https://github.com/Minaprotocol/mina/blob/32a9161/src/std_internal.ml#L131)
/// Args: Vec < crate :: bigint :: BigInt >
///
///
/// Gid: `50`
/// Location: [src/list0.ml:6:0](https://github.com/Minaprotocol/mina/blob/32a9161/src/list0.ml#L6)
/// Args: Vec < crate :: bigint :: BigInt >
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountUpdateBodyEventsStableV1(pub Vec<Vec<crate::bigint::BigInt>>);

/// **OCaml name**: `Mina_base__Account_update.Body.Stable.V1`
///
/// Gid: `774`
/// Location: [src/lib/mina_base/account_update.ml:1311:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_update.ml#L1311)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountUpdateBodyStableV1 {
    pub public_key: NonZeroCurvePoint,
    pub token_id: TokenIdKeyHash,
    pub update: MinaBaseAccountUpdateUpdateStableV1,
    pub balance_change: MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount,
    pub increment_nonce: bool,
    pub events: MinaBaseAccountUpdateBodyEventsStableV1,
    pub actions: MinaBaseAccountUpdateBodyEventsStableV1,
    pub call_data: crate::bigint::BigInt,
    pub preconditions: MinaBaseAccountUpdatePreconditionsStableV1,
    pub use_full_commitment: bool,
    pub implicit_account_creation_fee: bool,
    pub may_use_token: MinaBaseAccountUpdateMayUseTokenStableV1,
    pub authorization_kind: MinaBaseAccountUpdateAuthorizationKindStableV1,
}

/// **OCaml name**: `Mina_base__Account_update.Body.Fee_payer.Stable.V1`
///
/// Gid: `775`
/// Location: [src/lib/mina_base/account_update.ml:1441:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_update.ml#L1441)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountUpdateBodyFeePayerStableV1 {
    pub public_key: NonZeroCurvePoint,
    pub fee: CurrencyFeeStableV1,
    pub valid_until: Option<UnsignedExtendedUInt32StableV1>,
    pub nonce: UnsignedExtendedUInt32StableV1,
}

/// **OCaml name**: `Mina_base__Account_update.T.Stable.V1`
///
/// Gid: `778`
/// Location: [src/lib/mina_base/account_update.ml:1813:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_update.ml#L1813)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountUpdateTStableV1 {
    pub body: MinaBaseAccountUpdateBodyStableV1,
    pub authorization: MinaBaseControlStableV2,
}

/// **OCaml name**: `Mina_base__Account_update.Fee_payer.Stable.V1`
///
/// Gid: `779`
/// Location: [src/lib/mina_base/account_update.ml:1867:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_update.ml#L1867)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountUpdateFeePayerStableV1 {
    pub body: MinaBaseAccountUpdateBodyFeePayerStableV1,
    pub authorization: MinaBaseSignatureStableV1,
}

/// Derived name: `Mina_base__Zkapp_command.T.Stable.V1.Wire.Stable.V1.account_updates.a.a.calls.a`
///
/// Gid: `780`
/// Location: [src/lib/mina_base/with_stack_hash.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/with_stack_hash.ml#L6)
/// Args: Box < MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA > , ()
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAACallsA {
    pub elt: Box<MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA>,
    pub stack_hash: (),
}

/// Derived name: `Mina_base__Zkapp_command.T.Stable.V1.Wire.Stable.V1.account_updates.a`
///
/// Gid: `780`
/// Location: [src/lib/mina_base/with_stack_hash.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/with_stack_hash.ml#L6)
/// Args: MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA , ()
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA {
    pub elt: MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA,
    pub stack_hash: (),
}

/// Derived name: `Mina_transaction_logic.Transaction_applied.Coinbase_applied.Stable.V2.coinbase`
///
/// Gid: `781`
/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/with_status.ml#L6)
/// Args: MinaBaseCoinbaseStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedCoinbaseAppliedStableV2Coinbase {
    pub data: MinaBaseCoinbaseStableV1,
    pub status: MinaBaseTransactionStatusStableV2,
}

/// Derived name: `Mina_transaction_logic.Transaction_applied.Fee_transfer_applied.Stable.V2.fee_transfer`
///
/// Gid: `781`
/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/with_status.ml#L6)
/// Args: MinaBaseFeeTransferStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedFeeTransferAppliedStableV2FeeTransfer {
    pub data: MinaBaseFeeTransferStableV2,
    pub status: MinaBaseTransactionStatusStableV2,
}

/// Derived name: `Mina_transaction_logic.Transaction_applied.Signed_command_applied.Common.Stable.V2.user_command`
///
/// Gid: `781`
/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/with_status.ml#L6)
/// Args: MinaBaseSignedCommandStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV2UserCommand {
    pub data: MinaBaseSignedCommandStableV2,
    pub status: MinaBaseTransactionStatusStableV2,
}

/// Derived name: `Staged_ledger_diff__Diff.Make_str.Pre_diff_with_at_most_two_coinbase.Stable.V2.b`
///
/// Gid: `781`
/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/with_status.ml#L6)
/// Args: MinaBaseUserCommandStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B {
    pub data: MinaBaseUserCommandStableV2,
    pub status: MinaBaseTransactionStatusStableV2,
}

/// Derived name: `Mina_transaction_logic.Transaction_applied.Zkapp_command_applied.Stable.V1.command`
///
/// Gid: `781`
/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/with_status.ml#L6)
/// Args: MinaBaseZkappCommandTStableV1WireStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedZkappCommandAppliedStableV1Command {
    pub data: MinaBaseZkappCommandTStableV1WireStableV1,
    pub status: MinaBaseTransactionStatusStableV2,
}

/// Derived name: `Mina_base__Zkapp_command.T.Stable.V1.Wire.Stable.V1.account_updates.a.a`
///
/// Gid: `782`
/// Location: [src/lib/mina_base/zkapp_command.ml:11:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_command.ml#L11)
/// Args: MinaBaseAccountUpdateTStableV1 , () , ()
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA {
    pub account_update: MinaBaseAccountUpdateTStableV1,
    pub account_update_digest: (),
    pub calls: Vec<MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAACallsA>,
}

/// **OCaml name**: `Mina_base__Zkapp_command.T.Stable.V1.Wire.Stable.V1`
///
/// Gid: `791`
/// Location: [src/lib/mina_base/zkapp_command.ml:765:12](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_command.ml#L765)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappCommandTStableV1WireStableV1 {
    pub fee_payer: MinaBaseAccountUpdateFeePayerStableV1,
    pub account_updates: Vec<MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA>,
    pub memo: MinaBaseSignedCommandMemoStableV1,
}

/// **OCaml name**: `Mina_base__User_command.Stable.V2`
///
/// Gid: `801`
/// Location: [src/lib/mina_base/user_command.ml:79:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/user_command.ml#L79)
///
///
/// Gid: `799`
/// Location: [src/lib/mina_base/user_command.ml:7:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/user_command.ml#L7)
/// Args: MinaBaseSignedCommandStableV2 , MinaBaseZkappCommandTStableV1WireStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseUserCommandStableV2 {
    SignedCommand(MinaBaseSignedCommandStableV2),
    ZkappCommand(MinaBaseZkappCommandTStableV1WireStableV1),
}

/// **OCaml name**: `Mina_base__Fee_transfer.Make_str.Single.Stable.V2`
///
/// Gid: `805`
/// Location: [src/lib/mina_base/fee_transfer.ml:19:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/fee_transfer.ml#L19)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeTransferSingleStableV2 {
    pub receiver_pk: NonZeroCurvePoint,
    pub fee: CurrencyFeeStableV1,
    pub fee_token: TokenIdKeyHash,
}

/// **OCaml name**: `Mina_base__Fee_transfer.Make_str.Stable.V2`
///
/// Gid: `806`
/// Location: [src/lib/mina_base/fee_transfer.ml:69:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/fee_transfer.ml#L69)
///
///
/// Gid: `503`
/// Location: [src/lib/one_or_two/one_or_two.ml:7:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/one_or_two/one_or_two.ml#L7)
/// Args: MinaBaseFeeTransferSingleStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[polymorphic_variant]
pub enum MinaBaseFeeTransferStableV2 {
    One(MinaBaseFeeTransferSingleStableV2),
    Two(
        (
            MinaBaseFeeTransferSingleStableV2,
            MinaBaseFeeTransferSingleStableV2,
        ),
    ),
}

/// **OCaml name**: `Mina_base__Coinbase_fee_transfer.Make_str.Stable.V1`
///
/// Gid: `807`
/// Location: [src/lib/mina_base/coinbase_fee_transfer.ml:15:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/coinbase_fee_transfer.ml#L15)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseCoinbaseFeeTransferStableV1 {
    pub receiver_pk: NonZeroCurvePoint,
    pub fee: CurrencyFeeStableV1,
}

/// **OCaml name**: `Mina_base__Coinbase.Make_str.Stable.V1`
///
/// Gid: `808`
/// Location: [src/lib/mina_base/coinbase.ml:17:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/coinbase.ml#L17)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseCoinbaseStableV1 {
    pub receiver: NonZeroCurvePoint,
    pub amount: CurrencyAmountStableV1,
    pub fee_transfer: Option<MinaBaseCoinbaseFeeTransferStableV1>,
}

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.Stack_id.Stable.V1`
///
/// Gid: `810`
/// Location: [src/lib/mina_base/pending_coinbase.ml:106:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/pending_coinbase.ml#L106)
///
///
/// Gid: `163`
/// Location: [src/std_internal.ml:119:2](https://github.com/Minaprotocol/mina/blob/32a9161/src/std_internal.ml#L119)
///
///
/// Gid: `113`
/// Location: [src/int.ml:19:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/int.ml#L19)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStackIdStableV1(pub crate::number::Int32);

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.Coinbase_stack.Stable.V1`
///
/// Gid: `813`
/// Location: [src/lib/mina_base/pending_coinbase.ml:159:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/pending_coinbase.ml#L159)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseCoinbaseStackStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.Stack_hash.Stable.V1`
///
/// Gid: `818`
/// Location: [src/lib/mina_base/pending_coinbase.ml:219:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/pending_coinbase.ml#L219)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStackHashStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.State_stack.Stable.V1`
///
/// Gid: `822`
/// Location: [src/lib/mina_base/pending_coinbase.ml:255:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/pending_coinbase.ml#L255)
///
///
/// Gid: `821`
/// Location: [src/lib/mina_base/pending_coinbase.ml:245:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/pending_coinbase.ml#L245)
/// Args: MinaBasePendingCoinbaseStackHashStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStateStackStableV1 {
    pub init: CoinbaseStackHash,
    pub curr: CoinbaseStackHash,
}

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.Hash_builder.Stable.V1`
///
/// Gid: `825`
/// Location: [src/lib/mina_base/pending_coinbase.ml:372:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/pending_coinbase.ml#L372)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseHashBuilderStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.Stack_versioned.Stable.V1`
///
/// Gid: `832`
/// Location: [src/lib/mina_base/pending_coinbase.ml:521:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/pending_coinbase.ml#L521)
///
///
/// Gid: `831`
/// Location: [src/lib/mina_base/pending_coinbase.ml:510:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/pending_coinbase.ml#L510)
/// Args: MinaBasePendingCoinbaseCoinbaseStackStableV1 , MinaBasePendingCoinbaseStateStackStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStackVersionedStableV1 {
    pub data: MinaBasePendingCoinbaseCoinbaseStackStableV1,
    pub state: MinaBasePendingCoinbaseStateStackStableV1,
}

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.Hash_versioned.Stable.V1`
///
/// Gid: `833`
/// Location: [src/lib/mina_base/pending_coinbase.ml:534:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/pending_coinbase.ml#L534)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseHashVersionedStableV1(
    pub MinaBasePendingCoinbaseHashBuilderStableV1,
);

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.Merkle_tree_versioned.Stable.V2`
///
/// Gid: `834`
/// Location: [src/lib/mina_base/pending_coinbase.ml:546:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/pending_coinbase.ml#L546)
///
///
/// Gid: `626`
/// Location: [src/lib/sparse_ledger_lib/sparse_ledger.ml:38:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/sparse_ledger_lib/sparse_ledger.ml#L38)
/// Args: PendingCoinbaseHash , MinaBasePendingCoinbaseStackIdStableV1 , MinaBasePendingCoinbaseStackVersionedStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseMerkleTreeVersionedStableV2 {
    pub indexes: Vec<(MinaBasePendingCoinbaseStackIdStableV1, crate::number::Int32)>,
    pub depth: crate::number::Int32,
    pub tree: MinaBasePendingCoinbaseMerkleTreeVersionedStableV2Tree,
}

/// **OCaml name**: `Mina_base__Staged_ledger_hash.Make_str.Aux_hash.Stable.V1`
///
/// Gid: `837`
/// Location: [src/lib/mina_base/staged_ledger_hash.ml:27:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/staged_ledger_hash.ml#L27)
///
///
/// Gid: `170`
/// Location: [src/std_internal.ml:140:2](https://github.com/Minaprotocol/mina/blob/32a9161/src/std_internal.ml#L140)
///
///
/// Gid: `83`
/// Location: [src/string.ml:44:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/string.ml#L44)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashAuxHashStableV1(pub crate::string::ByteString);

/// **OCaml name**: `Mina_base__Staged_ledger_hash.Make_str.Pending_coinbase_aux.Stable.V1`
///
/// Gid: `838`
/// Location: [src/lib/mina_base/staged_ledger_hash.ml:110:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/staged_ledger_hash.ml#L110)
///
///
/// Gid: `170`
/// Location: [src/std_internal.ml:140:2](https://github.com/Minaprotocol/mina/blob/32a9161/src/std_internal.ml#L140)
///
///
/// Gid: `83`
/// Location: [src/string.ml:44:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/string.ml#L44)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashPendingCoinbaseAuxStableV1(pub crate::string::ByteString);

/// **OCaml name**: `Mina_base__Staged_ledger_hash.Make_str.Non_snark.Stable.V1`
///
/// Gid: `839`
/// Location: [src/lib/mina_base/staged_ledger_hash.ml:152:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/staged_ledger_hash.ml#L152)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashNonSnarkStableV1 {
    pub ledger_hash: LedgerHash,
    pub aux_hash: StagedLedgerHashAuxHash,
    pub pending_coinbase_aux: StagedLedgerHashPendingCoinbaseAux,
}

/// **OCaml name**: `Mina_base__Staged_ledger_hash.Make_str.Stable.V1`
///
/// Gid: `841`
/// Location: [src/lib/mina_base/staged_ledger_hash.ml:259:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/staged_ledger_hash.ml#L259)
///
///
/// Gid: `840`
/// Location: [src/lib/mina_base/staged_ledger_hash.ml:241:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/staged_ledger_hash.ml#L241)
/// Args: MinaBaseStagedLedgerHashNonSnarkStableV1 , PendingCoinbaseHash
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashStableV1 {
    pub non_snark: MinaBaseStagedLedgerHashNonSnarkStableV1,
    pub pending_coinbase_hash: PendingCoinbaseHash,
}

/// **OCaml name**: `Mina_base__Stack_frame.Make_str.Stable.V1`
///
/// Gid: `843`
/// Location: [src/lib/mina_base/stack_frame.ml:64:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/stack_frame.ml#L64)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStackFrameStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Sok_message.Make_str.Stable.V1`
///
/// Gid: `845`
/// Location: [src/lib/mina_base/sok_message.ml:14:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/sok_message.ml#L14)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSokMessageStableV1 {
    pub fee: CurrencyFeeStableV1,
    pub prover: NonZeroCurvePoint,
}

/// **OCaml name**: `Mina_base__Protocol_constants_checked.Value.Stable.V1`
///
/// Gid: `846`
/// Location: [src/lib/mina_base/protocol_constants_checked.ml:22:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/protocol_constants_checked.ml#L22)
///
///
/// Gid: `622`
/// Location: [src/lib/genesis_constants/genesis_constants.ml:239:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/genesis_constants/genesis_constants.ml#L239)
/// Args: UnsignedExtendedUInt32StableV1 , UnsignedExtendedUInt32StableV1 , BlockTimeTimeStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseProtocolConstantsCheckedValueStableV1 {
    pub k: UnsignedExtendedUInt32StableV1,
    pub slots_per_epoch: UnsignedExtendedUInt32StableV1,
    pub slots_per_sub_window: UnsignedExtendedUInt32StableV1,
    pub delta: UnsignedExtendedUInt32StableV1,
    pub genesis_state_timestamp: BlockTimeTimeStableV1,
}

/// **OCaml name**: `Mina_base__Proof.Stable.V2`
///
/// Gid: `847`
/// Location: [src/lib/mina_base/proof.ml:12:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/proof.ml#L12)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseProofStableV2(pub PicklesProofProofsVerified2ReprStableV2);

/// **OCaml name**: `Mina_base__Call_stack_digest.Make_str.Stable.V1`
///
/// Gid: `849`
/// Location: [src/lib/mina_base/call_stack_digest.ml:12:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/call_stack_digest.ml#L12)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseCallStackDigestStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Fee_with_prover.Stable.V1`
///
/// Gid: `850`
/// Location: [src/lib/mina_base/fee_with_prover.ml:7:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/fee_with_prover.ml#L7)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeWithProverStableV1 {
    pub fee: CurrencyFeeStableV1,
    pub prover: NonZeroCurvePoint,
}

/// **OCaml name**: `Network_peer__Peer.Id.Stable.V1`
///
/// Gid: `851`
/// Location: [src/lib/network_peer/peer.ml:10:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/network_peer/peer.ml#L10)
///
///
/// Gid: `170`
/// Location: [src/std_internal.ml:140:2](https://github.com/Minaprotocol/mina/blob/32a9161/src/std_internal.ml#L140)
///
///
/// Gid: `83`
/// Location: [src/string.ml:44:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/string.ml#L44)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NetworkPeerPeerIdStableV1(pub crate::string::ByteString);

/// **OCaml name**: `Mina_transaction_logic__Zkapp_command_logic.Local_state.Value.Stable.V1`
///
/// Gid: `866`
/// Location: [src/lib/transaction_logic/zkapp_command_logic.ml:251:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_logic/zkapp_command_logic.ml#L251)
///
///
/// Gid: `865`
/// Location: [src/lib/transaction_logic/zkapp_command_logic.ml:188:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_logic/zkapp_command_logic.ml#L188)
/// Args: MinaBaseStackFrameStableV1 , MinaBaseCallStackDigestStableV1 , TokenIdKeyHash , MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount , LedgerHash , bool , crate :: bigint :: BigInt , UnsignedExtendedUInt32StableV1 , MinaBaseTransactionStatusFailureCollectionStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1 {
    pub stack_frame: MinaBaseStackFrameStableV1,
    pub call_stack: MinaBaseCallStackDigestStableV1,
    pub transaction_commitment: crate::bigint::BigInt,
    pub full_transaction_commitment: crate::bigint::BigInt,
    pub token_id: TokenIdKeyHash,
    pub excess: MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount,
    pub supply_increase: MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount,
    pub ledger: LedgerHash,
    pub success: bool,
    pub account_update_index: UnsignedExtendedUInt32StableV1,
    pub failure_status_tbl: MinaBaseTransactionStatusFailureCollectionStableV1,
    pub will_succeed: bool,
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Signed_command_applied.Common.Stable.V2`
///
/// Gid: `868`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:17:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_logic/mina_transaction_logic.ml#L17)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV2 {
    pub user_command:
        MinaTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV2UserCommand,
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Signed_command_applied.Body.Stable.V2`
///
/// Gid: `869`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:31:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_logic/mina_transaction_logic.ml#L31)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaTransactionLogicTransactionAppliedSignedCommandAppliedBodyStableV2 {
    Payment {
        new_accounts: Vec<MinaBaseAccountIdStableV2>,
    },
    StakeDelegation {
        previous_delegate: Option<NonZeroCurvePoint>,
    },
    Failed,
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Signed_command_applied.Stable.V2`
///
/// Gid: `870`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:46:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_logic/mina_transaction_logic.ml#L46)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedSignedCommandAppliedStableV2 {
    pub common: MinaTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV2,
    pub body: MinaTransactionLogicTransactionAppliedSignedCommandAppliedBodyStableV2,
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Zkapp_command_applied.Stable.V1`
///
/// Gid: `871`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:65:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_logic/mina_transaction_logic.ml#L65)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedZkappCommandAppliedStableV1 {
    pub accounts: Vec<(
        MinaBaseAccountIdStableV2,
        Option<MinaBaseAccountBinableArgStableV2>,
    )>,
    pub command: MinaTransactionLogicTransactionAppliedZkappCommandAppliedStableV1Command,
    pub new_accounts: Vec<MinaBaseAccountIdStableV2>,
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Command_applied.Stable.V2`
///
/// Gid: `872`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:82:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_logic/mina_transaction_logic.ml#L82)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaTransactionLogicTransactionAppliedCommandAppliedStableV2 {
    SignedCommand(MinaTransactionLogicTransactionAppliedSignedCommandAppliedStableV2),
    ZkappCommand(MinaTransactionLogicTransactionAppliedZkappCommandAppliedStableV1),
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Fee_transfer_applied.Stable.V2`
///
/// Gid: `873`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:96:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_logic/mina_transaction_logic.ml#L96)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedFeeTransferAppliedStableV2 {
    pub fee_transfer: MinaTransactionLogicTransactionAppliedFeeTransferAppliedStableV2FeeTransfer,
    pub new_accounts: Vec<MinaBaseAccountIdStableV2>,
    pub burned_tokens: CurrencyAmountStableV1,
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Coinbase_applied.Stable.V2`
///
/// Gid: `874`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:112:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_logic/mina_transaction_logic.ml#L112)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedCoinbaseAppliedStableV2 {
    pub coinbase: MinaTransactionLogicTransactionAppliedCoinbaseAppliedStableV2Coinbase,
    pub new_accounts: Vec<MinaBaseAccountIdStableV2>,
    pub burned_tokens: CurrencyAmountStableV1,
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Varying.Stable.V2`
///
/// Gid: `875`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:128:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_logic/mina_transaction_logic.ml#L128)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaTransactionLogicTransactionAppliedVaryingStableV2 {
    Command(MinaTransactionLogicTransactionAppliedCommandAppliedStableV2),
    FeeTransfer(MinaTransactionLogicTransactionAppliedFeeTransferAppliedStableV2),
    Coinbase(MinaTransactionLogicTransactionAppliedCoinbaseAppliedStableV2),
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Stable.V2`
///
/// Gid: `876`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:142:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_logic/mina_transaction_logic.ml#L142)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedStableV2 {
    pub previous_hash: LedgerHash,
    pub varying: MinaTransactionLogicTransactionAppliedVaryingStableV2,
}

/// **OCaml name**: `Merkle_address.Binable_arg.Stable.V1`
///
/// Gid: `877`
/// Location: [src/lib/merkle_address/merkle_address.ml:48:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/merkle_address/merkle_address.ml#L48)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MerkleAddressBinableArgStableV1(pub crate::number::Int32, pub crate::string::ByteString);

/// **OCaml name**: `Trust_system__Banned_status.Stable.V1`
///
/// Gid: `883`
/// Location: [src/lib/trust_system/banned_status.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/trust_system/banned_status.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum TrustSystemBannedStatusStableV1 {
    Unbanned,
    BannedUntil(crate::number::Float64),
}

/// **OCaml name**: `Consensus_vrf.Output.Truncated.Stable.V1`
///
/// Gid: `898`
/// Location: [src/lib/consensus/vrf/consensus_vrf.ml:163:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/consensus/vrf/consensus_vrf.ml#L163)
///
///
/// Gid: `170`
/// Location: [src/std_internal.ml:140:2](https://github.com/Minaprotocol/mina/blob/32a9161/src/std_internal.ml#L140)
///
///
/// Gid: `83`
/// Location: [src/string.ml:44:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/string.ml#L44)
#[derive(Clone, Debug, Deref, PartialEq, BinProtRead, BinProtWrite)]
pub struct ConsensusVrfOutputTruncatedStableV1(pub crate::string::ByteString);

/// **OCaml name**: `Consensus__Body_reference.Stable.V1`
///
/// Gid: `916`
/// Location: [src/lib/consensus/body_reference.ml:17:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/consensus/body_reference.ml#L17)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusBodyReferenceStableV1(pub Blake2MakeStableV1);

/// **OCaml name**: `Consensus__Global_slot.Make_str.Stable.V1`
///
/// Gid: `923`
/// Location: [src/lib/consensus/global_slot.ml:33:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/consensus/global_slot.ml#L33)
///
///
/// Gid: `922`
/// Location: [src/lib/consensus/global_slot.ml:22:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/consensus/global_slot.ml#L22)
/// Args: UnsignedExtendedUInt32StableV1 , UnsignedExtendedUInt32StableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusGlobalSlotStableV1 {
    pub slot_number: UnsignedExtendedUInt32StableV1,
    pub slots_per_epoch: UnsignedExtendedUInt32StableV1,
}

/// **OCaml name**: `Consensus__Proof_of_stake.Make_str.Data.Epoch_data.Staking_value_versioned.Value.Stable.V1`
///
/// Gid: `933`
/// Location: [src/lib/consensus/proof_of_stake.ml:1090:14](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/consensus/proof_of_stake.ml#L1090)
///
///
/// Gid: `750`
/// Location: [src/lib/mina_base/epoch_data.ml:8:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/epoch_data.ml#L8)
/// Args: MinaBaseEpochLedgerValueStableV1 , EpochSeed , StateHash , StateHash , UnsignedExtendedUInt32StableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1 {
    pub ledger: MinaBaseEpochLedgerValueStableV1,
    pub seed: EpochSeed,
    pub start_checkpoint: StateHash,
    pub lock_checkpoint: StateHash,
    pub epoch_length: UnsignedExtendedUInt32StableV1,
}

/// **OCaml name**: `Consensus__Proof_of_stake.Make_str.Data.Epoch_data.Next_value_versioned.Value.Stable.V1`
///
/// Gid: `934`
/// Location: [src/lib/consensus/proof_of_stake.ml:1115:14](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/consensus/proof_of_stake.ml#L1115)
///
///
/// Gid: `750`
/// Location: [src/lib/mina_base/epoch_data.ml:8:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/epoch_data.ml#L8)
/// Args: MinaBaseEpochLedgerValueStableV1 , EpochSeed , StateHash , StateHash , UnsignedExtendedUInt32StableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1 {
    pub ledger: MinaBaseEpochLedgerValueStableV1,
    pub seed: EpochSeed,
    pub start_checkpoint: StateHash,
    pub lock_checkpoint: StateHash,
    pub epoch_length: UnsignedExtendedUInt32StableV1,
}

/// Derived name: `Mina_state__Blockchain_state.Value.Stable.V2.ledger_proof_statement.source`
///
/// Gid: `937`
/// Location: [src/lib/mina_state/registers.ml:8:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_state/registers.ml#L8)
/// Args: LedgerHash , MinaBasePendingCoinbaseStackVersionedStableV1 , MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateBlockchainStateValueStableV2LedgerProofStatementSource {
    pub first_pass_ledger: LedgerHash,
    pub second_pass_ledger: LedgerHash,
    pub pending_coinbase_stack: MinaBasePendingCoinbaseStackVersionedStableV1,
    pub local_state: MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1,
}

/// **OCaml name**: `Mina_state__Snarked_ledger_state.Make_str.Pending_coinbase_stack_state.Init_stack.Stable.V1`
///
/// Gid: `938`
/// Location: [src/lib/mina_state/snarked_ledger_state.ml:38:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_state/snarked_ledger_state.ml#L38)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaStateSnarkedLedgerStatePendingCoinbaseStackStateInitStackStableV1 {
    Base(MinaBasePendingCoinbaseStackVersionedStableV1),
    Merge,
}

/// Derived name: `Mina_state__Blockchain_state.Value.Stable.V2.ledger_proof_statement`
///
/// Gid: `943`
/// Location: [src/lib/mina_state/snarked_ledger_state.ml:107:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_state/snarked_ledger_state.ml#L107)
/// Args: LedgerHash , MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount , MinaBasePendingCoinbaseStackVersionedStableV1 , MinaBaseFeeExcessStableV1 , () , MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateBlockchainStateValueStableV2LedgerProofStatement {
    pub source: MinaStateBlockchainStateValueStableV2LedgerProofStatementSource,
    pub target: MinaStateBlockchainStateValueStableV2LedgerProofStatementSource,
    pub connecting_ledger_left: LedgerHash,
    pub connecting_ledger_right: LedgerHash,
    pub supply_increase: MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount,
    pub fee_excess: MinaBaseFeeExcessStableV1,
    pub sok_digest: (),
}

/// **OCaml name**: `Mina_state__Snarked_ledger_state.Make_str.Stable.V2`
///
/// Gid: `944`
/// Location: [src/lib/mina_state/snarked_ledger_state.ml:191:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_state/snarked_ledger_state.ml#L191)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateSnarkedLedgerStateStableV2(
    pub MinaStateBlockchainStateValueStableV2LedgerProofStatement,
);

/// **OCaml name**: `Mina_state__Snarked_ledger_state.Make_str.With_sok.Stable.V2`
///
/// Gid: `945`
/// Location: [src/lib/mina_state/snarked_ledger_state.ml:345:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_state/snarked_ledger_state.ml#L345)
///
///
/// Gid: `943`
/// Location: [src/lib/mina_state/snarked_ledger_state.ml:107:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_state/snarked_ledger_state.ml#L107)
/// Args: LedgerHash , MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount , MinaBasePendingCoinbaseStackVersionedStableV1 , MinaBaseFeeExcessStableV1 , MinaBaseZkappAccountZkappUriStableV1 , MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateSnarkedLedgerStateWithSokStableV2 {
    pub source: MinaStateBlockchainStateValueStableV2LedgerProofStatementSource,
    pub target: MinaStateBlockchainStateValueStableV2LedgerProofStatementSource,
    pub connecting_ledger_left: LedgerHash,
    pub connecting_ledger_right: LedgerHash,
    pub supply_increase: MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount,
    pub fee_excess: MinaBaseFeeExcessStableV1,
    pub sok_digest: MinaBaseZkappAccountZkappUriStableV1,
}

/// **OCaml name**: `Mina_state__Blockchain_state.Value.Stable.V2`
///
/// Gid: `949`
/// Location: [src/lib/mina_state/blockchain_state.ml:68:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_state/blockchain_state.ml#L68)
///
///
/// Gid: `948`
/// Location: [src/lib/mina_state/blockchain_state.ml:10:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_state/blockchain_state.ml#L10)
/// Args: MinaBaseStagedLedgerHashStableV1 , LedgerHash , MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1 , BlockTimeTimeStableV1 , ConsensusBodyReferenceStableV1 , MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount , MinaBasePendingCoinbaseStackVersionedStableV1 , MinaBaseFeeExcessStableV1 , ()
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateBlockchainStateValueStableV2 {
    pub staged_ledger_hash: MinaBaseStagedLedgerHashStableV1,
    pub genesis_ledger_hash: LedgerHash,
    pub ledger_proof_statement: MinaStateBlockchainStateValueStableV2LedgerProofStatement,
    pub timestamp: BlockTimeTimeStableV1,
    pub body_reference: ConsensusBodyReferenceStableV1,
}

/// **OCaml name**: `Mina_state__Protocol_state.Make_str.Body.Value.Stable.V2`
///
/// Gid: `955`
/// Location: [src/lib/mina_state/protocol_state.ml:82:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_state/protocol_state.ml#L82)
///
///
/// Gid: `953`
/// Location: [src/lib/mina_state/protocol_state.ml:62:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_state/protocol_state.ml#L62)
/// Args: StateHash , MinaStateBlockchainStateValueStableV2 , ConsensusProofOfStakeDataConsensusStateValueStableV1 , MinaBaseProtocolConstantsCheckedValueStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateProtocolStateBodyValueStableV2 {
    pub genesis_state_hash: StateHash,
    pub blockchain_state: MinaStateBlockchainStateValueStableV2,
    pub consensus_state: ConsensusProofOfStakeDataConsensusStateValueStableV1,
    pub constants: MinaBaseProtocolConstantsCheckedValueStableV1,
}

/// **OCaml name**: `Transaction_snark.Make_str.Proof.Stable.V2`
///
/// Gid: `963`
/// Location: [src/lib/transaction_snark/transaction_snark.ml:67:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_snark/transaction_snark.ml#L67)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkProofStableV2(pub PicklesProofProofsVerified2ReprStableV2);

/// **OCaml name**: `Transaction_snark.Make_str.Stable.V2`
///
/// Gid: `964`
/// Location: [src/lib/transaction_snark/transaction_snark.ml:78:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_snark/transaction_snark.ml#L78)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkStableV2 {
    pub statement: MinaStateSnarkedLedgerStateWithSokStableV2,
    pub proof: TransactionSnarkProofStableV2,
}

/// **OCaml name**: `Ledger_proof.Prod.Stable.V2`
///
/// Gid: `966`
/// Location: [src/lib/ledger_proof/ledger_proof.ml:10:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/ledger_proof/ledger_proof.ml#L10)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct LedgerProofProdStableV2(pub TransactionSnarkStableV2);

/// **OCaml name**: `Transaction_snark_work.Statement.Stable.V2`
///
/// Gid: `968`
/// Location: [src/lib/transaction_snark_work/transaction_snark_work.ml:23:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_snark_work/transaction_snark_work.ml#L23)
///
///
/// Gid: `503`
/// Location: [src/lib/one_or_two/one_or_two.ml:7:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/one_or_two/one_or_two.ml#L7)
/// Args: MinaStateSnarkedLedgerStateStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[polymorphic_variant]
pub enum TransactionSnarkWorkStatementStableV2 {
    One(MinaStateSnarkedLedgerStateStableV2),
    Two(
        (
            MinaStateSnarkedLedgerStateStableV2,
            MinaStateSnarkedLedgerStateStableV2,
        ),
    ),
}

/// **OCaml name**: `Transaction_snark_work.T.Stable.V2`
///
/// Gid: `972`
/// Location: [src/lib/transaction_snark_work/transaction_snark_work.ml:82:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_snark_work/transaction_snark_work.ml#L82)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkWorkTStableV2 {
    pub fee: CurrencyFeeStableV1,
    pub proofs: TransactionSnarkWorkTStableV2Proofs,
    pub prover: NonZeroCurvePoint,
}

/// Derived name: `Staged_ledger_diff__Diff.Make_str.Pre_diff_with_at_most_two_coinbase.Stable.V2.coinbase`
///
/// Gid: `973`
/// Location: [src/lib/staged_ledger_diff/diff.ml:28:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/staged_ledger_diff/diff.ml#L28)
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

/// Derived name: `Staged_ledger_diff__Diff.Make_str.Pre_diff_with_at_most_one_coinbase.Stable.V2.coinbase`
///
/// Gid: `974`
/// Location: [src/lib/staged_ledger_diff/diff.ml:64:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/staged_ledger_diff/diff.ml#L64)
/// Args: StagedLedgerDiffDiffFtStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2Coinbase {
    Zero,
    One(Option<StagedLedgerDiffDiffFtStableV1>),
}

/// **OCaml name**: `Staged_ledger_diff__Diff.Make_str.Ft.Stable.V1`
///
/// Gid: `975`
/// Location: [src/lib/staged_ledger_diff/diff.ml:88:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/staged_ledger_diff/diff.ml#L88)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffFtStableV1(pub MinaBaseCoinbaseFeeTransferStableV1);

/// **OCaml name**: `Staged_ledger_diff__Diff.Make_str.Pre_diff_with_at_most_two_coinbase.Stable.V2`
///
/// Gid: `978`
/// Location: [src/lib/staged_ledger_diff/diff.ml:168:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/staged_ledger_diff/diff.ml#L168)
///
///
/// Gid: `976`
/// Location: [src/lib/staged_ledger_diff/diff.ml:104:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/staged_ledger_diff/diff.ml#L104)
/// Args: TransactionSnarkWorkTStableV2 , StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2 {
    pub completed_works: Vec<TransactionSnarkWorkTStableV2>,
    pub commands: Vec<StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B>,
    pub coinbase: StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2Coinbase,
    pub internal_command_statuses: Vec<MinaBaseTransactionStatusStableV2>,
}

/// **OCaml name**: `Staged_ledger_diff__Diff.Make_str.Pre_diff_with_at_most_one_coinbase.Stable.V2`
///
/// Gid: `979`
/// Location: [src/lib/staged_ledger_diff/diff.ml:187:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/staged_ledger_diff/diff.ml#L187)
///
///
/// Gid: `977`
/// Location: [src/lib/staged_ledger_diff/diff.ml:136:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/staged_ledger_diff/diff.ml#L136)
/// Args: TransactionSnarkWorkTStableV2 , StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2 {
    pub completed_works: Vec<TransactionSnarkWorkTStableV2>,
    pub commands: Vec<StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B>,
    pub coinbase: StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2Coinbase,
    pub internal_command_statuses: Vec<MinaBaseTransactionStatusStableV2>,
}

/// **OCaml name**: `Staged_ledger_diff__Diff.Make_str.Diff.Stable.V2`
///
/// Gid: `980`
/// Location: [src/lib/staged_ledger_diff/diff.ml:206:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/staged_ledger_diff/diff.ml#L206)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffDiffStableV2(
    pub StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2,
    pub Option<StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2>,
);

/// **OCaml name**: `Staged_ledger_diff__Diff.Make_str.Stable.V2`
///
/// Gid: `981`
/// Location: [src/lib/staged_ledger_diff/diff.ml:223:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/staged_ledger_diff/diff.ml#L223)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffStableV2 {
    pub diff: StagedLedgerDiffDiffDiffStableV2,
}

/// **OCaml name**: `Staged_ledger_diff__Body.Make_str.Stable.V1`
///
/// Gid: `982`
/// Location: [src/lib/staged_ledger_diff/body.ml:18:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/staged_ledger_diff/body.ml#L18)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffBodyStableV1 {
    pub staged_ledger_diff: StagedLedgerDiffDiffStableV2,
}

/// **OCaml name**: `Protocol_version.Make_str.Stable.V1`
///
/// Gid: `1024`
/// Location: [src/lib/protocol_version/protocol_version.ml:14:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/protocol_version/protocol_version.ml#L14)
#[derive(Clone, Debug, PartialEq, BinProtRead, BinProtWrite)]
pub struct ProtocolVersionStableV1 {
    pub major: crate::number::Int32,
    pub minor: crate::number::Int32,
    pub patch: crate::number::Int32,
}

/// **OCaml name**: `Parallel_scan.Sequence_number.Stable.V1`
///
/// Gid: `1029`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:22:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/parallel_scan/parallel_scan.ml#L22)
///
///
/// Gid: `163`
/// Location: [src/std_internal.ml:119:2](https://github.com/Minaprotocol/mina/blob/32a9161/src/std_internal.ml#L119)
///
///
/// Gid: `113`
/// Location: [src/int.ml:19:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/int.ml#L19)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ParallelScanSequenceNumberStableV1(pub crate::number::Int32);

/// **OCaml name**: `Parallel_scan.Job_status.Stable.V1`
///
/// Gid: `1030`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:35:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/parallel_scan/parallel_scan.ml#L35)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum ParallelScanJobStatusStableV1 {
    Todo,
    Done,
}

/// **OCaml name**: `Parallel_scan.Weight.Stable.V1`
///
/// Gid: `1031`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:53:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/parallel_scan/parallel_scan.ml#L53)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ParallelScanWeightStableV1 {
    pub base: crate::number::Int32,
    pub merge: crate::number::Int32,
}

/// Derived name: `Transaction_snark_scan_state.Stable.V2.scan_state.trees.a.base_t.1.Full`
///
/// Gid: `1032`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:68:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/parallel_scan/parallel_scan.ml#L68)
/// Args: TransactionSnarkScanStateTransactionWithWitnessStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateStableV2ScanStateTreesABaseT1Full {
    pub job: TransactionSnarkScanStateTransactionWithWitnessStableV2,
    pub seq_no: ParallelScanSequenceNumberStableV1,
    pub status: ParallelScanJobStatusStableV1,
}

/// Derived name: `Transaction_snark_scan_state.Stable.V2.scan_state.trees.a.base_t.1`
///
/// Gid: `1033`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:84:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/parallel_scan/parallel_scan.ml#L84)
/// Args: TransactionSnarkScanStateTransactionWithWitnessStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum TransactionSnarkScanStateStableV2ScanStateTreesABaseT1 {
    Empty,
    Full(Box<TransactionSnarkScanStateStableV2ScanStateTreesABaseT1Full>),
}

/// Derived name: `Transaction_snark_scan_state.Stable.V2.scan_state.trees.a.merge_t.1.Full`
///
/// Gid: `1035`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:112:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/parallel_scan/parallel_scan.ml#L112)
/// Args: TransactionSnarkScanStateLedgerProofWithSokMessageStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateStableV2ScanStateTreesAMergeT1Full {
    pub left: TransactionSnarkScanStateLedgerProofWithSokMessageStableV2,
    pub right: TransactionSnarkScanStateLedgerProofWithSokMessageStableV2,
    pub seq_no: ParallelScanSequenceNumberStableV1,
    pub status: ParallelScanJobStatusStableV1,
}

/// Derived name: `Transaction_snark_scan_state.Stable.V2.scan_state.trees.a.merge_t.1`
///
/// Gid: `1036`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:130:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/parallel_scan/parallel_scan.ml#L130)
/// Args: TransactionSnarkScanStateLedgerProofWithSokMessageStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum TransactionSnarkScanStateStableV2ScanStateTreesAMergeT1 {
    Empty,
    Part(Box<TransactionSnarkScanStateLedgerProofWithSokMessageStableV2>),
    Full(Box<TransactionSnarkScanStateStableV2ScanStateTreesAMergeT1Full>),
}

/// Derived name: `Transaction_snark_scan_state.Stable.V2.scan_state`
///
/// Gid: `1043`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:803:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/parallel_scan/parallel_scan.ml#L803)
/// Args: TransactionSnarkScanStateLedgerProofWithSokMessageStableV2 , TransactionSnarkScanStateTransactionWithWitnessStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateStableV2ScanState {
    pub trees: (
        TransactionSnarkScanStateStableV2ScanStateTreesA,
        Vec<TransactionSnarkScanStateStableV2ScanStateTreesA>,
    ),
    pub acc: Option<(
        TransactionSnarkScanStateLedgerProofWithSokMessageStableV2,
        Vec<TransactionSnarkScanStateTransactionWithWitnessStableV2>,
    )>,
    pub curr_job_seq_no: crate::number::Int32,
    pub max_base_jobs: crate::number::Int32,
    pub delay: crate::number::Int32,
}

/// **OCaml name**: `Transaction_snark_scan_state.Transaction_with_witness.Stable.V2`
///
/// Gid: `1044`
/// Location: [src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml:40:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml#L40)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateTransactionWithWitnessStableV2 {
    pub transaction_with_info: MinaTransactionLogicTransactionAppliedStableV2,
    pub state_hash: (StateHash, StateBodyHash),
    pub statement: MinaStateSnarkedLedgerStateStableV2,
    pub init_stack: MinaStateSnarkedLedgerStatePendingCoinbaseStackStateInitStackStableV1,
    pub first_pass_ledger_witness: MinaBaseSparseLedgerBaseStableV2,
    pub second_pass_ledger_witness: MinaBaseSparseLedgerBaseStableV2,
    pub block_global_slot: UnsignedExtendedUInt32StableV1,
}

/// **OCaml name**: `Transaction_snark_scan_state.Ledger_proof_with_sok_message.Stable.V2`
///
/// Gid: `1045`
/// Location: [src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml:65:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml#L65)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateLedgerProofWithSokMessageStableV2(
    pub LedgerProofProdStableV2,
    pub MinaBaseSokMessageStableV1,
);

/// **OCaml name**: `Mina_block__Header.Make_str.Stable.V2`
///
/// Gid: `1047`
/// Location: [src/lib/mina_block/header.ml:21:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_block/header.ml#L21)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBlockHeaderStableV2 {
    pub protocol_state: MinaStateProtocolStateValueStableV2,
    pub protocol_state_proof: MinaBaseProofStableV2,
    pub delta_block_chain_proof: (StateHash, Vec<StateBodyHash>),
    pub current_protocol_version: ProtocolVersionStableV1,
    pub proposed_protocol_version_opt: Option<ProtocolVersionStableV1>,
}

/// Derived name: `Network_pool__Snark_pool.Diff_versioned.Stable.V2.Add_solved_work.1`
///
/// Gid: `1066`
/// Location: [src/lib/network_pool/priced_proof.ml:9:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/network_pool/priced_proof.ml#L9)
/// Args: TransactionSnarkWorkTStableV2Proofs
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NetworkPoolSnarkPoolDiffVersionedStableV2AddSolvedWork1 {
    pub proof: TransactionSnarkWorkTStableV2Proofs,
    pub fee: MinaBaseFeeWithProverStableV1,
}

/// Derived name: `Transaction_snark_scan_state.Stable.V2.previous_incomplete_zkapp_updates.1`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[polymorphic_variant]
pub enum TransactionSnarkScanStateStableV2PreviousIncompleteZkappUpdates1 {
    BorderBlockContinuedInTheNextTree(bool),
}
