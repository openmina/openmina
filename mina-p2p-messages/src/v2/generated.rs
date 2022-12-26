use binprot_derive::{BinProtRead, BinProtWrite};
use derive_more::Deref;
use serde::{Deserialize, Serialize};

use crate::pseq::PaddedSeq;

use super::manual::*;

/// **OCaml name**: `Mina_block__Block.Stable.V2`
///
/// Gid: `1057`
/// Location: [src/lib/mina_block/block.ml:8:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_block/block.ml#L8)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBlockBlockStableV2 {
    pub header: MinaBlockHeaderStableV2,
    pub body: StagedLedgerDiffBodyStableV1,
}

/// **OCaml name**: `Network_pool__Transaction_pool.Diff_versioned.Stable.V2`
///
/// Gid: `1084`
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
/// Gid: `1090`
/// Location: [src/lib/network_pool/snark_pool.ml:732:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/network_pool/snark_pool.ml#L732)
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
/// Gid: `849`
/// Location: [src/lib/mina_base/sparse_ledger_base.ml:8:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/sparse_ledger_base.ml#L8)
///
///
/// Gid: `628`
/// Location: [src/lib/sparse_ledger_lib/sparse_ledger.ml:38:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/sparse_ledger_lib/sparse_ledger.ml#L38)
/// Args: MinaBaseLedgerHash0StableV1 , MinaBaseAccountIdStableV2 , MinaBaseAccountBinableArgStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSparseLedgerBaseStableV2 {
    pub indexes: Vec<(MinaBaseAccountIdStableV2, crate::number::Int32)>,
    pub depth: crate::number::Int32,
    pub tree: MinaBaseSparseLedgerBaseStableV2Tree,
}

/// **OCaml name**: `Mina_base__Account.Binable_arg.Stable.V2`
///
/// Gid: `746`
/// Location: [src/lib/mina_base/account.ml:311:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account.ml#L311)
///
///
/// Gid: `743`
/// Location: [src/lib/mina_base/account.ml:226:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account.ml#L226)
/// Args: NonZeroCurvePointUncompressedStableV1 , MinaBaseTokenIdStableV2 , MinaBaseTokenPermissionsStableV1 , MinaBaseSokMessageDigestStableV1 , CurrencyBalanceStableV1 , UnsignedExtendedUInt32StableV1 , MinaBaseReceiptChainHashStableV1 , Option < NonZeroCurvePointUncompressedStableV1 > , DataHashLibStateHashStableV1 , MinaBaseAccountTimingStableV1 , MinaBasePermissionsStableV2 , Option < MinaBaseZkappAccountStableV2 >
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountBinableArgStableV2 {
    pub public_key: NonZeroCurvePoint,
    pub token_id: TokenIdKeyHash,
    pub token_permissions: MinaBaseTokenPermissionsStableV1,
    pub token_symbol: MinaBaseSokMessageDigestStableV1,
    pub balance: CurrencyBalanceStableV1,
    pub nonce: UnsignedExtendedUInt32StableV1,
    pub receipt_chain_hash: MinaBaseReceiptChainHashStableV1,
    pub delegate: Option<NonZeroCurvePoint>,
    pub voting_for: DataHashLibStateHashStableV1,
    pub timing: MinaBaseAccountTimingStableV1,
    pub permissions: MinaBasePermissionsStableV2,
    pub zkapp: Option<MinaBaseZkappAccountStableV2>,
}

/// **OCaml name**: `Network_peer__Peer.Stable.V1`
///
/// Gid: `885`
/// Location: [src/lib/network_peer/peer.ml:28:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/network_peer/peer.ml#L28)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NetworkPeerPeerStableV1 {
    pub host: crate::string::ByteString,
    pub libp2p_port: crate::number::Int32,
    pub peer_id: NetworkPeerPeerIdStableV1,
}

/// **OCaml name**: `Transaction_snark_scan_state.Stable.V2`
///
/// Gid: `1034`
/// Location: [src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml:153:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml#L153)
///
///
/// Gid: `1031`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:803:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/parallel_scan/parallel_scan.ml#L803)
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

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.Stable.V2`
///
/// Gid: `841`
/// Location: [src/lib/mina_base/pending_coinbase.ml:1268:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/pending_coinbase.ml#L1268)
///
///
/// Gid: `840`
/// Location: [src/lib/mina_base/pending_coinbase.ml:1256:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/pending_coinbase.ml#L1256)
/// Args: MinaBasePendingCoinbaseMerkleTreeVersionedStableV2 , MinaBasePendingCoinbaseStackIdStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStableV2 {
    pub tree: MinaBasePendingCoinbaseMerkleTreeVersionedStableV2,
    pub pos_list: Vec<MinaBasePendingCoinbaseStackIdStableV1>,
    pub new_pos: MinaBasePendingCoinbaseStackIdStableV1,
}

/// **OCaml name**: `Mina_state__Protocol_state.Make_str.Value.Stable.V2`
///
/// Gid: `954`
/// Location: [src/lib/mina_state/protocol_state.ml:205:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_state/protocol_state.ml#L205)
///
///
/// Gid: `950`
/// Location: [src/lib/mina_state/protocol_state.ml:38:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_state/protocol_state.ml#L38)
/// Args: DataHashLibStateHashStableV1 , MinaStateProtocolStateBodyValueStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateProtocolStateValueStableV2 {
    pub previous_state_hash: StateHash,
    pub body: MinaStateProtocolStateBodyValueStableV2,
}

/// **OCaml name**: `Mina_ledger__Sync_ledger.Query.Stable.V1`
///
/// Gid: `903`
/// Location: [src/lib/mina_ledger/sync_ledger.ml:71:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_ledger/sync_ledger.ml#L71)
///
///
/// Gid: `892`
/// Location: [src/lib/syncable_ledger/syncable_ledger.ml:17:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/syncable_ledger/syncable_ledger.ml#L17)
/// Args: MerkleAddressBinableArgStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaLedgerSyncLedgerQueryStableV1 {
    WhatChildHashes(MerkleAddressBinableArgStableV1),
    WhatContents(MerkleAddressBinableArgStableV1),
    NumAccounts,
    WhatAccountWithPath(NonZeroCurvePoint, TokenIdKeyHash),
}

/// **OCaml name**: `Mina_ledger__Sync_ledger.Answer.Stable.V2`
///
/// Gid: `902`
/// Location: [src/lib/mina_ledger/sync_ledger.ml:56:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_ledger/sync_ledger.ml#L56)
///
///
/// Gid: `893`
/// Location: [src/lib/syncable_ledger/syncable_ledger.ml:35:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/syncable_ledger/syncable_ledger.ml#L35)
/// Args: MinaBaseLedgerHash0StableV1 , MinaBaseAccountBinableArgStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaLedgerSyncLedgerAnswerStableV2 {
    ChildHashesAre(MinaBaseLedgerHash0StableV1, MinaBaseLedgerHash0StableV1),
    ContentsAre(Vec<MinaBaseAccountBinableArgStableV2>),
    NumAccounts(crate::number::Int32, MinaBaseLedgerHash0StableV1),
    AccountWithPath(MinaBaseAccountBinableArgStableV2, super::MerkleTreePath),
}

/// **OCaml name**: `Consensus__Proof_of_stake.Make_str.Data.Consensus_state.Value.Stable.V1`
///
/// Gid: `942`
/// Location: [src/lib/consensus/proof_of_stake.ml:1773:12](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/consensus/proof_of_stake.ml#L1773)
///
///
/// Gid: `941`
/// Location: [src/lib/consensus/proof_of_stake.ml:1728:12](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/consensus/proof_of_stake.ml#L1728)
/// Args: UnsignedExtendedUInt32StableV1 , ConsensusVrfOutputTruncatedStableV1 , CurrencyAmountStableV1 , ConsensusGlobalSlotStableV1 , UnsignedExtendedUInt32StableV1 , ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1 , ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1 , bool , NonZeroCurvePointUncompressedStableV1
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
/// Gid: `1114`
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
/// Gid: `890`
/// Location: [src/lib/trust_system/peer_status.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/trust_system/peer_status.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TrustSystemPeerStatusStableV1 {
    pub trust: crate::number::Float64,
    pub banned: TrustSystemBannedStatusStableV1,
}

/// **OCaml name**: `Blockchain_snark__Blockchain.Stable.V2`
///
/// Gid: `995`
/// Location: [src/lib/blockchain_snark/blockchain.ml:8:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/blockchain_snark/blockchain.ml#L8)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct BlockchainSnarkBlockchainStableV2 {
    pub state: MinaStateProtocolStateValueStableV2,
    pub proof: MinaBaseProofStableV2,
}

/// **OCaml name**: `Mina_base__Sok_message.Make_str.Digest.Stable.V1`
///
/// Gid: `73`
/// Location: [src/string.ml:14:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/string.ml#L14)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSokMessageDigestStableV1(pub crate::string::ByteString);

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#statement#fp`
///
/// Gid: `458`
/// Location: [src/lib/pickles_types/shifted_value.ml:96:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_types/shifted_value.ml#L96)
/// Args: crate :: bigint :: BigInt
#[derive(Clone, Debug, PartialEq, BinProtRead, BinProtWrite)]
pub enum PicklesProofProofsVerified2ReprStableV2StatementFp {
    ShiftedValue(crate::bigint::BigInt),
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#prev_evals#evals#evals#lookup#a`
///
/// Gid: `461`
/// Location: [src/lib/pickles_types/plonk_types.ml:179:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_types/plonk_types.ml#L179)
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
/// Location: [src/lib/pickles_types/plonk_types.ml:248:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_types/plonk_types.ml#L248)
/// Args: (Vec < crate :: bigint :: BigInt > , Vec < crate :: bigint :: BigInt > ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals {
    pub w: PaddedSeq<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>), 15>,
    pub z: (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
    pub s: PaddedSeq<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>), 6>,
    pub generic_selector: (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
    pub poseidon_selector: (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
    pub lookup: Option<PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvalsLookupA>,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#prev_evals#evals`
///
/// Gid: `463`
/// Location: [src/lib/pickles_types/plonk_types.ml:436:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_types/plonk_types.ml#L436)
/// Args: (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,) , (Vec < crate :: bigint :: BigInt > , Vec < crate :: bigint :: BigInt > ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PrevEvalsEvals {
    pub public_input: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub evals: PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#prev_evals`
///
/// Gid: `464`
/// Location: [src/lib/pickles_types/plonk_types.ml:471:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_types/plonk_types.ml#L471)
/// Args: crate :: bigint :: BigInt , Vec < crate :: bigint :: BigInt >
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PrevEvals {
    pub evals: PicklesProofProofsVerified2ReprStableV2PrevEvalsEvals,
    pub ft_eval1: crate::bigint::BigInt,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#proof#openings#proof`
///
/// Gid: `465`
/// Location: [src/lib/pickles_types/plonk_types.ml:520:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_types/plonk_types.ml#L520)
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
/// Location: [src/lib/pickles_types/plonk_types.ml:542:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_types/plonk_types.ml#L542)
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
/// Location: [src/lib/pickles_types/plonk_types.ml:606:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_types/plonk_types.ml#L606)
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
/// Location: [src/lib/pickles_types/plonk_types.ml:644:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_types/plonk_types.ml#L644)
/// Args: (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2ProofMessages {
    pub w_comm: PaddedSeq<Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>, 15>,
    pub z_comm: Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>,
    pub t_comm: Vec<(crate::bigint::BigInt, crate::bigint::BigInt)>,
    pub lookup: Option<PicklesProofProofsVerified2ReprStableV2ProofMessagesLookupA>,
}

/// Derived name: `Mina_base__Verification_key_wire.Stable.V1#wrap_index`
///
/// Gid: `473`
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

/// Derived name: `Pickles__Reduced_messages_for_next_proof_over_same_field.Wrap.Challenges_vector.Stable.V2#a#challenge`
///
/// Gid: `478`
/// Location: [src/lib/crypto/kimchi_backend/common/scalar_challenge.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/crypto/kimchi_backend/common/scalar_challenge.ml#L6)
/// Args: (LimbVectorConstantHex64StableV1 , (LimbVectorConstantHex64StableV1 , () ,) ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge {
    pub inner: PaddedSeq<LimbVectorConstantHex64StableV1, 2>,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#proof`
///
/// Gid: `497`
/// Location: [src/lib/crypto/kimchi_backend/common/plonk_dlog_proof.ml:160:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/crypto/kimchi_backend/common/plonk_dlog_proof.ml#L160)
///
///
/// Gid: `471`
/// Location: [src/lib/pickles_types/plonk_types.ml:693:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_types/plonk_types.ml#L693)
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

/// **OCaml name**: `Pickles_base__Proofs_verified.Stable.V1`
///
/// Gid: `510`
/// Location: [src/lib/pickles_base/proofs_verified.ml:8:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_base/proofs_verified.ml#L8)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum PicklesBaseProofsVerifiedStableV1 {
    N0,
    N1,
    N2,
}

/// **OCaml name**: `Limb_vector__Constant.Hex64.Stable.V1`
///
/// Gid: `519`
/// Location: [src/lib/pickles/limb_vector/constant.ml:60:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/limb_vector/constant.ml#L60)
///
///
/// Gid: `125`
/// Location: [src/int64.ml:6:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/int64.ml#L6)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct LimbVectorConstantHex64StableV1(pub crate::number::Int64);

/// **OCaml name**: `Composition_types__Branch_data.Make_str.Domain_log2.Stable.V1`
///
/// Gid: `520`
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
/// Gid: `521`
/// Location: [src/lib/pickles/composition_types/branch_data.ml:51:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/composition_types/branch_data.ml#L51)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CompositionTypesBranchDataStableV1 {
    pub proofs_verified: (PicklesBaseProofsVerifiedStableV1,),
    pub domain_log2: CompositionTypesBranchDataDomainLog2StableV1,
}

