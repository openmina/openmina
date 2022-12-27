use std::collections::HashSet;

use mina_hasher::Fp;
use mina_p2p_messages::v2::MinaStateProtocolStateValueStableV2;
use mina_signer::CompressedPubKey;

use crate::{
    scan_state::{
        fee_excess::FeeExcess,
        parallel_scan::{base, merge, JobStatus},
        pending_coinbase,
        scan_state::transaction_snark::{
            LedgerProofWithSokMessage, SokMessage, Statement, TransactionWithWitness,
        },
    },
    staged_ledger::sparse_ledger::SparseLedger,
    Account, AccountId,
};

use self::transaction_snark::{InitStack, LedgerProof, OneOrTwo, Registers};

use super::{
    currency::{Amount, Fee},
    parallel_scan::ParallelScan,
    snark_work,
    transaction_logic::{
        apply_transaction, local_state::LocalState, protocol_state::protocol_state_view,
        transaction_applied::TransactionApplied, transaction_witness::TransactionWitness,
        Transaction, WithStatus,
    },
};
// use super::parallel_scan::AvailableJob;

pub use super::parallel_scan::SpacePartition;

// type LedgerProof = LedgerProofProdStableV2;
// type LedgerProofWithSokMessage = TransactionSnarkScanStateLedgerProofWithSokMessageStableV2;
// type TransactionWithWitness = TransactionSnarkScanStateTransactionWithWitnessStableV2;

pub type AvailableJob = super::parallel_scan::AvailableJob<
    transaction_snark::TransactionWithWitness,
    transaction_snark::LedgerProofWithSokMessage,
>;

#[derive(Clone)]
pub struct ScanState {
    state: ParallelScan<
        transaction_snark::TransactionWithWitness,
        transaction_snark::LedgerProofWithSokMessage,
    >,
}

pub mod transaction_snark {
    use mina_hasher::Fp;
    use mina_p2p_messages::v2::TransactionSnarkProofStableV2;
    use mina_signer::CompressedPubKey;

    use crate::{
        scan_state::{
            currency::{Amount, Signed},
            fee_excess::FeeExcess,
            pending_coinbase,
            transaction_logic::{local_state::LocalState, transaction_applied::TransactionApplied},
        },
        staged_ledger::sparse_ledger::SparseLedger,
        Account, AccountId,
    };

    use super::Fee;

    pub type LedgerHash = Fp;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Registers {
        pub ledger: LedgerHash,
        pub pending_coinbase_stack: pending_coinbase::Stack,
        pub local_state: LocalState,
    }

