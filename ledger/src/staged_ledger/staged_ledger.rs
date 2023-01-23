use std::collections::{HashMap, HashSet};

use mina_hasher::Fp;
use mina_p2p_messages::v2::MinaStateProtocolStateValueStableV2;
use mina_signer::CompressedPubKey;

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
            AvailableJob, ConstraintConstants, ScanState, SpacePartition, StatementCheck,
        },
        snark_work::spec,
        transaction_logic::{
            apply_transaction,
            local_state::LocalState,
            protocol_state::{protocol_state_view, ProtocolStateView},
            transaction_applied::TransactionApplied,
            valid::{self, VerificationKeyHash},
            verifiable,
            zkapp_command::Control,
            CoinbaseFeeTransfer, Transaction, TransactionStatus, UserCommand, WithStatus,
        },
    },
    split_at, split_at_vec,
    staged_ledger::{pre_diff_info, resources::IncreaseBy},
    verifier::{Verifier, VerifierError},
    Account, AccountId, BaseLedger, Mask, TokenId,
};

use super::{
    diff::{with_valid_signatures_and_proofs, AtMostOne, AtMostTwo, Diff, PreDiffTwo},
    diff_creation_log::{DiffCreationLog, Partition},
    hash::StagedLedgerHash,
    pre_diff_info::PreDiffError,
    resources::Resources,
    sparse_ledger::SparseLedger,
    transaction_validator::HashlessLedger,
};

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
#[derive(Debug)]
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

#[derive(Debug)]
struct DiffResult {
    hash_after_applying: StagedLedgerHash,
    ledger_proof: Option<(LedgerProof, Vec<(WithStatus<Transaction>, Fp)>)>,
    pending_coinbase_update: (bool, Update),
}