/// Derived name: `Pickles__Reduced_messages_for_next_proof_over_same_field.Wrap.Challenges_vector.Stable.V2#a`
///
/// Gid: `522`
/// Location: [src/lib/pickles/composition_types/bulletproof_challenge.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/composition_types/bulletproof_challenge.ml#L6)
/// Args: PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A {
    pub prechallenge:
        PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge,
}

/// **OCaml name**: `Composition_types__Digest.Constant.Stable.V1`
///
/// Gid: `523`
/// Location: [src/lib/pickles/composition_types/digest.ml:13:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/composition_types/digest.ml#L13)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CompositionTypesDigestConstantStableV1(
    pub PaddedSeq<LimbVectorConstantHex64StableV1, 4>,
);

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#statement#plonk`
///
/// Gid: `524`
/// Location: [src/lib/pickles/composition_types/composition_types.ml:45:14](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/composition_types/composition_types.ml#L45)
/// Args: (LimbVectorConstantHex64StableV1 , (LimbVectorConstantHex64StableV1 , () ,) ,) , PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge
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
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#statement#proof_state#deferred_values`
///
/// Gid: `525`
/// Location: [src/lib/pickles/composition_types/composition_types.ml:206:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/composition_types/composition_types.ml#L206)
/// Args: PicklesProofProofsVerified2ReprStableV2StatementPlonk , PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge , PicklesProofProofsVerified2ReprStableV2StatementFp , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , () ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) , CompositionTypesBranchDataStableV1
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

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#messages_for_next_wrap_proof`
///
/// Gid: `526`
/// Location: [src/lib/pickles/composition_types/composition_types.ml:342:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/composition_types/composition_types.ml#L342)
/// Args: (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,) , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2 , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2 , () ,) ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof {
    pub challenge_polynomial_commitment: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub old_bulletproof_challenges:
        PaddedSeq<PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2, 2>,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#statement#proof_state`
///
/// Gid: `527`
/// Location: [src/lib/pickles/composition_types/composition_types.ml:375:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/composition_types/composition_types.ml#L375)
/// Args: PicklesProofProofsVerified2ReprStableV2StatementPlonk , PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge , PicklesProofProofsVerified2ReprStableV2StatementFp , PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof , CompositionTypesDigestConstantStableV1 , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , () ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) , CompositionTypesBranchDataStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2StatementProofState {
    pub deferred_values: PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValues,
    pub sponge_digest_before_evaluations: CompositionTypesDigestConstantStableV1,
    pub messages_for_next_wrap_proof:
        PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#statement`
///
/// Gid: `529`
/// Location: [src/lib/pickles/composition_types/composition_types.ml:625:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/composition_types/composition_types.ml#L625)
/// Args: (LimbVectorConstantHex64StableV1 , (LimbVectorConstantHex64StableV1 , () ,) ,) , PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge , PicklesProofProofsVerified2ReprStableV2StatementFp , PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof , CompositionTypesDigestConstantStableV1 , PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , () ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) , CompositionTypesBranchDataStableV1
///
///
/// Gid: `528`
/// Location: [src/lib/pickles/composition_types/composition_types.ml:588:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/composition_types/composition_types.ml#L588)
/// Args: PicklesProofProofsVerified2ReprStableV2StatementPlonk , PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge , PicklesProofProofsVerified2ReprStableV2StatementFp , PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof , CompositionTypesDigestConstantStableV1 , PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , (PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , () ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) ,) , CompositionTypesBranchDataStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2Statement {
    pub proof_state: PicklesProofProofsVerified2ReprStableV2StatementProofState,
    pub messages_for_next_step_proof:
        PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2#messages_for_next_step_proof`
///
/// Gid: `532`
/// Location: [src/lib/pickles/reduced_messages_for_next_proof_over_same_field.ml:16:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/reduced_messages_for_next_proof_over_same_field.ml#L16)
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
/// Gid: `533`
/// Location: [src/lib/pickles/reduced_messages_for_next_proof_over_same_field.ml:53:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/reduced_messages_for_next_proof_over_same_field.ml#L53)
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
/// Gid: `534`
/// Location: [src/lib/pickles/side_loaded_verification_key.ml:170:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/side_loaded_verification_key.ml#L170)
///
///
/// Gid: `515`
/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:132:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles_base/side_loaded_verification_key.ml#L132)
/// Args: (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseVerificationKeyWireStableV1 {
    pub max_proofs_verified: PicklesBaseProofsVerifiedStableV1,
    pub wrap_index: MinaBaseVerificationKeyWireStableV1WrapIndex,
}

/// **OCaml name**: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2`
///
/// Gid: `537`
/// Location: [src/lib/pickles/proof.ml:340:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/proof.ml#L340)
///
///
/// Gid: `536`
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
/// Gid: `538`
/// Location: [src/lib/pickles/proof.ml:413:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/proof.ml#L413)
///
///
/// Gid: `536`
/// Location: [src/lib/pickles/proof.ml:47:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/pickles/proof.ml#L47)
/// Args: PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof , PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerifiedMaxStableV2 {
    pub statement: PicklesProofProofsVerified2ReprStableV2Statement,
    pub prev_evals: PicklesProofProofsVerified2ReprStableV2PrevEvals,
    pub proof: PicklesProofProofsVerified2ReprStableV2Proof,
}

/// **OCaml name**: `Unsigned_extended.UInt64.Int64_for_version_tags.Stable.V1`
///
/// Gid: `541`
/// Location: [src/lib/unsigned_extended/unsigned_extended.ml:81:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/unsigned_extended/unsigned_extended.ml#L81)
///
///
/// Gid: `125`
/// Location: [src/int64.ml:6:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/int64.ml#L6)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct UnsignedExtendedUInt64Int64ForVersionTagsStableV1(pub crate::number::Int64);

/// **OCaml name**: `Unsigned_extended.UInt32.Stable.V1`
///
/// Gid: `545`
/// Location: [src/lib/unsigned_extended/unsigned_extended.ml:156:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/unsigned_extended/unsigned_extended.ml#L156)
///
///
/// Gid: `119`
/// Location: [src/int32.ml:6:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/int32.ml#L6)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct UnsignedExtendedUInt32StableV1(pub crate::number::Int32);

