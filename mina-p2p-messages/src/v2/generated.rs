use binprot_derive::{BinProtRead, BinProtWrite};
use derive_more::Deref;
use serde::{Deserialize, Serialize};

use crate::pseq::PaddedSeq;

use super::manual::*;

/// **OCaml name**: `Mina_block__Block.Stable.V2`
///
/// Gid: `1102`
/// Location: [src/lib/mina_block/block.ml:8:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_block/block.ml#L8)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBlockBlockStableV2 {
    pub header: MinaBlockHeaderStableV2,
    pub body: StagedLedgerDiffBodyStableV1,
}

/// **OCaml name**: `Network_pool__Transaction_pool.Diff_versioned.Stable.V2`
///
/// Gid: `1122`
/// Location: [src/lib/network_pool/transaction_pool.ml:47:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/network_pool/transaction_pool.ml#L47)
///
///
/// Gid: `167`
/// Location: [src/std_internal.ml:131:2](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/std_internal.ml#L131)
/// Args: MinaBaseUserCommandStableV2
///
///
/// Gid: `50`
/// Location: [src/list0.ml:6:0](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/list0.ml#L6)
/// Args: MinaBaseUserCommandStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct NetworkPoolTransactionPoolDiffVersionedStableV2(pub Vec<MinaBaseUserCommandStableV2>);

/// **OCaml name**: `Network_pool__Snark_pool.Diff_versioned.Stable.V2`
///
/// Gid: `1126`
/// Location: [src/lib/network_pool/snark_pool.ml:542:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/network_pool/snark_pool.ml#L542)
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
/// Gid: `882`
/// Location: [src/lib/mina_base/sparse_ledger_base.ml:8:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/sparse_ledger_base.ml#L8)
///
///
/// Gid: `661`
/// Location: [src/lib/sparse_ledger_lib/sparse_ledger.ml:38:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/sparse_ledger_lib/sparse_ledger.ml#L38)
/// Args: LedgerHash , MinaBaseAccountIdStableV2 , MinaBaseAccountBinableArgStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSparseLedgerBaseStableV2 {
    pub indexes: Vec<(MinaBaseAccountIdStableV2, crate::number::UInt64)>,
    pub depth: crate::number::UInt64,
    pub tree: MinaBaseSparseLedgerBaseStableV2Tree,
}

/// **OCaml name**: `Mina_base__Account.Binable_arg.Stable.V2`
///
/// Gid: `780`
/// Location: [src/lib/mina_base/account.ml:265:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/account.ml#L265)
///
///
/// Gid: `778`
/// Location: [src/lib/mina_base/account.ml:210:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/account.ml#L210)
/// Args: NonZeroCurvePoint , TokenIdKeyHash , crate :: string :: ByteString , CurrencyBalanceStableV1 , UnsignedExtendedUInt32StableV1 , MinaBaseReceiptChainHashStableV1 , Option < NonZeroCurvePoint > , StateHash , MinaBaseAccountTimingStableV2 , MinaBasePermissionsStableV2 , Option < MinaBaseZkappAccountStableV2 >
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountBinableArgStableV2 {
    pub public_key: NonZeroCurvePoint,
    pub token_id: TokenIdKeyHash,
    pub token_symbol: crate::string::ByteString,
    pub balance: CurrencyBalanceStableV1,
    pub nonce: UnsignedExtendedUInt32StableV1,
    pub receipt_chain_hash: MinaBaseReceiptChainHashStableV1,
    pub delegate: Option<NonZeroCurvePoint>,
    pub voting_for: StateHash,
    pub timing: MinaBaseAccountTimingStableV2,
    pub permissions: MinaBasePermissionsStableV2,
    pub zkapp: Option<MinaBaseZkappAccountStableV2>,
}

/// **OCaml name**: `Network_peer__Peer.Stable.V1`
///
/// Gid: `890`
/// Location: [src/lib/network_peer/peer.ml:56:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/network_peer/peer.ml#L56)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NetworkPeerPeerStableV1 {
    pub host: crate::string::ByteString,
    pub libp2p_port: crate::number::UInt64,
    pub peer_id: NetworkPeerPeerIdStableV1,
}

/// **OCaml name**: `Transaction_snark_scan_state.Stable.V2`
///
/// Gid: `1058`
/// Location: [src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml:160:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml#L160)
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
/// Gid: `874`
/// Location: [src/lib/mina_base/pending_coinbase.ml:1281:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/pending_coinbase.ml#L1281)
///
///
/// Gid: `873`
/// Location: [src/lib/mina_base/pending_coinbase.ml:1269:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/pending_coinbase.ml#L1269)
/// Args: MinaBasePendingCoinbaseMerkleTreeVersionedStableV2 , MinaBasePendingCoinbaseStackIdStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStableV2 {
    pub tree: MinaBasePendingCoinbaseMerkleTreeVersionedStableV2,
    pub pos_list: Vec<MinaBasePendingCoinbaseStackIdStableV1>,
    pub new_pos: MinaBasePendingCoinbaseStackIdStableV1,
}

/// **OCaml name**: `Mina_state__Protocol_state.Make_str.Value.Stable.V2`
///
/// Gid: `1004`
/// Location: [src/lib/mina_state/protocol_state.ml:203:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_state/protocol_state.ml#L203)
///
///
/// Gid: `1000`
/// Location: [src/lib/mina_state/protocol_state.ml:38:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_state/protocol_state.ml#L38)
/// Args: StateHash , MinaStateProtocolStateBodyValueStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateProtocolStateValueStableV2 {
    pub previous_state_hash: StateHash,
    pub body: MinaStateProtocolStateBodyValueStableV2,
}

/// **OCaml name**: `Mina_ledger__Sync_ledger.Query.Stable.V1`
///
/// Gid: `940`
/// Location: [src/lib/mina_ledger/sync_ledger.ml:83:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_ledger/sync_ledger.ml#L83)
///
///
/// Gid: `927`
/// Location: [src/lib/syncable_ledger/syncable_ledger.ml:17:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/syncable_ledger/syncable_ledger.ml#L17)
/// Args: MerkleAddressBinableArgStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaLedgerSyncLedgerQueryStableV1 {
    WhatChildHashes(MerkleAddressBinableArgStableV1),
    WhatContents(MerkleAddressBinableArgStableV1),
    NumAccounts,
}

/// **OCaml name**: `Mina_ledger__Sync_ledger.Answer.Stable.V2`
///
/// Gid: `939`
/// Location: [src/lib/mina_ledger/sync_ledger.ml:58:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_ledger/sync_ledger.ml#L58)
///
///
/// Gid: `928`
/// Location: [src/lib/syncable_ledger/syncable_ledger.ml:35:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/syncable_ledger/syncable_ledger.ml#L35)
/// Args: LedgerHash , MinaBaseAccountBinableArgStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaLedgerSyncLedgerAnswerStableV2 {
    ChildHashesAre(LedgerHash, LedgerHash),
    ContentsAre(Vec<MinaBaseAccountBinableArgStableV2>),
    NumAccounts(crate::number::UInt64, LedgerHash),
}

/// **OCaml name**: `Consensus__Proof_of_stake.Make_str.Data.Consensus_state.Value.Stable.V2`
///
/// Gid: `984`
/// Location: [src/lib/consensus/proof_of_stake.ml:1768:12](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/consensus/proof_of_stake.ml#L1768)
///
///
/// Gid: `983`
/// Location: [src/lib/consensus/proof_of_stake.ml:1723:12](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/consensus/proof_of_stake.ml#L1723)
/// Args: UnsignedExtendedUInt32StableV1 , ConsensusVrfOutputTruncatedStableV1 , CurrencyAmountStableV1 , ConsensusGlobalSlotStableV1 , MinaNumbersGlobalSlotSinceGenesisMStableV1 , ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1 , ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1 , bool , NonZeroCurvePoint
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusProofOfStakeDataConsensusStateValueStableV2 {
    pub blockchain_length: UnsignedExtendedUInt32StableV1,
    pub epoch_count: UnsignedExtendedUInt32StableV1,
    pub min_window_density: UnsignedExtendedUInt32StableV1,
    pub sub_window_densities: Vec<UnsignedExtendedUInt32StableV1>,
    pub last_vrf_output: ConsensusVrfOutputTruncatedStableV1,
    pub total_currency: CurrencyAmountStableV1,
    pub curr_global_slot_since_hard_fork: ConsensusGlobalSlotStableV1,
    pub global_slot_since_genesis: MinaNumbersGlobalSlotSinceGenesisMStableV1,
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
/// Gid: `1158`
/// Location: [src/lib/sync_status/sync_status.ml:55:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/sync_status/sync_status.ml#L55)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[polymorphic_variant]
pub enum SyncStatusTStableV1 {
    #[allow(non_camel_case_types)]
    Connecting,
    #[allow(non_camel_case_types)]
    Listening,
    #[allow(non_camel_case_types)]
    Offline,
    #[allow(non_camel_case_types)]
    Bootstrap,
    #[allow(non_camel_case_types)]
    Synced,
    #[allow(non_camel_case_types)]
    Catchup,
}

/// **OCaml name**: `Trust_system__Peer_status.Stable.V1`
///
/// Gid: `925`
/// Location: [src/lib/trust_system/peer_status.ml:6:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/trust_system/peer_status.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TrustSystemPeerStatusStableV1 {
    pub trust: crate::number::Float64,
    pub banned: TrustSystemBannedStatusStableV1,
}

/// **OCaml name**: `Blockchain_snark__Blockchain.Stable.V2`
///
/// Gid: `1069`
/// Location: [src/lib/blockchain_snark/blockchain.ml:8:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/blockchain_snark/blockchain.ml#L8)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct BlockchainSnarkBlockchainStableV2 {
    pub state: MinaStateProtocolStateValueStableV2,
    pub proof: MinaBaseProofStableV2,
}

/// **OCaml name**: `Transaction_witness.Stable.V2`
///
/// Gid: `1009`
/// Location: [src/lib/transaction_witness/transaction_witness.ml:54:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/transaction_witness/transaction_witness.ml#L54)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionWitnessStableV2 {
    pub transaction: MinaTransactionTransactionStableV2,
    pub first_pass_ledger: MinaBaseSparseLedgerBaseStableV2,
    pub second_pass_ledger: MinaBaseSparseLedgerBaseStableV2,
    pub protocol_state_body: MinaStateProtocolStateBodyValueStableV2,
    pub init_stack: MinaBasePendingCoinbaseStackVersionedStableV1,
    pub status: MinaBaseTransactionStatusStableV2,
    pub block_global_slot: MinaNumbersGlobalSlotSinceGenesisMStableV1,
}

/// **OCaml name**: `Prover.Extend_blockchain_input.Stable.V2`
///
/// Gid: `1280`
/// Location: [src/lib/prover/prover.ml:16:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/prover/prover.ml#L16)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ProverExtendBlockchainInputStableV2 {
    pub chain: BlockchainSnarkBlockchainStableV2,
    pub next_state: MinaStateProtocolStateValueStableV2,
    pub block: MinaStateSnarkTransitionValueStableV2,
    pub ledger_proof: Option<LedgerProofProdStableV2>,
    pub prover_state: ConsensusStakeProofStableV2,
    pub pending_coinbase: MinaBasePendingCoinbaseWitnessStableV2,
}

/// **OCaml name**: `Snark_worker.Worker.Rpcs_versioned.Get_work.V2.T.response`
///
/// Gid: `1135`
/// Location: [src/lib/snark_worker/snark_worker.ml:29:10](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/snark_worker/snark_worker.ml#L29)
///
///
/// Gid: `169`
/// Location: [src/std_internal.ml:137:2](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/std_internal.ml#L137)
/// Args: (SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0 , NonZeroCurvePoint ,)
///
///
/// Gid: `60`
/// Location: [src/option.ml:4:0](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/option.ml#L4)
/// Args: (SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0 , NonZeroCurvePoint ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponse(
    pub  Option<(
        SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0,
        NonZeroCurvePoint,
    )>,
);

