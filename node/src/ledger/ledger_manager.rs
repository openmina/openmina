use mina_p2p_messages::v2::{
    LedgerHash, MinaBaseAccountBinableArgStableV2, MinaBaseSparseLedgerBaseStableV2,
    MinaLedgerSyncLedgerAnswerStableV2, MinaLedgerSyncLedgerQueryStableV1,
    MinaStateProtocolStateValueStableV2, NonZeroCurvePoint, StateHash,
};
use openmina_core::block::ArcBlockWithHash;
use openmina_core::channels::mpsc;
use openmina_core::snark::{Snark, SnarkJobId};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::mpsc::{channel, RecvError, Sender};
use std::sync::Arc;
use std::thread;
use std::time;

use super::ledger_messages::{LedgerRequest, LedgerResponse};
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

use crate::event_source::Event;

struct LedgerRequestWithChan {
    request: LedgerRequest,
    responder: Option<Sender<Result<LedgerResponse, String>>>,
}

pub struct LedgerManager {
    sender: mpsc::UnboundedSender<LedgerRequestWithChan>,
    join_handle: thread::JoinHandle<LedgerCtx>,
}

impl LedgerManager {
    pub fn spawn<P>(
        additional_snarked_ledger_path: Option<P>,
        event_sender: Option<mpsc::UnboundedSender<Event>>,
    ) -> LedgerManager
    where
        P: AsRef<Path>,
    {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let mut ledger_ctx = LedgerCtx::new(event_sender, additional_snarked_ledger_path);
        let runtime = thread::spawn(move || {
            loop {
                openmina_core::info!(
                    openmina_core::log::system_time();
                    kind = "LedgerManager::loop",
                    summary = format!("Another loop pass.")
                );
                ledger_ctx.staged_ledger_reconstructions_finalize();
                match receiver.blocking_recv() {
                    Some(LedgerRequestWithChan { request, responder }) => {
                        let result = ledger_ctx.handle_request(request);
                        if let Some(r) = responder {
                            r.send(result).unwrap_or(())
                        }
                    }
                    None => {
                        // We still don't want to block on any
                        // particular thread here, because we want to
                        // handle finished reconstructions (and fire
                        // appropriate events) as soon as they're
                        // ready.
                        while ledger_ctx.pending_ledger_reconstructions() > 0 {
                            thread::sleep(time::Duration::from_millis(100));
                            ledger_ctx.staged_ledger_reconstructions_finalize()
                        }
                        break;
                    }
                }
            }
            ledger_ctx
        });
        LedgerManager {
            sender,
            join_handle: runtime,
        }
    }

    fn request(&self, request: LedgerRequest) -> Result<(), String> {
        self.sender
            .send(LedgerRequestWithChan {
                request,
                responder: None,
            })
            .map_err(|e| format!("LedgerManager request failed: {:?}", e))
    }

    fn call(&self, request: LedgerRequest) -> Result<LedgerResponse, String> {
        let (responder, receiver) = channel();
        self.sender
            .send(LedgerRequestWithChan {
                request,
                responder: Some(responder),
            })
            .unwrap();
        result_join(receiver.recv())
    }

    pub async fn wait_for_stop(self) -> std::thread::Result<LedgerCtx> {
        self.join_handle.join()
    }

    pub fn insert_genesis_ledger(&self, mask: Mask) {
        self.call(LedgerRequest::InsertGenesisLedger { mask })
            .unwrap();
    }

    pub fn get_mask(&self, ledger_hash: &LedgerHash) -> Option<(Mask, bool)> {
        match self.call(LedgerRequest::GetMask {
            ledger_hash: ledger_hash.clone(),
        }) {
            Ok(LedgerResponse::LedgerMask(mask)) => mask,
            _ => panic!("get_mask failed"),
        }
    }

    pub fn producers_with_delegates(
        &self,
        ledger_hash: &LedgerHash,
        filter: fn(&CompressedPubKey) -> bool,
    ) -> Option<BTreeMap<AccountPublicKey, Vec<(ledger::AccountIndex, AccountPublicKey, u64)>>>
    {
        match self.call(LedgerRequest::GetProducersWithDelegates {
            ledger_hash: ledger_hash.clone(),
            filter,
        }) {
            Ok(LedgerResponse::ProducersWithDelegatesMap(map)) => map,
            _ => panic!("producers_with_delegates failed"),
        }
    }
}

