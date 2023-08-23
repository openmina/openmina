use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use ledger::{
    scan_state::{
        currency::{Amount, Fee, Slot},
        scan_state::{
            AvailableJobMessage, ConstraintConstants, JobValueBase, JobValueMerge,
            JobValueWithIndex,
        },
        transaction_logic::{local_state::LocalState, protocol_state::protocol_state_view},
    },
    staged_ledger::{
        diff::Diff,
        staged_ledger::{SkipVerification, StagedLedger},
    },
    verifier::Verifier,
    AccountIndex, BaseLedger, Mask, TreeVersion,
};
use mina_hasher::Fp;
use mina_p2p_messages::v2::{
    DataHashLibStateHashStableV1, LedgerHash, MinaBaseAccountBinableArgStableV2,
    MinaBaseLedgerHash0StableV1, MinaBaseSokMessageStableV1, MinaBaseStagedLedgerHashStableV1,
    MinaLedgerSyncLedgerAnswerStableV2, MinaLedgerSyncLedgerQueryStableV1,
    MinaStateBlockchainStateValueStableV2LedgerProofStatement, MinaStateProtocolStateValueStableV2,
    MinaTransactionTransactionStableV2, StateHash,
};
use mina_signer::CompressedPubKey;
use shared::{block::ArcBlockWithHash, snark::SnarkJobId};

use crate::transition_frontier::sync::ledger::staged::StagedLedgerAuxAndPendingCoinbasesValid;
use crate::transition_frontier::sync::ledger::staged::TransitionFrontierSyncLedgerStagedService;
use crate::transition_frontier::TransitionFrontierService;
use crate::{
    p2p::channels::rpc::StagedLedgerAuxAndPendingCoinbases, transition_frontier::CommitResult,
};
use crate::{
    rpc::{
        RpcLedgerService, RpcScanStateSummaryBlockTransaction, RpcScanStateSummaryScanStateJob,
        RpcScanStateSummaryScanStateJobKind, RpcSnarkPoolJobSnarkWorkDone,
    },
    transition_frontier::sync::ledger::snarked::TransitionFrontierSyncLedgerSnarkedService,
};

use super::{ledger_empty_hash_at_depth, LedgerAddress, LEDGER_DEPTH};

const CONSTRAINT_CONSTANTS: ConstraintConstants = ConstraintConstants {
    sub_windows_per_window: 11,
    ledger_depth: 35,
    work_delay: 2,
    block_window_duration_ms: 180000,
    transaction_capacity_log_2: 7,
    pending_coinbase_depth: 5,
    coinbase_amount: Amount::from_u64(720000000000),
    supercharged_coinbase_factor: 2,
    account_creation_fee: Fee::from_u64(1000000000),
    fork: None,
};

fn ledger_hash(depth: usize, left: Fp, right: Fp) -> Fp {
    let height = LEDGER_DEPTH - depth - 1;
    ledger::V2::hash_node(height, left, right)
}

#[derive(Default)]
pub struct LedgerCtx {
    snarked_ledgers: BTreeMap<LedgerHash, Mask>,
    staged_ledgers: BTreeMap<LedgerHash, StagedLedger>,
    sync: LedgerSyncState,
}

#[derive(Default)]
struct LedgerSyncState {
    snarked_ledgers: BTreeMap<LedgerHash, Mask>,
    staged_ledgers: BTreeMap<LedgerHash, StagedLedger>,
}

impl LedgerCtx {
    fn mask(&self, hash: &LedgerHash) -> Option<(Mask, bool)> {
        self.snarked_ledgers
            .get(hash)
            .cloned()
            .map(|mask| (mask, true))
            .or_else(|| Some((self.staged_ledgers.get(hash)?.ledger(), true)))
            .or_else(|| self.sync.mask(hash))
    }

    fn staged_ledger_mut(&mut self, hash: &LedgerHash) -> Option<&mut StagedLedger> {
        match self.staged_ledgers.get_mut(&hash) {
            Some(v) => Some(v),
            None => self.sync.staged_ledger_mut(&hash),
        }
    }
}

impl LedgerSyncState {
    fn mask(&self, hash: &LedgerHash) -> Option<(Mask, bool)> {
        self.snarked_ledgers
            .get(hash)
            .cloned()
            .map(|mask| (mask, false))
            .or_else(|| Some((self.staged_ledgers.get(hash)?.ledger(), true)))
    }