/// **OCaml name**: `Snark_worker.Worker.Rpcs_versioned.Submit_work.V2.T.query`
///
/// Gid: `1136`
/// Location: [src/lib/snark_worker/snark_worker.ml:59:10](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/snark_worker/snark_worker.ml#L59)
///
///
/// Gid: `1040`
/// Location: [src/lib/snark_work_lib/work.ml:90:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/snark_work_lib/work.ml#L90)
/// Args: SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0 , LedgerProofProdStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct SnarkWorkerWorkerRpcsVersionedSubmitWorkV2TQuery {
    pub proofs: TransactionSnarkWorkTStableV2Proofs,
    pub metrics: SnarkWorkerWorkerRpcsVersionedSubmitWorkV2TQueryMetrics,
    pub spec: SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0,
    pub prover: NonZeroCurvePoint,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.statement.fp`
///
/// Gid: `461`
/// Location: [src/lib/pickles_types/shifted_value.ml:98:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles_types/shifted_value.ml#L98)
/// Args: crate :: bigint :: BigInt
#[derive(Clone, Debug, PartialEq, BinProtRead, BinProtWrite)]
pub enum PicklesProofProofsVerified2ReprStableV2StatementFp {
    ShiftedValue(crate::bigint::BigInt),
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.statement.proof_state.deferred_values.plonk.feature_flags`
///
/// Gid: `464`
/// Location: [src/lib/pickles_types/plonk_types.ml:194:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles_types/plonk_types.ml#L194)
/// Args: bool
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonkFeatureFlags
{
    pub range_check0: bool,
    pub range_check1: bool,
    pub foreign_field_add: bool,
    pub foreign_field_mul: bool,
    pub xor: bool,
    pub rot: bool,
    pub lookup: bool,
    pub runtime_tables: bool,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.prev_evals.evals.evals`
///
/// Gid: `465`
/// Location: [src/lib/pickles_types/plonk_types.ml:363:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles_types/plonk_types.ml#L363)
/// Args: (Vec < crate :: bigint :: BigInt > , Vec < crate :: bigint :: BigInt > ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals {
    pub w: PaddedSeq<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>), 15>,
    pub coefficients: PaddedSeq<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>), 15>,
    pub z: (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
    pub s: PaddedSeq<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>), 6>,
    pub generic_selector: (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
    pub poseidon_selector: (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
    pub complete_add_selector: (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
    pub mul_selector: (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
    pub emul_selector: (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
    pub endomul_scalar_selector: (Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>),
    pub range_check0_selector: Option<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>)>,
    pub range_check1_selector: Option<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>)>,
    pub foreign_field_add_selector:
        Option<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>)>,
    pub foreign_field_mul_selector:
        Option<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>)>,
    pub xor_selector: Option<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>)>,
    pub rot_selector: Option<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>)>,
    pub lookup_aggregation: Option<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>)>,
    pub lookup_table: Option<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>)>,
    pub lookup_sorted:
        PaddedSeq<Option<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>)>, 5>,
    pub runtime_lookup_table: Option<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>)>,
    pub runtime_lookup_table_selector:
        Option<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>)>,
    pub xor_lookup_selector: Option<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>)>,
    pub lookup_gate_lookup_selector:
        Option<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>)>,
    pub range_check_lookup_selector:
        Option<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>)>,
    pub foreign_field_mul_lookup_selector:
        Option<(Vec<crate::bigint::BigInt>, Vec<crate::bigint::BigInt>)>,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.prev_evals.evals`
///
/// Gid: `466`
/// Location: [src/lib/pickles_types/plonk_types.ml:1057:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles_types/plonk_types.ml#L1057)
/// Args: (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,) , (Vec < crate :: bigint :: BigInt > , Vec < crate :: bigint :: BigInt > ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PrevEvalsEvals {
    pub public_input: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub evals: PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.prev_evals`
///
/// Gid: `467`
/// Location: [src/lib/pickles_types/plonk_types.ml:1092:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles_types/plonk_types.ml#L1092)
/// Args: crate :: bigint :: BigInt , Vec < crate :: bigint :: BigInt >
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2PrevEvals {
    pub evals: PicklesProofProofsVerified2ReprStableV2PrevEvalsEvals,
    pub ft_eval1: crate::bigint::BigInt,
}

/// Derived name: `Pickles__Wrap_wire_proof.Stable.V1.bulletproof`
///
/// Gid: `468`
/// Location: [src/lib/pickles_types/plonk_types.ml:1141:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles_types/plonk_types.ml#L1141)
/// Args: (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,) , crate :: bigint :: BigInt
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesWrapWireProofStableV1Bulletproof {
    pub lr: Vec<(
        (crate::bigint::BigInt, crate::bigint::BigInt),
        (crate::bigint::BigInt, crate::bigint::BigInt),
    )>,
    pub z_1: crate::bigint::BigInt,
    pub z_2: crate::bigint::BigInt,
    pub delta: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub challenge_polynomial_commitment: (crate::bigint::BigInt, crate::bigint::BigInt),
}

/// Derived name: `Mina_base__Verification_key_wire.Stable.V1.wrap_index`
///
/// Gid: `476`
/// Location: [src/lib/pickles_types/plonk_verification_key_evals.ml:7:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles_types/plonk_verification_key_evals.ml#L7)
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
/// Gid: `482`
/// Location: [src/lib/crypto/kimchi_backend/common/scalar_challenge.ml:6:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/crypto/kimchi_backend/common/scalar_challenge.ml#L6)
/// Args: PaddedSeq < LimbVectorConstantHex64StableV1 , 2 >
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge {
    pub inner: PaddedSeq<LimbVectorConstantHex64StableV1, 2>,
}

/// Derived name: `Snark_worker.Worker.Rpcs_versioned.Submit_work.V2.T.query.metrics`
///
/// Gid: `508`
/// Location: [src/lib/one_or_two/one_or_two.ml:7:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/one_or_two/one_or_two.ml#L7)
/// Args: (crate :: number :: Float64 , SnarkWorkerWorkerRpcsVersionedSubmitWorkV2TQueryMetricsA1 ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[polymorphic_variant]
pub enum SnarkWorkerWorkerRpcsVersionedSubmitWorkV2TQueryMetrics {
    #[allow(non_camel_case_types)]
    One(
        (
            crate::number::Float64,
            SnarkWorkerWorkerRpcsVersionedSubmitWorkV2TQueryMetricsA1,
        ),
    ),
    #[allow(non_camel_case_types)]
    Two(
        (
            (
                crate::number::Float64,
                SnarkWorkerWorkerRpcsVersionedSubmitWorkV2TQueryMetricsA1,
            ),
            (
                crate::number::Float64,
                SnarkWorkerWorkerRpcsVersionedSubmitWorkV2TQueryMetricsA1,
            ),
        ),
    ),
}

/// Derived name: `Transaction_snark_work.T.Stable.V2.proofs`
///
/// Gid: `508`
/// Location: [src/lib/one_or_two/one_or_two.ml:7:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/one_or_two/one_or_two.ml#L7)
/// Args: LedgerProofProdStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[polymorphic_variant]
pub enum TransactionSnarkWorkTStableV2Proofs {
    #[allow(non_camel_case_types)]
    One(LedgerProofProdStableV2),
    #[allow(non_camel_case_types)]
    Two((LedgerProofProdStableV2, LedgerProofProdStableV2)),
}

/// Derived name: `Snark_worker.Worker.Rpcs_versioned.Get_work.V2.T.response.a.0.instances`
///
/// Gid: `508`
/// Location: [src/lib/one_or_two/one_or_two.ml:7:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/one_or_two/one_or_two.ml#L7)
/// Args: SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Single
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[polymorphic_variant]
pub enum SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Instances {
    #[allow(non_camel_case_types)]
    One(SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Single),
    #[allow(non_camel_case_types)]
    Two(
        (
            SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Single,
            SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Single,
        ),
    ),
}

/// **OCaml name**: `Pickles_base__Proofs_verified.Stable.V1`
///
/// Gid: `514`
/// Location: [src/lib/pickles_base/proofs_verified.ml:8:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles_base/proofs_verified.ml#L8)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum PicklesBaseProofsVerifiedStableV1 {
    N0,
    N1,
    N2,
}

/// **OCaml name**: `Limb_vector__Constant.Hex64.Stable.V1`
///
/// Gid: `523`
/// Location: [src/lib/pickles/limb_vector/constant.ml:60:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles/limb_vector/constant.ml#L60)
///
///
/// Gid: `125`
/// Location: [src/int64.ml:6:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/int64.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct LimbVectorConstantHex64StableV1(pub crate::number::UInt64);

/// **OCaml name**: `Composition_types__Branch_data.Make_str.Domain_log2.Stable.V1`
///
/// Gid: `524`
/// Location: [src/lib/pickles/composition_types/branch_data.ml:24:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles/composition_types/branch_data.ml#L24)
///
///
/// Gid: `161`
/// Location: [src/std_internal.ml:113:2](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/std_internal.ml#L113)
///
///
/// Gid: `89`
/// Location: [src/char.ml:8:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/char.ml#L8)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct CompositionTypesBranchDataDomainLog2StableV1(pub crate::char::Char);

/// **OCaml name**: `Composition_types__Branch_data.Make_str.Stable.V1`
///
/// Gid: `525`
/// Location: [src/lib/pickles/composition_types/branch_data.ml:51:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles/composition_types/branch_data.ml#L51)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct CompositionTypesBranchDataStableV1 {
    pub proofs_verified: PicklesBaseProofsVerifiedStableV1,
    pub domain_log2: CompositionTypesBranchDataDomainLog2StableV1,
}

/// Derived name: `Pickles__Reduced_messages_for_next_proof_over_same_field.Wrap.Challenges_vector.Stable.V2.a`
///
/// Gid: `526`
/// Location: [src/lib/pickles/composition_types/bulletproof_challenge.ml:4:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles/composition_types/bulletproof_challenge.ml#L4)
/// Args: PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A {
    pub prechallenge:
        PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge,
}

/// **OCaml name**: `Composition_types__Digest.Constant.Stable.V1`
///
/// Gid: `527`
/// Location: [src/lib/pickles/composition_types/digest.ml:13:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles/composition_types/digest.ml#L13)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct CompositionTypesDigestConstantStableV1(
    pub PaddedSeq<LimbVectorConstantHex64StableV1, 4>,
);

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.statement.proof_state.deferred_values.plonk`
///
/// Gid: `528`
/// Location: [src/lib/pickles/composition_types/composition_types.ml:45:14](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles/composition_types/composition_types.ml#L45)
/// Args: PaddedSeq < LimbVectorConstantHex64StableV1 , 2 > , PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge , bool
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonk {
    pub alpha:
        PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge,
    pub beta: PaddedSeq<LimbVectorConstantHex64StableV1, 2>,
    pub gamma: PaddedSeq<LimbVectorConstantHex64StableV1, 2>,
    pub zeta: PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge,
    pub joint_combiner: Option<
        PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge,
    >,
    pub feature_flags:
        PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonkFeatureFlags,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.statement.proof_state.deferred_values`
///
/// Gid: `530`
/// Location: [src/lib/pickles/composition_types/composition_types.ml:275:12](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles/composition_types/composition_types.ml#L275)
/// Args: PaddedSeq < LimbVectorConstantHex64StableV1 , 2 > , PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge , PicklesProofProofsVerified2ReprStableV2StatementFp , bool , PaddedSeq < PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , 16 > , CompositionTypesBranchDataStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValues {
    pub plonk: PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonk,
    pub bulletproof_challenges:
        PaddedSeq<PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A, 16>,
    pub branch_data: CompositionTypesBranchDataStableV1,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.messages_for_next_wrap_proof`
///
/// Gid: `531`
/// Location: [src/lib/pickles/composition_types/composition_types.ml:397:10](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles/composition_types/composition_types.ml#L397)
/// Args: (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,) , PaddedSeq < PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2 , 2 >
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof {
    pub challenge_polynomial_commitment: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub old_bulletproof_challenges:
        PaddedSeq<PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2, 2>,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.statement.proof_state`
///
/// Gid: `533`
/// Location: [src/lib/pickles/composition_types/composition_types.ml:466:10](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles/composition_types/composition_types.ml#L466)
/// Args: PaddedSeq < LimbVectorConstantHex64StableV1 , 2 > , PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge , PicklesProofProofsVerified2ReprStableV2StatementFp , bool , PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof , CompositionTypesDigestConstantStableV1 , PaddedSeq < PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , 16 > , CompositionTypesBranchDataStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2StatementProofState {
    pub deferred_values: PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValues,
    pub sponge_digest_before_evaluations: CompositionTypesDigestConstantStableV1,
    pub messages_for_next_wrap_proof:
        PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.statement`
///
/// Gid: `535`
/// Location: [src/lib/pickles/composition_types/composition_types.ml:714:10](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles/composition_types/composition_types.ml#L714)
/// Args: PaddedSeq < LimbVectorConstantHex64StableV1 , 2 > , PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge , PicklesProofProofsVerified2ReprStableV2StatementFp , bool , PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof , CompositionTypesDigestConstantStableV1 , PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof , PaddedSeq < PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A , 16 > , CompositionTypesBranchDataStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2Statement {
    pub proof_state: PicklesProofProofsVerified2ReprStableV2StatementProofState,
    pub messages_for_next_step_proof:
        PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof,
}

/// **OCaml name**: `Pickles__Wrap_wire_proof.Commitments.Stable.V1`
///
/// Gid: `537`
/// Location: [src/lib/pickles/wrap_wire_proof.ml:17:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles/wrap_wire_proof.ml#L17)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesWrapWireProofCommitmentsStableV1 {
    pub w_comm: PaddedSeq<(crate::bigint::BigInt, crate::bigint::BigInt), 15>,
    pub z_comm: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub t_comm: PaddedSeq<(crate::bigint::BigInt, crate::bigint::BigInt), 7>,
}

/// **OCaml name**: `Pickles__Wrap_wire_proof.Evaluations.Stable.V1`
///
/// Gid: `538`
/// Location: [src/lib/pickles/wrap_wire_proof.ml:55:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles/wrap_wire_proof.ml#L55)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesWrapWireProofEvaluationsStableV1 {
    pub w: PaddedSeq<(crate::bigint::BigInt, crate::bigint::BigInt), 15>,
    pub coefficients: PaddedSeq<(crate::bigint::BigInt, crate::bigint::BigInt), 15>,
    pub z: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub s: PaddedSeq<(crate::bigint::BigInt, crate::bigint::BigInt), 6>,
    pub generic_selector: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub poseidon_selector: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub complete_add_selector: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub mul_selector: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub emul_selector: (crate::bigint::BigInt, crate::bigint::BigInt),
    pub endomul_scalar_selector: (crate::bigint::BigInt, crate::bigint::BigInt),
}

