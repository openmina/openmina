use mina_hasher::Fp;
use mina_p2p_messages::v2::{MinaStateProtocolStateValueStableV2, StateHash};
use mina_signer::CompressedPubKey;

use crate::scan_state::{
    fee_excess::FeeExcess,
    parallel_scan::{base, merge, JobStatus},
    scan_state::transaction_snark::{
        LedgerProofWithSokMessage, SokMessage, Statement, TransactionWithWitness,
    },
};

use self::transaction_snark::LedgerProof;

use super::{currency::Fee, parallel_scan::ParallelScan};
// use super::parallel_scan::AvailableJob;

pub use super::parallel_scan::SpacePartition;

// type LedgerProof = LedgerProofProdStableV2;
// type LedgerProofWithSokMessage = TransactionSnarkScanStateLedgerProofWithSokMessageStableV2;
// type TransactionWithWitness = TransactionSnarkScanStateTransactionWithWitnessStableV2;

pub type AvailableJob = super::parallel_scan::AvailableJob<
    transaction_snark::TransactionWithWitness,
    transaction_snark::LedgerProofWithSokMessage,
>;

pub struct ScanState {
    state: ParallelScan<
        transaction_snark::TransactionWithWitness,
        transaction_snark::LedgerProofWithSokMessage,
    >,
}

pub mod transaction_snark {
    use mina_hasher::Fp;
    use mina_p2p_messages::v2::{
        MinaBasePendingCoinbaseStackVersionedStableV1, MinaBaseSparseLedgerBaseStableV2,
        MinaBaseStateBodyHashStableV1, MinaTransactionLogicTransactionAppliedStableV2,
        MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1, StateHash,
        TransactionSnarkPendingCoinbaseStackStateInitStackStableV1, TransactionSnarkProofStableV2,
    };
    use mina_signer::CompressedPubKey;

    use crate::{
        hash_noinputs,
        scan_state::{
            currency::{Amount, Signed},
            fee_excess::FeeExcess,
        },
    };

    use super::Fee;

    type LedgerHash = Fp;