/// **OCaml name**: `Mina_numbers__Nat.Make32.Stable.V1`
///
/// Gid: `549`
/// Location: [src/lib/mina_numbers/nat.ml:261:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_numbers/nat.ml#L261)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaNumbersNatMake32StableV1(pub UnsignedExtendedUInt32StableV1);

/// **OCaml name**: `Sgn.Stable.V1`
///
/// Gid: `577`
/// Location: [src/lib/sgn/sgn.ml:9:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/sgn/sgn.ml#L9)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum SgnStableV1 {
    Pos,
    Neg,
}

/// Derived name: `Mina_transaction_logic__Zkapp_command_logic.Local_state.Value.Stable.V1#signed_amount`
///
/// Gid: `578`
/// Location: [src/lib/currency/signed_poly.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/currency/signed_poly.ml#L6)
/// Args: CurrencyAmountStableV1 , SgnStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount {
    pub magnitude: CurrencyAmountStableV1,
    pub sgn: (SgnStableV1,),
}

/// Derived name: `Mina_base__Fee_excess.Stable.V1#fee`
///
/// Gid: `578`
/// Location: [src/lib/currency/signed_poly.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/currency/signed_poly.ml#L6)
/// Args: CurrencyFeeStableV1 , SgnStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeExcessStableV1Fee {
    pub magnitude: CurrencyFeeStableV1,
    pub sgn: (SgnStableV1,),
}

/// **OCaml name**: `Currency.Make_str.Fee.Stable.V1`
///
/// Gid: `579`
/// Location: [src/lib/currency/currency.ml:901:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/currency/currency.ml#L901)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CurrencyFeeStableV1(pub UnsignedExtendedUInt64Int64ForVersionTagsStableV1);

/// **OCaml name**: `Currency.Make_str.Amount.Make_str.Stable.V1`
///
/// Gid: `582`
/// Location: [src/lib/currency/currency.ml:1037:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/currency/currency.ml#L1037)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CurrencyAmountStableV1(pub UnsignedExtendedUInt64Int64ForVersionTagsStableV1);

/// **OCaml name**: `Currency.Make_str.Balance.Stable.V1`
///
/// Gid: `585`
/// Location: [src/lib/currency/currency.ml:1079:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/currency/currency.ml#L1079)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CurrencyBalanceStableV1(pub CurrencyAmountStableV1);

/// **OCaml name**: `Non_zero_curve_point.Uncompressed.Stable.V1`
///
/// Gid: `592`
/// Location: [src/lib/non_zero_curve_point/non_zero_curve_point.ml:46:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/non_zero_curve_point/non_zero_curve_point.ml#L46)
///
///
/// Gid: `586`
/// Location: [src/lib/non_zero_curve_point/compressed_poly.ml:13:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/non_zero_curve_point/compressed_poly.ml#L13)
/// Args: crate :: bigint :: BigInt , bool
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, BinProtRead, BinProtWrite,
)]
pub struct NonZeroCurvePointUncompressedStableV1 {
    pub x: crate::bigint::BigInt,
    pub is_odd: bool,
}

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

/// **OCaml name**: `Block_time.Make_str.Time.Stable.V1`
///
/// Gid: `625`
/// Location: [src/lib/block_time/block_time.ml:22:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/block_time/block_time.ml#L22)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct BlockTimeTimeStableV1(pub UnsignedExtendedUInt64Int64ForVersionTagsStableV1);

/// Derived name: `Mina_base__Sparse_ledger_base.Stable.V2#tree`
///
/// Gid: `627`
/// Location: [src/lib/sparse_ledger_lib/sparse_ledger.ml:9:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/sparse_ledger_lib/sparse_ledger.ml#L9)
/// Args: MinaBaseLedgerHash0StableV1 , MinaBaseAccountBinableArgStableV2
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

/// Derived name: `Mina_base__Pending_coinbase.Make_str.Merkle_tree_versioned.Stable.V2#tree`
///
/// Gid: `627`
/// Location: [src/lib/sparse_ledger_lib/sparse_ledger.ml:9:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/sparse_ledger_lib/sparse_ledger.ml#L9)
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
/// Gid: `629`
/// Location: [src/lib/one_or_two/one_or_two.ml:7:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/one_or_two/one_or_two.ml#L7)
/// Args: LedgerProofProdStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[polymorphic_variant]
pub enum TransactionSnarkWorkTStableV2Proofs {
    One(LedgerProofProdStableV2),
    Two((LedgerProofProdStableV2, LedgerProofProdStableV2)),
}

/// **OCaml name**: `Mina_base__Account_id.Make_str.Digest.Stable.V1`
///
/// Gid: `631`
/// Location: [src/lib/mina_base/account_id.ml:64:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_id.ml#L64)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountIdDigestStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Account_id.Make_str.Stable.V2`
///
/// Gid: `636`
/// Location: [src/lib/mina_base/account_id.ml:151:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_id.ml#L151)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountIdStableV2(pub NonZeroCurvePoint, pub MinaBaseAccountIdDigestStableV1);

/// **OCaml name**: `Mina_base__Account_timing.Stable.V1`
///
/// Gid: `642`
/// Location: [src/lib/mina_base/account_timing.ml:30:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_timing.ml#L30)
///
///
/// Gid: `641`
/// Location: [src/lib/mina_base/account_timing.ml:13:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_timing.ml#L13)
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
/// Gid: `646`
/// Location: [src/lib/mina_base/signature.ml:23:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/signature.ml#L23)
///
///
/// Gid: `643`
/// Location: [src/lib/mina_base/signature.ml:12:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/signature.ml#L12)
/// Args: crate :: bigint :: BigInt , crate :: bigint :: BigInt
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignatureStableV1(pub crate::bigint::BigInt, pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Control.Stable.V2`
///
/// Gid: `650`
/// Location: [src/lib/mina_base/control.ml:11:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/control.ml#L11)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseControlStableV2 {
    Proof(Box<PicklesProofProofsVerifiedMaxStableV2>),
    Signature(MinaBaseSignatureStableV1),
    NoneGiven,
}

/// **OCaml name**: `Mina_base__Token_id.Stable.V2`
///
/// Gid: `654`
/// Location: [src/lib/mina_base/token_id.ml:8:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/token_id.ml#L8)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseTokenIdStableV2(pub MinaBaseAccountIdDigestStableV1);

/// **OCaml name**: `Mina_base__Fee_excess.Stable.V1`
///
/// Gid: `659`
/// Location: [src/lib/mina_base/fee_excess.ml:124:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/fee_excess.ml#L124)
///
///
/// Gid: `658`
/// Location: [src/lib/mina_base/fee_excess.ml:54:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/fee_excess.ml#L54)
/// Args: MinaBaseTokenIdStableV2 , MinaBaseFeeExcessStableV1Fee
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeExcessStableV1 {
    pub fee_token_l: MinaBaseTokenIdStableV2,
    pub fee_excess_l: MinaBaseFeeExcessStableV1Fee,
    pub fee_token_r: MinaBaseTokenIdStableV2,
    pub fee_excess_r: MinaBaseFeeExcessStableV1Fee,
}

/// **OCaml name**: `Mina_base__Payment_payload.Stable.V2`
///
/// Gid: `664`
/// Location: [src/lib/mina_base/payment_payload.ml:39:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/payment_payload.ml#L39)
///
///
/// Gid: `660`
/// Location: [src/lib/mina_base/payment_payload.ml:14:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/payment_payload.ml#L14)
/// Args: NonZeroCurvePointUncompressedStableV1 , CurrencyAmountStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePaymentPayloadStableV2 {
    pub source_pk: NonZeroCurvePoint,
    pub receiver_pk: NonZeroCurvePoint,
    pub amount: CurrencyAmountStableV1,
}

/// **OCaml name**: `Mina_base__Ledger_hash0.Stable.V1`
///
/// Gid: `670`
/// Location: [src/lib/mina_base/ledger_hash0.ml:17:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/ledger_hash0.ml#L17)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseLedgerHash0StableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Permissions.Auth_required.Stable.V2`
///
/// Gid: `673`
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
/// Gid: `675`
/// Location: [src/lib/mina_base/permissions.ml:378:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/permissions.ml#L378)
///
///
/// Gid: `674`
/// Location: [src/lib/mina_base/permissions.ml:345:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/permissions.ml#L345)
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
/// Gid: `676`
/// Location: [src/lib/mina_base/signed_command_memo.ml:21:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/signed_command_memo.ml#L21)
///
///
/// Gid: `170`
/// Location: [src/std_internal.ml:140:2](https://github.com/Minaprotocol/mina/blob/32a9161/src/std_internal.ml#L140)
///
///
/// Gid: `83`
/// Location: [src/string.ml:44:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/string.ml#L44)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandMemoStableV1(pub crate::string::CharString);

/// **OCaml name**: `Mina_base__Stake_delegation.Stable.V1`
///
/// Gid: `679`
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
/// Gid: `682`
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
    ZkappCommandReplayCheckFailed,
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
    Cancelled,
}

/// **OCaml name**: `Mina_base__Transaction_status.Failure.Collection.Stable.V1`
///
/// Gid: `684`
/// Location: [src/lib/mina_base/transaction_status.ml:74:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/transaction_status.ml#L74)
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
/// Gid: `685`
/// Location: [src/lib/mina_base/transaction_status.ml:452:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/transaction_status.ml#L452)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseTransactionStatusStableV2 {
    Applied,
    Failed(MinaBaseTransactionStatusFailureCollectionStableV1),
}

/// **OCaml name**: `Mina_base__Signed_command_payload.Common.Stable.V2`
///
/// Gid: `690`
/// Location: [src/lib/mina_base/signed_command_payload.ml:75:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/signed_command_payload.ml#L75)
///
///
/// Gid: `686`
/// Location: [src/lib/mina_base/signed_command_payload.ml:40:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/signed_command_payload.ml#L40)
/// Args: CurrencyFeeStableV1 , NonZeroCurvePointUncompressedStableV1 , UnsignedExtendedUInt32StableV1 , UnsignedExtendedUInt32StableV1 , MinaBaseSignedCommandMemoStableV1
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
/// Gid: `694`
/// Location: [src/lib/mina_base/signed_command_payload.ml:187:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/signed_command_payload.ml#L187)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSignedCommandPayloadBodyStableV2 {
    Payment(MinaBasePaymentPayloadStableV2),
    StakeDelegation(MinaBaseStakeDelegationStableV1),
}