    impl Registers {
        /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_snark/transaction_snark.ml#L350
        pub fn check_equal(&self, other: &Self) -> bool {
            self.ledger == other.ledger
                && self.local_state == other.local_state
                && pending_coinbase::Stack::connected(
                    &self.pending_coinbase_stack,
                    &other.pending_coinbase_stack,
                    None,
                )
        }

        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_state/registers.ml#L47
        pub fn connected(r1: &Self, r2: &Self) -> bool {
            r1.ledger == r2.ledger
                && r1.local_state == r2.local_state
                && pending_coinbase::Stack::connected(
                    &r1.pending_coinbase_stack,
                    &r2.pending_coinbase_stack,
                    None,
                )
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Statement {
        pub source: Registers,
        pub target: Registers,
        pub supply_increase: Signed<Amount>,
        pub fee_excess: FeeExcess,
        pub sok_digest: Option<Vec<u8>>,
    }

    impl Statement {
        /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_snark/transaction_snark.ml#L348
        pub fn merge(&self, s2: &Statement) -> Result<Self, String> {
            let fee_excess = FeeExcess::combine(&self.fee_excess, &s2.fee_excess)?;
            let supply_increase = self
                .supply_increase
                .add(&s2.supply_increase)
                .ok_or_else(|| "Error adding supply_increase".to_string())?;

            assert!(self.target.check_equal(&s2.source));

            Ok(Self {
                source: self.source.clone(),
                target: s2.target.clone(),
                supply_increase,
                fee_excess,
                sok_digest: None,
            })
        }
    }

    pub mod work {
        use super::*;

        pub type Statement = OneOrTwo<super::Statement>;

        pub struct Work {
            pub fee: Fee,
            pub proofs: OneOrTwo<LedgerProof>,
            pub prover: CompressedPubKey,
        }

        pub type Unchecked = Work;
    }

    // TransactionSnarkPendingCoinbaseStackStateInitStackStableV1
    #[derive(Debug, Clone)]
    pub enum InitStack {
        Base(pending_coinbase::Stack),
        Merge,
    }

    #[derive(Debug, Clone)]
    pub struct TransactionWithWitness {
        pub transaction_with_info: TransactionApplied,
        pub state_hash: (Fp, Fp), // (StateHash, StateBodyHash)
        // pub state_hash: (StateHash, MinaBaseStateBodyHashStableV1),
        pub statement: Statement,
        pub init_stack: InitStack,
        pub ledger_witness: SparseLedger<AccountId, Account>,
    }

    #[derive(Debug, Clone)]
    pub struct TransactionSnark {
        pub statement: Statement,
        pub proof: TransactionSnarkProofStableV2,
    }

    #[derive(Debug, Clone)]
    pub struct LedgerProof(pub TransactionSnark);

    impl LedgerProof {
        pub fn create(statement: Statement, proof: TransactionSnarkProofStableV2) -> Self {
            Self(TransactionSnark { statement, proof })
        }

        pub fn statement(&self) -> &Statement {
            &self.0.statement
        }
    }

    #[derive(Debug, Clone)]
    pub struct SokMessage {
        pub fee: Fee,
        pub prover: CompressedPubKey,
    }

    impl SokMessage {
        pub fn create(fee: Fee, prover: CompressedPubKey) -> Self {
            Self { fee, prover }
        }
    }

    #[derive(Debug, Clone)]
    pub struct LedgerProofWithSokMessage {
        pub proof: LedgerProof,
        pub sok_message: SokMessage,
    }

    #[derive(Debug, Clone)]
    pub enum OneOrTwo<T> {
        One(T),
        Two((T, T)),
    }

    impl<T> OneOrTwo<T> {
        pub fn len(&self) -> usize {
            match self {
                OneOrTwo::One(_) => 1,
                OneOrTwo::Two(_) => 2,
            }
        }

        pub fn iter(&self) -> OneOrTwoIter<T> {
            let array = match self {
                OneOrTwo::One(a) => [Some(a), None],
                OneOrTwo::Two((a, b)) => [Some(a), Some(b)],
            };

            OneOrTwoIter {
                inner: array,
                index: 0,
            }
        }

        pub fn into_iter(self) -> OneOrTwoIntoIter<T> {
            let array = match self {
                OneOrTwo::One(a) => [Some(a), None],
                OneOrTwo::Two((a, b)) => [Some(a), Some(b)],
            };

            OneOrTwoIntoIter {
                inner: array,
                index: 0,
            }
        }

        pub fn map<F, R>(&self, fun: F) -> OneOrTwo<R>
        where
            F: Fn(&T) -> R,
        {
            match self {
                OneOrTwo::One(one) => OneOrTwo::One(fun(one)),
                OneOrTwo::Two((a, b)) => OneOrTwo::Two((fun(a), fun(b))),
            }
        }

        pub fn into_map<F, R>(self, fun: F) -> OneOrTwo<R>
        where
            F: Fn(T) -> R,
        {
            match self {
                OneOrTwo::One(one) => OneOrTwo::One(fun(one)),
                OneOrTwo::Two((a, b)) => OneOrTwo::Two((fun(a), fun(b))),
            }
        }

        pub fn into_map_err<F, R, E>(self, fun: F) -> Result<OneOrTwo<R>, E>
        where
            F: Fn(T) -> Result<R, E>,
        {
            match self {
                OneOrTwo::One(one) => Ok(OneOrTwo::One(fun(one)?)),
                OneOrTwo::Two((a, b)) => Ok(OneOrTwo::Two((fun(a)?, fun(b)?))),
            }
        }
    }

    pub struct OneOrTwoIter<'a, T> {
        inner: [Option<&'a T>; 2],
        index: usize,
    }

    impl<'a, T> Iterator for OneOrTwoIter<'a, T> {
        type Item = &'a T;

        fn next(&mut self) -> Option<Self::Item> {
            let value = self.inner.get(self.index)?.as_ref()?;
            self.index += 1;

            Some(value)
        }
    }

    pub struct OneOrTwoIntoIter<T> {
        inner: [Option<T>; 2],
        index: usize,
    }

    impl<T> Iterator for OneOrTwoIntoIter<T> {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            let value = self.inner.get_mut(self.index)?.take()?;
            self.index += 1;

            Some(value)
        }
    }
}

impl ScanState {
    pub fn hash(&self) -> Fp {
        todo!()

        // use binprot::BinProtWrite;
        // self.state.hash(
        //     |buffer, proof| {
        //         proof.binprot_write(buffer).unwrap();
        //     },
        //     |buffer, transaction| {
        //         transaction.binprot_write(buffer).unwrap();
        //     },
        // );
    }
}

#[derive(Clone, Debug)]
pub struct ForkConstants {
    previous_state_hash: Fp,   // Pickles.Backend.Tick.Field.Stable.Latest.t,
    previous_length: u32,      // Mina_numbers.Length.Stable.Latest.t,
    previous_global_slot: u32, // Mina_numbers.Global_slot.Stable.Latest.t,
}

#[derive(Clone, Debug)]
pub struct ConstraintConstants {
    pub sub_windows_per_window: u64,
    pub ledger_depth: u64,
    pub work_delay: u64,
    pub block_window_duration_ms: u64,
    pub transaction_capacity_log_2: u64,
    pub pending_coinbase_depth: u64,
    pub coinbase_amount: Amount, // Currency.Amount.Stable.Latest.t,
    pub supercharged_coinbase_factor: u64,
    pub account_creation_fee: Fee,   // Currency.Fee.Stable.Latest.t,
    pub fork: Option<ForkConstants>, // Fork_constants.t option,
}

/// https://github.com/MinaProtocol/mina/blob/e5183ca1dde1c085b4c5d37d1d9987e24c294c32/src/lib/transaction_snark_scan_state/transaction_snark_scan_state.ml#L175
fn create_expected_statement<F>(
    constraint_constants: &ConstraintConstants,
    get_state: F,
    TransactionWithWitness {
        transaction_with_info,
        state_hash,
        statement,
        init_stack,
        ledger_witness,
    }: &TransactionWithWitness,
) -> Result<Statement, String>
where
    F: Fn(&Fp) -> &MinaStateProtocolStateValueStableV2,
{
    let mut ledger_witness = ledger_witness.clone();
    let source_merkle_root = ledger_witness.merkle_root();

    let WithStatus {
        data: transaction, ..
    } = transaction_with_info.transaction();

    let protocol_state = get_state(&state_hash.0);
    let state_view = protocol_state_view(protocol_state);

    let empty_local_state = LocalState::empty();

    let coinbase = match &transaction {
        Transaction::Coinbase(coinbase) => Some(coinbase.clone()),
        _ => None,
    };
    // Keep the error, in OCaml it is throwned later
    let fee_excess_with_err = transaction.fee_excess();

    let applied_transaction = apply_transaction(
        constraint_constants,
        &state_view,
        &mut ledger_witness,
        transaction,
    )?;

    let target_merkle_root = ledger_witness.merkle_root();

    let pending_coinbase_before = match init_stack {
        transaction_snark::InitStack::Base(source) => source,
        transaction_snark::InitStack::Merge => {
            return Err(
                "Invalid init stack in Pending coinbase stack state . Expected Base found Merge"
                    .to_string(),
            );
        }
    };

    let pending_coinbase_after = {
        let state_body_hash = state_hash.1;

        let pending_coinbase_with_state = pending_coinbase_before.push_state(state_body_hash);

        match coinbase {
            Some(cb) => pending_coinbase_with_state.push_coinbase(cb),
            None => pending_coinbase_with_state,
        }
    };

    let fee_excess = fee_excess_with_err?;
    let supply_increase = applied_transaction.supply_increase(constraint_constants)?;

    Ok(Statement {
        source: Registers {
            ledger: source_merkle_root,
            pending_coinbase_stack: statement.source.pending_coinbase_stack.clone(),
            local_state: empty_local_state.clone(),
        },
        target: Registers {
            ledger: target_merkle_root,
            pending_coinbase_stack: pending_coinbase_after,
            local_state: empty_local_state,
        },
        supply_increase,
        fee_excess,
        sok_digest: None,
    })
}

fn completed_work_to_scanable_work(
    job: AvailableJob,
    (fee, current_proof, prover): (Fee, LedgerProof, CompressedPubKey),
) -> Result<LedgerProofWithSokMessage, String> {
    use super::parallel_scan::AvailableJob::{Base, Merge};

    let sok_digest = &current_proof.0.statement.sok_digest;
    let proof = &current_proof.0.proof;

    match job {
        Base(TransactionWithWitness { statement, .. }) => {
            // todo!()

            assert!(sok_digest.is_some());

            let statement_with_sok = transaction_snark::Statement {
                source: statement.source,
                target: statement.target,
                supply_increase: statement.supply_increase,
                fee_excess: statement.fee_excess,
                sok_digest: sok_digest.clone(),
            };

            let ledger_proof = LedgerProof::create(statement_with_sok, proof.clone());
            let sok_message = SokMessage::create(fee, prover);

            Ok(LedgerProofWithSokMessage {
                proof: ledger_proof,
                sok_message,
            })
        }
        Merge {
            left: proof1,
            right: proof2,
        } => {
            let s1: &Statement = &proof1.proof.0.statement;
            let s2: &Statement = &proof2.proof.0.statement;

            let fee_excess = FeeExcess::combine(&s1.fee_excess, &s2.fee_excess)?;

            let supply_increase = s1
                .supply_increase
                .add(&s2.supply_increase)
                .expect("Error adding supply_increases");

            if s1.target.pending_coinbase_stack != s2.source.pending_coinbase_stack {
                panic!("Invalid pending coinbase stack state");
            }

            let statement = Statement {
                source: s1.source.clone(),
                target: s2.target.clone(),
                supply_increase,
                fee_excess,
                sok_digest: None,
            };

            let ledger_proof = LedgerProof::create(statement, proof.clone());
            let sok_message = SokMessage::create(fee, prover);

            Ok(LedgerProofWithSokMessage {
                proof: ledger_proof,
                sok_message,
            })
        }
    }
}

fn total_proofs(works: &[transaction_snark::work::Work]) -> usize {
    works.iter().map(|work| work.proofs.len()).sum()
}

pub enum StatementCheck {
    Partial,
    Full(Box<dyn Fn(&Fp) -> &MinaStateProtocolStateValueStableV2>), // TODO: The fn returns a protocol state
}

#[derive(Debug, Clone)]
pub struct Verifier;

impl Verifier {
    pub fn verify(&self, _proofs: &[LedgerProofWithSokMessage]) -> Result<bool, String> {
        // Implement verification later
        //
        // https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/pickles/pickles.ml#L1122
        // https://viable-systems.slack.com/archives/D01SVA87PQC/p1671715846448749
        Ok(true)
    }
}

impl ScanState {
    pub fn scan_statement(
        &self,
        constraint_constants: &ConstraintConstants,
        statement_check: StatementCheck,
        verifier: &Verifier,
    ) -> Result<Statement, String> {
        struct Acc(Option<(Statement, Vec<LedgerProofWithSokMessage>)>);

        let merge_acc = |mut proofs: Vec<LedgerProofWithSokMessage>,
                         acc: Acc,
                         s2: &Statement|
         -> Result<Acc, String> {
            assert!(s2.sok_digest.is_none());
            assert!(acc
                .0
                .as_ref()
                .map(|v| v.0.sok_digest.is_none())
                .unwrap_or(true));

            match acc.0 {
                None => Ok(Acc(Some((s2.clone(), proofs)))),
                Some((s1, mut ps)) => {
                    let merged_statement = s1.merge(s2)?;
                    proofs.append(&mut ps);
                    Ok(Acc(Some((merged_statement, proofs))))
                }
            }
        };

        let merge_pc =
            |acc: Option<Statement>, s2: &Statement| -> Result<Option<Statement>, String> {
                match acc {
                    None => Ok(Some(s2.clone())),
                    Some(s1) => {
                        if !pending_coinbase::Stack::connected(
                            &s1.target.pending_coinbase_stack,
                            &s2.source.pending_coinbase_stack,
                            Some(&s1.source.pending_coinbase_stack),
                        ) {
                            return Err(format!(
                                "Base merge proof: invalid pending coinbase \
                             transition s1: {:?} s2: {:?}",
                                s1, s2
                            ));
                        }
                        Ok(Some(s2.clone()))
                    }
                }
            };

        let fold_step_a = |(acc_statement, acc_pc): (Acc, Option<Statement>),
                           job: &merge::Job<LedgerProofWithSokMessage>|
         -> Result<(Acc, Option<Statement>), String> {
            use merge::{
                Job::{Empty, Full, Part},
                Record,
            };
            use JobStatus::Done;

            match job {
                Part(ref ledger @ LedgerProofWithSokMessage { ref proof, .. }) => {
                    let statement = proof.statement();
                    let acc_stmt = merge_acc(vec![ledger.clone()], acc_statement, statement)?;
                    Ok((acc_stmt, acc_pc))
                }
                Empty | Full(Record { state: Done, .. }) => Ok((acc_statement, acc_pc)),
                Full(Record { left, right, .. }) => {
                    let LedgerProofWithSokMessage { proof: proof1, .. } = &left;
                    let LedgerProofWithSokMessage { proof: proof2, .. } = &right;

                    let stmt1 = proof1.statement();
                    let stmt2 = proof2.statement();
                    let merged_statement = stmt1.merge(stmt2)?;

                    let acc_stmt = merge_acc(
                        vec![left.clone(), right.clone()],
                        acc_statement,
                        &merged_statement,
                    )?;

                    Ok((acc_stmt, acc_pc))
                }
            }
        };

        let fold_step_d = |(acc_statement, acc_pc): (Acc, Option<Statement>),
                           job: &base::Job<TransactionWithWitness>|
         -> Result<(Acc, Option<Statement>), String> {
            use base::{
                Job::{Empty, Full},
                Record,
            };
            use JobStatus::Done;

            match job {
                Empty => Ok((acc_statement, acc_pc)),
                Full(Record {
                    state: Done,
                    job: transaction,
                    ..
                }) => {
                    let acc_pc = merge_pc(acc_pc, &transaction.statement)?;
                    Ok((acc_statement, acc_pc))
                }
                Full(Record {
                    job: transaction, ..
                }) => {
                    use StatementCheck::{Full, Partial};

                    let expected_statement = match &statement_check {
                        Full(get_state) => create_expected_statement(
                            constraint_constants,
                            &**get_state,
                            transaction,
                        )?,
                        Partial => transaction.statement.clone(),
                    };

                    if transaction.statement == expected_statement {
                        let acc_stmt =
                            merge_acc(Vec::new(), acc_statement, &transaction.statement)?;
                        let acc_pc = merge_pc(acc_pc, &transaction.statement)?;

                        Ok((acc_stmt, acc_pc))
                    } else {
                        Err(format!(
                            "Bad base statement expected: {:#?} got: {:#?}",
                            transaction.statement, expected_statement
                        ))
                    }
                }
            }
        };

        let res = self.state.fold_chronological_until_err(
            (Acc(None), None),
            |acc, merge::Merge { weight: _, job }| fold_step_a(acc, job),
            |acc, base::Base { weight: _, job }| fold_step_d(acc, job),
            |v| v,
        )?;

        match res {
            (Acc(None), _) => Err("Empty".to_string()),
            (Acc(Some((res, proofs))), _) => match verifier.verify(proofs.as_slice()) {
                Ok(true) => Ok(res),
                Ok(false) => Err("Bad proofs".to_string()),
                Err(e) => Err(e),
            },
        }
    }