/// **OCaml name**: `Pickles__Wrap_wire_proof.Stable.V1`
///
/// Gid: `539`
/// Location: [src/lib/pickles/wrap_wire_proof.ml:175:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles/wrap_wire_proof.ml#L175)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesWrapWireProofStableV1 {
    pub commitments: PicklesWrapWireProofCommitmentsStableV1,
    pub evaluations: PicklesWrapWireProofEvaluationsStableV1,
    pub ft_eval1: crate::bigint::BigInt,
    pub bulletproof: PicklesWrapWireProofStableV1Bulletproof,
}

/// Derived name: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2.messages_for_next_step_proof`
///
/// Gid: `542`
/// Location: [src/lib/pickles/reduced_messages_for_next_proof_over_same_field.ml:16:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles/reduced_messages_for_next_proof_over_same_field.ml#L16)
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
/// Gid: `543`
/// Location: [src/lib/pickles/reduced_messages_for_next_proof_over_same_field.ml:57:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles/reduced_messages_for_next_proof_over_same_field.ml#L57)
///
///
/// Gid: `488`
/// Location: [src/lib/crypto/kimchi_backend/pasta/basic/kimchi_pasta_basic.ml:32:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/crypto/kimchi_backend/pasta/basic/kimchi_pasta_basic.ml#L32)
/// Args: PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2(
    pub PaddedSeq<PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A, 15>,
);

/// **OCaml name**: `Mina_base__Verification_key_wire.Stable.V1`
///
/// Gid: `544`
/// Location: [src/lib/pickles/side_loaded_verification_key.ml:170:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles/side_loaded_verification_key.ml#L170)
///
///
/// Gid: `519`
/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:130:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles_base/side_loaded_verification_key.ml#L130)
/// Args: (crate :: bigint :: BigInt , crate :: bigint :: BigInt ,)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseVerificationKeyWireStableV1 {
    pub max_proofs_verified: PicklesBaseProofsVerifiedStableV1,
    pub actual_wrap_domain_size: PicklesBaseProofsVerifiedStableV1,
    pub wrap_index: MinaBaseVerificationKeyWireStableV1WrapIndex,
}

/// **OCaml name**: `Pickles__Proof.Proofs_verified_2.Repr.Stable.V2`
///
/// Gid: `547`
/// Location: [src/lib/pickles/proof.ml:342:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles/proof.ml#L342)
///
///
/// Gid: `546`
/// Location: [src/lib/pickles/proof.ml:47:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles/proof.ml#L47)
/// Args: PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof , PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerified2ReprStableV2 {
    pub statement: PicklesProofProofsVerified2ReprStableV2Statement,
    pub prev_evals: PicklesProofProofsVerified2ReprStableV2PrevEvals,
    pub proof: PicklesWrapWireProofStableV1,
}

/// **OCaml name**: `Pickles__Proof.Proofs_verified_max.Stable.V2`
///
/// Gid: `548`
/// Location: [src/lib/pickles/proof.ml:411:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles/proof.ml#L411)
///
///
/// Gid: `546`
/// Location: [src/lib/pickles/proof.ml:47:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/pickles/proof.ml#L47)
/// Args: PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof , PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PicklesProofProofsVerifiedMaxStableV2 {
    pub statement: PicklesProofProofsVerified2ReprStableV2Statement,
    pub prev_evals: PicklesProofProofsVerified2ReprStableV2PrevEvals,
    pub proof: PicklesWrapWireProofStableV1,
}

/// **OCaml name**: `Non_zero_curve_point.Uncompressed.Stable.V1`
///
/// Gid: `558`
/// Location: [src/lib/non_zero_curve_point/non_zero_curve_point.ml:46:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/non_zero_curve_point/non_zero_curve_point.ml#L46)
///
///
/// Gid: `552`
/// Location: [src/lib/non_zero_curve_point/compressed_poly.ml:13:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/non_zero_curve_point/compressed_poly.ml#L13)
/// Args: crate :: bigint :: BigInt , bool
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, BinProtRead, BinProtWrite,
)]
pub struct NonZeroCurvePointUncompressedStableV1 {
    pub x: crate::bigint::BigInt,
    pub is_odd: bool,
}

/// **OCaml name**: `Signature_lib__Private_key.Stable.V1`
///
/// Gid: `570`
/// Location: [src/lib/signature_lib/private_key.ml:11:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/signature_lib/private_key.ml#L11)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct SignatureLibPrivateKeyStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Unsigned_extended.UInt64.Int64_for_version_tags.Stable.V1`
///
/// Gid: `576`
/// Location: [src/lib/unsigned_extended/unsigned_extended.ml:81:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/unsigned_extended/unsigned_extended.ml#L81)
///
///
/// Gid: `125`
/// Location: [src/int64.ml:6:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/int64.ml#L6)
#[derive(Clone, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct UnsignedExtendedUInt64Int64ForVersionTagsStableV1(pub crate::number::UInt64);

/// **OCaml name**: `Unsigned_extended.UInt32.Stable.V1`
///
/// Gid: `580`
/// Location: [src/lib/unsigned_extended/unsigned_extended.ml:156:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/unsigned_extended/unsigned_extended.ml#L156)
///
///
/// Gid: `119`
/// Location: [src/int32.ml:6:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/int32.ml#L6)
#[derive(Clone, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref, Default)]
pub struct UnsignedExtendedUInt32StableV1(pub crate::number::UInt32);

/// **OCaml name**: `Protocol_version.Make_str.Stable.V2`
///
/// Gid: `584`
/// Location: [src/lib/protocol_version/protocol_version.ml:18:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/protocol_version/protocol_version.ml#L18)
#[derive(Clone, Debug, PartialEq, BinProtRead, BinProtWrite)]
pub struct ProtocolVersionStableV2 {
    pub transaction: crate::number::UInt64,
    pub network: crate::number::UInt64,
    pub patch: crate::number::UInt64,
}

/// **OCaml name**: `Mina_numbers__Nat.Make32.Stable.V1`
///
/// Gid: `585`
/// Location: [src/lib/mina_numbers/nat.ml:260:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_numbers/nat.ml#L260)
#[derive(
    Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref, Default,
)]
pub struct MinaNumbersNatMake32StableV1(pub UnsignedExtendedUInt32StableV1);

/// **OCaml name**: `Mina_numbers__Global_slot_span.Make_str.Stable.V1`
///
/// Gid: `608`
/// Location: [src/lib/mina_numbers/global_slot_span.ml:22:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_numbers/global_slot_span.ml#L22)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaNumbersGlobalSlotSpanStableV1 {
    GlobalSlotSpan(UnsignedExtendedUInt32StableV1),
}

/// **OCaml name**: `Mina_numbers__Global_slot_since_genesis.Make_str.M.Stable.V1`
///
/// Gid: `614`
/// Location: [src/lib/mina_numbers/global_slot_since_genesis.ml:27:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_numbers/global_slot_since_genesis.ml#L27)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaNumbersGlobalSlotSinceGenesisMStableV1 {
    SinceGenesis(UnsignedExtendedUInt32StableV1),
}

/// **OCaml name**: `Mina_numbers__Global_slot_since_hard_fork.Make_str.M.Stable.V1`
///
/// Gid: `620`
/// Location: [src/lib/mina_numbers/global_slot_since_hard_fork.ml:27:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_numbers/global_slot_since_hard_fork.ml#L27)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaNumbersGlobalSlotSinceHardForkMStableV1 {
    SinceHardFork(UnsignedExtendedUInt32StableV1),
}

/// **OCaml name**: `Sgn.Stable.V1`
///
/// Gid: `636`
/// Location: [src/lib/sgn/sgn.ml:9:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/sgn/sgn.ml#L9)
#[derive(Clone, Debug, PartialEq, BinProtRead, BinProtWrite)]
pub enum SgnStableV1 {
    Pos,
    Neg,
}

/// Derived name: `Mina_state__Blockchain_state.Value.Stable.V2.signed_amount`
///
/// Gid: `637`
/// Location: [src/lib/currency/signed_poly.ml:6:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/currency/signed_poly.ml#L6)
/// Args: CurrencyAmountStableV1 , SgnStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateBlockchainStateValueStableV2SignedAmount {
    pub magnitude: CurrencyAmountStableV1,
    pub sgn: SgnStableV1,
}

/// **OCaml name**: `Currency.Make_str.Fee.Stable.V1`
///
/// Gid: `638`
/// Location: [src/lib/currency/currency.ml:947:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/currency/currency.ml#L947)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct CurrencyFeeStableV1(pub UnsignedExtendedUInt64Int64ForVersionTagsStableV1);

/// **OCaml name**: `Currency.Make_str.Amount.Make_str.Stable.V1`
///
/// Gid: `641`
/// Location: [src/lib/currency/currency.ml:1094:10](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/currency/currency.ml#L1094)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct CurrencyAmountStableV1(pub UnsignedExtendedUInt64Int64ForVersionTagsStableV1);

/// **OCaml name**: `Currency.Make_str.Balance.Stable.V1`
///
/// Gid: `644`
/// Location: [src/lib/currency/currency.ml:1138:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/currency/currency.ml#L1138)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct CurrencyBalanceStableV1(pub CurrencyAmountStableV1);

/// **OCaml name**: `Data_hash_lib__State_hash.Stable.V1`
///
/// Gid: `651`
/// Location: [src/lib/data_hash_lib/state_hash.ml:44:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/data_hash_lib/state_hash.ml#L44)
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    BinProtRead,
    BinProtWrite,
    Deref,
)]
pub struct DataHashLibStateHashStableV1(pub crate::bigint::BigInt);

/// Derived name: `Mina_base__Sparse_ledger_base.Stable.V2.tree`
///
/// Gid: `660`
/// Location: [src/lib/sparse_ledger_lib/sparse_ledger.ml:9:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/sparse_ledger_lib/sparse_ledger.ml#L9)
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
/// Gid: `660`
/// Location: [src/lib/sparse_ledger_lib/sparse_ledger.ml:9:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/sparse_ledger_lib/sparse_ledger.ml#L9)
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
/// Gid: `662`
/// Location: [src/lib/block_time/block_time.ml:22:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/block_time/block_time.ml#L22)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct BlockTimeTimeStableV1(pub UnsignedExtendedUInt64Int64ForVersionTagsStableV1);

/// **OCaml name**: `Mina_base__Account_id.Make_str.Digest.Stable.V1`
///
/// Gid: `664`
/// Location: [src/lib/mina_base/account_id.ml:64:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/account_id.ml#L64)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct MinaBaseAccountIdDigestStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Account_id.Make_str.Stable.V2`
///
/// Gid: `669`
/// Location: [src/lib/mina_base/account_id.ml:151:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/account_id.ml#L151)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountIdStableV2(pub NonZeroCurvePoint, pub MinaBaseAccountIdDigestStableV1);

/// **OCaml name**: `Mina_base__Account_timing.Stable.V2`
///
/// Gid: `675`
/// Location: [src/lib/mina_base/account_timing.ml:39:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/account_timing.ml#L39)
///
///
/// Gid: `674`
/// Location: [src/lib/mina_base/account_timing.ml:22:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/account_timing.ml#L22)
/// Args: MinaNumbersGlobalSlotSinceGenesisMStableV1 , MinaNumbersGlobalSlotSpanStableV1 , CurrencyBalanceStableV1 , CurrencyAmountStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountTimingStableV2 {
    Untimed,
    Timed {
        initial_minimum_balance: CurrencyBalanceStableV1,
        cliff_time: MinaNumbersGlobalSlotSinceGenesisMStableV1,
        cliff_amount: CurrencyAmountStableV1,
        vesting_period: MinaNumbersGlobalSlotSpanStableV1,
        vesting_increment: CurrencyAmountStableV1,
    },
}

