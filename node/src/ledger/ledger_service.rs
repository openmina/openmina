use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::Arc,
};

use ledger::{
    scan_state::{
        currency::Slot,
        scan_state::{AvailableJobMessage, JobValueBase, JobValueMerge, JobValueWithIndex, Pass},
        transaction_logic::{
            local_state::LocalState,
            protocol_state::{protocol_state_view, ProtocolStateView},
            transaction_partially_applied::TransactionPartiallyApplied,
            Transaction,
        },
    },
    sparse_ledger::SparseLedger,
    staged_ledger::{
        diff::Diff,
        staged_ledger::{SkipVerification, StagedLedger},
        validate_block::block_body_hash,
    },
    verifier::Verifier,
    Account, AccountIndex, BaseLedger, Database, Mask, TreeVersion, UnregisterBehavior,
};
use mina_hasher::Fp;
use mina_p2p_messages::{
    binprot::BinProtRead,
    v2::{
        self, DataHashLibStateHashStableV1, LedgerHash, MinaBaseAccountBinableArgStableV2,
        MinaBaseLedgerHash0StableV1, MinaBasePendingCoinbaseStableV2,
        MinaBasePendingCoinbaseWitnessStableV2, MinaBaseSokMessageStableV1,
        MinaBaseStagedLedgerHashStableV1, MinaLedgerSyncLedgerAnswerStableV2,
        MinaLedgerSyncLedgerQueryStableV1,
        MinaStateBlockchainStateValueStableV2LedgerProofStatement,
        MinaStateProtocolStateValueStableV2, MinaTransactionTransactionStableV2, NonZeroCurvePoint,
        StateHash,
    },
};
use openmina_core::constants::CONSTRAINT_CONSTANTS;
use openmina_core::snark::{Snark, SnarkJobId};

use mina_signer::CompressedPubKey;
use openmina_core::block::ArcBlockWithHash;

use crate::block_producer::vrf_evaluator::BlockProducerVrfEvaluatorLedgerService;
use crate::block_producer::{
    BlockProducerLedgerService, BlockProducerWonSlot, StagedLedgerDiffCreateOutput,
};
use crate::transition_frontier::sync::ledger::staged::TransitionFrontierSyncLedgerStagedService;
use crate::transition_frontier::sync::{
    ledger::staged::StagedLedgerAuxAndPendingCoinbasesValid,
    TransitionFrontierRootSnarkedLedgerUpdates,
};
use crate::transition_frontier::TransitionFrontierService;
use crate::{account::AccountPublicKey, block_producer::vrf_evaluator::DelegatorTable};
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

fn ledger_hash(depth: usize, left: Fp, right: Fp) -> Fp {
    let height = LEDGER_DEPTH - depth - 1;
    ledger::V2::hash_node(height, left, right)
}

fn merkle_root(mask: &mut Mask) -> LedgerHash {
    MinaBaseLedgerHash0StableV1(mask.merkle_root().into()).into()
}

#[derive(Default)]
pub struct LedgerCtx {
    snarked_ledgers: BTreeMap<LedgerHash, Mask>,
    /// Additional snarked ledgers specified at startup (loaded from disk)
    additional_snarked_ledgers: BTreeMap<LedgerHash, Mask>,
    staged_ledgers: BTreeMap<LedgerHash, StagedLedger>,
    sync: LedgerSyncState,
}

#[derive(Default)]
struct LedgerSyncState {
    snarked_ledgers: BTreeMap<LedgerHash, Mask>,
    staged_ledgers: BTreeMap<LedgerHash, StagedLedger>,
}

impl LedgerCtx {
    pub fn new_with_additional_snarked_ledgers<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        use std::fs;

        let Ok(dir) = fs::read_dir(path) else {
            return Self::default();
        };

        let additional_snarked_ledgers = dir
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let hash = entry.file_name().to_str()?.parse().ok()?;
                let mut file = fs::File::open(entry.path()).ok()?;

                let _ = Option::<LedgerHash>::binprot_read(&mut file).ok()?;

