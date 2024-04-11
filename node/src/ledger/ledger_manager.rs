use mina_p2p_messages::v2::{
    LedgerHash, MinaBaseAccountBinableArgStableV2, MinaBaseSparseLedgerBaseStableV2,
    MinaLedgerSyncLedgerAnswerStableV2, MinaLedgerSyncLedgerQueryStableV1,
    MinaStateProtocolStateValueStableV2, NonZeroCurvePoint, StateHash,
};
use openmina_core::block::ArcBlockWithHash;
use openmina_core::channels::mpsc;
use openmina_core::snark::{Snark, SnarkJobId};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::mpsc::{channel, RecvError, Sender};
use std::sync::Arc;
use std::thread;

use super::ledger_service::LedgerCtx;
use crate::block_producer::vrf_evaluator::{
    BlockProducerVrfEvaluatorLedgerService, DelegatorTable,
};
use crate::block_producer::{
    BlockProducerLedgerService, BlockProducerWonSlot, StagedLedgerDiffCreateOutput,
};
use crate::ledger::LedgerAddress;
use crate::p2p::channels::rpc::StagedLedgerAuxAndPendingCoinbases;
use crate::rpc::{RpcLedgerService, RpcScanStateSummaryScanStateJob};
use crate::transition_frontier::sync::{
    ledger::snarked::TransitionFrontierSyncLedgerSnarkedService,
    ledger::staged::{
        StagedLedgerAuxAndPendingCoinbasesValid, TransitionFrontierSyncLedgerStagedService,
    },
    TransitionFrontierRootSnarkedLedgerUpdates,
};
use crate::transition_frontier::{CommitResult, TransitionFrontierService};
use ledger::Mask;
use mina_signer::CompressedPubKey;
use openmina_node_account::AccountPublicKey;

/// The type enumerating different requests that can be made to the
/// service. Each specific constructor has a specific response
/// constructor associated with it. Unfortunately, this relationship
/// can't be expressed in the Rust type system at the moment. For this
/// reason this type is private while functions wrapping the whole call
/// to the service are exposed as the service's methods.
enum LedgerRequest {
    AccountsSet {
        snarked_ledger_hash: LedgerHash,
        parent: LedgerAddress,
        accounts: Vec<MinaBaseAccountBinableArgStableV2>,
    }, // expected response: LedgerHash
    BlockApply {
        block: ArcBlockWithHash,
        pred_block: ArcBlockWithHash,
    }, // expected response: Success
    ChildHashesGet {
        snarked_ledger_hash: LedgerHash,
        parent: LedgerAddress,
    }, // expected response: ChildHashes
    Commit {
        ledgers_to_keep: BTreeSet<LedgerHash>,
        root_snarked_ledger_updates: TransitionFrontierRootSnarkedLedgerUpdates,
        needed_protocol_states: BTreeMap<StateHash, MinaStateProtocolStateValueStableV2>,
        new_root: ArcBlockWithHash,
        new_best_tip: ArcBlockWithHash,
    }, // expected response: CommitResult
    ComputeSnarkedLedgerHashes {
        snarked_ledger_hash: LedgerHash,
    }, // expected response: Success
    CopySnarkedLedgerContentsForSync {
        origin_snarked_ledger_hash: LedgerHash,
        target_snarked_ledger_hash: LedgerHash,
        overwrite: bool,
    }, // expected response: SnarkedLedgerContentsCopied
    GetProducerAndDelegates {
        ledger_hash: LedgerHash,
        producer: AccountPublicKey,
    }, // expected response: ProducerAndDelegates
    GetProducersWithDelegates {
        ledger_hash: LedgerHash,
        filter: fn(&CompressedPubKey) -> bool,
    }, // expected response: ProducersWithDelegatesMap
    GetMask {
        ledger_hash: LedgerHash,
    }, // expected response: LedgerMask
    GetScanStateSummary {
        ledger_hash: LedgerHash,
    }, // expected response: ScanStateSummary
    InsertGenesisLedger {
        mask: Mask,
    }, // expected response: Success
    LedgerQuery {
        ledger_hash: LedgerHash,
        query: MinaLedgerSyncLedgerQueryStableV1,
    }, // expected response: LedgerQueryResult
    StagedLedgerAuxAndPendingCoinbase {
        ledger_hash: LedgerHash,
        protocol_states: BTreeMap<StateHash, MinaStateProtocolStateValueStableV2>,
    }, // expected response: LedgerAuxAndCoinbaseResult
    StagedLedgerDiffCreate {
        pred_block: ArcBlockWithHash,
        won_slot: BlockProducerWonSlot,
        coinbase_receiver: NonZeroCurvePoint,
        completed_snarks: BTreeMap<SnarkJobId, Snark>,
        supercharge_coinbase: bool,
    }, // expected response: StagedLedgerDiff
    StagedLedgerReconstruct {
        snarked_ledger_hash: LedgerHash,
        parts: Option<Arc<StagedLedgerAuxAndPendingCoinbasesValid>>,
    }, // expected response: Success
    StagedLedgerReconstructResultStore {
        staged_ledger_hash: LedgerHash,
    }, // expected response: Success
    StakeProofSparseLedger {
        staking_ledger: LedgerHash,
        producer: NonZeroCurvePoint,
        delegator: NonZeroCurvePoint,
    }, // expected response: SparseLedgerBase
}

#[derive(Debug)]
pub enum LedgerResponse {
    ChildHashes(LedgerHash, LedgerHash),
    CommitResult(CommitResult),
    LedgerAuxAndCoinbaseResult(Option<Arc<StagedLedgerAuxAndPendingCoinbases>>),
    LedgerHash(LedgerHash),
    LedgerMask(Option<(Mask, bool)>),
    LedgerQueryResult(Option<MinaLedgerSyncLedgerAnswerStableV2>),
    ProducerAndDelegates(DelegatorTable),
    ProducersWithDelegatesMap(
        Option<BTreeMap<AccountPublicKey, Vec<(ledger::AccountIndex, AccountPublicKey, u64)>>>,
    ),
    ScanStateSummary(Vec<Vec<RpcScanStateSummaryScanStateJob>>),
    SnarkedLedgerContentsCopied(bool),
    SparseLedgerBase(Option<MinaBaseSparseLedgerBaseStableV2>),
    StagedLedgerDiff(StagedLedgerDiffCreateOutput),
    Success, // operation was performed and result stored; nothing to return.
}

struct LedgerRequestWithChan {
    request: LedgerRequest,
    responder: Sender<Result<LedgerResponse, String>>,
}

pub struct LedgerManager {
    sender: mpsc::UnboundedSender<LedgerRequestWithChan>,
    join_handle: thread::JoinHandle<LedgerCtx>,
}
