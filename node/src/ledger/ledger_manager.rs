use std::collections::BTreeMap;

use ledger::staged_ledger::staged_ledger::StagedLedger;
use mina_p2p_messages::v2::{self, LedgerHash, MinaBaseAccountBinableArgStableV2};
use openmina_core::channels::mpsc;
use openmina_core::thread;

use super::ledger_service::LedgerCtx;
use super::read::{LedgerReadId, LedgerReadRequest, LedgerReadResponse};
use super::write::{LedgerWriteRequest, LedgerWriteResponse};
use super::LedgerService;
use crate::account::AccountPublicKey;
use crate::ledger::LedgerAddress;
use crate::transition_frontier::sync::ledger::snarked::TransitionFrontierSyncLedgerSnarkedService;
use ledger::{Account, AccountId, Mask};
use mina_signer::CompressedPubKey;

/// The type enumerating different requests that can be made to the
/// service. Each specific constructor has a specific response
/// constructor associated with it. Unfortunately, this relationship
/// can't be expressed in the Rust type system at the moment. For this
/// reason this type is private while functions wrapping the whole call
/// to the service are exposed as the service's methods.
#[allow(dead_code)] // TODO
pub(super) enum LedgerRequest {
    Write(LedgerWriteRequest),
    Read(LedgerReadId, LedgerReadRequest),
    AccountsSet {
        snarked_ledger_hash: LedgerHash,
        parent: LedgerAddress,
        accounts: Vec<MinaBaseAccountBinableArgStableV2>,
    }, // expected response: LedgerHash
    AccountsGet {
        ledger_hash: LedgerHash,
        account_ids: Vec<AccountId>,
    }, // expected response: Vec<Account>
    ChildHashesGet {
        snarked_ledger_hash: LedgerHash,
        parent: LedgerAddress,
    }, // expected response: ChildHashes
    ComputeSnarkedLedgerHashes {
        snarked_ledger_hash: LedgerHash,
    }, // expected response: Success
    CopySnarkedLedgerContentsForSync {
        origin_snarked_ledger_hash: Vec<LedgerHash>,
        target_snarked_ledger_hash: LedgerHash,
        overwrite: bool,
    }, // expected response: SnarkedLedgerContentsCopied
    GetProducersWithDelegates {
        ledger_hash: LedgerHash,
        filter: fn(&CompressedPubKey) -> bool,
    }, // expected response: ProducersWithDelegatesMap
    GetMask {
        ledger_hash: LedgerHash,
    }, // expected response: LedgerMask
    InsertGenesisLedger {
        mask: Mask,
    },
    StagedLedgerReconstructResult {
        staged_ledger_hash: LedgerHash,
        result: Result<StagedLedger, String>,
    },
}

#[derive(Debug)]
pub enum LedgerResponse {
    Write(LedgerWriteResponse),
    Read(LedgerReadId, LedgerReadResponse),
    ChildHashes(Option<(LedgerHash, LedgerHash)>),
    AccountsSet(Result<LedgerHash, String>),
    AccountsGet(Result<Vec<Account>, String>),
    LedgerMask(Option<(Mask, bool)>),
    #[allow(clippy::type_complexity)]
    ProducersWithDelegatesMap(
        Option<BTreeMap<AccountPublicKey, Vec<(ledger::AccountIndex, AccountPublicKey, u64)>>>,
    ),
    SnarkedLedgerContentsCopied(Result<bool, String>),
    Success, // operation was performed and result stored; nothing to return.
}

