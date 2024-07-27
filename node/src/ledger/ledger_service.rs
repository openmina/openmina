use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::Arc,
};

use super::ledger_manager::{LedgerManager, LedgerRequest};
use ledger::{
    scan_state::{
        currency::Slot,
        scan_state::{AvailableJobMessage, JobValueBase, JobValueMerge, JobValueWithIndex, Pass},
        transaction_logic::{
            local_state::LocalState,
            protocol_state::{protocol_state_view, ProtocolStateView},
            transaction_partially_applied::TransactionPartiallyApplied,
            valid, Transaction,
        },
    },
    sparse_ledger::SparseLedger,
    staged_ledger::{
        diff::Diff,
        staged_ledger::{SkipVerification, StagedLedger},
        validate_block::block_body_hash,
    },
    verifier::Verifier,
    Account, AccountId, BaseLedger, Database, Mask, UnregisterBehavior,
};
use mina_hasher::Fp;
use mina_p2p_messages::{
    binprot::BinProtRead,
    v2::{
        self, DataHashLibStateHashStableV1, LedgerHash, MinaBaseLedgerHash0StableV1,
        MinaBasePendingCoinbaseStableV2, MinaBasePendingCoinbaseWitnessStableV2,
        MinaBaseSokMessageStableV1, MinaBaseStagedLedgerHashStableV1,
        MinaStateBlockchainStateValueStableV2LedgerProofStatement,
        MinaStateProtocolStateValueStableV2, MinaTransactionTransactionStableV2, NonZeroCurvePoint,
        StateHash,
    },
};
use openmina_core::constants::constraint_constants;
use openmina_core::snark::{Snark, SnarkJobId};
use openmina_core::thread;

use mina_signer::CompressedPubKey;
use openmina_core::block::ArcBlockWithHash;

use crate::account::AccountPublicKey;
use crate::block_producer::StagedLedgerDiffCreateOutput;
use crate::p2p::channels::rpc::StagedLedgerAuxAndPendingCoinbases;
use crate::rpc::{
    RpcScanStateSummaryBlockTransaction, RpcScanStateSummaryScanStateJob,
    RpcScanStateSummaryScanStateJobKind, RpcSnarkPoolJobSnarkWorkDone,
};
use crate::transition_frontier::sync::{
    ledger::staged::StagedLedgerAuxAndPendingCoinbasesValid,
    TransitionFrontierRootSnarkedLedgerUpdates,
};

use super::write::CommitResult;