    pub fn check_invariants(
        &self,
        constraint_constants: &ConstraintConstants,
        statement_check: StatementCheck,
        verifier: &Verifier,
        _error_prefix: &'static str,
        _registers_begin: Option<&Registers>,
        _registers_end: Registers,
    ) -> Result<(), String> {
        // TODO: OCaml does much more than this (pretty printing error)
        match self.scan_statement(constraint_constants, statement_check, verifier) {
            Ok(_) => Ok(()),
            Err(s) => Err(s),
        }
    }

    fn statement_of_job(job: &AvailableJob) -> Option<Statement> {
        use super::parallel_scan::AvailableJob::{Base, Merge};

        match job {
            Base(TransactionWithWitness { statement, .. }) => Some(statement.clone()),
            Merge { left, right } => {
                let LedgerProofWithSokMessage { proof: p1, .. } = left;
                let LedgerProofWithSokMessage { proof: p2, .. } = right;

                p1.statement().merge(p2.statement()).ok()
            }
        }
    }

    fn create(work_delay: u64, transaction_capacity_log_2: u64) -> Self {
        let k = 2u64.pow(transaction_capacity_log_2 as u32);

        Self {
            state: ParallelScan::empty(k, work_delay),
        }
    }

    pub fn empty(constraint_constants: &ConstraintConstants) -> Self {
        let work_delay = constraint_constants.work_delay;
        let transaction_capacity_log_2 = constraint_constants.transaction_capacity_log_2;

        Self::create(work_delay, transaction_capacity_log_2)
    }