fn result_join<T>(r: Result<Result<T, String>, RecvError>) -> Result<T, String> {
    match r {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(e.to_string()),
    }
}

fn format_response_error(method: &str, res: LedgerResponse) -> String {
    format!("LedgerManager::{method}: unexpected response: {res:?}")
}

impl redux::TimeService for LedgerManager {}

impl redux::Service for LedgerManager {}

impl TransitionFrontierSyncLedgerSnarkedService for LedgerManager {
    fn compute_snarked_ledger_hashes(
        &self,
        snarked_ledger_hash: &LedgerHash,
    ) -> Result<(), String> {
        self.call(LedgerRequest::ComputeSnarkedLedgerHashes {
            snarked_ledger_hash: snarked_ledger_hash.clone(),
        })
        .and_then(|res| {
            if let LedgerResponse::Success = res {
                Ok(())
            } else {
                Err(format_response_error("compute_snarked_ledger_hashes", res))
            }
        })
    }

    fn copy_snarked_ledger_contents_for_sync(
        &self,
        origin_snarked_ledger_hash: LedgerHash,
        target_snarked_ledger_hash: LedgerHash,
        overwrite: bool,
    ) -> Result<bool, String> {
        self.call(LedgerRequest::CopySnarkedLedgerContentsForSync {
            origin_snarked_ledger_hash,
            target_snarked_ledger_hash,
            overwrite,
        })
        .and_then(|res| {
            if let LedgerResponse::SnarkedLedgerContentsCopied(copied) = res {
                Ok(copied)
            } else {
                Err(format_response_error(
                    "copy_snarked_ledger_contents_for_sync",
                    res,
                ))
            }
        })
    }

    fn child_hashes_get(
        &self,
        snarked_ledger_hash: LedgerHash,
        parent: &LedgerAddress,
    ) -> Result<(LedgerHash, LedgerHash), String> {
        self.call(LedgerRequest::ChildHashesGet {
            snarked_ledger_hash,
            parent: parent.clone(),
        })
        .and_then(|res| {
            if let LedgerResponse::ChildHashes(left, right) = res {
                Ok((left, right))
            } else {
                Err(format_response_error("child_hashes_get", res))
            }
        })
    }

    fn accounts_set(
        &self,
        snarked_ledger_hash: LedgerHash,
        parent: &LedgerAddress,
        accounts: Vec<MinaBaseAccountBinableArgStableV2>,
    ) -> Result<LedgerHash, String> {
        self.call(LedgerRequest::AccountsSet {
            snarked_ledger_hash,
            parent: parent.clone(),
            accounts,
        })
        .and_then(|res| {
            if let LedgerResponse::LedgerHash(hash) = res {
                Ok(hash)
            } else {
                Err(format_response_error("accounts_set", res))
            }
        })
    }
}

impl TransitionFrontierSyncLedgerStagedService for LedgerManager {
    fn staged_ledger_reconstruct(
        &self,
        snarked_ledger_hash: LedgerHash,
        parts: Option<Arc<StagedLedgerAuxAndPendingCoinbasesValid>>,
    ) {
        self.request(LedgerRequest::StagedLedgerReconstruct {
            snarked_ledger_hash,
            parts,
        })
        .expect("LedgerManager::staged_ledger_reconstruct: sending request failed")
    }
}

impl TransitionFrontierService for LedgerManager {
    fn block_apply(
        &self,
        block: ArcBlockWithHash,
        pred_block: ArcBlockWithHash,
    ) -> Result<(), String> {
        self.call(LedgerRequest::BlockApply { block, pred_block })
            .and_then(|res| {
                if let LedgerResponse::Success = res {
                    Ok(())
                } else {
                    Err(format_response_error("block_apply", res))
                }
            })
    }

    fn commit(
        &self,
        ledgers_to_keep: BTreeSet<LedgerHash>,
        root_snarked_ledger_updates: TransitionFrontierRootSnarkedLedgerUpdates,
        needed_protocol_states: BTreeMap<StateHash, MinaStateProtocolStateValueStableV2>,
        new_root: &ArcBlockWithHash,
        new_best_tip: &ArcBlockWithHash,
    ) -> CommitResult {
        self.call(LedgerRequest::Commit {
            ledgers_to_keep,
            root_snarked_ledger_updates,
            needed_protocol_states,
            new_root: new_root.clone(),
            new_best_tip: new_best_tip.clone(),
        })
        .and_then(|res| {
            if let LedgerResponse::CommitResult(result) = res {
                Ok(result)
            } else {
                Err(format_response_error("commit", res))
            }
        })
        .expect("LedgerManager::commit: unexpected error")
    }