                let accounts = Vec::<Account>::binprot_read(&mut file).ok()?;
                let mut mask = Mask::new_root(Database::create(35));
                for account in accounts {
                    let account_id = account.id();
                    mask.get_or_create_account(account_id, account).unwrap();
                }
                Some((hash, mask))
            })
            .collect();

        LedgerCtx {
            additional_snarked_ledgers,
            ..Default::default()
        }
    }

    pub fn insert_genesis_ledger(&mut self, mut mask: Mask) {
        let hash = merkle_root(&mut mask);
        self.snarked_ledgers.insert(hash.clone(), mask.clone());
        let staged_ledger = StagedLedger::create_exn(CONSTRAINT_CONSTANTS, mask).unwrap();
        self.staged_ledgers.insert(hash.clone(), staged_ledger);
    }

    // TODO(tizoc): explain when `is_synced` is `true` and when it is `false`. Also use something else than a boolean.
    /// Returns a tuple of `(mask, is_synced)` for a [Mask] with the specified `hash` if it exists or `None` otherwise.
    pub fn mask(&self, hash: &LedgerHash) -> Option<(Mask, bool)> {
        self.snarked_ledgers
            .get(hash)
            .cloned()
            .map(|mask| (mask, true))
            .or_else(|| Some((self.staged_ledgers.get(hash)?.ledger(), true)))
            .or_else(|| self.sync.mask(hash))
    }

    /// Returns a mutable reference to the [StagedLedger] with the specified `hash` if it exists or `None` otherwise.
    fn staged_ledger_mut(&mut self, hash: &LedgerHash) -> Option<&mut StagedLedger> {
        match self.staged_ledgers.get_mut(&hash) {
            Some(v) => Some(v),
            None => self.sync.staged_ledger_mut(&hash),
        }
    }

    fn recreate_snarked_ledger(
        &mut self,
        root_snarked_ledger_updates: &TransitionFrontierRootSnarkedLedgerUpdates,
        needed_protocol_states: &BTreeMap<StateHash, MinaStateProtocolStateValueStableV2>,
        snarked_ledger_hash: &LedgerHash,
    ) -> Result<(), String> {
        let Some(update) = root_snarked_ledger_updates.get(snarked_ledger_hash) else {
            return Ok(());
        };
        self.recreate_snarked_ledger(
            root_snarked_ledger_updates,
            needed_protocol_states,
            &update.parent,
        )?;

        self.push_snarked_ledger(
            needed_protocol_states,
            &update.parent,
            snarked_ledger_hash,
            &update.staged_ledger_hash,
        )
    }

    fn push_snarked_ledger(
        &mut self,
        protocol_states: &BTreeMap<StateHash, MinaStateProtocolStateValueStableV2>,
        old_root_snarked_ledger_hash: &LedgerHash,
        new_root_snarked_ledger_hash: &LedgerHash,
        new_root_staged_ledger_hash: &LedgerHash,
    ) -> Result<(), String> {
        openmina_core::debug!(openmina_core::log::system_time();
            kind = "LedgerService::push_snarked_ledger",
            summary = format!("{old_root_snarked_ledger_hash} -> {new_root_snarked_ledger_hash}"));
        // Steps 4-7 from https://github.com/openmina/mina/blob/bc812dc9b90e05898c0c36ac76ba51ccf6cac137/src/lib/transition_frontier/full_frontier/full_frontier.ml#L354-L392
        let constraint_constants = &CONSTRAINT_CONSTANTS;

        // Step 4: create a new temporary mask `mt` with `s` as it's parent
        let root_snarked_ledger = self
            .snarked_ledgers
            .get(old_root_snarked_ledger_hash)
            .or_else(|| self.sync.snarked_ledgers.get(old_root_snarked_ledger_hash))
            .ok_or_else(|| {
                format!(
                    "push_snarked_ledger: could not find old root snarked ledger: {}",
                    old_root_snarked_ledger_hash.to_string(),
                )
            })?;
        let mut mt = root_snarked_ledger.make_child();

        // Step 5: apply any transactions to `mt` that appear in the transition between `s` and `s'`
        let apply_first_pass = |global_slot: Slot,
                                txn_state_view: &ProtocolStateView,
                                ledger: &mut Mask,
                                transaction: &Transaction| {
            ledger::scan_state::transaction_logic::apply_transaction_first_pass(
                constraint_constants,
                global_slot,
                txn_state_view,
                ledger,
                transaction,
            )
        };

        let apply_second_pass = |ledger: &mut Mask, tx: TransactionPartiallyApplied<Mask>| {
            ledger::scan_state::transaction_logic::apply_transaction_second_pass(
                constraint_constants,
                ledger,
                tx,
            )
        };

        let apply_first_pass_sparse_ledger =
            |global_slot: Slot,
             txn_state_view: &ProtocolStateView,
             sparse_ledger: &mut SparseLedger,
             transaction: &Transaction| {
                ledger::scan_state::transaction_logic::apply_transaction_first_pass(
                    constraint_constants,
                    global_slot,
                    txn_state_view,
                    sparse_ledger,
                    transaction,
                )
            };

        let get_protocol_state = |state_hash: Fp| {
            let state_hash = StateHash::from_fp(state_hash);
            if let Some(s) = protocol_states.get(&state_hash) {
                Ok(s.clone())
            } else {
                Err(format!(
                    "Failed to find protocol state for state hash: {}",
                    state_hash.to_string()
                ))
            }
        };

        let scan_state = self
            .staged_ledger_mut(new_root_staged_ledger_hash)
            .ok_or_else(|| {
                format!(
                    "Failed to find staged ledger with hash: {}",
                    new_root_staged_ledger_hash.to_string()
                )
            })?
            .scan_state();

        let Pass::FirstPassLedgerHash(_first_pass_ledger_target) = scan_state
            .get_snarked_ledger_sync(
                &mut mt,
                get_protocol_state,
                apply_first_pass,
                apply_second_pass,
                apply_first_pass_sparse_ledger,
            )?;

        // Assert that the obtained ledger is the one we expect
        let expected_hash = new_root_snarked_ledger_hash;
        let obtained_hash = LedgerHash::from_fp(mt.merkle_root());

        if expected_hash != &obtained_hash {
            return Err(format!(
                "Expected to obtain snarked root ledger hash {} but got {}",
                expected_hash.to_string(),
                obtained_hash.to_string()
            ));
        }

        self.sync
            .snarked_ledgers
            .insert(new_root_snarked_ledger_hash.clone(), mt);

        Ok(())
    }

    pub fn producers_with_delegates<F: FnMut(&CompressedPubKey) -> bool>(
        &self,
        ledger_hash: &LedgerHash,
        mut filter: F,
    ) -> Option<BTreeMap<AccountPublicKey, Vec<(ledger::AccountIndex, AccountPublicKey, u64)>>>
    {
        let (mask, _) = self.mask(ledger_hash)?;
        let mut accounts = Vec::new();

        mask.iter(|account| {
            if filter(&account.public_key)
                || account.delegate.as_ref().map_or(false, |key| filter(key))
            {
                accounts.push((
                    account.id(),
                    account.delegate.clone(),
                    account.balance.as_u64(),
                ))
            }
        });

        let producers = accounts.into_iter().fold(
            BTreeMap::<_, Vec<_>>::new(),
            |mut producers, (id, delegate, balance)| {
                let index = mask.index_of_account(id.clone()).unwrap();
                let pub_key = AccountPublicKey::from(id.public_key);
                let producer = delegate.map(Into::into).unwrap_or(pub_key.clone());
                producers
                    .entry(producer)
                    .or_default()
                    .push((index, pub_key, balance));
                producers
            },
        );
        Some(producers)
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

    /// Returns a [Mask] instance for the snarked ledger with [hash]. If it doesn't
    /// exist a new instance is created.
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
    ) -> Result<(), String> {
        let (left, right) = (left.0.to_field(), right.0.to_field());
        let hash = ledger_hash(parent.length(), left, right);

        let mask = self.ctx_mut().sync.snarked_ledger_mut(snarked_ledger_hash);

        if hash != mask.get_inner_hash_at_addr(parent.clone())? {
            return Err("Inner hash found at address but doesn't match the expected hash".into());
        }

        // TODO(binier): the `if` condition is temporary until we make
        // sure we don't call `hashes_set` for the same key for the
        // same ledger. This can happen E.g. if root snarked ledger
        // is the same as staking or next epoch ledger, in which case
        // we will sync same ledger twice. That causes assertion to fail
        // in `set_cached_hash_unchecked`.
        //
        // remove once we have an optimization to not sync same ledgers/addrs
        // multiple times.
        if mask.get_cached_hash(&parent.child_left()).is_none() {
            mask.set_cached_hash_unchecked(&parent.child_left(), left);
            mask.set_cached_hash_unchecked(&parent.child_right(), right);
        }

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
                mask.set_at_index(index, Box::new((&account).into()))
            })?;

        Ok(())
    }
}

