use mina_hasher::Fp;
use mina_p2p_messages::v2::MinaStateProtocolStateValueStableV2;
use mina_signer::CompressedPubKey;

use crate::{
    decompress_pk,
    scan_state::{
        currency::{Amount, Fee, Magnitude},
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
            AvailableJob, ConstraintConstants, ScanState, SpacePartition, StatementCheck,
        },
        snark_work::spec,
        transaction_logic::{
            apply_transaction,
            local_state::LocalState,
            protocol_state::{protocol_state_view, ProtocolStateView},
            transaction_applied::TransactionApplied,
            valid, verifiable, Transaction, TransactionStatus, UserCommand, WithStatus,
        },
    },
    split_at,
    verifier::{Verifier, VerifierError},
    AccountId, BaseLedger, Mask, TokenId,
};

use super::{diff::Diff, pre_diff_info::PreDiffError, sparse_ledger::SparseLedger};

/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#470
#[derive(Clone, Debug)]
pub struct StackStateWithInitStack {
    pub pc: StackState,
    pub init_stack: Stack,
}

// pub enum PreDiffError {
//     CoinbaseError(&'static str),
// }

/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L23
pub enum StagedLedgerError {
    NonZeroFeeExcess(Vec<WithStatus<Transaction>>, SpacePartition),
    InvalidProofs,
    CouldntReachVerifier,
    PreDiff(PreDiffError),
    InsufficientWork(String),
    MismatchedStatuses {
        transaction: WithStatus<Transaction>,
        got: TransactionStatus,
    },
    InvalidPublicKey(CompressedPubKey),
    Unexpected(String),
}

impl From<String> for StagedLedgerError {
    fn from(value: String) -> Self {
        Self::Unexpected(value)
    }
}

impl From<PreDiffError> for StagedLedgerError {
    fn from(value: PreDiffError) -> Self {
        Self::PreDiff(value)
    }
}

// module Staged_ledger_error = struct
//   type t =
//     | Non_zero_fee_excess of
//         Scan_state.Space_partition.t * Transaction.t With_status.t list
//     | Invalid_proofs of
//         ( Ledger_proof.t
//         * Transaction_snark.Statement.t
//         * Mina_base.Sok_message.t )
//         list
//     | Couldn't_reach_verifier of Error.t
//     | Pre_diff of Pre_diff_info.Error.t
//     | Insufficient_work of string
//     | Mismatched_statuses of
//         Transaction.t With_status.t * Transaction_status.t
//     | Invalid_public_key of Public_key.Compressed.t
//     | Unexpected of Error.t
//   [@@deriving sexp]

struct DiffResult {
    hash_after_applying: StagedLedgerHash,
    ledger_proof: Option<(LedgerProof, Vec<(WithStatus<Transaction>, Fp)>)>,
    pending_coinbase_update: (bool, Update),
}

struct StagedLedgerHash;

pub struct StagedLedger {
    scan_state: ScanState,
    ledger: Mask,
    constraint_constants: ConstraintConstants,
    pending_coinbase_collection: PendingCoinbase,
}