/// **OCaml name**: `Mina_base__Signed_command_payload.Stable.V2`
///
/// Gid: `701`
/// Location: [src/lib/mina_base/signed_command_payload.ml:288:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/signed_command_payload.ml#L288)
///
///
/// Gid: `698`
/// Location: [src/lib/mina_base/signed_command_payload.ml:270:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/signed_command_payload.ml#L270)
/// Args: MinaBaseSignedCommandPayloadCommonStableV2 , MinaBaseSignedCommandPayloadBodyStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandPayloadStableV2 {
    pub common: MinaBaseSignedCommandPayloadCommonStableV2,
    pub body: MinaBaseSignedCommandPayloadBodyStableV2,
}

/// **OCaml name**: `Mina_base__Signed_command.Make_str.Stable.V2`
///
/// Gid: `708`
/// Location: [src/lib/mina_base/signed_command.ml:52:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/signed_command.ml#L52)
///
///
/// Gid: `705`
/// Location: [src/lib/mina_base/signed_command.ml:27:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/signed_command.ml#L27)
/// Args: MinaBaseSignedCommandPayloadStableV2 , NonZeroCurvePointUncompressedStableV1 , MinaBaseSignatureStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandStableV2 {
    pub payload: MinaBaseSignedCommandPayloadStableV2,
    pub signer: NonZeroCurvePoint,
    pub signature: MinaBaseSignatureStableV1,
}

/// **OCaml name**: `Mina_base__Receipt.Chain_hash.Stable.V1`
///
/// Gid: `719`
/// Location: [src/lib/mina_base/receipt.ml:31:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/receipt.ml#L31)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseReceiptChainHashStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__State_body_hash.Stable.V1`
///
/// Gid: `724`
/// Location: [src/lib/mina_base/state_body_hash.ml:19:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/state_body_hash.ml#L19)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStateBodyHashStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Token_permissions.Stable.V1`
///
/// Gid: `729`
/// Location: [src/lib/mina_base/token_permissions.ml:9:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/token_permissions.ml#L9)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseTokenPermissionsStableV1 {
    TokenOwned { disable_new_accounts: bool },
    NotOwned { account_disabled: bool },
}

/// Derived name: `Mina_base__Account_update.Update.Stable.V1#voting_for`
///
/// Gid: `731`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: DataHashLibStateHashStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateUpdateStableV1VotingFor {
    Set(DataHashLibStateHashStableV1),
    Keep,
}

/// Derived name: `Mina_base__Account_update.Update.Stable.V1#timing`
///
/// Gid: `731`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: MinaBaseAccountUpdateUpdateTimingInfoStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateUpdateStableV1Timing {
    Set(Box<MinaBaseAccountUpdateUpdateTimingInfoStableV1>),
    Keep,
}

/// Derived name: `Mina_base__Account_update.Update.Stable.V1#permissions`
///
/// Gid: `731`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: MinaBasePermissionsStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateUpdateStableV1Permissions {
    Set(Box<MinaBasePermissionsStableV2>),
    Keep,
}

/// Derived name: `Mina_base__Account_update.Update.Stable.V1#token_symbol`
///
/// Gid: `731`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: MinaBaseSokMessageDigestStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateUpdateStableV1TokenSymbol {
    Set(MinaBaseSokMessageDigestStableV1),
    Keep,
}

/// Derived name: `Mina_base__Account_update.Update.Stable.V1#verification_key`
///
/// Gid: `731`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: MinaBaseVerificationKeyWireStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateUpdateStableV1VerificationKey {
    Set(Box<MinaBaseVerificationKeyWireStableV1>),
    Keep,
}

/// Derived name: `Mina_base__Account_update.Update.Stable.V1#delegate`
///
/// Gid: `731`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: NonZeroCurvePointUncompressedStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateUpdateStableV1Delegate {
    Set(NonZeroCurvePointUncompressedStableV1),
    Keep,
}

/// Derived name: `Mina_base__Account_update.Update.Stable.V1#app_state#a`
///
/// Gid: `731`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: crate :: bigint :: BigInt
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateUpdateStableV1AppStateA {
    Set(crate::bigint::BigInt),
    Keep,
}

/// Derived name: `Mina_base__Account_update.Update.Stable.V1#zkapp_uri`
///
/// Gid: `731`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: crate :: string :: ByteString
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateUpdateStableV1ZkappUri {
    Set(crate::string::ByteString),
    Keep,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Epoch_data.Stable.V1#start_checkpoint`
///
/// Gid: `732`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: DataHashLibStateHashStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionProtocolStateEpochDataStableV1StartCheckpoint {
    Check(StateHash),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Epoch_data.Stable.V1#epoch_seed`
///
/// Gid: `732`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: MinaBaseEpochSeedStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochSeed {
    Check(EpochSeed),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1#snarked_ledger_hash`
///
/// Gid: `732`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: MinaBaseLedgerHash0StableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash {
    Check(LedgerHash),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Account.Stable.V2#receipt_chain_hash`
///
/// Gid: `732`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: MinaBaseReceiptChainHashStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionAccountStableV2ReceiptChainHash {
    Check(MinaBaseReceiptChainHashStableV1),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Account.Stable.V2#delegate`
///
/// Gid: `732`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: NonZeroCurvePointUncompressedStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionAccountStableV2Delegate {
    Check(NonZeroCurvePoint),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Account.Stable.V2#proved_state`
///
/// Gid: `732`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: bool
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionAccountStableV2ProvedState {
    Check(bool),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Account.Stable.V2#state#a`
///
/// Gid: `732`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: crate :: bigint :: BigInt
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionAccountStableV2StateA {
    Check(crate::bigint::BigInt),
    Ignore,
}

/// **OCaml name**: `Mina_base__Zkapp_state.Value.Stable.V1`
///
/// Gid: `735`
/// Location: [src/lib/mina_base/zkapp_state.ml:50:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_state.ml#L50)
///
///
/// Gid: `734`
/// Location: [src/lib/mina_base/zkapp_state.ml:17:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_state.ml#L17)
/// Args: crate :: bigint :: BigInt
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappStateValueStableV1(pub PaddedSeq<crate::bigint::BigInt, 8>);

/// **OCaml name**: `Mina_base__Zkapp_account.Stable.V2`
///
/// Gid: `737`
/// Location: [src/lib/mina_base/zkapp_account.ml:218:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_account.ml#L218)
///
///
/// Gid: `736`
/// Location: [src/lib/mina_base/zkapp_account.ml:188:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_account.ml#L188)
/// Args: MinaBaseZkappStateValueStableV1 , Option < MinaBaseVerificationKeyWireStableV1 > , MinaNumbersNatMake32StableV1 , crate :: bigint :: BigInt , UnsignedExtendedUInt32StableV1 , bool , crate :: string :: ByteString
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappAccountStableV2 {
    pub app_state: MinaBaseZkappStateValueStableV1,
    pub verification_key: Option<MinaBaseVerificationKeyWireStableV1>,
    pub zkapp_version: MinaNumbersNatMake32StableV1,
    pub sequence_state: PaddedSeq<crate::bigint::BigInt, 5>,
    pub last_sequence_slot: UnsignedExtendedUInt32StableV1,
    pub proved_state: bool,
    pub zkapp_uri: crate::string::ByteString,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Epoch_data.Stable.V1#epoch_ledger`
///
/// Gid: `747`
/// Location: [src/lib/mina_base/epoch_ledger.ml:9:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/epoch_ledger.ml#L9)
/// Args: MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash , MinaBaseZkappPreconditionProtocolStateStableV1Amount
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochLedger {
    pub hash: MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash,
    pub total_currency: MinaBaseZkappPreconditionProtocolStateStableV1Amount,
}

/// **OCaml name**: `Mina_base__Epoch_ledger.Value.Stable.V1`
///
/// Gid: `748`
/// Location: [src/lib/mina_base/epoch_ledger.ml:23:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/epoch_ledger.ml#L23)
///
///
/// Gid: `747`
/// Location: [src/lib/mina_base/epoch_ledger.ml:9:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/epoch_ledger.ml#L9)
/// Args: MinaBaseLedgerHash0StableV1 , CurrencyAmountStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseEpochLedgerValueStableV1 {
    pub hash: LedgerHash,
    pub total_currency: CurrencyAmountStableV1,
}

/// **OCaml name**: `Mina_base__Epoch_seed.Stable.V1`
///
/// Gid: `751`
/// Location: [src/lib/mina_base/epoch_seed.ml:18:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/epoch_seed.ml#L18)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseEpochSeedStableV1(pub crate::bigint::BigInt);

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1#time#a`
///
/// Gid: `756`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:23:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_precondition.ml#L23)
/// Args: BlockTimeTimeStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionProtocolStateStableV1TimeA {
    pub lower: BlockTimeTimeStableV1,
    pub upper: BlockTimeTimeStableV1,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1#amount#a`
///
/// Gid: `756`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:23:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_precondition.ml#L23)
/// Args: CurrencyAmountStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionProtocolStateStableV1AmountA {
    pub lower: CurrencyAmountStableV1,
    pub upper: CurrencyAmountStableV1,
}

/// Derived name: `Mina_base__Zkapp_precondition.Account.Stable.V2#balance#a`
///
/// Gid: `756`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:23:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_precondition.ml#L23)
/// Args: CurrencyBalanceStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionAccountStableV2BalanceA {
    pub lower: CurrencyBalanceStableV1,
    pub upper: CurrencyBalanceStableV1,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1#length#a`
///
/// Gid: `756`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:23:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_precondition.ml#L23)
/// Args: UnsignedExtendedUInt32StableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionProtocolStateStableV1LengthA {
    pub lower: UnsignedExtendedUInt32StableV1,
    pub upper: UnsignedExtendedUInt32StableV1,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1#time`
///
/// Gid: `757`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:178:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_precondition.ml#L178)
/// Args: BlockTimeTimeStableV1
///
///
/// Gid: `732`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: MinaBaseZkappPreconditionProtocolStateStableV1TimeA
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionProtocolStateStableV1Time {
    Check(MinaBaseZkappPreconditionProtocolStateStableV1TimeA),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1#amount`
///
/// Gid: `757`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:178:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_precondition.ml#L178)
/// Args: CurrencyAmountStableV1
///
///
/// Gid: `732`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: MinaBaseZkappPreconditionProtocolStateStableV1AmountA
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionProtocolStateStableV1Amount {
    Check(MinaBaseZkappPreconditionProtocolStateStableV1AmountA),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Account.Stable.V2#balance`
///
/// Gid: `757`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:178:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_precondition.ml#L178)
/// Args: CurrencyBalanceStableV1
///
///
/// Gid: `732`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: MinaBaseZkappPreconditionAccountStableV2BalanceA
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionAccountStableV2Balance {
    Check(MinaBaseZkappPreconditionAccountStableV2BalanceA),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1#length`
///
/// Gid: `757`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:178:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_precondition.ml#L178)
/// Args: UnsignedExtendedUInt32StableV1
///
///
/// Gid: `732`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: MinaBaseZkappPreconditionProtocolStateStableV1LengthA
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionProtocolStateStableV1Length {
    Check(MinaBaseZkappPreconditionProtocolStateStableV1LengthA),
    Ignore,
}

/// **OCaml name**: `Mina_base__Zkapp_precondition.Account.Stable.V2`
///
/// Gid: `758`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:474:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_precondition.ml#L474)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionAccountStableV2 {
    pub balance: MinaBaseZkappPreconditionAccountStableV2Balance,
    pub nonce: MinaBaseZkappPreconditionProtocolStateStableV1Length,
    pub receipt_chain_hash: MinaBaseZkappPreconditionAccountStableV2ReceiptChainHash,
    pub delegate: MinaBaseZkappPreconditionAccountStableV2Delegate,
    pub state: PaddedSeq<MinaBaseZkappPreconditionAccountStableV2StateA, 8>,
    pub sequence_state: MinaBaseZkappPreconditionAccountStableV2StateA,
    pub proved_state: MinaBaseZkappPreconditionAccountStableV2ProvedState,
    pub is_new: MinaBaseZkappPreconditionAccountStableV2ProvedState,
}

/// **OCaml name**: `Mina_base__Zkapp_precondition.Protocol_state.Epoch_data.Stable.V1`
///
/// Gid: `759`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:790:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_precondition.ml#L790)
///
///
/// Gid: `754`
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
/// Gid: `761`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:970:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_precondition.ml#L970)
///
///
/// Gid: `760`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:921:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_precondition.ml#L921)
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

/// **OCaml name**: `Mina_base__Account_update.Authorization_kind.Stable.V1`
///
/// Gid: `768`
/// Location: [src/lib/mina_base/account_update.ml:27:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_update.ml#L27)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateAuthorizationKindStableV1 {
    NoneGiven,
    Signature,
    Proof,
}

/// **OCaml name**: `Mina_base__Account_update.Call_type.Stable.V1`
///
/// Gid: `769`
/// Location: [src/lib/mina_base/account_update.ml:126:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_update.ml#L126)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateCallTypeStableV1 {
    Call,
    DelegateCall,
}

/// **OCaml name**: `Mina_base__Account_update.Update.Timing_info.Stable.V1`
///
/// Gid: `770`
/// Location: [src/lib/mina_base/account_update.ml:163:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_update.ml#L163)
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
/// Gid: `771`
/// Location: [src/lib/mina_base/account_update.ml:319:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_update.ml#L319)
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
/// Gid: `772`
/// Location: [src/lib/mina_base/account_update.ml:613:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_update.ml#L613)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateAccountPreconditionStableV1 {
    Full(Box<MinaBaseZkappPreconditionAccountStableV2>),
    Nonce(UnsignedExtendedUInt32StableV1),
    Accept,
}