impl<T: LedgerService> TransitionFrontierSyncLedgerStagedService for T {
    fn staged_ledger_reconstruct(
        &mut self,
        snarked_ledger_hash: LedgerHash,
        parts: Option<Arc<StagedLedgerAuxAndPendingCoinbasesValid>>,
    ) -> Result<(), String> {
        let staged_ledger_hash = parts
            .as_ref()
            .map(|p| p.staged_ledger_hash.clone())
            .unwrap_or_else(|| snarked_ledger_hash.clone());
        let snarked_ledger = self
            .ctx_mut()
            .sync
            .snarked_ledger_mut(snarked_ledger_hash.clone());
        // TODO(binier): TMP. Remove for prod version.
        snarked_ledger
            .validate_inner_hashes()
            .map_err(|_| "downloaded hash and recalculated mismatch".to_owned())?;

        let mask = snarked_ledger.copy();

        let staged_ledger = if let Some(parts) = parts {
            let states = parts
                .needed_blocks
                .iter()
                .map(|state| (state.hash().to_fp().unwrap(), state.clone()))
                .collect::<BTreeMap<_, _>>();

            StagedLedger::of_scan_state_pending_coinbases_and_snarked_ledger(
                (),
                &CONSTRAINT_CONSTANTS,
                Verifier,
                (&parts.scan_state).into(),
                mask,
                LocalState::empty(),
                parts.staged_ledger_hash.0.to_field(),
                (&parts.pending_coinbase).into(),
                |key| states.get(&key).cloned().unwrap(),
            )?
        } else {
            StagedLedger::create_exn(CONSTRAINT_CONSTANTS.clone(), mask)?
        };

        self.ctx_mut()
            .sync
            .staged_ledgers
            .insert(staged_ledger_hash, staged_ledger);

        Ok(())
    }
}