impl StagedLedger {
    pub fn proof_txns_with_state_hashes(&self) -> Option<Vec<(WithStatus<Transaction>, Fp)>> {
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

    pub fn pending_coinbase_collection(&self) -> &PendingCoinbase {
        &self.pending_coinbase_collection
    }

    fn get_target(
        (proof_with_msg, _): (
            &LedgerProofWithSokMessage,
            Vec<(WithStatus<Transaction>, Fp)>,
        ),
    ) -> &Registers {
        &proof_with_msg.proof.statement().target
    }

    fn verify_scan_state_after_apply(
        constraint_constants: &ConstraintConstants,
        pending_coinbase_stack: Stack,
        ledger: LedgerHash,
        scan_state: &ScanState,
    ) -> Result<(), String> {
        let registers_end = Registers {
            ledger,
            pending_coinbase_stack,
            local_state: LocalState::empty(),
        };
        let statement_check = StatementCheck::Partial;
        let registers_begin = scan_state.latest_ledger_proof().map(Self::get_target);

        scan_state.check_invariants(
            constraint_constants,
            statement_check,
            &Verifier,
            "Error verifying the parallel scan state after applying the diff.",
            registers_begin,
            registers_end,
        )
    }

    fn of_scan_state_and_ledger<F>(
        _logger: (),
        constraint_constants: &ConstraintConstants,
        verifier: Verifier,
        snarked_registers: Registers,
        mut ledger: Mask,
        scan_state: ScanState,
        pending_coinbase_collection: PendingCoinbase,
        get_state: F,
    ) -> Result<Self, String>
    where
        F: Fn(&Fp) -> &MinaStateProtocolStateValueStableV2 + 'static,
    {
        let pending_coinbase_stack = pending_coinbase_collection.latest_stack(false);

        scan_state.check_invariants(
            constraint_constants,
            StatementCheck::Full(Box::new(get_state)),
            &verifier,
            "Staged_ledger.of_scan_state_and_ledger",
            Some(&snarked_registers),
            Registers {
                ledger: ledger.merkle_root(),
                pending_coinbase_stack,
                local_state: LocalState::empty(),
            },
        )?;

        Ok(Self {
            scan_state,
            ledger,
            constraint_constants: constraint_constants.clone(),
            pending_coinbase_collection,
        })
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L292
    fn of_scan_state_and_ledger_unchecked(
        constraint_constants: &ConstraintConstants,
        snarked_registers: Registers,
        mut ledger: Mask,
        scan_state: ScanState,
        pending_coinbase_collection: PendingCoinbase,
    ) -> Result<Self, String> {
        let pending_coinbase_stack = pending_coinbase_collection.latest_stack(false);

        scan_state.check_invariants(
            constraint_constants,
            StatementCheck::Partial,
            &Verifier, // null
            "Staged_ledger.of_scan_state_and_ledger",
            Some(&snarked_registers),
            Registers {
                ledger: ledger.merkle_root(),
                pending_coinbase_stack,
                local_state: LocalState::empty(),
            },
        )?;

        Ok(Self {
            scan_state,
            ledger,
            constraint_constants: constraint_constants.clone(),
            pending_coinbase_collection,
        })
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L318
    fn of_scan_state_pending_coinbases_and_snarked_ledger_prime<F, G>(
        constraint_constants: &ConstraintConstants,
        pending_coinbase: PendingCoinbase,
        scan_state: ScanState,
        mut snarked_ledger: Mask,
        snarked_local_state: LocalState,
        expected_merkle_root: LedgerHash,
        get_state: F,
        fun: G,
    ) -> Result<Self, String>
    where
        F: Fn(&Fp) -> &MinaStateProtocolStateValueStableV2 + 'static,
        G: FnOnce(
            &ConstraintConstants,
            Registers,
            Mask,
            ScanState,
            PendingCoinbase,
        ) -> Result<Self, String>,
    {
        let snarked_ledger_hash = snarked_ledger.merkle_root();
        let snarked_frozen_ledger_hash = snarked_ledger_hash;

        let txs_with_protocol_state =
            scan_state.staged_transactions_with_protocol_states(get_state);

        for (tx, protocol_state) in txs_with_protocol_state {
            let txn_with_info = apply_transaction(
                constraint_constants,
                &protocol_state_view(&protocol_state),
                &mut snarked_ledger,
                &tx.data,
            )?;

            let computed_status = txn_with_info.transaction_status();

            if &tx.status != computed_status {
                return Err(format!(
                    "Mismatched user command status. Expected: {:#?} Got: {:#?}",
                    tx.status, computed_status
                ));
            }
        }

        let staged_ledger_hash = snarked_ledger.merkle_root();
        if staged_ledger_hash != expected_merkle_root {
            return Err(format!(
                "Mismatching merkle root Expected:{:?} Got:{:?}",
                expected_merkle_root.to_string(),
                staged_ledger_hash.to_string()
            ));
        }

        let pending_coinbase_stack = match scan_state.latest_ledger_proof() {
            Some(proof) => Self::get_target(proof).pending_coinbase_stack.clone(),
            None => Stack::empty(),
        };

        fun(
            constraint_constants,
            Registers {
                ledger: snarked_frozen_ledger_hash,
                pending_coinbase_stack,
                local_state: snarked_local_state,
            },
            snarked_ledger,
            scan_state,
            pending_coinbase,
        )
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L378
    fn of_scan_state_pending_coinbases_and_snarked_ledger<F>(
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
        F: Fn(&Fp) -> &MinaStateProtocolStateValueStableV2 + Copy + 'static,
    {
        Self::of_scan_state_pending_coinbases_and_snarked_ledger_prime(
            constraint_constants,
            pending_coinbase,
            scan_state,
            snarked_ledger,
            snarked_local_state,
            expected_merkle_root,
            get_state,
            |constraint_constants,
             snarked_registers,
             ledger,
             scan_state,
             pending_coinbase_collection| {
                Self::of_scan_state_and_ledger(
                    logger,
                    constraint_constants,
                    verifier,
                    snarked_registers,
                    ledger,
                    scan_state,
                    pending_coinbase_collection,
                    get_state,
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
        F: Fn(&Fp) -> &MinaStateProtocolStateValueStableV2 + 'static,
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
    fn copy(&self) -> Self {
        let new_mask = Mask::new_unattached(self.ledger.depth() as usize);
        let new_mask = new_mask.register_mask(self.ledger.clone());

        Self {
            scan_state: self.scan_state.clone(), // TODO: Not sure if OCaml keeps the same pointer
            ledger: new_mask,
            constraint_constants: self.constraint_constants.clone(),
            pending_coinbase_collection: self.pending_coinbase_collection.clone(), // TODO: Not sure if OCaml keeps the same pointer
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#403
    fn hash(&self) -> StagedLedgerHash {
        todo!()
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#422
    fn ledger(&self) -> Mask {
        self.ledger.clone()
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#424
    fn create_exn(constraint_constants: ConstraintConstants, ledger: Mask) -> Result<Self, String> {
        let pending_coinbase_depth = constraint_constants.pending_coinbase_depth as usize;

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

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#446
    fn sum_fees<T, F>(fees: &[T], fun: F) -> Result<Fee, String>
    where
        F: Fn(&T) -> Fee,
    {
        let mut accum = Fee::zero();
        for fee in fees {
            accum = match accum.checked_add(&fun(fee)) {
                Some(accum) => accum,
                None => return Err("Fee overflow".to_string()),
            }
        }
        Ok(accum)
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#456
    fn working_stack(
        pending_coinbase_collection: &PendingCoinbase,
        is_new_stack: bool,
    ) -> Result<Stack, String> {
        Ok(pending_coinbase_collection.latest_stack(is_new_stack))
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#460
    fn push_coinbase(current_stack: Stack, transaction: &Transaction) -> Stack {
        match transaction {
            Transaction::Coinbase(c) => current_stack.push_coinbase(c.clone()),
            _ => current_stack,
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#467
    fn push_state(current_stack: Stack, state_body_hash: Fp) -> Stack {
        current_stack.push_state(state_body_hash)
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#477
    pub fn coinbase_amount(
        supercharge_coinbase: bool,
        constraint_constants: &ConstraintConstants,
    ) -> Option<Amount> {
        if supercharge_coinbase {
            constraint_constants
                .coinbase_amount
                .scale(constraint_constants.supercharged_coinbase_factor)
        } else {
            Some(constraint_constants.coinbase_amount)
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#501
    fn apply_transaction_and_get_statement(
        constraint_constants: &ConstraintConstants,
        mut ledger: Mask,
        pending_coinbase_stack_state: StackStateWithInitStack,
        transaction: &Transaction,
        txn_state_view: &ProtocolStateView,
    ) -> Result<(TransactionApplied, Statement, StackStateWithInitStack), StagedLedgerError> {
        let fee_excess = transaction.fee_excess()?;

        let source_merkle_root = ledger.merkle_root();

        let pending_coinbase_target =
            Self::push_coinbase(pending_coinbase_stack_state.pc.target, transaction);

        let new_init_stack =
            Self::push_coinbase(pending_coinbase_stack_state.init_stack, transaction);

        let empty_local_state = LocalState::empty();

        let applied_txn = apply_transaction(
            constraint_constants,
            txn_state_view,
            &mut ledger,
            transaction,
        )
        .map_err(|e| format!("Error when applying transaction: {:?}", e))?;

        let supply_increase = applied_txn.supply_increase(constraint_constants)?;

        let target_merkle_root = ledger.merkle_root();

        let statement = Statement {
            source: Registers {
                ledger: source_merkle_root,
                pending_coinbase_stack: pending_coinbase_stack_state.pc.source,
                local_state: empty_local_state.clone(),
            },
            target: Registers {
                ledger: target_merkle_root,
                pending_coinbase_stack: pending_coinbase_target.clone(),
                local_state: empty_local_state,
            },
            supply_increase,
            fee_excess,
            sok_digest: None,
        };

        let state = StackStateWithInitStack {
            pc: StackState {
                source: pending_coinbase_target.clone(),
                target: pending_coinbase_target,
            },
            init_stack: new_init_stack,
        };

        Ok((applied_txn, statement, state))
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L560
    fn apply_transaction_and_get_witness(
        constraint_constants: &ConstraintConstants,
        ledger: Mask,
        pending_coinbase_stack_state: StackStateWithInitStack,
        transaction: &Transaction,
        status: Option<&TransactionStatus>,
        txn_state_view: &ProtocolStateView,
        state_and_body_hash: (Fp, Fp),
    ) -> Result<(TransactionWithWitness, StackStateWithInitStack), StagedLedgerError> {
        let account_ids = |transaction: &Transaction| -> Vec<AccountId> {
            match transaction {
                Transaction::Command(cmd) => cmd.accounts_referenced(),
                Transaction::FeeTransfer(t) => t.receivers().collect(),
                Transaction::Coinbase(c) => {
                    let mut ids = Vec::with_capacity(2);

                    ids.push(AccountId::new(c.receiver.clone(), TokenId::default()));
                    if let Some(t) = c.fee_transfer.as_ref() {
                        ids.push(t.receiver());
                    };

                    ids
                }
            }
        };

        let ledger_witness =
            SparseLedger::of_ledger_subset_exn(ledger.clone(), &account_ids(transaction));

        let (applied_txn, statement, updated_pending_coinbase_stack_state) =
            Self::apply_transaction_and_get_statement(
                constraint_constants,
                ledger,
                pending_coinbase_stack_state.clone(),
                transaction,
                txn_state_view,
            )?;

        if let Some(status) = status {
            let got_status = applied_txn.transaction_status();

            if status != got_status {
                return Err(StagedLedgerError::MismatchedStatuses {
                    transaction: applied_txn.transaction(),
                    got: got_status.clone(),
                });
            }
        };

        let witness = TransactionWithWitness {
            transaction_with_info: applied_txn,
            state_hash: state_and_body_hash,
            statement,
            init_stack: InitStack::Base(pending_coinbase_stack_state.init_stack),
            ledger_witness,
        };

        Ok((witness, updated_pending_coinbase_stack_state))
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L611
    fn update_ledger_and_get_statements(
        constraint_constants: &ConstraintConstants,
        ledger: Mask,
        current_stack: Stack,
        transactions: &[WithStatus<Transaction>],
        current_state_view: &ProtocolStateView,
        state_and_body_hash: (Fp, Fp),
    ) -> Result<(Vec<TransactionWithWitness>, Stack), StagedLedgerError> {
        let current_stack_with_state = current_stack.push_state(state_and_body_hash.1);
        let mut witnesses = Vec::with_capacity(transactions.len());

        let mut pending_coinbase_stack_state = StackStateWithInitStack {
            pc: StackState {
                source: current_stack.clone(),
                target: current_stack_with_state,
            },
            init_stack: current_stack,
        };

        for transaction in transactions {
            let public_keys = transaction.data.public_keys();

            if let Some(pk) = public_keys.iter().find(|pk| decompress_pk(pk).is_none()) {
                return Err(StagedLedgerError::InvalidPublicKey(pk.clone()));
            }

            let (value, updated) = Self::apply_transaction_and_get_witness(
                constraint_constants,
                ledger.clone(),
                pending_coinbase_stack_state,
                &transaction.data,
                Some(&transaction.status),
                current_state_view,
                state_and_body_hash,
            )?;

            witnesses.push(value);
            pending_coinbase_stack_state = updated;
        }

        Ok((witnesses, pending_coinbase_stack_state.pc.target))
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L164
    fn verify(
        _logger: (),
        _verifier: &Verifier,
        _job_msg_proofs: Vec<(AvailableJob, SokMessage, LedgerProof)>,
    ) -> Result<(), StagedLedgerError> {
        todo!()
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
                Err(StagedLedgerError::NonZeroFeeExcess(txns, slots.clone()))
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
        scan_state: &ScanState,
        ledger: Mask,
        pending_coinbase_collection: &mut PendingCoinbase,
        transactions: Vec<WithStatus<Transaction>>,
        current_state_view: &ProtocolStateView,
        state_and_body_hash: (Fp, Fp),
    ) -> Result<(bool, Vec<TransactionWithWitness>, Action, StackUpdate), StagedLedgerError> {
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

                let (data, updated_stack) = Self::update_ledger_and_get_statements(
                    constraint_constants,
                    ledger,
                    working_stack,
                    &transactions,
                    current_state_view,
                    state_and_body_hash,
                )?;

                Ok((
                    is_new_stack,
                    data,
                    Action::One,
                    StackUpdate::One(updated_stack),
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
                    split_at(transactions.as_slice(), slots as usize);

                let coinbase_in_first_partition = coinbase_exists(txns_for_partition1);

                let working_stack1 = Self::working_stack(pending_coinbase_collection, false)?;
                // Push the new state to the state_stack from the previous block even in the second stack
                let working_stack2 = Stack::create_with(&working_stack1);

                let (mut data1, updated_stack1) = Self::update_ledger_and_get_statements(
                    constraint_constants,
                    ledger.clone(),
                    working_stack1,
                    txns_for_partition1,
                    current_state_view,
                    state_and_body_hash,
                )?;

                let (mut data2, updated_stack2) = Self::update_ledger_and_get_statements(
                    constraint_constants,
                    ledger,
                    working_stack2,
                    txns_for_partition2,
                    current_state_view,
                    state_and_body_hash,
                )?;

                let second_has_data = !txns_for_partition2.is_empty();

                let (pending_coinbase_action, stack_update) =
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

                data1.append(&mut data2);
                let data = data1;

                Ok((false, data, pending_coinbase_action, stack_update))
            }
        } else {
            Ok((false, Vec::new(), Action::None, StackUpdate::None))
        }
    }

    /// update the pending_coinbase tree with the updated/new stack and delete the oldest stack if a proof was emitted
    ///
    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L806
    fn update_pending_coinbase_collection(
        depth: usize,
        pending_coinbase: &mut PendingCoinbase,
        stack_update: StackUpdate,
        is_new_stack: bool,
        ledger_proof: &Option<(LedgerProof, Vec<(WithStatus<Transaction>, Fp)>)>,
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

        let (is_new_stack, data, stack_update_in_snark, stack_update) =
            Self::update_coinbase_stack_and_get_data(
                constraint_constants,
                &self.scan_state,
                new_ledger.clone(),
                &mut self.pending_coinbase_collection,
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
            constraint_constants.pending_coinbase_depth as usize,
            &mut self.pending_coinbase_collection,
            stack_update,
            is_new_stack,
            &res_opt,
        )?;

        let coinbase_amount = Self::coinbase_for_blockchain_snark(&coinbases)?;

        let latest_pending_coinbase_stack = self.pending_coinbase_collection.latest_stack(false);

        if !skip_verification {
            Self::verify_scan_state_after_apply(
                constraint_constants,
                latest_pending_coinbase_stack,
                new_ledger.merkle_root(),
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
        a: Vec<WithStatus<valid::Transaction>>,
        b: B,
        c: C,
        d: D,
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

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1020
    fn check_commands(
        ledger: Mask,
        verifier: &Verifier,
        cs: Vec<&UserCommand>,
    ) -> Result<Vec<valid::UserCommand>, VerifierError> {
        let cmds: Vec<verifiable::UserCommand> =
            cs.iter().map(|cmd| cmd.to_verifiable(&ledger)).collect();

        let xs = verifier.verify_commands(cmds)?;

        // TODO: OCaml does check the list `xs`

        Ok(xs)
    }

    fn apply(
        &mut self,
        skip_verification: Option<SkipVerification>,
        constraint_constants: &ConstraintConstants,
        witness: Diff,
        logger: (),
        verifier: &Verifier,
        current_state_view: &ProtocolStateView,
        state_and_body_hash: (Fp, Fp),
        coinbase_receiver: CompressedPubKey,
        supercharge_coinbase: bool,
    ) -> Result<(), StagedLedgerError> {
        let work = witness.completed_works();

        if skip_verification.is_none() {
            Self::check_completed_works(logger, verifier, &self.scan_state, work)?;
        }

        witness.get(
            |cmd| Self::check_commands(self.ledger.clone(), verifier, cmd),
            constraint_constants,
            coinbase_receiver,
            supercharge_coinbase,
        )?;

        Ok(())
    }
}

enum SkipVerification {
    All,
    Proofs,
}

// let apply ?skip_verification ~constraint_constants t
//     (witness : Staged_ledger_diff.t) ~logger ~verifier ~current_state_view
//     ~state_and_body_hash ~coinbase_receiver ~supercharge_coinbase =
//   let open Deferred.Result.Let_syntax in
//   let work = Staged_ledger_diff.completed_works witness in
//   let%bind () =
//     O1trace.thread "check_completed_works" (fun () ->
//         match skip_verification with
//         | Some `All | Some `Proofs ->
//             return ()
//         | None ->
//             check_completed_works ~logger ~verifier t.scan_state work )
//   in
//   let%bind prediff =
//     Pre_diff_info.get witness ~constraint_constants ~coinbase_receiver
//       ~supercharge_coinbase
//       ~check:(check_commands t.ledger ~verifier)
//     |> Deferred.map
//          ~f:
//            (Result.map_error ~f:(fun error ->
//                 Staged_ledger_error.Pre_diff error ) )
//   in
//   let apply_diff_start_time = Core.Time.now () in
//   let%map ((_, _, `Staged_ledger new_staged_ledger, _) as res) =
//     apply_diff
//       ~skip_verification:
//         ([%equal: [ `All | `Proofs ] option] skip_verification (Some `All))
//       ~constraint_constants t
//       (forget_prediff_info prediff)
//       ~logger ~current_state_view ~state_and_body_hash
//       ~log_prefix:"apply_diff"
//   in
//   [%log debug]
//     ~metadata:
//       [ ( "time_elapsed"
//         , `Float Core.Time.(Span.to_ms @@ diff (now ()) apply_diff_start_time)
//         )
//       ]
//     "Staged_ledger.apply_diff take $time_elapsed" ;
//   let () =
//     Or_error.iter_error (update_metrics new_staged_ledger witness)
//       ~f:(fun e ->
//         [%log error]
//           ~metadata:[ ("error", Error_json.error_to_yojson e) ]
//           !"Error updating metrics after applying staged_ledger diff: $error" )
//   in
//   res