    fn extract_txns(
        txns_with_witnesses: &[TransactionWithWitness],
    ) -> Vec<(WithStatus<Transaction>, Fp)> {
        txns_with_witnesses
            .iter()
            .map(|txns_with_witnesses: &TransactionWithWitness| {
                let txn = txns_with_witnesses.transaction_with_info.transaction();
                let state_hash = txns_with_witnesses.state_hash.0;
                (txn, state_hash)
            })
            .collect()
    }

    pub fn latest_ledger_proof(
        &self,
    ) -> Option<(
        &LedgerProofWithSokMessage,
        Vec<(WithStatus<Transaction>, Fp)>,
    )> {
        let (proof, txns_with_witnesses) = self.state.last_emitted_value()?;
        Some((proof, Self::extract_txns(txns_with_witnesses.as_slice())))
    }

    fn free_space(&self) -> u64 {
        self.state.free_space()
    }

    fn all_jobs(&self) -> Vec<Vec<AvailableJob>> {
        self.state.all_jobs()
    }

    fn next_on_new_tree(&self) -> bool {
        self.state.next_on_new_tree()
    }

    fn base_jobs_on_latest_tree(&self) -> Vec<AvailableJob> {
        self.state.base_jobs_on_latest_tree()
    }

    fn base_jobs_on_earlier_tree(&self, index: usize) -> Vec<AvailableJob> {
        self.state.base_jobs_on_earlier_tree(index)
    }