    fn snarked_ledger_mut(&mut self, hash: LedgerHash) -> &mut Mask {
        self.snarked_ledgers.entry(hash.clone()).or_insert_with(|| {
            let mut ledger = Mask::create(LEDGER_DEPTH);
            ledger.set_cached_hash_unchecked(&LedgerAddress::root(), hash.0.to_field());
            ledger
        })
    }

    fn staged_ledger_mut(&mut self, hash: &LedgerHash) -> Option<&mut StagedLedger> {
        self.staged_ledgers.get_mut(&hash)
    }
}

impl LedgerCtx {
    pub fn new() -> Self {
        Default::default()
    }
}

pub trait LedgerService: redux::Service {
    fn ctx(&self) -> &LedgerCtx;
    fn ctx_mut(&mut self) -> &mut LedgerCtx;
}

impl<T: LedgerService> TransitionFrontierSyncLedgerSnarkedService for T {
    fn hashes_set(
        &mut self,
        snarked_ledger_hash: LedgerHash,
        parent: &LedgerAddress,
        (left, right): (LedgerHash, LedgerHash),
    ) -> Result<(), ()> {
        let (left, right) = (left.0.to_field(), right.0.to_field());
        let hash = ledger_hash(parent.length(), left, right);

        let mask = self.ctx_mut().sync.snarked_ledger_mut(snarked_ledger_hash);

        if hash != mask.get_inner_hash_at_addr(parent.clone())? {
            return Err(());
        }

        mask.set_cached_hash_unchecked(&parent.child_left(), left);
        mask.set_cached_hash_unchecked(&parent.child_right(), right);

        Ok(())
    }

    fn accounts_set(
        &mut self,
        snarked_ledger_hash: LedgerHash,
        parent: &LedgerAddress,
        accounts: Vec<MinaBaseAccountBinableArgStableV2>,
    ) -> Result<(), ()> {
        // TODO(binier): validate hashes
        let mut addr = parent.clone();
        let first_addr = loop {
            if addr.length() == LEDGER_DEPTH {
                break addr;
            }
            addr = addr.child_left();
        };
        let mask = self.ctx_mut().sync.snarked_ledger_mut(snarked_ledger_hash);

        let first_index = first_addr.to_index();
        accounts
            .into_iter()
            .enumerate()
            .try_for_each(|(index, account)| {
                let index = AccountIndex(first_index.0 + index as u64);
                mask.set_at_index(index, (&account).into())
            })?;

        Ok(())
    }
}

impl<T: LedgerService> TransitionFrontierSyncLedgerStagedService for T {
    fn staged_ledger_reconstruct(
        &mut self,
        snarked_ledger_hash: LedgerHash,
        parts: Arc<StagedLedgerAuxAndPendingCoinbasesValid>,
    ) -> Result<(), String> {
        let snarked_ledger = self.ctx_mut().sync.snarked_ledger_mut(snarked_ledger_hash);
        // TODO(binier): TMP. Remove for prod version.
        snarked_ledger
            .validate_inner_hashes()
            .map_err(|_| "downloaded hash and recalculated mismatch".to_owned())?;

        let states = parts
            .needed_blocks
            .iter()
            .map(|state| (state.hash().to_fp().unwrap(), state.clone()))
            .collect::<BTreeMap<_, _>>();

        let mask = snarked_ledger.copy();
        let staged_ledger = StagedLedger::of_scan_state_pending_coinbases_and_snarked_ledger(
            (),
            &CONSTRAINT_CONSTANTS,
            Verifier,
            (&parts.scan_state).into(),
            mask,
            LocalState::empty(),
            parts.staged_ledger_hash.0.to_field(),
            (&parts.pending_coinbase).into(),
            |key| states.get(&key).cloned().unwrap(),
        )?;

        self.ctx_mut()
            .sync
            .staged_ledgers
            .insert(parts.staged_ledger_hash.clone(), staged_ledger);

        Ok(())
    }
}