impl LedgerRequest {
    fn handle(
        self,
        ledger_ctx: &mut LedgerCtx,
        caller: &LedgerCaller,
        force_sync: bool,
    ) -> LedgerResponse {
        match self {
            Self::Write(request) => LedgerResponse::Write(match request {
                LedgerWriteRequest::StagedLedgerReconstruct {
                    snarked_ledger_hash,
                    parts,
                } => {
                    if !force_sync {
                        let caller = caller.clone();
                        let cb = move |staged_ledger_hash, result| {
                            caller.call(LedgerRequest::StagedLedgerReconstructResult {
                                staged_ledger_hash,
                                result,
                            })
                        };
                        if let Err(e) =
                            ledger_ctx.staged_ledger_reconstruct(snarked_ledger_hash, parts, cb)
                        {
                            openmina_core::log::inner::error!(
                                "Failed to reconstruct staged ledger: {:?}",
                                e
                            );
                            // TODO: Handle the error in the state machine
                        }
                        return LedgerResponse::Success;
                    } else {
                        let (staged_ledger_hash, result) = match ledger_ctx
                            .staged_ledger_reconstruct_sync(snarked_ledger_hash, parts)
                        {
                            Ok(result) => result,
                            Err(e) => (v2::LedgerHash::zero(), Err(String::from(e))),
                        };

                        LedgerWriteResponse::StagedLedgerReconstruct {
                            staged_ledger_hash,
                            result,
                        }
                    }
                }
                LedgerWriteRequest::StagedLedgerDiffCreate {
                    pred_block,
                    global_slot_since_genesis: global_slot,
                    is_new_epoch,
                    producer,
                    delegator,
                    coinbase_receiver,
                    completed_snarks,
                    supercharge_coinbase,
                    transactions_by_fee,
                } => {
                    let pred_block_hash = pred_block.hash().clone();
                    let global_slot_since_genesis = global_slot.clone();
                    let result = ledger_ctx.staged_ledger_diff_create(
                        pred_block,
                        global_slot,
                        is_new_epoch,
                        producer,
                        delegator,
                        coinbase_receiver,
                        completed_snarks,
                        supercharge_coinbase,
                        transactions_by_fee,
                    );
                    LedgerWriteResponse::StagedLedgerDiffCreate {
                        pred_block_hash,
                        global_slot_since_genesis,
                        result: result.map(Into::into),
                    }
                }
                LedgerWriteRequest::BlockApply { block, pred_block } => {
                    let block_hash = block.hash().clone();
                    let result = ledger_ctx.block_apply(block, pred_block);
                    LedgerWriteResponse::BlockApply { block_hash, result }
                }
                LedgerWriteRequest::Commit {
                    ledgers_to_keep,
                    root_snarked_ledger_updates,
                    needed_protocol_states,
                    new_root,
                    new_best_tip,
                } => {
                    let best_tip_hash = new_best_tip.hash().clone();
                    let result = ledger_ctx.commit(
                        ledgers_to_keep,
                        root_snarked_ledger_updates,
                        needed_protocol_states,
                        &new_root,
                        &new_best_tip,
                    );
                    LedgerWriteResponse::Commit {
                        best_tip_hash,
                        result,
                    }
                }
            }),
            Self::Read(id, request) => LedgerResponse::Read(
                id,
                match request {
                    LedgerReadRequest::DelegatorTable(ledger_hash, producer) => {
                        let res = ledger_ctx
                            .producers_with_delegates(&ledger_hash, |pub_key| {
                                AccountPublicKey::from(pub_key.clone()) == producer
                            })
                            .and_then(|list| list.into_iter().next())
                            .map(|(_, table)| {
                                table
                                    .into_iter()
                                    .map(|(index, pub_key, balance)| (index, (pub_key, balance)))
                                    .collect()
                            });

                        LedgerReadResponse::DelegatorTable(res)
                    }
                    LedgerReadRequest::GetNumAccounts(ledger_hash) => {
                        let res = ledger_ctx.get_num_accounts(ledger_hash);
                        LedgerReadResponse::GetNumAccounts(res)
                    }
                    LedgerReadRequest::GetChildHashesAtAddr(ledger_hash, addr) => {
                        let res = ledger_ctx.get_child_hashes(ledger_hash, addr);
                        LedgerReadResponse::GetChildHashesAtAddr(res)
                    }
                    LedgerReadRequest::GetChildAccountsAtAddr(ledger_hash, addr) => {
                        let res = ledger_ctx.get_child_accounts(ledger_hash, addr);
                        LedgerReadResponse::GetChildAccountsAtAddr(res)
                    }
                    LedgerReadRequest::GetStagedLedgerAuxAndPendingCoinbases(data) => {
                        let res = ledger_ctx.staged_ledger_aux_and_pending_coinbase(
                            &data.ledger_hash,
                            data.protocol_states,
                        );
                        LedgerReadResponse::GetStagedLedgerAuxAndPendingCoinbases(res)
                    }
                    LedgerReadRequest::ScanStateSummary(ledger_hash) => {
                        let res = ledger_ctx.scan_state_summary(&ledger_hash);
                        LedgerReadResponse::ScanStateSummary(res)
                    }
                    LedgerReadRequest::GetAccounts(ledger_hash, account_ids) => {
                        let res = ledger_ctx.get_accounts(ledger_hash, account_ids);
                        LedgerReadResponse::GetAccounts(res)
                    }
                    LedgerReadRequest::AccountsForRpc(rpc_id, ledger_hash, public_key) => {
                        let res = ledger_ctx.get_accounts_for_rpc(ledger_hash, public_key);
                        LedgerReadResponse::AccountsForRpc(rpc_id, res)
                    }
                },
            ),
            LedgerRequest::AccountsSet {
                snarked_ledger_hash,
                parent,
                accounts,
            } => LedgerResponse::AccountsSet(ledger_ctx.accounts_set(
                snarked_ledger_hash,
                &parent,
                accounts,
            )),
            LedgerRequest::ChildHashesGet {
                snarked_ledger_hash,
                parent,
            } => {
                let res = ledger_ctx.get_child_hashes(snarked_ledger_hash, parent);
                LedgerResponse::ChildHashes(res)
            }
            LedgerRequest::ComputeSnarkedLedgerHashes {
                snarked_ledger_hash,
            } => {
                ledger_ctx
                    .compute_snarked_ledger_hashes(&snarked_ledger_hash)
                    .unwrap();
                LedgerResponse::Success
            }
            LedgerRequest::CopySnarkedLedgerContentsForSync {
                origin_snarked_ledger_hash,
                target_snarked_ledger_hash,
                overwrite,
            } => {
                let origin_snarked_ledger_hash = origin_snarked_ledger_hash
                    .iter()
                    .find(|hash| ledger_ctx.contains_snarked_ledger(hash))
                    .unwrap_or_else(|| {
                        origin_snarked_ledger_hash
                            .first()
                            .expect("origin_snarked_ledger_hash cannot be empty")
                    })
                    .clone();

                let res = ledger_ctx.copy_snarked_ledger_contents_for_sync(
                    origin_snarked_ledger_hash,
                    target_snarked_ledger_hash,
                    overwrite,
                );
                LedgerResponse::SnarkedLedgerContentsCopied(res)
            }
            LedgerRequest::GetMask { ledger_hash } => {
                LedgerResponse::LedgerMask(ledger_ctx.mask(&ledger_hash))
            }
            LedgerRequest::GetProducersWithDelegates {
                ledger_hash,
                filter,
            } => {
                let res = ledger_ctx.producers_with_delegates(&ledger_hash, filter);
                LedgerResponse::ProducersWithDelegatesMap(res)
            }
            LedgerRequest::InsertGenesisLedger { mask } => {
                ledger_ctx.insert_genesis_ledger(mask);
                LedgerResponse::Success
            }
            LedgerRequest::StagedLedgerReconstructResult {
                staged_ledger_hash,
                result,
            } => {
                let result = match result {
                    Err(err) => Err(err),
                    Ok(ledger) => {
                        ledger_ctx.staged_ledger_reconstruct_result_store(ledger);
                        Ok(())
                    }
                };
                LedgerResponse::Write(LedgerWriteResponse::StagedLedgerReconstruct {
                    staged_ledger_hash,
                    result,
                })
            }
            LedgerRequest::AccountsGet {
                ledger_hash,
                account_ids,
            } => {
                let res = ledger_ctx.get_accounts(ledger_hash, account_ids);
                LedgerResponse::AccountsGet(Ok(res))
            }
        }
    }
}