    fn staged_transactions(&self) -> Vec<WithStatus<Transaction>> {
        self.state
            .pending_data()
            .into_iter()
            .map(|transaction_with_witness| {
                transaction_with_witness.transaction_with_info.transaction()
            })
            .collect()
    }

    pub fn staged_transactions_with_protocol_states<F>(
        &self,
        get_state: F,
    ) -> Vec<(WithStatus<Transaction>, MinaStateProtocolStateValueStableV2)>
    where
        F: Fn(&Fp) -> &MinaStateProtocolStateValueStableV2,
    {
        self.state
            .pending_data()
            .into_iter()
            .map(|transaction_with_witness| {
                let txn = transaction_with_witness.transaction_with_info.transaction();
                let protocol_state = get_state(&transaction_with_witness.state_hash.0);
                (txn, protocol_state.clone())
            })
            .collect()
    }

    fn partition_if_overflowing(&self) -> SpacePartition {
        let bundle_count = |work_count: u64| (work_count + 1) / 2;
        let SpacePartition {
            first: (slots, job_count),
            second,
        } = self.state.partition_if_overflowing();

        SpacePartition {
            first: (slots, bundle_count(job_count)),
            second: second.map(|(slots, job_count)| (slots, bundle_count(job_count))),
        }
    }

    fn extract_from_job(job: AvailableJob) -> Extracted {
        use super::parallel_scan::AvailableJob::{Base, Merge};

        match job {
            Base(d) => Extracted::First {
                transaction_with_info: d.transaction_with_info,
                statement: Box::new(d.statement),
                state_hash: d.state_hash,
                ledger_witness: d.ledger_witness,
                init_stack: d.init_stack,
            },
            Merge {
                left: LedgerProofWithSokMessage { proof: p1, .. },
                right: LedgerProofWithSokMessage { proof: p2, .. },
            } => Extracted::Second(Box::new((p1, p2))),
        }
    }