    #[derive(Debug, Clone)]
    pub struct Registers {
        pub ledger: LedgerHash,
        pub pending_coinbase_stack: MinaBasePendingCoinbaseStackVersionedStableV1,
        pub local_state: MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/pending_coinbase.ml#L188
    fn empty_pending_coinbase() -> Fp {
        hash_noinputs("CoinbaseStack")
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/pending_coinbase.ml#L658
    pub fn connected(
        first: &MinaBasePendingCoinbaseStackVersionedStableV1,
        second: &MinaBasePendingCoinbaseStackVersionedStableV1,
        prev: Option<&MinaBasePendingCoinbaseStackVersionedStableV1>,
    ) -> bool {
        // same as old stack or second could be a new stack with empty data
        let coinbase_stack_connected = (first.data == second.data) || {
            let second: Fp = second.data.to_field();
            second == empty_pending_coinbase()
        };

        // 1. same as old stack or
        // 2. new stack initialized with the stack state of last block. Not possible to know this unless we track
        //    all the stack states because they are updated once per block (init=curr)
        // 3. [second] could be a new stack initialized with the latest state of [first] or
        // 4. [second] starts from the previous state of [first]. This is not available in either [first] or [second] *)
        let state_stack_connected = first.state == second.state
            || second.state.init == second.state.curr
            || first.state.curr == second.state.curr
            || prev
                .map(|prev| prev.state.curr == second.state.curr)
                .unwrap_or(true);

        coinbase_stack_connected && state_stack_connected
    }

    impl Registers {
        /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_snark/transaction_snark.ml#L350
        pub fn check_equal(&self, other: &Self) -> bool {
            self.ledger == other.ledger
                && self.local_state == other.local_state
                && connected(
                    &self.pending_coinbase_stack,
                    &other.pending_coinbase_stack,
                    None,
                )
        }
    }

    #[derive(Debug, Clone)]
    pub struct Statement {
        pub source: Registers,
        pub target: Registers,
        pub supply_increase: Signed<Amount>,
        pub fee_excess: FeeExcess,
        pub sok_digest: Option<Vec<u8>>,
    }

    impl Statement {
        /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_snark/transaction_snark.ml#L348
        pub fn merge(&self, s2: &Statement) -> Self {
            let fee_excess = FeeExcess::combine(&self.fee_excess, &s2.fee_excess);
            let supply_increase = self
                .supply_increase
                .add(&s2.supply_increase)
                .expect("Error adding supply_increase");

            assert!(self.target.check_equal(&s2.source));

            Self {
                source: self.source.clone(),
                target: s2.target.clone(),
                supply_increase,
                fee_excess,
                sok_digest: None,
            }
        }
    }

    /// TODO: Use a `Mask` here
    #[derive(Debug, Clone)]
    pub struct LedgerWitness;

    impl LedgerWitness {
        pub fn merkle_root(&self) -> Fp {
            todo!()
        }
    }

    #[derive(Debug, Clone)]
    pub struct TransactionWithWitness {
        pub transaction_with_info: MinaTransactionLogicTransactionAppliedStableV2,
        pub state_hash: (StateHash, MinaBaseStateBodyHashStableV1),
        pub statement: Statement,
        pub init_stack: TransactionSnarkPendingCoinbaseStackStateInitStackStableV1,
        pub ledger_witness: LedgerWitness,
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
    }

    #[derive(Debug, Clone)]
    pub struct Work {
        pub fee: Fee,
        pub proofs: OneOrTwo<LedgerProof>,
        pub prover: CompressedPubKey,
    }
}

impl ScanState {
    pub fn hash(&self) {
        use binprot::BinProtWrite;

        self.state.hash(
            |buffer, proof| {
                proof.binprot_write(buffer).unwrap();
            },
            |buffer, transaction| {
                transaction.binprot_write(buffer).unwrap();
            },
        );
    }
}

pub struct ForkConstants {
    previous_state_hash: Fp,   // Pickles.Backend.Tick.Field.Stable.Latest.t,
    previous_length: u32,      // Mina_numbers.Length.Stable.Latest.t,
    previous_global_slot: u32, // Mina_numbers.Global_slot.Stable.Latest.t,
}

pub struct GenesisConstants {
    sub_windows_per_window: u64,
    ledger_depth: u64,
    work_delay: u64,
    block_window_duration_ms: u64,
    transaction_capacity_log_2: u64,
    pending_coinbase_depth: u64,
    coinbase_amount: u64, // Currency.Amount.Stable.Latest.t,
    supercharged_coinbase_factor: u64,
    account_creation_fee: u64,   // Currency.Fee.Stable.Latest.t,
    fork: Option<ForkConstants>, // Fork_constants.t option,
}

// type GetState = impl Fn(&StateHash) -> MinaStateProtocolStateValueStableV2;

fn create_expected_statement<F>(
    constraint_constants: &GenesisConstants,
    get_state: F,
    TransactionWithWitness {
        transaction_with_info,
        state_hash,
        statement,
        init_stack,
        ledger_witness,
    }: &TransactionWithWitness,
) where
    F: Fn(&StateHash) -> &MinaStateProtocolStateValueStableV2,
{
    let source_merkle_root = ledger_witness.merkle_root();
}

// let create_expected_statement ~constraint_constants
//     ~(get_state : State_hash.t -> Mina_state.Protocol_state.value Or_error.t)
//     { Transaction_with_witness.transaction_with_info
//     ; state_hash
//     ; ledger_witness
//     ; init_stack
//     ; statement
//     } =
//   let open Or_error.Let_syntax in
//   let source_merkle_root =
//     Frozen_ledger_hash.of_ledger_hash
//     @@ Sparse_ledger.merkle_root ledger_witness
//   in
//   let { With_status.data = transaction; status = _ } =
//     Ledger.Transaction_applied.transaction transaction_with_info
//   in
//   let%bind protocol_state = get_state (fst state_hash) in
//   let state_view = Mina_state.Protocol_state.Body.view protocol_state.body in
//   let empty_local_state = Mina_state.Local_state.empty () in
//   let%bind after, applied_transaction =
//     Or_error.try_with (fun () ->
//         Sparse_ledger.apply_transaction ~constraint_constants
//           ~txn_state_view:state_view ledger_witness transaction )
//     |> Or_error.join
//   in
//   let target_merkle_root =
//     Sparse_ledger.merkle_root after |> Frozen_ledger_hash.of_ledger_hash
//   in
//   let%bind pending_coinbase_before =
//     match init_stack with
//     | Base source ->
//         Ok source
//     | Merge ->
//         Or_error.errorf
//           !"Invalid init stack in Pending coinbase stack state . Expected Base \
//             found Merge"
//   in
//   let pending_coinbase_after =
//     let state_body_hash = snd state_hash in
//     let pending_coinbase_with_state =
//       Pending_coinbase.Stack.push_state state_body_hash pending_coinbase_before
//     in
//     match transaction with
//     | Coinbase c ->
//         Pending_coinbase.Stack.push_coinbase c pending_coinbase_with_state
//     | _ ->
//         pending_coinbase_with_state
//   in
//   let%bind fee_excess = Transaction.fee_excess transaction in
//   let%map supply_increase =
//     Ledger.Transaction_applied.supply_increase applied_transaction
//   in
//   { Transaction_snark.Statement.source =
//       { ledger = source_merkle_root
//       ; pending_coinbase_stack = statement.source.pending_coinbase_stack
//       ; local_state = empty_local_state
//       }
//   ; target =
//       { ledger = target_merkle_root
//       ; pending_coinbase_stack = pending_coinbase_after
//       ; local_state = empty_local_state
//       }
//   ; fee_excess
//   ; supply_increase
//   ; sok_digest = ()
//   }

fn completed_work_to_scanable_work(
    job: AvailableJob,
    (fee, current_proof, prover): (Fee, LedgerProof, CompressedPubKey),
) -> LedgerProofWithSokMessage {
    use super::parallel_scan::AvailableJob::{Base, Merge};
    use transaction_snark::TransactionWithWitness;

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

            LedgerProofWithSokMessage {
                proof: ledger_proof,
                sok_message,
            }
        }
        Merge {
            left: proof1,
            right: proof2,
        } => {
            let s1: &Statement = &proof1.proof.0.statement;
            let s2: &Statement = &proof2.proof.0.statement;

            let fee_excess = FeeExcess::combine(&s1.fee_excess, &s2.fee_excess);

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

            LedgerProofWithSokMessage {
                proof: ledger_proof,
                sok_message,
            }
        }
    }
}