    fn answer_ledger_query(
        &self,
        ledger_hash: LedgerHash,
        query: MinaLedgerSyncLedgerQueryStableV1,
    ) -> Option<MinaLedgerSyncLedgerAnswerStableV2> {
        self.call(LedgerRequest::LedgerQuery { ledger_hash, query })
            .and_then(|res| {
                if let LedgerResponse::LedgerQueryResult(answer) = res {
                    Ok(answer)
                } else {
                    Err(format_response_error("ledger_query", res))
                }
            })
            .expect("LedgerManager::ledger_query: unexpected error")
    }

    fn staged_ledger_aux_and_pending_coinbase(
        &self,
        ledger_hash: LedgerHash,
        protocol_states: BTreeMap<StateHash, MinaStateProtocolStateValueStableV2>,
    ) -> Option<Arc<StagedLedgerAuxAndPendingCoinbases>> {
        self.call(LedgerRequest::StagedLedgerAuxAndPendingCoinbase {
            ledger_hash,
            protocol_states,
        })
        .and_then(|res| {
            if let LedgerResponse::LedgerAuxAndCoinbaseResult(result) = res {
                Ok(result)
            } else {
                Err(format_response_error(
                    "staged_ledger_aux_and_pending_coinbase",
                    res,
                ))
            }
        })
        .expect("LedgerManager::staged_ledger_aux_and_pending_coinbase: unexpected error")
    }
}

impl BlockProducerLedgerService for LedgerManager {
    fn staged_ledger_diff_create(
        &self,
        pred_block: &ArcBlockWithHash,
        won_slot: &BlockProducerWonSlot,
        coinbase_receiver: &NonZeroCurvePoint,
        completed_snarks: BTreeMap<SnarkJobId, Snark>,
        supercharge_coinbase: bool,
    ) -> Result<StagedLedgerDiffCreateOutput, String> {
        self.call(LedgerRequest::StagedLedgerDiffCreate {
            pred_block: pred_block.clone(),
            won_slot: won_slot.clone(),
            coinbase_receiver: coinbase_receiver.clone(),
            completed_snarks,
            supercharge_coinbase,
        })
        .and_then(|res| {
            if let LedgerResponse::StagedLedgerDiff(result) = res {
                Ok(result)
            } else {
                Err(format_response_error("staged_ledger_diff_create", res))
            }
        })
    }

    fn stake_proof_sparse_ledger(
        &self,
        staking_ledger: LedgerHash,
        producer: NonZeroCurvePoint,
        delegator: NonZeroCurvePoint,
    ) -> Option<MinaBaseSparseLedgerBaseStableV2> {
        self.call(LedgerRequest::StakeProofSparseLedger {
            staking_ledger,
            producer,
            delegator,
        })
        .and_then(|res| {
            if let LedgerResponse::SparseLedgerBase(result) = res {
                Ok(result)
            } else {
                Err(format_response_error("stake_proof_sparse_ledger", res))
            }
        })
        .expect("LedgerManager::stake_proof_sparse_ledger: unexpected error")
    }
}

impl RpcLedgerService for LedgerManager {
    fn scan_state_summary(
        &self,
        ledger_hash: LedgerHash,
    ) -> Vec<Vec<RpcScanStateSummaryScanStateJob>> {
        self.call(LedgerRequest::GetScanStateSummary { ledger_hash })
            .and_then(|res| {
                if let LedgerResponse::ScanStateSummary(summary) = res {
                    Ok(summary)
                } else {
                    Err(format_response_error("scan_state_summary", res))
                }
            })
            .expect("LedgerManager::scan_state_summary: unexpected error")
    }
}

impl BlockProducerVrfEvaluatorLedgerService for LedgerManager {
    fn get_producer_and_delegates(
        &self,
        ledger_hash: LedgerHash,
        producer: AccountPublicKey,
    ) -> DelegatorTable {
        self.call(LedgerRequest::GetProducerAndDelegates {
            ledger_hash,
            producer,
        })
        .and_then(|res| {
            if let LedgerResponse::ProducerAndDelegates(table) = res {
                Ok(table)
            } else {
                Err(format_response_error("get_producer_and_delegates", res))
            }
        })
        .expect("LedgerManager::get_producer_and_delegates: unexpected error")
    }
}