struct LedgerRequestWithChan {
    request: LedgerRequest,
    responder: Option<std::sync::mpsc::SyncSender<LedgerResponse>>,
}

pub struct LedgerManager {
    caller: LedgerCaller,
    join_handle: thread::JoinHandle<LedgerCtx>,
}

#[derive(Clone)]
pub(super) struct LedgerCaller(mpsc::UnboundedSender<LedgerRequestWithChan>);

impl LedgerManager {
    pub fn spawn(mut ledger_ctx: LedgerCtx) -> LedgerManager {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let caller = LedgerCaller(sender);
        let ledger_caller = caller.clone();

        let ledger_manager_loop = move || {
            while let Some(LedgerRequestWithChan { request, responder }) = receiver.blocking_recv()
            {
                let response = request.handle(&mut ledger_ctx, &ledger_caller, responder.is_some());
                match (response, responder) {
                    (LedgerResponse::Write(resp), None) => {
                        ledger_ctx.send_write_response(resp);
                    }
                    (LedgerResponse::Write(resp), Some(responder)) => {
                        ledger_ctx.send_write_response(resp.clone());
                        let _ = responder.send(LedgerResponse::Write(resp));
                    }
                    (LedgerResponse::Read(id, resp), None) => {
                        ledger_ctx.send_read_response(id, resp);
                    }
                    (LedgerResponse::Read(id, resp), Some(responder)) => {
                        ledger_ctx.send_read_response(id, resp.clone());
                        let _ = responder.send(LedgerResponse::Read(id, resp));
                    }
                    (resp, Some(responder)) => {
                        let _ = responder.send(resp);
                    }
                    (_, None) => {}
                }
            }
            ledger_ctx
        };
        let join_handle = thread::Builder::new()
            .name("ledger-manager".into())
            .spawn(ledger_manager_loop)
            .expect("Failed: ledger manager");
        LedgerManager {
            caller,
            join_handle,
        }
    }