    pub fn all_work_statements_exn(&self) -> Vec<transaction_snark::work::Statement> {
        let work_seqs = self.all_jobs();

        let s = |job: &AvailableJob| Self::statement_of_job(job).unwrap();

        work_seqs
            .iter()
            .flat_map(|work_seq| group_list(work_seq, s))
            .collect()
    }

    fn required_work_pairs(&self, slots: u64) -> Vec<OneOrTwo<AvailableJob>> {
        let work_list = self.state.jobs_for_slots(slots);
        work_list
            .iter()
            .flat_map(|works| group_list(works, |job| job.clone()))
            .collect()
    }

    fn k_work_pairs_for_new_diff(&self, k: u64) -> Vec<OneOrTwo<AvailableJob>> {
        let work_list = self.state.jobs_for_next_update();
        work_list
            .iter()
            .flat_map(|works| group_list(works, |job| job.clone()))
            .take(k as usize)
            .collect()
    }

    // Always the same pairing of jobs
    fn work_statements_for_new_diff(&self) -> Vec<transaction_snark::work::Statement> {
        let work_list = self.state.jobs_for_next_update();

        let s = |job: &AvailableJob| Self::statement_of_job(job).unwrap();

        work_list
            .iter()
            .flat_map(|works| group_list(works, s))
            .collect()
    }