/// **OCaml name**: `Mina_base__Signature.Stable.V1`
///
/// Gid: `679`
/// Location: [src/lib/mina_base/signature.ml:23:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/signature.ml#L23)
///
///
/// Gid: `676`
/// Location: [src/lib/mina_base/signature.ml:12:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/signature.ml#L12)
/// Args: crate :: bigint :: BigInt , crate :: bigint :: BigInt
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignatureStableV1(pub crate::bigint::BigInt, pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Control.Stable.V2`
///
/// Gid: `683`
/// Location: [src/lib/mina_base/control.ml:11:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/control.ml#L11)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseControlStableV2 {
    Proof(Box<PicklesProofProofsVerifiedMaxStableV2>),
    Signature(Signature),
    NoneGiven,
}

/// **OCaml name**: `Mina_base__Token_id.Stable.V2`
///
/// Gid: `687`
/// Location: [src/lib/mina_base/token_id.ml:8:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/token_id.ml#L8)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct MinaBaseTokenIdStableV2(pub MinaBaseAccountIdDigestStableV1);

/// **OCaml name**: `Mina_base__Payment_payload.Stable.V2`
///
/// Gid: `697`
/// Location: [src/lib/mina_base/payment_payload.ml:39:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/payment_payload.ml#L39)
///
///
/// Gid: `693`
/// Location: [src/lib/mina_base/payment_payload.ml:14:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/payment_payload.ml#L14)
/// Args: NonZeroCurvePoint , CurrencyAmountStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePaymentPayloadStableV2 {
    pub receiver_pk: NonZeroCurvePoint,
    pub amount: CurrencyAmountStableV1,
}

/// **OCaml name**: `Mina_base__Ledger_hash0.Stable.V1`
///
/// Gid: `703`
/// Location: [src/lib/mina_base/ledger_hash0.ml:17:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/ledger_hash0.ml#L17)
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    BinProtRead,
    BinProtWrite,
    Deref,
)]
pub struct MinaBaseLedgerHash0StableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Permissions.Auth_required.Stable.V2`
///
/// Gid: `706`
/// Location: [src/lib/mina_base/permissions.ml:53:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/permissions.ml#L53)
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
/// Gid: `708`
/// Location: [src/lib/mina_base/permissions.ml:399:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/permissions.ml#L399)
///
///
/// Gid: `707`
/// Location: [src/lib/mina_base/permissions.ml:357:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/permissions.ml#L357)
/// Args: MinaBasePermissionsAuthRequiredStableV2 , UnsignedExtendedUInt32StableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePermissionsStableV2 {
    pub edit_state: MinaBasePermissionsAuthRequiredStableV2,
    pub access: MinaBasePermissionsAuthRequiredStableV2,
    pub send: MinaBasePermissionsAuthRequiredStableV2,
    pub receive: MinaBasePermissionsAuthRequiredStableV2,
    pub set_delegate: MinaBasePermissionsAuthRequiredStableV2,
    pub set_permissions: MinaBasePermissionsAuthRequiredStableV2,
    pub set_verification_key: (
        MinaBasePermissionsAuthRequiredStableV2,
        UnsignedExtendedUInt32StableV1,
    ),
    pub set_zkapp_uri: MinaBasePermissionsAuthRequiredStableV2,
    pub edit_action_state: MinaBasePermissionsAuthRequiredStableV2,
    pub set_token_symbol: MinaBasePermissionsAuthRequiredStableV2,
    pub increment_nonce: MinaBasePermissionsAuthRequiredStableV2,
    pub set_voting_for: MinaBasePermissionsAuthRequiredStableV2,
    pub set_timing: MinaBasePermissionsAuthRequiredStableV2,
}

/// **OCaml name**: `Mina_base__Stake_delegation.Stable.V2`
///
/// Gid: `712`
/// Location: [src/lib/mina_base/stake_delegation.ml:11:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/stake_delegation.ml#L11)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseStakeDelegationStableV2 {
    SetDelegate { new_delegate: NonZeroCurvePoint },
}

/// **OCaml name**: `Mina_base__Transaction_status.Failure.Stable.V2`
///
/// Gid: `718`
/// Location: [src/lib/mina_base/transaction_status.ml:9:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/transaction_status.ml#L9)
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
    AccountAppStatePreconditionUnsatisfied(crate::number::UInt64),
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
/// Gid: `720`
/// Location: [src/lib/mina_base/transaction_status.ml:77:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/transaction_status.ml#L77)
///
///
/// Gid: `167`
/// Location: [src/std_internal.ml:131:2](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/std_internal.ml#L131)
/// Args: Vec < MinaBaseTransactionStatusFailureStableV2 >
///
///
/// Gid: `50`
/// Location: [src/list0.ml:6:0](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/list0.ml#L6)
/// Args: Vec < MinaBaseTransactionStatusFailureStableV2 >
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct MinaBaseTransactionStatusFailureCollectionStableV1(
    pub Vec<Vec<MinaBaseTransactionStatusFailureStableV2>>,
);

/// **OCaml name**: `Mina_base__Transaction_status.Stable.V2`
///
/// Gid: `721`
/// Location: [src/lib/mina_base/transaction_status.ml:476:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/transaction_status.ml#L476)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseTransactionStatusStableV2 {
    Applied,
    Failed(MinaBaseTransactionStatusFailureCollectionStableV1),
}

/// **OCaml name**: `Mina_base__Signed_command_payload.Common.Stable.V2`
///
/// Gid: `726`
/// Location: [src/lib/mina_base/signed_command_payload.ml:76:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/signed_command_payload.ml#L76)
///
///
/// Gid: `722`
/// Location: [src/lib/mina_base/signed_command_payload.ml:41:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/signed_command_payload.ml#L41)
/// Args: CurrencyFeeStableV1 , NonZeroCurvePoint , UnsignedExtendedUInt32StableV1 , MinaNumbersGlobalSlotSinceGenesisMStableV1 , MinaBaseSignedCommandMemoStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandPayloadCommonStableV2 {
    pub fee: CurrencyFeeStableV1,
    pub fee_payer_pk: NonZeroCurvePoint,
    pub nonce: UnsignedExtendedUInt32StableV1,
    pub valid_until: MinaNumbersGlobalSlotSinceGenesisMStableV1,
    pub memo: MinaBaseSignedCommandMemoStableV1,
}

/// **OCaml name**: `Mina_base__Signed_command_payload.Body.Stable.V2`
///
/// Gid: `730`
/// Location: [src/lib/mina_base/signed_command_payload.ml:189:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/signed_command_payload.ml#L189)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseSignedCommandPayloadBodyStableV2 {
    Payment(MinaBasePaymentPayloadStableV2),
    StakeDelegation(MinaBaseStakeDelegationStableV2),
}

/// **OCaml name**: `Mina_base__Signed_command_payload.Stable.V2`
///
/// Gid: `737`
/// Location: [src/lib/mina_base/signed_command_payload.ml:267:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/signed_command_payload.ml#L267)
///
///
/// Gid: `734`
/// Location: [src/lib/mina_base/signed_command_payload.ml:249:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/signed_command_payload.ml#L249)
/// Args: MinaBaseSignedCommandPayloadCommonStableV2 , MinaBaseSignedCommandPayloadBodyStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandPayloadStableV2 {
    pub common: MinaBaseSignedCommandPayloadCommonStableV2,
    pub body: MinaBaseSignedCommandPayloadBodyStableV2,
}

/// **OCaml name**: `Mina_base__Signed_command.Make_str.Stable.V2`
///
/// Gid: `744`
/// Location: [src/lib/mina_base/signed_command.ml:52:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/signed_command.ml#L52)
///
///
/// Gid: `741`
/// Location: [src/lib/mina_base/signed_command.ml:27:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/signed_command.ml#L27)
/// Args: MinaBaseSignedCommandPayloadStableV2 , NonZeroCurvePoint , Signature
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSignedCommandStableV2 {
    pub payload: MinaBaseSignedCommandPayloadStableV2,
    pub signer: NonZeroCurvePoint,
    pub signature: Signature,
}

/// **OCaml name**: `Mina_base__Receipt.Chain_hash.Stable.V1`
///
/// Gid: `755`
/// Location: [src/lib/mina_base/receipt.ml:31:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/receipt.ml#L31)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct MinaBaseReceiptChainHashStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__State_body_hash.Stable.V1`
///
/// Gid: `760`
/// Location: [src/lib/mina_base/state_body_hash.ml:19:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/state_body_hash.ml#L19)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct MinaBaseStateBodyHashStableV1(pub crate::bigint::BigInt);

/// Derived name: `Mina_base__Account_update.Update.Stable.V1.timing`
///
/// Gid: `766`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: MinaBaseAccountUpdateUpdateTimingInfoStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateUpdateStableV1Timing {
    Set(Box<MinaBaseAccountUpdateUpdateTimingInfoStableV1>),
    Keep,
}

/// Derived name: `Mina_base__Account_update.Update.Stable.V1.permissions`
///
/// Gid: `766`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: MinaBasePermissionsStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateUpdateStableV1Permissions {
    Set(Box<MinaBasePermissionsStableV2>),
    Keep,
}

/// Derived name: `Mina_base__Account_update.Update.Stable.V1.verification_key`
///
/// Gid: `766`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: MinaBaseVerificationKeyWireStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateUpdateStableV1VerificationKey {
    Set(Box<MinaBaseVerificationKeyWireStableV1>),
    Keep,
}

/// Derived name: `Mina_base__Account_update.Update.Stable.V1.delegate`
///
/// Gid: `766`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: NonZeroCurvePoint
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateUpdateStableV1Delegate {
    Set(NonZeroCurvePoint),
    Keep,
}

/// Derived name: `Mina_base__Account_update.Update.Stable.V1.voting_for`
///
/// Gid: `766`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: StateHash
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateUpdateStableV1VotingFor {
    Set(StateHash),
    Keep,
}

/// Derived name: `Mina_base__Account_update.Update.Stable.V1.app_state.a`
///
/// Gid: `766`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: crate :: bigint :: BigInt
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateUpdateStableV1AppStateA {
    Set(crate::bigint::BigInt),
    Keep,
}