fn total_proofs(works: &[transaction_snark::Work]) -> usize {
    works.iter().map(|work| work.proofs.len()).sum()
}

#[derive(Debug, Clone)]
pub enum StatementCheck {
    Partial,
    Full, // TODO: The fn returns a protocol state
}

#[derive(Debug, Clone)]
pub struct Verifier;

impl Verifier {
    pub fn verify() {
        todo!()
    }
}

impl ScanState {
    pub fn scan_statement(
        &self,
        constraint_constants: &GenesisConstants,
        statement_check: StatementCheck,
        verifier: &Verifier,
    ) -> Result<Statement, ()> {
        struct Acc(Option<(Statement, Vec<LedgerProofWithSokMessage>)>);

        let merge_acc = |mut proofs: Vec<LedgerProofWithSokMessage>, acc: Acc, s2: &Statement| {
            assert!(s2.sok_digest.is_none());
            assert!(acc
                .0
                .as_ref()
                .map(|v| v.0.sok_digest.is_none())
                .unwrap_or(true));

            match acc.0 {
                None => Some((s2.clone(), proofs)),
                Some((s1, mut ps)) => {
                    let merged_statement = s1.merge(&s2);
                    proofs.append(&mut ps);
                    Some((merged_statement, proofs))
                }
            }
        };

        let merge_pc = |acc: Option<Statement>, s2: Statement| match acc {
            None => Some(s2),
            Some(s1) => {
                if !transaction_snark::connected(
                    &s1.target.pending_coinbase_stack,
                    &s2.source.pending_coinbase_stack,
                    Some(&s1.source.pending_coinbase_stack),
                ) {
                    panic!(
                        "Base merge proof: invalid pending coinbase \
                         transition s1: {:?} s2: {:?}",
                        s1, s2
                    )
                }
                Some(s2)
            }
        };

        let fold_step_a = |acc_statement: Acc,
                           acc_pc: (),
                           job: merge::Job<LedgerProofWithSokMessage>| {
            use merge::{
                Job::{Empty, Full, Part},
                Record,
            };
            use JobStatus::Done;

            match job {
                Part(ref ledger @ LedgerProofWithSokMessage { ref proof, .. }) => {
                    let statement = proof.statement();
                    let acc_stmt = merge_acc(vec![ledger.clone()], acc_statement, statement);
                    (Acc(acc_stmt), acc_pc)
                }
                Empty | Full(Record { state: Done, .. }) => (acc_statement, acc_pc),
                Full(Record { left, right, .. }) => {
                    let LedgerProofWithSokMessage { proof: proof1, .. } = &left;
                    let LedgerProofWithSokMessage { proof: proof2, .. } = &right;

                    let stmt1 = proof1.statement();
                    let stmt2 = proof2.statement();
                    let merged_statement = stmt1.merge(stmt2);

                    let acc_stmt = merge_acc(vec![left, right], acc_statement, &merged_statement);

                    (Acc(acc_stmt), acc_pc)
                }
            }
        };

        let fold_step_d = |acc_statement: Acc,
                           acc_pc: Option<Statement>,
                           job: base::Job<TransactionWithWitness>| {
            use base::{
                Job::{Empty, Full},
                Record,
            };
            use JobStatus::Done;

            match job {
                Empty => (acc_statement, acc_pc),
                Full(Record {
                    state: Done,
                    job: transaction,
                    ..
                }) => {
                    let acc_pc = merge_pc(acc_pc, transaction.statement);
                    (acc_statement, acc_pc)
                }
                Full(Record {
                    job: transaction, ..
                }) => {
                    use StatementCheck::{Full, Partial};

                    match statement_check {
                        Full => {
                            todo!()
                        }
                        Partial => todo!(),
                    }

                    todo!()
                }
            }
        };

        todo!()
    }
}
// let fold_step_d (acc_statement, acc_pc) job =
//   match job with
//   | Parallel_scan.Base.Job.Empty ->
//       return (acc_statement, acc_pc)
//   | Full
//       { status = Parallel_scan.Job_status.Done
//       ; job = (transaction : Transaction_with_witness.t)
//       ; _
//       } ->
//       let%map acc_pc =
//         Deferred.return (merge_pc acc_pc transaction.statement)
//       in
//       (acc_statement, acc_pc)
//   | Full { job = transaction; _ } ->
//       with_error "Bad base statement" ~f:(fun () ->
//           let%bind expected_statement =
//             match statement_check with
//             | `Full get_state ->
//                 let%bind result =
//                   Timer.time timer
//                     (sprintf "create_expected_statement:%s" __LOC__)
//                     (fun () ->
//                       Deferred.return
//                         (create_expected_statement ~constraint_constants
//                            ~get_state transaction ) )
//                 in
//                 let%map () = yield_always () in
//                 result
//             | `Partial ->
//                 return transaction.statement
//           in
//           let%bind () = yield_always () in
//           if
//             Transaction_snark.Statement.equal transaction.statement
//               expected_statement
//           then
//             let%bind acc_stmt =
//               merge_acc ~proofs:[] acc_statement transaction.statement
//             in
//             let%map acc_pc =
//               merge_pc acc_pc transaction.statement |> Deferred.return
//             in
//             (acc_stmt, acc_pc)
//           else
//             Deferred.Or_error.error_string
//               (sprintf
//                  !"Bad base statement expected: \
//                    %{sexp:Transaction_snark.Statement.t} got: \
//                    %{sexp:Transaction_snark.Statement.t}"
//                  transaction.statement expected_statement ) )
// in

// let scan_statement ~constraint_constants tree ~statement_check ~verifier :
//     ( Transaction_snark.Statement.t
//     , [ `Error of Error.t | `Empty ] )
//     Deferred.Result.t =