    pub fn all_work_pairs<F>(
        &self,
        get_state: F,
    ) -> Result<Vec<OneOrTwo<snark_work::spec::Work>>, String>
    where
        F: Fn(&Fp) -> &MinaStateProtocolStateValueStableV2,
    {
        let all_jobs = self.all_jobs();

        let single_spec = |job: AvailableJob| match Self::extract_from_job(job) {
            Extracted::First {
                transaction_with_info,
                statement,
                state_hash,
                ledger_witness,
                init_stack,
            } => {
                let witness = {
                    let WithStatus {
                        data: transaction,
                        status,
                    } = transaction_with_info.transaction();

                    let protocol_state_body = {
                        let state = get_state(&state_hash.0);
                        state.body.clone()
                    };

                    let init_stack = match init_stack {
                        InitStack::Base(x) => x,
                        InitStack::Merge => return Err("init_stack was Merge".to_string()),
                    };

                    TransactionWitness {
                        transaction,
                        ledger: ledger_witness,
                        protocol_state_body,
                        init_stack,
                        status,
                    }
                };

                Ok(snark_work::spec::Work::Transition((statement, witness)))
            }
            Extracted::Second(s) => {
                let merged = s.0.statement().merge(s.1.statement())?;
                Ok(snark_work::spec::Work::Merge((merged, s)))
            }
        };

        all_jobs
            .iter()
            .flat_map(|jobs| {
                group_list(jobs.as_slice(), |j| j.clone())
                    .map(|group| group.into_map_err(single_spec))
            })
            .collect()
    }