/// Derived name: `Mina_base__Account_update.Update.Stable.V1.zkapp_uri`
///
/// Gid: `766`
/// Location: [src/lib/mina_base/zkapp_basic.ml:100:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_basic.ml#L100)
/// Args: crate :: string :: ByteString
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateUpdateStableV1ZkappUri {
    Set(crate::string::ByteString),
    Keep,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Epoch_data.Stable.V1.epoch_seed`
///
/// Gid: `767`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: EpochSeed
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochSeed {
    Check(EpochSeed),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1.snarked_ledger_hash`
///
/// Gid: `767`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: LedgerHash
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash {
    Check(LedgerHash),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Account.Stable.V2.receipt_chain_hash`
///
/// Gid: `767`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: MinaBaseReceiptChainHashStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionAccountStableV2ReceiptChainHash {
    Check(MinaBaseReceiptChainHashStableV1),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Account.Stable.V2.delegate`
///
/// Gid: `767`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: NonZeroCurvePoint
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionAccountStableV2Delegate {
    Check(NonZeroCurvePoint),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Epoch_data.Stable.V1.start_checkpoint`
///
/// Gid: `767`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: StateHash
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionProtocolStateEpochDataStableV1StartCheckpoint {
    Check(StateHash),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Account.Stable.V2.proved_state`
///
/// Gid: `767`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: bool
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionAccountStableV2ProvedState {
    Check(bool),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Account.Stable.V2.state.a`
///
/// Gid: `767`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: crate :: bigint :: BigInt
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionAccountStableV2StateA {
    Check(crate::bigint::BigInt),
    Ignore,
}

/// **OCaml name**: `Mina_base__Zkapp_state.Value.Stable.V1`
///
/// Gid: `770`
/// Location: [src/lib/mina_base/zkapp_state.ml:46:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_state.ml#L46)
///
///
/// Gid: `769`
/// Location: [src/lib/mina_base/zkapp_state.ml:17:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_state.ml#L17)
/// Args: crate :: bigint :: BigInt
#[derive(
    Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref, Default,
)]
pub struct MinaBaseZkappStateValueStableV1(pub PaddedSeq<crate::bigint::BigInt, 8>);

/// **OCaml name**: `Mina_base__Zkapp_account.Stable.V2`
///
/// Gid: `772`
/// Location: [src/lib/mina_base/zkapp_account.ml:224:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_account.ml#L224)
///
///
/// Gid: `771`
/// Location: [src/lib/mina_base/zkapp_account.ml:194:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_account.ml#L194)
/// Args: MinaBaseZkappStateValueStableV1 , Option < MinaBaseVerificationKeyWireStableV1 > , MinaNumbersNatMake32StableV1 , crate :: bigint :: BigInt , MinaNumbersGlobalSlotSinceGenesisMStableV1 , bool , crate :: string :: ByteString
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappAccountStableV2 {
    pub app_state: MinaBaseZkappStateValueStableV1,
    pub verification_key: Option<MinaBaseVerificationKeyWireStableV1>,
    pub zkapp_version: MinaNumbersNatMake32StableV1,
    pub action_state: PaddedSeq<crate::bigint::BigInt, 5>,
    pub last_action_slot: MinaNumbersGlobalSlotSinceGenesisMStableV1,
    pub proved_state: bool,
    pub zkapp_uri: crate::string::ByteString,
}

/// **OCaml name**: `Mina_base__Account.Index.Stable.V1`
///
/// Gid: `773`
/// Location: [src/lib/mina_base/account.ml:18:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/account.ml#L18)
///
///
/// Gid: `163`
/// Location: [src/std_internal.ml:119:2](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/std_internal.ml#L119)
///
///
/// Gid: `113`
/// Location: [src/int.ml:19:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/int.ml#L19)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct MinaBaseAccountIndexStableV1(pub crate::number::UInt64);

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Epoch_data.Stable.V1.epoch_ledger`
///
/// Gid: `781`
/// Location: [src/lib/mina_base/epoch_ledger.ml:9:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/epoch_ledger.ml#L9)
/// Args: MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash , MinaBaseZkappPreconditionProtocolStateStableV1Amount
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochLedger {
    pub hash: MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash,
    pub total_currency: MinaBaseZkappPreconditionProtocolStateStableV1Amount,
}

/// **OCaml name**: `Mina_base__Epoch_ledger.Value.Stable.V1`
///
/// Gid: `782`
/// Location: [src/lib/mina_base/epoch_ledger.ml:23:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/epoch_ledger.ml#L23)
///
///
/// Gid: `781`
/// Location: [src/lib/mina_base/epoch_ledger.ml:9:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/epoch_ledger.ml#L9)
/// Args: LedgerHash , CurrencyAmountStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseEpochLedgerValueStableV1 {
    pub hash: LedgerHash,
    pub total_currency: CurrencyAmountStableV1,
}

/// **OCaml name**: `Mina_base__Epoch_seed.Stable.V1`
///
/// Gid: `785`
/// Location: [src/lib/mina_base/epoch_seed.ml:14:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/epoch_seed.ml#L14)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct MinaBaseEpochSeedStableV1(pub crate::bigint::BigInt);

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1.amount.a`
///
/// Gid: `790`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:23:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_precondition.ml#L23)
/// Args: CurrencyAmountStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionProtocolStateStableV1AmountA {
    pub lower: CurrencyAmountStableV1,
    pub upper: CurrencyAmountStableV1,
}

/// Derived name: `Mina_base__Zkapp_precondition.Account.Stable.V2.balance.a`
///
/// Gid: `790`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:23:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_precondition.ml#L23)
/// Args: CurrencyBalanceStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionAccountStableV2BalanceA {
    pub lower: CurrencyBalanceStableV1,
    pub upper: CurrencyBalanceStableV1,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1.global_slot.a`
///
/// Gid: `790`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:23:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_precondition.ml#L23)
/// Args: MinaNumbersGlobalSlotSinceGenesisMStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlotA {
    pub lower: MinaNumbersGlobalSlotSinceGenesisMStableV1,
    pub upper: MinaNumbersGlobalSlotSinceGenesisMStableV1,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1.length.a`
///
/// Gid: `790`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:23:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_precondition.ml#L23)
/// Args: UnsignedExtendedUInt32StableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionProtocolStateStableV1LengthA {
    pub lower: UnsignedExtendedUInt32StableV1,
    pub upper: UnsignedExtendedUInt32StableV1,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1.amount`
///
/// Gid: `791`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:165:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_precondition.ml#L165)
/// Args: CurrencyAmountStableV1
///
///
/// Gid: `767`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: MinaBaseZkappPreconditionProtocolStateStableV1AmountA
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionProtocolStateStableV1Amount {
    Check(MinaBaseZkappPreconditionProtocolStateStableV1AmountA),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Account.Stable.V2.balance`
///
/// Gid: `791`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:165:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_precondition.ml#L165)
/// Args: CurrencyBalanceStableV1
///
///
/// Gid: `767`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: MinaBaseZkappPreconditionAccountStableV2BalanceA
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionAccountStableV2Balance {
    Check(MinaBaseZkappPreconditionAccountStableV2BalanceA),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1.global_slot`
///
/// Gid: `791`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:165:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_precondition.ml#L165)
/// Args: MinaNumbersGlobalSlotSinceGenesisMStableV1
///
///
/// Gid: `767`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlotA
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlot {
    Check(MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlotA),
    Ignore,
}

/// Derived name: `Mina_base__Zkapp_precondition.Protocol_state.Stable.V1.length`
///
/// Gid: `791`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:165:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_precondition.ml#L165)
/// Args: UnsignedExtendedUInt32StableV1
///
///
/// Gid: `767`
/// Location: [src/lib/mina_base/zkapp_basic.ml:232:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_basic.ml#L232)
/// Args: MinaBaseZkappPreconditionProtocolStateStableV1LengthA
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseZkappPreconditionProtocolStateStableV1Length {
    Check(MinaBaseZkappPreconditionProtocolStateStableV1LengthA),
    Ignore,
}

/// **OCaml name**: `Mina_base__Zkapp_precondition.Account.Stable.V2`
///
/// Gid: `792`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:465:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_precondition.ml#L465)
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
/// Gid: `793`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:792:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_precondition.ml#L792)
///
///
/// Gid: `788`
/// Location: [src/lib/mina_base/epoch_data.ml:8:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/epoch_data.ml#L8)
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
/// Gid: `795`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:967:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_precondition.ml#L967)
///
///
/// Gid: `794`
/// Location: [src/lib/mina_base/zkapp_precondition.ml:923:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_precondition.ml#L923)
/// Args: MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash , MinaBaseZkappPreconditionProtocolStateStableV1Length , MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlot , MinaBaseZkappPreconditionProtocolStateStableV1Amount , MinaBaseZkappPreconditionProtocolStateEpochDataStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappPreconditionProtocolStateStableV1 {
    pub snarked_ledger_hash: MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash,
    pub blockchain_length: MinaBaseZkappPreconditionProtocolStateStableV1Length,
    pub min_window_density: MinaBaseZkappPreconditionProtocolStateStableV1Length,
    pub total_currency: MinaBaseZkappPreconditionProtocolStateStableV1Amount,
    pub global_slot_since_genesis: MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlot,
    pub staking_epoch_data: MinaBaseZkappPreconditionProtocolStateEpochDataStableV1,
    pub next_epoch_data: MinaBaseZkappPreconditionProtocolStateEpochDataStableV1,
}

/// **OCaml name**: `Mina_base__Account_update.Authorization_kind.Stable.V1`
///
/// Gid: `803`
/// Location: [src/lib/mina_base/account_update.ml:28:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/account_update.ml#L28)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateAuthorizationKindStableV1 {
    Signature,
    Proof(crate::bigint::BigInt),
    NoneGiven,
}

/// **OCaml name**: `Mina_base__Account_update.May_use_token.Stable.V1`
///
/// Gid: `804`
/// Location: [src/lib/mina_base/account_update.ml:161:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/account_update.ml#L161)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBaseAccountUpdateMayUseTokenStableV1 {
    No,
    ParentsOwnToken,
    InheritFromParent,
}

/// **OCaml name**: `Mina_base__Account_update.Update.Timing_info.Stable.V1`
///
/// Gid: `805`
/// Location: [src/lib/mina_base/account_update.ml:532:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/account_update.ml#L532)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountUpdateUpdateTimingInfoStableV1 {
    pub initial_minimum_balance: CurrencyBalanceStableV1,
    pub cliff_time: MinaNumbersGlobalSlotSinceGenesisMStableV1,
    pub cliff_amount: CurrencyAmountStableV1,
    pub vesting_period: MinaNumbersGlobalSlotSpanStableV1,
    pub vesting_increment: CurrencyAmountStableV1,
}

/// **OCaml name**: `Mina_base__Account_update.Update.Stable.V1`
///
/// Gid: `806`
/// Location: [src/lib/mina_base/account_update.ml:692:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/account_update.ml#L692)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountUpdateUpdateStableV1 {
    pub app_state: PaddedSeq<MinaBaseAccountUpdateUpdateStableV1AppStateA, 8>,
    pub delegate: MinaBaseAccountUpdateUpdateStableV1Delegate,
    pub verification_key: MinaBaseAccountUpdateUpdateStableV1VerificationKey,
    pub permissions: MinaBaseAccountUpdateUpdateStableV1Permissions,
    pub zkapp_uri: MinaBaseAccountUpdateUpdateStableV1ZkappUri,
    pub token_symbol: MinaBaseAccountUpdateUpdateStableV1ZkappUri,
    pub timing: MinaBaseAccountUpdateUpdateStableV1Timing,
    pub voting_for: MinaBaseAccountUpdateUpdateStableV1VotingFor,
}

/// **OCaml name**: `Mina_base__Account_update.Account_precondition.Stable.V1`
///
/// Gid: `807`
/// Location: [src/lib/mina_base/account_update.ml:958:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/account_update.ml#L958)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct MinaBaseAccountUpdateAccountPreconditionStableV1(
    pub MinaBaseZkappPreconditionAccountStableV2,
);

/// **OCaml name**: `Mina_base__Account_update.Preconditions.Stable.V1`
///
/// Gid: `808`
/// Location: [src/lib/mina_base/account_update.ml:1029:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/account_update.ml#L1029)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountUpdatePreconditionsStableV1 {
    pub network: MinaBaseZkappPreconditionProtocolStateStableV1,
    pub account: MinaBaseAccountUpdateAccountPreconditionStableV1,
    pub valid_while: MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlot,
}

/// **OCaml name**: `Mina_base__Account_update.Body.Events'.Stable.V1`
///
/// Gid: `809`
/// Location: [src/lib/mina_base/account_update.ml:1116:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/account_update.ml#L1116)
///
///
/// Gid: `167`
/// Location: [src/std_internal.ml:131:2](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/std_internal.ml#L131)
/// Args: Vec < crate :: bigint :: BigInt >
///
///
/// Gid: `50`
/// Location: [src/list0.ml:6:0](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/list0.ml#L6)
/// Args: Vec < crate :: bigint :: BigInt >
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct MinaBaseAccountUpdateBodyEventsStableV1(pub Vec<Vec<crate::bigint::BigInt>>);

/// **OCaml name**: `Mina_base__Account_update.Body.Stable.V1`
///
/// Gid: `812`
/// Location: [src/lib/mina_base/account_update.ml:1216:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/account_update.ml#L1216)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountUpdateBodyStableV1 {
    pub public_key: NonZeroCurvePoint,
    pub token_id: TokenIdKeyHash,
    pub update: MinaBaseAccountUpdateUpdateStableV1,
    pub balance_change: MinaStateBlockchainStateValueStableV2SignedAmount,
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
/// Gid: `813`
/// Location: [src/lib/mina_base/account_update.ml:1322:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/account_update.ml#L1322)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountUpdateBodyFeePayerStableV1 {
    pub public_key: NonZeroCurvePoint,
    pub fee: CurrencyFeeStableV1,
    pub valid_until: Option<MinaNumbersGlobalSlotSinceGenesisMStableV1>,
    pub nonce: UnsignedExtendedUInt32StableV1,
}

/// **OCaml name**: `Mina_base__Account_update.T.Stable.V1`
///
/// Gid: `816`
/// Location: [src/lib/mina_base/account_update.ml:1694:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/account_update.ml#L1694)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountUpdateTStableV1 {
    pub body: MinaBaseAccountUpdateBodyStableV1,
    pub authorization: MinaBaseControlStableV2,
}

/// **OCaml name**: `Mina_base__Account_update.Fee_payer.Stable.V1`
///
/// Gid: `817`
/// Location: [src/lib/mina_base/account_update.ml:1738:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/account_update.ml#L1738)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseAccountUpdateFeePayerStableV1 {
    pub body: MinaBaseAccountUpdateBodyFeePayerStableV1,
    pub authorization: Signature,
}

/// Derived name: `Mina_base__Zkapp_command.T.Stable.V1.Wire.Stable.V1.account_updates.a.a.calls.a`
///
/// Gid: `818`
/// Location: [src/lib/mina_base/with_stack_hash.ml:6:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/with_stack_hash.ml#L6)
/// Args: Box < MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA > , ()
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAACallsA {
    pub elt: Box<MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA>,
    pub stack_hash: (),
}

/// Derived name: `Mina_base__Zkapp_command.T.Stable.V1.Wire.Stable.V1.account_updates.a`
///
/// Gid: `818`
/// Location: [src/lib/mina_base/with_stack_hash.ml:6:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/with_stack_hash.ml#L6)
/// Args: MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA , ()
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA {
    pub elt: MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA,
    pub stack_hash: (),
}

/// Derived name: `Mina_transaction_logic.Transaction_applied.Coinbase_applied.Stable.V2.coinbase`
///
/// Gid: `819`
/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/with_status.ml#L6)
/// Args: MinaBaseCoinbaseStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedCoinbaseAppliedStableV2Coinbase {
    pub data: MinaBaseCoinbaseStableV1,
    pub status: MinaBaseTransactionStatusStableV2,
}

/// Derived name: `Mina_transaction_logic.Transaction_applied.Fee_transfer_applied.Stable.V2.fee_transfer`
///
/// Gid: `819`
/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/with_status.ml#L6)
/// Args: MinaBaseFeeTransferStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedFeeTransferAppliedStableV2FeeTransfer {
    pub data: MinaBaseFeeTransferStableV2,
    pub status: MinaBaseTransactionStatusStableV2,
}

/// Derived name: `Mina_transaction_logic.Transaction_applied.Signed_command_applied.Common.Stable.V2.user_command`
///
/// Gid: `819`
/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/with_status.ml#L6)
/// Args: MinaBaseSignedCommandStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV2UserCommand {
    pub data: MinaBaseSignedCommandStableV2,
    pub status: MinaBaseTransactionStatusStableV2,
}

/// Derived name: `Staged_ledger_diff__Diff.Make_str.Pre_diff_with_at_most_two_coinbase.Stable.V2.b`
///
/// Gid: `819`
/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/with_status.ml#L6)
/// Args: MinaBaseUserCommandStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B {
    pub data: MinaBaseUserCommandStableV2,
    pub status: MinaBaseTransactionStatusStableV2,
}

/// Derived name: `Mina_transaction_logic.Transaction_applied.Zkapp_command_applied.Stable.V1.command`
///
/// Gid: `819`
/// Location: [src/lib/mina_base/with_status.ml:6:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/with_status.ml#L6)
/// Args: MinaBaseZkappCommandTStableV1WireStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedZkappCommandAppliedStableV1Command {
    pub data: MinaBaseZkappCommandTStableV1WireStableV1,
    pub status: MinaBaseTransactionStatusStableV2,
}

/// Derived name: `Mina_base__Zkapp_command.T.Stable.V1.Wire.Stable.V1.account_updates.a.a`
///
/// Gid: `820`
/// Location: [src/lib/mina_base/zkapp_command.ml:11:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_command.ml#L11)
/// Args: MinaBaseAccountUpdateTStableV1 , () , ()
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA {
    pub account_update: MinaBaseAccountUpdateTStableV1,
    pub account_update_digest: (),
    pub calls: Vec<MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAACallsA>,
}

