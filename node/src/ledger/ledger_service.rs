use super::{
    ledger_empty_hash_at_depth,
    read::{LedgerReadId, LedgerReadRequest, LedgerReadResponse},
    write::{CommitResult, LedgerWriteRequest, LedgerWriteResponse, LedgersToKeep},
    LedgerAddress, LedgerEvent, LEDGER_DEPTH,
};
use crate::{
    account::AccountPublicKey,
    block_producer_effectful::StagedLedgerDiffCreateOutput,
    ledger::{
        ledger_manager::{LedgerManager, LedgerRequest},
        write::{BlockApplyResult, BlockApplyResultArchive},
    },
    p2p::channels::rpc::StagedLedgerAuxAndPendingCoinbases,
    rpc::{
        RpcScanStateSummaryBlockTransaction, RpcScanStateSummaryScanStateJob,
        RpcScanStateSummaryScanStateJobKind, RpcSnarkPoolJobSnarkWorkDone,
    },
    transition_frontier::{
        genesis::empty_pending_coinbase_hash,
        sync::{
            ledger::staged::StagedLedgerAuxAndPendingCoinbasesValid,
            TransitionFrontierRootSnarkedLedgerUpdates,
        },
    },
};
use ark_ff::fields::arithmetic::InvalidBigInt;
use ledger::{
    scan_state::{
        currency::Slot,
        scan_state::{AvailableJobMessage, JobValueBase, JobValueMerge, JobValueWithIndex, Pass},
        transaction_logic::{
            local_state::LocalState,
            protocol_state::{protocol_state_view, ProtocolStateView},
            transaction_partially_applied::TransactionPartiallyApplied,
            valid,
            zkapp_command::AccessedOrNot,
            Transaction, TransactionStatus, UserCommand,
        },
    },
    sparse_ledger::SparseLedger,
    staged_ledger::{
        diff::Diff,
        staged_ledger::{SkipVerification, StagedLedger},
        validate_block::block_body_hash,
    },
    verifier::Verifier,
    Account, AccountId, AccountIndex, BaseLedger, Database, Mask, TokenId, UnregisterBehavior,
};
use mina_hasher::Fp;
use mina_p2p_messages::{
    binprot::BinProtRead,
    list::List,
    v2::{
        self, DataHashLibStateHashStableV1, LedgerHash, MinaBaseLedgerHash0StableV1,
        MinaBasePendingCoinbaseStableV2, MinaBasePendingCoinbaseWitnessStableV2,
        MinaBaseSokMessageStableV1, MinaBaseStagedLedgerHashStableV1,
        MinaStateBlockchainStateValueStableV2LedgerProofStatement,
        MinaStateProtocolStateValueStableV2, MinaTransactionTransactionStableV2, NonZeroCurvePoint,
        StateHash,
    },
};
use mina_signer::CompressedPubKey;
use openmina_core::{
    block::{AppliedBlock, ArcBlockWithHash},
    bug_condition,
    constants::constraint_constants,
    snark::{Snark, SnarkJobId},
    thread,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::Arc,
};

fn merkle_root(mask: &mut Mask) -> LedgerHash {
    MinaBaseLedgerHash0StableV1(mask.merkle_root().into()).into()
}

fn error_to_string(e: InvalidBigInt) -> String {
    format!("{:?}", e)
}

/// Indexing `StagedLedger` both by their "merkle root hash" and their "staged ledger hash"
#[derive(Default)]
struct StagedLedgersStorage {
    /// 1 merkle root hash can refer to 1 or more `StagedLedger`
    by_merkle_root_hash: BTreeMap<LedgerHash, Vec<Arc<MinaBaseStagedLedgerHashStableV1>>>,
    staged_ledgers: BTreeMap<Arc<MinaBaseStagedLedgerHashStableV1>, StagedLedger>,
}

impl StagedLedgersStorage {
    /// Slow, it will recompute the full staged ledger hash
    /// Prefer `Self::insert` when you have the "staged ledger hash" around
    fn insert_by_recomputing_hash(&mut self, mut staged_ledger: StagedLedger) {
        let staged_ledger_hash: MinaBaseStagedLedgerHashStableV1 = (&staged_ledger.hash()).into();
        self.insert(Arc::new(staged_ledger_hash), staged_ledger);
    }