impl<T: LedgerService> TransitionFrontierService for T {
    fn block_apply(
        &mut self,
        block: ArcBlockWithHash,
        pred_block: ArcBlockWithHash,
    ) -> Result<(), String> {
        openmina_core::debug!(openmina_core::log::system_time();
            kind = "LedgerService::block_apply",
            summary = format!("{}, {} <- {}", block.height(), block.hash(), block.pred_hash()),
            snarked_ledger_hash = block.snarked_ledger_hash().to_string(),
            staged_ledger_hash = block.staged_ledger_hash().to_string(),
        );
        let mut staged_ledger = self
            .ctx_mut()
            .staged_ledger_mut(&pred_block.staged_ledger_hash())
            .ok_or_else(|| "parent staged ledger missing")?
            .clone();

        let global_slot = block.global_slot_since_genesis();
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
            let staged_ledger = self
                .ctx_mut()
                .staged_ledger_mut(&pred_block.staged_ledger_hash())
                .unwrap(); // We already know the ledger exists, see the same call a few lines above

            match dump_application_to_file(staged_ledger, block.clone(), pred_block) {
                Ok(filename) => openmina_core::info!(
                    openmina_core::log::system_time();
                    kind = "LedgerService::dump - Failed application",
                    summary = format!("StagedLedger and block saved to: {filename:?}")
                ),
                Err(e) => openmina_core::error!(
                    openmina_core::log::system_time();
                    kind = "LedgerService::dump - Failed application",
                    summary = format!("Failed to save block application to file: {e:?}")
                ),
            }

            panic!("staged ledger hash mismatch. found: {ledger_hashes:#?}, expected: {expected_ledger_hashes:#?}");
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
        root_snarked_ledger_updates: TransitionFrontierRootSnarkedLedgerUpdates,
        needed_protocol_states: BTreeMap<StateHash, MinaStateProtocolStateValueStableV2>,
        new_root: &ArcBlockWithHash,
        new_best_tip: &ArcBlockWithHash,
    ) -> CommitResult {
        let ctx = self.ctx_mut();

        openmina_core::debug!(openmina_core::log::system_time();
            kind = "LedgerService::commit",
            summary = format!("commit {}, {}", new_best_tip.height(), new_best_tip.hash()),
            new_root = format!("{}, {}", new_root.height(), new_root.hash()),
            new_root_staking_epoch_ledger = new_root.staking_epoch_ledger_hash().to_string(),
            new_root_next_epoch_ledger = new_root.next_epoch_ledger_hash().to_string(),
            new_root_snarked_ledger = new_root.snarked_ledger_hash().to_string(),
        );
        ctx.recreate_snarked_ledger(
            &root_snarked_ledger_updates,
            &needed_protocol_states,
            new_root.snarked_ledger_hash(),
        )
        .unwrap();

        ctx.snarked_ledgers.retain(|hash, _| {
            let keep = ledgers_to_keep.contains(hash);
            if !keep {
                openmina_core::debug!(openmina_core::log::system_time();
                    kind = "LedgerService::commit - snarked_ledgers.drop",
                    summary = format!("drop snarked ledger {hash}"));
            }
            keep
        });
        ctx.snarked_ledgers.extend(
            std::mem::take(&mut ctx.sync.snarked_ledgers)
                .into_iter()
                .filter(|(hash, _)| {
                    let keep = ledgers_to_keep.contains(hash);
                    if !keep {
                        openmina_core::debug!(openmina_core::log::system_time();
                            kind = "LedgerService::commit - snarked_ledgers.drop",
                            summary = format!("drop snarked ledger {hash}"));
                    }
                    keep
                }),
        );

        ctx.staged_ledgers
            .retain(|hash, _| ledgers_to_keep.contains(hash));
        ctx.staged_ledgers.extend(
            std::mem::take(&mut ctx.sync.staged_ledgers)
                .into_iter()
                .filter(|(hash, _)| ledgers_to_keep.contains(hash)),
        );

        for ledger_hash in [
            new_best_tip.staking_epoch_ledger_hash(),
            new_root.snarked_ledger_hash(),
            new_root.staged_ledger_hash(),
        ] {
            if let Some((mut mask, is_synced)) = ctx.mask(ledger_hash) {
                if !is_synced {
                    panic!("ledger mask expected to be synced: {ledger_hash}");
                }
                let calculated = merkle_root(&mut mask);
                assert_eq!(
                    ledger_hash, &calculated,
                    "ledger mask hash mismatch, expected: {ledger_hash}, found {calculated}"
                );
            } else {
                panic!("ledger mask is missing: {ledger_hash}");
            }
        }

        for (ledger_hash, snarked_ledger) in ctx.snarked_ledgers.iter_mut() {
            while let Some((parent_hash, parent)) = snarked_ledger
                .get_parent()
                .map(|mut parent| (merkle_root(&mut parent), parent))
                .filter(|(parent_hash, _)| !ledgers_to_keep.contains(parent_hash))
            {
                openmina_core::debug!(openmina_core::log::system_time();
                    kind = "LedgerService::commit - mask.commit_and_reparent",
                    summary = format!("{ledger_hash} -> {parent_hash}"));
                snarked_ledger.commit();
                snarked_ledger.unregister_mask(UnregisterBehavior::Check);
                *snarked_ledger = parent;
            }
        }

        // TODO(tizoc): should this fail silently?
        let Some(new_root_ledger) = ctx.staged_ledgers.get_mut(new_root.staged_ledger_hash())
        else {
            return Default::default();
        };

        // Make staged ledger mask new root.
        new_root_ledger.commit_and_reparent_to_root();

        let needed_protocol_states = ctx
            .staged_ledger_mut(new_root.staged_ledger_hash())
            .map(|l| {
                l.scan_state()
                    .required_state_hashes()
                    .into_iter()
                    .map(|fp| DataHashLibStateHashStableV1(fp.into()).into())
                    .collect()
            })
            .unwrap_or_default();

        let available_jobs = ctx
            .staged_ledger_mut(new_best_tip.staged_ledger_hash())
            .map(|l| {
                l.scan_state()
                    .all_job_pairs_iter()
                    .map(|job| job.map(|single| AvailableJobMessage::from(single)))
                    .collect()
            })
            .unwrap_or_default();

        CommitResult {
            available_jobs,
            needed_protocol_states,
        }
    }