/// **OCaml name**: `Mina_base__Zkapp_command.T.Stable.V1.Wire.Stable.V1`
///
/// Gid: `829`
/// Location: [src/lib/mina_base/zkapp_command.ml:684:12](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/zkapp_command.ml#L684)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseZkappCommandTStableV1WireStableV1 {
    pub fee_payer: MinaBaseAccountUpdateFeePayerStableV1,
    pub account_updates: Vec<MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA>,
    pub memo: MinaBaseSignedCommandMemoStableV1,
}

/// **OCaml name**: `Mina_base__User_command.Stable.V2`
///
/// Gid: `839`
/// Location: [src/lib/mina_base/user_command.ml:79:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/user_command.ml#L79)
///
///
/// Gid: `837`
/// Location: [src/lib/mina_base/user_command.ml:7:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/user_command.ml#L7)
/// Args: MinaBaseSignedCommandStableV2 , MinaBaseZkappCommandTStableV1WireStableV1
#[derive(
    Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, derive_more::From,
)]
pub enum MinaBaseUserCommandStableV2 {
    SignedCommand(MinaBaseSignedCommandStableV2),
    ZkappCommand(MinaBaseZkappCommandTStableV1WireStableV1),
}

/// **OCaml name**: `Mina_base__Fee_transfer.Make_str.Single.Stable.V2`
///
/// Gid: `843`
/// Location: [src/lib/mina_base/fee_transfer.ml:19:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/fee_transfer.ml#L19)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeTransferSingleStableV2 {
    pub receiver_pk: NonZeroCurvePoint,
    pub fee: CurrencyFeeStableV1,
    pub fee_token: TokenIdKeyHash,
}

/// **OCaml name**: `Mina_base__Fee_transfer.Make_str.Stable.V2`
///
/// Gid: `844`
/// Location: [src/lib/mina_base/fee_transfer.ml:69:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/fee_transfer.ml#L69)
///
///
/// Gid: `508`
/// Location: [src/lib/one_or_two/one_or_two.ml:7:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/one_or_two/one_or_two.ml#L7)
/// Args: MinaBaseFeeTransferSingleStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[polymorphic_variant]
pub enum MinaBaseFeeTransferStableV2 {
    #[allow(non_camel_case_types)]
    One(MinaBaseFeeTransferSingleStableV2),
    #[allow(non_camel_case_types)]
    Two(
        (
            MinaBaseFeeTransferSingleStableV2,
            MinaBaseFeeTransferSingleStableV2,
        ),
    ),
}

/// **OCaml name**: `Mina_base__Coinbase_fee_transfer.Make_str.Stable.V1`
///
/// Gid: `845`
/// Location: [src/lib/mina_base/coinbase_fee_transfer.ml:15:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/coinbase_fee_transfer.ml#L15)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseCoinbaseFeeTransferStableV1 {
    pub receiver_pk: NonZeroCurvePoint,
    pub fee: CurrencyFeeStableV1,
}

/// **OCaml name**: `Mina_base__Coinbase.Make_str.Stable.V1`
///
/// Gid: `846`
/// Location: [src/lib/mina_base/coinbase.ml:17:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/coinbase.ml#L17)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseCoinbaseStableV1 {
    pub receiver: NonZeroCurvePoint,
    pub amount: CurrencyAmountStableV1,
    pub fee_transfer: Option<MinaBaseCoinbaseFeeTransferStableV1>,
}

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.Stack_id.Stable.V1`
///
/// Gid: `848`
/// Location: [src/lib/mina_base/pending_coinbase.ml:106:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/pending_coinbase.ml#L106)
///
///
/// Gid: `163`
/// Location: [src/std_internal.ml:119:2](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/std_internal.ml#L119)
///
///
/// Gid: `113`
/// Location: [src/int.ml:19:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/int.ml#L19)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct MinaBasePendingCoinbaseStackIdStableV1(pub crate::number::UInt64);

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.Coinbase_stack.Stable.V1`
///
/// Gid: `851`
/// Location: [src/lib/mina_base/pending_coinbase.ml:159:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/pending_coinbase.ml#L159)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct MinaBasePendingCoinbaseCoinbaseStackStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.Stack_hash.Stable.V1`
///
/// Gid: `856`
/// Location: [src/lib/mina_base/pending_coinbase.ml:219:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/pending_coinbase.ml#L219)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct MinaBasePendingCoinbaseStackHashStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.State_stack.Stable.V1`
///
/// Gid: `860`
/// Location: [src/lib/mina_base/pending_coinbase.ml:255:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/pending_coinbase.ml#L255)
///
///
/// Gid: `859`
/// Location: [src/lib/mina_base/pending_coinbase.ml:245:10](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/pending_coinbase.ml#L245)
/// Args: CoinbaseStackHash
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStateStackStableV1 {
    pub init: CoinbaseStackHash,
    pub curr: CoinbaseStackHash,
}

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.Hash_builder.Stable.V1`
///
/// Gid: `863`
/// Location: [src/lib/mina_base/pending_coinbase.ml:373:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/pending_coinbase.ml#L373)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct MinaBasePendingCoinbaseHashBuilderStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.Update.Action.Stable.V1`
///
/// Gid: `866`
/// Location: [src/lib/mina_base/pending_coinbase.ml:407:10](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/pending_coinbase.ml#L407)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaBasePendingCoinbaseUpdateActionStableV1 {
    UpdateNone,
    UpdateOne,
    UpdateTwoCoinbaseInFirst,
    UpdateTwoCoinbaseInSecond,
}

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.Update.Stable.V1`
///
/// Gid: `868`
/// Location: [src/lib/mina_base/pending_coinbase.ml:473:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/pending_coinbase.ml#L473)
///
///
/// Gid: `867`
/// Location: [src/lib/mina_base/pending_coinbase.ml:463:10](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/pending_coinbase.ml#L463)
/// Args: MinaBasePendingCoinbaseUpdateActionStableV1 , CurrencyAmountStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseUpdateStableV1 {
    pub action: MinaBasePendingCoinbaseUpdateActionStableV1,
    pub coinbase_amount: CurrencyAmountStableV1,
}

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.Stack_versioned.Stable.V1`
///
/// Gid: `870`
/// Location: [src/lib/mina_base/pending_coinbase.ml:522:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/pending_coinbase.ml#L522)
///
///
/// Gid: `869`
/// Location: [src/lib/mina_base/pending_coinbase.ml:511:10](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/pending_coinbase.ml#L511)
/// Args: CoinbaseStackData , MinaBasePendingCoinbaseStateStackStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseStackVersionedStableV1 {
    pub data: CoinbaseStackData,
    pub state: MinaBasePendingCoinbaseStateStackStableV1,
}

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.Hash_versioned.Stable.V1`
///
/// Gid: `871`
/// Location: [src/lib/mina_base/pending_coinbase.ml:535:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/pending_coinbase.ml#L535)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct MinaBasePendingCoinbaseHashVersionedStableV1(
    pub MinaBasePendingCoinbaseHashBuilderStableV1,
);

/// **OCaml name**: `Mina_base__Pending_coinbase.Make_str.Merkle_tree_versioned.Stable.V2`
///
/// Gid: `872`
/// Location: [src/lib/mina_base/pending_coinbase.ml:547:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/pending_coinbase.ml#L547)
///
///
/// Gid: `661`
/// Location: [src/lib/sparse_ledger_lib/sparse_ledger.ml:38:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/sparse_ledger_lib/sparse_ledger.ml#L38)
/// Args: PendingCoinbaseHash , MinaBasePendingCoinbaseStackIdStableV1 , MinaBasePendingCoinbaseStackVersionedStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseMerkleTreeVersionedStableV2 {
    pub indexes: Vec<(
        MinaBasePendingCoinbaseStackIdStableV1,
        crate::number::UInt64,
    )>,
    pub depth: crate::number::UInt64,
    pub tree: MinaBasePendingCoinbaseMerkleTreeVersionedStableV2Tree,
}

/// **OCaml name**: `Mina_base__Staged_ledger_hash.Make_str.Aux_hash.Stable.V1`
///
/// Gid: `875`
/// Location: [src/lib/mina_base/staged_ledger_hash.ml:27:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/staged_ledger_hash.ml#L27)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct MinaBaseStagedLedgerHashAuxHashStableV1(pub crate::string::ByteString);

/// **OCaml name**: `Mina_base__Staged_ledger_hash.Make_str.Pending_coinbase_aux.Stable.V1`
///
/// Gid: `876`
/// Location: [src/lib/mina_base/staged_ledger_hash.ml:111:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/staged_ledger_hash.ml#L111)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct MinaBaseStagedLedgerHashPendingCoinbaseAuxStableV1(pub crate::string::ByteString);

/// **OCaml name**: `Mina_base__Staged_ledger_hash.Make_str.Non_snark.Stable.V1`
///
/// Gid: `877`
/// Location: [src/lib/mina_base/staged_ledger_hash.ml:154:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/staged_ledger_hash.ml#L154)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashNonSnarkStableV1 {
    pub ledger_hash: LedgerHash,
    pub aux_hash: StagedLedgerHashAuxHash,
    pub pending_coinbase_aux: StagedLedgerHashPendingCoinbaseAux,
}

/// **OCaml name**: `Mina_base__Staged_ledger_hash.Make_str.Stable.V1`
///
/// Gid: `879`
/// Location: [src/lib/mina_base/staged_ledger_hash.ml:261:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/staged_ledger_hash.ml#L261)
///
///
/// Gid: `878`
/// Location: [src/lib/mina_base/staged_ledger_hash.ml:243:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/staged_ledger_hash.ml#L243)
/// Args: MinaBaseStagedLedgerHashNonSnarkStableV1 , PendingCoinbaseHash
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseStagedLedgerHashStableV1 {
    pub non_snark: MinaBaseStagedLedgerHashNonSnarkStableV1,
    pub pending_coinbase_hash: PendingCoinbaseHash,
}

/// **OCaml name**: `Mina_base__Stack_frame.Make_str.Stable.V1`
///
/// Gid: `881`
/// Location: [src/lib/mina_base/stack_frame.ml:64:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/stack_frame.ml#L64)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct MinaBaseStackFrameStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Sok_message.Make_str.Stable.V1`
///
/// Gid: `883`
/// Location: [src/lib/mina_base/sok_message.ml:14:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/sok_message.ml#L14)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseSokMessageStableV1 {
    pub fee: CurrencyFeeStableV1,
    pub prover: NonZeroCurvePoint,
}

/// **OCaml name**: `Mina_base__Protocol_constants_checked.Value.Stable.V1`
///
/// Gid: `884`
/// Location: [src/lib/mina_base/protocol_constants_checked.ml:22:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/protocol_constants_checked.ml#L22)
///
///
/// Gid: `657`
/// Location: [src/lib/genesis_constants/genesis_constants.ml:240:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/genesis_constants/genesis_constants.ml#L240)
/// Args: UnsignedExtendedUInt32StableV1 , UnsignedExtendedUInt32StableV1 , BlockTimeTimeStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseProtocolConstantsCheckedValueStableV1 {
    pub k: UnsignedExtendedUInt32StableV1,
    pub slots_per_epoch: UnsignedExtendedUInt32StableV1,
    pub slots_per_sub_window: UnsignedExtendedUInt32StableV1,
    pub grace_period_slots: UnsignedExtendedUInt32StableV1,
    pub delta: UnsignedExtendedUInt32StableV1,
    pub genesis_state_timestamp: BlockTimeTimeStableV1,
}

/// **OCaml name**: `Mina_base__Proof.Stable.V2`
///
/// Gid: `885`
/// Location: [src/lib/mina_base/proof.ml:12:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/proof.ml#L12)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct MinaBaseProofStableV2(pub PicklesProofProofsVerified2ReprStableV2);

/// **OCaml name**: `Mina_base__Pending_coinbase_witness.Stable.V2`
///
/// Gid: `886`
/// Location: [src/lib/mina_base/pending_coinbase_witness.ml:6:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/pending_coinbase_witness.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBasePendingCoinbaseWitnessStableV2 {
    pub pending_coinbases: MinaBasePendingCoinbaseStableV2,
    pub is_new_stack: bool,
}

/// **OCaml name**: `Mina_base__Call_stack_digest.Make_str.Stable.V1`
///
/// Gid: `887`
/// Location: [src/lib/mina_base/call_stack_digest.ml:12:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/call_stack_digest.ml#L12)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct MinaBaseCallStackDigestStableV1(pub crate::bigint::BigInt);

/// **OCaml name**: `Mina_base__Fee_with_prover.Stable.V1`
///
/// Gid: `888`
/// Location: [src/lib/mina_base/fee_with_prover.ml:7:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/fee_with_prover.ml#L7)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeWithProverStableV1 {
    pub fee: CurrencyFeeStableV1,
    pub prover: NonZeroCurvePoint,
}

/// **OCaml name**: `Network_peer__Peer.Id.Stable.V1`
///
/// Gid: `889`
/// Location: [src/lib/network_peer/peer.ml:10:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/network_peer/peer.ml#L10)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct NetworkPeerPeerIdStableV1(pub crate::string::ByteString);