/// **OCaml name**: `Mina_base__Account_update.Preconditions.Stable.V1`
///
/// Gid: `773`
/// Location: [src/lib/mina_base/account_update.ml:758:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_update.ml#L758)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountUpdatePreconditionsStableV1 {
    pub network: MinaBaseZkappPreconditionProtocolStateStableV1,
    pub account: MinaBaseAccountUpdateAccountPreconditionStableV1,
}

/// **OCaml name**: `Mina_base__Account_update.Body.Events'.Stable.V1`
///
/// Gid: `774`
/// Location: [src/lib/mina_base/account_update.ml:834:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_update.ml#L834)
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

/// **OCaml name**: `Mina_base__Account_update.Body.Wire.Stable.V1`
///
/// Gid: `775`
/// Location: [src/lib/mina_base/account_update.ml:846:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_update.ml#L846)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountUpdateBodyWireStableV1 {
    pub public_key: NonZeroCurvePoint,
    pub token_id: TokenIdKeyHash,
    pub update: MinaBaseAccountUpdateUpdateStableV1,
    pub balance_change: MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount,
    pub increment_nonce: bool,
    pub events: MinaBaseAccountUpdateBodyEventsStableV1,
    pub sequence_events: MinaBaseAccountUpdateBodyEventsStableV1,
    pub call_data: crate::bigint::BigInt,
    pub preconditions: MinaBaseAccountUpdatePreconditionsStableV1,
    pub use_full_commitment: bool,
    pub caller: MinaBaseAccountUpdateCallTypeStableV1,
    pub authorization_kind: MinaBaseAccountUpdateAuthorizationKindStableV1,
}

/// **OCaml name**: `Mina_base__Account_update.Body.Fee_payer.Stable.V1`
///
/// Gid: `779`
/// Location: [src/lib/mina_base/account_update.ml:1081:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_update.ml#L1081)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountUpdateBodyFeePayerStableV1 {
    pub public_key: NonZeroCurvePoint,
    pub fee: CurrencyFeeStableV1,
    pub valid_until: Option<UnsignedExtendedUInt32StableV1>,
    pub nonce: UnsignedExtendedUInt32StableV1,
}

/// **OCaml name**: `Mina_base__Account_update.T.Wire.Stable.V1`
///
/// Gid: `782`
/// Location: [src/lib/mina_base/account_update.ml:1410:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_update.ml#L1410)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountUpdateTWireStableV1 {
    pub body: MinaBaseAccountUpdateBodyWireStableV1,
    pub authorization: MinaBaseControlStableV2,
}

/// **OCaml name**: `Mina_base__Account_update.Fee_payer.Stable.V1`
///
/// Gid: `784`
/// Location: [src/lib/mina_base/account_update.ml:1484:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/account_update.ml#L1484)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountUpdateFeePayerStableV1 {
    pub body: MinaBaseAccountUpdateBodyFeePayerStableV1,
    pub authorization: MinaBaseSignatureStableV1,
}

/// Derived name: `Mina_base__Zkapp_command.T.Stable.V1.Wire.Stable.V1#account_updates#a#a#calls#a`
///
/// Gid: `785`
/// Location: [src/lib/mina_base/with_stack_hash.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/with_stack_hash.ml#L6)
/// Args: Box < MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA > , ()
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAACallsA {
    pub elt: Box<MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA>,
    pub stack_hash: (),
}

/// Derived name: `Mina_base__Zkapp_command.T.Stable.V1.Wire.Stable.V1#account_updates#a`
///
/// Gid: `785`
/// Location: [src/lib/mina_base/with_stack_hash.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/with_stack_hash.ml#L6)
/// Args: MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA , ()
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA {
    pub elt: MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA,
    pub stack_hash: (),
}

/// Derived name: `Mina_base__Zkapp_command.T.Stable.V1.Wire.Stable.V1#account_updates#a#a`
///
/// Gid: `786`
/// Location: [src/lib/mina_base/zkapp_command.ml:49:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_command.ml#L49)
/// Args: MinaBaseAccountUpdateTWireStableV1 , () , ()
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA {
    pub account_update: MinaBaseAccountUpdateTWireStableV1,
    pub account_update_digest: (),
    pub calls: Vec<MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAACallsA>,
}

/// **OCaml name**: `Mina_base__Zkapp_command.T.Stable.V1.Wire.Stable.V1`
///
/// Gid: `795`
/// Location: [src/lib/mina_base/zkapp_command.ml:976:12](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/zkapp_command.ml#L976)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappCommandTStableV1WireStableV1 {
    pub fee_payer: MinaBaseAccountUpdateFeePayerStableV1,
    pub account_updates: Vec<MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA>,
    pub memo: MinaBaseSignedCommandMemoStableV1,
}

/// Derived name: `Mina_transaction_logic.Transaction_applied.Coinbase_applied.Stable.V2#coinbase`
///
/// Gid: `803`
/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/with_status.ml#L6)
/// Args: MinaBaseCoinbaseStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedCoinbaseAppliedStableV2Coinbase {
    pub data: MinaBaseCoinbaseStableV1,
    pub status: MinaBaseTransactionStatusStableV2,
}

/// Derived name: `Mina_transaction_logic.Transaction_applied.Fee_transfer_applied.Stable.V2#fee_transfer`
///
/// Gid: `803`
/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/with_status.ml#L6)
/// Args: MinaBaseFeeTransferStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedFeeTransferAppliedStableV2FeeTransfer {
    pub data: MinaBaseFeeTransferStableV2,
    pub status: MinaBaseTransactionStatusStableV2,
}

/// Derived name: `Mina_transaction_logic.Transaction_applied.Signed_command_applied.Common.Stable.V2#user_command`
///
/// Gid: `803`
/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/with_status.ml#L6)
/// Args: MinaBaseSignedCommandStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV2UserCommand {
    pub data: MinaBaseSignedCommandStableV2,
    pub status: MinaBaseTransactionStatusStableV2,
}

/// Derived name: `Staged_ledger_diff__Diff.Make_str.Pre_diff_with_at_most_two_coinbase.Stable.V2#b`
///
/// Gid: `803`
/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/with_status.ml#L6)
/// Args: MinaBaseUserCommandStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B {
    pub data: MinaBaseUserCommandStableV2,
    pub status: MinaBaseTransactionStatusStableV2,
}