    pub(super) fn call(&self, request: LedgerRequest) {
        self.caller.call(request)
    }

    pub(super) fn call_sync(
        &self,
        request: LedgerRequest,
    ) -> Result<LedgerResponse, std::sync::mpsc::RecvError> {
        self.caller.call_sync(request)
    }

    pub async fn wait_for_stop(self) -> thread::Result<LedgerCtx> {
        self.join_handle.join()
    }

    pub fn insert_genesis_ledger(&self, mask: Mask) {
        self.call(LedgerRequest::InsertGenesisLedger { mask });
    }

    pub fn get_mask(&self, ledger_hash: &LedgerHash) -> Option<(Mask, bool)> {
        match self.call_sync(LedgerRequest::GetMask {
            ledger_hash: ledger_hash.clone(),
        }) {
            Ok(LedgerResponse::LedgerMask(mask)) => mask,
            _ => panic!("get_mask failed"),
        }
    }

    pub fn get_accounts(
        &self,
        ledger_hash: &LedgerHash,
        account_ids: Vec<AccountId>,
    ) -> Result<Vec<Account>, String> {
        // TODO: this should be asynchronous
        match self.call_sync(LedgerRequest::AccountsGet {
            ledger_hash: ledger_hash.clone(),
            account_ids,
        }) {
            Ok(LedgerResponse::AccountsGet(result)) => result,
            _ => panic!("get_accounts failed"),
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn producers_with_delegates(
        &self,
        ledger_hash: &LedgerHash,
        filter: fn(&CompressedPubKey) -> bool,
    ) -> Option<BTreeMap<AccountPublicKey, Vec<(ledger::AccountIndex, AccountPublicKey, u64)>>>
    {
        match self.call_sync(LedgerRequest::GetProducersWithDelegates {
            ledger_hash: ledger_hash.clone(),
            filter,
        }) {
            Ok(LedgerResponse::ProducersWithDelegatesMap(map)) => map,
            _ => panic!("producers_with_delegates failed"),
        }
    }
}

impl LedgerCaller {
    pub fn call(&self, request: LedgerRequest) {
        self.0
            .send(LedgerRequestWithChan {
                request,
                responder: None,
            })
            .unwrap();
    }

    fn call_sync(
        &self,
        request: LedgerRequest,
    ) -> Result<LedgerResponse, std::sync::mpsc::RecvError> {
        let (responder, receiver) = std::sync::mpsc::sync_channel(0);
        self.0
            .send(LedgerRequestWithChan {
                request,
                responder: Some(responder),
            })
            .unwrap();
        receiver.recv()
    }
}

fn format_response_error(method: &str, res: LedgerResponse) -> String {
    format!("LedgerManager::{method}: unexpected response: {res:?}")
}

impl<T: LedgerService> TransitionFrontierSyncLedgerSnarkedService for T {
    fn compute_snarked_ledger_hashes(
        &self,
        snarked_ledger_hash: &LedgerHash,
    ) -> Result<(), String> {
        self.ledger_manager()
            .call_sync(LedgerRequest::ComputeSnarkedLedgerHashes {
                snarked_ledger_hash: snarked_ledger_hash.clone(),
            })
            .map_err(|_| "compute_snarked_ledger_hashes responder dropped".to_owned())
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
        origin_snarked_ledger_hash: Vec<LedgerHash>,
        target_snarked_ledger_hash: LedgerHash,
        overwrite: bool,
    ) -> Result<bool, String> {
        self.ledger_manager()
            .call_sync(LedgerRequest::CopySnarkedLedgerContentsForSync {
                origin_snarked_ledger_hash,
                target_snarked_ledger_hash,
                overwrite,
            })
            .map_err(|_| "copy_snarked_ledger_contents_for_sync responder dropped".to_owned())
            .and_then(|res| {
                if let LedgerResponse::SnarkedLedgerContentsCopied(copied) = res {
                    copied
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
        self.ledger_manager()
            .call_sync(LedgerRequest::ChildHashesGet {
                snarked_ledger_hash,
                parent: parent.clone(),
            })
            .map_err(|_| "child_hashes_get responder dropped".to_owned())
            .and_then(|res| {
                if let LedgerResponse::ChildHashes(Some(res)) = res {
                    Ok(res)
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
        self.ledger_manager()
            .call_sync(LedgerRequest::AccountsSet {
                snarked_ledger_hash,
                parent: parent.clone(),
                accounts,
            })
            .map_err(|_| "accounts_set responder dropped".to_owned())
            .and_then(|res| {
                if let LedgerResponse::AccountsSet(res) = res {
                    res
                } else {
                    Err(format_response_error("accounts_set", res))
                }
            })
    }
}
