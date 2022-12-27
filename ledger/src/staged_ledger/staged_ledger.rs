use mina_hasher::Fp;
use mina_p2p_messages::v2::MinaStateProtocolStateValueStableV2;

use crate::{
    scan_state::{
        currency::{Amount, Fee, Magnitude},
        pending_coinbase::{PendingCoinbase, Stack, StackState},
        scan_state::{
            transaction_snark::{
                work, InitStack, LedgerHash, LedgerProofWithSokMessage, OneOrTwo, Registers,
                Statement, TransactionWithWitness,
            },
            ConstraintConstants, ScanState, StatementCheck, Verifier,
        },
        snark_work::spec,
        transaction_logic::{
            apply_transaction,
            local_state::LocalState,
            protocol_state::{protocol_state_view, ProtocolStateView},
            transaction_applied::TransactionApplied,
            Transaction, TransactionStatus, WithStatus,
        },
    },
    AccountId, BaseLedger, Mask, TokenId,
};

use super::sparse_ledger::SparseLedger;

/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/staged_ledger.ml#470
#[derive(Clone, Debug)]
pub struct StackStateWithInitStack {
    pub pc: StackState,
    pub init_stack: Stack,
}

pub enum StagedLedgerError {
    NonZeroFeeExcess,
    InvalidProofs,
    CouldntReachVerifier,
    PreDiff,
    InsufficientWork,
    MismatchedStatuses {
        transaction: WithStatus<Transaction>,
        got: TransactionStatus,
    },
    InvalidPublicKey,
    Unexpected(String),
}

impl From<String> for StagedLedgerError {
    fn from(value: String) -> Self {
        Self::Unexpected(value)
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
                tx.data,
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
    fn hash(&self) -> Fp {
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
        pending_coinbase_collection: PendingCoinbase,
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
    fn coinbase_amount(
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
        transaction: Transaction,
        txn_state_view: &ProtocolStateView,
    ) -> Result<(TransactionApplied, Statement, StackStateWithInitStack), StagedLedgerError> {
        let fee_excess = transaction.fee_excess()?;

        let source_merkle_root = ledger.merkle_root();

        let pending_coinbase_target =
            Self::push_coinbase(pending_coinbase_stack_state.pc.target, &transaction);

        let new_init_stack =
            Self::push_coinbase(pending_coinbase_stack_state.init_stack, &transaction);

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

    fn apply_transaction_and_get_witness(
        constraint_constants: &ConstraintConstants,
        mut ledger: Mask,
        pending_coinbase_stack_state: StackStateWithInitStack,
        transaction: Transaction,
        status: Option<TransactionStatus>,
        txn_state_view: &ProtocolStateView,
        state_and_body_hash: (Fp, Fp),
    ) -> Result<(), StagedLedgerError> {
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
            SparseLedger::of_ledger_subset_exn(ledger.clone(), &account_ids(&transaction));

        let (applied_txn, statement, updated_pending_coinbase_stack_state) =
            Self::apply_transaction_and_get_statement(
                constraint_constants,
                ledger,
                pending_coinbase_stack_state.clone(),
                transaction,
                txn_state_view,
            )?;

        if let Some(status) = status.as_ref() {
            let got_status = applied_txn.transaction_status();

            if status != got_status {
                return Err(StagedLedgerError::MismatchedStatuses {
                    transaction: applied_txn.transaction(),
                    got: got_status.clone(),
                });
            }
        };

        TransactionWithWitness {
            transaction_with_info: applied_txn,
            state_hash: state_and_body_hash,
            statement,
            init_stack: InitStack::Base(pending_coinbase_stack_state.init_stack),
            ledger_witness: todo!(),
        };

        Ok(())
    }
}

// let apply_transaction_and_get_witness ~constraint_constants ledger
//     pending_coinbase_stack_state s status txn_state_view state_and_body_hash =
//   (* Core.Printf.eprintf "MY_LOG.APPLY_TRANSACTION_AND_GET_WITNESS\n%!" ; *)
//   let open Deferred.Result.Let_syntax in
//   let account_ids : Transaction.t -> _ = function
//     | Fee_transfer t ->
//         Fee_transfer.receivers t
//     | Command t ->
//         let t = (t :> User_command.t) in
//         User_command.accounts_referenced t
//     | Coinbase c ->
//         let ft_receivers =
//           Option.map ~f:Coinbase.Fee_transfer.receiver c.fee_transfer
//           |> Option.to_list
//         in
//         Account_id.create c.receiver Token_id.default :: ft_receivers
//   in
//   (* Core.Printf.eprintf "MY_LOG.APPLY_TRANSACTION_AND_GET_WITNESS_111\n%!" ; *)
//   let ledger_witness =
//     O1trace.sync_thread "create_ledger_witness" (fun () ->
//         Sparse_ledger.of_ledger_subset_exn ledger (account_ids s) )
//   in
//   (* Core.Printf.eprintf "MY_LOG.APPLY_TRANSACTION_AND_GET_WITNESS_222\n%!" ; *)
//   let%bind () = yield_result () in

//   let%bind applied_txn, statement, updated_pending_coinbase_stack_state =
//     O1trace.sync_thread "apply_transaction_to_scan_state" (fun () ->
//         apply_transaction_and_get_statement ~constraint_constants ledger
//           pending_coinbase_stack_state s txn_state_view )
//     |> Deferred.return
//   in
//   (* Core.Printf.eprintf "MY_LOG.APPLY_TRANSACTION_AND_GET_STATEMENT.AAA\n%!" ; *)
//   let%bind () = yield_result () in
//   (* Core.Printf.eprintf "MY_LOG.APPLY_TRANSACTION_AND_GET_STATEMENT.BBB\n%!" ; *)
//   let%map () =
//     match status with
//     | None ->
//         (* Core.Printf.eprintf *)
//         (*   "MY_LOG.APPLY_TRANSACTION_AND_GET_STATEMENT.NONE\n%!" ; *)
//         return ()
//     | Some status ->
//         (* Validate that command status matches. *)
//         let got_status =
//           Ledger.Transaction_applied.transaction_status applied_txn
//         in
//         (* Core.Printf.eprintf *)
//         (*   "MY_LOG.APPLY_TRANSACTION_AND_GET_STATEMENT STATUS=%s GOT_STATUS=%s\n\ *)
//         (*    %!" *)
//         (*   ( match status with *)
//         (*   | Transaction_status.Applied -> *)
//         (*       "applied" *)
//         (*   | Transaction_status.Failed _e -> *)
//         (*       "failed" ) *)
//         (*   ( match got_status with *)
//         (*   | Transaction_status.Applied -> *)
//         (*       "applied" *)
//         (*   | Transaction_status.Failed _ -> *)
//         (*       "failed" ) ; *)
//         if Transaction_status.equal status got_status then (
//           (* Core.Printf.eprintf *)
//           (*   "MY_LOG.APPLY_TRANSACTION_AND_GET_STATEMENT.EQUAL\n%!" ; *)
//           return () )
//         else
//           Deferred.Result.fail
//             (Staged_ledger_error.Mismatched_statuses
//                ({ With_status.data = s; status }, got_status) )
//   in
//   (* Core.Printf.eprintf "MY_LOG.APPLY_TRANSACTION_AND_GET_STATEMENT.SUCCESS\n%!" ; *)
//   ( { Scan_state.Transaction_with_witness.transaction_with_info = applied_txn
//     ; state_hash = state_and_body_hash
//     ; ledger_witness
//     ; init_stack = Base pending_coinbase_stack_state.init_stack
//     ; statement
//     }
//   , updated_pending_coinbase_stack_state )