impl<T: LedgerService> TransitionFrontierService for T {
    fn block_apply(
        &mut self,
        block: ArcBlockWithHash,
        pred_block: ArcBlockWithHash,
    ) -> Result<(), String> {
        let mut staged_ledger = self
            .ctx_mut()
            .staged_ledger_mut(&pred_block.staged_ledger_hash())
            .ok_or_else(|| "parent staged ledger missing")?
            .clone();

        let global_slot = block.global_slot();
        let prev_protocol_state = &pred_block.header().protocol_state;
        let prev_state_view = protocol_state_view(prev_protocol_state);

        let consensus_state = &block.header().protocol_state.body.consensus_state;
        let coinbase_receiver: CompressedPubKey = (&consensus_state.coinbase_receiver).into();
        let _supercharge_coinbase = consensus_state.supercharge_coinbase;

        // FIXME: Using `supercharge_coinbase` (from block) above does not work
        let supercharge_coinbase = false;

        let diff: Diff = (&block.block.body.staged_ledger_diff).into();

        let result = staged_ledger
            .apply(
                // TODO(binier): SEC
                Some(SkipVerification::All),
                &CONSTRAINT_CONSTANTS,
                Slot::from_u32(global_slot),
                diff,
                (),
                &Verifier,
                &prev_state_view,
                ledger::scan_state::protocol_state::hashes(prev_protocol_state),
                coinbase_receiver,
                supercharge_coinbase,
            )
            .map_err(|err| format!("{err:?}"))?;
        let ledger_hashes = MinaBaseStagedLedgerHashStableV1::from(&result.hash_after_applying);

        // TODO(binier): return error if not matching.
        let expected_ledger_hashes = block.staged_ledger_hashes();
        if &ledger_hashes != expected_ledger_hashes {
            panic!("staged ledger hash mismatch. found: {ledger_hashes:?}, expected: {expected_ledger_hashes:?}");
        }

        let ledger_hash = block.staged_ledger_hash();
        self.ctx_mut()
            .sync
            .staged_ledgers
            .insert(ledger_hash.clone(), staged_ledger);

        Ok(())
    }

    fn commit(
        &mut self,
        ledgers_to_keep: BTreeSet<LedgerHash>,
        new_root: &ArcBlockWithHash,
        new_best_tip: &ArcBlockWithHash,
    ) -> CommitResult {
        let ctx = self.ctx_mut();

        ctx.snarked_ledgers
            .retain(|hash, _| ledgers_to_keep.contains(hash));
        ctx.snarked_ledgers.extend(
            std::mem::take(&mut ctx.sync.snarked_ledgers)
                .into_iter()
                .filter(|(hash, _)| ledgers_to_keep.contains(hash)),
        );

        ctx.staged_ledgers
            .retain(|hash, _| ledgers_to_keep.contains(hash));
        ctx.staged_ledgers.extend(
            std::mem::take(&mut ctx.sync.staged_ledgers)
                .into_iter()
                .filter(|(hash, _)| ledgers_to_keep.contains(hash)),
        );

        let Some(new_root_ledger) = ctx.staged_ledgers.get_mut(new_root.staged_ledger_hash()) else { return Default::default() };
        // Make ledger mask new root.
        new_root_ledger.commit_and_reparent_to_root();

        ctx.staged_ledger_mut(new_best_tip.staged_ledger_hash())
            .map(|l| {
                let available_jobs = l
                    .scan_state()
                    .all_job_pairs_iter()
                    .map(|job| job.map(|single| AvailableJobMessage::from(single)))
                    .collect();
                CommitResult {
                    available_jobs,
                    needed_protocol_states: l
                        .scan_state()
                        .required_state_hashes()
                        .into_iter()
                        .map(|fp| DataHashLibStateHashStableV1(fp.into()).into())
                        .collect(),
                }
            })
            .unwrap_or_default()
    }

