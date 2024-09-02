use std::sync::Arc;

use mina_hasher::Fp;
use mina_p2p_messages::v2::MinaStateProtocolStateValueStableV2;
use mina_signer::CompressedPubKey;
use openmina_core::constants::ConstraintConstants;

use crate::{
    decompress_pk,
    scan_state::{
        self,
        currency::{Amount, Magnitude, Slot},
        fee_excess::FeeExcess,
        pending_coinbase::{
            update::{Action, StackUpdate, Update},
            PendingCoinbase, Stack, StackState,
        },
        scan_state::{
            transaction_snark::{
                work, InitStack, LedgerHash, LedgerProof, LedgerProofWithSokMessage, OneOrTwo,
                Registers, SokMessage, Statement, TransactionWithWitness,
            },
            AvailableJob, Pass, ScanState, SpacePartition, StatementCheck, TransactionsOrdered,
        },
        snark_work::spec,
        transaction_logic::{
            apply_transaction_first_pass, apply_transaction_second_pass, local_state::LocalState,
            protocol_state::ProtocolStateView,
            transaction_partially_applied::TransactionPartiallyApplied, valid,
            zkapp_command::MaybeWithStatus, CoinbaseFeeTransfer, Transaction, TransactionStatus,
            UserCommand, WithStatus,
        },
    },
    sparse_ledger::SparseLedger,
    split_at, split_at_vec,
    staged_ledger::{pre_diff_info, resources::IncreaseBy, transaction_validator},
    verifier::{Verifier, VerifierError},
    zkapps::non_snark::LedgerNonSnark,
    AccountId, BaseLedger, Mask, TokenId,
};

use super::{
    diff::{with_valid_signatures_and_proofs, AtMostOne, AtMostTwo, Diff, PreDiffTwo},
    diff_creation_log::{DiffCreationLog, Partition},
    hash::StagedLedgerHash,
    pre_diff_info::PreDiffError,
    resources::Resources,
};

/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#470
#[derive(Clone, Debug)]
pub struct StackStateWithInitStack {
    pub pc: StackState,
    pub init_stack: Stack,
}

/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L23
#[derive(Debug, derive_more::From)]
pub enum StagedLedgerError {
    NonZeroFeeExcess(Vec<WithStatus<Transaction>>, Box<SpacePartition>),
    InvalidProofs(Vec<(LedgerProof, Statement<()>, SokMessage)>, String),
    CouldntReachVerifier,
    #[from]
    PreDiff(PreDiffError),
    InsufficientWork(String),
    MismatchedStatuses {
        transaction: Box<WithStatus<Transaction>>,
        got: Box<TransactionStatus>,
    },
    InvalidPublicKey(Box<CompressedPubKey>),
    ZkAppsExceedLimit {
        count: usize,
        limit: usize,
    },
    #[from]
    Unexpected(String),
}

const ZKAPP_LIMIT_PER_BLOCK: Option<usize> = None;

pub struct PreStatement<L: LedgerNonSnark> {
    partially_applied_transaction: TransactionPartiallyApplied<L>,
    expected_status: TransactionStatus,
    accounts_accessed: Vec<AccountId>,
    fee_excess: FeeExcess,
    first_pass_ledger_witness: SparseLedger,
    first_pass_ledger_source_hash: LedgerHash,
    first_pass_ledger_target_hash: LedgerHash,
    pending_coinbase_stack_source: Stack,
    pending_coinbase_stack_target: Stack,
    init_stack: InitStack,
}

#[derive(Debug)]
pub struct DiffResult {
    pub hash_after_applying: StagedLedgerHash<Fp>,
    pub ledger_proof: Option<(
        LedgerProof,
        Vec<TransactionsOrdered<(WithStatus<Transaction>, Fp, Slot)>>,
    )>,
    pub pending_coinbase_update: (bool, Update),
}

#[derive(Clone, Copy, Debug)]
pub enum SkipVerification {
    All,
    Proofs,
}

#[derive(Clone)]
pub struct StagedLedger {
    scan_state: ScanState,
    ledger: Mask,
    constraint_constants: ConstraintConstants,
    pending_coinbase_collection: PendingCoinbase,
}

impl StagedLedger {
    pub fn proof_txns_with_state_hashes(
        &self,
    ) -> Option<Vec<TransactionsOrdered<(WithStatus<Transaction>, Fp, Slot)>>> {
        self.scan_state.latest_ledger_proof().map(|(_, list)| list)
    }

    pub fn scan_state(&self) -> &ScanState {
        &self.scan_state
    }

    fn all_work_pairs<F>(&self, get_state: F) -> Result<Vec<OneOrTwo<spec::Work>>, String>
    where
        F: Fn(&Fp) -> &MinaStateProtocolStateValueStableV2,
    {
        self.scan_state.all_work_pairs(get_state)
    }

    fn all_work_statements_exn(&self) -> Vec<work::Statement> {
        self.scan_state.all_work_statements_exn()
    }

    pub fn pending_coinbase_collection_merkle_root(&mut self) -> Fp {
        self.pending_coinbase_collection.merkle_root()
    }

    pub fn pending_coinbase_collection(&self) -> &PendingCoinbase {
        &self.pending_coinbase_collection
    }

    fn verify_scan_state_after_apply(
        constraint_constants: &ConstraintConstants,
        pending_coinbase_stack: Stack,
        first_pass_ledger_end: LedgerHash,
        second_pass_ledger_end: LedgerHash,
        scan_state: &ScanState,
    ) -> Result<(), String> {
        let registers_end = Registers {
            pending_coinbase_stack,
            local_state: LocalState::empty(),
            first_pass_ledger: first_pass_ledger_end,
            second_pass_ledger: second_pass_ledger_end,
        };
        let statement_check =
            StatementCheck::<fn(Fp) -> MinaStateProtocolStateValueStableV2>::Partial;
        let last_proof_statement = scan_state
            .latest_ledger_proof()
            .map(|(proof_with_sok, _)| proof_with_sok.proof.statement());

        scan_state.check_invariants(
            constraint_constants,
            statement_check,
            &Verifier,
            "Error verifying the parallel scan state after applying the diff.",
            last_proof_statement,
            registers_end,
        )
    }

    fn of_scan_state_and_ledger<F>(
        _logger: (),
        constraint_constants: &ConstraintConstants,
        verifier: Verifier,
        last_proof_statement: Option<Statement<()>>,
        mut ledger: Mask,
        scan_state: ScanState,
        pending_coinbase_collection: PendingCoinbase,
        get_state: F,
        first_pass_ledger_target: LedgerHash,
    ) -> Result<Self, String>
    where
        F: Fn(Fp) -> MinaStateProtocolStateValueStableV2,
    {
        let pending_coinbase_stack = pending_coinbase_collection.latest_stack(false);

        scan_state.check_invariants(
            constraint_constants,
            StatementCheck::Full(Box::new(get_state)),
            &verifier,
            "Staged_ledger.of_scan_state_and_ledger",
            last_proof_statement,
            Registers {
                pending_coinbase_stack,
                local_state: LocalState::empty(),
                first_pass_ledger: first_pass_ledger_target,
                second_pass_ledger: ledger.merkle_root(),
            },
        )?;

        Ok(Self {
            scan_state,
            ledger,
            constraint_constants: constraint_constants.clone(),
            pending_coinbase_collection,
        })
    }

    /// https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/staged_ledger/staged_ledger.ml#L325
    fn of_scan_state_and_ledger_unchecked(
        constraint_constants: &ConstraintConstants,
        last_proof_statement: Option<Statement<()>>,
        mut ledger: Mask,
        scan_state: ScanState,
        pending_coinbase_collection: PendingCoinbase,
        first_pass_ledger_target: LedgerHash,
    ) -> Result<Self, String> {
        let pending_coinbase_stack = pending_coinbase_collection.latest_stack(false);

        scan_state.check_invariants(
            constraint_constants,
            StatementCheck::<fn(Fp) -> MinaStateProtocolStateValueStableV2>::Partial,
            &Verifier, // null
            "Staged_ledger.of_scan_state_and_ledger",
            last_proof_statement,
            Registers {
                pending_coinbase_stack,
                local_state: LocalState::empty(),
                first_pass_ledger: first_pass_ledger_target,
                second_pass_ledger: ledger.merkle_root(),
            },
        )?;

        Ok(Self {
            scan_state,
            ledger,
            constraint_constants: constraint_constants.clone(),
            pending_coinbase_collection,
        })
    }