enum SkipVerification {
    All,
    Proofs,
}

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
        &proof_with_msg.proof.statement_ref().target
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
        let new_mask = self.ledger.make_child();

        Self {
            scan_state: self.scan_state.clone(), // TODO: Not sure if OCaml keeps the same pointer
            ledger: new_mask,
            constraint_constants: self.constraint_constants.clone(),
            pending_coinbase_collection: self.pending_coinbase_collection.clone(), // TODO: Not sure if OCaml keeps the same pointer
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#403
    fn hash(&mut self) -> StagedLedgerHash {
        StagedLedgerHash::of_aux_ledger_and_coinbase_hash(
            self.scan_state.hash(),
            self.ledger.merkle_root(),
            &mut self.pending_coinbase_collection,
        )
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
    ) -> Result<(TransactionApplied, Statement<()>, StackStateWithInitStack), StagedLedgerError>
    {
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
            sok_digest: (),
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
        // TODO
        Ok(())
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

        Self::check_zero_fee_excess(&self.scan_state, &data)?;

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
    ) -> Result<DiffResult, StagedLedgerError> {
        let work = witness.completed_works();

        if skip_verification.is_none() {
            Self::check_completed_works(logger, verifier, &self.scan_state, work)?;
        }

        let prediff = witness.get(
            |cmd| Self::check_commands(self.ledger.clone(), verifier, cmd),
            constraint_constants,
            coinbase_receiver,
            supercharge_coinbase,
        )?;

        self.apply_diff(
            logger,
            skip_verification.map(|s| matches!(s, SkipVerification::All)),
            Self::forget_prediff_info(prediff),
            constraint_constants,
            current_state_view,
            state_and_body_hash,
            "apply_diff",
        )
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1095
    fn apply_diff_unchecked(
        &mut self,
        constraint_constants: &ConstraintConstants,
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
                    if let Some(uc) = uc_opt {
                        log.discard_command(NoSpace, &uc.data);
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
            if let Some(uc) = uc_opt {
                log.discard_command(NoWork, &uc.data);
            };
            Self::check_constraints_and_update(constraint_constants, resources, log);
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1624
    fn one_prediff(
        constraint_constants: &ConstraintConstants,
        cw_seq: Vec<work::Unchecked>,
        ts_seq: Vec<WithStatus<valid::UserCommand>>,
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
        ts_seq: Vec<WithStatus<valid::UserCommand>>,
        receiver: &CompressedPubKey,
        is_coinbase_receiver_new: bool,
        supercharge_coinbase: bool,
        partitions: scan_state::SpacePartition,
    ) -> (
        (
            PreDiffTwo<work::Work, WithStatus<valid::UserCommand>>,
            Option<super::diff::PreDiffOne<work::Work, WithStatus<valid::UserCommand>>>,
        ),
        Vec<DiffCreationLog>,
    ) {
        let pre_diff_with_one =
            |mut res: Resources| -> with_valid_signatures_and_proofs::PreDiffWithAtMostOneCoinbase {
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
                with_valid_signatures_and_proofs::PreDiffWithAtMostOneCoinbase {
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
            |mut res: Resources| -> with_valid_signatures_and_proofs::PreDiffWithAtMostTwoCoinbase {
                with_valid_signatures_and_proofs::PreDiffWithAtMostTwoCoinbase {
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
                PreDiffTwo<work::Work, WithStatus<valid::UserCommand>>,
                Option<super::diff::PreDiffOne<work::Work, WithStatus<valid::UserCommand>>>,
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

                let (res1, res2) = if res.commands_rev.is_empty() {
                    let res = try_with_coinbase();
                    (res, None)
                } else {
                    match res.available_space() {
                        0 => {
                            // generate the next prediff with a coinbase at least
                            let res2 = second_pre_diff(res.clone(), y, true, cw_seq_2);
                            ((res, log1), Some(res2))
                        }
                        1 => {
                            // There's a slot available in the first partition, fill it
                            // with coinbase and create another pre_diff for the slots
                            // in the second partiton with the remaining user commands and work

                            incr_coinbase_and_compute(res, IncreaseBy::One)
                        }
                        2 => {
                            // There are two slots which cannot be filled using user commands,
                            // so we split the coinbase into two parts and fill those two spots

                            incr_coinbase_and_compute(res, IncreaseBy::Two)
                        }
                        _ => {
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
        epoch_ledger: &SparseLedger<AccountId, Account>,
        global_slot: Slot,
    ) -> bool {
        !epoch_ledger.has_locked_tokens_exn(global_slot, AccountId::new(winner, TokenId::default()))
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1787
    fn validate_account_update_proofs(
        _logger: (),
        validating_ledger: &HashlessLedger,
        txn: &valid::UserCommand,
    ) -> bool {
        use super::sparse_ledger::LedgerIntf;

        let get_verification_keys = |account_ids: &HashSet<AccountId>| {
            let get_vk = |account_id: &AccountId| -> Option<VerificationKeyHash> {
                let addr = validating_ledger.location_of_account(account_id)?;
                let account = validating_ledger.get(&addr)?;
                let vk = account.zkapp.as_ref()?.verification_key.as_ref()?;
                // TODO: In OCaml this is a field (using `WithHash`)
                Some(VerificationKeyHash(vk.hash()))
            };

            let mut map = HashMap::with_capacity(128);

            for id in account_ids {
                match get_vk(id) {
                    Some(vk) => {
                        map.insert(id.clone(), vk);
                    }
                    None => {
                        eprintln!(
                            "Staged_ledger_diff creation: Verification key not found for \
                             account_update with proof authorization and account_id \
                             {:?}",
                            id
                        );
                        return HashMap::new();
                    }
                }
            }

            map
        };

        match txn {
            valid::UserCommand::ZkAppCommand(p) => {
                let checked_verification_keys: HashMap<AccountId, VerificationKeyHash> =
                    p.verification_keys.iter().cloned().collect();

                let proof_zkapp_command = p.zkapp_command.account_updates.fold(
                    HashSet::with_capacity(128),
                    |mut accum, (update, _)| {
                        if let Control::Proof(_) = &update.authorization {
                            accum.insert(update.account_id());
                        }
                        accum
                    },
                );

                let current_verification_keys = get_verification_keys(&proof_zkapp_command);

                if proof_zkapp_command.len() == checked_verification_keys.len()
                    && checked_verification_keys == current_verification_keys
                {
                    true
                } else {
                    eprintln!(
                        "Staged_ledger_diff creation: Verifcation keys used for verifying \
                             proofs {:#?} and verification keys in the \
                             ledger {:#?} don't match",
                        checked_verification_keys, current_verification_keys
                    );
                    false
                }
            }
            valid::UserCommand::SignedCommand(_) => true,
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L1863
    fn create_diff<F>(
        &self,
        constraint_constants: &ConstraintConstants,
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
        use super::sparse_ledger::LedgerIntf;

        let _log_block_creation = log_block_creation.unwrap_or(false);

        let mut validating_ledger = HashlessLedger::create(self.ledger.clone());

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
                        || cw_checked.fee >= constraint_constants.account_creation_fee
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
            let res = Self::validate_account_update_proofs(logger, &validating_ledger, &txn)
                .then_some(())
                .ok_or_else(|| "Verification key mismatch".to_string())
                .and_then(|_| {
                    let txn = Transaction::Command(txn.forget_check());
                    validating_ledger.apply_transaction(
                        constraint_constants,
                        current_state_view,
                        &txn,
                    )
                });

            match res {
                Err(e) => {
                    eprintln!(
                        "Staged_ledger_diff creation: Skipping user command: {:#?} due to error: {:?}",
                        txn, e
                    );
                    invalid_on_this_ledger.push((txn, e));
                }
                Ok(status) => {
                    let txn_with_status = WithStatus { data: txn, status };
                    valid_on_this_ledger.push(txn_with_status);
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

        let diff = {
            // Fill in the statuses for commands.
            let mut status_ledger = HashlessLedger::create(self.ledger.clone());

            let mut generate_status = |txn: Transaction| -> Result<TransactionStatus, String> {
                status_ledger.apply_transaction(constraint_constants, current_state_view, &txn)
            };

            pre_diff_info::compute_statuses::<valid::UserCommand, valid::Transaction, _>(
                constraint_constants,
                diff,
                coinbase_receiver,
                Self::coinbase_amount(supercharge_coinbase, constraint_constants)
                    .expect("OCaml throws here"),
                &mut generate_status,
            )?
        };

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
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#L2024
    fn latest_block_accounts_created(&self, previous_block_state_hash: Fp) -> Vec<AccountId> {
        use scan_state::transaction_logic::transaction_applied::signed_command_applied::Body;
        use scan_state::transaction_logic::transaction_applied::CommandApplied;
        use scan_state::transaction_logic::transaction_applied::Varying;

        let block_transactions_applied = {
            let f = |TransactionWithWitness {
                         transaction_with_info,
                         state_hash: (leaf_block_hash, _),
                         ..
                     }: TransactionWithWitness| {
                if leaf_block_hash == previous_block_state_hash {
                    Some(transaction_with_info.varying)
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
    use std::str::FromStr;

    use ark_ff::{UniformRand, Zero};
    use mina_signer::{Keypair, Signer};
    use o1_utils::FieldHelpers;
    use once_cell::sync::Lazy;
    use rand::{seq::SliceRandom, CryptoRng, Rng};

    use crate::{
        dummy::{self, trivial_verification_key},
        gen_keypair,
        scan_state::{
            currency::{Balance, BlockTime, Fee, Length, Nonce},
            scan_state::transaction_snark::SokDigest,
            transaction_logic::{
                protocol_state::{EpochData, EpochLedger},
                signed_command::{self, PaymentPayload, SignedCommand, SignedCommandPayload},
                transaction_union_payload::TransactionUnionPayload,
                zkapp_command::{self, SetOrKeep, WithHash},
                Memo, Signature, TransactionFailure,
            },
        },
        staged_ledger::{
            diff::{PreDiffOne, PreDiffWithAtMostOneCoinbase, PreDiffWithAtMostTwoCoinbase},
            pre_diff_info::HashableCompressedPubKey,
        },
        util, AuthRequired, FpExt, Permissions, VerificationKey, ZkAppAccount,
    };

    use super::*;

    // const

    static SELF_PK: Lazy<CompressedPubKey> = Lazy::new(|| gen_keypair().public.into_compressed());

    static COINBASE_RECEIVER: Lazy<CompressedPubKey> =
        Lazy::new(|| gen_keypair().public.into_compressed());

    /// Same values when we run `dune runtest src/lib/staged_ledger -f`
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
        state_and_body_hash: (Fp, Fp),
        sl: &mut StagedLedger,
        txns: &[valid::UserCommand],
        stmt_to_work: F,
    ) -> (
        Option<(LedgerProof, Vec<(WithStatus<Transaction>, Fp)>)>,
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

        let supercharge_coinbase = supercharge_coinbase(
            sl.ledger.clone(),
            winner,
            current_state_view.global_slot_since_genesis,
        );

        let (diff, _invalid_txns) = sl
            .create_diff(
                &CONSTRAINT_CONSTANTS,
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
                        sender_pk.clone(),
                        nonce,
                        None,
                        memo,
                        Body::Payment(PaymentPayload {
                            source_pk: sender_pk,
                            receiver_pk: receiver,
                            amount,
                        }),
                    )
                };

                let signature = match sign_kind {
                    SignKind::Fake => Signature::dummy(),
                    SignKind::Real => {
                        // let tx = TransactionUnionPayload::of_user_command_payload(&payload);
                        // let signature_testnet = create "CodaSignature"
                        // let signature_mainnet = create "MinaSignatureMainnet"
                        // mina_signer::create_kimchi("CodaSignature")
                        //     .sign(sender_pk, &tx.to_input_legacy());

                        // TODO
                        Signature::dummy()
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
        use crate::staged_ledger::sparse_ledger::LedgerIntf;

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

    /// Run the given function inside of the Deferred monad, with a staged
    ///   ledger and a separate test ledger, after applying the given
    ///   init_state to both. In the below tests we apply the same commands to
    ///   the staged and test ledgers, and verify they are in the same state.
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2180
    fn async_with_given_ledger<F>(mask: Mask, fun: F)
    where
        F: Fn(StagedLedger, Mask),
    {
        let test_mask = mask.make_child();
        let sl = StagedLedger::create_exn(CONSTRAINT_CONSTANTS, mask).unwrap();
        fun(sl, test_mask.clone());
        test_mask.unregister_mask(crate::UnregisterBehavior::Check);
    }

    /// populate the ledger from an initial state before running the function
    ///
    /// Print the generated state when a panic occurs
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2192
    fn async_with_ledgers<F>(
        ledger_init_state: &LedgerInitialState,
        cmds: Vec<valid::UserCommand>,
        cmd_iters: Vec<Option<usize>>,
        fun: F,
    ) where
        F: Fn(StagedLedger, Mask) + std::panic::UnwindSafe,
    {
        match std::panic::catch_unwind(move || {
            let mut ephemeral_ledger =
                Mask::new_unattached(CONSTRAINT_CONSTANTS.ledger_depth as usize);

            apply_initialize_ledger_state(&mut ephemeral_ledger, ledger_init_state);
            async_with_given_ledger(ephemeral_ledger, fun);
        }) {
            Ok(_) => {}
            Err(_) => {
                eprintln!("state={:#?}", ledger_init_state);
                eprintln!("cmds[{}]={:#?}", cmds.len(), cmds);
                eprintln!("cmd_iters[{}]={:?}", cmd_iters.len(), cmd_iters);
                panic!("test failed (see logs above)");
            }
        }
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

                let cmds = util::drop(cmds, cmds_applied_count).to_vec();
                let counts_rest = &cmd_iters[1..];

                iter_cmds_acc(&cmds, counts_rest, acc, fun)
            }
        }
    }

    /// Same values when we run `dune runtest src/lib/staged_ledger -f`
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2142
    fn dummy_state_view(global_slot_since_genesis: Option<Slot>) -> ProtocolStateView {
        // TODO: Use OCaml implementation, not hardcoded value

        let f = |s: &str| Fp::from_str(s).unwrap();

        ProtocolStateView {
            snarked_ledger_hash: f("19095410909873291354237217869735884756874834695933531743203428046904386166496"),
            timestamp: BlockTime::from_u64(1600251300000),
            blockchain_length: Length::from_u32(1),
            min_window_density: Length::from_u32(77),
            last_vrf_output: (),
            total_currency: Amount::from_u64(10016100000000000),
            global_slot_since_hard_fork: Slot::from_u32(0),
            global_slot_since_genesis: global_slot_since_genesis.unwrap_or_else(Slot::zero),
            staking_epoch_data: EpochData {
                ledger: EpochLedger {
                    hash: f("19095410909873291354237217869735884756874834695933531743203428046904386166496"),
                    total_currency: Amount::from_u64(10016100000000000),
                },
                seed: Fp::zero(),
                start_checkpoint: Fp::zero(),
                lock_checkpoint: Fp::zero(),
                epoch_length: Length::from_u32(1),
            },
            next_epoch_data: EpochData {
                ledger: EpochLedger {
                    hash: f("19095410909873291354237217869735884756874834695933531743203428046904386166496"),
                    total_currency: Amount::from_u64(10016100000000000),
                },
                seed: f("18512313064034685696641580142878809378857342939026666126913761777372978255172"),
                start_checkpoint: Fp::zero(),
                lock_checkpoint: f("9196091926153144288494889289330016873963015481670968646275122329689722912273"),
                epoch_length: Length::from_u32(2),
            }
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2164
    fn create_and_apply<F>(
        coinbase_receiver: Option<CompressedPubKey>,
        winner: Option<CompressedPubKey>,
        sl: &mut StagedLedger,
        txns: &[valid::UserCommand],
        stmt_to_work: F,
    ) -> (
        Option<(LedgerProof, Vec<(WithStatus<Transaction>, Fp)>)>,
        Diff,
    )
    where
        F: Fn(&work::Statement) -> Option<work::Checked>,
    {
        let (ledger_proof, diff, _, _, _) = create_and_apply_with_state_body_hash(
            coinbase_receiver,
            winner,
            &dummy_state_view(None),
            (Fp::zero(), Fp::zero()),
            sl,
            txns,
            stmt_to_work,
        );
        (ledger_proof, diff)
    }

    /// Fee excess at top level ledger proofs should always be zero
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2377
    fn assert_fee_excess(proof: &Option<(LedgerProof, Vec<(WithStatus<Transaction>, Fp)>)>) {
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
    fn assert_ledger(
        test_ledger: Mask,
        coinbase_cost: Fee,
        staged_ledger: &StagedLedger,
        cmds_all: &[valid::UserCommand],
        cmds_used: usize,
        pks_to_check: &[AccountId],
    ) {
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

        for cmd in util::take(cmds_all, cmds_used) {
            let cmd = cmd.forget_check();
            let tx = Transaction::Command(cmd);

            apply_transaction(
                &CONSTRAINT_CONSTANTS,
                &dummy_state_view(None),
                &mut test_ledger,
                &tx,
            )
            .unwrap();
        }

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
                    .checked_add(&CONSTRAINT_CONSTANTS.account_creation_fee)
                    .unwrap()
            } else {
                coinbase_cost
            };

            let reward = CONSTRAINT_CONSTANTS
                .coinbase_amount
                .checked_sub(&Amount::of_fee(&total_cost))
                .unwrap();

            old_producer_balance.add_amount(reward).unwrap()
        };

        let new_producer_balance =
            get_account_exn(&staged_ledger.ledger, &producer_account_id).balance;

        assert!(new_producer_balance >= producer_balance_with_coinbase);
    }

    /// Generic test framework.
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2427
    fn test_simple<F>(
        account_ids_to_check: Vec<AccountId>,
        cmds: Vec<valid::UserCommand>,
        cmd_iters: Vec<Option<usize>>,
        mut sl: StagedLedger,
        // Number of ledger proofs expected
        expected_proof_count: Option<usize>,
        allow_failure: Option<bool>,
        test_mask: Mask,
        provers: NumProvers,
        stmt_to_work: &F,
    ) where
        F: Fn(&work::Statement) -> Option<work::Checked>,
    {
        eprintln!(
            "test_simple ncmds={:?} niters={:?}",
            cmds.len(),
            cmd_iters.len()
        );

        let allow_failure = allow_failure.unwrap_or(false);

        let mut niters = 0;

        let total_ledger_proofs = iter_cmds_acc(
            &cmds,
            &cmd_iters,
            0,
            |cmds_left, count_opt, cmds_this_iter, mut proof_count| {
                eprintln!("######## Start new batch {} ########", niters);
                eprintln!("nto_applied={:?}", cmds_this_iter.len());

                let (ledger_proof, diff) =
                    create_and_apply(None, None, &mut sl, cmds_this_iter, stmt_to_work);

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
                    &sl,
                    cmds_left,
                    cmds_applied_this_iter,
                    &account_ids_to_check,
                );

                eprintln!(
                    "######## Batch {} done: {} applied ########\n",
                    niters, cmds_applied_this_iter
                );

                niters += 1;

                (diff, proof_count)
            },
        );

        // Should have enough blocks to generate at least expected_proof_count
        // proofs
        if let Some(expected_proof_count) = expected_proof_count {
            debug_assert_eq!(total_ledger_proofs, expected_proof_count);
        };
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
            accum.extend_from_slice(&v.target.ledger.to_bytes());
            accum
        });
        let rng: Pcg64 = Seeder::from(&seed).make_rng();

        Keypair::rand(&mut MyRng(rng)).public.into_compressed()
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
        let prover = Keypair::rand(&mut rng).public.into_compressed();

        Some(work::Checked {
            fee: CONSTRAINT_CONSTANTS.account_creation_fee,
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

        let (ledger_init_state, cmds, iters) = gen_at_capacity_fixed_blocks(EXPECTED_PROOF_COUNT);

        async_with_ledgers(
            &ledger_init_state,
            cmds.clone(),
            iters.clone(),
            |sl, test_mask| {
                test_simple(
                    init_pks(&ledger_init_state),
                    cmds.to_vec(),
                    iters.to_vec(),
                    sl,
                    Some(EXPECTED_PROOF_COUNT),
                    None,
                    test_mask,
                    NumProvers::Many,
                    &stmt_to_work_random_prover,
                )
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
    fn max_throughput() {
        let (ledger_init_state, cmds, iters) = gen_at_capacity();

        async_with_ledgers(
            &ledger_init_state,
            cmds.clone(),
            iters.clone(),
            |sl, test_mask| {
                test_simple(
                    init_pks(&ledger_init_state),
                    cmds.to_vec(),
                    iters.to_vec(),
                    sl,
                    None,
                    None,
                    test_mask,
                    NumProvers::Many,
                    &stmt_to_work_random_prover,
                )
            },
        );
    }

    const MINIMUM_USER_COMMAND_FEE: Fee = Fee::from_u64(1000000);

    /// Value of `ledger_depth` when we run `dune runtest src/lib/staged_ledger -f`
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_generators/user_command_generators.ml#L15
    const LEDGER_DEPTH: usize = 35;

    fn zkapp_command_with_ledger(
        num_keypairs: Option<usize>,
        max_account_updates: Option<usize>,
        max_token_updates: Option<usize>,
        account_state_tbl: &mut HashSet<AccountId>,
        vk: Option<WithHash<VerificationKey>>,
        failure: Option<()>,
    ) {
        let mut rng = rand::thread_rng();

        // Need a fee payer keypair, a keypair for the "balancing" account (so that the balance changes
        // sum to zero), and max_account_updates * 2 keypairs, because all the other zkapp_command
        // might be new and their accounts not in the ledger; or they might all be old and in the ledger

        // We'll put the fee payer account and max_account_updates accounts in the
        // ledger, and have max_account_updates keypairs available for new accounts
        let max_account_updates = max_account_updates.unwrap_or(MAX_ACCOUNT_UPDATES);
        let max_token_updates = max_token_updates.unwrap_or(MAX_TOKEN_UPDATES);
        let num_keypairs =
            num_keypairs.unwrap_or((max_account_updates * 2) + (max_token_updates * 3) + 2);

        let keypairs: Vec<Keypair> = (0..num_keypairs).map(|_| gen_keypair()).collect();

        let keymap: HashMap<HashableCompressedPubKey, Keypair> = keypairs
            .iter()
            .map(|kp| {
                let compressed = kp.public.into_compressed();
                (HashableCompressedPubKey(compressed), kp.clone())
            })
            .collect();

        let num_keypairs_in_ledger = num_keypairs / 2;
        let keypairs_in_ledger = util::take(&keypairs, num_keypairs_in_ledger);

        let account_ids: Vec<AccountId> = keypairs_in_ledger
            .iter()
            .map(|Keypair { public, .. }| {
                AccountId::create(public.into_compressed(), TokenId::default())
            })
            .collect();

        let verification_key = vk.unwrap_or_else(|| {
            let dummy_vk = VerificationKey::dummy();
            let hash = dummy_vk.hash();
            WithHash {
                data: dummy_vk,
                hash,
            }
        });

        let balances: Vec<Balance> = {
            let min_cmd_fee = MINIMUM_USER_COMMAND_FEE;

            let min_balance = {
                let balance = min_cmd_fee.as_u64() + 100_000_000_000_000_000;
                Balance::from_u64(balance)
            };

            // max balance to avoid overflow when adding deltas
            let max_balance = {
                let max_bal = Balance::of_formatted_string("2000000000.0");

                assert_eq!(max_bal.as_u64(), 2000000000000000000);

                min_balance
                    .checked_add(&max_bal)
                    .expect("zkapp_command_with_ledger: overflow for max_balance")
            };

            (0..num_keypairs_in_ledger)
                .map(move |_| {
                    let balance = rng.gen_range(min_balance.as_u64()..max_balance.as_u64());
                    Balance::from_u64(balance)
                })
                .collect()
        };

        let account_ids_and_balances: Vec<(AccountId, Balance)> =
            account_ids.iter().cloned().zip(balances).collect();

        let snappify_account = |mut account: Account| {
            let permissions = Permissions {
                edit_state: AuthRequired::Either,
                send: AuthRequired::Either,
                set_delegate: AuthRequired::Either,
                set_permissions: AuthRequired::Either,
                set_verification_key: AuthRequired::Either,
                set_zkapp_uri: AuthRequired::Either,
                edit_sequence_state: AuthRequired::Either,
                set_token_symbol: AuthRequired::Either,
                increment_nonce: AuthRequired::Either,
                set_voting_for: AuthRequired::Either,
                //receive: AuthRequired::Either,
                ..Permissions::user_default()
            };

            let verification_key = Some(verification_key.data.clone());
            let zkapp = Some(ZkAppAccount {
                verification_key,
                ..ZkAppAccount::default()
            });

            account.zkapp = zkapp;
            account.permissions = permissions;

            account
        };

        // half zkApp accounts, half non-zkApp accounts
        let accounts =
            account_ids_and_balances
                .iter()
                .enumerate()
                .map(|(ndx, (account_id, balance))| {
                    let account = Account::create_with(account_id.clone(), *balance);
                    if ndx % 2 == 0 {
                        account
                    } else {
                        snappify_account(account)
                    }
                });

        let fee_payer_keypair = keypairs.first().unwrap();

        let mut ledger = Mask::create(LEDGER_DEPTH);

        account_ids.iter().zip(accounts).for_each(|(id, account)| {
            let res = ledger
                .get_or_create_account(id.clone(), account)
                .expect("zkapp_command: error adding account for account id");
            assert!(
                matches!(res, crate::GetOrCreated::Added(_)),
                "zkapp_command: account for account id already exists"
            );
        });

        // to keep track of account states across transactions
    }

    //   (* to keep track of account states across transactions *)
    //   let account_state_tbl =
    //     Option.value account_state_tbl ~default:(Account_id.Table.create ())
    //   in
    //   let%bind zkapp_command =
    //     Zkapp_command_generators.gen_zkapp_command_from ~max_account_updates
    //       ~max_token_updates ~fee_payer_keypair ~keymap ~ledger ~account_state_tbl
    //       ?vk ?failure ()
    //   in
    //   let zkapp_command =
    //     Option.value_exn
    //       (Zkapp_command.Valid.to_valid ~ledger ~get:Ledger.get
    //          ~location_of_account:Ledger.location_of_account zkapp_command )
    //   in
    //   (* include generated ledger in result *)
    //   return
    //     (User_command.Zkapp_command zkapp_command, fee_payer_keypair, keymap, ledger)

    /// keep max_account_updates small, so zkApp integration tests don't need lots
    /// of block producers
    /// because the other zkapp_command are split into a permissions-setter
    /// and another account_update, the actual number of other zkapp_command is
    /// twice this value, plus one, for the "balancing" account_update
    /// when we have separate transaction accounts in integration tests
    /// this number can be increased
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_generators/zkapp_command_generators.ml#L1111
    const MAX_ACCOUNT_UPDATES: usize = 2;

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_generators/zkapp_command_generators.ml#L1113
    const MAX_TOKEN_UPDATES: usize = 2;

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_generators/user_command_generators.ml#L146
    fn sequence_zkapp_command_with_ledger(
        max_account_updates: Option<usize>,
        max_token_updates: Option<usize>,
        length: Option<usize>,
        vk: Option<WithHash<VerificationKey>>,
        failure: Option<()>,
    ) {
        let mut rng = rand::thread_rng();

        let length = length.unwrap_or_else(|| rng.gen::<usize>() % 100);
        let max_account_updates = max_account_updates.unwrap_or(MAX_ACCOUNT_UPDATES);
        let max_token_updates = max_token_updates.unwrap_or(MAX_TOKEN_UPDATES);

        let num_keypairs = length * max_account_updates * 2;

        // Keep track of account states across multiple zkapp_command transaction
        let account_state_tbl = HashSet::<AccountId>::new();
    }

    //   let num_keypairs = length * max_account_updates * 2 in
    //   (* Keep track of account states across multiple zkapp_command transaction *)
    //   let account_state_tbl = Account_id.Table.create () in
    //   let%bind zkapp_command, fee_payer_keypair, keymap, ledger =
    //     zkapp_command_with_ledger ~num_keypairs ~max_account_updates
    //       ~max_token_updates ~account_state_tbl ?vk ?failure ()
    //   in
    //   let rec go zkapp_command_and_fee_payer_keypairs n =
    //     if n <= 1 then
    //       return
    //         ( (zkapp_command, fee_payer_keypair, keymap)
    //           :: List.rev zkapp_command_and_fee_payer_keypairs
    //         , ledger )
    //     else
    //       let%bind zkapp_command =
    //         Zkapp_command_generators.gen_zkapp_command_from ~max_account_updates
    //           ~max_token_updates ~fee_payer_keypair ~keymap ~ledger
    //           ~account_state_tbl ?vk ?failure ()
    //       in
    //       let valid_zkapp_command =
    //         Option.value_exn
    //           (Zkapp_command.Valid.to_valid ~ledger ~get:Ledger.get
    //              ~location_of_account:Ledger.location_of_account zkapp_command )
    //       in
    //       let zkapp_command_and_fee_payer_keypairs' =
    //         ( User_command.Zkapp_command valid_zkapp_command
    //         , fee_payer_keypair
    //         , keymap )
    //         :: zkapp_command_and_fee_payer_keypairs
    //       in
    //       go zkapp_command_and_fee_payer_keypairs' (n - 1)
    //   in
    //   go [] length

    static VK: Lazy<WithHash<VerificationKey>> = Lazy::new(|| {
        let vk = trivial_verification_key();
        let hash = vk.hash();

        assert_eq!(
            hash.to_decimal(),
            "19499466121496341533850180868238667461929019416054840058730806488105861126057"
        );

        WithHash { data: vk, hash }
    });

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2525
    fn gen_zkapps(_failure: Option<bool>, _num_zkapps: usize, _iters: usize) {
        // TODO (requires pickles)
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2571
    fn gen_zkapps_at_capacity() {
        // TODO (requires pickles)
    }

    /// Max throughput (zkapps)
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2664
    #[test]
    fn max_throughput_zkapps() {
        let vk = VK.clone();
        println!("VK={:#?}", vk);

        // TODO (requires pickles)
    }

    /// Max_throughput with zkApp transactions that may fail
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2675
    // #[test]
    fn max_throughput_zkapps_that_may_fail() {
        // TODO (requires pickles)
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

        let iters = rng.gen_range(1..=iters_max);

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

    /// Be able to include random number of commands
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2686
    #[test]
    fn be_able_to_include_random_number_of_commands_many() {
        let (ledger_init_state, cmds, iters) = gen_below_capacity(None);

        async_with_ledgers(
            &ledger_init_state,
            cmds.to_vec(),
            iters.to_vec(),
            |sl, test_mask| {
                test_simple(
                    init_pks(&ledger_init_state),
                    cmds.to_vec(),
                    iters.to_vec(),
                    sl,
                    None,
                    None,
                    test_mask,
                    NumProvers::Many,
                    &stmt_to_work_random_prover,
                )
            },
        );
    }

    /// Generate states that were known to fail
    ///
    /// See https://github.com/name-placeholder/ledger/commit/6de803f082ea986aa71e3cf30d7d83e54d2f5a3e
    fn gen_below_capacity_failed() -> (
        LedgerInitialState,
        Vec<valid::UserCommand>,
        Vec<Option<usize>>,
    ) {
        let state = gen_initial_ledger_state();
        let total_cmds = 1277;

        let cmds = signed_command_sequence(total_cmds, SignKind::Real, &state);
        assert_eq!(cmds.len(), total_cmds);

        let iters = [
            7, 17, 26, 35, 50, 13, 54, 12, 29, 54, 62, 36, 44, 44, 7, 8, 25, 8, 3, 42, 4, 46, 61,
            6, 60, 24, 34, 39, 9, 58, 23, 34, 10, 22, 15, 8, 4, 1, 42, 25, 5, 17, 60, 49, 45,
        ];

        (state, cmds, iters.into_iter().map(Some).collect())
    }

    /// This test was failing, due to incorrect discarding user command
    /// Note: Something interesting is that the batch 11 applies 0 commands
    ///
    /// See https://github.com/name-placeholder/ledger/commit/6de803f082ea986aa71e3cf30d7d83e54d2f5a3e
    #[test]
    fn be_able_to_include_random_number_of_commands_many_failed() {
        let (ledger_init_state, cmds, iters) = gen_below_capacity_failed();

        async_with_ledgers(
            &ledger_init_state,
            cmds.to_vec(),
            iters.to_vec(),
            |sl, test_mask| {
                test_simple(
                    init_pks(&ledger_init_state),
                    cmds.to_vec(),
                    iters.to_vec(),
                    sl,
                    None,
                    None,
                    test_mask,
                    NumProvers::Many,
                    &stmt_to_work_random_prover,
                )
            },
        );
    }

    /// Be able to include random number of commands (zkapps)
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2694
    // #[test]
    fn be_able_to_include_random_number_of_commands_zkapps() {
        // TODO (requires pickles)
    }

    /// Be able to include random number of commands (One prover)
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2704
    #[test]
    fn be_able_to_include_random_number_of_commands_one_prover() {
        let (ledger_init_state, cmds, iters) = gen_below_capacity(None);

        async_with_ledgers(
            &ledger_init_state,
            cmds.to_vec(),
            iters.to_vec(),
            |sl, test_mask| {
                test_simple(
                    init_pks(&ledger_init_state),
                    cmds.to_vec(),
                    iters.to_vec(),
                    sl,
                    None,
                    None,
                    test_mask,
                    NumProvers::One,
                    &stmt_to_work_random_prover,
                )
            },
        );
    }

    /// This test was failing, due to incorrect discarding user command
    ///
    /// See https://github.com/name-placeholder/ledger/commit/6de803f082ea986aa71e3cf30d7d83e54d2f5a3e
    #[test]
    fn be_able_to_include_random_number_of_commands_one_prover_failed() {
        let (ledger_init_state, cmds, iters) = gen_below_capacity_failed();

        async_with_ledgers(
            &ledger_init_state,
            cmds.to_vec(),
            iters.to_vec(),
            |sl, test_mask| {
                test_simple(
                    init_pks(&ledger_init_state),
                    cmds.to_vec(),
                    iters.to_vec(),
                    sl,
                    None,
                    None,
                    test_mask,
                    NumProvers::One,
                    &stmt_to_work_random_prover,
                )
            },
        );
    }

    /// Be able to include random number of commands (One prover, zkapps)
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2712
    // #[test]
    fn be_able_to_include_random_number_of_commands_one_prover_zkapps() {
        // TODO (requires pickles)
    }

    /// Fixed public key for when there is only one snark worker.
    static SNARK_WORKER_PK: Lazy<CompressedPubKey> =
        Lazy::new(|| gen_keypair().public.into_compressed());

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

        let (ledger_init_state, cmds, iters) = gen_at_capacity_fixed_blocks(EXPECTED_PROOF_COUNT);

        async_with_ledgers(
            &ledger_init_state,
            cmds.clone(),
            iters.clone(),
            |sl, test_mask| {
                test_simple(
                    init_pks(&ledger_init_state),
                    cmds.to_vec(),
                    iters.to_vec(),
                    sl,
                    Some(EXPECTED_PROOF_COUNT),
                    None,
                    test_mask.clone(),
                    NumProvers::One,
                    &stmt_to_work_zero_fee,
                );

                assert!(test_mask.location_of_account(&account_id_prover).is_none());
            },
        );
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2745
    fn compute_statutes(
        ledger: Mask,
        coinbase_amount: Amount,
        diff: (
            PreDiffTwo<work::Work, WithStatus<UserCommand>>,
            Option<PreDiffOne<work::Work, WithStatus<UserCommand>>>,
        ),
    ) -> (
        PreDiffTwo<work::Work, WithStatus<UserCommand>>,
        Option<PreDiffOne<work::Work, WithStatus<UserCommand>>>,
    ) {
        // Fill in the statuses for commands.
        let mut status_ledger = HashlessLedger::create(ledger);

        let mut generate_status = |txn: Transaction| -> Result<TransactionStatus, String> {
            status_ledger.apply_transaction(&CONSTRAINT_CONSTANTS, &dummy_state_view(None), &txn)
        };

        pre_diff_info::compute_statuses::<UserCommand, valid::Transaction, _>(
            &CONSTRAINT_CONSTANTS,
            diff,
            COINBASE_RECEIVER.clone(),
            coinbase_amount,
            &mut generate_status,
        )
        .unwrap()
    }

    /// Invalid diff test: check zero fee excess for partitions
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2761
    #[test]
    fn check_zero_fee_excess_for_partitions() {
        let create_diff_with_non_zero_fee_excess =
            |ledger: Mask,
             coinbase_amount: Amount,
             txns: Vec<WithStatus<UserCommand>>,
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
                            diff: compute_statutes(ledger, coinbase_amount, (a, Some(b))),
                        }
                    }
                }
            };

        let empty_diff = Diff::empty();

        let (ledger_init_state, cmds, iters) = gen_at_capacity();

        async_with_ledgers(
            &ledger_init_state,
            cmds.clone(),
            iters.clone(),
            |mut sl, _test_mask| {
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

                        let cmds_this_iter: Vec<WithStatus<UserCommand>> = cmds_this_iter
                            .iter()
                            .map(|cmd| WithStatus {
                                data: cmd.forget_check(),
                                status: TransactionStatus::Applied,
                            })
                            .collect();

                        let diff = create_diff_with_non_zero_fee_excess(
                            sl.ledger.clone(),
                            CONSTRAINT_CONSTANTS.coinbase_amount,
                            cmds_this_iter,
                            work_done,
                            partitions,
                        );

                        let apply_res = sl.apply(
                            None,
                            &CONSTRAINT_CONSTANTS,
                            diff.clone(),
                            (),
                            &Verifier,
                            &dummy_state_view(None),
                            (Fp::zero(), Fp::zero()),
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

    const WORK_FEE: Fee = CONSTRAINT_CONSTANTS.account_creation_fee;

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

        async_with_ledgers(
            &ledger_init_state,
            cmds.to_vec(),
            iters.to_vec(),
            |sl, _test_mask| {
                iter_cmds_acc(
                    &cmds,
                    &iters,
                    (),
                    |_cmds_left, _count_opt, cmds_this_iter, _| {
                        let (diff, _invalid_txns) = sl
                            .create_diff(
                                &CONSTRAINT_CONSTANTS,
                                None,
                                COINBASE_RECEIVER.clone(),
                                LOGGER,
                                &dummy_state_view(None),
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
                eprintln!("######## Start new batch {} ########", niters);
                eprintln!("nto_applied={:?}", cmds_this_iter.len());

                let work_list = sl.scan_state.all_work_statements_exn();

                let proofs_available_this_iter = *proofs_available_left.first().unwrap();

                let (proof, diff) = create_and_apply(
                    None,
                    None,
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
            |sl, test_mask| {
                test_random_number_of_proofs(
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

        let proofs_available: Vec<usize> = iters
            .iter()
            .map(|cmds_opt| rng.gen_range(0..(3 * cmds_opt.unwrap())))
            .collect();

        async_with_ledgers(
            &ledger_init_state,
            cmds.clone(),
            iters.clone(),
            |sl, test_mask| {
                test_random_number_of_proofs(
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

        let proofs_available: Vec<usize> = iters
            .iter()
            .map(|cmds_opt| rng.gen_range(0..(3 * cmds_opt.unwrap())))
            .collect();

        async_with_ledgers(
            &ledger_init_state,
            cmds.clone(),
            iters.clone(),
            |sl, test_mask| {
                test_random_number_of_proofs(
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
                eprintln!("######## Start new batch {} ########", niters);
                eprintln!("nto_applied={:?}", cmds_this_iter.len());

                let work_list = sl.scan_state.all_work_statements_exn();

                let (proofs_available_this_iter, fees_for_each) =
                    proofs_available_left.first().unwrap();
                let proofs_available_this_iter = *proofs_available_this_iter;

                let work_to_be_done = {
                    let work_list = util::take(&work_list, proofs_available_this_iter).to_vec();
                    let fees = util::take(fees_for_each, work_list.len()).to_vec();
                    work_list.into_iter().zip(fees).collect::<Vec<_>>()
                };

                let (_proof, diff) = create_and_apply(
                    None,
                    None,
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
                        .unwrap_or_else(Vec::new)
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
            |sl, test_mask| {
                test_random_proof_fee(
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
            |sl, test_mask| {
                test_random_proof_fee(
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
        init: &LedgerInitialState,
        cmds: Vec<valid::UserCommand>,
        cmd_iters: Vec<Option<usize>>,
        proof_available: Vec<usize>,
        state_body_hashes: Vec<(Fp, Fp)>,
        current_state_view: &ProtocolStateView,
        mut sl: StagedLedger,
        test_mask: Mask,
        provers: NumProvers,
    ) {
        let (proofs_available_left, _state_body_hashes_left) = iter_cmds_acc(
            &cmds,
            &cmd_iters,
            (proof_available, state_body_hashes),
            |cmds_left,
             _count_opt,
             cmds_this_iter,
             (mut proofs_available_left, mut state_body_hashes)| {
                let work_list = sl.scan_state.all_work_statements_exn();
                let proofs_available_this_iter = proofs_available_left[0];

                let state_body_hash = state_body_hashes[0];

                let (proof, diff, _is_new_stack, _pc_update, _supercharge_coinbase) =
                    create_and_apply_with_state_body_hash(
                        None,
                        None,
                        current_state_view,
                        state_body_hash,
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
                    &sl,
                    cmds_left,
                    cmds_applied_this_iter,
                    &init_pks(init),
                );

                proofs_available_left.remove(0);
                state_body_hashes.remove(0);

                (diff, (proofs_available_left, state_body_hashes))
            },
        );

        assert!(proofs_available_left.is_empty());
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3348
    fn pending_coinbase_test(prover: NumProvers) {
        let mut rng = rand::thread_rng();

        let (ledger_init_state, cmds, iters) = gen_below_capacity(Some(true));

        let proofs_available: Vec<usize> = iters
            .iter()
            .map(|cmds_opt| rng.gen_range(0..(3 * cmds_opt.unwrap())))
            .collect();

        let state_body_hashes: Vec<(Fp, Fp)> = iters
            .iter()
            .map(|_| (Fp::rand(&mut rng), Fp::rand(&mut rng)))
            .collect();

        let current_state_view = dummy_state_view(None);

        async_with_ledgers(
            &ledger_init_state,
            cmds.clone(),
            iters.clone(),
            |sl, test_mask| {
                test_pending_coinbase(
                    &ledger_init_state,
                    cmds.clone(),
                    iters.clone(),
                    proofs_available.clone(),
                    state_body_hashes.clone(),
                    &current_state_view,
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
            vesting_period: Slot::from_u32(2),
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
            create_and_apply_with_state_body_hash(
                Some(coinbase_receiver.public_key.clone()),
                Some(delegator.public_key.clone()),
                &dummy_state_view(Some(Slot::from_u32(block_count.try_into().unwrap()))),
                (Fp::zero(), Fp::zero()),
                sl,
                &[],
                stmt_to_work_zero_fee(this.public_key.clone()),
            );
            check_receiver_account(sl, block_count);
        });
    }

    const NORMAL_COINBASE: Amount = CONSTRAINT_CONSTANTS.coinbase_amount;

    const fn scale_exn(amount: Amount, i: u64) -> Amount {
        match amount.scale(i) {
            Some(amount) => amount,
            None => panic!(),
        }
    }

    const SUPERCHARGED_COINBASE: Amount = scale_exn(
        CONSTRAINT_CONSTANTS.coinbase_amount,
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

        async_with_ledgers(&ledger_init_state, vec![], vec![], |mut sl, _test_mask| {
            supercharge_coinbase_test(
                this.clone(),
                this.clone(),
                block_count,
                f_expected_balance,
                &mut sl,
            )
        });
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

        async_with_ledgers(&ledger_init_state, vec![], vec![], |mut sl, _test_mask| {
            supercharge_coinbase_test(
                locked_this.clone(),
                unlocked_delegator.clone(),
                block_count,
                f_expected_balance,
                &mut sl,
            )
        });
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

        async_with_ledgers(&ledger_init_state, vec![], vec![], |mut sl, _test_mask| {
            supercharge_coinbase_test(
                unlocked_this.clone(),
                locked_delegator.clone(),
                block_count,
                f_expected_balance,
                &mut sl,
            )
        });
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

        async_with_ledgers(&ledger_init_state, vec![], vec![], |mut sl, _test_mask| {
            supercharge_coinbase_test(
                locked_this.clone(),
                locked_delegator.clone(),
                block_count,
                f_expected_balance,
                &mut sl,
            )
        });
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3612
    fn command_insufficient_funds() -> (LedgerInitialState, valid::UserCommand) {
        let ledger_initial_state = gen_initial_ledger_state();
        let (kp, balance, nonce, _) = &ledger_initial_state.state[0];

        let receiver_pk = gen_keypair().public.into_compressed();

        let insufficient_account_creation_fee =
            Amount::from_u64(CONSTRAINT_CONSTANTS.account_creation_fee.as_u64() / 2);

        let source_pk = kp.public.into_compressed();

        let body = signed_command::Body::Payment(PaymentPayload {
            source_pk: source_pk.clone(),
            receiver_pk,
            amount: insufficient_account_creation_fee,
        });
        let fee = Fee::from_u64(balance.as_u64());

        let payload =
            SignedCommandPayload::create(fee, source_pk, *nonce, None, Memo::dummy(), body);

        let payload_to_sign = TransactionUnionPayload::of_user_command_payload(&payload);

        let mut signer = mina_signer::create_legacy(mina_signer::NetworkId::TESTNET);
        let _signature = signer.sign(kp, &payload_to_sign);

        let signed_command = SignedCommand {
            payload,
            signer: kp.public.into_compressed(),
            signature: Signature::dummy(), // TODO: Use `_signature` above
        };

        let cmd = valid::UserCommand::SignedCommand(Box::new(signed_command));
        (ledger_initial_state, cmd)
    }

    /// Commands with Insufficient funds are not included
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L3643
    #[test]
    fn commands_with_insufficient_funds_are_not_included() {
        let (ledger_init_state, invalid_commands) = command_insufficient_funds();

        async_with_ledgers(
            &ledger_init_state,
            vec![invalid_commands.clone()],
            vec![],
            |sl, _test_mask| {
                let (diff, _invalid_txns) = sl
                    .create_diff(
                        &CONSTRAINT_CONSTANTS,
                        None,
                        COINBASE_RECEIVER.clone(),
                        (),
                        &dummy_state_view(None),
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
        // let (kp, balance, nonce, _) = &ledger_initial_state.state[0];

        let command = |kp: Keypair, balance: Amount, nonce: Nonce, validity: Validity| {
            let receiver_pk = gen_keypair().public.into_compressed();

            let (account_creation_fee, fee) = {
                match validity {
                    Validity::Valid => {
                        let account_creation_fee =
                            Amount::of_fee(&CONSTRAINT_CONSTANTS.account_creation_fee);
                        let fee = balance.checked_sub(&account_creation_fee).unwrap();
                        (account_creation_fee, Fee::from_u64(fee.as_u64()))
                    }
                    Validity::Invalid => {
                        // Not enough account creation fee and using full balance for fee
                        let account_creation_fee =
                            CONSTRAINT_CONSTANTS.account_creation_fee.as_u64() / 2;
                        let account_creation_fee = Amount::from_u64(account_creation_fee);
                        (account_creation_fee, Fee::from_u64(balance.as_u64()))
                    }
                }
            };

            let source_pk = kp.public.into_compressed();
            let body = signed_command::Body::Payment(PaymentPayload {
                source_pk: source_pk.clone(),
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
            let _signature = signer.sign(&kp, &payload_to_sign);

            let signed_command = SignedCommand {
                payload,
                signer: kp.public.into_compressed(),
                signature: Signature::dummy(), // TODO: Use `_signature` above
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
            |mut sl, _test_mask| {
                let (diff, _invalid_txns) = sl
                    .create_diff(
                        &CONSTRAINT_CONSTANTS,
                        None,
                        COINBASE_RECEIVER.clone(),
                        (),
                        &dummy_state_view(None),
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
                    diff.forget(),
                    (),
                    &Verifier,
                    &dummy_state_view(None),
                    (Fp::zero(), Fp::zero()),
                    COINBASE_RECEIVER.clone(),
                    false,
                );

                let expected = format!(
                    "Error when applying transaction: {:?}",
                    TransactionFailure::SourceInsufficientBalance.to_string()
                );

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

        let pks: HashSet<_> = test_spec
            .init_ledger
            .0
            .iter()
            .map(|(kp, _)| HashableCompressedPubKey(kp.public.into_compressed()))
            .collect();

        let kp = loop {
            let keypair = gen_keypair();
            if !pks.contains(&HashableCompressedPubKey(keypair.public.into_compressed())) {
                break keypair;
            }
        };

        let TestSpec {
            init_ledger,
            specs: _,
        } = test_spec;
        let new_kp = kp;

        let fee = Fee::from_u64(1_000_000);
        let amount = Amount::from_u64(10_000_000_000);

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
            sequence_events: vec![],
            events: vec![],
            call_data: Fp::zero(),
            preconditions: None,
        };

        let mut ledger = Mask::new_unattached(CONSTRAINT_CONSTANTS.ledger_depth as usize);

        init_ledger.init(&mut ledger);

        // create a snapp account
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
            let mut new_mask = ledger.make_child();
            // for_tests::create_trivial_zkapp_account(Some(snapp_permissions), vk, &mut new_mask, snapp_pk);
        };
    }
}