/// **OCaml name**: `Mina_transaction__Transaction.Stable.V2`
///
/// Gid: `896`
/// Location: [src/lib/transaction/transaction.ml:46:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/transaction/transaction.ml#L46)
///
///
/// Gid: `894`
/// Location: [src/lib/transaction/transaction.ml:8:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/transaction/transaction.ml#L8)
/// Args: MinaBaseUserCommandStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaTransactionTransactionStableV2 {
    Command(Box<MinaBaseUserCommandStableV2>),
    FeeTransfer(MinaBaseFeeTransferStableV2),
    Coinbase(MinaBaseCoinbaseStableV1),
}

/// **OCaml name**: `Mina_transaction_logic__Zkapp_command_logic.Local_state.Value.Stable.V1`
///
/// Gid: `906`
/// Location: [src/lib/transaction_logic/zkapp_command_logic.ml:255:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/transaction_logic/zkapp_command_logic.ml#L255)
///
///
/// Gid: `905`
/// Location: [src/lib/transaction_logic/zkapp_command_logic.ml:196:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/transaction_logic/zkapp_command_logic.ml#L196)
/// Args: MinaBaseStackFrameStableV1 , MinaBaseCallStackDigestStableV1 , SignedAmount , LedgerHash , bool , crate :: bigint :: BigInt , UnsignedExtendedUInt32StableV1 , MinaBaseTransactionStatusFailureCollectionStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1 {
    pub stack_frame: MinaBaseStackFrameStableV1,
    pub call_stack: MinaBaseCallStackDigestStableV1,
    pub transaction_commitment: crate::bigint::BigInt,
    pub full_transaction_commitment: crate::bigint::BigInt,
    pub excess: SignedAmount,
    pub supply_increase: SignedAmount,
    pub ledger: LedgerHash,
    pub success: bool,
    pub account_update_index: UnsignedExtendedUInt32StableV1,
    pub failure_status_tbl: MinaBaseTransactionStatusFailureCollectionStableV1,
    pub will_succeed: bool,
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Signed_command_applied.Common.Stable.V2`
///
/// Gid: `908`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:17:10](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/transaction_logic/mina_transaction_logic.ml#L17)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV2 {
    pub user_command:
        MinaTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV2UserCommand,
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Signed_command_applied.Body.Stable.V2`
///
/// Gid: `909`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:31:10](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/transaction_logic/mina_transaction_logic.ml#L31)
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
/// Gid: `910`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:46:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/transaction_logic/mina_transaction_logic.ml#L46)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedSignedCommandAppliedStableV2 {
    pub common: MinaTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV2,
    pub body: MinaTransactionLogicTransactionAppliedSignedCommandAppliedBodyStableV2,
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Zkapp_command_applied.Stable.V1`
///
/// Gid: `911`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:65:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/transaction_logic/mina_transaction_logic.ml#L65)
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
/// Gid: `912`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:82:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/transaction_logic/mina_transaction_logic.ml#L82)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaTransactionLogicTransactionAppliedCommandAppliedStableV2 {
    SignedCommand(MinaTransactionLogicTransactionAppliedSignedCommandAppliedStableV2),
    ZkappCommand(MinaTransactionLogicTransactionAppliedZkappCommandAppliedStableV1),
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Fee_transfer_applied.Stable.V2`
///
/// Gid: `913`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:96:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/transaction_logic/mina_transaction_logic.ml#L96)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedFeeTransferAppliedStableV2 {
    pub fee_transfer: MinaTransactionLogicTransactionAppliedFeeTransferAppliedStableV2FeeTransfer,
    pub new_accounts: Vec<MinaBaseAccountIdStableV2>,
    pub burned_tokens: CurrencyAmountStableV1,
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Coinbase_applied.Stable.V2`
///
/// Gid: `914`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:112:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/transaction_logic/mina_transaction_logic.ml#L112)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedCoinbaseAppliedStableV2 {
    pub coinbase: MinaTransactionLogicTransactionAppliedCoinbaseAppliedStableV2Coinbase,
    pub new_accounts: Vec<MinaBaseAccountIdStableV2>,
    pub burned_tokens: CurrencyAmountStableV1,
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Varying.Stable.V2`
///
/// Gid: `915`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:128:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/transaction_logic/mina_transaction_logic.ml#L128)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaTransactionLogicTransactionAppliedVaryingStableV2 {
    Command(MinaTransactionLogicTransactionAppliedCommandAppliedStableV2),
    FeeTransfer(MinaTransactionLogicTransactionAppliedFeeTransferAppliedStableV2),
    Coinbase(MinaTransactionLogicTransactionAppliedCoinbaseAppliedStableV2),
}

/// **OCaml name**: `Mina_transaction_logic.Transaction_applied.Stable.V2`
///
/// Gid: `916`
/// Location: [src/lib/transaction_logic/mina_transaction_logic.ml:142:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/transaction_logic/mina_transaction_logic.ml#L142)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaTransactionLogicTransactionAppliedStableV2 {
    pub previous_hash: LedgerHash,
    pub varying: MinaTransactionLogicTransactionAppliedVaryingStableV2,
}

/// **OCaml name**: `Merkle_address.Binable_arg.Stable.V1`
///
/// Gid: `917`
/// Location: [src/lib/merkle_address/merkle_address.ml:48:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/merkle_address/merkle_address.ml#L48)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MerkleAddressBinableArgStableV1(
    pub crate::number::UInt64,
    pub crate::string::ByteString,
);

/// **OCaml name**: `Trust_system__Banned_status.Stable.V1`
///
/// Gid: `924`
/// Location: [src/lib/trust_system/banned_status.ml:6:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/trust_system/banned_status.ml#L6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum TrustSystemBannedStatusStableV1 {
    Unbanned,
    BannedUntil(crate::number::Float64),
}

/// **OCaml name**: `Consensus_vrf.Output.Truncated.Stable.V1`
///
/// Gid: `941`
/// Location: [src/lib/consensus/vrf/consensus_vrf.ml:168:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/consensus/vrf/consensus_vrf.ml#L168)
#[derive(Clone, Debug, PartialEq, BinProtRead, BinProtWrite, Deref)]
pub struct ConsensusVrfOutputTruncatedStableV1(pub crate::string::ByteString);

/// **OCaml name**: `Consensus__Stake_proof.Stable.V2`
///
/// Gid: `951`
/// Location: [src/lib/consensus/stake_proof.ml:10:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/consensus/stake_proof.ml#L10)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusStakeProofStableV2 {
    pub delegator: MinaBaseAccountIndexStableV1,
    pub delegator_pk: NonZeroCurvePoint,
    pub coinbase_receiver_pk: NonZeroCurvePoint,
    pub ledger: MinaBaseSparseLedgerBaseStableV2,
    pub producer_private_key: SignatureLibPrivateKeyStableV1,
    pub producer_public_key: NonZeroCurvePoint,
}

/// **OCaml name**: `Consensus__Body_reference.Stable.V1`
///
/// Gid: `959`
/// Location: [src/lib/consensus/body_reference.ml:17:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/consensus/body_reference.ml#L17)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct ConsensusBodyReferenceStableV1(pub crate::string::ByteString);

/// **OCaml name**: `Consensus__Global_slot.Make_str.Stable.V1`
///
/// Gid: `966`
/// Location: [src/lib/consensus/global_slot.ml:33:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/consensus/global_slot.ml#L33)
///
///
/// Gid: `965`
/// Location: [src/lib/consensus/global_slot.ml:22:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/consensus/global_slot.ml#L22)
/// Args: MinaNumbersGlobalSlotSinceHardForkMStableV1 , UnsignedExtendedUInt32StableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ConsensusGlobalSlotStableV1 {
    pub slot_number: MinaNumbersGlobalSlotSinceHardForkMStableV1,
    pub slots_per_epoch: UnsignedExtendedUInt32StableV1,
}

/// **OCaml name**: `Consensus__Proof_of_stake.Make_str.Data.Epoch_data.Staking_value_versioned.Value.Stable.V1`
///
/// Gid: `981`
/// Location: [src/lib/consensus/proof_of_stake.ml:1072:14](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/consensus/proof_of_stake.ml#L1072)
///
///
/// Gid: `788`
/// Location: [src/lib/mina_base/epoch_data.ml:8:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/epoch_data.ml#L8)
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
/// Gid: `982`
/// Location: [src/lib/consensus/proof_of_stake.ml:1097:14](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/consensus/proof_of_stake.ml#L1097)
///
///
/// Gid: `788`
/// Location: [src/lib/mina_base/epoch_data.ml:8:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_base/epoch_data.ml#L8)
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
/// Gid: `985`
/// Location: [src/lib/mina_state/registers.ml:8:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_state/registers.ml#L8)
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
/// Gid: `986`
/// Location: [src/lib/mina_state/snarked_ledger_state.ml:38:10](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_state/snarked_ledger_state.ml#L38)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum MinaStateSnarkedLedgerStatePendingCoinbaseStackStateInitStackStableV1 {
    Base(MinaBasePendingCoinbaseStackVersionedStableV1),
    Merge,
}

/// Derived name: `Mina_state__Blockchain_state.Value.Stable.V2.ledger_proof_statement`
///
/// Gid: `991`
/// Location: [src/lib/mina_state/snarked_ledger_state.ml:107:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_state/snarked_ledger_state.ml#L107)
/// Args: LedgerHash , MinaStateBlockchainStateValueStableV2SignedAmount , MinaBasePendingCoinbaseStackVersionedStableV1 , MinaBaseFeeExcessStableV1 , () , MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateBlockchainStateValueStableV2LedgerProofStatement {
    pub source: MinaStateBlockchainStateValueStableV2LedgerProofStatementSource,
    pub target: MinaStateBlockchainStateValueStableV2LedgerProofStatementSource,
    pub connecting_ledger_left: LedgerHash,
    pub connecting_ledger_right: LedgerHash,
    pub supply_increase: MinaStateBlockchainStateValueStableV2SignedAmount,
    pub fee_excess: MinaBaseFeeExcessStableV1,
    pub sok_digest: (),
}

/// **OCaml name**: `Mina_state__Snarked_ledger_state.Make_str.Stable.V2`
///
/// Gid: `992`
/// Location: [src/lib/mina_state/snarked_ledger_state.ml:191:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_state/snarked_ledger_state.ml#L191)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct MinaStateSnarkedLedgerStateStableV2(
    pub MinaStateBlockchainStateValueStableV2LedgerProofStatement,
);

/// **OCaml name**: `Mina_state__Snarked_ledger_state.Make_str.With_sok.Stable.V2`
///
/// Gid: `993`
/// Location: [src/lib/mina_state/snarked_ledger_state.ml:345:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_state/snarked_ledger_state.ml#L345)
///
///
/// Gid: `991`
/// Location: [src/lib/mina_state/snarked_ledger_state.ml:107:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_state/snarked_ledger_state.ml#L107)
/// Args: LedgerHash , MinaStateBlockchainStateValueStableV2SignedAmount , MinaBasePendingCoinbaseStackVersionedStableV1 , MinaBaseFeeExcessStableV1 , crate :: string :: ByteString , MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateSnarkedLedgerStateWithSokStableV2 {
    pub source: MinaStateBlockchainStateValueStableV2LedgerProofStatementSource,
    pub target: MinaStateBlockchainStateValueStableV2LedgerProofStatementSource,
    pub connecting_ledger_left: LedgerHash,
    pub connecting_ledger_right: LedgerHash,
    pub supply_increase: MinaStateBlockchainStateValueStableV2SignedAmount,
    pub fee_excess: MinaBaseFeeExcessStableV1,
    pub sok_digest: crate::string::ByteString,
}

/// **OCaml name**: `Mina_state__Blockchain_state.Value.Stable.V2`
///
/// Gid: `997`
/// Location: [src/lib/mina_state/blockchain_state.ml:68:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_state/blockchain_state.ml#L68)
///
///
/// Gid: `996`
/// Location: [src/lib/mina_state/blockchain_state.ml:10:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_state/blockchain_state.ml#L10)
/// Args: MinaBaseStagedLedgerHashStableV1 , LedgerHash , MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1 , BlockTimeTimeStableV1 , ConsensusBodyReferenceStableV1 , MinaStateBlockchainStateValueStableV2SignedAmount , MinaBasePendingCoinbaseStackVersionedStableV1 , MinaBaseFeeExcessStableV1 , ()
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateBlockchainStateValueStableV2 {
    pub staged_ledger_hash: MinaBaseStagedLedgerHashStableV1,
    pub genesis_ledger_hash: LedgerHash,
    pub ledger_proof_statement: MinaStateBlockchainStateValueStableV2LedgerProofStatement,
    pub timestamp: BlockTimeTimeStableV1,
    pub body_reference: ConsensusBodyReferenceStableV1,
}

/// **OCaml name**: `Mina_state__Snark_transition.Value.Stable.V2`
///
/// Gid: `999`
/// Location: [src/lib/mina_state/snark_transition.ml:25:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_state/snark_transition.ml#L25)
///
///
/// Gid: `998`
/// Location: [src/lib/mina_state/snark_transition.ml:8:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_state/snark_transition.ml#L8)
/// Args: MinaStateBlockchainStateValueStableV2 , MinaNumbersGlobalSlotSinceHardForkMStableV1 , MinaBasePendingCoinbaseUpdateStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateSnarkTransitionValueStableV2 {
    pub blockchain_state: MinaStateBlockchainStateValueStableV2,
    pub consensus_transition: MinaNumbersGlobalSlotSinceHardForkMStableV1,
    pub pending_coinbase_update: MinaBasePendingCoinbaseUpdateStableV1,
}