    fn answer_ledger_query(
        &mut self,
        ledger_hash: LedgerHash,
        query: MinaLedgerSyncLedgerQueryStableV1,
    ) -> Option<MinaLedgerSyncLedgerAnswerStableV2> {
        let ctx = self.ctx_mut();
        let (mask, is_synced) = ctx.mask(&ledger_hash).filter(|(_, is_synced)| *is_synced)?;

        Some(match query {
            MinaLedgerSyncLedgerQueryStableV1::WhatChildHashes(addr) => {
                let addr = LedgerAddress::from(addr);
                let get_hash = |addr: LedgerAddress| {
                    let depth = addr.length();
                    mask.get_hash(addr)
                        .map(|fp| MinaBaseLedgerHash0StableV1(fp.into()).into())
                        .or_else(|| {
                            if is_synced {
                                Some(ledger_empty_hash_at_depth(depth))
                            } else {
                                None
                            }
                        })
                };
                let (left, right) = (addr.child_left(), addr.child_right());
                let (left, right) = (get_hash(left)?, get_hash(right)?);
                MinaLedgerSyncLedgerAnswerStableV2::ChildHashesAre(left, right)
            }
            MinaLedgerSyncLedgerQueryStableV1::WhatContents(addr) => {
                let addr = LedgerAddress::from(addr);
                // TODO(binier): SEC maybe we need to check addr depth?
                let accounts = mask
                    .get_all_accounts_rooted_at(addr)?
                    .into_iter()
                    .map(|(_, account)| (&account).into())
                    .collect();
                MinaLedgerSyncLedgerAnswerStableV2::ContentsAre(accounts)
            }
            MinaLedgerSyncLedgerQueryStableV1::NumAccounts => {
                let num = (mask.num_accounts() as u64).into();
                MinaLedgerSyncLedgerAnswerStableV2::NumAccounts(num, ledger_hash)
            }
        })
    }

    fn staged_ledger_aux_and_pending_coinbase(
        &mut self,
        ledger_hash: LedgerHash,
        protocol_states: BTreeMap<StateHash, MinaStateProtocolStateValueStableV2>,
    ) -> Option<Arc<StagedLedgerAuxAndPendingCoinbases>> {
        let ctx = self.ctx_mut();
        let ledger = ctx.staged_ledger_mut(&ledger_hash)?;
        let needed_blocks = ledger
            .scan_state()
            .required_state_hashes()
            .into_iter()
            .map(|fp| DataHashLibStateHashStableV1(fp.into()))
            .map(|hash| protocol_states.get(&hash.into()).ok_or(()).cloned())
            .collect::<Result<_, _>>()
            .ok()?;
        Some(
            StagedLedgerAuxAndPendingCoinbases {
                scan_state: (ledger.scan_state()).into(),
                staged_ledger_hash: ledger_hash,
                pending_coinbase: (ledger.pending_coinbase_collection()).into(),
                needed_blocks,
            }
            .into(),
        )
    }
}