/// Derived name: `Mina_transaction_logic.Transaction_applied.Zkapp_command_applied.Stable.V1#command`
///
/// Gid: `803`
/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/with_status.ml#L6)
/// Args: MinaBaseZkappCommandTStableV1WireStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedZkappCommandAppliedStableV1Command {
    pub data: MinaBaseZkappCommandTStableV1WireStableV1,
    pub status: MinaBaseTransactionStatusStableV2,
}

/// **OCaml name**: `Mina_base__User_command.Stable.V2`
///
/// Gid: `806`
/// Location: [src/lib/mina_base/user_command.ml:79:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/user_command.ml#L79)
///
///
/// Gid: `804`
/// Location: [src/lib/mina_base/user_command.ml:7:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/user_command.ml#L7)
/// Args: MinaBaseSignedCommandStableV2 , MinaBaseZkappCommandTStableV1WireStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseUserCommandStableV2 {
    SignedCommand(MinaBaseSignedCommandStableV2),
    ZkappCommand(MinaBaseZkappCommandTStableV1WireStableV1),
}

/// **OCaml name**: `Mina_base__Fee_transfer.Make_str.Single.Stable.V2`
///
/// Gid: `810`
/// Location: [src/lib/mina_base/fee_transfer.ml:19:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/fee_transfer.ml#L19)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeTransferSingleStableV2 {
    pub receiver_pk: NonZeroCurvePoint,
    pub fee: CurrencyFeeStableV1,
    pub fee_token: MinaBaseTokenIdStableV2,
}

/// **OCaml name**: `Mina_base__Fee_transfer.Make_str.Stable.V2`
///
/// Gid: `811`
/// Location: [src/lib/mina_base/fee_transfer.ml:68:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/fee_transfer.ml#L68)
///
///
/// Gid: `629`
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
/// Gid: `812`
/// Location: [src/lib/mina_base/coinbase_fee_transfer.ml:15:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/coinbase_fee_transfer.ml#L15)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseCoinbaseFeeTransferStableV1 {
    pub receiver_pk: NonZeroCurvePoint,
    pub fee: CurrencyFeeStableV1,
}

/// **OCaml name**: `Mina_base__Coinbase.Make_str.Stable.V1`
///
/// Gid: `813`
/// Location: [src/lib/mina_base/coinbase.ml:17:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/coinbase.ml#L17)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseCoinbaseStableV1 {
    pub receiver: NonZeroCurvePoint,
    pub amount: CurrencyAmountStableV1,
    pub fee_transfer: Option<MinaBaseCoinbaseFeeTransferStableV1>,
}

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.Stack_id.Stable.V1`
///
/// Gid: `815`
/// Location: [src/lib/mina_base/pending_coinbase.ml:110:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/pending_coinbase.ml#L110)
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
/// Gid: `818`
/// Location: [src/lib/mina_base/pending_coinbase.ml:163:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/pending_coinbase.ml#L163)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseCoinbaseStackStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.Stack_hash.Stable.V1`
///
/// Gid: `823`
/// Location: [src/lib/mina_base/pending_coinbase.ml:223:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/pending_coinbase.ml#L223)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStackHashStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.State_stack.Stable.V1`
///
/// Gid: `827`
/// Location: [src/lib/mina_base/pending_coinbase.ml:259:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/pending_coinbase.ml#L259)
///
///
/// Gid: `826`
/// Location: [src/lib/mina_base/pending_coinbase.ml:249:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/pending_coinbase.ml#L249)
/// Args: MinaBasePendingCoinbaseStackHashStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStateStackStableV1 {
    pub init: MinaBasePendingCoinbaseStackHashStableV1,
    pub curr: MinaBasePendingCoinbaseStackHashStableV1,
}

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.Hash_builder.Stable.V1`
///
/// Gid: `830`
/// Location: [src/lib/mina_base/pending_coinbase.ml:370:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/pending_coinbase.ml#L370)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseHashBuilderStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.Stack_versioned.Stable.V1`
///
/// Gid: `837`
/// Location: [src/lib/mina_base/pending_coinbase.ml:519:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/pending_coinbase.ml#L519)
///
///
/// Gid: `836`
/// Location: [src/lib/mina_base/pending_coinbase.ml:508:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/pending_coinbase.ml#L508)
/// Args: MinaBasePendingCoinbaseCoinbaseStackStableV1 , MinaBasePendingCoinbaseStateStackStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStackVersionedStableV1 {
    pub data: MinaBasePendingCoinbaseCoinbaseStackStableV1,
    pub state: MinaBasePendingCoinbaseStateStackStableV1,
}

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.Hash_versioned.Stable.V1`
///
/// Gid: `838`
/// Location: [src/lib/mina_base/pending_coinbase.ml:532:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/pending_coinbase.ml#L532)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseHashVersionedStableV1(
    pub MinaBasePendingCoinbaseHashBuilderStableV1,
);

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.Merkle_tree_versioned.Stable.V2`
///
/// Gid: `839`
/// Location: [src/lib/mina_base/pending_coinbase.ml:544:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/pending_coinbase.ml#L544)
///
///
/// Gid: `628`
/// Location: [src/lib/sparse_ledger_lib/sparse_ledger.ml:38:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/sparse_ledger_lib/sparse_ledger.ml#L38)
/// Args: MinaBasePendingCoinbaseHashVersionedStableV1 , MinaBasePendingCoinbaseStackIdStableV1 , MinaBasePendingCoinbaseStackVersionedStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseMerkleTreeVersionedStableV2 {
    pub indexes: Vec<(MinaBasePendingCoinbaseStackIdStableV1, crate::number::Int32)>,
    pub depth: crate::number::Int32,
    pub tree: MinaBasePendingCoinbaseMerkleTreeVersionedStableV2Tree,
}

/// **OCaml name**: `Mina_base__Staged_ledger_hash.Make_str.Aux_hash.Stable.V1`
///
/// Gid: `842`
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
/// Gid: `843`
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
/// Gid: `844`
/// Location: [src/lib/mina_base/staged_ledger_hash.ml:152:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/staged_ledger_hash.ml#L152)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashNonSnarkStableV1 {
    pub ledger_hash: LedgerHash,
    pub aux_hash: StagedLedgerHashAuxHash,
    pub pending_coinbase_aux: StagedLedgerHashPendingCoinbaseAux,
}

/// **OCaml name**: `Mina_base__Staged_ledger_hash.Make_str.Stable.V1`
///
/// Gid: `846`
/// Location: [src/lib/mina_base/staged_ledger_hash.ml:259:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/staged_ledger_hash.ml#L259)
///
///
/// Gid: `845`
/// Location: [src/lib/mina_base/staged_ledger_hash.ml:241:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/staged_ledger_hash.ml#L241)
/// Args: MinaBaseStagedLedgerHashNonSnarkStableV1 , MinaBasePendingCoinbaseHashVersionedStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashStableV1 {
    pub non_snark: MinaBaseStagedLedgerHashNonSnarkStableV1,
    pub pending_coinbase_hash: PendingCoinbaseHash,
}

/// **OCaml name**: `Mina_base__Stack_frame.Make_str.Stable.V1`
///
/// Gid: `848`
/// Location: [src/lib/mina_base/stack_frame.ml:64:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/stack_frame.ml#L64)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStackFrameStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Sok_message.Make_str.Stable.V1`
///
/// Gid: `850`
/// Location: [src/lib/mina_base/sok_message.ml:14:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/sok_message.ml#L14)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSokMessageStableV1 {
    pub fee: CurrencyFeeStableV1,
    pub prover: NonZeroCurvePoint,
}

/// **OCaml name**: `Mina_base__Protocol_constants_checked.Value.Stable.V1`
///
/// Gid: `851`
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
/// Gid: `852`
/// Location: [src/lib/mina_base/proof.ml:12:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/proof.ml#L12)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseProofStableV2(pub PicklesProofProofsVerified2ReprStableV2);

/// **OCaml name**: `Mina_base__Call_stack_digest.Make_str.Stable.V1`
///
/// Gid: `854`
/// Location: [src/lib/mina_base/call_stack_digest.ml:12:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/call_stack_digest.ml#L12)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseCallStackDigestStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Fee_with_prover.Stable.V1`
///
/// Gid: `855`
/// Location: [src/lib/mina_base/fee_with_prover.ml:7:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/fee_with_prover.ml#L7)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeWithProverStableV1 {
    pub fee: CurrencyFeeStableV1,
    pub prover: NonZeroCurvePoint,
}