use super::{
    ledger_empty_hash_at_depth, read::LedgerReadResponse, write::LedgerWriteResponse,
    LedgerAddress, LedgerEvent, LEDGER_DEPTH,
};
use super::{
    read::{LedgerReadId, LedgerReadRequest},
    write::LedgerWriteRequest,
};

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
    event_sender:
        Option<openmina_core::channels::mpsc::UnboundedSender<crate::event_source::Event>>,
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

    // TODO(tizoc): Only used for the current workaround to make staged ledger
    // reconstruction async, can be removed when the ledger services are made async
    pub fn set_event_sender(
        &mut self,
        event_sender: openmina_core::channels::mpsc::UnboundedSender<crate::event_source::Event>,
    ) {
        self.event_sender = Some(event_sender);
    }

    pub(super) fn send_event(&self, event: LedgerEvent) {
        if let Some(tx) = self.event_sender.as_ref() {
            let _ = tx.send(event.into());
        }
    }

    pub(super) fn send_write_response(&self, resp: LedgerWriteResponse) {
        self.send_event(LedgerEvent::Write(resp))
    }

    pub(super) fn send_read_response(&self, id: LedgerReadId, resp: LedgerReadResponse) {
        self.send_event(LedgerEvent::Read(id, resp))
    }

    pub fn insert_genesis_ledger(&mut self, mut mask: Mask) {
        let hash = merkle_root(&mut mask);
        let staged_ledger =
            StagedLedger::create_exn(constraint_constants().clone(), mask.copy()).unwrap();
        self.snarked_ledgers.insert(hash.clone(), mask);
        self.staged_ledgers.insert(hash.clone(), staged_ledger);
    }

    pub fn staged_ledger_reconstruct_result_store(&mut self, ledger: StagedLedger) {
        let hash = merkle_root(&mut ledger.ledger().clone());
        self.staged_ledgers.insert(hash, ledger);
    }

    // TODO(adonagy): Uh-oh, clean this up
    pub fn get_accounts_for_rpc(
        &self,
        ledger_hash: LedgerHash,
        requested_public_key: Option<AccountPublicKey>,
    ) -> Vec<Account> {
        if let Some((mask, _)) = self.mask(&ledger_hash) {
            let mut accounts = Vec::new();
            let mut single_account = Vec::new();

            mask.iter(|account| {
                accounts.push(account.clone());
                if let Some(public_key) = requested_public_key.as_ref() {
                    if public_key == &AccountPublicKey::from(account.public_key.clone()) {
                        single_account.push(account.clone());
                    }
                }
            });

            if requested_public_key.is_some() {
                single_account
            } else {
                accounts
            }
        } else {
            vec![]
        }
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
            .or_else(|| {
                self.additional_snarked_ledgers
                    .get(hash)
                    .map(|l| (l.clone(), true))
            })
    }

    /// Returns the mask for a snarked ledger being synchronized or an error if it is not present
    pub fn pending_sync_snarked_ledger_mask(&self, hash: &LedgerHash) -> Result<Mask, String> {
        self.sync.pending_sync_snarked_ledger_mask(hash)
    }

    /// Copies the contents of an existing snarked ledger into the target
    /// hash under the pending sync snarked ledgers state.
    pub fn copy_snarked_ledger_contents_for_sync(
        &mut self,
        origin_snarked_ledger_hash: LedgerHash,
        target_snarked_ledger_hash: LedgerHash,
        overwrite: bool,
    ) -> Result<bool, String> {
        if !overwrite
            && self
                .sync
                .snarked_ledgers
                .contains_key(&target_snarked_ledger_hash)
        {
            return Ok(false);
        }

        let origin = self
            .snarked_ledgers
            .get(&origin_snarked_ledger_hash)
            .or_else(|| {
                // If it doesn't exist in completed ledgers, it may be
                // an in-progress ledger from a previous attempt that we can reuse
                self.sync.snarked_ledgers.get(&origin_snarked_ledger_hash)
            })
            .ok_or(format!(
                "Tried to copy from non-existing snarked ledger with hash: {}",
                origin_snarked_ledger_hash
            ))?;

        let target = origin.copy();
        self.sync
            .snarked_ledgers
            .insert(target_snarked_ledger_hash, target);

        Ok(true)
    }

    pub fn compute_snarked_ledger_hashes(
        &mut self,
        snarked_ledger_hash: &LedgerHash,
    ) -> Result<(), String> {
        let origin = self
            .snarked_ledgers
            .get_mut(snarked_ledger_hash)
            .or_else(|| self.sync.snarked_ledgers.get_mut(snarked_ledger_hash))
            .ok_or(format!(
                "Cannot hash non-existing snarked ledger: {}",
                snarked_ledger_hash
            ))?;

        // Our ledger is lazy when it comes to hashing, but retrieving the
        // merkle root hash forces all pending hashes to be computed.
        let _force_hashing = origin.merkle_root();

        Ok(())
    }

    /// Returns a mutable reference to the [StagedLedger] with the specified `hash` if it exists or `None` otherwise.
    fn staged_ledger_mut(&mut self, hash: &LedgerHash) -> Option<&mut StagedLedger> {
        match self.staged_ledgers.get_mut(hash) {
            Some(v) => Some(v),
            None => self.sync.staged_ledger_mut(hash),
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
        let constraint_constants = &constraint_constants();

        // Step 4: create a new temporary mask `mt` with `s` as it's parent
        let root_snarked_ledger = self
            .snarked_ledgers
            .get(old_root_snarked_ledger_hash)
            .or_else(|| self.sync.snarked_ledgers.get(old_root_snarked_ledger_hash))
            .ok_or_else(|| {
                format!(
                    "push_snarked_ledger: could not find old root snarked ledger: {}",
                    old_root_snarked_ledger_hash,
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
                    state_hash
                ))
            }
        };

        let scan_state = self
            .staged_ledger_mut(new_root_staged_ledger_hash)
            .ok_or_else(|| {
                format!(
                    "Failed to find staged ledger with hash: {}",
                    new_root_staged_ledger_hash
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
                expected_hash, obtained_hash
            ));
        }

        self.sync
            .snarked_ledgers
            .insert(new_root_snarked_ledger_hash.clone(), mt);

        Ok(())
    }

    #[allow(clippy::type_complexity)]
    pub fn producers_with_delegates<F: FnMut(&CompressedPubKey) -> bool>(
        &self,
        ledger_hash: &LedgerHash,
        mut filter: F,
    ) -> Option<BTreeMap<AccountPublicKey, Vec<(ledger::AccountIndex, AccountPublicKey, u64)>>>
    {
        let (mask, _) = self.mask(ledger_hash)?;
        let mut accounts = Vec::new();

        mask.iter(|account| {
            if filter(&account.public_key) || account.delegate.as_ref().map_or(false, &mut filter) {
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

    pub fn child_hashes_get(
        &mut self,
        snarked_ledger_hash: LedgerHash,
        parent: &LedgerAddress,
    ) -> Result<(LedgerHash, LedgerHash), String> {
        let mut mask = self.pending_sync_snarked_ledger_mask(&snarked_ledger_hash)?;
        let left_hash = LedgerHash::from_fp(mask.get_inner_hash_at_addr(parent.child_left())?);
        let right_hash = LedgerHash::from_fp(mask.get_inner_hash_at_addr(parent.child_right())?);

        Ok((left_hash, right_hash))
    }

    pub fn accounts_set(
        &mut self,
        snarked_ledger_hash: LedgerHash,
        parent: &LedgerAddress,
        accounts: Vec<v2::MinaBaseAccountBinableArgStableV2>,
    ) -> Result<LedgerHash, String> {
        let mut mask = self.pending_sync_snarked_ledger_mask(&snarked_ledger_hash)?;
        let accounts: Vec<_> = accounts
            .into_iter()
            .map(|account| Box::new((&account).into()))
            .collect();

        mask.set_all_accounts_rooted_at(parent.clone(), &accounts)
            .map_err(|_| "Failed when setting accounts".to_owned())?;

        let computed_hash = LedgerHash::from_fp(mask.get_inner_hash_at_addr(parent.clone())?);

        Ok(computed_hash)
    }

    pub fn staged_ledger_reconstruct<F>(
        &mut self,
        snarked_ledger_hash: LedgerHash,
        parts: Option<Arc<StagedLedgerAuxAndPendingCoinbasesValid>>,
        callback: F,
    ) where
        F: 'static + FnOnce(v2::LedgerHash, Result<StagedLedger, String>) + Send,
    {
        let snarked_ledger = self
            .sync
            .snarked_ledger_mut(snarked_ledger_hash.clone())
            .copy();

        thread::Builder::new()
            .name("staged-ledger-reconstruct".into())
            .spawn(move || {
                let (staged_ledger_hash, result) =
                    staged_ledger_reconstruct(snarked_ledger, snarked_ledger_hash, parts);
                callback(staged_ledger_hash, result);
            })
            .expect("Failed: staged ledger reconstruct thread");
    }

    pub fn staged_ledger_reconstruct_sync(
        &mut self,
        snarked_ledger_hash: LedgerHash,
        parts: Option<Arc<StagedLedgerAuxAndPendingCoinbasesValid>>,
    ) -> (v2::LedgerHash, Result<(), String>) {
        let snarked_ledger = self
            .sync
            .snarked_ledger_mut(snarked_ledger_hash.clone())
            .copy();
        let (staged_ledger_hash, result) =
            staged_ledger_reconstruct(snarked_ledger, snarked_ledger_hash, parts);
        let result = match result {
            Err(err) => Err(err),
            Ok(staged_ledger) => {
                self.staged_ledger_reconstruct_result_store(staged_ledger);
                Ok(())
            }
        };

        (staged_ledger_hash, result)
    }

    pub fn block_apply(
        &mut self,
        block: ArcBlockWithHash,
        pred_block: ArcBlockWithHash,
    ) -> Result<(), String> {
        openmina_core::info!(openmina_core::log::system_time();
            kind = "LedgerService::block_apply",
            summary = format!("{}, {} <- {}", block.height(), block.hash(), block.pred_hash()),
            pred_staged_ledger_hash = pred_block.staged_ledger_hash().to_string(),
            staged_ledger_hash = block.staged_ledger_hash().to_string(),
        );
        let mut staged_ledger = self
            .staged_ledger_mut(pred_block.staged_ledger_hash())
            .ok_or_else(|| {
                format!(
                    "parent staged ledger missing: {}",
                    pred_block.staged_ledger_hash()
                )
            })?
            .clone();

        let global_slot = block.global_slot_since_genesis();
        let prev_protocol_state = &pred_block.header().protocol_state;
        let prev_state_view = protocol_state_view(prev_protocol_state);

        let consensus_state = &block.header().protocol_state.body.consensus_state;
        let coinbase_receiver: CompressedPubKey = (&consensus_state.coinbase_receiver).into();
        let supercharge_coinbase = consensus_state.supercharge_coinbase;

        let diff: Diff = (&block.block.body.staged_ledger_diff).into();

        let result = staged_ledger
            .apply(
                // TODO(binier): SEC
                Some(SkipVerification::All),
                constraint_constants(),
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
                .staged_ledger_mut(pred_block.staged_ledger_hash())
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
        self.sync
            .staged_ledgers
            .insert(ledger_hash.clone(), staged_ledger);

        Ok(())
    }

    pub fn commit(
        &mut self,
        ledgers_to_keep: BTreeSet<LedgerHash>,
        root_snarked_ledger_updates: TransitionFrontierRootSnarkedLedgerUpdates,
        needed_protocol_states: BTreeMap<StateHash, MinaStateProtocolStateValueStableV2>,
        new_root: &ArcBlockWithHash,
        new_best_tip: &ArcBlockWithHash,
    ) -> CommitResult {
        openmina_core::debug!(openmina_core::log::system_time();
            kind = "LedgerService::commit",
            summary = format!("commit {}, {}", new_best_tip.height(), new_best_tip.hash()),
            new_root = format!("{}, {}", new_root.height(), new_root.hash()),
            new_root_staking_epoch_ledger = new_root.staking_epoch_ledger_hash().to_string(),
            new_root_next_epoch_ledger = new_root.next_epoch_ledger_hash().to_string(),
            new_root_snarked_ledger = new_root.snarked_ledger_hash().to_string(),
        );
        self.recreate_snarked_ledger(
            &root_snarked_ledger_updates,
            &needed_protocol_states,
            new_root.snarked_ledger_hash(),
        )
        .unwrap();

        self.snarked_ledgers.retain(|hash, _| {
            let keep = ledgers_to_keep.contains(hash);
            if !keep {
                openmina_core::debug!(openmina_core::log::system_time();
                    kind = "LedgerService::commit - snarked_ledgers.drop",
                    summary = format!("drop snarked ledger {hash}"));
            }
            keep
        });
        self.snarked_ledgers.extend(
            std::mem::take(&mut self.sync.snarked_ledgers)
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

        self.staged_ledgers
            .retain(|hash, _| ledgers_to_keep.contains(hash));
        self.staged_ledgers.extend(
            std::mem::take(&mut self.sync.staged_ledgers)
                .into_iter()
                .filter(|(hash, _)| ledgers_to_keep.contains(hash)),
        );

        for ledger_hash in [
            new_best_tip.staking_epoch_ledger_hash(),
            new_root.snarked_ledger_hash(),
            new_root.staged_ledger_hash(),
        ] {
            if let Some((mut mask, is_synced)) = self.mask(ledger_hash) {
                if !is_synced {
                    panic!("ledger mask expected to be synced: {ledger_hash}");
                }
                let calculated = merkle_root(&mut mask);
                assert_eq!(ledger_hash, &calculated, "ledger mask hash mismatch");
            } else {
                panic!("ledger mask is missing: {ledger_hash}");
            }
        }

        for (ledger_hash, snarked_ledger) in self.snarked_ledgers.iter_mut() {
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
        let Some(new_root_ledger) = self.staged_ledgers.get_mut(new_root.staged_ledger_hash())
        else {
            return Default::default();
        };

        // Make staged ledger mask new root.
        new_root_ledger.commit_and_reparent_to_root();

        let needed_protocol_states = self
            .staged_ledger_mut(new_root.staged_ledger_hash())
            .map(|l| {
                l.scan_state()
                    .required_state_hashes()
                    .into_iter()
                    .map(|fp| DataHashLibStateHashStableV1(fp.into()).into())
                    .collect()
            })
            .unwrap_or_default();

        let available_jobs = self
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

    pub fn get_num_accounts(
        &mut self,
        ledger_hash: v2::LedgerHash,
    ) -> Option<(u64, v2::LedgerHash)> {
        let (mask, _) = self
            .mask(&ledger_hash)
            .filter(|(_, is_synced)| *is_synced)?;
        // fix(binier): incorrect ledger hash, must be a hash of a populated subtree.

        let num_accounts = mask.num_accounts() as u64;
        let first_node_addr = ledger::Address::first(
            LEDGER_DEPTH - super::tree_height_for_num_accounts(num_accounts),
        );
        let hash = LedgerHash::from_fp(mask.get_hash(first_node_addr)?);
        Some((num_accounts, hash))
    }

    pub fn get_child_hashes(
        &mut self,
        ledger_hash: v2::LedgerHash,
        addr: LedgerAddress,
    ) -> Option<(v2::LedgerHash, v2::LedgerHash)> {
        let (mask, is_synced) = self.mask(&ledger_hash)?;
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
        Some((get_hash(left)?, get_hash(right)?))
    }

    pub fn get_child_accounts(
        &mut self,
        ledger_hash: v2::LedgerHash,
        addr: LedgerAddress,
    ) -> Option<Vec<v2::MinaBaseAccountBinableArgStableV2>> {
        let (mask, _) = self
            .mask(&ledger_hash)
            .filter(|(_, is_synced)| *is_synced)?;
        // TODO(binier): SEC maybe we need to check addr depth?
        let accounts = mask
            .get_all_accounts_rooted_at(addr)?
            .into_iter()
            .map(|(_, account)| (&*account).into())
            .collect();
        Some(accounts)
    }

    pub fn get_accounts(
        &mut self,
        ledger_hash: v2::LedgerHash,
        ids: Vec<AccountId>,
    ) -> Vec<Account> {
        let Some((mask, _)) = self.mask(&ledger_hash) else {
            openmina_core::warn!(
                openmina_core::log::system_time();
                kind = "LedgerService::get_accounts",
                summary = format!("Ledger not found: {ledger_hash:?}")
            );
            return Vec::new();
        };
        let addrs = mask
            .location_of_account_batch(&ids)
            .into_iter()
            .filter_map(|(_id, addr)| addr)
            .collect::<Vec<_>>();

        mask.get_batch(&addrs)
            .into_iter()
            .filter_map(|(_, account)| account.map(|account| *account))
            .collect::<Vec<_>>()
    }

    pub fn staged_ledger_aux_and_pending_coinbase(
        &mut self,
        ledger_hash: LedgerHash,
        protocol_states: BTreeMap<StateHash, MinaStateProtocolStateValueStableV2>,
    ) -> Option<Arc<StagedLedgerAuxAndPendingCoinbases>> {
        let ledger = self.staged_ledger_mut(&ledger_hash)?;
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

    #[allow(clippy::too_many_arguments)]
    pub fn staged_ledger_diff_create(
        &mut self,
        pred_block: ArcBlockWithHash,
        global_slot_since_genesis: v2::MinaNumbersGlobalSlotSinceGenesisMStableV1,
        producer: NonZeroCurvePoint,
        delegator: NonZeroCurvePoint,
        coinbase_receiver: NonZeroCurvePoint,
        completed_snarks: BTreeMap<SnarkJobId, Snark>,
        supercharge_coinbase: bool,
        transactions_by_fee: Vec<valid::UserCommand>,
    ) -> Result<StagedLedgerDiffCreateOutput, String> {
        let mut staged_ledger = self
            .staged_ledger_mut(pred_block.staged_ledger_hash())
            .ok_or_else(|| {
                format!(
                    "parent staged ledger missing: {}",
                    pred_block.staged_ledger_hash()
                )
            })?
            .clone();

        // calculate merkle root hash, otherwise `MinaBasePendingCoinbaseStableV2::from` fails.
        staged_ledger.hash();
        let pending_coinbase_witness =
            MinaBasePendingCoinbaseStableV2::from(staged_ledger.pending_coinbase_collection());

        let protocol_state_view = protocol_state_view(&pred_block.header().protocol_state);

        // TODO(binier): include `invalid_txns` in output.
        let (pre_diff, _invalid_txns) = staged_ledger
            .create_diff(
                constraint_constants(),
                (&global_slot_since_genesis).into(),
                Some(true),
                (&coinbase_receiver).into(),
                (),
                &protocol_state_view,
                transactions_by_fee,
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
                constraint_constants(),
                (&global_slot_since_genesis).into(),
                pre_diff,
                (),
                &protocol_state_view,
                (pred_block.hash().0.to_field(), pred_body_hash.0.to_field()),
                (&coinbase_receiver).into(),
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
            stake_proof_sparse_ledger: self.stake_proof_sparse_ledger(
                pred_block.staking_epoch_ledger_hash(),
                &producer,
                &delegator,
            ),
        })
    }

    pub fn stake_proof_sparse_ledger(
        &mut self,
        staking_ledger: &LedgerHash,
        producer: &NonZeroCurvePoint,
        delegator: &NonZeroCurvePoint,
    ) -> v2::MinaBaseSparseLedgerBaseStableV2 {
        let mask = self.snarked_ledgers.get(staking_ledger).unwrap();
        let producer_id = ledger::AccountId::new(producer.into(), ledger::TokenId::default());
        let delegator_id = ledger::AccountId::new(delegator.into(), ledger::TokenId::default());
        let sparse_ledger = ledger::sparse_ledger::SparseLedger::of_ledger_subset_exn(
            mask.clone(),
            &[producer_id, delegator_id],
        );
        (&sparse_ledger).into()
    }

    pub fn scan_state_summary(
        &self,
        staged_ledger_hash: LedgerHash,
    ) -> Vec<Vec<RpcScanStateSummaryScanStateJob>> {
        use ledger::scan_state::scan_state::JobValue;

        let ledger = self.staged_ledgers.get(&staged_ledger_hash);
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
                            job: Box::new(job_kind),
                            seq_no,
                            snark: Box::new(RpcSnarkPoolJobSnarkWorkDone {
                                snarker: sok_message.prover,
                                fee: sok_message.fee,
                            }),
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

impl LedgerSyncState {
    fn mask(&self, hash: &LedgerHash) -> Option<(Mask, bool)> {
        self.snarked_ledgers
            .get(hash)
            .cloned()
            .map(|mask| (mask, false))
            .or_else(|| Some((self.staged_ledgers.get(hash)?.ledger(), true)))
    }

    fn pending_sync_snarked_ledger_mask(&self, hash: &LedgerHash) -> Result<Mask, String> {
        self.snarked_ledgers
            .get(hash)
            .cloned()
            .ok_or_else(|| format!("Missing sync snarked ledger {}", hash))
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
        self.staged_ledgers.get_mut(hash)
    }
}

fn staged_ledger_reconstruct(
    snarked_ledger: Mask,
    snarked_ledger_hash: LedgerHash,
    parts: Option<Arc<StagedLedgerAuxAndPendingCoinbasesValid>>,
) -> (v2::LedgerHash, Result<StagedLedger, String>) {
    let staged_ledger_hash = parts
        .as_ref()
        .map(|p| p.staged_ledger_hash.clone())
        .unwrap_or_else(|| snarked_ledger_hash.clone());

    let result = if let Some(parts) = parts {
        let states = parts
            .needed_blocks
            .iter()
            .map(|state| (state.hash().to_fp().unwrap(), state.clone()))
            .collect::<BTreeMap<_, _>>();

        StagedLedger::of_scan_state_pending_coinbases_and_snarked_ledger(
            (),
            constraint_constants(),
            Verifier,
            (&parts.scan_state).into(),
            snarked_ledger,
            LocalState::empty(),
            parts.staged_ledger_hash.0.to_field(),
            (&parts.pending_coinbase).into(),
            |key| states.get(&key).cloned().unwrap(),
        )
    } else {
        StagedLedger::create_exn(constraint_constants().clone(), snarked_ledger)
    };

    (staged_ledger_hash, result)
}

pub trait LedgerService: redux::Service {
    fn ledger_manager(&self) -> &LedgerManager;
    fn force_sync_calls(&self) -> bool {
        false
    }

    fn write_init(&mut self, request: LedgerWriteRequest) {
        let request = LedgerRequest::Write(request);
        if self.force_sync_calls() {
            let _ = self.ledger_manager().call_sync(request);
        } else {
            self.ledger_manager().call(request);
        }
    }

    fn read_init(&mut self, id: LedgerReadId, request: LedgerReadRequest) {
        let request = LedgerRequest::Read(id, request);
        if self.force_sync_calls() {
            let _ = self.ledger_manager().call_sync(request);
        } else {
            self.ledger_manager().call(request);
        }
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
    use mina_p2p_messages::binprot::{
        self,
        macros::{BinProtRead, BinProtWrite},
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

    use crate::ledger::hash_node_at_depth;

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
            let hash = hash_node_at_depth(address.length(), left.0.to_field(), right.0.to_field());
            let hash: LedgerHash = MinaBaseLedgerHash0StableV1(hash.into()).into();
            assert_eq!(hash.to_string(), expected_hash);
        });
    }
}