    fn fill_work_and_enqueue_transactions(
        &mut self,
        transactions: Vec<TransactionWithWitness>,
        work: Vec<transaction_snark::work::Unchecked>,
    ) -> Result<Option<(LedgerProof, Vec<(WithStatus<Transaction>, Fp)>)>, String> {
        let fill_in_transaction_snark_work =
            |works: Vec<transaction_snark::work::Work>| -> Result<Vec<LedgerProofWithSokMessage>, String>
        {
            let next_jobs = self
                .state
                .jobs_for_next_update()
                .into_iter()
                .flatten()
                .take(total_proofs(&works));

            let works = works.into_iter().flat_map(|transaction_snark::work::Work { fee, proofs, prover }| {
                proofs.into_map(|proof| (fee, proof, prover.clone())).into_iter()
            });

            next_jobs.zip(works).map(|(job, work)| completed_work_to_scanable_work(job, work)).collect()
        };

        let old_proof = self.state.last_emitted_value().cloned();
        let work_list = fill_in_transaction_snark_work(work)?;

        let proof_opt = self.state.update(transactions, work_list).unwrap();

        match proof_opt {
            None => Ok(None),
            Some((LedgerProofWithSokMessage { proof, .. }, txns_with_witnesses)) => {
                let curr_source = &proof.statement().source;

                // TODO(OCaml): get genesis ledger hash if the old_proof is none

                let prev_target = old_proof
                    .as_ref()
                    .map(|p| &p.0.proof.statement().target)
                    .unwrap_or(curr_source);

                // prev_target is connected to curr_source- Order of the arguments is
                // important here

                if Registers::connected(prev_target, curr_source) {
                    Ok(Some((proof, Self::extract_txns(&txns_with_witnesses))))
                } else {
                    Err("Unexpected ledger proof emitted".to_string())
                }
            }
        }
    }

    fn required_state_hashes(&self) -> HashSet<Fp> {
        self.state
            .pending_data()
            .iter()
            .map(|t| t.state_hash.0)
            .collect()
    }

    fn check_required_protocol_states(&self, _protocol_states: ()) {
        todo!() // Not sure what is the type of `protocol_states` here
    }
}

fn group_list<'a, F, T, R>(slice: &'a [T], fun: F) -> impl Iterator<Item = OneOrTwo<R>> + '_
where
    F: Fn(&'a T) -> R + 'a,
{
    slice.chunks(2).map(move |subslice| match subslice {
        [a, b] => OneOrTwo::Two((fun(a), fun(b))),
        [a] => OneOrTwo::One(fun(a)),
        _ => panic!(),
    })
}

pub enum Extracted {
    First {
        transaction_with_info: TransactionApplied,
        statement: Box<Statement>,
        state_hash: (Fp, Fp),
        ledger_witness: SparseLedger<AccountId, Account>,
        init_stack: InitStack,
    },
    Second(Box<(LedgerProof, LedgerProof)>),
}