impl<T: LedgerService> RpcLedgerService for T {
    fn scan_state_summary(
        &self,
        staged_ledger_hash: LedgerHash,
    ) -> Vec<Vec<RpcScanStateSummaryScanStateJob>> {
        use ledger::scan_state::scan_state::JobValue;

        let ledger = self.ctx().staged_ledgers.get(&staged_ledger_hash);
        let Some(ledger) = ledger else { return vec![] };
        ledger
            .scan_state()
            .view()
            .map(|jobs| {
                let jobs = jobs.collect::<Vec<JobValueWithIndex<'_>>>();
                let mut iter = jobs.iter().peekable();
                let mut res = Vec::with_capacity(jobs.len());

                loop {
                    let Some(job) = iter.next() else { break };

                    let (stmt, seq_no, job_kind, is_done) = match &job.job {
                        JobValue::Leaf(JobValueBase::Empty)
                        | JobValue::Node(JobValueMerge::Empty)
                        | JobValue::Node(JobValueMerge::Part(_)) => {
                            res.push(RpcScanStateSummaryScanStateJob::Empty);
                            continue;
                        }
                        JobValue::Leaf(JobValueBase::Full(job)) => {
                            let stmt = &job.job.statement;
                            let tx = job.job.transaction_with_info.transaction();
                            let status = (&tx.status).into();
                            let tx = MinaTransactionTransactionStableV2::from(&tx.data);
                            let kind = RpcScanStateSummaryScanStateJobKind::Base(
                                RpcScanStateSummaryBlockTransaction {
                                    hash: tx.hash().ok(),
                                    kind: (&tx).into(),
                                    status,
                                },
                            );
                            let seq_no = job.seq_no.as_u64();
                            (stmt.clone(), seq_no, kind, job.state.is_done())
                        }
                        JobValue::Node(JobValueMerge::Full(job)) => {
                            let stmt = job
                                .left
                                .proof
                                .statement()
                                .merge(&job.right.proof.statement())
                                .unwrap();
                            let kind = RpcScanStateSummaryScanStateJobKind::Merge;
                            let seq_no = job.seq_no.as_u64();
                            (stmt, seq_no, kind, job.state.is_done())
                        }
                    };
                    let stmt: MinaStateBlockchainStateValueStableV2LedgerProofStatement =
                        (&stmt).into();
                    let job_id: SnarkJobId = (&stmt.source, &stmt.target).into();

                    let bundle =
                        job.bundle_sibling()
                            .and_then(|(sibling_index, is_sibling_left)| {
                                let sibling_job = jobs.get(sibling_index)?;
                                let sibling_stmt: MinaStateBlockchainStateValueStableV2LedgerProofStatement = match &sibling_job.job {
                                    JobValue::Leaf(JobValueBase::Full(job)) => {
                                        (&job.job.statement).into()
                                    }
                                    JobValue::Node(JobValueMerge::Full(job)) => (&job
                                        .left
                                        .proof
                                        .statement()
                                        .merge(&job.right.proof.statement())
                                        .unwrap()).into(),
                                    _ => return None,
                                };
                                let bundle_job_id: SnarkJobId = match is_sibling_left {
                                    false => (&stmt.source, &sibling_stmt.target).into(),
                                    true => (&sibling_stmt.source, &stmt.target).into(),
                                };
                                Some((bundle_job_id, is_sibling_left))
                            });

                    let bundle_job_id = bundle
                        .as_ref()
                        .map_or_else(|| job_id.clone(), |(id, _)| id.clone());

                    res.push(if is_done {
                        let is_left =
                            bundle.map_or_else(|| true, |(_, is_sibling_left)| !is_sibling_left);
                        let sok_message: MinaBaseSokMessageStableV1 = job
                            .parent()
                            .and_then(|parent| {
                                let job = jobs.get(parent)?;
                                let sok_message = match &job.job {
                                    JobValue::Node(JobValueMerge::Part(job)) if is_left => {
                                        (&job.sok_message).into()
                                    }
                                    JobValue::Node(JobValueMerge::Full(job)) => {
                                        if is_left {
                                            (&job.left.sok_message).into()
                                        } else {
                                            (&job.right.sok_message).into()
                                        }
                                    }
                                    state => panic!(
                                        "parent of a `Done` job can't be in this state: {:?}",
                                        state
                                    ),
                                };
                                Some(sok_message)
                            })
                            .unwrap();
                        RpcScanStateSummaryScanStateJob::Done {
                            job_id,
                            bundle_job_id,
                            job: job_kind,
                            seq_no,
                            snark: RpcSnarkPoolJobSnarkWorkDone {
                                snarker: sok_message.prover,
                                fee: sok_message.fee,
                            },
                        }
                    } else {
                        RpcScanStateSummaryScanStateJob::Todo {
                            job_id,
                            bundle_job_id,
                            job: job_kind,
                            seq_no,
                        }
                    })
                }
                res
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use mina_p2p_messages::v2::MinaBaseLedgerHash0StableV1;

    use super::*;

    #[test]
    fn test_ledger_hash() {
        IntoIterator::into_iter([(
            LedgerAddress::root(),
            "jx5YAT36bv62M8mPcREYYfZWXaKqqMzDCP8wmc21uf4CfDKAHCr",
            "jxo5pSyt16XGwA9UeuAdiFDzrwFH3smbNTJF7fxq98w1y9Jem2m",
            "jwq3nCDr8XejL8HKDxR5qVhFJbKoUTGZgtLBZCp3MrqLTnqmjdP",
        )])
        .map(|(addr, expected_hash, left, right)| {
            let left: LedgerHash = left.parse().unwrap();
            let right: LedgerHash = right.parse().unwrap();
            (addr, expected_hash, left, right)
        })
        .for_each(|(address, expected_hash, left, right)| {
            let hash = ledger_hash(address.length(), left.0.to_field(), right.0.to_field());
            let hash: LedgerHash = MinaBaseLedgerHash0StableV1(hash.into()).into();
            assert_eq!(hash.to_string(), expected_hash);
        });
    }
}