/// **OCaml name**: `Mina_state__Protocol_state.Make_str.Body.Value.Stable.V2`
///
/// Gid: `1003`
/// Location: [src/lib/mina_state/protocol_state.ml:82:10](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_state/protocol_state.ml#L82)
///
///
/// Gid: `1001`
/// Location: [src/lib/mina_state/protocol_state.ml:62:10](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_state/protocol_state.ml#L62)
/// Args: StateHash , MinaStateBlockchainStateValueStableV2 , ConsensusProofOfStakeDataConsensusStateValueStableV2 , MinaBaseProtocolConstantsCheckedValueStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaStateProtocolStateBodyValueStableV2 {
    pub genesis_state_hash: StateHash,
    pub blockchain_state: MinaStateBlockchainStateValueStableV2,
    pub consensus_state: ConsensusProofOfStakeDataConsensusStateValueStableV2,
    pub constants: MinaBaseProtocolConstantsCheckedValueStableV1,
}

/// **OCaml name**: `Transaction_snark.Make_str.Proof.Stable.V2`
///
/// Gid: `1011`
/// Location: [src/lib/transaction_snark/transaction_snark.ml:69:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/transaction_snark/transaction_snark.ml#L69)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct TransactionSnarkProofStableV2(pub PicklesProofProofsVerified2ReprStableV2);

/// **OCaml name**: `Transaction_snark.Make_str.Stable.V2`
///
/// Gid: `1012`
/// Location: [src/lib/transaction_snark/transaction_snark.ml:80:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/transaction_snark/transaction_snark.ml#L80)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkStableV2 {
    pub statement: MinaStateSnarkedLedgerStateWithSokStableV2,
    pub proof: TransactionSnarkProofStableV2,
}

/// **OCaml name**: `Ledger_proof.Prod.Stable.V2`
///
/// Gid: `1014`
/// Location: [src/lib/ledger_proof/ledger_proof.ml:10:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/ledger_proof/ledger_proof.ml#L10)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct LedgerProofProdStableV2(pub TransactionSnarkStableV2);

/// **OCaml name**: `Transaction_snark_work.Statement.Stable.V2`
///
/// Gid: `1016`
/// Location: [src/lib/transaction_snark_work/transaction_snark_work.ml:23:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/transaction_snark_work/transaction_snark_work.ml#L23)
///
///
/// Gid: `508`
/// Location: [src/lib/one_or_two/one_or_two.ml:7:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/one_or_two/one_or_two.ml#L7)
/// Args: MinaStateSnarkedLedgerStateStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[polymorphic_variant]
pub enum TransactionSnarkWorkStatementStableV2 {
    #[allow(non_camel_case_types)]
    One(MinaStateSnarkedLedgerStateStableV2),
    #[allow(non_camel_case_types)]
    Two(
        (
            MinaStateSnarkedLedgerStateStableV2,
            MinaStateSnarkedLedgerStateStableV2,
        ),
    ),
}

/// **OCaml name**: `Transaction_snark_work.T.Stable.V2`
///
/// Gid: `1024`
/// Location: [src/lib/transaction_snark_work/transaction_snark_work.ml:83:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/transaction_snark_work/transaction_snark_work.ml#L83)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkWorkTStableV2 {
    pub fee: CurrencyFeeStableV1,
    pub proofs: TransactionSnarkWorkTStableV2Proofs,
    pub prover: NonZeroCurvePoint,
}

/// Derived name: `Staged_ledger_diff__Diff.Make_str.Pre_diff_with_at_most_two_coinbase.Stable.V2.coinbase`
///
/// Gid: `1025`
/// Location: [src/lib/staged_ledger_diff/diff.ml:28:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/staged_ledger_diff/diff.ml#L28)
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
/// Gid: `1026`
/// Location: [src/lib/staged_ledger_diff/diff.ml:64:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/staged_ledger_diff/diff.ml#L64)
/// Args: StagedLedgerDiffDiffFtStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2Coinbase {
    Zero,
    One(Option<StagedLedgerDiffDiffFtStableV1>),
}

/// **OCaml name**: `Staged_ledger_diff__Diff.Make_str.Ft.Stable.V1`
///
/// Gid: `1027`
/// Location: [src/lib/staged_ledger_diff/diff.ml:88:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/staged_ledger_diff/diff.ml#L88)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct StagedLedgerDiffDiffFtStableV1(pub MinaBaseCoinbaseFeeTransferStableV1);

/// **OCaml name**: `Staged_ledger_diff__Diff.Make_str.Pre_diff_with_at_most_two_coinbase.Stable.V2`
///
/// Gid: `1030`
/// Location: [src/lib/staged_ledger_diff/diff.ml:168:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/staged_ledger_diff/diff.ml#L168)
///
///
/// Gid: `1028`
/// Location: [src/lib/staged_ledger_diff/diff.ml:104:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/staged_ledger_diff/diff.ml#L104)
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
/// Gid: `1031`
/// Location: [src/lib/staged_ledger_diff/diff.ml:187:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/staged_ledger_diff/diff.ml#L187)
///
///
/// Gid: `1029`
/// Location: [src/lib/staged_ledger_diff/diff.ml:136:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/staged_ledger_diff/diff.ml#L136)
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
/// Gid: `1032`
/// Location: [src/lib/staged_ledger_diff/diff.ml:206:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/staged_ledger_diff/diff.ml#L206)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffDiffStableV2(
    pub StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2,
    pub Option<StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2>,
);

/// **OCaml name**: `Staged_ledger_diff__Diff.Make_str.Stable.V2`
///
/// Gid: `1033`
/// Location: [src/lib/staged_ledger_diff/diff.ml:223:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/staged_ledger_diff/diff.ml#L223)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffDiffStableV2 {
    pub diff: StagedLedgerDiffDiffDiffStableV2,
}

/// **OCaml name**: `Staged_ledger_diff__Body.Make_str.Stable.V1`
///
/// Gid: `1034`
/// Location: [src/lib/staged_ledger_diff/body.ml:18:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/staged_ledger_diff/body.ml#L18)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct StagedLedgerDiffBodyStableV1 {
    pub staged_ledger_diff: StagedLedgerDiffDiffStableV2,
}

/// Derived name: `Snark_worker.Worker.Rpcs_versioned.Get_work.V2.T.response.a.0.single`
///
/// Gid: `1038`
/// Location: [src/lib/snark_work_lib/work.ml:12:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/snark_work_lib/work.ml#L12)
/// Args: TransactionWitnessStableV2 , LedgerProofProdStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Single {
    Transition(
        MinaStateSnarkedLedgerStateStableV2,
        TransactionWitnessStableV2,
    ),
    Merge(
        Box<(
            MinaStateSnarkedLedgerStateStableV2,
            LedgerProofProdStableV2,
            LedgerProofProdStableV2,
        )>,
    ),
}

/// Derived name: `Snark_worker.Worker.Rpcs_versioned.Get_work.V2.T.response.a.0`
///
/// Gid: `1039`
/// Location: [src/lib/snark_work_lib/work.ml:61:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/snark_work_lib/work.ml#L61)
/// Args: SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Single
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0 {
    pub instances: SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Instances,
    pub fee: CurrencyFeeStableV1,
}

/// **OCaml name**: `Parallel_scan.Sequence_number.Stable.V1`
///
/// Gid: `1041`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:22:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/parallel_scan/parallel_scan.ml#L22)
///
///
/// Gid: `163`
/// Location: [src/std_internal.ml:119:2](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/std_internal.ml#L119)
///
///
/// Gid: `113`
/// Location: [src/int.ml:19:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/int.ml#L19)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct ParallelScanSequenceNumberStableV1(pub crate::number::UInt64);

/// **OCaml name**: `Parallel_scan.Job_status.Stable.V1`
///
/// Gid: `1042`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:35:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/parallel_scan/parallel_scan.ml#L35)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum ParallelScanJobStatusStableV1 {
    Todo,
    Done,
}

/// **OCaml name**: `Parallel_scan.Weight.Stable.V1`
///
/// Gid: `1043`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:53:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/parallel_scan/parallel_scan.ml#L53)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ParallelScanWeightStableV1 {
    pub base: crate::number::UInt64,
    pub merge: crate::number::UInt64,
}

/// Derived name: `Transaction_snark_scan_state.Stable.V2.scan_state.trees.a.base_t.1.Full`
///
/// Gid: `1044`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:68:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/parallel_scan/parallel_scan.ml#L68)
/// Args: TransactionSnarkScanStateTransactionWithWitnessStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateStableV2ScanStateTreesABaseT1Full {
    pub job: TransactionSnarkScanStateTransactionWithWitnessStableV2,
    pub seq_no: ParallelScanSequenceNumberStableV1,
    pub status: ParallelScanJobStatusStableV1,
}

/// Derived name: `Transaction_snark_scan_state.Stable.V2.scan_state.trees.a.base_t.1`
///
/// Gid: `1045`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:84:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/parallel_scan/parallel_scan.ml#L84)
/// Args: TransactionSnarkScanStateTransactionWithWitnessStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum TransactionSnarkScanStateStableV2ScanStateTreesABaseT1 {
    Empty,
    Full(Box<TransactionSnarkScanStateStableV2ScanStateTreesABaseT1Full>),
}

/// Derived name: `Transaction_snark_scan_state.Stable.V2.scan_state.trees.a.merge_t.1.Full`
///
/// Gid: `1047`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:112:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/parallel_scan/parallel_scan.ml#L112)
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
/// Gid: `1048`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:130:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/parallel_scan/parallel_scan.ml#L130)
/// Args: TransactionSnarkScanStateLedgerProofWithSokMessageStableV2
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum TransactionSnarkScanStateStableV2ScanStateTreesAMergeT1 {
    Empty,
    Part(Box<TransactionSnarkScanStateLedgerProofWithSokMessageStableV2>),
    Full(Box<TransactionSnarkScanStateStableV2ScanStateTreesAMergeT1Full>),
}

/// Derived name: `Transaction_snark_scan_state.Stable.V2.scan_state`
///
/// Gid: `1055`
/// Location: [src/lib/parallel_scan/parallel_scan.ml:803:8](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/parallel_scan/parallel_scan.ml#L803)
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
    pub curr_job_seq_no: crate::number::UInt64,
    pub max_base_jobs: crate::number::UInt64,
    pub delay: crate::number::UInt64,
}

/// **OCaml name**: `Transaction_snark_scan_state.Transaction_with_witness.Stable.V2`
///
/// Gid: `1056`
/// Location: [src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml:40:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml#L40)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateTransactionWithWitnessStableV2 {
    pub transaction_with_info: MinaTransactionLogicTransactionAppliedStableV2,
    pub state_hash: (StateHash, StateBodyHash),
    pub statement: MinaStateSnarkedLedgerStateStableV2,
    pub init_stack: MinaStateSnarkedLedgerStatePendingCoinbaseStackStateInitStackStableV1,
    pub first_pass_ledger_witness: MinaBaseSparseLedgerBaseStableV2,
    pub second_pass_ledger_witness: MinaBaseSparseLedgerBaseStableV2,
    pub block_global_slot: MinaNumbersGlobalSlotSinceGenesisMStableV1,
}

/// **OCaml name**: `Transaction_snark_scan_state.Ledger_proof_with_sok_message.Stable.V2`
///
/// Gid: `1057`
/// Location: [src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml:65:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml#L65)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TransactionSnarkScanStateLedgerProofWithSokMessageStableV2(
    pub LedgerProofProdStableV2,
    pub MinaBaseSokMessageStableV1,
);

/// **OCaml name**: `Mina_block__Header.Make_str.Stable.V2`
///
/// Gid: `1101`
/// Location: [src/lib/mina_block/header.ml:21:6](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/mina_block/header.ml#L21)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBlockHeaderStableV2 {
    pub protocol_state: MinaStateProtocolStateValueStableV2,
    pub protocol_state_proof: MinaBaseProofStableV2,
    pub delta_block_chain_proof: (StateHash, Vec<StateBodyHash>),
    pub current_protocol_version: ProtocolVersionStableV2,
    pub proposed_protocol_version_opt: Option<ProtocolVersionStableV2>,
}

/// Derived name: `Network_pool__Snark_pool.Diff_versioned.Stable.V2.Add_solved_work.1`
///
/// Gid: `1121`
/// Location: [src/lib/network_pool/priced_proof.ml:9:4](https://github.com/MinaProtocol/mina/blob/1551e2faaa/src/lib/network_pool/priced_proof.ml#L9)
/// Args: TransactionSnarkWorkTStableV2Proofs
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NetworkPoolSnarkPoolDiffVersionedStableV2AddSolvedWork1 {
    pub proof: TransactionSnarkWorkTStableV2Proofs,
    pub fee: MinaBaseFeeWithProverStableV1,
}

/// Derived name: `Snark_worker.Worker.Rpcs_versioned.Submit_work.V2.T.query.metrics.a.1`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[polymorphic_variant]
pub enum SnarkWorkerWorkerRpcsVersionedSubmitWorkV2TQueryMetricsA1 {
    #[allow(non_camel_case_types)]
    Transition,
    #[allow(non_camel_case_types)]
    Merge,
}

/// Derived name: `Transaction_snark_scan_state.Stable.V2.previous_incomplete_zkapp_updates.1`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[polymorphic_variant]
pub enum TransactionSnarkScanStateStableV2PreviousIncompleteZkappUpdates1 {
    #[allow(non_camel_case_types)]
    Border_block_continued_in_the_next_tree(bool),
}