/// **OCaml name**: `Mina_transaction_logic__Zkapp_command_logic.Local_state.Value.Stable.V1`
///
/// Gid: `867`
/// Location: [src/lib/transaction_logic/zkapp_command_logic.ml:235:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_logic/zkapp_command_logic.ml#L235)
///
///
/// Gid: `866`
/// Location: [src/lib/transaction_logic/zkapp_command_logic.ml:174:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_logic/zkapp_command_logic.ml#L174)
/// Args: MinaBaseStackFrameStableV1 , MinaBaseCallStackDigestStableV1 , MinaBaseTokenIdStableV2 , MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount , MinaBaseLedgerHash0StableV1 , bool , crate :: bigint :: BigInt , UnsignedExtendedUInt32StableV1 , MinaBaseTransactionStatusFailureCollectionStableV1
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
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Signed_command_applied.Common.Stable.V2`
///
/// Gid: `869`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:17:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_logic/mina_transaction_logic.ml#L17)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV2 {
    pub user_command:
        MinaTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV2UserCommand,
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Signed_command_applied.Body.Stable.V2`
///
/// Gid: `870`
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
/// Gid: `871`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:46:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_logic/mina_transaction_logic.ml#L46)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedSignedCommandAppliedStableV2 {
    pub common: MinaTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV2,
    pub body: MinaTransactionLogicTransactionAppliedSignedCommandAppliedBodyStableV2,
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Zkapp_command_applied.Stable.V1`
///
/// Gid: `872`
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
/// Gid: `873`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:82:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_logic/mina_transaction_logic.ml#L82)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaTransactionLogicTransactionAppliedCommandAppliedStableV2 {
    SignedCommand(MinaTransactionLogicTransactionAppliedSignedCommandAppliedStableV2),
    ZkappCommand(MinaTransactionLogicTransactionAppliedZkappCommandAppliedStableV1),
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Fee_transfer_applied.Stable.V2`
///
/// Gid: `874`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:96:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_logic/mina_transaction_logic.ml#L96)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedFeeTransferAppliedStableV2 {
    pub fee_transfer: MinaTransactionLogicTransactionAppliedFeeTransferAppliedStableV2FeeTransfer,
    pub new_accounts: Vec<MinaBaseAccountIdStableV2>,
    pub burned_tokens: CurrencyAmountStableV1,
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Coinbase_applied.Stable.V2`
///
/// Gid: `875`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:112:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_logic/mina_transaction_logic.ml#L112)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedCoinbaseAppliedStableV2 {
    pub coinbase: MinaTransactionLogicTransactionAppliedCoinbaseAppliedStableV2Coinbase,
    pub new_accounts: Vec<MinaBaseAccountIdStableV2>,
    pub burned_tokens: CurrencyAmountStableV1,
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Varying.Stable.V2`
///
/// Gid: `876`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:128:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_logic/mina_transaction_logic.ml#L128)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaTransactionLogicTransactionAppliedVaryingStableV2 {
    Command(MinaTransactionLogicTransactionAppliedCommandAppliedStableV2),
    FeeTransfer(MinaTransactionLogicTransactionAppliedFeeTransferAppliedStableV2),
    Coinbase(MinaTransactionLogicTransactionAppliedCoinbaseAppliedStableV2),
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Stable.V2`
///
/// Gid: `877`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:142:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_logic/mina_transaction_logic.ml#L142)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedStableV2 {
    pub previous_hash: LedgerHash,
    pub varying: MinaTransactionLogicTransactionAppliedVaryingStableV2,
}

/// **OCaml name**: `Merkle_address.Binable_arg.Stable.V1`
///
/// Gid: `878`
/// Location: [src/lib/merkle_address/merkle_address.ml:48:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/merkle_address/merkle_address.ml#L48)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MerkleAddressBinableArgStableV1(pub crate::number::Int32, pub crate::string::ByteString);

/// **OCaml name**: `Network_peer__Peer.Id.Stable.V1`
///
/// Gid: `884`
/// Location: [src/lib/network_peer/peer.ml:10:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/network_peer/peer.ml#L10)
///
///
/// Gid: `170`
/// Location: [src/std_internal.ml:140:2](https://github.com/Minaprotocol/mina/blob/32a9161/src/std_internal.ml#L140)
///
///
/// Gid: `83`
/// Location: [src/string.ml:44:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/string.ml#L44)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NetworkPeerPeerIdStableV1(pub crate::string::ByteString);

/// **OCaml name**: `Trust_system__Banned_status.Stable.V1`
///
/// Gid: `889`
/// Location: [src/lib/trust_system/banned_status.ml:6:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/trust_system/banned_status.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum TrustSystemBannedStatusStableV1 {
    Unbanned,
    BannedUntil(crate::number::Float64),
}

/// **OCaml name**: `Consensus_vrf.Output.Truncated.Stable.V1`
///
/// Gid: `904`
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
/// Gid: `922`
/// Location: [src/lib/consensus/body_reference.ml:17:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/consensus/body_reference.ml#L17)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusBodyReferenceStableV1(pub Blake2MakeStableV1);

/// **OCaml name**: `Consensus__Global_slot.Make_str.Stable.V1`
///
/// Gid: `929`
/// Location: [src/lib/consensus/global_slot.ml:33:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/consensus/global_slot.ml#L33)
///
///
/// Gid: `928`
/// Location: [src/lib/consensus/global_slot.ml:22:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/consensus/global_slot.ml#L22)
/// Args: UnsignedExtendedUInt32StableV1 , UnsignedExtendedUInt32StableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusGlobalSlotStableV1 {
    pub slot_number: UnsignedExtendedUInt32StableV1,
    pub slots_per_epoch: UnsignedExtendedUInt32StableV1,
}

/// **OCaml name**: `Consensus__Proof_of_stake.Make_str.Data.Epoch_data.Staking_value_versioned.Value.Stable.V1`
///
/// Gid: `939`
/// Location: [src/lib/consensus/proof_of_stake.ml:1079:14](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/consensus/proof_of_stake.ml#L1079)
///
///
/// Gid: `754`
/// Location: [src/lib/mina_base/epoch_data.ml:8:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/epoch_data.ml#L8)
/// Args: MinaBaseEpochLedgerValueStableV1 , MinaBaseEpochSeedStableV1 , DataHashLibStateHashStableV1 , DataHashLibStateHashStableV1 , UnsignedExtendedUInt32StableV1
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
/// Gid: `940`
/// Location: [src/lib/consensus/proof_of_stake.ml:1103:14](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/consensus/proof_of_stake.ml#L1103)
///
///
/// Gid: `754`
/// Location: [src/lib/mina_base/epoch_data.ml:8:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_base/epoch_data.ml#L8)
/// Args: MinaBaseEpochLedgerValueStableV1 , MinaBaseEpochSeedStableV1 , DataHashLibStateHashStableV1 , DataHashLibStateHashStableV1 , UnsignedExtendedUInt32StableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1 {
    pub ledger: MinaBaseEpochLedgerValueStableV1,
    pub seed: EpochSeed,
    pub start_checkpoint: StateHash,
    pub lock_checkpoint: StateHash,
    pub epoch_length: UnsignedExtendedUInt32StableV1,
}

/// Derived name: `Mina_state__Blockchain_state.Value.Stable.V2#registers`
///
/// Gid: `945`
/// Location: [src/lib/mina_state/registers.ml:8:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_state/registers.ml#L8)
/// Args: MinaBaseLedgerHash0StableV1 , () , MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateBlockchainStateValueStableV2Registers {
    pub ledger: LedgerHash,
    pub pending_coinbase_stack: (),
    pub local_state: MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1,
}

/// Derived name: `Transaction_snark.Make_str.Statement.With_sok.Stable.V2#source`
///
/// Gid: `945`
/// Location: [src/lib/mina_state/registers.ml:8:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_state/registers.ml#L8)
/// Args: MinaBaseLedgerHash0StableV1 , MinaBasePendingCoinbaseStackVersionedStableV1 , MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkStatementWithSokStableV2Source {
    pub ledger: LedgerHash,
    pub pending_coinbase_stack: MinaBasePendingCoinbaseStackVersionedStableV1,
    pub local_state: MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1,
}

/// **OCaml name**: `Mina_state__Blockchain_state.Value.Stable.V2`
///
/// Gid: `947`
/// Location: [src/lib/mina_state/blockchain_state.ml:49:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_state/blockchain_state.ml#L49)
///
///
/// Gid: `946`
/// Location: [src/lib/mina_state/blockchain_state.ml:9:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_state/blockchain_state.ml#L9)
/// Args: MinaBaseStagedLedgerHashStableV1 , MinaBaseLedgerHash0StableV1 , MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1 , BlockTimeTimeStableV1 , ConsensusBodyReferenceStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateBlockchainStateValueStableV2 {
    pub staged_ledger_hash: MinaBaseStagedLedgerHashStableV1,
    pub genesis_ledger_hash: LedgerHash,
    pub registers: MinaStateBlockchainStateValueStableV2Registers,
    pub timestamp: BlockTimeTimeStableV1,
    pub body_reference: ConsensusBodyReferenceStableV1,
}

/// **OCaml name**: `Mina_state__Protocol_state.Make_str.Body.Value.Stable.V2`
///
/// Gid: `953`
/// Location: [src/lib/mina_state/protocol_state.ml:82:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_state/protocol_state.ml#L82)
///
///
/// Gid: `951`
/// Location: [src/lib/mina_state/protocol_state.ml:62:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_state/protocol_state.ml#L62)
/// Args: DataHashLibStateHashStableV1 , MinaStateBlockchainStateValueStableV2 , ConsensusProofOfStakeDataConsensusStateValueStableV1 , MinaBaseProtocolConstantsCheckedValueStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateProtocolStateBodyValueStableV2 {
    pub genesis_state_hash: StateHash,
    pub blockchain_state: MinaStateBlockchainStateValueStableV2,
    pub consensus_state: ConsensusProofOfStakeDataConsensusStateValueStableV1,
    pub constants: MinaBaseProtocolConstantsCheckedValueStableV1,
}

/// **OCaml name**: `Transaction_snark.Make_str.Pending_coinbase_stack_state.Init_stack.Stable.V1`
///
/// Gid: `982`
/// Location: [src/lib/transaction_snark/transaction_snark.ml:79:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_snark/transaction_snark.ml#L79)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum TransactionSnarkPendingCoinbaseStackStateInitStackStableV1 {
    Base(MinaBasePendingCoinbaseStackVersionedStableV1),
    Merge,
}

/// **OCaml name**: `Transaction_snark.Make_str.Statement.Stable.V2`
///
/// Gid: `988`
/// Location: [src/lib/transaction_snark/transaction_snark.ml:239:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_snark/transaction_snark.ml#L239)
///
///
/// Gid: `987`
/// Location: [src/lib/transaction_snark/transaction_snark.ml:149:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_snark/transaction_snark.ml#L149)
/// Args: MinaBaseLedgerHash0StableV1 , MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount , MinaBasePendingCoinbaseStackVersionedStableV1 , MinaBaseFeeExcessStableV1 , () , MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkStatementStableV2 {
    pub source: TransactionSnarkStatementWithSokStableV2Source,
    pub target: TransactionSnarkStatementWithSokStableV2Source,
    pub supply_increase: MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount,
    pub fee_excess: MinaBaseFeeExcessStableV1,
    pub sok_digest: (),
}