    /// https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/staged_ledger/staged_ledger.ml#L353
    fn of_scan_state_pending_coinbases_and_snarked_ledger_prime<F, G>(
        constraint_constants: &ConstraintConstants,
        pending_coinbase: PendingCoinbase,
        scan_state: ScanState,
        mut snarked_ledger: Mask,
        _snarked_local_state: LocalState,
        expected_merkle_root: LedgerHash,
        get_state: F,
        fun: G,
    ) -> Result<Self, String>
    where
        F: Fn(Fp) -> MinaStateProtocolStateValueStableV2,
        G: FnOnce(
            &ConstraintConstants,
            Option<Statement<()>>,
            Mask,
            ScanState,
            PendingCoinbase,
            LedgerHash,
        ) -> Result<Self, String>,
    {
        let apply_first_pass = |global_slot: Slot,
                                txn_state_view: &ProtocolStateView,
                                ledger: &mut Mask,
                                transaction: &Transaction| {
            apply_transaction_first_pass(
                constraint_constants,
                global_slot,
                txn_state_view,
                ledger,
                transaction,
            )
        };

        let apply_second_pass = |ledger: &mut Mask, tx: TransactionPartiallyApplied<Mask>| {
            apply_transaction_second_pass(constraint_constants, ledger, tx)
        };

        let apply_first_pass_sparse_ledger =
            |global_slot: Slot,
             txn_state_view: &ProtocolStateView,
             sparse_ledger: &mut SparseLedger,
             transaction: &Transaction| {
                apply_transaction_first_pass(
                    constraint_constants,
                    global_slot,
                    txn_state_view,
                    sparse_ledger,
                    transaction,
                )
            };

        let Pass::FirstPassLedgerHash(first_pass_ledger_target) = scan_state
            .get_staged_ledger_sync(
                &mut snarked_ledger,
                |hash| Ok(get_state(hash)),
                apply_first_pass,
                apply_second_pass,
                apply_first_pass_sparse_ledger,
            )?;

        let staged_ledger_hash = snarked_ledger.merkle_root();
        if staged_ledger_hash != expected_merkle_root {
            return Err(format!(
                "Mismatching merkle root Expected:{:?} Got:{:?}",
                expected_merkle_root, staged_ledger_hash
            ));
        }

        let last_proof_statement = scan_state
            .latest_ledger_proof()
            .map(|(proof_with_sok, _)| proof_with_sok.proof.statement());

        fun(
            constraint_constants,
            last_proof_statement,
            snarked_ledger,
            scan_state,
            pending_coinbase,
            first_pass_ledger_target,
        )
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L378
    pub fn of_scan_state_pending_coinbases_and_snarked_ledger<F>(
        logger: (),
        constraint_constants: &ConstraintConstants,
        verifier: Verifier,
        scan_state: ScanState,
        snarked_ledger: Mask,
        snarked_local_state: LocalState,
        expected_merkle_root: LedgerHash,
        pending_coinbase: PendingCoinbase,
        get_state: F,
    ) -> Result<Self, String>
    where
        F: Fn(Fp) -> MinaStateProtocolStateValueStableV2,
    {
        Self::of_scan_state_pending_coinbases_and_snarked_ledger_prime(
            constraint_constants,
            pending_coinbase,
            scan_state,
            snarked_ledger,
            snarked_local_state,
            expected_merkle_root,
            &get_state,
            |constraint_constants,
             last_proof_statement,
             ledger,
             scan_state,
             pending_coinbase_collection,
             first_pass_ledger_target| {
                Self::of_scan_state_and_ledger(
                    logger,
                    constraint_constants,
                    verifier,
                    last_proof_statement,
                    ledger,
                    scan_state,
                    pending_coinbase_collection,
                    &get_state,
                    first_pass_ledger_target,
                )
            },
        )
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L386
    fn of_scan_state_pending_coinbases_and_snarked_ledger_unchecked<F>(
        constraint_constants: &ConstraintConstants,
        scan_state: ScanState,
        snarked_ledger: Mask,
        snarked_local_state: LocalState,
        expected_merkle_root: LedgerHash,
        pending_coinbase: PendingCoinbase,
        get_state: F,
    ) -> Result<Self, String>
    where
        F: Fn(Fp) -> MinaStateProtocolStateValueStableV2,
    {
        Self::of_scan_state_pending_coinbases_and_snarked_ledger_prime(
            constraint_constants,
            pending_coinbase,
            scan_state,
            snarked_ledger,
            snarked_local_state,
            expected_merkle_root,
            get_state,
            Self::of_scan_state_and_ledger_unchecked,
        )
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L393
    pub fn copy(&self) -> Self {
        let new_mask = self.ledger.make_child();

        Self {
            scan_state: self.scan_state.clone(), // TODO: Not sure if OCaml keeps the same pointer
            ledger: new_mask,
            constraint_constants: self.constraint_constants.clone(),
            pending_coinbase_collection: self.pending_coinbase_collection.clone(), // TODO: Not sure if OCaml keeps the same pointer
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#403
    pub fn hash(&mut self) -> StagedLedgerHash<Fp> {
        StagedLedgerHash::of_aux_ledger_and_coinbase_hash(
            self.scan_state.hash(),
            self.ledger.merkle_root(),
            &mut self.pending_coinbase_collection,
        )
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#422
    pub fn ledger(&self) -> Mask {
        self.ledger.clone()
    }

    /// commit all the masks from current mask all the way upto the root
    /// while also detaching all intermediary masks. Sets current mask to root.
    pub fn commit_and_reparent_to_root(&mut self) {
        if let Some(new_mask) = self.ledger.commit_and_reparent_to_root() {
            self.ledger = new_mask;
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#424
    pub fn create_exn(
        constraint_constants: ConstraintConstants,
        ledger: Mask,
    ) -> Result<Self, String> {
        let pending_coinbase_depth = constraint_constants.pending_coinbase_depth;

        Ok(Self {
            scan_state: ScanState::empty(&constraint_constants),
            ledger,
            constraint_constants,
            pending_coinbase_collection: PendingCoinbase::create(pending_coinbase_depth),
        })
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#434
    fn current_ledger_proof(&self) -> Option<&LedgerProofWithSokMessage> {
        self.scan_state.latest_ledger_proof().map(|(f, _)| f)
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#439
    fn replace_ledger_exn(&mut self, mut ledger: Mask) {
        assert_eq!(
            ledger.merkle_root(),
            self.ledger.merkle_root(),
            "Cannot replace ledger since merkle_root differs"
        );
        self.ledger = ledger;
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#456
    fn working_stack(
        pending_coinbase_collection: &PendingCoinbase,
        is_new_stack: bool,
    ) -> Result<Stack, String> {
        Ok(pending_coinbase_collection.latest_stack(is_new_stack))
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#460
    fn push_coinbase(current_stack: &Stack, transaction: &Transaction) -> Stack {
        match transaction {
            Transaction::Coinbase(c) => current_stack.push_coinbase(c.clone()),
            _ => current_stack.clone(),
        }
    }

    fn push_state(current_stack: Stack, state_body_hash: Fp, global_slot: Slot) -> Stack {
        current_stack.push_state(state_body_hash, global_slot)
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#477
    pub fn coinbase_amount(
        supercharge_coinbase: bool,
        constraint_constants: &ConstraintConstants,
    ) -> Option<Amount> {
        let coinbase_amount = Amount::from_u64(constraint_constants.coinbase_amount);
        if supercharge_coinbase {
            coinbase_amount.scale(constraint_constants.supercharged_coinbase_factor)
        } else {
            Some(coinbase_amount)
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/staged_ledger/staged_ledger.ml#L518
    pub fn apply_single_transaction_first_pass(
        constraint_constants: &ConstraintConstants,
        global_slot: Slot,
        mut ledger: Mask,
        pending_coinbase_stack_state: &StackStateWithInitStack,
        txn_with_status: &WithStatus<Transaction>,
        txn_state_view: &ProtocolStateView,
    ) -> Result<(PreStatement<Mask>, StackStateWithInitStack), StagedLedgerError> {
        let txn = &txn_with_status.data;
        let expected_status = txn_with_status.status.clone();

        // TODO(OCaml): for zkapps, we should actually narrow this by segments
        let accounts_accessed = txn.accounts_referenced();

        let fee_excess = txn.fee_excess()?;
        let source_ledger_hash = ledger.merkle_root();
        let ledger_witness = SparseLedger::of_ledger_subset_exn(ledger.clone(), &accounts_accessed);

        let pending_coinbase_target =
            Self::push_coinbase(&pending_coinbase_stack_state.pc.target, txn);
        let new_init_stack = Self::push_coinbase(&pending_coinbase_stack_state.init_stack, txn);

        let partially_applied_transaction = apply_transaction_first_pass(
            constraint_constants,
            global_slot,
            txn_state_view,
            &mut ledger,
            txn,
        )?;

        let target_ledger_hash = ledger.merkle_root();

        Ok((
            PreStatement {
                partially_applied_transaction,
                expected_status,
                accounts_accessed,
                fee_excess,
                first_pass_ledger_witness: ledger_witness,
                first_pass_ledger_source_hash: source_ledger_hash,
                first_pass_ledger_target_hash: target_ledger_hash,
                pending_coinbase_stack_source: pending_coinbase_stack_state.pc.source.clone(),
                pending_coinbase_stack_target: pending_coinbase_target.clone(),
                init_stack: InitStack::Base(pending_coinbase_stack_state.init_stack.clone()),
            },
            StackStateWithInitStack {
                pc: StackState {
                    source: pending_coinbase_target.clone(),
                    target: pending_coinbase_target,
                },
                init_stack: new_init_stack,
            },
        ))
    }

    pub fn apply_single_transaction_second_pass(
        constraint_constants: &ConstraintConstants,
        connecting_ledger: LedgerHash,
        mut ledger: Mask,
        state_and_body_hash: (Fp, Fp),
        global_slot: Slot,
        pre_stmt: PreStatement<Mask>,
    ) -> Result<TransactionWithWitness, StagedLedgerError> {
        let empty_local_state = LocalState::empty();
        let second_pass_ledger_source_hash = ledger.merkle_root();
        let ledger_witness =
            SparseLedger::of_ledger_subset_exn(ledger.clone(), &pre_stmt.accounts_accessed);
        let applied_txn = apply_transaction_second_pass(
            constraint_constants,
            &mut ledger,
            pre_stmt.partially_applied_transaction,
        )?;

        let second_pass_ledger_target_hash = ledger.merkle_root();
        let supply_increase = applied_txn.supply_increase(constraint_constants)?;

        let actual_status = applied_txn.transaction_status();

        if actual_status != &pre_stmt.expected_status {
            let txn_with_expected_status = WithStatus {
                data: applied_txn.transaction().data,
                status: pre_stmt.expected_status,
            };
            return Err(StagedLedgerError::MismatchedStatuses {
                transaction: Box::new(txn_with_expected_status),
                got: Box::new(actual_status.clone()),
            });
        }

        let statement = Statement {
            source: Registers {
                first_pass_ledger: pre_stmt.first_pass_ledger_source_hash,
                second_pass_ledger: second_pass_ledger_source_hash,
                pending_coinbase_stack: pre_stmt.pending_coinbase_stack_source,
                local_state: empty_local_state.clone(),
            },
            target: Registers {
                first_pass_ledger: pre_stmt.first_pass_ledger_target_hash,
                second_pass_ledger: second_pass_ledger_target_hash,
                pending_coinbase_stack: pre_stmt.pending_coinbase_stack_target,
                local_state: empty_local_state,
            },
            connecting_ledger_left: connecting_ledger,
            connecting_ledger_right: connecting_ledger,
            supply_increase,
            fee_excess: pre_stmt.fee_excess,
            sok_digest: (),
        };

        Ok(TransactionWithWitness {
            transaction_with_info: applied_txn,
            state_hash: state_and_body_hash,
            statement,
            init_stack: pre_stmt.init_stack,
            first_pass_ledger_witness: pre_stmt.first_pass_ledger_witness,
            second_pass_ledger_witness: ledger_witness,
            block_global_slot: global_slot,
        })
    }

    fn apply_transactions_first_pass(
        constraint_constants: &ConstraintConstants,
        global_slot: Slot,
        ledger: Mask,
        init_pending_coinbase_stack_state: StackStateWithInitStack,
        ts: Vec<WithStatus<Transaction>>,
        current_state_view: &ProtocolStateView,
    ) -> Result<(Vec<PreStatement<Mask>>, Stack), StagedLedgerError> {
        let apply = |pending_coinbase_stack_state: &StackStateWithInitStack,
                     txn: &WithStatus<Transaction>| {
            if let Some(pk) = txn
                .data
                .public_keys()
                .iter()
                .find(|pk| decompress_pk(pk).is_none())
            {
                return Err(StagedLedgerError::InvalidPublicKey(Box::new(pk.clone())));
            }

            Self::apply_single_transaction_first_pass(
                constraint_constants,
                global_slot,
                ledger.clone(),
                pending_coinbase_stack_state,
                txn,
                current_state_view,
            )
        };

        let mut pending_coinbase_stack_state = init_pending_coinbase_stack_state;

        let tx_with_witness = ts
            .iter()
            .map(|transaction| {
                let (tx_with_witness, new_stack_state) =
                    { apply(&pending_coinbase_stack_state, transaction)? };

                pending_coinbase_stack_state = new_stack_state;

                Ok(tx_with_witness)
            })
            .collect::<Result<_, StagedLedgerError>>()?;

        Ok((tx_with_witness, pending_coinbase_stack_state.pc.target))
    }

    fn apply_transactions_second_pass(
        constraint_constants: &ConstraintConstants,
        global_slot: Slot,
        ledger: &mut Mask,
        state_and_body_hash: (Fp, Fp),
        pre_stmts: Vec<PreStatement<Mask>>,
    ) -> Result<Vec<TransactionWithWitness>, StagedLedgerError> {
        let connecting_ledger = ledger.merkle_root();

        pre_stmts
            .into_iter()
            .map(|pre_stmt| {
                Self::apply_single_transaction_second_pass(
                    constraint_constants,
                    connecting_ledger,
                    ledger.clone(),
                    state_and_body_hash,
                    global_slot,
                    pre_stmt,
                )
            })
            .collect()
    }

    pub fn update_ledger_and_get_statements(
        constraint_constants: &ConstraintConstants,
        global_slot: Slot,
        mut ledger: Mask,
        current_stack: &Stack,
        tss: (
            Vec<WithStatus<Transaction>>,
            Option<Vec<WithStatus<Transaction>>>,
        ),
        current_state_view: &ProtocolStateView,
        state_and_body_hash: (Fp, Fp),
    ) -> Result<(Vec<TransactionWithWitness>, Stack, Stack, Fp), StagedLedgerError> {
        let (_, state_body_hash) = state_and_body_hash;
        let (ts, ts_opt) = tss;

        let apply_first_pass = |working_stack: &Stack, ts| {
            let working_stack_with_state =
                Self::push_state(working_stack.clone(), state_body_hash, global_slot);
            let init_pending_coinbase_stack_state = StackStateWithInitStack {
                pc: StackState {
                    source: working_stack.clone(),
                    target: working_stack_with_state,
                },
                init_stack: working_stack.clone(),
            };

            Self::apply_transactions_first_pass(
                constraint_constants,
                global_slot,
                ledger.clone(),
                init_pending_coinbase_stack_state,
                ts,
                current_state_view,
            )
        };

        let (pre_stmts1, updated_stack1) = apply_first_pass(current_stack, ts)?;

        let (pre_stmts2, updated_stack2) = match ts_opt {
            None => (vec![], updated_stack1.clone()),
            Some(ts) => {
                let current_stack2 = Stack::create_with(current_stack);
                apply_first_pass(&current_stack2, ts)?
            }
        };

        let first_pass_ledger_end = ledger.merkle_root();
        let txns_with_witnesses = Self::apply_transactions_second_pass(
            constraint_constants,
            global_slot,
            &mut ledger,
            state_and_body_hash,
            pre_stmts1.into_iter().chain(pre_stmts2).collect(),
        )?;

        Ok((
            txns_with_witnesses,
            updated_stack1,
            updated_stack2,
            first_pass_ledger_end,
        ))
    }

    fn verify_proofs(
        _logger: (),
        verifier: &Verifier,
        proofs: Vec<(LedgerProof, Statement<()>, SokMessage)>,
    ) -> Result<(), StagedLedgerError> {
        if proofs
            .iter()
            .any(|(proof, statement, _msg)| &proof.statement() != statement)
        {
            return Err(
                "Invalid transaction snark for statement: Statement and proof do not match"
                    .to_string()
                    .into(),
            );
        }

        match verifier.verify_transaction_snarks(
            proofs
                .into_iter()
                .map(|(proof, _, msg)| (proof, msg))
                .collect(),
        ) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!(
                "Verifier error when checking transaction snark for statement: {:?}",
                e
            )
            .into()),
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L164
    fn verify(
        logger: (),
        verifier: &Verifier,
        job_msg_proofs: Vec<(AvailableJob, SokMessage, LedgerProof)>,
    ) -> Result<(), StagedLedgerError> {
        fn map_opt<F, R>(xs: Vec<(AvailableJob, SokMessage, LedgerProof)>, fun: F) -> Option<Vec<R>>
        where
            F: Fn(AvailableJob, SokMessage, LedgerProof) -> Option<R>,
        {
            xs.into_iter()
                .map(|(job, msg, proof)| fun(job, msg, proof))
                .collect()
        }

        match map_opt(job_msg_proofs, |job, msg, proof| {
            ScanState::statement_of_job(&job).map(|s| (proof, s, msg))
        }) {
            None => Err("Error creating statement from job".to_string().into()),
            Some(proof_statement_msgs) => {
                Self::verify_proofs(logger, verifier, proof_statement_msgs)
            }
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L654
    fn check_completed_works(
        logger: (),
        verifier: &Verifier,
        scan_state: &ScanState,
        completed_works: Vec<work::Work>,
    ) -> Result<(), StagedLedgerError> {
        let work_count = completed_works.len() as u64;
        let jobs_pairs = scan_state.k_work_pairs_for_new_diff(work_count);

        let job_msg_proofs: Vec<(AvailableJob, SokMessage, LedgerProof)> = jobs_pairs
            .into_iter()
            .zip(completed_works)
            .flat_map(|(jobs, work)| {
                let message = SokMessage::create(work.fee, work.prover);
                OneOrTwo::zip(jobs, work.proofs)
                    .into_map(|(job, proof)| (job, message.clone(), proof))
                    .into_iter()
            })
            .collect();

        Self::verify(logger, verifier, job_msg_proofs)
    }

    /// The total fee excess caused by any diff should be zero. In the case where
    /// the slots are split into two partitions, total fee excess of the transactions
    /// to be enqueued on each of the partitions should be zero respectively
    ///
    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L674
    fn check_zero_fee_excess(
        scan_state: &ScanState,
        data: &[TransactionWithWitness],
    ) -> Result<(), StagedLedgerError> {
        let partitions = scan_state.partition_if_overflowing();

        let txns_from_data = |data: &[TransactionWithWitness]| -> Vec<WithStatus<Transaction>> {
            data.iter()
                .map(|tx| tx.transaction_with_info.transaction())
                .collect::<Vec<_>>()
        };

        let total_fee_excess = |txns: &[WithStatus<Transaction>]| {
            txns.iter().try_fold(FeeExcess::empty(), |accum, txn| {
                let fee_excess = txn.data.fee_excess()?;
                FeeExcess::combine(&accum, &fee_excess)
            })
        };

        let check = |data: &[TransactionWithWitness],
                     slots: &SpacePartition|
         -> Result<(), StagedLedgerError> {
            let txns = txns_from_data(data);
            let fee_excess = total_fee_excess(&txns)?;

            if fee_excess.is_zero() {
                Ok(())
            } else {
                Err(StagedLedgerError::NonZeroFeeExcess(
                    txns,
                    Box::new(slots.clone()),
                ))
            }
        };

        let (first, second) = split_at(data, partitions.first.0 as usize);

        check(first, &partitions)?;

        if partitions.second.is_some() {
            check(second, &partitions)?;
        };

        Ok(())
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L712
    fn update_coinbase_stack_and_get_data(
        constraint_constants: &ConstraintConstants,
        global_slot: Slot,
        scan_state: &ScanState,
        mut ledger: Mask,
        pending_coinbase_collection: &PendingCoinbase,
        transactions: Vec<WithStatus<Transaction>>,
        current_state_view: &ProtocolStateView,
        state_and_body_hash: (Fp, Fp),
    ) -> Result<(bool, Vec<TransactionWithWitness>, Action, StackUpdate, Pass), StagedLedgerError>
    {
        let coinbase_exists = |txns: &[WithStatus<Transaction>]| {
            txns.iter()
                .any(|tx| matches!(tx.data, Transaction::Coinbase(_)))
        };

        let SpacePartition {
            first: (slots, _),
            second,
        } = scan_state.partition_if_overflowing();

        if !transactions.is_empty() {
            if second.is_none() {
                // Single partition:
                // 1.Check if a new stack is required and get a working stack [working_stack]
                // 2.create data for enqueuing onto the scan state *)

                let is_new_stack = scan_state.next_on_new_tree();
                let working_stack = Self::working_stack(pending_coinbase_collection, is_new_stack)?;

                let (data, updated_stack, _, first_pass_ledger_end) =
                    Self::update_ledger_and_get_statements(
                        constraint_constants,
                        global_slot,
                        ledger,
                        &working_stack,
                        (transactions, None),
                        current_state_view,
                        state_and_body_hash,
                    )?;

                Ok((
                    is_new_stack,
                    data,
                    Action::One,
                    StackUpdate::One(updated_stack),
                    Pass::FirstPassLedgerHash(first_pass_ledger_end),
                ))
            } else {
                // Two partition:
                // Assumption: Only one of the partition will have coinbase transaction(s)in it.
                // 1. Get the latest stack for coinbase in the first set of transactions
                // 2. get the first set of scan_state data[data1]
                // 3. get a new stack for the second partion because the second set of transactions would start from the begining of the next tree in the scan_state
                // 4. Initialize the new stack with the state from the first stack
                // 5. get the second set of scan_state data[data2]*)

                let (txns_for_partition1, txns_for_partition2) =
                    split_at_vec(transactions, slots as usize);

                let txns_for_partition2_is_empty = txns_for_partition2.is_empty();

                let coinbase_in_first_partition = coinbase_exists(&txns_for_partition1);

                let working_stack1 = Self::working_stack(pending_coinbase_collection, false)?;

                let (data, updated_stack1, updated_stack2, first_pass_ledger_end) =
                    Self::update_ledger_and_get_statements(
                        constraint_constants,
                        global_slot,
                        ledger.clone(),
                        &working_stack1,
                        (txns_for_partition1, Some(txns_for_partition2)),
                        current_state_view,
                        state_and_body_hash,
                    )?;

                let second_has_data = !txns_for_partition2_is_empty;

                let (pending_coinbase_action, stack_update) =
                    // NOTE: Only branch `(true, false)` and `(false, true)` are taken here
                    match (coinbase_in_first_partition, second_has_data) {
                        (true, true) => {
                            (
                                Action::TwoCoinbaseInFirst,
                                StackUpdate::Two((updated_stack1, updated_stack2)),
                            )
                            // updated_stack2 does not have coinbase and but has the state from the previous stack
                        }
                        (true, false) => {
                            // updated_stack1 has some new coinbase but parition 2 has no
                            // data and so we have only one stack to update
                            (Action::One, StackUpdate::One(updated_stack1))
                        }
                        (false, true) => {
                            // updated_stack1 just has the new state. [updated stack2] might have coinbase,
                            // definitely has some data and therefore will have a non-dummy state.
                            (
                                Action::TwoCoinbaseInSecond,
                                StackUpdate::Two((updated_stack1, updated_stack2)),
                            )
                        }
                        (false, false) => {
                            // a diff consists of only non-coinbase transactions. This is
                            // currently not possible because a diff will have a coinbase
                            // at the very least, so don't update anything?*)
                            (Action::None, StackUpdate::None)
                        }
                    };

                Ok((
                    false,
                    data,
                    pending_coinbase_action,
                    stack_update,
                    Pass::FirstPassLedgerHash(first_pass_ledger_end),
                ))
            }
        } else {
            Ok((
                false,
                Vec::new(),
                Action::None,
                StackUpdate::None,
                Pass::FirstPassLedgerHash(ledger.merkle_root()),
            ))
        }
    }

    /// update the pending_coinbase tree with the updated/new stack and delete the oldest stack if a proof was emitted
    ///
    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L806
    fn update_pending_coinbase_collection<A>(
        depth: usize,
        pending_coinbase: &mut PendingCoinbase,
        stack_update: StackUpdate,
        is_new_stack: bool,
        ledger_proof: &Option<(LedgerProof, A)>,
    ) -> Result<(), StagedLedgerError> {
        // Deleting oldest stack if proof emitted
        if let Some((proof, _)) = ledger_proof {
            let oldest_stack = pending_coinbase.remove_coinbase_stack(depth)?;
            let ledger_proof_stack = &proof.statement().target.pending_coinbase_stack;

            if &oldest_stack != ledger_proof_stack {
                return Err("Pending coinbase stack of the ledger proof did not \
                     match the oldest stack in the pending coinbase tree."
                    .to_string())?;
            }
        };

        match stack_update {
            StackUpdate::None => {}
            StackUpdate::One(stack1) => {
                pending_coinbase.update_coinbase_stack(depth, stack1, is_new_stack)?;
            }
            StackUpdate::Two((stack1, stack2)) => {
                // The case when some of the transactions go into the old tree and
                // remaining on to the new tree
                pending_coinbase.update_coinbase_stack(depth, stack1, false)?;
                pending_coinbase.update_coinbase_stack(depth, stack2, true)?;
            }
        };

        Ok(())
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L855
    fn coinbase_for_blockchain_snark(amounts: &[Amount]) -> Result<Amount, StagedLedgerError> {
        match amounts {
            [] => Ok(Amount::zero()),
            [amount] => Ok(*amount),
            [amount1, _amount2] => Ok(*amount1),
            _ => Err(StagedLedgerError::PreDiff(PreDiffError::CoinbaseError(
                "More than two coinbase parts".to_string(),
            ))),
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L868
    fn apply_diff(
        &mut self,
        _logger: (),
        skip_verification: Option<bool>,
        pre_diff_info: (
            Vec<WithStatus<Transaction>>,
            Vec<work::Work>,
            usize,
            Vec<Amount>,
        ),
        constraint_constants: &ConstraintConstants,
        global_slot: Slot,
        current_state_view: &ProtocolStateView,
        state_and_body_hash: (Fp, Fp),
        log_prefix: &'static str,
    ) -> Result<DiffResult, StagedLedgerError> {
        let skip_verification = skip_verification.unwrap_or(false);

        let max_throughput = 2u64.pow(constraint_constants.transaction_capacity_log_2 as u32);

        let (_spots_available, _proofs_waiting) = {
            let jobs = self.scan_state.all_work_statements_exn();
            let free_space = self.scan_state.free_space();
            (free_space.min(max_throughput), jobs.len())
        };

        let mut new_ledger = self.ledger.make_child();

        let (transactions, works, _commands_count, coinbases) = pre_diff_info;

        if let Some(zkapp_limit) = ZKAPP_LIMIT_PER_BLOCK {
            let zkapp_count = transactions.iter().filter(|t| t.data.is_zkapp()).count();
            if zkapp_count > zkapp_limit {
                return Err(StagedLedgerError::ZkAppsExceedLimit {
                    count: zkapp_count,
                    limit: zkapp_limit,
                });
            }
        };

        let (
            is_new_stack,
            data,
            stack_update_in_snark,
            stack_update,
            Pass::FirstPassLedgerHash(first_pass_ledger_end),
        ) = Self::update_coinbase_stack_and_get_data(
            constraint_constants,
            global_slot,
            &self.scan_state,
            new_ledger.clone(),
            &self.pending_coinbase_collection,
            transactions,
            current_state_view,
            state_and_body_hash,
        )?;

        let slots = data.len();
        let work_count = works.len();
        let required_pairs = self.scan_state.work_statements_for_new_diff();

        {
            let required = required_pairs.len();
            if work_count < required
                && data.len() > (self.scan_state.free_space() as usize - required + work_count)
            {
                return Err(StagedLedgerError::InsufficientWork(format!(
                    "Insufficient number of transaction snark work (slots \
                     occupying: {})  required {}, got {}",
                    slots, required, work_count
                )));
            }
        }

        Self::check_zero_fee_excess(&self.scan_state, &data)?;

        let data_is_empty = data.is_empty();
        let data: Vec<_> = data.into_iter().map(Arc::new).collect();

        let res_opt = {
            self.scan_state
                .fill_work_and_enqueue_transactions(data, works)
                .map_err(|e| {
                    format!(
                        "{}: Unexpected error when applying diff data $data to \
                     the scan_state: {:?}",
                        log_prefix, e
                    )
                })?
            // TODO: OCaml print the error in json format here
        };

        Self::update_pending_coinbase_collection(
            constraint_constants.pending_coinbase_depth,
            &mut self.pending_coinbase_collection,
            stack_update,
            is_new_stack,
            &res_opt,
        )?;

        let coinbase_amount = Self::coinbase_for_blockchain_snark(&coinbases)?;

        let latest_pending_coinbase_stack = self.pending_coinbase_collection.latest_stack(false);

        if !(skip_verification || data_is_empty) {
            Self::verify_scan_state_after_apply(
                constraint_constants,
                latest_pending_coinbase_stack,
                first_pass_ledger_end,    // first_pass_ledger_end
                new_ledger.merkle_root(), // second_pass_ledger_end
                &self.scan_state,
            )?;
        }

        self.ledger = new_ledger;
        self.constraint_constants = constraint_constants.clone();

        Ok(DiffResult {
            hash_after_applying: self.hash(),
            ledger_proof: res_opt,
            pending_coinbase_update: (
                is_new_stack,
                Update {
                    action: stack_update_in_snark,
                    coinbase_amount,
                },
            ),
        })
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1016
    fn forget_prediff_info<B, C, D>(
        (a, b, c, d): (Vec<WithStatus<valid::Transaction>>, B, C, D),
    ) -> (Vec<WithStatus<Transaction>>, B, C, D) {
        (
            a.iter()
                .map(|with_status| with_status.map(|t| t.forget()))
                .collect(),
            b,
            c,
            d,
        )
    }

    /// https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/staged_ledger/staged_ledger.ml#L1089
    fn check_commands(
        ledger: Mask,
        verifier: &Verifier,
        cs: Vec<WithStatus<UserCommand>>,
        skip_verification: Option<SkipVerification>,
    ) -> Result<Vec<valid::UserCommand>, VerifierError> {
        use scan_state::transaction_logic::zkapp_command::from_applied_sequence::{
            self, FromAppliedSequence,
        };

        let cs = cs
            .into_iter()
            .map(MaybeWithStatus::from)
            .collect::<Vec<_>>();
        let cs = UserCommand::to_all_verifiable::<FromAppliedSequence, _>(cs, |account_ids| {
            let cache = UserCommand::load_vks_from_ledger(account_ids, &ledger);
            from_applied_sequence::Cache::new(cache)
        })
        .unwrap(); // TODO: No unwrap
        let cs = cs.into_iter().map(WithStatus::from).collect::<Vec<_>>();

        verifier
            .verify_commands(cs, skip_verification)
            .into_iter()
            .collect()
    }

    pub fn apply(
        &mut self,
        skip_verification: Option<SkipVerification>,
        constraint_constants: &ConstraintConstants,
        global_slot: Slot,
        witness: Diff,
        logger: (),
        verifier: &Verifier,
        current_state_view: &ProtocolStateView,
        state_and_body_hash: (Fp, Fp),
        coinbase_receiver: CompressedPubKey,
        supercharge_coinbase: bool,
    ) -> Result<DiffResult, StagedLedgerError> {
        let work = witness.completed_works();
        let works_count = work.len();

        let now = redux::Instant::now();
        if skip_verification.is_none() {
            Self::check_completed_works(logger, verifier, &self.scan_state, work)?;
        }
        eprintln!(
            "verification time={:?} ({works_count} completed works)",
            now.elapsed()
        );

        let prediff = witness.get(
            |cmd| Self::check_commands(self.ledger.clone(), verifier, cmd, skip_verification),
            constraint_constants,
            coinbase_receiver,
            supercharge_coinbase,
        )?;

        self.apply_diff(
            logger,
            skip_verification.map(|s| matches!(s, SkipVerification::All)),
            Self::forget_prediff_info(prediff),
            constraint_constants,
            global_slot,
            current_state_view,
            state_and_body_hash,
            "apply_diff",
        )
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1095
    pub fn apply_diff_unchecked(
        &mut self,
        constraint_constants: &ConstraintConstants,
        global_slot: Slot,
        sl_diff: with_valid_signatures_and_proofs::Diff,
        logger: (),
        current_state_view: &ProtocolStateView,
        state_and_body_hash: (Fp, Fp),
        coinbase_receiver: CompressedPubKey,
        supercharge_coinbase: bool,
    ) -> Result<DiffResult, StagedLedgerError> {
        let prediff = sl_diff.get_unchecked(
            constraint_constants,
            coinbase_receiver,
            supercharge_coinbase,
        )?;

        self.apply_diff(
            logger,
            None,
            Self::forget_prediff_info(prediff),
            constraint_constants,
            global_slot,
            current_state_view,
            state_and_body_hash,
            "apply_diff_unchecked",
        )
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1580
    fn check_constraints_and_update(
        constraint_constants: &ConstraintConstants,
        resources: &mut Resources,
        log: &mut DiffCreationLog,
    ) {
        use super::diff_creation_log::Reason::*;

        if resources.slots_occupied() == 0 {
            // Done
        } else if resources.work_constraint_satisfied() {
            // There's enough work. Check if they satisfy other constraints
            if resources.budget_sufficient() {
                if resources.space_constraint_satisfied() {
                    return;
                } else if resources.worked_more(constraint_constants) {
                    // There are too many fee_transfers(from the proofs)
                    // occupying the slots. discard one and check
                    let work_opt = resources.discard_last_work(constraint_constants);
                    if let Some(work) = work_opt {
                        log.discard_completed_work(ExtraWork, &work);
                    };
                    Self::check_constraints_and_update(constraint_constants, resources, log);
                } else {
                    // Well, there's no space; discard a user command
                    let uc_opt = resources.discard_user_command();
                    if let Some(uc) = uc_opt.as_ref() {
                        log.discard_command(NoSpace, uc);
                    };
                    Self::check_constraints_and_update(constraint_constants, resources, log);
                }
            } else {
                // insufficient budget; reduce the cost
                let work_opt = resources.discard_last_work(constraint_constants);
                if let Some(work) = work_opt {
                    log.discard_completed_work(InsufficientFees, &work);
                };
                Self::check_constraints_and_update(constraint_constants, resources, log);
            }
        } else {
            // There isn't enough work for the transactions. Discard a
            // transaction and check again
            let uc_opt = resources.discard_user_command();
            if let Some(uc) = uc_opt.as_ref() {
                log.discard_command(NoWork, uc);
            };
            Self::check_constraints_and_update(constraint_constants, resources, log);
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1624
    fn one_prediff(
        constraint_constants: &ConstraintConstants,
        cw_seq: Vec<work::Unchecked>,
        ts_seq: Vec<valid::UserCommand>,
        receiver: &CompressedPubKey,
        add_coinbase: bool,
        slot_job_count: (u64, u64),
        logger: (),
        is_coinbase_receiver_new: bool,
        partition: Partition,
        supercharge_coinbase: bool,
    ) -> (Resources, DiffCreationLog) {
        let mut init_resources = Resources::init(
            constraint_constants,
            ts_seq,
            cw_seq,
            slot_job_count,
            receiver.clone(),
            add_coinbase,
            supercharge_coinbase,
            logger,
            is_coinbase_receiver_new,
        );

        let mut log = DiffCreationLog::init(
            &init_resources.completed_work_rev,
            &init_resources.commands_rev,
            &init_resources.coinbase,
            partition,
            slot_job_count.0,
            slot_job_count.1,
        );

        Self::check_constraints_and_update(constraint_constants, &mut init_resources, &mut log);

        (init_resources, log)
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1643
    fn generate(
        constraint_constants: &ConstraintConstants,
        logger: (),
        cw_seq: Vec<work::Unchecked>,
        ts_seq: Vec<valid::UserCommand>,
        receiver: &CompressedPubKey,
        is_coinbase_receiver_new: bool,
        supercharge_coinbase: bool,
        partitions: scan_state::SpacePartition,
    ) -> (
        (
            PreDiffTwo<work::Work, valid::UserCommand>,
            Option<super::diff::PreDiffOne<work::Work, valid::UserCommand>>,
        ),
        Vec<DiffCreationLog>,
    ) {
        let pre_diff_with_one =
            |mut res: Resources| -> super::diff::PreDiffOne<work::Checked, valid::UserCommand> {
                let to_at_most_one = |two: AtMostTwo<CoinbaseFeeTransfer>| match two {
                    AtMostTwo::Zero => AtMostOne::Zero,
                    AtMostTwo::One(v) => AtMostOne::One(v),
                    AtMostTwo::Two(_) => {
                        eprintln!(
                            "Error creating staged ledger diff: Should have at most one \
                             coinbase in the second pre_diff"
                        );
                        AtMostOne::Zero
                    }
                };

                // We have to reverse here because we only know they work in THIS order
                super::diff::PreDiffOne::<work::Checked, valid::UserCommand> {
                    commands: {
                        res.commands_rev.reverse();
                        res.commands_rev
                    },
                    completed_works: {
                        res.completed_work_rev.reverse();
                        res.completed_work_rev
                    },
                    coinbase: to_at_most_one(res.coinbase),
                    internal_command_statuses: {
                        // updated later based on application result
                        Vec::new()
                    },
                }
            };

        let pre_diff_with_two =
            |mut res: Resources| -> super::diff::PreDiffTwo<work::Checked, valid::UserCommand> {
                super::diff::PreDiffTwo::<work::Checked, valid::UserCommand> {
                    commands: {
                        res.commands_rev.reverse();
                        res.commands_rev
                    },
                    completed_works: {
                        res.completed_work_rev.reverse();
                        res.completed_work_rev
                    },
                    coinbase: res.coinbase,
                    internal_command_statuses: {
                        // updated later based on application result
                        Vec::new()
                    },
                }
            };

        let end_log = |(res, log): &mut (Resources, DiffCreationLog)| {
            log.end_log(&res.completed_work_rev, &res.commands_rev, &res.coinbase)
        };

        let make_diff = |mut res1: (Resources, DiffCreationLog),
                         res2: Option<(Resources, DiffCreationLog)>|
         -> (
            (
                PreDiffTwo<work::Work, valid::UserCommand>,
                Option<super::diff::PreDiffOne<work::Work, valid::UserCommand>>,
            ),
            Vec<DiffCreationLog>,
        ) {
            match res2 {
                Some(mut res2) => {
                    end_log(&mut res1);
                    end_log(&mut res2);

                    (
                        (pre_diff_with_two(res1.0), Some(pre_diff_with_one(res2.0))),
                        vec![res1.1, res2.1],
                    )
                }
                None => {
                    end_log(&mut res1);
                    ((pre_diff_with_two(res1.0), None), vec![res1.1])
                }
            }
        };

        let has_no_commands = |res: &Resources| -> bool { res.commands_rev.is_empty() };

        let second_pre_diff = |res: Resources,
                               partition: (u64, u64),
                               add_coinbase: bool,
                               work: Vec<work::Unchecked>|
         -> (Resources, DiffCreationLog) {
            Self::one_prediff(
                constraint_constants,
                work,
                res.discarded.commands_rev,
                receiver,
                add_coinbase,
                partition,
                logger,
                is_coinbase_receiver_new,
                Partition::Second,
                supercharge_coinbase,
            )
        };

        let is_empty = |res: &Resources| has_no_commands(res) && res.coinbase_added() == 0;

        // Partitioning explained in PR #687 (Mina repo)

        match partitions.second {
            None => {
                let (res, log) = Self::one_prediff(
                    constraint_constants,
                    cw_seq,
                    ts_seq,
                    receiver,
                    true,
                    partitions.first,
                    logger,
                    is_coinbase_receiver_new,
                    Partition::First,
                    supercharge_coinbase,
                );
                make_diff((res, log), None)
            }
            Some(y) => {
                assert!(cw_seq.len() as u64 <= partitions.first.1 + y.1);

                let (cw_seq_1, cw_seq_2) = split_at_vec(cw_seq, partitions.first.1 as usize);

                let (res, log1) = Self::one_prediff(
                    constraint_constants,
                    cw_seq_1.clone(),
                    ts_seq.clone(),
                    receiver,
                    false,
                    partitions.first,
                    logger,
                    is_coinbase_receiver_new,
                    Partition::First,
                    supercharge_coinbase,
                );

                let incr_coinbase_and_compute = |mut res: Resources,
                                                 count: IncreaseBy|
                 -> (
                    (Resources, DiffCreationLog),
                    Option<(Resources, DiffCreationLog)>,
                ) {
                    res.incr_coinbase_part_by(constraint_constants, count);

                    if res.space_available() {
                        // All slots could not be filled either because of budget
                        // constraints or not enough work done. Don't create the second
                        // prediff instead recompute first diff with just once coinbase
                        let res = Self::one_prediff(
                            constraint_constants,
                            cw_seq_1.clone(),
                            ts_seq.clone(),
                            receiver,
                            true,
                            partitions.first,
                            logger,
                            is_coinbase_receiver_new,
                            Partition::First,
                            supercharge_coinbase,
                        );

                        (res, None)
                    } else {
                        let (res2, log2) = second_pre_diff(res.clone(), y, false, cw_seq_2.clone());

                        if is_empty(&res2) {
                            // Don't create the second prediff instead recompute first
                            // diff with just once coinbase
                            let res = Self::one_prediff(
                                constraint_constants,
                                cw_seq_1.clone(),
                                ts_seq.clone(),
                                receiver,
                                true,
                                partitions.first,
                                logger,
                                is_coinbase_receiver_new,
                                Partition::First,
                                supercharge_coinbase,
                            );

                            (res, None)
                        } else {
                            ((res, log1.clone()), Some((res2, log2)))
                        }
                    }
                };

                let try_with_coinbase = || -> (Resources, DiffCreationLog) {
                    Self::one_prediff(
                        constraint_constants,
                        cw_seq_1.clone(),
                        ts_seq.clone(),
                        receiver,
                        true,
                        partitions.first,
                        logger,
                        is_coinbase_receiver_new,
                        Partition::First,
                        supercharge_coinbase,
                    )
                };

                dbg!(res.available_space_dbg());

                let (res1, res2) = if res.commands_rev.is_empty() {
                    println!("edge_case: No user command added, add a coinbase only (Sequence.is_empty res.commands_rev)");

                    let res = try_with_coinbase();
                    (res, None)
                } else {
                    match res.available_space() {
                        0 => {
                            println!("edge_case: Split commands in 2 diffs/partitions with at least 1 coinbase on next tree (Resources.available_space res = 0)");
                            // generate the next prediff with a coinbase at least
                            let res2 = second_pre_diff(res.clone(), y, true, cw_seq_2);
                            ((res, log1), Some(res2))
                        }
                        1 => {
                            println!("edge_case: only 1 slot available, add coinbase and rest on 2nd diff/partition (Resources.available_space res = 1)");
                            // There's a slot available in the first partition, fill it
                            // with coinbase and create another pre_diff for the slots
                            // in the second partiton with the remaining user commands and work

                            incr_coinbase_and_compute(res, IncreaseBy::One)
                        }
                        2 => {
                            println!("edge_case: 2 slots available, split coinbase in 2 (Resources.available_space res = 2)");
                            // There are two slots which cannot be filled using user commands,
                            // so we split the coinbase into two parts and fill those two spots

                            incr_coinbase_and_compute(res, IncreaseBy::Two)
                        }
                        _ => {
                            println!("edge_case: transactions fit in current tree (Resources.available_space res = _)");
                            // Too many slots left in the first partition. Either there wasn't
                            // enough work to add transactions or there weren't enough
                            // transactions. Create a new pre_diff for just the first partition

                            let res = try_with_coinbase();
                            (res, None)
                        }
                    }
                };

                let coinbase_added = res1.0.coinbase_added()
                    + res2
                        .as_ref()
                        .map(|(res, _)| res.coinbase_added())
                        .unwrap_or(0);

                if coinbase_added > 0 {
                    make_diff(res1, res2)
                } else {
                    // Coinbase takes priority over user-commands. Create a diff in
                    // partitions.first with coinbase first and user commands if possible

                    let res = try_with_coinbase();
                    make_diff(res, None)
                }
            }
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1781
    fn can_apply_supercharged_coinbase_exn(
        winner: CompressedPubKey,
        epoch_ledger: &SparseLedger,
        global_slot: Slot,
    ) -> bool {
        !epoch_ledger.has_locked_tokens_exn(global_slot, AccountId::new(winner, TokenId::default()))
    }

    fn with_ledger_mask<F, R>(base_ledger: Mask, fun: F) -> R
    where
        F: FnOnce(&mut Mask) -> R,
    {
        let mut mask = base_ledger.make_child();
        fun(&mut mask)
    }

    // /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1787
    // fn validate_account_update_proofs(
    //     _logger: (),
    //     validating_ledger: &HashlessLedger,
    //     txn: &valid::UserCommand,
    // ) -> bool {
    //     use super::sparse_ledger::LedgerIntf;

    //     let get_verification_keys = |account_ids: &HashSet<AccountId>| {
    //         let get_vk = |account_id: &AccountId| -> Option<VerificationKeyHash> {
    //             let addr = validating_ledger.location_of_account(account_id)?;
    //             let account = validating_ledger.get(&addr)?;
    //             let vk = account.zkapp.as_ref()?.verification_key.as_ref()?;
    //             // TODO: In OCaml this is a field (using `WithHash`)
    //             Some(VerificationKeyHash(vk.hash()))
    //         };

    //         let mut map = HashMap::with_capacity(128);

    //         for id in account_ids {
    //             match get_vk(id) {
    //                 Some(vk) => {
    //                     map.insert(id.clone(), vk);
    //                 }
    //                 None => {
    //                     eprintln!(
    //                         "Staged_ledger_diff creation: Verification key not found for \
    //                          account_update with proof authorization and account_id \
    //                          {:?}",
    //                         id
    //                     );
    //                     return HashMap::new();
    //                 }
    //             }
    //         }

    //         map
    //     };

    //     match txn {
    //         valid::UserCommand::ZkAppCommand(p) => {
    //             let checked_verification_keys: HashMap<AccountId, VerificationKeyHash> =
    //                 p.verification_keys.iter().cloned().collect();

    //             let proof_zkapp_command = p.zkapp_command.account_updates.fold(
    //                 HashSet::with_capacity(128),
    //                 |mut accum, update| {
    //                     if let Control::Proof(_) = &update.authorization {
    //                         accum.insert(update.account_id());
    //                     }
    //                     accum
    //                 },
    //             );

    //             let current_verification_keys = get_verification_keys(&proof_zkapp_command);

    //             if proof_zkapp_command.len() == checked_verification_keys.len()
    //                 && checked_verification_keys == current_verification_keys
    //             {
    //                 true
    //             } else {
    //                 eprintln!(
    //                     "Staged_ledger_diff creation: Verifcation keys used for verifying \
    //                          proofs {:#?} and verification keys in the \
    //                          ledger {:#?} don't match",
    //                     checked_verification_keys, current_verification_keys
    //                 );
    //                 false
    //             }
    //         }
    //         valid::UserCommand::SignedCommand(_) => true,
    //     }
    // }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1863
    pub fn create_diff<F>(
        &self,
        constraint_constants: &ConstraintConstants,
        global_slot: Slot,
        log_block_creation: Option<bool>,
        coinbase_receiver: CompressedPubKey,
        logger: (),
        current_state_view: &ProtocolStateView,
        transactions_by_fee: Vec<valid::UserCommand>,
        get_completed_work: F,
        supercharge_coinbase: bool,
    ) -> Result<
        (
            with_valid_signatures_and_proofs::Diff,
            Vec<(valid::UserCommand, String)>,
        ),
        PreDiffError,
    >
    where
        F: Fn(&work::Statement) -> Option<work::Checked>,
    {
        let _log_block_creation = log_block_creation.unwrap_or(false);

        Self::with_ledger_mask(self.ledger.clone(), move |validating_ledger| {
            let is_new_account = |pk: &CompressedPubKey| {
                validating_ledger
                    .location_of_account(&AccountId::new(pk.clone(), TokenId::default()))
                    .is_none()
            };

            let is_coinbase_receiver_new = is_new_account(&coinbase_receiver);

            if supercharge_coinbase {
                // println!("No locked tokens in the delegator/delegatee account, applying supercharged coinbase");
            }

            let partitions = self.scan_state.partition_if_overflowing();
            let work_to_do = self.scan_state.work_statements_for_new_diff();

            let mut completed_works_seq = Vec::with_capacity(work_to_do.len());
            let mut proof_count = 0;

            for work in work_to_do {
                match get_completed_work(&work) {
                    Some(cw_checked) => {
                        // If new provers can't pay the account-creation-fee then discard
                        // their work unless their fee is zero in which case their account
                        // won't be created. This is to encourage using an existing accounts
                        // for snarking.
                        // This also imposes new snarkers to have a min fee until one of
                        // their snarks are purchased and their accounts get created*)

                        if cw_checked.fee.is_zero()
                            || cw_checked.fee.as_u64() >= constraint_constants.account_creation_fee
                            || !(is_new_account(&cw_checked.prover))
                        {
                            proof_count += cw_checked.proofs.len();
                            completed_works_seq.push(cw_checked);
                        } else {
                            eprintln!(
                                "Staged_ledger_diff creation: Snark fee {:?} \
                                 insufficient to create the snark worker account",
                                cw_checked.fee,
                            );
                            break;
                        }
                    }
                    None => {
                        eprintln!(
                            "Staged_ledger_diff creation: No snark work found for {:#?}",
                            work
                        );
                        break;
                    }
                }
            }

            // Transactions in reverse order for faster removal if there is no space when creating the diff

            let length = transactions_by_fee.len();
            let mut valid_on_this_ledger = Vec::with_capacity(length);
            let mut invalid_on_this_ledger = Vec::with_capacity(length);
            let mut count = 0;

            let _transactions_by_fee_len = transactions_by_fee.len();

            for txn in transactions_by_fee {
                let res = transaction_validator::apply_transaction_first_pass(
                    constraint_constants,
                    global_slot,
                    current_state_view,
                    validating_ledger,
                    &Transaction::Command(txn.forget_check()),
                );

                match res {
                    Err(e) => {
                        eprintln!(
                            "Staged_ledger_diff creation: Skipping user command: {:#?} due to error: {:?}",
                            txn, e
                        );
                        invalid_on_this_ledger.push((txn, e));
                    }
                    Ok(_txn_partially_applied) => {
                        valid_on_this_ledger.push(txn);
                        count += 1;
                        if count >= self.scan_state.free_space() {
                            break;
                        }
                    }
                }
            }

            valid_on_this_ledger.reverse();
            invalid_on_this_ledger.reverse();

            let _valid_on_this_ledger_len = valid_on_this_ledger.len();

            let (diff, _log) = Self::generate(
                constraint_constants,
                logger,
                completed_works_seq,
                valid_on_this_ledger,
                &coinbase_receiver,
                is_coinbase_receiver_new,
                supercharge_coinbase,
                partitions,
            );

            // let diff: Result<_, PreDiffError> = {
            let diff = {
                // Fill in the statuses for commands.
                Self::with_ledger_mask(self.ledger.clone(), |status_ledger| {
                    pre_diff_info::compute_statuses::<valid::Transaction>(
                        constraint_constants,
                        diff,
                        coinbase_receiver,
                        Self::coinbase_amount(supercharge_coinbase, constraint_constants)
                            .expect("OCaml throws here"),
                        global_slot,
                        current_state_view,
                        status_ledger,
                    )
                })
            }?;

            // let diff = diff?;

            // curr_job_seq_no is incremented later, but for the logs we increment it now
            println!(
                "sequence_number={:?}",
                self.scan_state.scan_state.curr_job_seq_no.incr()
            );

            let _ = proof_count;
            // println!(
            //     "Number of proofs ready for purchase: {} Number of user \
            //      commands ready to be included: {} Diff creation log: {:#?}",
            //     proof_count,
            //     valid_on_this_ledger_len,
            //     log.iter().map(|l| &l.summary).collect::<Vec<_>>()
            // );

            // if log_block_creation {
            //     println!("Detailed diff creation log: {:#?}", {
            //         let mut details = log.iter().map(|l| &l.detail).collect::<Vec<_>>();
            //         details.reverse();
            //         details
            //     })
            // }

            let diff = with_valid_signatures_and_proofs::Diff { diff };

            Ok((diff, invalid_on_this_ledger))
        })
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L2024
    fn latest_block_accounts_created(&self, previous_block_state_hash: Fp) -> Vec<AccountId> {
        use scan_state::transaction_logic::transaction_applied::signed_command_applied::Body;
        use scan_state::transaction_logic::transaction_applied::CommandApplied;
        use scan_state::transaction_logic::transaction_applied::Varying;

        let block_transactions_applied = {
            let f = |t: Arc<TransactionWithWitness>| {
                let TransactionWithWitness {
                    transaction_with_info,
                    state_hash: (leaf_block_hash, _),
                    ..
                } = t.as_ref();
                if leaf_block_hash == &previous_block_state_hash {
                    Some(transaction_with_info.varying.to_owned())
                } else {
                    None
                }
            };

            let latest = self.scan_state.base_jobs_on_latest_tree().filter_map(f);

            let earlier = self.scan_state.base_jobs_on_earlier_tree(0).filter_map(f);

            latest.chain(earlier)
        };

        block_transactions_applied
            .flat_map(|cmd| match cmd {
                Varying::Command(CommandApplied::SignedCommand(cmd)) => match cmd.body {
                    Body::Payments { new_accounts } => new_accounts,
                    Body::StakeDelegation { .. } => Vec::new(),
                    Body::Failed => Vec::new(),
                },
                Varying::Command(CommandApplied::ZkappCommand(cmd)) => cmd.new_accounts,
                Varying::FeeTransfer(ft) => ft.new_accounts,
                Varying::Coinbase(cb) => cb.new_accounts,
            })
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests_ocaml {
    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use std::{
        collections::{BTreeSet, HashMap},
        panic::AssertUnwindSafe,
        str::FromStr,
        sync::atomic::{AtomicUsize, Ordering::Relaxed},
    };

    use ark_ec::{AffineCurve, ProjectiveCurve};
    use ark_ff::Zero;
    use mina_curves::pasta::Fq;
    use mina_signer::Signer;
    use mina_signer::{Keypair, Signature};
    use o1_utils::FieldHelpers;
    use once_cell::sync::Lazy;
    use rand::{seq::SliceRandom, CryptoRng, Rng};

    use crate::{
        dummy::{self, trivial_verification_key},
        gen_keypair,
        generators::{
            user_command::sequence_zkapp_command_with_ledger, zkapp_command_builder, Failure,
        },
        scan_state::{
            currency::{Balance, Fee, Nonce, SlotSpan},
            scan_state::transaction_snark::SokDigest,
            transaction_logic::{
                apply_transactions,
                protocol_state::protocol_state_view,
                signed_command::{
                    self, Common, PaymentPayload, SignedCommand, SignedCommandPayload,
                },
                transaction_union_payload::TransactionUnionPayload,
                zkapp_command::{self, verifiable::find_vk_via_ledger, SetOrKeep, WithHash},
                Memo, TransactionFailure,
            },
        },
        staged_ledger::diff::{
            PreDiffOne, PreDiffWithAtMostOneCoinbase, PreDiffWithAtMostTwoCoinbase,
        },
        util, Account, AuthRequired, Permissions, VerificationKey, VerificationKeyWire,
    };

    use super::*;

    // const

    static SELF_PK: Lazy<CompressedPubKey> = Lazy::new(|| gen_keypair().public.into_compressed());

    static COINBASE_RECEIVER: Lazy<CompressedPubKey> = Lazy::new(|| {
        CompressedPubKey::from_address("B62qmkso2Knz9pxo5V9YEZFJ9Frq57GZfKgem1DVTKiYH9D5H3n2DGS")
            .unwrap()
    });
    // Lazy::new(|| gen_keypair().public.into_compressed());

    /// Same values when we run `dune runtest src/lib/staged_ledger -f`
    pub const CONSTRAINT_CONSTANTS: ConstraintConstants = ConstraintConstants {
        sub_windows_per_window: 11,
        ledger_depth: 35,
        work_delay: 2,
        block_window_duration_ms: 180000,
        transaction_capacity_log_2: 7,
        pending_coinbase_depth: 5,
        coinbase_amount: 720000000000,
        supercharged_coinbase_factor: 2,
        account_creation_fee: 1000000000,
        fork: None,
    };

    const LOGGER: () = ();

    const VERIFIER: Verifier = Verifier;

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2092
    fn supercharge_coinbase(ledger: Mask, winner: CompressedPubKey, global_slot: Slot) -> bool {
        // using staged ledger to confirm coinbase amount is correctly generated

        let epoch_ledger = SparseLedger::of_ledger_subset_exn(
            ledger,
            &[AccountId::new(winner.clone(), TokenId::default())],
        );

        StagedLedger::can_apply_supercharged_coinbase_exn(winner, &epoch_ledger, global_slot)
    }

    /// Functor for testing with different instantiated staged ledger modules.
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2102
    fn create_and_apply_with_state_body_hash<F>(
        coinbase_receiver: Option<CompressedPubKey>,
        winner: Option<CompressedPubKey>,
        current_state_view: &ProtocolStateView,
        global_slot: Slot,
        state_and_body_hash: (Fp, Fp),
        sl: &mut StagedLedger,
        txns: &[valid::UserCommand],
        stmt_to_work: F,
    ) -> (
        Option<(
            LedgerProof,
            Vec<TransactionsOrdered<(WithStatus<Transaction>, Fp, Slot)>>,
        )>,
        Diff,
        bool,
        Update,
        bool,
    )
    where
        F: Fn(&work::Statement) -> Option<work::Unchecked>,
    {
        let coinbase_receiver = coinbase_receiver.unwrap_or_else(|| COINBASE_RECEIVER.clone());
        let winner = winner.unwrap_or_else(|| SELF_PK.clone());

        let supercharge_coinbase = supercharge_coinbase(sl.ledger.clone(), winner, global_slot);

        let (diff, _invalid_txns) = sl
            .create_diff(
                &CONSTRAINT_CONSTANTS,
                global_slot,
                None,
                coinbase_receiver.clone(),
                LOGGER,
                current_state_view,
                txns.to_vec(),
                stmt_to_work,
                supercharge_coinbase,
            )
            .unwrap();

        let diff = diff.forget();

        let DiffResult {
            hash_after_applying: hash,
            ledger_proof,
            pending_coinbase_update: (is_new_stack, pc_update),
        } = sl
            .apply(
                None,
                &CONSTRAINT_CONSTANTS,
                global_slot,
                diff.clone(),
                LOGGER,
                &VERIFIER,
                current_state_view,
                state_and_body_hash,
                coinbase_receiver,
                supercharge_coinbase,
            )
            .unwrap();

        assert_eq!(hash, sl.hash());

        (
            ledger_proof,
            diff,
            is_new_stack,
            pc_update,
            supercharge_coinbase,
        )
    }

    #[derive(Debug)]
    struct LedgerInitialState {
        state: Vec<(Keypair, Amount, Nonce, crate::account::Timing)>,
    }

    fn gen<T>(n_accounts: usize, fun: impl FnMut() -> T) -> Vec<T> {
        std::iter::repeat_with(fun)
            .take(n_accounts)
            .collect::<Vec<_>>()
    }

    /// https://github.com/MinaProtocol/mina/blob/3a78f0e0c1343d14e2729c8b00205baa2ec70c93/src/lib/mina_ledger/ledger.ml#L408
    fn gen_initial_ledger_state() -> LedgerInitialState {
        let mut rng = rand::thread_rng();

        let n_accounts = rng.gen_range(2..10);

        let keypairs = gen(n_accounts, gen_keypair);
        let balances = gen(n_accounts, || {
            let balance: u64 = rng.gen_range(500_000_000..1_000_000_000);
            Amount::from_u64(balance.checked_mul(1_000_000_000).unwrap())
        });
        let nonces = gen(n_accounts, || {
            let nonce: u32 = rng.gen_range(0..1000);
            Nonce::from_u32(nonce)
        });

        let state = keypairs
            .into_iter()
            .zip(balances)
            .zip(nonces)
            .map(|((keypair, balance), nonce)| {
                (keypair, balance, nonce, crate::account::Timing::Untimed)
            })
            .collect();

        LedgerInitialState { state }
    }

    /// How many blocks do we need to fully exercise the ledger
    /// behavior and produce one ledger proof *)
    const MIN_BLOCKS_FOR_FIRST_SNARKED_LEDGER_GENERIC: usize =
        (CONSTRAINT_CONSTANTS.transaction_capacity_log_2 as usize + 1)
            * (CONSTRAINT_CONSTANTS.work_delay as usize + 1)
            + 1;

    const TRANSACTION_CAPACITY: usize =
        2u64.pow(CONSTRAINT_CONSTANTS.transaction_capacity_log_2 as u32) as usize;

    // let transaction_capacity =
    //   Int.pow 2 constraint_constants.transaction_capacity_log_2

    /// n-1 extra blocks for n ledger proofs since we are already producing one
    /// proof *)
    fn max_blocks_for_coverage(n: usize) -> usize {
        MIN_BLOCKS_FOR_FIRST_SNARKED_LEDGER_GENERIC + n - 1
    }

    #[derive(Debug, Copy, Clone)]
    enum SignKind {
        Fake,
        Real,
    }

    /// [gen_division n k] generates a list of [k] integers which sum to [n]
    /// val gen_division : int -> int -> int list Generator.t
    fn gen_division(n: usize, k: usize) -> Vec<usize> {
        // TODO: Improve that

        let mut rng = rand::thread_rng();
        let mut sum = 0;

        let vec = (0..k)
            .map(|index| {
                let int = rng.gen_range(1..(n / k) - 1);

                let int = if index == k - 1 {
                    n - sum
                } else {
                    int.min(n - sum)
                };

                sum += int;
                int
            })
            .collect::<Vec<_>>();

        assert_eq!(vec.len(), k);
        assert_eq!(vec.iter().sum::<usize>(), n);

        vec
    }

    fn gen_division_currency(amount: Amount, k: usize) -> Vec<Amount> {
        let amount = amount.as_u64() as usize;
        gen_division(amount, k)
            .into_iter()
            .map(|amount| Amount::from_u64(amount as u64))
            .collect()
    }

    fn signed_command_sequence_impl(
        length: usize,
        sign_kind: SignKind,
        ledger: &LedgerInitialState,
    ) -> Result<Vec<valid::SignedCommand>, ()> {
        use scan_state::transaction_logic::signed_command::Body;

        let mut rng = rand::thread_rng();
        let n_commands = length;

        if n_commands == 0 {
            return Ok(vec![]);
        }

        let n_accounts = ledger.state.len();

        let (command_senders, mut currency_splits) = loop {
            // How many commands will be issued from each account?
            let command_splits = gen_division(n_commands, n_accounts);

            // List of payment senders in the final order.
            let mut command_senders = command_splits
                .iter()
                .enumerate()
                .flat_map(|(idx, cmds)| vec![idx; *cmds])
                .collect::<Vec<_>>();
            command_senders.shuffle(&mut rng);

            // within the accounts, how will the currency be split into separate
            // payments?
            let currency_splits = (0..n_accounts)
                .map(|i| {
                    let spend_all: bool = rng.gen();
                    let (_, balance, _, _) = &ledger.state[i];
                    let amount_to_spend = if spend_all {
                        *balance
                    } else {
                        Amount::from_u64(balance.as_u64() / 2)
                    };

                    gen_division_currency(amount_to_spend, command_splits[i])
                })
                .collect::<Vec<_>>();

            // We need to ensure each command has enough currency for a fee of 2
            // or more, so it'll be enough to buy the requisite transaction
            // snarks. It's important that the backtracking from filter goes and
            // redraws command_splits as well as currency_splits, so we don't get
            // stuck in a situation where it's very unlikely for the predicate to
            // pass.
            if currency_splits.iter().all(|list| {
                !list.is_empty()
                    && list
                        .iter()
                        .all(|amount| amount >= &Amount::from_u64(2_000_000_000))
            }) {
                break (command_senders, currency_splits);
            }
        };

        let mut account_nonces: Vec<Nonce> =
            ledger.state.iter().map(|(_, _, nonce, _)| *nonce).collect();

        command_senders
            .into_iter()
            .enumerate()
            .map(|(number, sender)| {
                if currency_splits[sender].is_empty() {
                    return Err(());
                }

                let (this_split, rest_splits) = currency_splits[sender].split_at(1);
                let this_split = this_split[0];

                let (sender_pk, _, _, _) = &ledger.state[sender];

                currency_splits[sender] = rest_splits.to_vec();

                let nonce = account_nonces[sender];
                account_nonces[sender] = nonce.incr();

                // println!("this={:?}", this_split);
                let min = 6000000000;
                let fee = rng.gen_range(min..(10000000000.min(this_split.as_u64()).max(min + 1)));
                let fee = Fee::from_u64(fee);

                let amount = match this_split.checked_sub(&Amount::of_fee(&fee)) {
                    Some(amount) => amount,
                    None => return Err(()),
                };

                let receiver = {
                    // Take random item in `ledger.state`
                    let (kp, _, _, _) = ledger.state.choose(&mut rng).unwrap();
                    kp.public.into_compressed()
                };

                // let memo = Memo::dummy();
                let memo = Memo::with_number(number);

                let payload = {
                    let sender_pk = sender_pk.public.into_compressed();

                    SignedCommandPayload::create(
                        fee,
                        sender_pk,
                        nonce,
                        None,
                        memo,
                        Body::Payment(PaymentPayload {
                            receiver_pk: receiver,
                            amount,
                        }),
                    )
                };

                let signature = match sign_kind {
                    SignKind::Fake => Signature::dummy(),
                    SignKind::Real => {
                        let payload_to_sign =
                            TransactionUnionPayload::of_user_command_payload(&payload);

                        let mut signer =
                            mina_signer::create_legacy(mina_signer::NetworkId::TESTNET);
                        signer.sign(sender_pk, &payload_to_sign)
                    }
                };

                Ok(SignedCommand {
                    payload,
                    signer: sender_pk.public.into_compressed(),
                    signature,
                })
            })
            .collect::<Result<Vec<_>, ()>>()
    }

    /// Generate a valid sequence of payments based on the initial state of a
    /// ledger. Use this together with Ledger.gen_initial_ledger_state.
    ///
    /// https://github.com/MinaProtocol/mina/blob/3a78f0e0c1343d14e2729c8b00205baa2ec70c93/src/lib/mina_base/signed_command.ml#L246
    fn signed_command_sequence(
        length: usize,
        sign_kind: SignKind,
        ledger: &LedgerInitialState,
    ) -> Vec<valid::UserCommand> {
        // Not clean but it's what OCaml does when an exception is throwned, if I understand correctly
        for _ in 0..100 {
            if let Ok(commands) = signed_command_sequence_impl(length, sign_kind, ledger) {
                return commands
                    .into_iter()
                    .map(|cmd| valid::UserCommand::SignedCommand(Box::new(cmd)))
                    .collect();
            };
        }
        panic!("Failed to generate random user commands");
    }

    /// Same as gen_at_capacity except that the number of iterations[iters] is
    /// the function of [extra_block_count] and is same for all generated values
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2597
    fn gen_at_capacity_fixed_blocks(
        extra_block_count: usize,
    ) -> (
        LedgerInitialState,
        Vec<valid::UserCommand>,
        Vec<Option<usize>>,
    ) {
        let state = gen_initial_ledger_state();
        let iters = max_blocks_for_coverage(extra_block_count);
        let total_cmds = TRANSACTION_CAPACITY * iters;

        let cmds = signed_command_sequence(total_cmds, SignKind::Real, &state);
        assert_eq!(cmds.len(), total_cmds);

        (state, cmds, vec![None; iters])
    }

    fn apply_initialize_ledger_state(mask: &mut Mask, init_state: &LedgerInitialState) {
        use crate::sparse_ledger::LedgerIntf;

        for (kp, balance, nonce, timing) in &init_state.state {
            let pk_compressed = kp.public.into_compressed();
            let account_id = AccountId::new(pk_compressed, TokenId::default());
            let mut account = Account::initialize(&account_id);
            account.balance = Balance::from_u64(balance.as_u64());
            account.nonce = *nonce;
            account.timing = timing.clone();

            mask.create_new_account(account_id, account).unwrap();
        }
    }

    static NITERS: AtomicUsize = AtomicUsize::new(0);

    type SnarkedLedger = Mask;

    /// Run the given function inside of the Deferred monad, with a staged
    ///   ledger and a separate test ledger, after applying the given
    ///   init_state to both. In the below tests we apply the same commands to
    ///   the staged and test ledgers, and verify they are in the same state.
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2180
    #[allow(clippy::blocks_in_conditions)]
    fn async_with_given_ledger<F, R>(
        _ledger_init_state: &LedgerInitialState,
        cmds: Vec<valid::UserCommand>,
        cmd_iters: Vec<Option<usize>>,
        mask: Mask,
        fun: F,
    ) where
        F: FnOnce(SnarkedLedger, StagedLedger, Mask) -> R,
    {
        match std::panic::catch_unwind(AssertUnwindSafe(move || {
            let test_mask = mask.make_child();
            let snarked_ledger_mask = mask.make_child();
            let sl = StagedLedger::create_exn(CONSTRAINT_CONSTANTS, mask).unwrap();
            fun(snarked_ledger_mask, sl, test_mask.clone());
            test_mask.unregister_mask(crate::UnregisterBehavior::Check);
        })) {
            Ok(_) => {}
            Err(_) => {
                let niters = NITERS.load(Relaxed);
                let iters = cmd_iters
                    .iter()
                    .filter_map(|n| n.as_ref())
                    .take(niters + 1)
                    .collect::<Vec<_>>();

                println!("NITERS_LA={}", niters);
                eprintln!("state={:#?}", "ignored");
                // eprintln!("state={:#?}", ledger_init_state);
                eprintln!("cmds[{}]={:#?}", cmds.len(), "ignored");
                // eprintln!("cmds[{}]={:#?}", cmds.len(), cmds);
                eprintln!("cmd_iters[{}]={:?}", iters.len(), iters);
                panic!("test failed (see logs above)");
            }
        }
    }

    /// populate the ledger from an initial state before running the function
    ///
    /// Print the generated state when a panic occurs
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2192
    fn async_with_ledgers<F, R>(
        ledger_init_state: &LedgerInitialState,
        cmds: Vec<valid::UserCommand>,
        cmd_iters: Vec<Option<usize>>,
        fun: F,
    ) where
        F: FnOnce(SnarkedLedger, StagedLedger, Mask) -> R,
    {
        let mut ephemeral_ledger = Mask::new_unattached(CONSTRAINT_CONSTANTS.ledger_depth as usize);

        apply_initialize_ledger_state(&mut ephemeral_ledger, ledger_init_state);
        async_with_given_ledger(ledger_init_state, cmds, cmd_iters, ephemeral_ledger, fun);
    }

    /// Get the public keys from a ledger init state.
    fn init_pks(init: &LedgerInitialState) -> Vec<AccountId> {
        init.state
            .iter()
            .map(|(kp, _, _, _)| AccountId::new(kp.public.into_compressed(), TokenId::default()))
            .collect()
    }

    #[derive(Copy, Debug, Clone)]
    enum NumProvers {
        One,
        Many,
    }

    /// Abstraction for the pattern of taking a list of commands and applying it
    /// in chunks up to a given max size.
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2392
    fn iter_cmds_acc<A, F>(
        cmds: &[valid::UserCommand],
        cmd_iters: &[Option<usize>],
        acc: A,
        mut fun: F,
    ) -> A
    where
        F: FnMut(
            &[valid::UserCommand], // all remaining commands
            Option<usize>,         // Current chunk size.
            &[valid::UserCommand], // Sequence of commands to apply.
            A,
        ) -> (Diff, A),
    {
        match cmd_iters.first() {
            None => acc,
            Some(count_opt) => {
                let cmds_this_iter_max = match count_opt {
                    None => cmds,
                    Some(count) => {
                        assert!(*count <= cmds.len());
                        util::take(cmds, *count)
                    }
                };

                let (diff, acc) = fun(cmds, *count_opt, cmds_this_iter_max, acc);

                let cmds_applied_count = diff.commands().len();

                let cmds = util::drop(cmds, cmds_applied_count);
                let counts_rest = &cmd_iters[1..];

                iter_cmds_acc(cmds, counts_rest, acc, fun)
            }
        }
    }

    fn dummy_state_and_view(
        global_slot: Option<Slot>,
    ) -> (
        mina_p2p_messages::v2::MinaStateProtocolStateValueStableV2,
        ProtocolStateView,
    ) {
        let mut state = dummy::for_tests::dummy_protocol_state();

        if let Some(global_slot) = global_slot {
            let prev_global_slot = global_slot
                .checked_sub(&Slot::from_u32(1))
                .unwrap_or_else(Slot::zero);

            let new_global_slot = prev_global_slot;

            let global_slot_since_genesis = {
                let since_genesis = &state.body.consensus_state.global_slot_since_genesis;
                let curr = &state
                    .body
                    .consensus_state
                    .curr_global_slot_since_hard_fork
                    .slot_number;

                let since_genesis: Slot = since_genesis.into();
                let curr: Slot = curr.into();

                (since_genesis.checked_sub(&curr).unwrap())
                    .checked_add(&new_global_slot)
                    .unwrap()
            };

            let cs = &mut state.body.consensus_state;
            cs.curr_global_slot_since_hard_fork.slot_number = (&new_global_slot).into();
            cs.global_slot_since_genesis = (&global_slot_since_genesis).into();
        };

        let view = protocol_state_view(&state).unwrap();

        (state, view)
    }

    fn dummy_state_view(global_slot: Option<Slot>) -> ProtocolStateView {
        dummy_state_and_view(global_slot).1
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2164
    fn create_and_apply<F>(
        coinbase_receiver: Option<CompressedPubKey>,
        winner: Option<CompressedPubKey>,
        global_slot: Slot,
        protocol_state_view: &ProtocolStateView,
        state_and_body_hash: (Fp, Fp),
        sl: &mut StagedLedger,
        txns: &[valid::UserCommand],
        stmt_to_work: F,
    ) -> (
        Option<(
            LedgerProof,
            Vec<TransactionsOrdered<(WithStatus<Transaction>, Fp, Slot)>>,
        )>,
        Diff,
    )
    where
        F: Fn(&work::Statement) -> Option<work::Checked>,
    {
        let (ledger_proof, diff, _, _, _) = create_and_apply_with_state_body_hash(
            coinbase_receiver,
            winner,
            protocol_state_view,
            global_slot,
            state_and_body_hash,
            sl,
            txns,
            stmt_to_work,
        );
        (ledger_proof, diff)
    }

    /// Fee excess at top level ledger proofs should always be zero
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2377
    fn assert_fee_excess(
        proof: &Option<(
            LedgerProof,
            Vec<TransactionsOrdered<(WithStatus<Transaction>, Fp, Slot)>>,
        )>,
    ) {
        if let Some((proof, _txns)) = proof {
            assert!(proof.statement().fee_excess.is_zero());
        };
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2322
    fn coinbase_first_prediff(
        v: &AtMostTwo<CoinbaseFeeTransfer>,
    ) -> (usize, Vec<&CoinbaseFeeTransfer>) {
        match v {
            AtMostTwo::Zero => (0, vec![]),
            AtMostTwo::One(None) => (1, vec![]),
            AtMostTwo::One(Some(ft)) => (1, vec![ft]),
            AtMostTwo::Two(None) => (2, vec![]),
            AtMostTwo::Two(Some((ft, None))) => (2, vec![ft]),
            AtMostTwo::Two(Some((ft, Some(ft2)))) => (2, vec![ft, ft2]),
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2336
    fn coinbase_second_prediff(
        v: &AtMostOne<CoinbaseFeeTransfer>,
    ) -> (usize, Vec<&CoinbaseFeeTransfer>) {
        match v {
            AtMostOne::Zero => (0, vec![]),
            AtMostOne::One(None) => (1, vec![]),
            AtMostOne::One(Some(ft)) => (1, vec![ft]),
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2344
    fn coinbase_count(sl_diff: &Diff) -> usize {
        coinbase_first_prediff(&sl_diff.diff.0.coinbase).0
            + sl_diff
                .diff
                .1
                .as_ref()
                .map(|d| coinbase_second_prediff(&d.coinbase).0)
                .unwrap_or(0)
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2349
    fn coinbase_cost(sl_diff: &Diff) -> Fee {
        let first = coinbase_first_prediff(&sl_diff.diff.0.coinbase).1;
        let snd = sl_diff
            .diff
            .1
            .as_ref()
            .map(|d| coinbase_second_prediff(&d.coinbase).1)
            .unwrap_or_default();

        first
            .into_iter()
            .chain(snd)
            .fold(Fee::zero(), |total, ft| total.checked_add(&ft.fee).unwrap())
    }

    /// Assert the given staged ledger is in the correct state after applying
    /// the first n user commands passed to the given base ledger. Checks the
    /// states of the block producer account and user accounts but ignores
    /// snark workers for simplicity.
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2203
    #[allow(unused)]
    fn assert_ledger(
        test_ledger: Mask,
        coinbase_cost: Fee,
        global_slot: Slot,
        protocol_state_view: &ProtocolStateView,
        staged_ledger: &StagedLedger,
        cmds_all: &[valid::UserCommand],
        cmds_used: usize,
        pks_to_check: &[AccountId],
    ) {
        // return;

        let producer_account_id = AccountId::new(COINBASE_RECEIVER.clone(), TokenId::default());
        let producer_account = test_ledger
            .location_of_account(&producer_account_id)
            .and_then(|loc| test_ledger.get(loc));

        let is_producer_acc_new = producer_account.is_none();
        let old_producer_balance = match producer_account.as_ref() {
            Some(account) => account.balance,
            None => Balance::zero(),
        };

        let mut test_ledger = test_ledger;

        let cmds: Vec<_> = util::take(cmds_all, cmds_used)
            .iter()
            .map(|cmd| Transaction::Command(cmd.forget_check()))
            .collect();

        apply_transactions(
            &CONSTRAINT_CONSTANTS,
            global_slot,
            protocol_state_view,
            &mut test_ledger,
            &cmds,
        )
        .unwrap();

        let get_account_exn = |ledger: &Mask, id: &AccountId| {
            let loc = ledger.location_of_account(id).unwrap();
            ledger.get(loc).unwrap()
        };

        // Check the user accounts in the updated staged ledger are as
        // expected.

        for id in pks_to_check {
            let expect = get_account_exn(&test_ledger, id);
            let actual = get_account_exn(&staged_ledger.ledger, id);
            assert_eq!(expect, actual);
        }

        // We only test that the block producer got the coinbase reward here, since calculating
        // the exact correct amount depends on the snark fees and tx fees.
        let producer_balance_with_coinbase = {
            let total_cost = if is_producer_acc_new {
                coinbase_cost
                    .checked_add(&Fee::from_u64(CONSTRAINT_CONSTANTS.account_creation_fee))
                    .unwrap()
            } else {
                coinbase_cost
            };

            let reward = Amount::from_u64(CONSTRAINT_CONSTANTS.coinbase_amount)
                .checked_sub(&Amount::of_fee(&total_cost))
                .unwrap();

            old_producer_balance.add_amount(reward).unwrap()
        };

        let new_producer_balance =
            get_account_exn(&staged_ledger.ledger, &producer_account_id).balance;

        assert!(new_producer_balance >= producer_balance_with_coinbase);
    }

    fn hashes_abstract(
        state: &mina_p2p_messages::v2::MinaStateProtocolStateValueStableV2,
    ) -> (Fp, Fp) {
        let state: crate::proofs::block::ProtocolState = state.try_into().unwrap();
        state.hashes()
    }

    struct TestSimpleParams<F: Fn(&work::Statement) -> Option<work::Checked>> {
        global_slot: Slot,
        account_ids_to_check: Vec<AccountId>,
        cmds: Vec<valid::UserCommand>,
        cmd_iters: Vec<Option<usize>>,
        sl: StagedLedger,
        // Number of ledger proofs expected
        expected_proof_count: Option<usize>,
        allow_failure: Option<bool>,
        check_snarked_ledger_transition: Option<bool>,
        snarked_ledger: SnarkedLedger,
        test_mask: Mask,
        provers: NumProvers,
        stmt_to_work: F,
    }

    /// Generic test framework.
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2427
    fn test_simple<F>(params: TestSimpleParams<F>) -> StagedLedger
    where
        F: Fn(&work::Statement) -> Option<work::Checked>,
    {
        let TestSimpleParams {
            global_slot,
            account_ids_to_check,
            cmds,
            cmd_iters,
            mut sl,
            expected_proof_count,
            allow_failure,
            check_snarked_ledger_transition,
            snarked_ledger,
            test_mask,
            provers,
            stmt_to_work,
        } = params;

        eprintln!(
            "test_simple ncmds={:?} niters={:?}",
            cmds.len(),
            cmd_iters.len()
        );

        let allow_failure = allow_failure.unwrap_or(false);
        let check_snarked_ledger_transition = check_snarked_ledger_transition.unwrap_or(false);

        let mut state_tbl = HashMap::with_capacity(128);
        let (genesis, _) = dummy_state_and_view(None);

        let (state_hash, _) = hashes_abstract(&genesis);

        state_tbl.insert(state_hash, genesis);

        // let mut niters = 0;

        let (total_ledger_proofs, _) = iter_cmds_acc(
            &cmds,
            &cmd_iters,
            (0, global_slot),
            |cmds_left, count_opt, cmds_this_iter, (mut proof_count, global_slot)| {
                let niters = NITERS.load(std::sync::atomic::Ordering::Relaxed);

                println!("\n######## Start new batch {} ########", niters);
                println!("attempt_to_apply_nuser_commands={:?}", cmds_this_iter.len());

                let (current_state, current_view) = dummy_state_and_view(Some(global_slot));

                let (state_hash, state_body_hash) = hashes_abstract(&current_state);

                state_tbl.insert(state_hash, current_state.clone());

                let state_and_body_hash = (state_hash, state_body_hash);

                let (ledger_proof, diff) = create_and_apply(
                    None,
                    None,
                    global_slot,
                    &current_view,
                    state_and_body_hash,
                    &mut sl,
                    cmds_this_iter,
                    &stmt_to_work,
                );

                for cmd in diff.commands() {
                    if allow_failure {
                        continue;
                    }
                    if let TransactionStatus::Failed(e) = &cmd.status {
                        panic!(
                            "Transaction application failed for command {:#?}. Failures {:#?}",
                            cmd, e
                        );
                    };
                }

                let do_snarked_ledger_transition = |proof_opt: Option<&(
                    LedgerProof,
                    Vec<TransactionsOrdered<(WithStatus<Transaction>, Fp, Slot)>>,
                )>| {
                    let mut snarked_ledger = snarked_ledger.clone();

                    let apply_first_pass =
                        |global_slot: Slot,
                         txn_state_view: &ProtocolStateView,
                         ledger: &mut Mask,
                         transaction: &Transaction| {
                            apply_transaction_first_pass(
                                &CONSTRAINT_CONSTANTS,
                                global_slot,
                                txn_state_view,
                                ledger,
                                transaction,
                            )
                        };

                    let apply_second_pass =
                        |ledger: &mut Mask, tx: TransactionPartiallyApplied<Mask>| {
                            apply_transaction_second_pass(&CONSTRAINT_CONSTANTS, ledger, tx)
                        };

                    let apply_first_pass_sparse_ledger =
                        |global_slot: Slot,
                         txn_state_view: &ProtocolStateView,
                         sparse_ledger: &mut SparseLedger,
                         transaction: &Transaction| {
                            apply_transaction_first_pass(
                                &CONSTRAINT_CONSTANTS,
                                global_slot,
                                txn_state_view,
                                sparse_ledger,
                                transaction,
                            )
                        };

                    let get_state = |hash: Fp| Ok(state_tbl.get(&hash).cloned().unwrap());

                    if let Some((proof, _transactions)) = proof_opt.as_ref() {
                        // update snarked ledger with the transactions in the most recently emitted proof
                        let _res = sl
                            .scan_state
                            .get_staged_ledger_sync(
                                &mut snarked_ledger,
                                get_state,
                                apply_first_pass,
                                apply_second_pass,
                                apply_first_pass_sparse_ledger,
                            )
                            .unwrap();

                        let target_snarked_ledger = {
                            let stmt = proof.statement();
                            stmt.target.first_pass_ledger
                        };

                        assert_eq!(target_snarked_ledger, snarked_ledger.merkle_root());
                    };

                    // Check snarked_ledger to staged_ledger transition
                    let mut sl_of_snarked_ledger = snarked_ledger.make_child();

                    let expected_staged_ledger_merkle_root = sl.ledger.clone().merkle_root();

                    let get_state = |hash: Fp| state_tbl.get(&hash).cloned().unwrap();

                    StagedLedger::of_scan_state_pending_coinbases_and_snarked_ledger(
                        (),
                        &CONSTRAINT_CONSTANTS,
                        crate::verifier::Verifier,
                        sl.scan_state.clone(),
                        snarked_ledger.clone(),
                        {
                            let registers: Registers = (&current_state
                                .body
                                .blockchain_state
                                .ledger_proof_statement
                                .target)
                                .try_into()
                                .unwrap();
                            registers.local_state
                        },
                        expected_staged_ledger_merkle_root,
                        sl.pending_coinbase_collection.clone(),
                        get_state,
                    )
                    .unwrap();

                    assert_eq!(
                        sl_of_snarked_ledger.merkle_root(),
                        sl.ledger.clone().merkle_root()
                    );
                };

                if check_snarked_ledger_transition {
                    do_snarked_ledger_transition(ledger_proof.as_ref());
                }

                if ledger_proof.is_some() {
                    proof_count += 1;
                }

                assert_fee_excess(&ledger_proof);

                let cmds_applied_this_iter = diff.commands().len();

                let cb = coinbase_count(&diff);

                match provers {
                    NumProvers::One => assert_eq!(cb, 1),
                    NumProvers::Many => assert!(cb > 0 && cb < 3, "cb={:?}", cb),
                }

                if count_opt.is_some() {
                    // There is an edge case where cmds_applied_this_iter = 0, when
                    // there is only enough space for coinbase transactions.
                    assert!(cmds_applied_this_iter <= cmds_this_iter.len());

                    let cmds = diff
                        .commands()
                        .into_iter()
                        .map(|w| w.data)
                        .collect::<Vec<_>>();
                    let cmds2 = util::take(cmds_this_iter, cmds_applied_this_iter)
                        .iter()
                        .map(|c| c.forget_check())
                        .collect::<Vec<_>>();
                    assert_eq!(cmds, cmds2);
                };

                let coinbase_cost = coinbase_cost(&diff);

                assert_ledger(
                    test_mask.clone(),
                    coinbase_cost,
                    global_slot,
                    &current_view,
                    &sl,
                    cmds_left,
                    cmds_applied_this_iter,
                    &account_ids_to_check,
                );

                eprintln!(
                    "######## Batch {} done: {} commands (base jobs) applied ########\n",
                    niters, cmds_applied_this_iter
                );

                NITERS.store(niters + 1, Relaxed);

                // println!("niters_ici={}", niters);

                // increment global slots to simulate multiple blocks
                (diff, (proof_count, global_slot.succ()))
            },
        );

        // Should have enough blocks to generate at least expected_proof_count
        // proofs
        if let Some(expected_proof_count) = expected_proof_count {
            debug_assert_eq!(total_ledger_proofs, expected_proof_count);
        };

        sl
    }

    /// Deterministically compute a prover public key from a snark work statement.
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2279
    fn stmt_to_prover(stmt: &work::Statement) -> CompressedPubKey {
        use rand::RngCore;
        use rand_pcg::Pcg64;
        use rand_seeder::Seeder;

        struct MyRng(Pcg64);

        impl RngCore for MyRng {
            fn next_u32(&mut self) -> u32 {
                self.0.next_u32()
            }

            fn next_u64(&mut self) -> u64 {
                self.0.next_u64()
            }

            fn fill_bytes(&mut self, dest: &mut [u8]) {
                self.0.fill_bytes(dest)
            }

            fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
                self.0.try_fill_bytes(dest)
            }
        }

        impl CryptoRng for MyRng {}

        let seed = stmt.fold(vec![b'P'], |mut accum, v| {
            accum.extend_from_slice(&v.target.first_pass_ledger.to_bytes());
            accum.extend_from_slice(&v.target.second_pass_ledger.to_bytes());
            accum
        });
        let rng: Pcg64 = Seeder::from(&seed).make_rng();

        Keypair::rand(&mut MyRng(rng))
            .unwrap()
            .public
            .into_compressed()
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2289
    fn proofs(stmt: &work::Statement) -> OneOrTwo<LedgerProof> {
        stmt.map(|statement| {
            LedgerProof::create(
                statement.clone(),
                SokDigest::default(),
                dummy::dummy_transaction_proof(),
            )
        })
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2295
    fn stmt_to_work_random_prover(stmt: &work::Statement) -> Option<work::Checked> {
        let mut rng = rand::thread_rng();
        // TODO: In OCaml it is "deterministic"
        let prover = Keypair::rand(&mut rng).unwrap().public.into_compressed();

        Some(work::Checked {
            fee: Fee::from_u64(CONSTRAINT_CONSTANTS.account_creation_fee),
            proofs: proofs(stmt),
            prover,
        })
    }

    /// Max throughput-ledger proof count-fixed blocks
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2636
    #[test]
    fn max_throughput_ledger_proof_count_fixed_blocks() {
        const EXPECTED_PROOF_COUNT: usize = 3;

        let (ledger_init_state, cmds, cmd_iters) =
            gen_at_capacity_fixed_blocks(EXPECTED_PROOF_COUNT);
        let global_slot = Slot::gen_small();

        async_with_ledgers(
            &ledger_init_state,
            cmds.clone(),
            cmd_iters.clone(),
            |snarked_ledger, sl, test_mask| {
                test_simple(TestSimpleParams {
                    global_slot,
                    account_ids_to_check: init_pks(&ledger_init_state),
                    cmds,
                    cmd_iters,
                    sl,
                    expected_proof_count: Some(EXPECTED_PROOF_COUNT),
                    allow_failure: None,
                    check_snarked_ledger_transition: None,
                    snarked_ledger,
                    test_mask,
                    provers: NumProvers::Many,
                    stmt_to_work: stmt_to_work_random_prover,
                })
            },
        );
    }

    /// Max throughput-ledger proof count-fixed blocks, one prover
    #[test]
    fn max_throughput_ledger_proof_count_fixed_blocks_one_prover() {
        const EXPECTED_PROOF_COUNT: usize = 3;

        let (ledger_init_state, cmds, cmd_iters) =
            gen_at_capacity_fixed_blocks(EXPECTED_PROOF_COUNT);
        let global_slot = Slot::gen_small();

        async_with_ledgers(
            &ledger_init_state,
            cmds.clone(),
            cmd_iters.clone(),
            |snarked_ledger, sl, test_mask| {
                test_simple(TestSimpleParams {
                    global_slot,
                    account_ids_to_check: init_pks(&ledger_init_state),
                    cmds,
                    cmd_iters,
                    sl,
                    expected_proof_count: Some(EXPECTED_PROOF_COUNT),
                    allow_failure: None,
                    check_snarked_ledger_transition: None,
                    snarked_ledger,
                    test_mask,
                    provers: NumProvers::One,
                    stmt_to_work: stmt_to_work_one_prover,
                })
            },
        );
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2511
    fn gen_at_capacity() -> (
        LedgerInitialState,
        Vec<valid::UserCommand>,
        Vec<Option<usize>>,
    ) {
        let mut rng = rand::thread_rng();

        let state = gen_initial_ledger_state();
        let iters = rng.gen_range(1..max_blocks_for_coverage(0));
        let total_cmds = TRANSACTION_CAPACITY * iters;

        let cmds = signed_command_sequence(total_cmds, SignKind::Real, &state);
        assert_eq!(cmds.len(), total_cmds);

        (state, cmds, vec![None; iters])
    }

    /// Max throughput
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2651
    #[test]
    fn max_throughput_normal() {
        let (ledger_init_state, cmds, cmd_iters) = gen_at_capacity();
        let global_slot = Slot::gen_small();

        async_with_ledgers(
            &ledger_init_state,
            cmds.clone(),
            cmd_iters.clone(),
            |snarked_ledger, sl, test_mask| {
                test_simple(TestSimpleParams {
                    global_slot,
                    account_ids_to_check: init_pks(&ledger_init_state),
                    cmds,
                    cmd_iters,
                    sl,
                    expected_proof_count: None,
                    allow_failure: None,
                    check_snarked_ledger_transition: None,
                    snarked_ledger,
                    test_mask,
                    provers: NumProvers::Many,
                    stmt_to_work: stmt_to_work_random_prover,
                })
            },
        );
    }

    /// Max throughput, one prover
    #[test]
    fn max_throughput_normal_one_prover() {
        let (ledger_init_state, cmds, cmd_iters) = gen_at_capacity();
        let global_slot = Slot::gen_small();

        async_with_ledgers(
            &ledger_init_state,
            cmds.clone(),
            cmd_iters.clone(),
            |snarked_ledger, sl, test_mask| {
                test_simple(TestSimpleParams {
                    global_slot,
                    account_ids_to_check: init_pks(&ledger_init_state),
                    cmds,
                    cmd_iters,
                    sl,
                    expected_proof_count: None,
                    allow_failure: None,
                    check_snarked_ledger_transition: None,
                    snarked_ledger,
                    test_mask,
                    provers: NumProvers::One,
                    stmt_to_work: stmt_to_work_one_prover,
                })
            },
        );
    }

    static VK: Lazy<WithHash<VerificationKey>> = Lazy::new(|| {
        let vk = trivial_verification_key();
        let hash = vk.hash();
        WithHash { data: vk, hash }
    });

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2525
    fn gen_zkapps(
        failure: Option<Failure>,
        num_zkapps: usize,
        zkapps_per_iter: Vec<Option<usize>>,
    ) -> (Mask, Vec<valid::UserCommand>, Vec<Option<usize>>) {
        let vk = VK.clone();
        let vk = VerificationKeyWire::with_hash(vk.data, vk.hash);

        let (zkapp_command_and_fee_payer_keypairs, ledger) = sequence_zkapp_command_with_ledger(
            None,
            Some(1),
            Some(num_zkapps),
            Some(vk),
            failure.as_ref(),
        );

        let zkapps: Vec<_> = zkapp_command_and_fee_payer_keypairs
            .into_iter()
            .map(|zkapp| {
                let (valid::UserCommand::ZkAppCommand(zkapp), _, keymap) = zkapp else {
                    panic!("Expected a Zkapp_command, got a Signed command");
                };

                let mut zkapp = zkapp.forget();
                zkapp_command_builder::replace_authorizations(None, &keymap, &mut zkapp);

                use crate::scan_state::transaction_logic::TransactionStatus::Applied;

                let valid_zkapp_command_with_auths = zkapp_command::valid::to_valid(
                    zkapp,
                    &Applied,
                    |expected_vk_hash, account_id| {
                        find_vk_via_ledger(ledger.clone(), expected_vk_hash, account_id)
                    },
                )
                .expect("Could not create Zkapp_command.Valid.t");

                valid::UserCommand::ZkAppCommand(Box::new(valid_zkapp_command_with_auths))
            })
            .collect();

        assert_eq!(zkapps.len(), num_zkapps);
        (ledger, zkapps, zkapps_per_iter)
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2571
    fn gen_zkapps_at_capacity() -> (Mask, Vec<valid::UserCommand>, Vec<Option<usize>>) {
        let mut rng = rand::thread_rng();

        let iters = rng.gen_range(1..max_blocks_for_coverage(0));
        let num_zkapps = TRANSACTION_CAPACITY * iters;
        gen_zkapps(None, num_zkapps, vec![None; iters])
    }

    /// https://github.com/MinaProtocol/mina/blob/f6756507ff7380a691516ce02a3cf7d9d32915ae/src/lib/staged_ledger/staged_ledger.ml#L2560
    fn gen_failing_zkapps_at_capacity() -> (Mask, Vec<valid::UserCommand>, Vec<Option<usize>>) {
        let mut rng = rand::thread_rng();

        let iters = rng.gen_range(1..max_blocks_for_coverage(0));
        let num_zkapps = TRANSACTION_CAPACITY * iters;
        gen_zkapps(
            Some(Failure::InvalidAccountPrecondition),
            num_zkapps,
            vec![None; iters],
        )
    }

    fn gen_zkapps_at_capacity_fixed_blocks(
        extra_block_count: usize,
    ) -> (Mask, Vec<valid::UserCommand>, Vec<Option<usize>>) {
        let iters = max_blocks_for_coverage(extra_block_count);
        let num_zkapps = TRANSACTION_CAPACITY * iters;
        gen_zkapps(None, num_zkapps, vec![None; iters])
    }

    #[test]
    fn de_serialize_zkapps() {
        let (_ledger, zkapps, _iters) = gen_zkapps_at_capacity();

        for (index, zkapp) in zkapps.into_iter().enumerate() {
            let zkapp = match zkapp {
                valid::UserCommand::SignedCommand(_) => todo!(),
                valid::UserCommand::ZkAppCommand(zkapp) => zkapp,
            };

            let zkapp = zkapp.forget();

            let a: mina_p2p_messages::v2::MinaBaseZkappCommandTStableV1WireStableV1 =
                (&zkapp).into();
            let b: zkapp_command::ZkAppCommand = (&a).try_into().unwrap();
            b.account_updates.accumulate_hashes();

            assert_eq!(zkapp, b, "failed at {:?}", index);
        }
    }

    /// Max throughput (zkapps)
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2664
    // #[test]
    fn max_throughput_zkapps() {
        let (ledger, zkapps, cmd_iters) = gen_zkapps_at_capacity();
        let global_slot = Slot::gen_small();

        async_with_given_ledger(
            &LedgerInitialState { state: vec![] },
            zkapps.clone(),
            cmd_iters.clone(),
            ledger.clone(),
            |snarked_ledger, sl, test_mask| {
                let account_ids: Vec<_> = ledger.accounts().into_iter().collect();

                test_simple(TestSimpleParams {
                    global_slot,
                    account_ids_to_check: account_ids,
                    cmds: zkapps,
                    cmd_iters,
                    sl,
                    expected_proof_count: None,
                    allow_failure: None,
                    check_snarked_ledger_transition: None,
                    snarked_ledger,
                    test_mask,
                    provers: NumProvers::Many,
                    stmt_to_work: stmt_to_work_random_prover,
                })
            },
        );
    }

    /// Max_throughput with zkApp transactions that may fail
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2675
    // #[test]
    fn max_throughput_zkapps_that_may_fail() {
        let (ledger, zkapps, cmd_iters) = gen_failing_zkapps_at_capacity();
        let global_slot = Slot::gen_small();

        async_with_given_ledger(
            &LedgerInitialState { state: vec![] },
            zkapps.clone(),
            cmd_iters.clone(),
            ledger.clone(),
            |snarked_ledger, sl, test_mask| {
                let account_ids: Vec<_> = ledger.accounts().into_iter().collect();

                test_simple(TestSimpleParams {
                    global_slot,
                    account_ids_to_check: account_ids,
                    cmds: zkapps,
                    cmd_iters,
                    sl,
                    expected_proof_count: None,
                    allow_failure: Some(true),
                    check_snarked_ledger_transition: None,
                    snarked_ledger,
                    test_mask,
                    provers: NumProvers::Many,
                    stmt_to_work: stmt_to_work_random_prover,
                })
            },
        );
    }

    /// Max throughput-ledger proof count-fixed blocks (zkApps)
    // #[test]
    fn max_throughput_ledger_proof_count_fixed_blocks_zkapp() {
        const EXPECTED_PROOF_COUNT: usize = 3;

        let now = redux::Instant::now();

        let (ledger, zkapps, cmd_iters) = gen_zkapps_at_capacity_fixed_blocks(EXPECTED_PROOF_COUNT);
        let global_slot = Slot::gen_small();

        eprintln!("generated in {:?}", now.elapsed());

        async_with_given_ledger(
            &LedgerInitialState { state: vec![] },
            zkapps.clone(),
            cmd_iters.clone(),
            ledger.clone(),
            |snarked_ledger, sl, test_mask| {
                let account_ids: Vec<_> = ledger.accounts().into_iter().collect();

                test_simple(TestSimpleParams {
                    global_slot,
                    account_ids_to_check: account_ids,
                    cmds: zkapps,
                    cmd_iters,
                    sl,
                    expected_proof_count: Some(EXPECTED_PROOF_COUNT),
                    allow_failure: None,
                    check_snarked_ledger_transition: Some(true),
                    snarked_ledger,
                    test_mask,
                    provers: NumProvers::Many,
                    stmt_to_work: stmt_to_work_random_prover,
                })
            },
        );
    }

    /// Random number of commands (zkapp + signed command)
    // #[test]
    fn random_number_of_commands_zkapps_plus_signed() {
        let (ledger, cmds, cmd_iters) = gen_all_user_commands_below_capacity();
        let global_slot = Slot::gen_small();

        async_with_given_ledger(
            &LedgerInitialState { state: vec![] },
            cmds.clone(),
            cmd_iters.clone(),
            ledger.clone(),
            |snarked_ledger, sl, test_mask| {
                let account_ids: Vec<_> = ledger.accounts().into_iter().collect();

                test_simple(TestSimpleParams {
                    global_slot,
                    account_ids_to_check: account_ids,
                    cmds,
                    cmd_iters,
                    sl,
                    expected_proof_count: None,
                    allow_failure: None,
                    check_snarked_ledger_transition: Some(true),
                    snarked_ledger,
                    test_mask,
                    provers: NumProvers::Many,
                    stmt_to_work: stmt_to_work_random_prover,
                })
            },
        );
    }

    /// Generator for when we have less commands than needed to fill all slots.
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2612
    fn gen_below_capacity(
        extra_blocks: Option<bool>,
    ) -> (
        LedgerInitialState,
        Vec<valid::UserCommand>,
        Vec<Option<usize>>,
    ) {
        let extra_blocks = extra_blocks.unwrap_or(false);

        let mut rng = rand::thread_rng();

        let state = gen_initial_ledger_state();
        let iters_max = max_blocks_for_coverage(0) * if extra_blocks { 4 } else { 2 };
        let iters_min = max_blocks_for_coverage(0);

        let iters = rng.gen_range(iters_min..=iters_max);

        // N.B. user commands per block is much less than transactions per block
        // due to fee transfers and coinbases, especially with worse case number
        // of provers, so in order to exercise not filling the scan state
        // completely we always apply <= 1/2 transaction_capacity commands.

        let cmds_per_iter: Vec<usize> = (0..iters)
            .map(|_| rng.gen_range(1..((TRANSACTION_CAPACITY / 2) - 1)))
            .collect();

        let total_cmds = cmds_per_iter.iter().sum();

        let cmds = signed_command_sequence(total_cmds, SignKind::Real, &state);
        assert_eq!(cmds.len(), total_cmds);

        (state, cmds, cmds_per_iter.into_iter().map(Some).collect())
    }

    fn gen_all_user_commands_below_capacity() -> (Mask, Vec<valid::UserCommand>, Vec<Option<usize>>)
    {
        let (mut ledger, zkapps, iters_zkapps) = gen_zkapps_below_capacity(None);
        let (ledger_init_state, signed_cmds, iters_signed_commands) = gen_below_capacity(None);

        apply_initialize_ledger_state(&mut ledger, &ledger_init_state);

        let iters: Vec<_> = iters_zkapps
            .into_iter()
            .chain(iters_signed_commands)
            .collect();

        let mut cmds = Vec::with_capacity(zkapps.len() + signed_cmds.len());

        let mut zkapps = zkapps.into_iter().peekable();
        let mut payments = signed_cmds.into_iter().peekable();

        let mut rng = rand::thread_rng();

        loop {
            match (zkapps.peek(), payments.peek()) {
                (None, None) => break,
                (None, Some(_)) => {
                    cmds.push(payments.next().unwrap());
                }
                (Some(_), None) => {
                    cmds.push(zkapps.next().unwrap());
                }
                (Some(_), Some(_)) => {
                    let n = rng.gen_range(1..TRANSACTION_CAPACITY);
                    let take_zkapps: bool = rng.gen();

                    let iter = if take_zkapps {
                        &mut zkapps
                    } else {
                        &mut payments
                    };

                    cmds.extend(iter.take(n));
                }
            }
        }

        (ledger, cmds, iters)
    }

    /// Be able to include random number of commands
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2686
    #[test]
    fn be_able_to_include_random_number_of_commands_many_normal() {
        let (ledger_init_state, cmds, cmd_iters) = gen_below_capacity(None);
        let global_slot = Slot::gen_small();

        async_with_ledgers(
            &ledger_init_state,
            cmds.to_vec(),
            cmd_iters.to_vec(),
            |snarked_ledger, sl, test_mask| {
                test_simple(TestSimpleParams {
                    global_slot,
                    account_ids_to_check: init_pks(&ledger_init_state),
                    cmds,
                    cmd_iters,
                    sl,
                    expected_proof_count: None,
                    allow_failure: None,
                    check_snarked_ledger_transition: None,
                    snarked_ledger,
                    test_mask,
                    provers: NumProvers::Many,
                    stmt_to_work: stmt_to_work_random_prover,
                })
            },
        );
    }

    /// Generate states that were known to fail
    ///
    /// See https://github.com/openmina/ledger/commit/6de803f082ea986aa71e3cf30d7d83e54d2f5a3e
    fn gen_below_capacity_failed() -> (
        LedgerInitialState,
        Vec<valid::UserCommand>,
        Vec<Option<usize>>,
    ) {
        let state = gen_initial_ledger_state();
        // let iters = [
        //     7, 17, 26, 35, 50, 13, 54, 12, 29, 54, 62, 36, 44, 44, 7, 8, 25, 8, 3, 42, 4, 46, 61,
        //     6, 60, 24, 34, 39, 9, 58, 23, 34, 10, 22, 15, 8, 4, 1, 42, 25, 5, 17, 60, 49, 45,
        // ];

        // // 2 slots availables
        // let iters = [
        //     124, 17, 80, 80
        // ];

        // let iters = vec![126; 25]
        //     .into_iter()
        //     .chain([
        //         62, 17, 100, // 124, 17
        //     ])
        //     .collect::<Vec<_>>();

        // let mut rng = rand::thread_rng();
        // let iters: Vec<_> = (1..1024).map(|_| {
        //     rng.gen_range(1..63)
        // }).collect();

        // // Failed with AAAA 1/2 (random)
        // let iters = [
        //     15,
        //     6,
        //     24,
        //     10,
        //     7,
        //     12,
        //     26,
        //     4,
        //     7,
        //     57,
        //     23,
        //     59,
        //     52,
        //     35,
        //     5,
        //     12,
        //     33,
        //     12,
        //     49,
        //     29,
        //     35,
        //     37,
        //     23,
        //     33,
        //     28,
        //     38,
        //     16,
        //     10,
        // ];

        // Failed with AAAA2 (one_prover)
        // let iters = [57, 16, 3, 16, 61, 26, 15, 21, 7, 34, 1, 52, 21, 29, 50, 40, 25];

        // let iters = [
        //     121, 4
        // ];

        // panic at incr coinbase -> two
        let iters = [
            12, 40, 10, 13, 1, 25, 3, 20, 41, 16, 30, 37, 26, 47, 33, 45, 44, 62, 18, 24, 55, 10,
            53, 25, 19, 35, 44, 54, 60, 62, 32, 48, 31, 10, 20, 32, 57, 48, 37, 38,
        ];

        let total_cmds = iters.iter().sum();
        eprintln!("total_cmds={:?}", total_cmds);

        let cmds = signed_command_sequence(total_cmds, SignKind::Real, &state);
        assert_eq!(cmds.len(), total_cmds);

        (state, cmds, iters.into_iter().map(Some).collect())
    }

    /// This test was failing, due to incorrect discarding user command
    /// Note: Something interesting is that the batch 11 applies 0 commands
    ///
    /// See https://github.com/openmina/ledger/commit/6de803f082ea986aa71e3cf30d7d83e54d2f5a3e
    #[test]
    fn be_able_to_include_random_number_of_commands_many_failed() {
        let (ledger_init_state, cmds, cmd_iters) = gen_below_capacity_failed();
        let global_slot = Slot::gen_small();

        async_with_ledgers(
            &ledger_init_state,
            cmds.to_vec(),
            cmd_iters.to_vec(),
            |snarked_ledger, sl, test_mask| {
                test_simple(TestSimpleParams {
                    global_slot,
                    account_ids_to_check: init_pks(&ledger_init_state),
                    cmds,
                    cmd_iters,
                    sl,
                    expected_proof_count: None,
                    allow_failure: None,
                    check_snarked_ledger_transition: None,
                    snarked_ledger,
                    test_mask,
                    provers: NumProvers::Many,
                    stmt_to_work: stmt_to_work_random_prover,
                })
            },
        );
    }

    // Deterministic, to get staged ledger hash
    fn gen_for_hash(
        iters: &[usize],
        mut cmds: Vec<valid::UserCommand>,
    ) -> (
        LedgerInitialState,
        Vec<valid::UserCommand>,
        Vec<Option<usize>>,
    ) {
        fn keypair_from_private(private: &str) -> Keypair {
            let bytes = bs58::decode(private).into_vec().unwrap();
            let bytes = &bytes[1..]; // ignore base58 check byte

            let secret = mina_signer::ScalarField::from_bytes(&bytes[1..]).unwrap();
            let public: mina_signer::CurvePoint =
                mina_signer::CurvePoint::prime_subgroup_generator()
                    .mul(secret)
                    .into_affine();

            if !public.is_on_curve() {
                panic!()
                // return Err(KeypairError::NonCurvePoint);
            }

            // Safe now because we checked point is on the curve
            Keypair::from_parts_unsafe(secret, public)
        }

        //        ledger_init_state=((((public_key B62qqrHu7qJJrUekPYqNEbsMMzxDebqfApuyT5y6K9xgwm4TUe77kNd)
        //   (private_key EKDpdyjwhn5PWZzz2EumvUTDVtRKdy5QeP96i2iFntzhCQK5M8uU))
        //  870234598000000000 555 Untimed)
        // (((public_key B62qnxPe7DM72bh59QrubnREEyeNoeLM4J9s8iufT6Gi2iuUm6fe73R)
        //   (private_key EKEVz18GAQ4zJaEHXRHrzbKYh5G4gzy1Kf4Y4x8MHuJN7pv1jmGx))
        //  677094385000000000 289 Untimed)
        // (((public_key B62qq4FFooVJ6TRGKA3chxE7M1Xh1F11jsanLBYmcZEqJZaYGPdye3D)
        //   (private_key EKEkFGPRpiXfAfrgRi9DUSQ3zQ5eV4FGaD9rHLJwTBz7Ue4jWMnU))
        //  966785966000000000 697 Untimed)
        // (((public_key B62qiuynJSwKPepZGm8fcYbZ3zT2nynjcM23CD1Xzpofy5yKwMaC5N7)
        //   (private_key EKFPQBAbjYkjM6p6fEaZAzufQgQs3spvUw1Uyq2Ghta81cpKrfGg))
        //  871707103000000000 387 Untimed)
        // (((public_key B62qrpPoKmGZWcWMn1Cb749dSym7cabp1wwoUS7AThuaCNJCfXHCLNw)
        //   (private_key EKENSzF3YWq2Z3Rh9GXMwqN6bEVps9HxkfU38LkXfR3s2Lf3VPcD))
        //  955060948000000000 468 Untimed))

        let state = LedgerInitialState {
            state: vec![
                (
                    keypair_from_private("EKDpdyjwhn5PWZzz2EumvUTDVtRKdy5QeP96i2iFntzhCQK5M8uU"),
                    Amount::from_u64(870234598000000000),
                    Nonce::from_u32(555),
                    crate::Timing::Untimed,
                ),
                (
                    keypair_from_private("EKEVz18GAQ4zJaEHXRHrzbKYh5G4gzy1Kf4Y4x8MHuJN7pv1jmGx"),
                    Amount::from_u64(677094385000000000),
                    Nonce::from_u32(289),
                    crate::Timing::Untimed,
                ),
                (
                    keypair_from_private("EKEkFGPRpiXfAfrgRi9DUSQ3zQ5eV4FGaD9rHLJwTBz7Ue4jWMnU"),
                    Amount::from_u64(966785966000000000),
                    Nonce::from_u32(697),
                    crate::Timing::Untimed,
                ),
                (
                    keypair_from_private("EKFPQBAbjYkjM6p6fEaZAzufQgQs3spvUw1Uyq2Ghta81cpKrfGg"),
                    Amount::from_u64(871707103000000000),
                    Nonce::from_u32(387),
                    crate::Timing::Untimed,
                ),
                (
                    keypair_from_private("EKENSzF3YWq2Z3Rh9GXMwqN6bEVps9HxkfU38LkXfR3s2Lf3VPcD"),
                    Amount::from_u64(955060948000000000),
                    Nonce::from_u32(468),
                    crate::Timing::Untimed,
                ),
            ],
        };

        // let state = gen_initial_ledger_state();
        println!("state={:#?}", state);

        let iters_total: usize = iters.iter().sum();
        assert!(iters_total <= cmds.len());

        cmds.truncate(iters_total);

        (state, cmds, iters.iter().copied().map(Some).collect())
    }

    fn test_hash(
        cmds: Vec<valid::UserCommand>,
        iters: &[usize],
        expected_hash: &StagedLedgerHash<Fp>,
    ) {
        let (ledger_init_state, cmds, cmd_iters) = gen_for_hash(iters, cmds);
        let global_slot = Slot::gen_small();

        async_with_ledgers(
            &ledger_init_state,
            cmds.to_vec(),
            cmd_iters.to_vec(),
            |snarked_ledger, mut sl, test_mask| {
                // dbg!(sl.ledger.num_accounts());

                let hash = sl.hash();

                assert_eq!(hash, StagedLedgerHash::from_ocaml_strings(
                    "7213023165825031994332898585791275635753820608093286100176380057570051468967",
                    r"T\249\245k\176]TJ\216\183\001\204\177\131\030\244o\178\188\191US\156\192Hi\194P\223\004\000\003",
                    r"_\236\235f\255\200o8\217Rxlmily\194\219\1949\221N\145\180g)\215:'\251W\233",
                    "25504365445533103805898245102289650498571312278321176071043666991586378788150"
                ));

                let mut sl = test_simple(TestSimpleParams {
                    global_slot,
                    account_ids_to_check: init_pks(&ledger_init_state),
                    cmds,
                    cmd_iters,
                    sl,
                    expected_proof_count: None,
                    allow_failure: None,
                    check_snarked_ledger_transition: None,
                    snarked_ledger,
                    test_mask,
                    provers: NumProvers::One,
                    stmt_to_work: stmt_to_work_one_prover,
                });

                let mut accounts = sl.ledger.accounts().into_iter().collect::<Vec<_>>();
                accounts.sort_by_key(|a| a.public_key.x);

                let hash = sl.hash();

                let job: Vec<_> = sl.scan_state.base_jobs_on_latest_tree().collect();
                dbg!(job.len(), &job);

                for job in job {
                    // 1st
                    let before = job.first_pass_ledger_witness.clone().merkle_root();
                    let ledger: mina_p2p_messages::v2::MinaBaseSparseLedgerBaseStableV2 =
                        (&job.first_pass_ledger_witness).into();

                    let mut ledger: SparseLedger = (&ledger).try_into().unwrap();
                    let after = ledger.merkle_root();
                    assert_eq!(before, after);
                    assert_eq!(ledger, job.first_pass_ledger_witness);

                    // 2nd
                    let before = job.second_pass_ledger_witness.clone().merkle_root();
                    let ledger: mina_p2p_messages::v2::MinaBaseSparseLedgerBaseStableV2 =
                        (&job.second_pass_ledger_witness).into();

                    let mut ledger: SparseLedger = (&ledger).try_into().unwrap();
                    let after = ledger.merkle_root();
                    assert_eq!(before, after);
                    assert_eq!(ledger, job.second_pass_ledger_witness);
                }

                assert_eq!(
                    &hash, expected_hash,
                    "\ngot={:#?}\nexpected={:#?}",
                    hash, expected_hash
                );

                // staged_ledger_hash=
                // ((non_snark
                //   ((ledger_hash
                //     7403973954047799970700856317480467373315236045518635088954201291392464592256)
                //    (aux_hash
                //     "\183At\174\178]\186\b$\182\245&\003=\183\241\190\214\131@r\162&\138f\187\234\191\2002\148\249")
                //    (pending_coinbase_aux
                //     "\147\141\184\201\248,\140\181\141?>\244\253%\0006\164\141&\167\018u=/\222Z\189\003\168\\\171\244")))
                //  (pending_coinbase_hash
                //   3086764415430464582862741061906779337283239657859408605572481484787647764860))
            },
        );
    }

    // #[test]
    fn staged_ledger_hash() {
        let cmds = vec![valid::UserCommand::SignedCommand(Box::new(SignedCommand {
            payload: SignedCommandPayload {
                common: Common {
                    fee: Fee::from_u64(8688709898),
                    fee_payer_pk: CompressedPubKey::from_address(
                        "B62qqrHu7qJJrUekPYqNEbsMMzxDebqfApuyT5y6K9xgwm4TUe77kNd",
                    )
                    .unwrap(),
                    nonce: Nonce::from_u32(555),
                    valid_until: Slot::from_u32(4294967295),
                    memo: Memo::from_ocaml_str(
                        r"\000 \014WQ\192&\229C\178\232\171.\176`\153\218\161\209\229\223Gw\143w\135\250\171E\205\241/\227\168",
                    ),
                },
                body: signed_command::Body::Payment(PaymentPayload {
                    receiver_pk: CompressedPubKey::from_address(
                        "B62qnxPe7DM72bh59QrubnREEyeNoeLM4J9s8iufT6Gi2iuUm6fe73R",
                    )
                    .unwrap(),
                    amount: Amount::from_u64(435117290311290102),
                }),
            },
            signer: CompressedPubKey::from_address(
                "B62qqrHu7qJJrUekPYqNEbsMMzxDebqfApuyT5y6K9xgwm4TUe77kNd",
            )
            .unwrap(),
            signature: Signature {
                rx: Fp::from_str(
                    "6619317331104517676070771956470715552778579643404520621914362882786941822698",
                )
                .unwrap(),
                s: Fq::from_str(
                    "4273977355509408506621406872367289068113165398006813670168359703884853847009",
                )
                .unwrap(),
            },
        }))];

        let iters = [1];
        let expected = StagedLedgerHash::from_ocaml_strings(
            "7403973954047799970700856317480467373315236045518635088954201291392464592256",
            r"\183At\174\178]\186\b$\182\245&\003=\183\241\190\214\131@r\162&\138f\187\234\191\2002\148\249",
            r"\147\141\184\201\248,\140\181\141?>\244\253%\0006\164\141&\167\018u=/\222Z\189\003\168\\\171\244",
            "3086764415430464582862741061906779337283239657859408605572481484787647764860",
        );

        test_hash(cmds, &iters[..], &expected);

        let cmds = vec![
            valid::UserCommand::SignedCommand(Box::new(SignedCommand {
                payload: SignedCommandPayload {
                    common: Common {
                        fee: Fee::from_u64(9552806101),
                        fee_payer_pk: CompressedPubKey::from_address(
                            "B62qqrHu7qJJrUekPYqNEbsMMzxDebqfApuyT5y6K9xgwm4TUe77kNd",
                        )
                            .unwrap(),
                        nonce: Nonce::from_u32(555),
                        valid_until: Slot::from_u32(4294967295),
                        memo: Memo::from_ocaml_str(
                            r"\000 \014WQ\192&\229C\178\232\171.\176`\153\218\161\209\229\223Gw\143w\135\250\171E\205\241/\227\168",
                        ),
                    },
                    body: signed_command::Body::Payment(PaymentPayload {
                        receiver_pk: CompressedPubKey::from_address(
                            "B62qq4FFooVJ6TRGKA3chxE7M1Xh1F11jsanLBYmcZEqJZaYGPdye3D",
                        )
                            .unwrap(),
                        amount: Amount::from_u64(469201635493516843),
                    }),
                },
                signer: CompressedPubKey::from_address(
                    "B62qqrHu7qJJrUekPYqNEbsMMzxDebqfApuyT5y6K9xgwm4TUe77kNd",
                )
                    .unwrap(),
                signature: Signature {
                    rx: Fp::from_str(
                        "13044376504321844748116412242934215884445305434730105658311275966836245013702",
                    )
                        .unwrap(),
                    s: Fq::from_str(
                        "12273578804512625613925108666574978157352049391462385103152803971242598973329",
                    )
                        .unwrap(),
                },
            })),
            valid::UserCommand::SignedCommand(Box::new(SignedCommand {
                payload: SignedCommandPayload {
                    common: Common {
                        fee: Fee::from_u64(9575291918),
                        fee_payer_pk: CompressedPubKey::from_address(
                            "B62qqrHu7qJJrUekPYqNEbsMMzxDebqfApuyT5y6K9xgwm4TUe77kNd",
                        )
                            .unwrap(),
                        nonce: Nonce::from_u32(556),
                        valid_until: Slot::from_u32(4294967295),
                        memo: Memo::from_ocaml_str(
                            r"\000 \014WQ\192&\229C\178\232\171.\176`\153\218\161\209\229\223Gw\143w\135\250\171E\205\241/\227\168",
                        ),
                    },
                    body: signed_command::Body::Payment(PaymentPayload {
                        receiver_pk: CompressedPubKey::from_address(
                            "B62qqrHu7qJJrUekPYqNEbsMMzxDebqfApuyT5y6K9xgwm4TUe77kNd",
                        )
                            .unwrap(),
                        amount: Amount::from_u64(296784327960059186),
                    }),
                },
                signer: CompressedPubKey::from_address(
                    "B62qqrHu7qJJrUekPYqNEbsMMzxDebqfApuyT5y6K9xgwm4TUe77kNd",
                )
                    .unwrap(),
                signature: Signature {
                    rx: Fp::from_str(
                        "19459175650095724075239264136789847339652723660248405017158921230047021100311",
                    )
                        .unwrap(),
                    s: Fq::from_str(
                        "10619126644528613945289745005934480034669523474364234529309635443435990216318",
                    )
                        .unwrap(),
                },
            })),
            valid::UserCommand::SignedCommand(Box::new(SignedCommand {
                payload: SignedCommandPayload {
                    common: Common {
                        fee: Fee::from_u64(6000000000),
                        fee_payer_pk: CompressedPubKey::from_address(
                            "B62qqrHu7qJJrUekPYqNEbsMMzxDebqfApuyT5y6K9xgwm4TUe77kNd",
                        )
                            .unwrap(),
                        nonce: Nonce::from_u32(557),
                        valid_until: Slot::from_u32(4294967295),
                        memo: Memo::from_ocaml_str(
                            r"\000 \014WQ\192&\229C\178\232\171.\176`\153\218\161\209\229\223Gw\143w\135\250\171E\205\241/\227\168",
                        ),
                    },
                    body: signed_command::Body::Payment(PaymentPayload {
                        receiver_pk: CompressedPubKey::from_address(
                            "B62qnxPe7DM72bh59QrubnREEyeNoeLM4J9s8iufT6Gi2iuUm6fe73R",
                        )
                            .unwrap(),
                        amount: Amount::from_u64(104248609418325952),
                    }),
                },
                signer: CompressedPubKey::from_address(
                    "B62qqrHu7qJJrUekPYqNEbsMMzxDebqfApuyT5y6K9xgwm4TUe77kNd",
                )
                    .unwrap(),
                signature: Signature {
                    rx: Fp::from_str(
                        "19184633593539053772878146013599475038165210841217779716455315092747869436447",
                    )
                        .unwrap(),
                    s: Fq::from_str(
                        "17897057128192645856092038224382359168869279402582452391679534470779618649008",
                    )
                        .unwrap(),
                },
            }))
        ];

        let iters = [1, 2];
        let expected = StagedLedgerHash::from_ocaml_strings(
            "21222488975521454969719816807666201571945933237784599135716531110886279420130",
            r"C\004\240\167\1597Y\205\236br\128\2324<\219[&\027u\173\152\212\180N\142\193yP\189\129\031",
            r"\147\141\184\201\248,\140\181\141?>\244\253%\0006\164\141&\167\018u=/\222Z\189\003\168\\\171\244",
            "8909119019222126918708891766510490679262830386343981426610706812406879559705",
        );

        // staged_ledger_hash=
        // ((non_snark
        //   ((ledger_hash
        //     21222488975521454969719816807666201571945933237784599135716531110886279420130)
        //    (aux_hash
        //     "C\004\240\167\1597Y\205\236br\128\2324<\219[&\027u\173\152\212\180N\142\193yP\189\129\031")
        //    (pending_coinbase_aux
        //     "\147\141\184\201\248,\140\181\141?>\244\253%\0006\164\141&\167\018u=/\222Z\189\003\168\\\171\244")))
        //  (pending_coinbase_hash
        //   8909119019222126918708891766510490679262830386343981426610706812406879559705))

        test_hash(cmds, &iters[..], &expected);

        let cmds = dummy::for_tests::list_of_cmds();
        let iters = [126, 1];

        let expected = StagedLedgerHash::from_ocaml_strings(
            "17477048617623399278357380851139583927261600258225227089274676141994175991491",
            r"\031\154\249\228\236\218\178\144\220\147|8\217p3\158ivC\192\129\208>\t\2402\n\232\225\004\172\204",
            r"7\131O/%v/#\225\247JS\028\190D]\183=ge\235\230\bx\167\223\190\205}J\246\225",
            "13628016176671996634125618970711117372294565467015081467173689084579176689763",
        );

        test_hash(cmds.clone(), &iters[..], &expected);

        let iters = [126, 126, 126, 1];

        let expected = StagedLedgerHash::from_ocaml_strings(
            "3849036020651863715306758036604903453092892010402969196144652234888763588784",
            r"l\134Q\000\138DI\190\215\219BqR\142\201#\187\219e\138\210\142*\210\206E\195Q\221Un3",
            r"\245;\157@E\202~a\226R\155\006&\201)\030&[\238\156\027w&\204_\150G<U;\1803",
            "25017623412619027645642100741662603532612211443127692037797169213370969546910",
        );

        test_hash(cmds.clone(), &iters[..], &expected);

        let iters = [126, 126, 126, 126, 126, 126, 126];
        let expected = StagedLedgerHash::from_ocaml_strings(
            "18582860218764414485081234471609377222894570081548691702645303871998665679024",
            r"0\136Wg\182DbX\203kLi\212%\199\206\142#\213`L\160bpCB\1413\240\193\171K",
            r"\n\220\211\153\014A\191\006\019\231/\244\155\005\212\1310|\227\133\176O\196\131\023t\152\178\130?\206U",
            "7755910003612203694232340741198062502757785525513434577565209492737983651491",
        );

        test_hash(cmds, &iters[..], &expected);
    }

    /// https://github.com/MinaProtocol/mina/blob/f6756507ff7380a691516ce02a3cf7d9d32915ae/src/lib/staged_ledger/staged_ledger.ml#L2579
    fn gen_zkapps_below_capacity(
        extra_blocks: Option<bool>,
    ) -> (Mask, Vec<valid::UserCommand>, Vec<Option<usize>>) {
        let extra_blocks = extra_blocks.unwrap_or(false);
        let mut rng = rand::thread_rng();

        let iters_max = max_blocks_for_coverage(0) * if extra_blocks { 4 } else { 2 };
        let iters_min = max_blocks_for_coverage(0);

        let iters = rng.gen_range(iters_min..iters_max);

        // see comment in gen_below_capacity for rationale

        let zkapps_per_iter: Vec<usize> = (0..iters)
            .map(|_| rng.gen_range(1..((TRANSACTION_CAPACITY / 2) - 1)))
            .collect();

        let num_zkapps: usize = zkapps_per_iter.iter().sum();
        gen_zkapps(
            None,
            num_zkapps,
            zkapps_per_iter.into_iter().map(Some).collect(),
        )
    }

    /// Be able to include random number of commands (zkapps)
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2694
    // #[test]
    fn be_able_to_include_random_number_of_commands_zkapps() {
        let (ledger, zkapps, cmd_iters) = gen_zkapps_below_capacity(None);
        let global_slot = Slot::gen_small();

        async_with_given_ledger(
            &LedgerInitialState { state: vec![] },
            zkapps.clone(),
            cmd_iters.clone(),
            ledger.clone(),
            |snarked_ledger, sl, test_mask| {
                let account_ids: Vec<_> = ledger.accounts().into_iter().collect();

                test_simple(TestSimpleParams {
                    global_slot,
                    account_ids_to_check: account_ids,
                    cmds: zkapps,
                    cmd_iters,
                    sl,
                    expected_proof_count: None,
                    allow_failure: None,
                    check_snarked_ledger_transition: None,
                    snarked_ledger,
                    test_mask,
                    provers: NumProvers::Many,
                    stmt_to_work: stmt_to_work_random_prover,
                })
            },
        );
    }

    /// Be able to include random number of commands (One prover)
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2704
    #[test]
    fn be_able_to_include_random_number_of_commands_one_prover_normal() {
        let (ledger_init_state, cmds, cmd_iters) = gen_below_capacity(None);
        let global_slot = Slot::gen_small();

        async_with_ledgers(
            &ledger_init_state,
            cmds.to_vec(),
            cmd_iters.to_vec(),
            |snarked_ledger, sl, test_mask| {
                test_simple(TestSimpleParams {
                    global_slot,
                    account_ids_to_check: init_pks(&ledger_init_state),
                    cmds,
                    cmd_iters,
                    sl,
                    expected_proof_count: None,
                    allow_failure: None,
                    check_snarked_ledger_transition: None,
                    snarked_ledger,
                    test_mask,
                    provers: NumProvers::One,
                    stmt_to_work: stmt_to_work_one_prover,
                })
            },
        );
    }

    /// This test was failing, due to incorrect discarding user command
    ///
    /// See https://github.com/openmina/ledger/commit/6de803f082ea986aa71e3cf30d7d83e54d2f5a3e
    #[test]
    fn be_able_to_include_random_number_of_commands_one_prover_failed() {
        let (ledger_init_state, cmds, cmd_iters) = gen_below_capacity_failed();
        let global_slot = Slot::gen_small();

        async_with_ledgers(
            &ledger_init_state,
            cmds.to_vec(),
            cmd_iters.to_vec(),
            |snarked_ledger, sl, test_mask| {
                test_simple(TestSimpleParams {
                    global_slot,
                    account_ids_to_check: init_pks(&ledger_init_state),
                    cmds,
                    cmd_iters,
                    sl,
                    expected_proof_count: None,
                    allow_failure: None,
                    check_snarked_ledger_transition: None,
                    snarked_ledger,
                    test_mask,
                    provers: NumProvers::One,
                    stmt_to_work: stmt_to_work_one_prover,
                })
            },
        );
    }

    /// Be able to include random number of commands (One prover, zkapps)
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2712
    // #[test]
    fn be_able_to_include_random_number_of_commands_one_prover_zkapps() {
        let (ledger, zkapps, cmd_iters) = gen_zkapps_below_capacity(Some(true));
        let global_slot = Slot::gen_small();

        async_with_given_ledger(
            &LedgerInitialState { state: vec![] },
            zkapps.clone(),
            cmd_iters.clone(),
            ledger.clone(),
            |snarked_ledger, sl, test_mask| {
                let account_ids: Vec<_> = ledger.accounts().into_iter().collect();

                test_simple(TestSimpleParams {
                    global_slot,
                    account_ids_to_check: account_ids,
                    cmds: zkapps,
                    cmd_iters,
                    sl,
                    expected_proof_count: None,
                    allow_failure: None,
                    check_snarked_ledger_transition: Some(true),
                    snarked_ledger,
                    test_mask,
                    provers: NumProvers::One,
                    stmt_to_work: stmt_to_work_one_prover,
                })
            },
        );
    }

    /// Fixed public key for when there is only one snark worker.
    static SNARK_WORKER_PK: Lazy<CompressedPubKey> = Lazy::new(|| {
        CompressedPubKey::from_address("B62qkEfRowNNxqpA4KZX5FsWu3EDa15SYyxkjC3KvxqKVPbpQZyLofw")
            .unwrap()
    });
    // Lazy::new(|| gen_keypair().public.into_compressed());

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2295
    fn stmt_to_work_one_prover(stmt: &work::Statement) -> Option<work::Checked> {
        Some(work::Checked {
            fee: WORK_FEE,
            proofs: proofs(stmt),
            prover: SNARK_WORKER_PK.clone(),
        })
    }

    /// Zero proof-fee should not create a fee transfer
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2723
    #[test]
    fn zero_proof_fee_should_not_create_a_fee_transfer() {
        const EXPECTED_PROOF_COUNT: usize = 3;

        let stmt_to_work_zero_fee = |stmts: &OneOrTwo<Statement<()>>| {
            Some(work::Checked {
                fee: Fee::zero(),
                proofs: proofs(stmts),
                prover: SNARK_WORKER_PK.clone(),
            })
        };

        let account_id_prover = AccountId::new(SNARK_WORKER_PK.clone(), TokenId::default());

        let (ledger_init_state, cmds, cmd_iters) =
            gen_at_capacity_fixed_blocks(EXPECTED_PROOF_COUNT);
        let global_slot = Slot::gen_small();

        async_with_ledgers(
            &ledger_init_state,
            cmds.clone(),
            cmd_iters.clone(),
            |snarked_ledger, sl, test_mask| {
                test_simple(TestSimpleParams {
                    global_slot,
                    account_ids_to_check: init_pks(&ledger_init_state),
                    cmds,
                    cmd_iters,
                    sl,
                    expected_proof_count: Some(EXPECTED_PROOF_COUNT),
                    allow_failure: None,
                    check_snarked_ledger_transition: None,
                    snarked_ledger,
                    test_mask: test_mask.clone(),
                    provers: NumProvers::One,
                    stmt_to_work: stmt_to_work_zero_fee,
                });

                assert!(test_mask.location_of_account(&account_id_prover).is_none());
            },
        );
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2745
    fn compute_statutes(
        ledger: Mask,
        coinbase_amount: Amount,
        global_slot: Slot,
        diff: (
            PreDiffTwo<work::Work, valid::UserCommand>,
            Option<PreDiffOne<work::Work, valid::UserCommand>>,
        ),
    ) -> (
        PreDiffTwo<work::Work, WithStatus<UserCommand>>,
        Option<PreDiffOne<work::Work, WithStatus<UserCommand>>>,
    ) {
        StagedLedger::with_ledger_mask(ledger, |status_ledger| {
            let diff = pre_diff_info::compute_statuses::<valid::Transaction>(
                &CONSTRAINT_CONSTANTS,
                diff,
                COINBASE_RECEIVER.clone(),
                coinbase_amount,
                global_slot,
                &dummy_state_view(Some(global_slot)),
                status_ledger,
            )
            .unwrap();

            with_valid_signatures_and_proofs::Diff { diff }
                .forget()
                .diff
        })
    }

    /// Invalid diff test: check zero fee excess for partitions
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2761
    #[test]
    fn check_zero_fee_excess_for_partitions() {
        let create_diff_with_non_zero_fee_excess =
            |ledger: Mask,
             coinbase_amount: Amount,
             global_slot: Slot,
             txns: Vec<valid::UserCommand>,
             completed_works: Vec<work::Unchecked>,
             partition: SpacePartition| {
                // With exact number of user commands in partition.first, the fee transfers that
                // settle the fee_excess would be added to the next tree causing a non-zero fee excess
                let (slots, job_count1) = partition.first;
                match partition.second {
                    None => Diff {
                        diff: {
                            compute_statutes(
                                ledger,
                                coinbase_amount,
                                global_slot,
                                (
                                    PreDiffTwo {
                                        completed_works: completed_works
                                            .iter()
                                            .take(job_count1 as usize)
                                            .cloned()
                                            .collect(),
                                        commands: txns
                                            .iter()
                                            .take(slots as usize)
                                            .cloned()
                                            .collect(),
                                        coinbase: AtMostTwo::Zero,
                                        internal_command_statuses: vec![],
                                    },
                                    None,
                                ),
                            )
                        },
                    },
                    Some(_) => {
                        let txns_in_second_diff = util::drop(&txns, slots as usize).to_vec();

                        let a = PreDiffTwo {
                            completed_works: completed_works
                                .iter()
                                .take(job_count1 as usize)
                                .cloned()
                                .collect(),
                            commands: txns.iter().take(slots as usize).cloned().collect(),
                            coinbase: AtMostTwo::Zero,
                            internal_command_statuses: vec![],
                        };

                        let b = PreDiffOne {
                            completed_works: if txns_in_second_diff.is_empty() {
                                vec![]
                            } else {
                                util::drop(&completed_works, job_count1 as usize).to_vec()
                            },
                            commands: txns_in_second_diff,
                            coinbase: AtMostOne::Zero,
                            internal_command_statuses: vec![],
                        };

                        Diff {
                            diff: compute_statutes(
                                ledger,
                                coinbase_amount,
                                global_slot,
                                (a, Some(b)),
                            ),
                        }
                    }
                }
            };

        let empty_diff = Diff::empty();

        let (ledger_init_state, cmds, iters) = gen_at_capacity();
        let global_slot = Slot::gen_small();

        async_with_ledgers(
            &ledger_init_state,
            cmds.clone(),
            iters.clone(),
            |_snarked_ledger, mut sl, _test_mask| {
                let checked = iter_cmds_acc(
                    &cmds,
                    &iters,
                    true,
                    |_cmds_left, _count_opt, cmds_this_iter, checked| {
                        let work = sl.scan_state.work_statements_for_new_diff();
                        let partitions = sl.scan_state.partition_if_overflowing();

                        let work_done: Vec<work::Checked> = work
                            .iter()
                            .map(|work| work::Checked {
                                fee: Fee::zero(),
                                proofs: proofs(work),
                                prover: SNARK_WORKER_PK.clone(),
                            })
                            .collect();

                        let cmds_this_iter: Vec<valid::UserCommand> = cmds_this_iter.to_vec();

                        let diff = create_diff_with_non_zero_fee_excess(
                            sl.ledger.clone(),
                            Amount::from_u64(CONSTRAINT_CONSTANTS.coinbase_amount),
                            global_slot,
                            cmds_this_iter,
                            work_done,
                            partitions,
                        );

                        let (current_state, current_view) = dummy_state_and_view(Some(global_slot));
                        let state_hashes = hashes_abstract(&current_state);

                        let apply_res = sl.apply(
                            None,
                            &CONSTRAINT_CONSTANTS,
                            global_slot,
                            diff.clone(),
                            (),
                            &Verifier,
                            &current_view,
                            state_hashes,
                            COINBASE_RECEIVER.clone(),
                            true,
                        );

                        let (new_checked, diff) = match apply_res {
                            Err(StagedLedgerError::NonZeroFeeExcess(..)) => {
                                (true, empty_diff.clone())
                            }
                            Err(e) => panic!("Expecting Non-zero-fee-excess error, got {:?}", e),
                            Ok(DiffResult { .. }) => (false, diff),
                        };

                        dbg!(new_checked, checked);

                        (diff, checked | new_checked)
                    },
                );

                // Note(OCaml): if this fails, try increasing the number of trials to get a diff that does fail
                assert!(checked);
            },
        );
    }

    const WORK_FEE: Fee = Fee::from_u64(CONSTRAINT_CONSTANTS.account_creation_fee);

    /// Provers can't pay the account creation fee
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2866
    #[test]
    fn provers_cant_pay_the_account_creation_fee() {
        let no_work_included = |diff: &Diff| diff.completed_works().is_empty();

        let stmt_to_work = |stmts: &work::Statement| {
            Some(work::Checked {
                fee: WORK_FEE.checked_sub(&Fee::from_u64(1)).unwrap(),
                proofs: proofs(stmts),
                prover: stmt_to_prover(stmts),
            })
        };
        let (ledger_init_state, cmds, iters) = gen_below_capacity(None);
        let global_slot = Slot::gen_small();

        async_with_ledgers(
            &ledger_init_state,
            cmds.to_vec(),
            iters.to_vec(),
            |_snarked_ledger, sl, _test_mask| {
                iter_cmds_acc(
                    &cmds,
                    &iters,
                    (),
                    |_cmds_left, _count_opt, cmds_this_iter, _| {
                        let current_state_view = dummy_state_view(Some(global_slot));

                        let (diff, _invalid_txns) = sl
                            .create_diff(
                                &CONSTRAINT_CONSTANTS,
                                global_slot,
                                None,
                                COINBASE_RECEIVER.clone(),
                                LOGGER,
                                &current_state_view,
                                cmds_this_iter.to_vec(),
                                stmt_to_work,
                                true,
                            )
                            .unwrap();

                        let diff = diff.forget();

                        // No proofs were purchased since the fee for the proofs are not
                        // sufficient to pay for account creation
                        assert!(no_work_included(&diff));

                        (diff, ())
                    },
                );
            },
        );
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2914
    fn stmt_to_work_restricted(
        work_list: &[work::Statement],
        provers: NumProvers,
    ) -> impl Fn(&work::Statement) -> Option<work::Checked> + '_ {
        move |stmts: &work::Statement| {
            if work_list.contains(stmts) {
                let prover = match provers {
                    NumProvers::Many => stmt_to_prover(stmts),
                    NumProvers::One => SNARK_WORKER_PK.clone(),
                };
                Some(work::Checked {
                    fee: WORK_FEE,
                    proofs: proofs(stmts),
                    prover,
                })
            } else {
                None
            }
        }
    }

    /// Like test_simple but with a random number of completed jobs available.
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2939
    fn test_random_number_of_proofs(
        global_slot: Slot,
        init: &LedgerInitialState,
        cmds: Vec<valid::UserCommand>,
        cmd_iters: Vec<Option<usize>>,
        proof_available: Vec<usize>,
        mut sl: StagedLedger,
        test_mask: Mask,
        provers: NumProvers,
    ) {
        let mut niters = 0;

        let proofs_available_left = iter_cmds_acc(
            &cmds,
            &cmd_iters,
            proof_available,
            |cmds_left, _count_opt, cmds_this_iter, mut proofs_available_left| {
                println!("######## Start new batch {} ########", niters);
                println!("nto_applied={:?}", cmds_this_iter.len());

                let work_list = sl.scan_state.all_work_statements_exn();

                let proofs_available_this_iter = *proofs_available_left.first().unwrap();

                let (current_state, current_state_view) = dummy_state_and_view(Some(global_slot));
                let state_and_body_hash = { hashes_abstract(&current_state) };

                let (proof, diff) = create_and_apply(
                    None,
                    None,
                    global_slot,
                    &current_state_view,
                    state_and_body_hash,
                    &mut sl,
                    cmds_this_iter,
                    stmt_to_work_restricted(
                        util::take(&work_list, proofs_available_this_iter),
                        provers,
                    ),
                );

                assert_fee_excess(&proof);

                let cmds_applied_this_iter = diff.commands().len();

                let cb = coinbase_count(&diff);

                assert!(proofs_available_this_iter == 0 || cb > 0);

                match provers {
                    NumProvers::One => assert!(cb <= 1),
                    NumProvers::Many => assert!(cb <= 2),
                }

                let coinbase_cost = coinbase_cost(&diff);

                assert_ledger(
                    test_mask.clone(),
                    coinbase_cost,
                    global_slot,
                    &current_state_view,
                    &sl,
                    cmds_left,
                    cmds_applied_this_iter,
                    &init_pks(init),
                );

                proofs_available_left.remove(0);

                eprintln!(
                    "######## Batch {} done: {} applied, {} proofs ########\n",
                    niters, cmds_applied_this_iter, proofs_available_this_iter
                );

                niters += 1;

                (diff, proofs_available_left)
            },
        );

        assert!(proofs_available_left.is_empty());
    }

    /// max throughput-random number of proofs-worst case provers
    ///
    /// Always at worst case number of provers
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2983
    #[test]
    fn max_throughput_random_number_of_proofs_worst_case_provers() {
        let mut rng = rand::thread_rng();

        let (ledger_init_state, cmds, iters) = gen_at_capacity();
        let global_slot = Slot::gen_small();

        // How many proofs will be available at each iteration.
        //
        // (OCaml) I think in the worst case every user command begets 1.5
        // transactions - one for the command and half of one for a fee
        // transfer - and the merge overhead means you need (amortized) twice
        // as many SNARKs as transactions, but since a SNARK work usually
        // covers two SNARKS it cancels. So we need to admit up to (1.5 * the
        // number of commands) works. I make it twice as many for simplicity
        // and to cover coinbases.
        let proofs_available: Vec<usize> = iters
            .iter()
            .map(|_| rng.gen_range(0..(TRANSACTION_CAPACITY * 2)))
            .collect();

        async_with_ledgers(
            &ledger_init_state,
            cmds.clone(),
            iters.clone(),
            |_snarked_ledger, sl, test_mask| {
                test_random_number_of_proofs(
                    global_slot,
                    &ledger_init_state,
                    cmds.clone(),
                    iters.clone(),
                    proofs_available.clone(),
                    sl,
                    test_mask,
                    NumProvers::Many,
                )
            },
        );
    }

    /// random no of transactions-random number of proofs-worst case provers
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3008
    #[test]
    fn random_number_of_transactions_random_number_of_proofs_worst_case_provers() {
        let mut rng = rand::thread_rng();

        let (ledger_init_state, cmds, iters) = gen_below_capacity(Some(true));
        let global_slot = Slot::gen_small();

        let proofs_available: Vec<usize> = iters
            .iter()
            .map(|cmds_opt| rng.gen_range(0..(3 * cmds_opt.unwrap())))
            .collect();

        async_with_ledgers(
            &ledger_init_state,
            cmds.clone(),
            iters.clone(),
            |_snarked_ledger, sl, test_mask| {
                test_random_number_of_proofs(
                    global_slot,
                    &ledger_init_state,
                    cmds.clone(),
                    iters.clone(),
                    proofs_available.clone(),
                    sl,
                    test_mask,
                    NumProvers::Many,
                )
            },
        );
    }

    /// Random number of commands-random number of proofs-one prover
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3057
    #[test]
    fn random_number_of_commands_random_number_of_proofs_one_prover() {
        let mut rng = rand::thread_rng();

        let (ledger_init_state, cmds, iters) = gen_below_capacity(Some(true));
        let global_slot = Slot::gen_small();

        let proofs_available: Vec<usize> = iters
            .iter()
            .map(|cmds_opt| rng.gen_range(0..(3 * cmds_opt.unwrap())))
            .collect();

        async_with_ledgers(
            &ledger_init_state,
            cmds.clone(),
            iters.clone(),
            |_snarked_ledger, sl, test_mask| {
                test_random_number_of_proofs(
                    global_slot,
                    &ledger_init_state,
                    cmds.clone(),
                    iters.clone(),
                    proofs_available.clone(),
                    sl,
                    test_mask,
                    NumProvers::One,
                )
            },
        );
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3076
    fn stmt_to_work_random_fee(
        work_list: &[(work::Statement, Fee)],
        provers: NumProvers,
    ) -> impl Fn(&work::Statement) -> Option<work::Checked> + '_ {
        move |stmts: &work::Statement| {
            work_list.iter().find(|(w, _)| w == stmts).map(|(_, fee)| {
                let prover = match provers {
                    NumProvers::Many => stmt_to_prover(stmts),
                    NumProvers::One => SNARK_WORKER_PK.clone(),
                };

                work::Checked {
                    fee: *fee,
                    proofs: proofs(stmts),
                    prover,
                }
            })
        }
    }

    /// Like test_random_number_of_proofs but with random proof fees.
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3095
    fn test_random_proof_fee(
        global_slot: Slot,
        _init: &LedgerInitialState,
        cmds: Vec<valid::UserCommand>,
        cmd_iters: Vec<Option<usize>>,
        proof_available: Vec<(usize, Vec<Fee>)>,
        mut sl: StagedLedger,
        _test_mask: Mask,
        provers: NumProvers,
    ) {
        let mut niters = 0;

        let proofs_available_left = iter_cmds_acc(
            &cmds,
            &cmd_iters,
            proof_available,
            |_cmds_left, _count_opt, cmds_this_iter, mut proofs_available_left| {
                println!("######## Start new batch {} ########", niters);
                println!("nto_applied={:?}", cmds_this_iter.len());

                let work_list = sl.scan_state.all_work_statements_exn();

                let (proofs_available_this_iter, fees_for_each) =
                    proofs_available_left.first().unwrap();
                let proofs_available_this_iter = *proofs_available_this_iter;

                let work_to_be_done = {
                    let work_list = util::take(&work_list, proofs_available_this_iter).to_vec();
                    let fees = util::take(fees_for_each, work_list.len()).to_vec();
                    work_list.into_iter().zip(fees).collect::<Vec<_>>()
                };

                let (current_state, current_state_view) = dummy_state_and_view(Some(global_slot));
                let state_and_body_hash = { hashes_abstract(&current_state) };

                let (_proof, diff) = create_and_apply(
                    None,
                    None,
                    global_slot,
                    &current_state_view,
                    state_and_body_hash,
                    &mut sl,
                    cmds_this_iter,
                    stmt_to_work_random_fee(&work_to_be_done, provers),
                );

                let sorted_work_from_diff1 = |pre_diff: &PreDiffWithAtMostTwoCoinbase| {
                    let mut pre_diff = pre_diff.completed_works.clone();
                    pre_diff.sort_by_key(|v| v.fee);
                    pre_diff
                };

                let sorted_work_from_diff2 = |pre_diff: &Option<PreDiffWithAtMostOneCoinbase>| {
                    pre_diff
                        .as_ref()
                        .map(|pre_diff| {
                            let mut pre_diff = pre_diff.completed_works.clone();
                            pre_diff.sort_by_key(|v| v.fee);
                            pre_diff
                        })
                        .unwrap_or_default()
                };

                {
                    let assert_same_fee = |cb: CoinbaseFeeTransfer, fee: Fee| {
                        assert_eq!(cb.fee, fee);
                    };

                    let (first_pre_diff, second_pre_diff_opt) = &diff.diff;

                    match (
                        first_pre_diff.coinbase.clone(),
                        second_pre_diff_opt
                            .as_ref()
                            .map(|d| d.coinbase.clone())
                            .unwrap_or(AtMostOne::Zero),
                    ) {
                        (AtMostTwo::Zero, AtMostOne::Zero) => {}
                        (AtMostTwo::Two(None), AtMostOne::Zero) => {}

                        (AtMostTwo::One(ft_opt), AtMostOne::Zero) => {
                            if let Some(single) = ft_opt {
                                let work = sorted_work_from_diff1(first_pre_diff);
                                let work = work[0].clone().forget();
                                assert_same_fee(single, work.fee);
                            };
                        }

                        (AtMostTwo::Zero, AtMostOne::One(ft_opt)) => {
                            if let Some(single) = ft_opt {
                                let work = sorted_work_from_diff2(second_pre_diff_opt);
                                let work = work[0].clone().forget();
                                assert_same_fee(single, work.fee);
                            };
                        }

                        (AtMostTwo::Two(Some((ft, ft_opt))), AtMostOne::Zero) => {
                            let work_done = sorted_work_from_diff1(first_pre_diff);
                            let work = work_done[0].clone().forget();
                            assert_same_fee(ft, work.fee);

                            if let Some(single) = ft_opt {
                                let work = util::drop(&work_done, 1);
                                let work = work[0].clone().forget();
                                assert_same_fee(single, work.fee);
                            };
                        }

                        (AtMostTwo::One(_), AtMostOne::One(_)) => {
                            panic!("Incorrect coinbase in the diff {:?}", &diff)
                        }
                        (AtMostTwo::Two(_), AtMostOne::One(_)) => {
                            panic!("Incorrect coinbase in the diff {:?}", &diff)
                        }
                    }
                }

                proofs_available_left.remove(0);

                eprintln!(
                    "######## Batch {} done: {} proofs ########\n",
                    niters, proofs_available_this_iter
                );

                niters += 1;

                (diff, proofs_available_left)
            },
        );

        assert!(proofs_available_left.is_empty());
    }

    /// max throughput-random-random fee-number of proofs-worst case provers
    ///
    /// Always at worst case number of provers
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3188
    #[test]
    fn max_throughput_random_number_fee_number_of_proofs_worst_case_provers() {
        let mut rng = rand::thread_rng();

        let (ledger_init_state, cmds, iters) = gen_at_capacity();
        let global_slot = Slot::gen_small();

        // How many proofs will be available at each iteration.
        let proofs_available: Vec<(usize, Vec<Fee>)> = iters
            .iter()
            .map(|_| {
                let number_of_proofs = rng.gen_range(0..TRANSACTION_CAPACITY * 2);
                let fees = (0..number_of_proofs)
                    .map(|_| {
                        let fee = rng.gen_range(1..20);
                        Fee::from_u64(fee)
                    })
                    .collect();

                (number_of_proofs, fees)
            })
            .collect();

        async_with_ledgers(
            &ledger_init_state,
            cmds.clone(),
            iters.clone(),
            |_snarked_ledger, sl, test_mask| {
                test_random_proof_fee(
                    global_slot,
                    &ledger_init_state,
                    cmds.clone(),
                    iters.clone(),
                    proofs_available.clone(),
                    sl,
                    test_mask,
                    NumProvers::Many,
                )
            },
        );
    }

    /// Max throughput-random fee
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3214
    #[test]
    fn max_throughput_random_fee() {
        let mut rng = rand::thread_rng();

        let (ledger_init_state, cmds, iters) = gen_at_capacity();
        let global_slot = Slot::gen_small();

        // How many proofs will be available at each iteration.
        let proofs_available: Vec<(usize, Vec<Fee>)> = iters
            .iter()
            .map(|_| {
                let number_of_proofs = TRANSACTION_CAPACITY;
                // All proofs are available

                let fees = (0..number_of_proofs)
                    .map(|_| {
                        let fee = rng.gen_range(1..20);
                        Fee::from_u64(fee)
                    })
                    .collect();

                (number_of_proofs, fees)
            })
            .collect();

        async_with_ledgers(
            &ledger_init_state,
            cmds.clone(),
            iters.clone(),
            |_snarked_ledger, sl, test_mask| {
                test_random_proof_fee(
                    global_slot,
                    &ledger_init_state,
                    cmds.clone(),
                    iters.clone(),
                    proofs_available.clone(),
                    sl,
                    test_mask,
                    NumProvers::Many,
                )
            },
        );
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3244
    fn check_pending_coinbase() {
        // TODO: this seems to be related to proof generation ? Which we don't support yet
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3290
    fn test_pending_coinbase(
        global_slot: Slot,
        init: &LedgerInitialState,
        cmds: Vec<valid::UserCommand>,
        cmd_iters: Vec<Option<usize>>,
        proof_available: Vec<usize>,
        // state_body_hashes: Vec<(Fp, Fp)>,
        // current_state_view: &ProtocolStateView,
        mut sl: StagedLedger,
        test_mask: Mask,
        provers: NumProvers,
    ) {
        let (proofs_available_left, _state_body_hashes_left) = iter_cmds_acc(
            &cmds,
            &cmd_iters,
            (proof_available, global_slot),
            |cmds_left, _count_opt, cmds_this_iter, (mut proofs_available_left, global_slot)| {
                let work_list = sl.scan_state.all_work_statements_exn();
                let proofs_available_this_iter = proofs_available_left[0];

                let (current_state, current_state_view) = dummy_state_and_view(Some(global_slot));
                let state_and_body_hash = { hashes_abstract(&current_state) };

                let (proof, diff, _is_new_stack, _pc_update, _supercharge_coinbase) =
                    create_and_apply_with_state_body_hash(
                        None,
                        None,
                        &current_state_view,
                        global_slot,
                        state_and_body_hash,
                        &mut sl,
                        cmds_this_iter,
                        stmt_to_work_restricted(
                            util::take(&work_list, proofs_available_this_iter),
                            provers,
                        ),
                    );

                check_pending_coinbase();

                assert_fee_excess(&proof);

                let cmds_applied_this_iter = diff.commands().len();

                let cb = coinbase_count(&diff);

                assert!(proofs_available_this_iter == 0 || cb > 0);

                match provers {
                    NumProvers::One => assert!(cb <= 1),
                    NumProvers::Many => assert!(cb <= 2),
                }

                let coinbase_cost = coinbase_cost(&diff);

                assert_ledger(
                    test_mask.clone(),
                    coinbase_cost,
                    global_slot,
                    &current_state_view,
                    &sl,
                    cmds_left,
                    cmds_applied_this_iter,
                    &init_pks(init),
                );

                proofs_available_left.remove(0);

                (diff, (proofs_available_left, global_slot.succ()))
            },
        );

        assert!(proofs_available_left.is_empty());
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3348
    fn pending_coinbase_test(prover: NumProvers) {
        let mut rng = rand::thread_rng();

        let (ledger_init_state, cmds, iters) = gen_below_capacity(Some(true));
        let global_slot = Slot::gen_small();

        let proofs_available: Vec<usize> = iters
            .iter()
            .map(|cmds_opt| rng.gen_range(0..(3 * cmds_opt.unwrap())))
            .collect();

        async_with_ledgers(
            &ledger_init_state,
            cmds.clone(),
            iters.clone(),
            |_snarked_ledger, sl, test_mask| {
                test_pending_coinbase(
                    global_slot,
                    &ledger_init_state,
                    cmds.clone(),
                    iters.clone(),
                    proofs_available.clone(),
                    sl,
                    test_mask,
                    prover,
                )
            },
        );
    }

    /// Validate pending coinbase for random number of
    /// commands-random number of proofs-one prover)
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3379
    #[test]
    fn validate_pending_coinbase_for_random_number_of_commands_one_prover() {
        pending_coinbase_test(NumProvers::One);
    }

    /// Validate pending coinbase for random number of
    /// commands-random number of proofs-many prover)
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3383
    #[test]
    fn validate_pending_coinbase_for_random_number_of_commands_many_prover() {
        pending_coinbase_test(NumProvers::Many);
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3387
    fn timed_account(_n: usize) -> (Keypair, Account) {
        let keypair = gen_keypair();
        let account_id = AccountId::new(keypair.public.into_compressed(), TokenId::default());
        let balance = Balance::from_u64(100_000_000_000);
        // Should fully vest by slot = 7
        let mut account = Account::create_with(account_id, balance);
        account.timing = crate::Timing::Timed {
            initial_minimum_balance: balance,
            cliff_time: Slot::from_u32(4),
            cliff_amount: Amount::zero(),
            vesting_period: SlotSpan::from_u32(2),
            vesting_increment: Amount::from_u64(50_000_000_000),
        };
        (keypair, account)
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3410
    fn untimed_account(_n: usize) -> (Keypair, Account) {
        let keypair = gen_keypair();
        let account_id = AccountId::new(keypair.public.into_compressed(), TokenId::default());
        let balance = Balance::from_u64(100_000_000_000);
        let account = Account::create_with(account_id, balance);
        (keypair, account)
    }

    fn stmt_to_work_zero_fee(
        prover: CompressedPubKey,
    ) -> impl Fn(&work::Statement) -> Option<work::Checked> {
        move |stmts: &work::Statement| {
            Some(work::Checked {
                fee: Fee::zero(),
                proofs: proofs(stmts),
                prover: prover.clone(),
            })
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3425
    fn supercharge_coinbase_test<F>(
        this: Account,
        delegator: Account,
        block_count: usize,
        f_expected_balance: F,
        sl: &mut StagedLedger,
    ) where
        F: Fn(usize, Balance) -> Balance,
    {
        let coinbase_receiver = &this;
        let init_balance = coinbase_receiver.balance;

        let check_receiver_account = |sl: &StagedLedger, count: usize| {
            let location = sl
                .ledger
                .location_of_account(&coinbase_receiver.id())
                .unwrap();
            let account = sl.ledger.get(location).unwrap();
            dbg!(account.balance, f_expected_balance(count, init_balance));
            assert_eq!(account.balance, f_expected_balance(count, init_balance));
        };

        (0..block_count).map(|n| n + 1).for_each(|block_count| {
            let global_slot = Slot::from_u32(block_count.try_into().unwrap());

            let (current_state, current_state_view) = dummy_state_and_view(Some(global_slot));
            let state_and_body_hash = { hashes_abstract(&current_state) };

            create_and_apply_with_state_body_hash(
                Some(coinbase_receiver.public_key.clone()),
                Some(delegator.public_key.clone()),
                &current_state_view,
                global_slot,
                state_and_body_hash,
                sl,
                &[],
                stmt_to_work_zero_fee(this.public_key.clone()),
            );
            check_receiver_account(sl, block_count);
        });
    }

    const NORMAL_COINBASE: Amount = Amount::from_u64(CONSTRAINT_CONSTANTS.coinbase_amount);

    const fn scale_exn(amount: Amount, i: u64) -> Amount {
        match amount.scale(i) {
            Some(amount) => amount,
            None => panic!(),
        }
    }

    const SUPERCHARGED_COINBASE: Amount = scale_exn(
        Amount::from_u64(CONSTRAINT_CONSTANTS.coinbase_amount),
        CONSTRAINT_CONSTANTS.supercharged_coinbase_factor,
    );

    /// Supercharged coinbase - staking
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3468
    #[test]
    fn supercharged_coinbase_staking() {
        let (keypair_this, this) = timed_account(1);

        // calculated from the timing values for timed_accounts
        let slots_with_locked_tokens = 7;

        let block_count = slots_with_locked_tokens + 5;

        let f_expected_balance = |block_no: usize, init_balance: Balance| {
            if block_no <= slots_with_locked_tokens {
                init_balance
                    .add_amount(scale_exn(NORMAL_COINBASE, block_no as u64))
                    .unwrap()
            } else {
                // init balance +
                //    (normal_coinbase * slots_with_locked_tokens) +
                //    (supercharged_coinbase * remaining slots))*)
                let balance = init_balance
                    .add_amount(scale_exn(NORMAL_COINBASE, slots_with_locked_tokens as u64))
                    .unwrap();
                let amount = scale_exn(
                    SUPERCHARGED_COINBASE,
                    (block_no.checked_sub(slots_with_locked_tokens).unwrap()) as u64,
                );
                balance.add_amount(amount).unwrap()
            }
        };

        let mut ledger_init_state = gen_initial_ledger_state();

        ledger_init_state.state.insert(
            0,
            (
                keypair_this,
                this.balance.to_amount(),
                this.nonce,
                this.timing.clone(),
            ),
        );

        async_with_ledgers(
            &ledger_init_state,
            vec![],
            vec![],
            |_snaked_ledger, mut sl, _test_mask| {
                supercharge_coinbase_test(
                    this.clone(),
                    this.clone(),
                    block_count,
                    f_expected_balance,
                    &mut sl,
                )
            },
        );
    }

    /// Supercharged coinbase - unlocked account delegating to locked account
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3505
    #[test]
    fn supercharged_coinbase_unlocked_account_delegating_to_locked_account() {
        let (keypair_this, locked_this) = timed_account(1);
        let (keypair_delegator, unlocked_delegator) = untimed_account(1);

        // calculated from the timing values for timed_accounts
        let slots_with_locked_tokens = 7;

        let block_count = slots_with_locked_tokens + 2;

        let f_expected_balance = |block_no: usize, init_balance: Balance| {
            init_balance
                .add_amount(scale_exn(SUPERCHARGED_COINBASE, block_no as u64))
                .unwrap()
        };

        let state = [
            (
                keypair_this,
                locked_this.balance.to_amount(),
                locked_this.nonce,
                locked_this.timing.clone(),
            ),
            (
                keypair_delegator,
                unlocked_delegator.balance.to_amount(),
                unlocked_delegator.nonce,
                unlocked_delegator.timing.clone(),
            ),
        ]
        .into_iter()
        .chain(gen_initial_ledger_state().state)
        .collect();

        let ledger_init_state = LedgerInitialState { state };

        async_with_ledgers(
            &ledger_init_state,
            vec![],
            vec![],
            |_snarked_ledger, mut sl, _test_mask| {
                supercharge_coinbase_test(
                    locked_this.clone(),
                    unlocked_delegator.clone(),
                    block_count,
                    f_expected_balance,
                    &mut sl,
                )
            },
        );
    }

    /// Supercharged coinbase - locked account delegating to unlocked account
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3537
    #[test]
    fn supercharged_coinbase_locked_account_delegating_to_unlocked_account() {
        let (keypair_this, unlocked_this) = untimed_account(1);
        let (keypair_delegator, locked_delegator) = timed_account(1);

        // calculated from the timing values for timed_accounts
        let slots_with_locked_tokens = 7;

        let block_count = slots_with_locked_tokens + 2;

        let f_expected_balance = |block_no: usize, init_balance: Balance| {
            if block_no <= slots_with_locked_tokens {
                init_balance
                    .add_amount(scale_exn(NORMAL_COINBASE, block_no as u64))
                    .unwrap()
            } else {
                // init balance +
                //    (normal_coinbase * slots_with_locked_tokens) +
                //    (supercharged_coinbase * remaining slots))*)
                let balance = init_balance
                    .add_amount(scale_exn(NORMAL_COINBASE, slots_with_locked_tokens as u64))
                    .unwrap();
                let amount = scale_exn(
                    SUPERCHARGED_COINBASE,
                    (block_no.checked_sub(slots_with_locked_tokens).unwrap()) as u64,
                );
                balance.add_amount(amount).unwrap()
            }
        };

        let state = [
            (
                keypair_this,
                unlocked_this.balance.to_amount(),
                unlocked_this.nonce,
                unlocked_this.timing.clone(),
            ),
            (
                keypair_delegator,
                locked_delegator.balance.to_amount(),
                locked_delegator.nonce,
                locked_delegator.timing.clone(),
            ),
        ]
        .into_iter()
        .chain(gen_initial_ledger_state().state)
        .collect();

        let ledger_init_state = LedgerInitialState { state };

        async_with_ledgers(
            &ledger_init_state,
            vec![],
            vec![],
            |_snarked_ledger, mut sl, _test_mask| {
                supercharge_coinbase_test(
                    unlocked_this.clone(),
                    locked_delegator.clone(),
                    block_count,
                    f_expected_balance,
                    &mut sl,
                )
            },
        );
    }

    /// Supercharged coinbase - locked account delegating to locked account
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3580
    #[test]
    fn supercharged_coinbase_locked_account_delegating_to_locked_account() {
        let (keypair_this, locked_this) = timed_account(1);
        let (keypair_delegator, locked_delegator) = timed_account(2);

        // calculated from the timing values for timed_accounts
        let slots_with_locked_tokens = 7;

        let block_count = slots_with_locked_tokens;

        let f_expected_balance = |block_no: usize, init_balance: Balance| {
            // running the test as long as both the accounts remain locked and hence
            // normal coinbase in all the blocks
            init_balance
                .add_amount(scale_exn(NORMAL_COINBASE, block_no as u64))
                .unwrap()
        };

        let state = [
            (
                keypair_this,
                locked_this.balance.to_amount(),
                locked_this.nonce,
                locked_this.timing.clone(),
            ),
            (
                keypair_delegator,
                locked_delegator.balance.to_amount(),
                locked_delegator.nonce,
                locked_delegator.timing.clone(),
            ),
        ]
        .into_iter()
        .chain(gen_initial_ledger_state().state)
        .collect();

        let ledger_init_state = LedgerInitialState { state };

        async_with_ledgers(
            &ledger_init_state,
            vec![],
            vec![],
            |_snarked_ledger, mut sl, _test_mask| {
                supercharge_coinbase_test(
                    locked_this.clone(),
                    locked_delegator.clone(),
                    block_count,
                    f_expected_balance,
                    &mut sl,
                )
            },
        );
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3612
    fn command_insufficient_funds() -> (LedgerInitialState, valid::UserCommand, Slot) {
        let ledger_initial_state = gen_initial_ledger_state();
        let global_slot = Slot::gen_small();

        let (kp, balance, nonce, _) = &ledger_initial_state.state[0];

        let receiver_pk = gen_keypair().public.into_compressed();

        let insufficient_account_creation_fee =
            Amount::from_u64(CONSTRAINT_CONSTANTS.account_creation_fee / 2);

        let source_pk = kp.public.into_compressed();

        let body = signed_command::Body::Payment(PaymentPayload {
            receiver_pk,
            amount: insufficient_account_creation_fee,
        });
        let fee = Fee::from_u64(balance.as_u64());

        let payload =
            SignedCommandPayload::create(fee, source_pk, *nonce, None, Memo::dummy(), body);

        let payload_to_sign = TransactionUnionPayload::of_user_command_payload(&payload);

        let mut signer = mina_signer::create_legacy(mina_signer::NetworkId::TESTNET);
        let signature = signer.sign(kp, &payload_to_sign);

        let signed_command = SignedCommand {
            payload,
            signer: kp.public.into_compressed(),
            signature,
        };

        let cmd = valid::UserCommand::SignedCommand(Box::new(signed_command));
        (ledger_initial_state, cmd, global_slot)
    }

    /// Commands with Insufficient funds are not included
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3643
    #[test]
    fn commands_with_insufficient_funds_are_not_included() {
        let (ledger_init_state, invalid_commands, global_slot) = command_insufficient_funds();

        async_with_ledgers(
            &ledger_init_state,
            vec![invalid_commands.clone()],
            vec![],
            |_snarked_ledger, sl, _test_mask| {
                let current_state_view = dummy_state_view(Some(global_slot));

                let (diff, _invalid_txns) = sl
                    .create_diff(
                        &CONSTRAINT_CONSTANTS,
                        global_slot,
                        None,
                        COINBASE_RECEIVER.clone(),
                        (),
                        &current_state_view,
                        vec![invalid_commands.clone()],
                        stmt_to_work_zero_fee(SELF_PK.clone()),
                        false,
                    )
                    .unwrap();

                assert!(diff.commands().is_empty());
            },
        );
    }

    /// Blocks having commands with insufficient funds are rejected
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3665
    #[test]
    fn blocks_having_commands_with_sufficient_funds_are_rejected() {
        enum Validity {
            Valid,
            Invalid,
        }

        let ledger_init_state = gen_initial_ledger_state();
        let global_slot = Slot::gen_small();
        // let (kp, balance, nonce, _) = &ledger_initial_state.state[0];

        let command = |kp: Keypair, balance: Amount, nonce: Nonce, validity: Validity| {
            let receiver_pk = gen_keypair().public.into_compressed();

            let (account_creation_fee, fee) = {
                match validity {
                    Validity::Valid => {
                        let account_creation_fee =
                            Amount::from_u64(CONSTRAINT_CONSTANTS.account_creation_fee);
                        let fee = balance.checked_sub(&account_creation_fee).unwrap();
                        (account_creation_fee, Fee::from_u64(fee.as_u64()))
                    }
                    Validity::Invalid => {
                        // Not enough account creation fee and using full balance for fee
                        let account_creation_fee = CONSTRAINT_CONSTANTS.account_creation_fee / 2;
                        let account_creation_fee = Amount::from_u64(account_creation_fee);
                        (account_creation_fee, Fee::from_u64(balance.as_u64()))
                    }
                }
            };

            let source_pk = kp.public.into_compressed();
            let body = signed_command::Body::Payment(PaymentPayload {
                receiver_pk,
                amount: account_creation_fee,
            });
            let payload = signed_command::SignedCommandPayload::create(
                fee,
                source_pk,
                nonce,
                None,
                Memo::dummy(),
                body,
            );

            let payload_to_sign = TransactionUnionPayload::of_user_command_payload(&payload);

            let mut signer = mina_signer::create_legacy(mina_signer::NetworkId::TESTNET);
            let signature = signer.sign(&kp, &payload_to_sign);

            let signed_command = SignedCommand {
                payload,
                signer: kp.public.into_compressed(),
                signature,
            };

            valid::UserCommand::SignedCommand(Box::new(signed_command))
        };

        let signed_command = {
            let (kp, balance, nonce, _) = ledger_init_state.state[0].clone();
            command(kp, balance, nonce, Validity::Valid)
        };

        let invalid_command = {
            let (kp, balance, nonce, _) = ledger_init_state.state[1].clone();
            command(kp, balance, nonce, Validity::Invalid)
        };

        async_with_ledgers(
            &ledger_init_state,
            vec![invalid_command.clone(), signed_command.clone()],
            vec![],
            |_snarked_ledger, mut sl, _test_mask| {
                let (current_state, current_state_view) = dummy_state_and_view(Some(global_slot));
                let state_and_body_hash = { hashes_abstract(&current_state) };

                let (diff, _invalid_txns) = sl
                    .create_diff(
                        &CONSTRAINT_CONSTANTS,
                        global_slot,
                        None,
                        COINBASE_RECEIVER.clone(),
                        (),
                        &current_state_view,
                        vec![signed_command.clone()],
                        stmt_to_work_zero_fee(SELF_PK.clone()),
                        false,
                    )
                    .unwrap();

                assert_eq!(diff.commands().len(), 1);

                let (mut f, s) = diff.diff;

                let failed_command = WithStatus {
                    data: invalid_command.clone(),
                    status: TransactionStatus::Failed(vec![vec![
                        TransactionFailure::AmountInsufficientToCreateAccount,
                    ]]),
                };

                // Replace the valid command with an invalid command
                f.commands = vec![failed_command];
                let diff = with_valid_signatures_and_proofs::Diff { diff: (f, s) };

                let res = sl.apply(
                    None,
                    &CONSTRAINT_CONSTANTS,
                    global_slot,
                    diff.forget(),
                    (),
                    &Verifier,
                    &current_state_view,
                    state_and_body_hash,
                    COINBASE_RECEIVER.clone(),
                    false,
                );

                let expected = TransactionFailure::SourceInsufficientBalance.to_string();

                assert!(
                    matches!(&res, Err(StagedLedgerError::Unexpected(s)) if {
                        s == &expected
                    }),
                    "{:?}",
                    res
                );
            },
        );
    }

    /// Mismatched verification keys in zkApp accounts and and transactions
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3776
    // #[test] // TODO: This test requires the prover
    #[allow(unused)]
    fn mismatched_vk_in_zkapp_accounts_and_transactions() {
        use scan_state::transaction_logic::for_tests::{TestSpec, UpdateStatesSpec};

        let test_spec = TestSpec::gen();

        let pks: BTreeSet<_> = test_spec
            .init_ledger
            .0
            .iter()
            .map(|(kp, _)| kp.public.into_compressed())
            .collect();

        let kp = loop {
            let keypair = gen_keypair();
            if !pks.contains(&keypair.public.into_compressed()) {
                break keypair;
            }
        };

        let TestSpec {
            init_ledger,
            specs: _,
        } = test_spec;
        let new_kp = kp;

        let fee = Fee::from_u64(1_000_000);
        let amount = Amount::of_mina_int_exn(10);

        let snapp_pk = new_kp.public.into_compressed();

        let mut snapp_update = zkapp_command::Update::dummy();
        snapp_update.delegate = SetOrKeep::Set(snapp_pk.clone());

        let memo = Memo::dummy();

        let test_spec = UpdateStatesSpec {
            fee,
            sender: (new_kp.clone(), Nonce::zero()),
            fee_payer: None,
            receivers: vec![],
            amount,
            zkapp_account_keypairs: vec![new_kp],
            memo,
            new_zkapp_account: false,
            snapp_update,
            current_auth: AuthRequired::Proof,
            actions: vec![],
            events: vec![],
            call_data: Fp::zero(),
            preconditions: None,
        };

        let mut ledger = Mask::new_unattached(CONSTRAINT_CONSTANTS.ledger_depth as usize);

        init_ledger.init(None, &mut ledger);

        let global_slot = Slot::gen_small();

        // create a zkApp account
        let mut snapp_permissions = Permissions::user_default();
        snapp_permissions.set_delegate = AuthRequired::Proof;

        let snapp_account_id = AccountId::new(snapp_pk, TokenId::default());

        let dummy_vk = {
            let dummy_vk = VerificationKey::dummy();
            let vk_hash = dummy_vk.digest();

            WithHash {
                data: dummy_vk,
                hash: vk_hash,
            }
        };

        #[allow(clippy::let_unit_value)]
        let valid_against_ledger = {
            let mut _new_mask = ledger.make_child();
            // for_tests::create_trivial_zkapp_account(Some(snapp_permissions), vk, &mut new_mask, snapp_pk);
        };
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use std::{collections::BTreeMap, fs::File};

    use mina_hasher::Fp;
    use mina_p2p_messages::{binprot, list::List};

    use crate::{
        scan_state::{
            currency::Slot,
            pending_coinbase::PendingCoinbase,
            scan_state::{transaction_snark::LedgerProof, ScanState},
            transaction_logic::{local_state::LocalState, protocol_state::protocol_state_view},
        },
        staged_ledger::{
            diff::Diff,
            staged_ledger::{tests_ocaml::CONSTRAINT_CONSTANTS, StagedLedger},
            validate_block::validate_block,
        },
        verifier::{get_srs, Verifier},
        Account, BaseLedger, Database, Mask,
    };
    use binprot::BinProtRead;
    use mina_p2p_messages::{rpc::GetStagedLedgerAuxAndPendingCoinbasesAtHashV2Response, v2};
    use mina_signer::CompressedPubKey;

    #[test]
    fn staged_ledger_hash() {
        fn elapsed<R>(label: &str, fun: impl FnOnce() -> R) -> R {
            let now = redux::Instant::now();
            let result = fun();
            println!("{} elapsed={:?}", label, now.elapsed());
            result
        }

        let Ok(mut snarked_ledger_file) = File::open("target/snarked_ledger") else {
            eprintln!("File target/snarked_ledger not found");
            return;
        };

        let Ok(mut staged_ledger_file) = File::open("target/staged_ledger") else {
            eprintln!("File target/staged_ledger not found");
            return;
        };

        let mut snarked_ledger = Mask::new_root(Database::create(
            CONSTRAINT_CONSTANTS.ledger_depth.try_into().unwrap(),
        ));

        for account in Vec::<Account>::binprot_read(&mut snarked_ledger_file).unwrap() {
            let account_id = account.id();
            snarked_ledger
                .get_or_create_account(account_id, account)
                .unwrap();
        }

        let info = elapsed("staged ledger parsing", || {
            GetStagedLedgerAuxAndPendingCoinbasesAtHashV2Response::binprot_read(
                &mut staged_ledger_file,
            )
            .unwrap()
        });

        println!("Prepare snarked ledger");

        let (scan_state, expected_ledger_hash, pending_coinbase, states) = info.unwrap();
        let states = states
            .into_iter()
            .map(|state| {
                let s: crate::proofs::block::ProtocolState = (&state).try_into().unwrap();
                (crate::scan_state::protocol_state::MinaHash::hash(&s), state)
            })
            .collect::<BTreeMap<_, _>>();

        println!("Load staged ledger info");

        let scan_state: ScanState = elapsed("scan_state conversion", || {
            (&scan_state).try_into().unwrap()
        });
        let pending_coinbase: PendingCoinbase = elapsed("pending_coinbase conversion", || {
            (&pending_coinbase).try_into().unwrap()
        });

        let mut staged_ledger =
            elapsed("of_scan_state_pending_coinbases_and_snarked_ledger", || {
                StagedLedger::of_scan_state_pending_coinbases_and_snarked_ledger(
                    (),
                    &CONSTRAINT_CONSTANTS,
                    Verifier,
                    scan_state,
                    snarked_ledger,
                    LocalState::empty(),
                    expected_ledger_hash.try_into().unwrap(),
                    pending_coinbase,
                    |key| states.get(&key).cloned().unwrap(),
                )
                .unwrap()
            });

        println!("Prepare staged ledger");

        let hash = v2::MinaBaseStagedLedgerHashStableV1::from(&staged_ledger.hash());
        let reference = r#"{"non_snark":{"ledger_hash":"jwcznejL82UzKAgTUCpoQ9aw4NBfph25nsgMLoowXBx8iknMt2H","aux_hash":"VU7r5u7vbm9FtBgU2R5nJhFNcBRfDXXyJuVX2qiXgjD5fzqYQ7","pending_coinbase_aux":"XyLefxPzSEbi25gRvSVZvn65fDtowhbVY1dHT1XuYgaEUsnA4z"},"pending_coinbase_hash":"2n1QN8RQT8Au6uMmihuEBMp6mbfcuduG88EQDFMBJbZbQ3SPqsuA"}"#;
        assert_eq!(reference, serde_json::to_string(&hash).unwrap());
    }

    #[test]
    fn apply_berkeleynet() {
        #[allow(unused)]
        // --serialize
        use binprot::{
            macros::{BinProtRead, BinProtWrite},
            BinProtRead, BinProtWrite,
        };
        // use serde::{Deserialize, Serialize};
        // #[derive(BinProtRead, BinProtWrite)]
        // struct Ser {
        //     accounts: Vec<v2::MinaBaseAccountBinableArgStableV2>,
        //     scan_state: v2::TransactionSnarkScanStateStableV2,
        //     pending_coinbase: v2::MinaBasePendingCoinbaseStableV2,
        //     block: v2::MinaBlockBlockStableV2,
        //     pred_block: v2::MinaBlockBlockStableV2,
        // }

        #[derive(BinProtRead, BinProtWrite)]
        struct Ser2 {
            accounts: Vec<v2::MinaBaseAccountBinableArgStableV2>,
            scan_state: v2::TransactionSnarkScanStateStableV2,
            pending_coinbase: v2::MinaBasePendingCoinbaseStableV2,
            pred_block: v2::MinaBlockBlockStableV2,
            blocks: Vec<v2::MinaBlockBlockStableV2>,
        }

        let Ok(mut f) = std::fs::File::open("blocks.bin") else {
            eprintln!("blocks.bin not found");
            return;
        };

        let now = redux::Instant::now();

        let Ser2 {
            accounts,
            scan_state,
            pending_coinbase,
            mut pred_block,
            blocks,
        } = BinProtRead::binprot_read(&mut f).unwrap();

        let accounts: Vec<Account> = accounts
            .iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .unwrap();
        let scan_state: ScanState = (&scan_state).try_into().unwrap();
        let pending_coinbase: PendingCoinbase = (&pending_coinbase).try_into().unwrap();

        let mut root = Mask::new_root(Database::create(35));
        for account in accounts {
            root.get_or_create_account(account.id(), account).unwrap();
        }

        let ledger = root.make_child();

        let mut staged_ledger = StagedLedger {
            scan_state,
            ledger,
            constraint_constants: CONSTRAINT_CONSTANTS.clone(),
            pending_coinbase_collection: pending_coinbase,
        };

        let block_verifier = crate::proofs::verifiers::BlockVerifier::make();

        println!("initialized in {:?}", now.elapsed());

        dbg!(staged_ledger.ledger.nmasks_to_root());

        let srs = get_srs::<Fp>();

        for (index, block) in blocks.into_iter().enumerate() {
            validate_block(&block).unwrap();

            let block_height = block
                .header
                .protocol_state
                .body
                .consensus_state
                .blockchain_length
                .0
                .as_u32();

            let global_slot = block
                .header
                .protocol_state
                .body
                .consensus_state
                .global_slot_since_genesis
                .as_u32();

            crate::proofs::verification::verify_block(&block.header, &block_verifier, &srs);

            let diff: Diff = (&block.body.staged_ledger_diff).try_into().unwrap();

            let prev_protocol_state = &pred_block.header.protocol_state;
            let prev_state_view = protocol_state_view(prev_protocol_state).unwrap();

            let prev_state: crate::proofs::block::ProtocolState =
                prev_protocol_state.try_into().unwrap();
            let prev_state_and_body_hash = prev_state.hashes();

            let consensus_state = &block.header.protocol_state.body.consensus_state;
            let coinbase_receiver: CompressedPubKey =
                (&consensus_state.coinbase_receiver).try_into().unwrap();
            let _supercharge_coinbase = consensus_state.supercharge_coinbase;
            let supercharge_coinbase = false;

            let now = redux::Instant::now();

            let result = staged_ledger
                .apply(
                    None,
                    &CONSTRAINT_CONSTANTS,
                    Slot::from_u32(global_slot),
                    diff,
                    (),
                    &Verifier,
                    &prev_state_view,
                    prev_state_and_body_hash,
                    coinbase_receiver,
                    supercharge_coinbase,
                )
                .unwrap();

            // eprintln!("apply {:?}", now.elapsed());
            let ledger_hashes =
                v2::MinaBaseStagedLedgerHashStableV1::from(&result.hash_after_applying);

            // TODO(binier): return error if not matching.
            let expected_ledger_hashes = &block
                .header
                .protocol_state
                .body
                .blockchain_state
                .staged_ledger_hash;

            if &ledger_hashes != expected_ledger_hashes {
                panic!("staged ledger hash mismatch. found: {ledger_hashes:?}, expected: {expected_ledger_hashes:?}");
            }

            dbg!(staged_ledger.ledger.nmasks_to_root());
            eprintln!(
                "block {:?} applied in {:?} napplied={:?}",
                block_height,
                now.elapsed(),
                index + 1
            );

            // if block_height == 1020 {
            //     break;
            // }

            pred_block = block;
        }

        println!("total={:?}", now.elapsed());

        // let mut staged = None;
        // let mut pred_block = None;

        // let mut blocks = Vec::with_capacity(1000);

        // for height in 1016..1202 {
        //     dbg!(height);
        //     let file = format!("/home/sebastien/travaux/openmina/apply_{}.bin", height);
        //     let mut f = std::fs::File::open(file).unwrap();
        //     let v: Ser = BinProtRead::binprot_read(&mut f).unwrap();

        //     if staged.is_none() {
        //         staged = Some((v.accounts, v.scan_state, v.pending_coinbase));
        //         pred_block = Some(v.pred_block);
        //     }
        //     blocks.push(v.block);
        // }

        // let (accounts, scan_state, pending_coinbase) = staged.unwrap();
        // let pred_block = pred_block.unwrap();

        // let ser = Ser2 {
        //     accounts,
        //     scan_state,
        //     pending_coinbase,
        //     pred_block,
        //     blocks,
        // };

        // let mut encoded = Vec::with_capacity(100 * 1024);
        // BinProtWrite::binprot_write(&ser, &mut encoded).unwrap();
        // let mut f = std::fs::File::create("blocks.bin").unwrap();
        // std::io::Write::write_all(&mut f, &encoded).unwrap();
        // // f.write_all(&encoded).unwrap();
        // f.sync_all().unwrap();

        // eprintln!("OK");

        // let ser = Ser {
        //     accounts: staged_ledger
        //         .ledger()
        //         .to_list()
        //         .into_iter()
        //         .map(|v| v.into())
        //         .collect(),
        //     scan_state: staged_ledger.scan_state().into(),
        //     pending_coinbase: staged_ledger.pending_coinbase_collection().into(),
        //     block: (*block.block).clone(),
        //     pred_block: (*pred_block.block).clone(),
        // };
        // eprintln!("4");
        // let mut encoded = vec![];
        // BinProtWrite::binprot_write(&ser, &mut encoded).unwrap();
        // let mut f = std::fs::File::create(format!("apply_{}.bin", block.height())).unwrap();
        // f.write_all(&encoded).unwrap();
        // f.flush().unwrap();
        // --serialize
    }

    #[test]
    fn test_tx_proof() {
        use crate::proofs::to_field_elements::ToFieldElements;
        #[allow(unused)]
        use binprot::{BinProtRead, BinProtWrite};

        let Ok(ledger_proof) = std::fs::read("ledger_proof2.bin") else {
            eprintln!("ledger_proof.bin not found");
            return;
        };
        let Ok(_sok_msg) = std::fs::read("sok_msg2.bin") else {
            eprintln!("sok_msg2.bin not found");
            return;
        };

        let mut ledger_proof = std::io::Cursor::new(ledger_proof);
        let ledger_proof: v2::LedgerProofProdStableV2 =
            BinProtRead::binprot_read(&mut ledger_proof).unwrap();

        let ledger_proof: LedgerProof = (&ledger_proof).try_into().unwrap();
        let stmt = ledger_proof.statement_ref();

        dbg!(stmt.to_field_elements_owned());
    }

    #[test]
    fn reconstruct_staged_ledger() {
        #[allow(unused)]
        use binprot::{
            macros::{BinProtRead, BinProtWrite},
            BinProtRead, BinProtWrite,
        };

        #[derive(BinProtRead, BinProtWrite)]
        struct ReconstructContext {
            accounts: Vec<v2::MinaBaseAccountBinableArgStableV2>,
            scan_state: v2::TransactionSnarkScanStateStableV2,
            pending_coinbase: v2::MinaBasePendingCoinbaseStableV2,
            staged_ledger_hash: v2::LedgerHash,
            states: List<v2::MinaStateProtocolStateValueStableV2>,
        }

        let now = std::time::Instant::now();

        let Ok(file) = std::fs::read("/tmp/failed_reconstruct_ctx.binprot") else {
            eprintln!("no reconstruct context found");
            return;
        };

        let ReconstructContext {
            accounts,
            scan_state,
            pending_coinbase,
            staged_ledger_hash,
            states,
        } = ReconstructContext::binprot_read(&mut file.as_slice()).unwrap();

        let states = states
            .iter()
            .map(|state| {
                (
                    state.try_hash().unwrap().to_field::<Fp>().unwrap(),
                    state.clone(),
                )
            })
            .collect::<BTreeMap<_, _>>();

        const LEDGER_DEPTH: usize = 35;
        let mut ledger = Mask::create(LEDGER_DEPTH);
        for account in &accounts {
            let account: Account = account.try_into().unwrap();
            let id = account.id();
            ledger.get_or_create_account(id, account).unwrap();
        }
        assert_eq!(ledger.num_accounts(), accounts.len());

        eprintln!("time to parse and restore state: {:?}", now.elapsed());
        let now = std::time::Instant::now();

        let scan_state = (&scan_state).try_into().unwrap();
        eprintln!("time to convert scan state: {:?}", now.elapsed());

        let mut staged_ledger = StagedLedger::of_scan_state_pending_coinbases_and_snarked_ledger(
            (),
            openmina_core::constants::constraint_constants(),
            Verifier,
            scan_state,
            ledger,
            LocalState::empty(),
            staged_ledger_hash.0.to_field().unwrap(),
            (&pending_coinbase).try_into().unwrap(),
            |key| states.get(&key).cloned().unwrap(),
        )
        .unwrap();

        eprintln!("time to reconstruct: {:?}", now.elapsed());
        let now = std::time::Instant::now();
        dbg!(staged_ledger.hash());
        eprintln!("time to hash: {:?}", now.elapsed());
    }
}
