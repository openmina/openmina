use mina_hasher::Fp;
use mina_p2p_messages::v2::MinaStateProtocolStateValueStableV2;

use crate::{
    scan_state::{
        pending_coinbase::{PendingCoinbase, Stack},
        scan_state::{
            transaction_snark::{work, LedgerHash, LedgerProofWithSokMessage, OneOrTwo, Registers},
            ConstraintConstants, ScanState, StatementCheck, Verifier,
        },
        snark_work::spec,
        transaction_logic::{local_state::LocalState, Transaction, WithStatus},
    },
    BaseLedger, Mask,
};

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

    fn of_scan_state_and_ledger_unchecked(
        ledger: Mask,
        scan_state: ScanState,
        constraint_constants: ConstraintConstants,
        pending_coinbase_collection: PendingCoinbase,
    ) -> Self {
        Self {
            scan_state,
            ledger,
            constraint_constants,
            pending_coinbase_collection,
        }
    }

    fn of_scan_state_and_ledger<F>(
        _logger: (),
        constraint_constants: ConstraintConstants,
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
            &constraint_constants,
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

        Ok(Self::of_scan_state_and_ledger_unchecked(
            ledger,
            scan_state,
            constraint_constants,
            pending_coinbase_collection,
        ))
    }
}