    fn answer_ledger_query(
        &mut self,
        ledger_hash: LedgerHash,
        query: MinaLedgerSyncLedgerQueryStableV1,
    ) -> Option<MinaLedgerSyncLedgerAnswerStableV2> {
        let ctx = self.ctx_mut();
        let (mask, is_synced) = ctx
            .mask(&ledger_hash)
            .filter(|(_, is_synced)| *is_synced)
            .or_else(|| {
                ctx.additional_snarked_ledgers
                    .get(&ledger_hash)
                    .map(|l| (l.clone(), true))
            })?;

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
                    .map(|(_, account)| (&*account).into())
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

impl<T: LedgerService> BlockProducerLedgerService for T {
    fn staged_ledger_diff_create(
        &mut self,
        pred_block: &ArcBlockWithHash,
        won_slot: &BlockProducerWonSlot,
        coinbase_receiver: &NonZeroCurvePoint,
        completed_snarks: BTreeMap<SnarkJobId, Snark>,
        supercharge_coinbase: bool,
    ) -> Result<StagedLedgerDiffCreateOutput, String> {
        let mut staged_ledger = self
            .ctx_mut()
            .staged_ledger_mut(&pred_block.staged_ledger_hash())
            .ok_or_else(|| "parent staged ledger missing")?
            .clone();

        // calculate merkle root hash, otherwise `MinaBasePendingCoinbaseStableV2::from` fails.
        staged_ledger.hash();
        let pending_coinbase_witness =
            MinaBasePendingCoinbaseStableV2::from(staged_ledger.pending_coinbase_collection());

        let protocol_state_view = protocol_state_view(&pred_block.header().protocol_state);
        let global_slot_since_genesis =
            won_slot.global_slot_since_genesis(pred_block.global_slot_diff());

        // TODO(binier): include `invalid_txns` in output.
        let (pre_diff, _invalid_txns) = staged_ledger
            .create_diff(
                &CONSTRAINT_CONSTANTS,
                (&global_slot_since_genesis).into(),
                Some(true),
                coinbase_receiver.into(),
                (),
                &protocol_state_view,
                // TODO(binier): once we have transaction pool, pass
                // transactions here.
                Vec::new(),
                |stmt| {
                    let job_id = SnarkJobId::from(stmt);
                    completed_snarks.get(&job_id).map(Into::into)
                },
                supercharge_coinbase,
            )
            .map_err(|err| format!("{err:?}"))?;

        // TODO(binier): maybe here, check if block reward is above threshold.
        // https://github.com/minaprotocol/mina/blob/b3d418a8c0ae4370738886c2b26f0ec7bdb49303/src/lib/block_producer/block_producer.ml#L222

        let pred_body_hash = pred_block.header().protocol_state.body.hash();
        let diff = (&pre_diff).into();

        let res = staged_ledger
            .apply_diff_unchecked(
                &CONSTRAINT_CONSTANTS,
                (&global_slot_since_genesis).into(),
                pre_diff,
                (),
                &protocol_state_view,
                (pred_block.hash().0.to_field(), pred_body_hash.0.to_field()),
                coinbase_receiver.into(),
                supercharge_coinbase,
            )
            .map_err(|err| format!("{err:?}"))?;

        let diff_hash = block_body_hash(&diff).map_err(|err| format!("{err:?}"))?;

        Ok(StagedLedgerDiffCreateOutput {
            diff,
            diff_hash,
            staged_ledger_hash: (&res.hash_after_applying).into(),
            emitted_ledger_proof: res
                .ledger_proof
                .map(|(proof, ..)| (&proof).into())
                .map(Box::new),
            pending_coinbase_update: (&res.pending_coinbase_update.1).into(),
            pending_coinbase_witness: MinaBasePendingCoinbaseWitnessStableV2 {
                pending_coinbases: pending_coinbase_witness,
                is_new_stack: res.pending_coinbase_update.0,
            },
        })
    }

    fn stake_proof_sparse_ledger(
        &mut self,
        staking_ledger: LedgerHash,
        producer: NonZeroCurvePoint,
        delegator: NonZeroCurvePoint,
    ) -> Option<v2::MinaBaseSparseLedgerBaseStableV2> {
        let mask = self.ctx_mut().snarked_ledgers.get(&staking_ledger)?;
        let producer_id = ledger::AccountId::new((&producer).into(), ledger::TokenId::default());
        let delegator_id = ledger::AccountId::new((&delegator).into(), ledger::TokenId::default());
        let sparse_ledger = ledger::sparse_ledger::SparseLedger::of_ledger_subset_exn(
            mask.clone(),
            &[producer_id, delegator_id],
        );
        Some((&sparse_ledger).into())
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

impl<T: LedgerService> BlockProducerVrfEvaluatorLedgerService for T {
    fn get_producer_and_delegates(
        &mut self,
        ledger_hash: LedgerHash,
        producer: AccountPublicKey,
    ) -> DelegatorTable {
        // TODO(adonagy): Error handling
        let delegate_table = self
            .ctx()
            .producers_with_delegates(&ledger_hash, |pub_key| {
                AccountPublicKey::from(pub_key.clone()) == producer
            })
            .unwrap()
            .into_values()
            .next()
            .unwrap();

        delegate_table
            .into_iter()
            .map(|(index, pub_key, balance)| (index, (pub_key, balance)))
            .collect()
    }
}

/// Save staged ledger and block to file, when the application fail.
/// So we can easily reproduce the application both in Rust and OCaml, to compare them.
/// - https://github.com/openmina/openmina/blob/8e68037aafddd43842a54c8439baeafee4c6e1eb/ledger/src/staged_ledger/staged_ledger.rs#L5959
/// - TODO: Find OCaml link, I remember having the same test in OCaml but I can't find where
fn dump_application_to_file(
    staged_ledger: &StagedLedger,
    block: ArcBlockWithHash,
    pred_block: ArcBlockWithHash,
) -> std::io::Result<String> {
    use mina_p2p_messages::{
        binprot,
        binprot::macros::{BinProtRead, BinProtWrite},
    };

    #[derive(BinProtRead, BinProtWrite)]
    struct ApplyContext {
        accounts: Vec<v2::MinaBaseAccountBinableArgStableV2>,
        scan_state: v2::TransactionSnarkScanStateStableV2,
        pending_coinbase: v2::MinaBasePendingCoinbaseStableV2,
        pred_block: v2::MinaBlockBlockStableV2,
        blocks: Vec<v2::MinaBlockBlockStableV2>,
    }

    let cs = &block.block.header.protocol_state.body.consensus_state;
    let block_height = cs.blockchain_length.as_u32();

    let apply_context = ApplyContext {
        accounts: staged_ledger
            .ledger()
            .to_list()
            .iter()
            .map(v2::MinaBaseAccountBinableArgStableV2::from)
            .collect::<Vec<_>>(),
        scan_state: staged_ledger.scan_state().into(),
        pending_coinbase: staged_ledger.pending_coinbase_collection().into(),
        pred_block: (*pred_block.block).clone(),
        blocks: vec![(*block.block).clone()],
    };

    use mina_p2p_messages::binprot::BinProtWrite;
    let filename = format!("/tmp/failed_application_ctx_{}.binprot", block_height);
    let mut file = std::fs::File::create(&filename)?;
    apply_context.binprot_write(&mut file)?;
    file.sync_all()?;

    Ok(filename)
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