/// **OCaml name**: `Transaction_snark.Make_str.Statement.With_sok.Stable.V2`
///
/// Gid: `989`
/// Location: [src/lib/transaction_snark/transaction_snark.ml:257:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_snark/transaction_snark.ml#L257)
///
///
/// Gid: `987`
/// Location: [src/lib/transaction_snark/transaction_snark.ml:149:10](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_snark/transaction_snark.ml#L149)
/// Args: MinaBaseLedgerHash0StableV1 , MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount , MinaBasePendingCoinbaseStackVersionedStableV1 , MinaBaseFeeExcessStableV1 , MinaBaseSokMessageDigestStableV1 , MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkStatementWithSokStableV2 {
    pub source: TransactionSnarkStatementWithSokStableV2Source,
    pub target: TransactionSnarkStatementWithSokStableV2Source,
    pub supply_increase: MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount,
    pub fee_excess: MinaBaseFeeExcessStableV1,
    pub sok_digest: MinaBaseSokMessageDigestStableV1,
}

/// **OCaml name**: `Transaction_snark.Make_str.Proof.Stable.V2`
///
/// Gid: `992`
/// Location: [src/lib/transaction_snark/transaction_snark.ml:413:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_snark/transaction_snark.ml#L413)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkProofStableV2(pub PicklesProofProofsVerified2ReprStableV2);

/// **OCaml name**: `Transaction_snark.Make_str.Stable.V2`
///
/// Gid: `993`
/// Location: [src/lib/transaction_snark/transaction_snark.ml:424:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_snark/transaction_snark.ml#L424)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkStableV2 {
    pub statement: TransactionSnarkStatementWithSokStableV2,
    pub proof: TransactionSnarkProofStableV2,
}

/// **OCaml name**: `Ledger_proof.Prod.Stable.V2`
///
/// Gid: `996`
/// Location: [src/lib/ledger_proof/ledger_proof.ml:10:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/ledger_proof/ledger_proof.ml#L10)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct LedgerProofProdStableV2(pub TransactionSnarkStableV2);

/// **OCaml name**: `Transaction_snark_work.Statement.Stable.V2`
///
/// Gid: `998`
/// Location: [src/lib/transaction_snark_work/transaction_snark_work.ml:23:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_snark_work/transaction_snark_work.ml#L23)
///
///
/// Gid: `629`
/// Location: [src/lib/one_or_two/one_or_two.ml:7:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/one_or_two/one_or_two.ml#L7)
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
/// Gid: `1002`
/// Location: [src/lib/transaction_snark_work/transaction_snark_work.ml:82:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_snark_work/transaction_snark_work.ml#L82)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkWorkTStableV2 {
    pub fee: CurrencyFeeStableV1,
    pub proofs: TransactionSnarkWorkTStableV2Proofs,
    pub prover: NonZeroCurvePoint,
}

/// Derived name: `Staged_ledger_diff__Diff.Make_str.Pre_diff_with_at_most_two_coinbase.Stable.V2#coinbase`
///
/// Gid: `1003`
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

/// Derived name: `Staged_ledger_diff__Diff.Make_str.Pre_diff_with_at_most_one_coinbase.Stable.V2#coinbase`
///
/// Gid: `1004`
/// Location: [src/lib/staged_ledger_diff/diff.ml:64:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/staged_ledger_diff/diff.ml#L64)
/// Args: StagedLedgerDiffDiffFtStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2Coinbase {
    Zero,
    One(Option<StagedLedgerDiffDiffFtStableV1>),
}

/// **OCaml name**: `Staged_ledger_diff__Diff.Make_str.Ft.Stable.V1`
///
/// Gid: `1005`
/// Location: [src/lib/staged_ledger_diff/diff.ml:88:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/staged_ledger_diff/diff.ml#L88)
#[derive(Clone, Debug, Deref, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffFtStableV1(pub MinaBaseCoinbaseFeeTransferStableV1);

/// **OCaml name**: `Staged_ledger_diff__Diff.Make_str.Pre_diff_with_at_most_two_coinbase.Stable.V2`
///
/// Gid: `1008`
/// Location: [src/lib/staged_ledger_diff/diff.ml:168:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/staged_ledger_diff/diff.ml#L168)
///
///
/// Gid: `1006`
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
/// Gid: `1009`
/// Location: [src/lib/staged_ledger_diff/diff.ml:187:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/staged_ledger_diff/diff.ml#L187)
///
///
/// Gid: `1007`
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
/// Gid: `1010`
/// Location: [src/lib/staged_ledger_diff/diff.ml:206:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/staged_ledger_diff/diff.ml#L206)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffDiffStableV2(
    pub StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2,
    pub Option<StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2>,
);

/// **OCaml name**: `Staged_ledger_diff__Diff.Make_str.Stable.V2`
///
/// Gid: `1011`
/// Location: [src/lib/staged_ledger_diff/diff.ml:223:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/staged_ledger_diff/diff.ml#L223)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffStableV2 {
    pub diff: StagedLedgerDiffDiffDiffStableV2,
}

/// **OCaml name**: `Staged_ledger_diff__Body.Make_str.Stable.V1`
///
/// Gid: `1012`
/// Location: [src/lib/staged_ledger_diff/body.ml:18:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/staged_ledger_diff/body.ml#L18)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffBodyStableV1 {
    pub staged_ledger_diff: StagedLedgerDiffDiffStableV2,
}

/// **OCaml name**: `Parallel_scan.Sequence_number.Stable.V1`
///
/// Gid: `1017`
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
/// Gid: `1018`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:35:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/parallel_scan/parallel_scan.ml#L35)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum ParallelScanJobStatusStableV1 {
    Todo,
    Done,
}

/// **OCaml name**: `Parallel_scan.Weight.Stable.V1`
///
/// Gid: `1019`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:53:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/parallel_scan/parallel_scan.ml#L53)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ParallelScanWeightStableV1 {
    pub base: crate::number::Int32,
    pub merge: crate::number::Int32,
}

/// Derived name: `Transaction_snark_scan_state.Stable.V2#trees#a#base_t#1#Full`
///
/// Gid: `1020`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:68:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/parallel_scan/parallel_scan.ml#L68)
/// Args: TransactionSnarkScanStateTransactionWithWitnessStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateStableV2TreesABaseT1Full {
    pub job: TransactionSnarkScanStateTransactionWithWitnessStableV2,
    pub seq_no: ParallelScanSequenceNumberStableV1,
    pub status: ParallelScanJobStatusStableV1,
}

/// Derived name: `Transaction_snark_scan_state.Stable.V2#trees#a#base_t#1`
///
/// Gid: `1021`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:84:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/parallel_scan/parallel_scan.ml#L84)
/// Args: TransactionSnarkScanStateTransactionWithWitnessStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum TransactionSnarkScanStateStableV2TreesABaseT1 {
    Empty,
    Full(Box<TransactionSnarkScanStateStableV2TreesABaseT1Full>),
}

/// Derived name: `Transaction_snark_scan_state.Stable.V2#trees#a#merge_t#1#Full`
///
/// Gid: `1023`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:112:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/parallel_scan/parallel_scan.ml#L112)
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
/// Gid: `1024`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:130:8](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/parallel_scan/parallel_scan.ml#L130)
/// Args: TransactionSnarkScanStateLedgerProofWithSokMessageStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum TransactionSnarkScanStateStableV2TreesAMergeT1 {
    Empty,
    Part(Box<TransactionSnarkScanStateLedgerProofWithSokMessageStableV2>),
    Full(Box<TransactionSnarkScanStateStableV2TreesAMergeT1Full>),
}

/// **OCaml name**: `Transaction_snark_scan_state.Transaction_with_witness.Stable.V2`
///
/// Gid: `1032`
/// Location: [src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml:40:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml#L40)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateTransactionWithWitnessStableV2 {
    pub transaction_with_info: MinaTransactionLogicTransactionAppliedStableV2,
    pub state_hash: (StateHash, MinaBaseStateBodyHashStableV1),
    pub statement: TransactionSnarkStatementStableV2,
    pub init_stack: TransactionSnarkPendingCoinbaseStackStateInitStackStableV1,
    pub ledger_witness: MinaBaseSparseLedgerBaseStableV2,
}

/// **OCaml name**: `Transaction_snark_scan_state.Ledger_proof_with_sok_message.Stable.V2`
///
/// Gid: `1033`
/// Location: [src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml:61:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml#L61)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateLedgerProofWithSokMessageStableV2(
    pub LedgerProofProdStableV2,
    pub MinaBaseSokMessageStableV1,
);

/// **OCaml name**: `Protocol_version.Make_str.Stable.V1`
///
/// Gid: `1053`
/// Location: [src/lib/protocol_version/protocol_version.ml:14:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/protocol_version/protocol_version.ml#L14)
#[derive(Clone, Debug, PartialEq, BinProtRead, BinProtWrite)]
pub struct ProtocolVersionStableV1 {
    pub major: crate::number::Int32,
    pub minor: crate::number::Int32,
    pub patch: crate::number::Int32,
}

/// **OCaml name**: `Mina_block__Header.Make_str.Stable.V2`
///
/// Gid: `1056`
/// Location: [src/lib/mina_block/header.ml:21:6](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/mina_block/header.ml#L21)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBlockHeaderStableV2 {
    pub protocol_state: MinaStateProtocolStateValueStableV2,
    pub protocol_state_proof: MinaBaseProofStableV2,
    pub delta_block_chain_proof: (StateHash, Vec<MinaBaseStateBodyHashStableV1>),
    pub current_protocol_version: ProtocolVersionStableV1,
    pub proposed_protocol_version_opt: Option<ProtocolVersionStableV1>,
}

/// Derived name: `Network_pool__Snark_pool.Diff_versioned.Stable.V2#Add_solved_work#1`
///
/// Gid: `1083`
/// Location: [src/lib/network_pool/priced_proof.ml:9:4](https://github.com/Minaprotocol/mina/blob/32a9161/src/lib/network_pool/priced_proof.ml#L9)
/// Args: TransactionSnarkWorkTStableV2Proofs
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NetworkPoolSnarkPoolDiffVersionedStableV2AddSolvedWork1 {
    pub proof: TransactionSnarkWorkTStableV2Proofs,
    pub fee: MinaBaseFeeWithProverStableV1,
}