    fn insert(
        &mut self,
        staged_ledger_hash: Arc<MinaBaseStagedLedgerHashStableV1>,
        staged_ledger: StagedLedger,
    ) {
        let merkle_root_hash: LedgerHash = merkle_root(&mut staged_ledger.ledger());
        self.by_merkle_root_hash
            .entry(merkle_root_hash.clone())
            .or_insert_with(|| Vec::with_capacity(1))
            .extend([staged_ledger_hash.clone()]);
        self.staged_ledgers
            .insert(staged_ledger_hash, staged_ledger);
    }

    fn get_mask(&self, root_hash: &LedgerHash) -> Option<Mask> {
        let staged_ledger_hashes: &Vec<_> = self.by_merkle_root_hash.get(root_hash)?;
        // Note: there can be multiple `staged_ledger_hashes`, but they all have the same
        // mask, so we just take the 1st one
        self.staged_ledgers
            .get(staged_ledger_hashes.first()?)
            .map(|staged_ledger| staged_ledger.ledger())
    }

    fn get(&self, staged_ledger_hash: &MinaBaseStagedLedgerHashStableV1) -> Option<&StagedLedger> {
        self.staged_ledgers.get(staged_ledger_hash)
    }

    fn get_mut(
        &mut self,
        staged_ledger_hash: &MinaBaseStagedLedgerHashStableV1,
    ) -> Option<&mut StagedLedger> {
        self.staged_ledgers.get_mut(staged_ledger_hash)
    }

    fn retain<F>(&mut self, fun: F)
    where
        F: Fn(&MinaBaseStagedLedgerHashStableV1) -> bool,
    {
        self.by_merkle_root_hash.retain(|_, staged_ledger_hashes| {
            staged_ledger_hashes.retain(|hash| {
                if fun(hash) {
                    return true;
                }
                self.staged_ledgers.remove(hash);
                false
            });
            !staged_ledger_hashes.is_empty()
        });
    }

    fn extend<I>(&mut self, iterator: I)
    where
        I: IntoIterator<Item = (Arc<MinaBaseStagedLedgerHashStableV1>, StagedLedger)>,
    {
        for (hash, staged_ledger) in iterator.into_iter() {
            self.insert(hash, staged_ledger);
        }
    }

    fn take(&mut self) -> BTreeMap<Arc<MinaBaseStagedLedgerHashStableV1>, StagedLedger> {
        let Self {
            by_merkle_root_hash,
            staged_ledgers,
        } = self;

        let _ = std::mem::take(by_merkle_root_hash);
        std::mem::take(staged_ledgers)
    }
}

#[derive(Default)]
pub struct LedgerCtx {
    snarked_ledgers: BTreeMap<LedgerHash, Mask>,
    /// Additional snarked ledgers specified at startup (loaded from disk)
    additional_snarked_ledgers: BTreeMap<LedgerHash, Mask>,
    staged_ledgers: StagedLedgersStorage,
    sync: LedgerSyncState,
    /// Returns more data on block application necessary for archive node
    archive_mode: bool,
    event_sender:
        Option<openmina_core::channels::mpsc::UnboundedSender<crate::event_source::Event>>,
}

#[derive(Default)]
struct LedgerSyncState {
    snarked_ledgers: BTreeMap<LedgerHash, Mask>,
    staged_ledgers: StagedLedgersStorage,
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

    pub fn set_archive_mode(&mut self) {
        self.archive_mode = true;
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
        let merkle_root_hash = merkle_root(&mut mask);
        let staged_ledger =
            StagedLedger::create_exn(constraint_constants().clone(), mask.copy()).unwrap();
        self.snarked_ledgers.insert(merkle_root_hash.clone(), mask);
        // The genesis ledger is a specific case, some of its hashes are zero
        let staged_ledger_hash =
            MinaBaseStagedLedgerHashStableV1::zero(merkle_root_hash, empty_pending_coinbase_hash());
        self.staged_ledgers
            .insert(Arc::new(staged_ledger_hash), staged_ledger);
    }

    pub fn staged_ledger_reconstruct_result_store(&mut self, ledger: StagedLedger) {
        self.staged_ledgers.insert_by_recomputing_hash(ledger);
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
            .or_else(|| Some((self.staged_ledgers.get_mask(hash)?, true)))
            .or_else(|| self.sync.mask(hash))
            .or_else(|| {
                self.additional_snarked_ledgers
                    .get(hash)
                    .map(|l| (l.clone(), true))
            })
    }

