use super::LedgerAddress;
use ledger::Mask;
use mina_p2p_messages::v2::{
    LedgerHash, MinaBaseAccountBinableArgStableV2, MinaBaseSparseLedgerBaseStableV2,
    MinaLedgerSyncLedgerAnswerStableV2, MinaLedgerSyncLedgerQueryStableV1,
    MinaStateProtocolStateValueStableV2, NonZeroCurvePoint, StateHash,
};
use mina_signer::CompressedPubKey;
use openmina_core::{
    block::ArcBlockWithHash,
    snark::{Snark, SnarkJobId},
};
use openmina_node_account::AccountPublicKey;
use p2p::channels::rpc::StagedLedgerAuxAndPendingCoinbases;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    sync::{mpsc::Sender, Arc},
};

use crate::{
    block_producer::{
        vrf_evaluator::DelegatorTable, BlockProducerWonSlot, StagedLedgerDiffCreateOutput,
    },
    rpc::RpcScanStateSummaryScanStateJob,
    transition_frontier::{
        sync::{
            ledger::staged::StagedLedgerAuxAndPendingCoinbasesValid,
            TransitionFrontierRootSnarkedLedgerUpdates,
        },
        CommitResult,
    },
};

/// This type represents Events raised by the LedgerManager in response to
/// asynchronous requests. Functions making asynchronous requests will always
/// return `Result<(), String>` immediately, while the actual result of
/// computation will be delivered via one or more of these events.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum LedgerEvent {
    LedgerReconstructSuccess(LedgerHash),
    LedgerReconstructError(String),
}

impl fmt::Display for LedgerEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LedgerEvent::LedgerReconstructSuccess(ledger_hash) => {
                write!(f, "LedgerReconstructSuccess: {}", ledger_hash)
            }
            LedgerEvent::LedgerReconstructError(msg) => {
                write!(f, "LedgerReconstructError: {}", msg)
            }
        }
    }
}

/// The type enumerating different requests that can be made to the
/// service. Each specific constructor has a specific response
/// constructor associated with it. Unfortunately, this relationship
/// can't be expressed in the Rust type system at the moment. For this
/// reason this type is private while functions wrapping the whole call
/// to the service are exposed as the service's methods.
pub enum LedgerRequest {
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
    StagedLedgerReconstructionSpawn {
        snarked_ledger_hash: LedgerHash,
        parts: Option<Arc<StagedLedgerAuxAndPendingCoinbasesValid>>,
    }, // expected response: Success
    StagedLedgerReconstructionFinalize {
        ledger_hash: LedgerHash
    }, // expected response: Success
    StakeProofSparseLedger {
        staking_ledger: LedgerHash,
        producer: NonZeroCurvePoint,
        delegator: NonZeroCurvePoint,
    }, // expected response: SparseLedgerBase
}

/// This type represents LedgerManager's responses to synchronous request.
/// Each variant corresponds to a specific request type, but unfortunately,
/// this relationship can't be expressed in the Rust type system at the moment.
/// For this reason this type is kept private and its variants are converted to
/// different types by their respective request handlers.
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

pub struct LedgerRequestWithChan {
    pub request: LedgerRequest,
    pub responder: Option<Sender<Result<LedgerResponse, String>>>,
}