    pub fn contains_snarked_ledger(&self, hash: &LedgerHash) -> bool {
        self.snarked_ledgers.contains_key(hash)
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
    fn staged_ledger_mut(
        &mut self,
        hash: &MinaBaseStagedLedgerHashStableV1,
    ) -> Option<&mut StagedLedger> {
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
        new_root_staged_ledger_hash: &MinaBaseStagedLedgerHashStableV1,
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
                    "Failed to find staged ledger with hash: {:#?}",
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

    pub fn get_account_delegators(
        &self,
        ledger_hash: &LedgerHash,
        account_id: &AccountId,
    ) -> Option<Vec<Account>> {
        let (mask, _) = self.mask(ledger_hash)?;
        let mut accounts = Vec::new();

        mask.iter(|account| {
            if account.delegate == Some(account_id.public_key.clone()) {
                accounts.push(account.clone());
            }
        });

        Some(accounts)
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
            if filter(account.delegate.as_ref().unwrap_or(&account.public_key)) {
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
            .map(|account| Ok(Box::new((&account).try_into()?)))
            .collect::<Result<Vec<_>, InvalidBigInt>>()
            .map_err(error_to_string)?;

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
    ) -> Result<(), InvalidBigInt>
    where
        F: 'static + FnOnce(v2::LedgerHash, Result<StagedLedger, String>) + Send,
    {
        let snarked_ledger = self
            .sync
            .snarked_ledger_mut(snarked_ledger_hash.clone())?
            .copy();

        thread::Builder::new()
            .name("staged-ledger-reconstruct".into())
            .spawn(move || {
                match staged_ledger_reconstruct(snarked_ledger, snarked_ledger_hash, parts) {
                    Ok((staged_ledger_hash, result)) => {
                        callback(staged_ledger_hash, result);
                    }
                    Err(e) => callback(v2::LedgerHash::zero(), Err(format!("{:?}", e))),
                }
            })
            .expect("Failed: staged ledger reconstruct thread");

        Ok(())
    }

    pub fn staged_ledger_reconstruct_sync(
        &mut self,
        snarked_ledger_hash: LedgerHash,
        parts: Option<Arc<StagedLedgerAuxAndPendingCoinbasesValid>>,
    ) -> Result<(v2::LedgerHash, Result<(), String>), InvalidBigInt> {
        let snarked_ledger = self
            .sync
            .snarked_ledger_mut(snarked_ledger_hash.clone())?
            .copy();
        let (staged_ledger_hash, result) =
            staged_ledger_reconstruct(snarked_ledger, snarked_ledger_hash, parts)?;
        let result = match result {
            Err(err) => Err(err),
            Ok(staged_ledger) => {
                self.staged_ledger_reconstruct_result_store(staged_ledger);
                Ok(())
            }
        };

        Ok((staged_ledger_hash, result))
    }

    pub fn block_apply(
        &mut self,
        block: ArcBlockWithHash,
        pred_block: AppliedBlock,
        skip_verification: Option<SkipVerification>,
    ) -> Result<BlockApplyResult, String> {
        openmina_core::info!(openmina_core::log::system_time();
            kind = "LedgerService::block_apply",
            summary = format!("{}, {} <- {}", block.height(), block.hash(), block.pred_hash()),
            pred_staged_ledger_hash = pred_block.merkle_root_hash().to_string(),
            staged_ledger_hash = block.merkle_root_hash().to_string(),
        );
        let mut staged_ledger = self
            .staged_ledger_mut(pred_block.staged_ledger_hashes())
            .ok_or_else(|| {
                format!(
                    "parent staged ledger missing: {:#?}",
                    pred_block.staged_ledger_hashes()
                )
            })?
            .clone();

        let global_slot = block.global_slot_since_genesis();
        let prev_protocol_state = &pred_block.header().protocol_state;
        let prev_state_view = protocol_state_view(prev_protocol_state).map_err(error_to_string)?;

        let consensus_state = &block.header().protocol_state.body.consensus_state;
        let coinbase_receiver: CompressedPubKey = (&consensus_state.coinbase_receiver)
            .try_into()
            .map_err(error_to_string)?;
        let supercharge_coinbase = consensus_state.supercharge_coinbase;

        let diff: Diff = (&block.body().staged_ledger_diff)
            .try_into()
            .map_err(error_to_string)?;

        let prev_protocol_state: ledger::proofs::block::ProtocolState =
            prev_protocol_state.try_into()?;

        let result = staged_ledger
            .apply(
                skip_verification,
                constraint_constants(),
                Slot::from_u32(global_slot),
                diff,
                (),
                &Verifier,
                &prev_state_view,
                prev_protocol_state.hashes(),
                coinbase_receiver.clone(),
                supercharge_coinbase,
            )
            .map_err(|err| format!("{err:?}"))?;
        let just_emitted_a_proof = result.ledger_proof.is_some();
        let ledger_hashes = MinaBaseStagedLedgerHashStableV1::from(&result.hash_after_applying);

        // TODO(binier): return error if not matching.
        let expected_ledger_hashes = block.staged_ledger_hashes();
        if &ledger_hashes != expected_ledger_hashes {
            let staged_ledger = self
                .staged_ledger_mut(pred_block.staged_ledger_hashes())
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

        let archive_data = if self.archive_mode {
            let senders = block
                .body()
                .transactions()
                .filter_map(|tx| UserCommand::try_from(tx).ok().map(|cmd| cmd.fee_payer()))
                .collect::<BTreeSet<_>>()
                .into_iter();

            let coinbase_receiver_id = AccountId::new(coinbase_receiver, TokenId::default());

            // https://github.com/MinaProtocol/mina/blob/85149735ca3a76d026e8cf36b8ff22941a048e31/src/app/archive/lib/diff.ml#L78
            let (accessed, not_accessed): (BTreeSet<_>, BTreeSet<_>) = block
                .body()
                .tranasctions_with_status()
                .flat_map(|(tx, status)| {
                    let status: TransactionStatus = status.into();
                    UserCommand::try_from(tx)
                        .ok()
                        .map(|cmd| cmd.account_access_statuses(&status))
                        .into_iter()
                        .flatten()
                })
                .partition(|(_, status)| *status == AccessedOrNot::Accessed);

            let mut account_ids_accessed: BTreeSet<_> =
                accessed.into_iter().map(|(id, _)| id).collect();
            let mut account_ids_not_accessed: BTreeSet<_> =
                not_accessed.into_iter().map(|(id, _)| id).collect();

            // Coinbase receiver is included only when the block has a coinbase transaction
            // Note: If for whatever reason the network has set the coinbase amount to zero,
            // to mimic the behavior of the ocaml node, we still include the coinbase receiver
            // in the accessed accounts as a coinbase transaction is created regardless of the coinbase amount.
            // https://github.com/MinaProtocol/mina/blob/b595a2bf00ae138d745737da628bd94bb2bd91e2/src/lib/staged_ledger/pre_diff_info.ml#L139
            let has_coinbase = block.body().has_coinbase();

            if has_coinbase {
                account_ids_accessed.insert(coinbase_receiver_id);
            } else {
                account_ids_not_accessed.insert(coinbase_receiver_id);
            }

            // Include the coinbase fee transfer accounts
            let fee_transfer_accounts =
                block.body().coinbase_fee_transfers_iter().filter_map(|cb| {
                    let receiver: CompressedPubKey = cb.receiver_pk.inner().try_into().ok()?;
                    let account_id = AccountId::new(receiver, TokenId::default());
                    Some(account_id)
                });
            account_ids_accessed.extend(fee_transfer_accounts);

            // TODO(adonagy): Create a struct instead of tuple
            let accounts_accessed: Vec<(AccountIndex, Account)> = account_ids_accessed
                .iter()
                .filter_map(|id| {
                    staged_ledger
                        .ledger()
                        .index_of_account(id.clone())
                        .and_then(|index| {
                            staged_ledger
                                .ledger()
                                .get_at_index(index)
                                .map(|account| (index, *account))
                        })
                })
                .collect();

            let account_creation_fee = constraint_constants().account_creation_fee;

            // TODO(adonagy): Create a struct instead of tuple
            let accounts_created: Vec<(AccountId, u64)> = staged_ledger
                .latest_block_accounts_created(pred_block.hash().to_field()?)
                .iter()
                .map(|id| (id.clone(), account_creation_fee))
                .collect();

            // A token is used regardless of txn status
            // https://github.com/MinaProtocol/mina/blob/85149735ca3a76d026e8cf36b8ff22941a048e31/src/app/archive/lib/diff.ml#L114
            let all_account_ids: BTreeSet<_> = account_ids_accessed
                .union(&account_ids_not_accessed)
                .collect();
            let tokens_used: BTreeSet<(TokenId, Option<AccountId>)> = if has_coinbase {
                all_account_ids
                    .iter()
                    .map(|id| {
                        let token_id = id.token_id.clone();
                        let token_owner = staged_ledger.ledger().token_owner(token_id.clone());
                        (token_id, token_owner)
                    })
                    .collect()
            } else {
                BTreeSet::new()
            };

            let sender_receipt_chains_from_parent_ledger = senders
                .filter_map(|sender| {
                    if let Some(location) = staged_ledger.ledger().location_of_account(&sender) {
                        staged_ledger.ledger().get(location).map(|account| {
                            (
                                sender,
                                v2::ReceiptChainHash::from(account.receipt_chain_hash),
                            )
                        })
                    } else {
                        None
                    }
                })
                .collect();
            Some(BlockApplyResultArchive {
                accounts_accessed,
                accounts_created,
                tokens_used,
                sender_receipt_chains_from_parent_ledger,
            })
        } else {
            None
        };

        self.sync
            .staged_ledgers
            .insert(Arc::new(ledger_hashes), staged_ledger);

        // staged_ledger.ledger().get_at_index(index)

        Ok(BlockApplyResult {
            block,
            just_emitted_a_proof,
            archive_data,
        })
    }

    pub fn commit(
        &mut self,
        ledgers_to_keep: LedgersToKeep,
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
            .retain(|hash| ledgers_to_keep.contains(hash));
        self.staged_ledgers.extend(
            self.sync
                .staged_ledgers
                .take()
                .into_iter()
                .filter(|(hash, _)| ledgers_to_keep.contains(&**hash)),
        );

        for ledger_hash in [
            new_best_tip.staking_epoch_ledger_hash(),
            new_root.snarked_ledger_hash(),
            new_root.merkle_root_hash(),
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
        let Some(new_root_ledger) = self.staged_ledgers.get_mut(new_root.staged_ledger_hashes())
        else {
            return Default::default();
        };

        // Make staged ledger mask new root.
        new_root_ledger.commit_and_reparent_to_root();

        let needed_protocol_states = self
            .staged_ledger_mut(new_root.staged_ledger_hashes())
            .map(|l| {
                l.scan_state()
                    .required_state_hashes()
                    .into_iter()
                    .map(|fp| DataHashLibStateHashStableV1(fp.into()).into())
                    .collect()
            })
            .unwrap_or_default();

        let available_jobs = Arc::new(
            self.staged_ledger_mut(new_best_tip.staged_ledger_hashes())
                .map(|l| {
                    l.scan_state()
                        .all_job_pairs_iter()
                        .map(|job| job.map(|single| AvailableJobMessage::from(single)))
                        .collect()
                })
                .unwrap_or_default(),
        );

        // self.check_alive_masks();

        CommitResult {
            alive_masks: ::ledger::mask::alive_len(),
            available_jobs,
            needed_protocol_states,
        }
    }

    #[allow(dead_code)]
    fn check_alive_masks(&mut self) {
        let mut alive: BTreeSet<_> = ::ledger::mask::alive_collect();
        let staged_ledgers = self
            .staged_ledgers
            .staged_ledgers
            .iter()
            .map(|(hash, ledger)| (&hash.non_snark.ledger_hash, ledger.ledger_ref()));

        let alive_ledgers = self
            .snarked_ledgers
            .iter()
            .chain(staged_ledgers)
            .map(|(hash, mask)| {
                let uuid = mask.get_uuid();
                if !alive.remove(&uuid) {
                    bug_condition!("mask not found among alive masks! uuid: {uuid}, hash: {hash}");
                }
                (uuid, hash)
            })
            .collect::<Vec<_>>();
        openmina_core::debug!(redux::Timestamp::global_now(); "alive_ledgers_after_commit: {alive_ledgers:#?}");

        if !alive.is_empty() {
            bug_condition!(
                "masks alive which are no longer part of the ledger service: {alive:#?}"
            );
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
            LEDGER_DEPTH.saturating_sub(super::tree_height_for_num_accounts(num_accounts)),
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
        ledger_hash: &MinaBaseStagedLedgerHashStableV1,
        protocol_states: BTreeMap<StateHash, MinaStateProtocolStateValueStableV2>,
    ) -> Option<Arc<StagedLedgerAuxAndPendingCoinbases>> {
        let ledger = self.staged_ledger_mut(ledger_hash)?;
        let needed_blocks = ledger
            .scan_state()
            .required_state_hashes()
            .into_iter()
            .map(|fp| DataHashLibStateHashStableV1(fp.into()))
            .map(|hash| protocol_states.get(&hash.into()).ok_or(()).cloned())
            .collect::<Result<_, _>>()
            .ok()?;
        // Required so that we can perform the conversion bellow, which
        // will not work if the hash is not available already.
        ledger.pending_coinbase_collection_merkle_root();
        Some(
            StagedLedgerAuxAndPendingCoinbases {
                scan_state: (ledger.scan_state()).into(),
                staged_ledger_hash: ledger_hash.non_snark.ledger_hash.clone(),
                pending_coinbase: (ledger.pending_coinbase_collection()).into(),
                needed_blocks,
            }
            .into(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn staged_ledger_diff_create(
        &mut self,
        pred_block: AppliedBlock,
        global_slot_since_genesis: v2::MinaNumbersGlobalSlotSinceGenesisMStableV1,
        is_new_epoch: bool,
        producer: NonZeroCurvePoint,
        delegator: NonZeroCurvePoint,
        coinbase_receiver: NonZeroCurvePoint,
        completed_snarks: BTreeMap<SnarkJobId, Snark>,
        supercharge_coinbase: bool,
        transactions_by_fee: Vec<valid::UserCommand>,
    ) -> Result<StagedLedgerDiffCreateOutput, String> {
        let mut staged_ledger = self
            .staged_ledger_mut(pred_block.staged_ledger_hashes())
            .ok_or_else(|| {
                format!(
                    "parent staged ledger missing: {}",
                    pred_block.merkle_root_hash()
                )
            })?
            .clone();

        // calculate merkle root hash, otherwise `MinaBasePendingCoinbaseStableV2::from` fails.
        staged_ledger.hash();
        let pending_coinbase_witness =
            MinaBasePendingCoinbaseStableV2::from(staged_ledger.pending_coinbase_collection());

        let protocol_state_view =
            protocol_state_view(&pred_block.header().protocol_state).map_err(error_to_string)?;

        // TODO(binier): include `invalid_txns` in output.
        let (pre_diff, _invalid_txns) = staged_ledger
            .create_diff(
                constraint_constants(),
                (&global_slot_since_genesis).into(),
                Some(true),
                (&coinbase_receiver).try_into().map_err(error_to_string)?,
                (),
                &protocol_state_view,
                transactions_by_fee,
                |stmt| {
                    let job_id = SnarkJobId::from(stmt);
                    match completed_snarks.get(&job_id) {
                        Some(snark) => snark.try_into().ok(),
                        None => None,
                    }
                },
                supercharge_coinbase,
            )
            .map_err(|err| format!("{err:?}"))?;

        // TODO(binier): maybe here, check if block reward is above threshold.
        // https://github.com/minaprotocol/mina/blob/b3d418a8c0ae4370738886c2b26f0ec7bdb49303/src/lib/block_producer/block_producer.ml#L222

        let pred_body_hash = pred_block
            .header()
            .protocol_state
            .body
            .try_hash()
            .map_err(error_to_string)?;
        let diff = (&pre_diff).into();

        let res = staged_ledger
            .apply_diff_unchecked(
                constraint_constants(),
                (&global_slot_since_genesis).into(),
                pre_diff,
                (),
                &protocol_state_view,
                (
                    pred_block.hash().0.to_field().map_err(error_to_string)?,
                    pred_body_hash.0.to_field().map_err(error_to_string)?,
                ),
                (&coinbase_receiver).try_into().map_err(error_to_string)?,
                supercharge_coinbase,
            )
            .map_err(|err| format!("{err:?}"))?;

        let diff_hash = block_body_hash(&diff).map_err(|err| format!("{err:?}"))?;
        let staking_ledger_hash = if is_new_epoch {
            pred_block.next_epoch_ledger_hash()
        } else {
            pred_block.staking_epoch_ledger_hash()
        };

        Ok(StagedLedgerDiffCreateOutput {
            diff,
            diff_hash,
            staged_ledger_hash: (&res.hash_after_applying).into(),
            emitted_ledger_proof: res
                .ledger_proof
                .map(|(proof, ..)| (&proof).into())
                .map(Arc::new),
            pending_coinbase_update: (&res.pending_coinbase_update.1).into(),
            pending_coinbase_witness: MinaBasePendingCoinbaseWitnessStableV2 {
                pending_coinbases: pending_coinbase_witness,
                is_new_stack: res.pending_coinbase_update.0,
            },
            stake_proof_sparse_ledger: self
                .stake_proof_sparse_ledger(staking_ledger_hash, &producer, &delegator)
                .map_err(error_to_string)?,
        })
    }

    pub fn stake_proof_sparse_ledger(
        &mut self,
        staking_ledger: &LedgerHash,
        producer: &NonZeroCurvePoint,
        delegator: &NonZeroCurvePoint,
    ) -> Result<v2::MinaBaseSparseLedgerBaseStableV2, InvalidBigInt> {
        let mask = self.snarked_ledgers.get(staking_ledger).unwrap();
        let producer_id = ledger::AccountId::new(producer.try_into()?, ledger::TokenId::default());
        let delegator_id =
            ledger::AccountId::new(delegator.try_into()?, ledger::TokenId::default());
        let sparse_ledger = ledger::sparse_ledger::SparseLedger::of_ledger_subset_exn(
            mask.clone(),
            &[producer_id, delegator_id],
        );
        Ok((&sparse_ledger).into())
    }

    pub fn scan_state_summary(
        &self,
        staged_ledger_hash: &MinaBaseStagedLedgerHashStableV1,
    ) -> Result<Vec<Vec<RpcScanStateSummaryScanStateJob>>, String> {
        use ledger::scan_state::scan_state::JobValue;

        let ledger = self.staged_ledgers.get(staged_ledger_hash);
        let Some(ledger) = ledger else {
            return Ok(Vec::new());
        };
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

                    if is_done {
                        let is_left =
                            bundle.map_or_else(|| true, |(_, is_sibling_left)| !is_sibling_left);
                        let parent = job.parent().ok_or_else(|| format!("job(depth: {}, index: {}) has no parent", job.depth(), job.index()))?;
                        let sok_message: MinaBaseSokMessageStableV1 = {
                                let job = jobs.get(parent).ok_or_else(|| format!("job(depth: {}, index: {}) parent not found", job.depth(), job.index()))?;
                                match &job.job {
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
                                    _state => {
                                        // Parent of a `Done` job can't be in this state.
                                        // But we are bug-compatible with the OCaml node here, in which sometimes for
                                        // some reason there is an empty row in the scan state trees, so Empty
                                        // is used instead.
                                        res.push(RpcScanStateSummaryScanStateJob::Empty);
                                        continue;
                                    }
                                }
                        };
                        res.push(RpcScanStateSummaryScanStateJob::Done {
                            job_id,
                            bundle_job_id,
                            job: Box::new(job_kind),
                            seq_no,
                            snark: Box::new(RpcSnarkPoolJobSnarkWorkDone {
                                snarker: sok_message.prover,
                                fee: sok_message.fee,
                            }),
                        });
                    } else {
                        res.push(RpcScanStateSummaryScanStateJob::Todo {
                            job_id,
                            bundle_job_id,
                            job: job_kind,
                            seq_no,
                        });
                    }
                }
                Ok(res)
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
            .or_else(|| Some((self.staged_ledgers.get_mask(hash)?, true)))
    }

    fn pending_sync_snarked_ledger_mask(&self, hash: &LedgerHash) -> Result<Mask, String> {
        self.snarked_ledgers
            .get(hash)
            .cloned()
            .ok_or_else(|| format!("Missing sync snarked ledger {}", hash))
    }

    /// Returns a [Mask] instance for the snarked ledger with [hash]. If it doesn't
    /// exist a new instance is created.
    fn snarked_ledger_mut(&mut self, hash: LedgerHash) -> Result<&mut Mask, InvalidBigInt> {
        let hash_fp = hash.to_field()?;
        Ok(self.snarked_ledgers.entry(hash.clone()).or_insert_with(|| {
            let mut ledger = Mask::create(LEDGER_DEPTH);
            ledger.set_cached_hash_unchecked(&LedgerAddress::root(), hash_fp);
            ledger
        }))
    }

    fn staged_ledger_mut(
        &mut self,
        hash: &MinaBaseStagedLedgerHashStableV1,
    ) -> Option<&mut StagedLedger> {
        self.staged_ledgers.get_mut(hash)
    }
}

fn staged_ledger_reconstruct(
    snarked_ledger: Mask,
    snarked_ledger_hash: LedgerHash,
    parts: Option<Arc<StagedLedgerAuxAndPendingCoinbasesValid>>,
) -> Result<(v2::LedgerHash, Result<StagedLedger, String>), InvalidBigInt> {
    let staged_ledger_hash = parts
        .as_ref()
        .map(|p| p.staged_ledger_hash.clone())
        .unwrap_or_else(|| snarked_ledger_hash.clone());

    let ledger = snarked_ledger.make_child();

    let mut result = if let Some(parts) = &parts {
        let states = parts
            .needed_blocks
            .iter()
            .map(|state| Ok((state.try_hash()?.to_field()?, state.clone())))
            .collect::<Result<BTreeMap<Fp, _>, _>>()?;

        StagedLedger::of_scan_state_pending_coinbases_and_snarked_ledger(
            (),
            constraint_constants(),
            Verifier,
            (&parts.scan_state).try_into()?,
            ledger,
            LocalState::empty(),
            parts.staged_ledger_hash.to_field()?,
            (&parts.pending_coinbase).try_into()?,
            |key| states.get(&key).cloned().unwrap(),
        )
    } else {
        StagedLedger::create_exn(constraint_constants().clone(), ledger)
    };

    match result.as_mut() {
        Ok(staged_ledger) => {
            staged_ledger.commit_and_reparent_to_root();
        }
        Err(_) => {
            if let Err(e) = dump_reconstruct_to_file(&snarked_ledger, &parts) {
                openmina_core::error!(
                    openmina_core::log::system_time();
                    kind = "LedgerService::dump - Failed reconstruct",
                    summary = format!("Failed to save reconstruction to file: {e:?}")
                );
            }
        }
    }

    Ok((staged_ledger_hash, result))
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

/// Save reconstruction to file, when it fails.
/// So we can easily reproduce the application both in Rust and OCaml, to compare them.
fn dump_reconstruct_to_file(
    snarked_ledger: &Mask,
    parts: &Option<Arc<StagedLedgerAuxAndPendingCoinbasesValid>>,
) -> std::io::Result<()> {
    use mina_p2p_messages::binprot::{
        self,
        macros::{BinProtRead, BinProtWrite},
    };

    #[derive(BinProtRead, BinProtWrite)]
    struct ReconstructContext {
        accounts: Vec<v2::MinaBaseAccountBinableArgStableV2>,
        scan_state: v2::TransactionSnarkScanStateStableV2,
        pending_coinbase: v2::MinaBasePendingCoinbaseStableV2,
        staged_ledger_hash: LedgerHash,
        states: List<v2::MinaStateProtocolStateValueStableV2>,
    }

    let Some(parts) = parts else {
        return Err(std::io::ErrorKind::Other.into());
    };

    let StagedLedgerAuxAndPendingCoinbasesValid {
        scan_state,
        staged_ledger_hash,
        pending_coinbase,
        needed_blocks,
    } = &**parts;

    let reconstruct_context = ReconstructContext {
        accounts: snarked_ledger
            .to_list()
            .iter()
            .map(v2::MinaBaseAccountBinableArgStableV2::from)
            .collect(),
        scan_state: scan_state.clone(),
        pending_coinbase: pending_coinbase.clone(),
        staged_ledger_hash: staged_ledger_hash.clone(),
        states: needed_blocks.clone(),
    };

    let debug_dir = openmina_core::get_debug_dir();
    let filename = debug_dir
        .join("failed_reconstruct_ctx.binprot")
        .to_string_lossy()
        .to_string();
    std::fs::create_dir_all(&debug_dir)?;

    use mina_p2p_messages::binprot::BinProtWrite;
    let mut file = std::fs::File::create(&filename)?;
    reconstruct_context.binprot_write(&mut file)?;
    file.sync_all()?;

    openmina_core::info!(
        openmina_core::log::system_time();
        kind = "LedgerService::dump - Failed reconstruct",
        summary = format!("Reconstruction saved to: {filename:?}")
    );

    Ok(())
}

/// Save staged ledger and block to file, when the application fail.
/// So we can easily reproduce the application both in Rust and OCaml, to compare them.
/// - https://github.com/openmina/openmina/blob/8e68037aafddd43842a54c8439baeafee4c6e1eb/ledger/src/staged_ledger/staged_ledger.rs#L5959
/// - TODO: Find OCaml link, I remember having the same test in OCaml but I can't find where
fn dump_application_to_file(
    staged_ledger: &StagedLedger,
    block: ArcBlockWithHash,
    pred_block: AppliedBlock,
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
        pred_block: (**pred_block.block()).clone(),
        blocks: vec![(*block.block).clone()],
    };

    let debug_dir = openmina_core::get_debug_dir();
    let filename = debug_dir
        .join(format!("failed_application_ctx_{}.binprot", block_height))
        .to_string_lossy()
        .to_string();
    std::fs::create_dir_all(&debug_dir)?;

    let mut file = std::fs::File::create(&filename)?;

    use mina_p2p_messages::binprot::BinProtWrite;
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
            let hash = hash_node_at_depth(
                address.length(),
                left.0.to_field().unwrap(),
                right.0.to_field().unwrap(),
            );
            let hash: LedgerHash = MinaBaseLedgerHash0StableV1(hash.into()).into();
            assert_eq!(hash.to_string(), expected_hash);
        });
    }
}
