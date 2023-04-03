use std::collections::HashSet;

use blake2::{
    digest::{generic_array::GenericArray, typenum::U32},
    Digest,
};
use mina_hasher::Fp;
use mina_p2p_messages::v2::MinaStateProtocolStateValueStableV2;
use mina_signer::CompressedPubKey;
use sha2::Sha256;

use crate::{
    scan_state::{
        parallel_scan::{base, merge, JobStatus},
        pending_coinbase,
        scan_state::transaction_snark::{
            LedgerProofWithSokMessage, SokMessage, Statement, TransactionWithWitness,
        },
        transaction_logic::{
            apply_transaction_first_pass, apply_transaction_second_pass,
            local_state::LocalStateEnv,
            protocol_state::GlobalState,
            transaction_partially_applied::{
                TransactionPartiallyApplied, ZkappCommandPartiallyApplied,
            },
            TransactionStatus,
        },
    },
    staged_ledger::{
        hash::AuxHash,
        sparse_ledger::{LedgerIntf, SparseLedger},
    },
    verifier::Verifier,
    Account, AccountId,
};

use self::transaction_snark::{InitStack, LedgerProof, OneOrTwo, Registers};

use super::{
    currency::{Amount, Fee, Length, Slot},
    parallel_scan::ParallelScan,
    snark_work,
    transaction_logic::{
        local_state::LocalState,
        protocol_state::{protocol_state_view, ProtocolStateView},
        transaction_applied::TransactionApplied,
        transaction_witness::TransactionWitness,
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

#[derive(Clone, Debug)]
struct BorderBlockContinuedInTheNextTree(bool);

/// Scan state and any zkapp updates that were applied to the to the most recent
/// snarked ledger but are from the tree just before the tree corresponding to
/// the snarked ledger*)
#[derive(Clone)]
pub struct ScanState {
    pub scan_state: ParallelScan<
        transaction_snark::TransactionWithWitness,
        transaction_snark::LedgerProofWithSokMessage,
    >,
    pub previous_incomplete_zkapp_updates: (
        Vec<transaction_snark::TransactionWithWitness>,
        BorderBlockContinuedInTheNextTree,
    ),
}

pub mod transaction_snark {
    use std::rc::Rc;

    use itertools::Itertools;
    use mina_hasher::Fp;
    use mina_p2p_messages::v2::TransactionSnarkProofStableV2;
    use mina_signer::CompressedPubKey;

    use crate::{
        scan_state::{
            currency::{Amount, Signed, Slot},
            fee_excess::FeeExcess,
            pending_coinbase,
            transaction_logic::{local_state::LocalState, transaction_applied::TransactionApplied},
        },
        staged_ledger::{hash::OCamlString, sparse_ledger::SparseLedger},
        Account, AccountId,
    };

    use super::Fee;

    pub type LedgerHash = Fp;

    /// https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/mina_state/registers.ml
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Registers {
        pub first_pass_ledger: LedgerHash,
        pub second_pass_ledger: LedgerHash,
        pub pending_coinbase_stack: pending_coinbase::Stack,
        pub local_state: LocalState,
    }

    impl Registers {
        /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_snark/transaction_snark.ml#L350
        pub fn check_equal(&self, other: &Self) -> bool {
            let Self {
                first_pass_ledger,
                second_pass_ledger,
                pending_coinbase_stack,
                local_state,
            } = self;

            first_pass_ledger == &other.first_pass_ledger
                && second_pass_ledger == &other.second_pass_ledger
                && local_state == &other.local_state
                && pending_coinbase::Stack::connected(
                    &pending_coinbase_stack,
                    &other.pending_coinbase_stack,
                    None,
                )
        }

        /// https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/mina_state/registers.ml#L55
        pub fn connected(r1: &Self, r2: &Self) -> bool {
            let Self {
                first_pass_ledger,
                second_pass_ledger,
                pending_coinbase_stack,
                local_state,
            } = r1;

            first_pass_ledger == &r2.first_pass_ledger
                && second_pass_ledger == &r2.second_pass_ledger
                && local_state == &r2.local_state
                && pending_coinbase::Stack::connected(
                    &pending_coinbase_stack,
                    &r2.pending_coinbase_stack,
                    None,
                )
        }
    }

    #[derive(Clone, PartialEq, Eq, derive_more::Deref)]
    pub struct SokDigest(pub Vec<u8>);

    impl OCamlString for SokDigest {
        fn to_ocaml_str(&self) -> String {
            crate::staged_ledger::hash::to_ocaml_str(&self.0)
        }

        fn from_ocaml_str(s: &str) -> Self {
            let bytes: [u8; 32] = crate::staged_ledger::hash::from_ocaml_str(s);
            Self(bytes.to_vec())
        }
    }

    impl std::fmt::Debug for SokDigest {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_fmt(format_args!("SokDigest({})", self.to_ocaml_str()))
        }
    }

    impl Default for SokDigest {
        /// https://github.com/MinaProtocol/mina/blob/3a78f0e0c1343d14e2729c8b00205baa2ec70c93/src/lib/mina_base/sok_message.ml#L76
        fn default() -> Self {
            Self(vec![0; 32])
        }
    }

    struct StatementLedgers {
        first_pass_ledger_source: LedgerHash,
        first_pass_ledger_target: LedgerHash,
        second_pass_ledger_source: LedgerHash,
        second_pass_ledger_target: LedgerHash,
        connecting_ledger_left: LedgerHash,
        connecting_ledger_right: LedgerHash,
        local_state_ledger_source: Fp,
        local_state_ledger_target: Fp,
    }

    impl StatementLedgers {
        /// https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/mina_state/snarked_ledger_state.ml#L530
        fn of_statement<T>(s: &Statement<T>) -> Self {
            Self {
                first_pass_ledger_source: s.source.first_pass_ledger,
                first_pass_ledger_target: s.target.first_pass_ledger,
                second_pass_ledger_source: s.source.second_pass_ledger,
                second_pass_ledger_target: s.target.second_pass_ledger,
                connecting_ledger_left: s.connecting_ledger_left,
                connecting_ledger_right: s.connecting_ledger_right,
                local_state_ledger_source: s.source.local_state.ledger,
                local_state_ledger_target: s.target.local_state.ledger,
            }
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/mina_state/snarked_ledger_state.ml#L546
    fn validate_ledgers_at_merge(
        s1: &StatementLedgers,
        s2: &StatementLedgers,
    ) -> Result<bool, String> {
        // Check ledgers are valid based on the rules described in
        // https://github.com/MinaProtocol/mina/discussions/12000
        let is_same_block_at_shared_boundary = {
            // First statement ends and the second statement starts in the
            // same block. It could be within a single scan state tree
            // or across two scan state trees
            &s1.connecting_ledger_right == &s2.connecting_ledger_left
        };

        // Rule 1
        let l1 = if is_same_block_at_shared_boundary {
            // first pass ledger continues
            &s2.first_pass_ledger_source
        } else {
            // s1's first pass ledger stops at the end of a block's transactions,
            // check that it is equal to the start of the block's second pass ledger
            &s1.connecting_ledger_right
        };
        let rule1 = "First pass ledger continues or first pass ledger connects to the \
             same block's start of the second pass ledger";
        let res1 = &s1.first_pass_ledger_target == l1;

        // Rule 2
        // s1's second pass ledger ends at say, block B1. s2 is in the next block, say B2
        let l2 = if is_same_block_at_shared_boundary {
            // second pass ledger continues
            &s1.second_pass_ledger_target
        } else {
            // s2's second pass ledger starts where B2's first pass ledger ends
            &s2.connecting_ledger_left
        };
        let rule2 = "Second pass ledger continues or second pass ledger of the statement on \
             the right connects to the same block's end of first pass ledger";
        let res2 = &s2.second_pass_ledger_source == l2;

        // Rule 3
        let l3 = if is_same_block_at_shared_boundary {
            // no-op
            &s1.second_pass_ledger_target
        } else {
            // s2's first pass ledger starts where B1's second pass ledger ends
            &s2.first_pass_ledger_source
        };
        let rule3 = "First pass ledger of the statement on the right connects to the second \
             pass ledger of the statement on the left";
        let res3 = &s1.second_pass_ledger_target == l3;

        let rule4 = "local state ledgers are equal or transition correctly from first pass \
             to second pass";
        let res4 = {
            let local_state_ledger_equal =
                &s2.local_state_ledger_source == &s1.local_state_ledger_target;

            let local_state_ledger_transitions = s2.local_state_ledger_source
                == s2.second_pass_ledger_source
                && s1.local_state_ledger_target == s1.first_pass_ledger_target;

            local_state_ledger_equal || local_state_ledger_transitions
        };

        let faileds = [(res1, rule1), (res2, rule2), (res3, rule3), (res4, rule4)]
            .iter()
            .filter_map(|(v, s)| if *v { None } else { Some(*s) })
            .collect::<Vec<_>>();

        if !faileds.is_empty() {
            return Err(format!("Constraints failed: {}", faileds.iter().join(",")));
        }

        Ok(res1 && res2 && res3 && res4)
    }

    fn valid_ledgers_at_merge_unchecked(
        s1: &StatementLedgers,
        s2: &StatementLedgers,
    ) -> Result<bool, String> {
        validate_ledgers_at_merge(s1, s2)
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Statement<D> {
        pub source: Registers,
        pub target: Registers,
        pub connecting_ledger_left: LedgerHash,
        pub connecting_ledger_right: LedgerHash,
        pub supply_increase: Signed<Amount>,
        pub fee_excess: FeeExcess,
        pub sok_digest: D,
    }

    impl Statement<()> {
        /// https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/mina_state/snarked_ledger_state.ml#L631
        pub fn merge(&self, s2: &Statement<()>) -> Result<Self, String> {
            let or_error_of_bool = |b: bool, error: &str| {
                if b {
                    Ok(())
                } else {
                    Err(format!(
                        "Error merging statements left: {:#?} right {:#?}: {}",
                        self, s2, error
                    ))
                }
            };

            // check ledgers are connected
            let s1_ledger = StatementLedgers::of_statement(self);
            let s2_ledger = StatementLedgers::of_statement(s2);

            valid_ledgers_at_merge_unchecked(&s1_ledger, &s2_ledger)?;

            // Check pending coinbase stack is connected
            or_error_of_bool(
                pending_coinbase::Stack::connected(
                    &self.target.pending_coinbase_stack,
                    &s2.source.pending_coinbase_stack,
                    None,
                ),
                "Pending coinbase stacks are not connected",
            )?;

            self.source.local_state;

            // Check local states sans ledger are equal. Local state ledgers are checked
            // in [valid_ledgers_at_merge_uncheckeds]
            or_error_of_bool(
                self.target
                    .local_state
                    .equal_without_ledger(&s2.source.local_state),
                "Local states are not connected",
            )?;

            let connecting_ledger_left = self.connecting_ledger_left;
            let connecting_ledger_right = s2.connecting_ledger_right;

            let fee_excess = FeeExcess::combine(&self.fee_excess, &s2.fee_excess)?;
            let supply_increase = self
                .supply_increase
                .add(&s2.supply_increase)
                .ok_or_else(|| "Error adding supply_increase".to_string())?;

            // assert!(self.target.check_equal(&s2.source));

            Ok(Self {
                source: self.source.clone(),
                target: s2.target.clone(),
                supply_increase,
                fee_excess,
                sok_digest: (),
                connecting_ledger_left,
                connecting_ledger_right,
            })
        }
    }

    pub mod work {
        use super::*;

        pub type Statement = OneOrTwo<super::Statement<()>>;

        #[derive(Debug, Clone)]
        pub struct Work {
            pub fee: Fee,
            pub proofs: OneOrTwo<LedgerProof>,
            pub prover: CompressedPubKey,
        }

        pub type Unchecked = Work;

        pub type Checked = Work;

        impl Work {
            pub fn statement(&self) -> Statement {
                self.proofs.map(|p| {
                    let statement = p.statement();
                    super::Statement::<()> {
                        source: statement.source,
                        target: statement.target,
                        supply_increase: statement.supply_increase,
                        fee_excess: statement.fee_excess,
                        sok_digest: (),
                        connecting_ledger_left: statement.connecting_ledger_left,
                        connecting_ledger_right: statement.connecting_ledger_right,
                    }
                })
            }
        }

        impl Checked {
            /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/transaction_snark_work/transaction_snark_work.ml#L121
            pub fn forget(self) -> Unchecked {
                self
            }
        }
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
        pub statement: Statement<()>,
        pub init_stack: InitStack,
        pub first_pass_ledger_witness: SparseLedger<AccountId, Account>,
        pub second_pass_ledger_witness: SparseLedger<AccountId, Account>,
        pub block_global_slot: Slot,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct TransactionSnark<D> {
        pub statement: Statement<D>,
        pub proof: Rc<TransactionSnarkProofStableV2>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct LedgerProof(pub TransactionSnark<SokDigest>);

    impl LedgerProof {
        pub fn create(
            statement: Statement<()>,
            sok_digest: SokDigest,
            proof: Rc<TransactionSnarkProofStableV2>,
        ) -> Self {
            let statement = Statement::<SokDigest> {
                source: statement.source,
                target: statement.target,
                supply_increase: statement.supply_increase,
                fee_excess: statement.fee_excess,
                sok_digest,
                connecting_ledger_left: statement.connecting_ledger_left,
                connecting_ledger_right: statement.connecting_ledger_right,
            };

            Self(TransactionSnark { statement, proof })
        }

        pub fn statement(&self) -> Statement<()> {
            let Statement {
                source,
                target,
                connecting_ledger_left,
                connecting_ledger_right,
                supply_increase,
                fee_excess,
                sok_digest: _,
            } = &self.0.statement;

            Statement::<()> {
                source: source.clone(),
                target: target.clone(),
                supply_increase: *supply_increase,
                fee_excess: fee_excess.clone(),
                sok_digest: (),
                connecting_ledger_left: *connecting_ledger_left,
                connecting_ledger_right: *connecting_ledger_right,
            }
        }

        pub fn statement_ref(&self) -> &Statement<SokDigest> {
            &self.0.statement
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct SokMessage {
        pub fee: Fee,
        pub prover: CompressedPubKey,
    }

    impl SokMessage {
        pub fn create(fee: Fee, prover: CompressedPubKey) -> Self {
            Self { fee, prover }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct LedgerProofWithSokMessage {
        pub proof: LedgerProof,
        pub sok_message: SokMessage,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
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

        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/one_or_two/one_or_two.ml#L54
        pub fn zip<B>(a: OneOrTwo<T>, b: OneOrTwo<B>) -> OneOrTwo<(T, B)> {
            use OneOrTwo::*;

            match (a, b) {
                (One(a), One(b)) => One((a, b)),
                (Two((a1, a2)), Two((b1, b2))) => Two(((a1, b1), (a2, b2))),
                (One(_), Two(_)) => panic!("One_or_two.zip mismatched"),
                (Two(_), One(_)) => panic!("One_or_two.zip mismatched"),
            }
        }

        pub fn fold<A, F>(&self, init: A, fun: F) -> A
        where
            F: Fn(A, &T) -> A,
        {
            match self {
                OneOrTwo::One(a) => fun(init, a),
                OneOrTwo::Two((a, b)) => fun(fun(init, a), b),
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

fn sha256_digest(bytes: &[u8]) -> GenericArray<u8, U32> {
    let mut sha: Sha256 = Sha256::new();
    sha.update(bytes);
    sha.finalize()
}

impl ScanState {
    pub fn hash(&self) -> AuxHash {
        use binprot::BinProtWrite;

        let Self {
            scan_state,
            previous_incomplete_zkapp_updates,
        } = self;

        let state_hash = scan_state.hash(
            |buffer, proof| {
                #[cfg(test)]
                {
                    let a: mina_p2p_messages::v2::TransactionSnarkScanStateLedgerProofWithSokMessageStableV2 = proof.into();
                    let b: LedgerProofWithSokMessage = (&a).into();
                    assert_eq!(&b, proof);
                }

                proof.binprot_write(buffer).unwrap();
            },
            |buffer, transaction| {
                transaction.binprot_write(buffer).unwrap();
            },
        );

        let (
            previous_incomplete_zkapp_updates,
            BorderBlockContinuedInTheNextTree(continue_in_next_tree),
        ) = previous_incomplete_zkapp_updates;

        let incomplete_updates = previous_incomplete_zkapp_updates.iter().fold(
            Vec::with_capacity(1024 * 32),
            |accum, tx| {
                tx.binprot_write(&mut accum);
                accum
            },
        );
        let incomplete_updates = sha256_digest(&incomplete_updates);

        let continue_in_next_tree = match continue_in_next_tree {
            true => "true",
            false => "false",
        };
        let continue_in_next_tree = sha256_digest(continue_in_next_tree.as_bytes());

        let mut bytes = Vec::with_capacity(2048);
        bytes.extend_from_slice(&state_hash);
        bytes.extend_from_slice(&incomplete_updates);
        bytes.extend_from_slice(&continue_in_next_tree);
        let digest = sha256_digest(&bytes);

        AuxHash(digest.into())
    }
}

#[derive(Clone, Debug)]
pub struct ForkConstants {
    previous_state_hash: Fp,    // Pickles.Backend.Tick.Field.Stable.Latest.t,
    previous_length: Length,    // Mina_numbers.Length.Stable.Latest.t,
    previous_global_slot: Slot, // Mina_numbers.Global_slot.Stable.Latest.t,
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
    connecting_merkle_root: Fp,
    TransactionWithWitness {
        transaction_with_info,
        state_hash,
        statement,
        init_stack,
        first_pass_ledger_witness,
        second_pass_ledger_witness,
        block_global_slot,
    }: &TransactionWithWitness,
) -> Result<Statement<()>, String>
where
    F: Fn(&Fp) -> &MinaStateProtocolStateValueStableV2,
{
    let source_first_pass_merkle_root = first_pass_ledger_witness.merkle_root();
    let source_second_pass_merkle_root = second_pass_ledger_witness.merkle_root();

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

    let (target_first_pass_merkle_root, target_second_pass_merkle_root, supply_increase) = {
        let mut first_pass_ledger_witness = first_pass_ledger_witness.clone();
        let partially_applied_transaction = apply_transaction_first_pass(
            constraint_constants,
            *block_global_slot,
            &state_view,
            &mut first_pass_ledger_witness,
            &transaction,
        )?;

        let mut second_pass_ledger_witness = second_pass_ledger_witness.clone();
        let applied_transaction = apply_transaction_second_pass(
            &mut second_pass_ledger_witness,
            partially_applied_transaction,
        )?;

        let target_first_pass_merkle_root = first_pass_ledger_witness.merkle_root();
        let target_second_pass_merkle_root = second_pass_ledger_witness.merkle_root();

        // TODO: `supply_increase` has no parameter
        let supply_increase = applied_transaction.supply_increase(constraint_constants)?;

        (
            target_first_pass_merkle_root,
            target_second_pass_merkle_root,
            supply_increase,
        )
    };

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

        let pending_coinbase_with_state =
            pending_coinbase_before.push_state(state_body_hash, *block_global_slot);

        match coinbase {
            Some(cb) => pending_coinbase_with_state.push_coinbase(cb),
            None => pending_coinbase_with_state,
        }
    };

    let fee_excess = fee_excess_with_err?;

    Ok(Statement {
        source: Registers {
            first_pass_ledger: source_first_pass_merkle_root,
            second_pass_ledger: source_second_pass_merkle_root,
            pending_coinbase_stack: statement.source.pending_coinbase_stack.clone(),
            local_state: empty_local_state.clone(),
        },
        target: Registers {
            first_pass_ledger: target_first_pass_merkle_root,
            second_pass_ledger: target_second_pass_merkle_root,
            pending_coinbase_stack: pending_coinbase_after,
            local_state: empty_local_state,
        },
        connecting_ledger_left: connecting_merkle_root,
        connecting_ledger_right: connecting_merkle_root,
        supply_increase,
        fee_excess,
        sok_digest: (),
    })
}

fn completed_work_to_scanable_work(
    job: AvailableJob,
    (fee, current_proof, prover): (Fee, LedgerProof, CompressedPubKey),
) -> Result<LedgerProofWithSokMessage, String> {
    use super::parallel_scan::AvailableJob::{Base, Merge};

    let sok_digest = current_proof.0.statement.sok_digest;

    let proof = &current_proof.0.proof;

    match job {
        Base(TransactionWithWitness { statement, .. }) => {
            let ledger_proof = LedgerProof::create(statement, sok_digest, proof.clone());
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
            let s1 = proof1.proof.statement();
            let s2 = proof2.proof.statement();

            let statement = s1.merge(&s2)?;

            let ledger_proof = LedgerProof::create(statement, sok_digest, proof.clone());
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

impl ScanState {
    pub fn scan_statement(
        &self,
        constraint_constants: &ConstraintConstants,
        statement_check: StatementCheck,
        verifier: &Verifier,
    ) -> Result<Statement<()>, String> {
        struct Acc(Option<(Statement<()>, Vec<LedgerProofWithSokMessage>)>);

        let merge_acc = |mut proofs: Vec<LedgerProofWithSokMessage>,
                         acc: Acc,
                         s2: &Statement<()>|
         -> Result<Acc, String> {
            match acc.0 {
                None => Ok(Acc(Some((s2.clone(), proofs)))),
                Some((s1, mut ps)) => {
                    let merged_statement = s1.merge(s2)?;
                    proofs.append(&mut ps);
                    Ok(Acc(Some((merged_statement, proofs))))
                }
            }
        };

        let merge_pc = |acc: Option<Statement<()>>,
                        s2: &Statement<()>|
         -> Result<Option<Statement<()>>, String> {
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

        let fold_step_a = |(acc_statement, acc_pc): (Acc, Option<Statement<()>>),
                           job: &merge::Job<LedgerProofWithSokMessage>|
         -> Result<(Acc, Option<Statement<()>>), String> {
            use merge::{
                Job::{Empty, Full, Part},
                Record,
            };
            use JobStatus::Done;

            match job {
                Part(ref ledger @ LedgerProofWithSokMessage { ref proof, .. }) => {
                    let statement = proof.statement();
                    let acc_stmt = merge_acc(vec![ledger.clone()], acc_statement, &statement)?;
                    Ok((acc_stmt, acc_pc))
                }
                Empty | Full(Record { state: Done, .. }) => Ok((acc_statement, acc_pc)),
                Full(Record { left, right, .. }) => {
                    let LedgerProofWithSokMessage { proof: proof1, .. } = &left;
                    let LedgerProofWithSokMessage { proof: proof2, .. } = &right;

                    let stmt1 = proof1.statement();
                    let stmt2 = proof2.statement();
                    let merged_statement = stmt1.merge(&stmt2)?;

                    let acc_stmt = merge_acc(
                        vec![left.clone(), right.clone()],
                        acc_statement,
                        &merged_statement,
                    )?;

                    Ok((acc_stmt, acc_pc))
                }
            }
        };

        let check_base = |(acc_statement, acc_pc), transaction: &TransactionWithWitness| {
            use StatementCheck::{Full, Partial};

            let expected_statement = match &statement_check {
                Full(get_state) => create_expected_statement(
                    constraint_constants,
                    &**get_state,
                    transaction.statement.connecting_ledger_left,
                    transaction,
                )?,
                Partial => transaction.statement.clone(),
            };

            if transaction.statement == expected_statement {
                let acc_stmt = merge_acc(Vec::new(), acc_statement, &transaction.statement)?;
                let acc_pc = merge_pc(acc_pc, &transaction.statement)?;

                Ok((acc_stmt, acc_pc))
            } else {
                Err(format!(
                    "Bad base statement expected: {:#?} got: {:#?}",
                    transaction.statement, expected_statement
                ))
            }
        };

        let fold_step_d = |(acc_statement, acc_pc): (Acc, Option<Statement<()>>),
                           job: &base::Job<TransactionWithWitness>|
         -> Result<(Acc, Option<Statement<()>>), String> {
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
                }) => check_base((acc_statement, acc_pc), transaction),
            }
        };

        let res = self.scan_state.fold_chronological_until_err(
            (Acc(None), None),
            |acc, merge::Merge { weight: _, job }| fold_step_a(acc, job),
            |acc, base::Base { weight: _, job }| fold_step_d(acc, job),
            |v| v,
        )?;

        match res {
            (Acc(None), _) => Err("Empty".to_string()),
            (Acc(Some((res, proofs))), _) => match verifier.verify(proofs.as_slice()) {
                Ok(Ok(())) => Ok(res),
                Ok(Err(e)) => Err(format!("Verifier issue {:?}", e)),
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
        _last_proof_statement: Option<Statement<()>>,
        _registers_end: Registers,
    ) -> Result<(), String> {
        // TODO: OCaml does much more than this (pretty printing error)
        match self.scan_statement(constraint_constants, statement_check, verifier) {
            Ok(_) => Ok(()),
            Err(s) => Err(s),
        }
    }

    fn statement_of_job(job: &AvailableJob) -> Option<Statement<()>> {
        use super::parallel_scan::AvailableJob::{Base, Merge};

        match job {
            Base(TransactionWithWitness { statement, .. }) => Some(statement.clone()),
            Merge { left, right } => {
                let LedgerProofWithSokMessage { proof: p1, .. } = left;
                let LedgerProofWithSokMessage { proof: p2, .. } = right;

                p1.statement().merge(&p2.statement()).ok()
            }
        }
    }

    fn create(work_delay: u64, transaction_capacity_log_2: u64) -> Self {
        let k = 2u64.pow(transaction_capacity_log_2 as u32);

        Self {
            scan_state: ParallelScan::empty(k, work_delay),
            previous_incomplete_zkapp_updates: (
                Vec::with_capacity(1024),
                BorderBlockContinuedInTheNextTree(false),
            ),
        }
    }

    pub fn empty(constraint_constants: &ConstraintConstants) -> Self {
        let work_delay = constraint_constants.work_delay;
        let transaction_capacity_log_2 = constraint_constants.transaction_capacity_log_2;

        Self::create(work_delay, transaction_capacity_log_2)
    }

    fn extract_txn_and_global_slot(
        txn_with_witness: TransactionWithWitness,
    ) -> (WithStatus<Transaction>, Fp, Slot) {
        let txn = txn_with_witness.transaction_with_info.transaction();

        let state_hash = txn_with_witness.state_hash.0;
        let global_slot = txn_with_witness.block_global_slot;
        (txn, state_hash, global_slot)
    }

    fn latest_ledger_proof_impl(
        &self,
    ) -> Option<(
        &LedgerProofWithSokMessage,
        Vec<TransactionsOrdered<TransactionWithWitness>>,
    )> {
        let (proof, txns_with_witnesses) = self.scan_state.last_emitted_value()?;

        let (previous_incomplete, BorderBlockContinuedInTheNextTree(continued_in_next_tree)) =
            self.previous_incomplete_zkapp_updates.clone();

        let txns = {
            if continued_in_next_tree {
                TransactionsOrdered::first_and_second_pass_transactions_per_tree(
                    previous_incomplete,
                    txns_with_witnesses.clone(),
                )
            } else {
                let mut txns = TransactionsOrdered::first_and_second_pass_transactions_per_tree(
                    vec![],
                    txns_with_witnesses.clone(),
                );

                if previous_incomplete.is_empty() {
                    txns
                } else {
                    txns.insert(
                        0,
                        TransactionsOrdered {
                            first_pass: vec![],
                            second_pass: vec![],
                            previous_incomplete,
                            current_incomplete: vec![],
                        },
                    );
                    txns
                }
            }
        };

        Some((proof, txns))
    }

    pub fn latest_ledger_proof(
        &self,
    ) -> Option<(
        &LedgerProofWithSokMessage,
        Vec<TransactionsOrdered<(WithStatus<Transaction>, Fp, Slot)>>,
    )> {
        self.latest_ledger_proof_impl().map(|(p, txns)| {
            let txns = txns
                .into_iter()
                .map(|ordered| ordered.map(Self::extract_txn_and_global_slot))
                .collect::<Vec<_>>();

            (p, txns)
        })
    }

    fn incomplete_txns_from_recent_proof_tree(
        &self,
    ) -> Option<(
        &LedgerProofWithSokMessage,
        (
            Vec<TransactionWithWitness>,
            BorderBlockContinuedInTheNextTree,
        ),
    )> {
        let (proof, txns_per_block) = self.latest_ledger_proof_impl()?;

        let txns = match txns_per_block.last() {
            None => (vec![], BorderBlockContinuedInTheNextTree(false)),
            Some(txns_in_last_block) => {
                // First pass ledger is considered as the snarked ledger, so any
                // account update whether completed in the same tree or not
                // should be included in the next tree

                if !txns_in_last_block.second_pass.is_empty() {
                    (
                        txns_in_last_block.second_pass.clone(),
                        BorderBlockContinuedInTheNextTree(false),
                    )
                } else {
                    (
                        txns_in_last_block.current_incomplete.clone(),
                        BorderBlockContinuedInTheNextTree(true),
                    )
                }
            }
        };

        Some((proof, txns))
    }

    fn staged_transactions(&self) -> Vec<TransactionsOrdered<TransactionWithWitness>> {
        let (previous_incomplete, BorderBlockContinuedInTheNextTree(continued_in_next_tree)) =
            match self.incomplete_txns_from_recent_proof_tree() {
                Some((_proof, v)) => v,
                None => (vec![], BorderBlockContinuedInTheNextTree(false)),
            };

        let txns = {
            if continued_in_next_tree {
                TransactionsOrdered::first_and_second_pass_transactions_per_forest(
                    self.scan_state.pending_data(),
                    previous_incomplete,
                )
            } else {
                let mut txns = TransactionsOrdered::first_and_second_pass_transactions_per_forest(
                    self.scan_state.pending_data(),
                    vec![],
                );

                if previous_incomplete.is_empty() {
                    txns
                } else {
                    txns.insert(
                        0,
                        vec![TransactionsOrdered {
                            first_pass: vec![],
                            second_pass: vec![],
                            previous_incomplete,
                            current_incomplete: vec![],
                        }],
                    );
                    txns
                }
            }
        };

        txns.into_iter().flatten().collect::<Vec<_>>()
    }

    /// All the transactions in the order in which they were applied along with
    /// the parent protocol state of the blocks that contained them
    fn staged_transactions_with_state_hash(
        &self,
    ) -> Vec<TransactionsOrdered<(WithStatus<Transaction>, Fp, Slot)>> {
        self.staged_transactions()
            .into_iter()
            .map(|ordered| ordered.map(Self::extract_txn_and_global_slot))
            .collect::<Vec<_>>()
    }

    fn apply_ordered_txns_stepwise<L, F, ApplyFirst, ApplySecond, ApplyFirstSparse>(
        stop_at_first_pass: Option<bool>,
        ordered_txns: Vec<TransactionsOrdered<TransactionWithWitness>>,
        ledger: &mut L,
        get_protocol_state: F,
        apply_first_pass: ApplyFirst,
        apply_second_pass: ApplySecond,
        apply_first_pass_sparse_ledger: ApplyFirstSparse,
    ) -> Result<Pass, String>
    where
        L: LedgerIntf + Clone,
        F: Fn(Fp) -> Result<MinaStateProtocolStateValueStableV2, String>,
        ApplyFirst: Fn(
            Slot,
            &ProtocolStateView,
            &mut L,
            &Transaction,
        ) -> Result<TransactionPartiallyApplied<L>, String>,
        ApplySecond:
            Fn(&mut L, TransactionPartiallyApplied<L>) -> Result<TransactionApplied, String>,
        ApplyFirstSparse:
            Fn(
                Slot,
                &ProtocolStateView,
                &mut SparseLedger<AccountId, Account>,
                &Transaction,
            )
                -> Result<TransactionPartiallyApplied<SparseLedger<AccountId, Account>>, String>,
    {
        let stop_at_first_pass = stop_at_first_pass.unwrap_or(false);

        enum PreviousIncompleteTxns<L: LedgerIntf + Clone> {
            Unapplied(Vec<TransactionWithWitness>),
            PartiallyApplied(Vec<(TransactionStatus, TransactionPartiallyApplied<L>)>),
        }

        fn apply<L, F, Apply>(
            apply: Apply,
            ledger: &mut L,
            tx: &Transaction,
            state_hash: Fp,
            block_global_slot: Slot,
            get_protocol_state: F,
        ) -> Result<TransactionPartiallyApplied<L>, String>
        where
            L: LedgerIntf + Clone,
            F: Fn(Fp) -> Result<MinaStateProtocolStateValueStableV2, String>,
            Apply: Fn(
                Slot,
                &ProtocolStateView,
                &mut L,
                &Transaction,
            ) -> Result<TransactionPartiallyApplied<L>, String>,
        {
            match get_protocol_state(state_hash) {
                Ok(state) => {
                    let txn_state_view = protocol_state_view(&state);
                    apply(block_global_slot, &txn_state_view, ledger, tx)
                }
                Err(e) => Err(format!(
                    "Coudln't find protocol state with hash {:?}: {}",
                    state_hash, e
                )),
            }
        }

        // let apply = |apply: ApplyFirst, ledger: &mut L, tx: &Transaction, state_hash: Fp, block_global_slot: Slot| {
        //     match get_protocol_state(state_hash) {
        //         Ok(state) => {
        //             let txn_state_view = protocol_state_view(&state);
        //             apply(block_global_slot, &txn_state_view, ledger, tx)
        //         },
        //         Err(e) => {
        //             Err(format!("Coudln't find protocol state with hash {:?}: {}", state_hash, e))
        //         },
        //     }
        // };

        type Acc<L> = Vec<(TransactionStatus, TransactionPartiallyApplied<L>)>;

        let apply_txns_first_pass = |acc: Acc<L>,
                                     txns: Vec<TransactionWithWitness>,
                                     k: Box<dyn Fn(Pass, Acc<L>) -> Result<Pass, String>>|
         -> Result<Pass, String> {
            let ledger = ledger.clone();

            for txn in txns {
                let (transaction, state_hash, block_global_slot) =
                    Self::extract_txn_and_global_slot(txn);
                let expected_status = transaction.status;

                let partially_applied_txn = apply(
                    &apply_first_pass,
                    &mut ledger,
                    &transaction.data,
                    state_hash,
                    block_global_slot,
                    &get_protocol_state,
                )?;

                acc.push((expected_status, partially_applied_txn));
            }

            k(Pass::FirstPassLedgerHash(ledger.merkle_root()), acc)
        };

        let apply_txns_second_pass = |partially_applied_txns: Acc<L>,
                                      k: Box<dyn Fn() -> Result<Pass, String>>|
         -> Result<Pass, String> {
            let ledger = ledger.clone();

            for (expected_status, partially_applied_txn) in partially_applied_txns {
                let res = apply_second_pass(&mut ledger, partially_applied_txn)?;
                let status = res.transaction_status();

                if &expected_status != status {
                    return Err(format!(
                        "Transaction produced unxpected application status.\
                                        Expected {:#?}\
                                        Got: {:#?}\
                                        Transaction: {:#?}",
                        expected_status, status, "TODO"
                    ));
                    // Transaction: {:#?}", expected_status, status, partially_applied_txn));
                }
            }

            k()
        };

        let apply_previous_incomplete_txns = |txns: PreviousIncompleteTxns<L>,
                                              k: Box<dyn Fn() -> Result<Pass, String>>|
         -> Result<Pass, String> {
            // Note: Previous incomplete transactions refer to the block's transactions
            // from previous scan state tree that were split between the two trees.
            // The set in the previous tree have gone through the first pass. For the
            // second pass that is to happen after the rest of the set goes through the
            // first pass, we need partially applied state - result of previous tree's
            // transactions' first pass. To generate the partial state, we do a a first
            // pass application of previous tree's transaction on a sparse ledger created
            // from witnesses stored in the scan state and then use it to apply to the
            // ledger here

            let inject_ledger_info = |partially_applied_txn: TransactionPartiallyApplied<
                SparseLedger<AccountId, Account>,
            >| {
                use TransactionPartiallyApplied as P;

                match partially_applied_txn {
                    P::ZkappCommand(zkapp) => {
                        let original_first_pass_account_states = zkapp
                            .original_first_pass_account_states
                            .into_iter()
                            .map(|(id, loc_opt)| match loc_opt {
                                None => Ok((id, None)),
                                Some((_sparse_ledger_loc, account)) => {
                                    match ledger.location_of_account(&id) {
                                        Some(loc) => Ok((id, Some((loc, account)))),
                                        None => {
                                            Err("Original accounts states from partially applied \
                                                         transactions don't exist in the ledger")
                                        }
                                    }
                                }
                            })
                            .collect::<Result<Vec<_>, &'static str>>()
                            .unwrap(); // TODO

                        let global_state = GlobalState {
                            first_pass_ledger: ledger.clone(),
                            second_pass_ledger: ledger.clone(),
                            fee_excess: zkapp.global_state.fee_excess,
                            supply_increase: zkapp.global_state.supply_increase,
                            protocol_state: zkapp.global_state.protocol_state,
                            block_global_slot: zkapp.global_state.block_global_slot,
                        };

                        let local_state = LocalStateEnv {
                            stack_frame: zkapp.local_state.stack_frame,
                            call_stack: zkapp.local_state.call_stack,
                            transaction_commitment: zkapp.local_state.transaction_commitment,
                            full_transaction_commitment: zkapp
                                .local_state
                                .full_transaction_commitment,
                            token_id: zkapp.local_state.token_id,
                            excess: zkapp.local_state.excess,
                            supply_increase: zkapp.local_state.supply_increase,
                            ledger: ledger.clone(),
                            success: zkapp.local_state.success,
                            account_update_index: zkapp.local_state.account_update_index,
                            failure_status_tbl: zkapp.local_state.failure_status_tbl,
                            will_succeed: zkapp.local_state.will_succeed,
                        };

                        TransactionPartiallyApplied::ZkappCommand(ZkappCommandPartiallyApplied {
                            command: zkapp.command,
                            previous_hash: zkapp.previous_hash,
                            original_first_pass_account_states,
                            constraint_constants: zkapp.constraint_constants,
                            state_view: zkapp.state_view,
                            global_state,
                            local_state,
                        })
                    }
                    P::SignedCommand(c) => P::SignedCommand(c),
                    P::FeeTransfer(ft) => P::FeeTransfer(ft),
                    P::Coinbase(cb) => P::Coinbase(cb),
                }
            };

            let apply_txns_to_witnesses_first_pass = |acc: Acc<L>,
                                                      txns: Vec<TransactionWithWitness>,
                                                      k: Box<
                dyn Fn(Acc<L>) -> Result<Pass, String>,
            >| {
                let acc = txns
                    .into_iter()
                    .map(|txn| {
                        let (transaction, state_hash, block_global_slot) =
                            Self::extract_txn_and_global_slot(txn);
                        let expected_status = transaction.status.clone();

                        let mut first_pass_ledger_witness = txn.first_pass_ledger_witness.clone();
                        let partially_applied_txn = apply(
                            &apply_first_pass_sparse_ledger,
                            &mut first_pass_ledger_witness,
                            &transaction.data,
                            state_hash,
                            block_global_slot,
                            &get_protocol_state,
                        )?;

                        let partially_applied_txn = inject_ledger_info(partially_applied_txn);

                        Ok((expected_status, partially_applied_txn))
                    })
                    .collect::<Result<Vec<_>, String>>()?;

                k(acc)
            };

            use PreviousIncompleteTxns::{PartiallyApplied, Unapplied};

            match txns {
                Unapplied(txns) => apply_txns_to_witnesses_first_pass(
                    Vec::with_capacity(txns.len()),
                    txns,
                    Box::new(|partially_applied_txns| {
                        apply_txns_second_pass(partially_applied_txns, k)
                    }),
                ),
                PartiallyApplied(partially_applied_txns) => {
                    apply_txns_second_pass(partially_applied_txns, k)
                }
            }
        };

        fn apply_txns<L>(
            previous_incomplete: PreviousIncompleteTxns<L>,
            ordered_txns: Vec<TransactionsOrdered<TransactionWithWitness>>,
            first_pass_ledger_hash: Pass,
            stop_at_first_pass: bool,
            apply_previous_incomplete_txns: impl Fn(
                PreviousIncompleteTxns<L>,
                Box<dyn Fn() -> Result<Pass, String>>,
            ) -> Result<Pass, String>,
            apply_txns_first_pass: impl Fn(
                Acc<L>,
                Vec<TransactionWithWitness>,
                Box<dyn Fn(Pass, Acc<L>) -> Result<Pass, String>>,
            ) -> Result<Pass, String>,
            apply_txns_second_pass: impl Fn(
                Acc<L>,
                Box<dyn Fn() -> Result<Pass, String>>,
            ) -> Result<Pass, String>,
        ) -> Result<Pass, String>
        where
            L: LedgerIntf + Clone,
        {
            use PreviousIncompleteTxns::{PartiallyApplied, Unapplied};

            // filter out any non-zkapp transactions for second pass application
            let previous_incomplete = match previous_incomplete {
                Unapplied(txns) => Unapplied(
                    txns.into_iter()
                        .filter(|txn| {
                            use crate::scan_state::transaction_logic::transaction_applied::{
                                CommandApplied::ZkappCommand, Varying::Command,
                            };

                            matches!(&txn.transaction_with_info.varying, Command(ZkappCommand(_)))
                        })
                        .collect(),
                ),
                PartiallyApplied(txns) => PartiallyApplied(
                    txns.into_iter()
                        .filter(|(_, txn)| {
                            matches!(&txn, TransactionPartiallyApplied::ZkappCommand(_))
                        })
                        .collect(),
                ),
            };

            match ordered_txns.len() {
                0 => apply_previous_incomplete_txns(
                    previous_incomplete,
                    Box::new(|| Ok(first_pass_ledger_hash)),
                ),
                1 if stop_at_first_pass => {
                    // Last block; don't apply second pass.
                    // This is for snarked ledgers which are first pass ledgers
                    let txns_per_block = ordered_txns[0];
                    apply_txns_first_pass(
                        Vec::with_capacity(256),
                        txns_per_block.first_pass,
                        Box::new(|first_pass_ledger_hash, _partially_applied_txns| {
                            // Skip previous_incomplete: If there are previous_incomplete txns
                            // then there’d be at least two sets of txns_per_block and the
                            // previous_incomplete txns will be applied when processing the first
                            // set. The subsequent sets shouldn’t have any previous-incomplete.

                            apply_txns(
                                Unapplied(vec![]),
                                vec![],
                                first_pass_ledger_hash,
                                stop_at_first_pass,
                                apply_previous_incomplete_txns,
                                apply_txns_first_pass,
                                apply_txns_second_pass,
                            )
                        }),
                    )
                }
                _ => {
                    let (txns_per_block, ordered_txns) = {
                        let first = ordered_txns.remove(0);
                        (first, ordered_txns)
                    };

                    // Apply first pass of a blocks transactions either new or
                    // continued from previous tree
                    apply_txns_first_pass(
                        Vec::with_capacity(256),
                        txns_per_block.first_pass,
                        Box::new(|first_pass_ledger_hash, partially_applied_txns| {
                            // Apply second pass of previous tree's transactions, if any

                            apply_previous_incomplete_txns(
                                previous_incomplete,
                                Box::new(|| {
                                    let continue_previous_tree_s_txns = {
                                        // If this is a continuation from previous tree for
                                        // the same block (incomplete txns in both sets) then
                                        // do second pass now
                                        let previous_not_empty = match &previous_incomplete {
                                            Unapplied(txns) => !txns.is_empty(),
                                            PartiallyApplied(txns) => !txns.is_empty(),
                                        };

                                        previous_not_empty
                                            && !txns_per_block.current_incomplete.is_empty()
                                    };

                                    let do_second_pass = {
                                        // if transactions completed in the same tree; do second pass now
                                        (!txns_per_block.second_pass.is_empty())
                                            || continue_previous_tree_s_txns
                                    };

                                    if do_second_pass {
                                        apply_txns_second_pass(
                                            partially_applied_txns,
                                            Box::new(|| {
                                                apply_txns(
                                                    Unapplied(vec![]),
                                                    ordered_txns,
                                                    first_pass_ledger_hash,
                                                    stop_at_first_pass,
                                                    apply_previous_incomplete_txns,
                                                    apply_txns_first_pass,
                                                    apply_txns_second_pass,
                                                )
                                            }),
                                        )
                                    } else {
                                        // Transactions not completed in this tree, so second pass after
                                        // first pass of remaining transactions for the same block
                                        // in the next tree

                                        apply_txns(
                                            PartiallyApplied(partially_applied_txns),
                                            ordered_txns,
                                            first_pass_ledger_hash,
                                            stop_at_first_pass,
                                            apply_previous_incomplete_txns,
                                            apply_txns_first_pass,
                                            apply_txns_second_pass,
                                        )
                                    }
                                }),
                            )
                        }),
                    )
                }
            }
        }

        let previous_incomplete = match ordered_txns.first() {
            None => PreviousIncompleteTxns::<L>::Unapplied(vec![]),
            Some(first_block) => {
                PreviousIncompleteTxns::Unapplied(first_block.previous_incomplete.clone())
            }
        };

        // Assuming this function is called on snarked ledger and snarked
        // ledger is the first pass ledger
        let first_pass_ledger_hash = Pass::FirstPassLedgerHash(ledger.merkle_root());

        apply_txns(
            previous_incomplete,
            ordered_txns,
            first_pass_ledger_hash,
            stop_at_first_pass,
            apply_previous_incomplete_txns,
            apply_txns_first_pass,
            apply_txns_second_pass,
        )
    }

    fn apply_ordered_txns_sync<L, F, ApplyFirst, ApplySecond, ApplyFirstSparse>(
        stop_at_first_pass: Option<bool>,
        ordered_txns: Vec<TransactionsOrdered<TransactionWithWitness>>,
        ledger: &mut L,
        get_protocol_state: F,
        apply_first_pass: ApplyFirst,
        apply_second_pass: ApplySecond,
        apply_first_pass_sparse_ledger: ApplyFirstSparse,
    ) -> Result<Pass, String>
    where
        L: LedgerIntf + Clone,
        F: Fn(Fp) -> Result<MinaStateProtocolStateValueStableV2, String>,
        ApplyFirst: Fn(
            Slot,
            &ProtocolStateView,
            &mut L,
            &Transaction,
        ) -> Result<TransactionPartiallyApplied<L>, String>,
        ApplySecond:
            Fn(&mut L, TransactionPartiallyApplied<L>) -> Result<TransactionApplied, String>,
        ApplyFirstSparse:
            Fn(
                Slot,
                &ProtocolStateView,
                &mut SparseLedger<AccountId, Account>,
                &Transaction,
            )
                -> Result<TransactionPartiallyApplied<SparseLedger<AccountId, Account>>, String>,
    {
        Self::apply_ordered_txns_stepwise(
            stop_at_first_pass,
            ordered_txns,
            ledger,
            get_protocol_state,
            apply_first_pass,
            apply_second_pass,
            apply_first_pass_sparse_ledger,
        )
    }

    fn get_snarked_ledger_sync<L, F, ApplyFirst, ApplySecond, ApplyFirstSparse>(
        &self,
        ordered_txns: Vec<TransactionsOrdered<TransactionWithWitness>>,
        ledger: &mut L,
        get_protocol_state: F,
        apply_first_pass: ApplyFirst,
        apply_second_pass: ApplySecond,
        apply_first_pass_sparse_ledger: ApplyFirstSparse,
    ) -> Result<Pass, String>
    where
        L: LedgerIntf + Clone,
        F: Fn(Fp) -> Result<MinaStateProtocolStateValueStableV2, String>,
        ApplyFirst: Fn(
            Slot,
            &ProtocolStateView,
            &mut L,
            &Transaction,
        ) -> Result<TransactionPartiallyApplied<L>, String>,
        ApplySecond:
            Fn(&mut L, TransactionPartiallyApplied<L>) -> Result<TransactionApplied, String>,
        ApplyFirstSparse:
            Fn(
                Slot,
                &ProtocolStateView,
                &mut SparseLedger<AccountId, Account>,
                &Transaction,
            )
                -> Result<TransactionPartiallyApplied<SparseLedger<AccountId, Account>>, String>,
    {
        match self.latest_ledger_proof_impl() {
            None => Err("No transactions found".to_string()),
            Some((_, txns_per_block)) => Self::apply_ordered_txns_sync(
                Some(true),
                txns_per_block,
                ledger,
                get_protocol_state,
                apply_first_pass,
                apply_second_pass,
                apply_first_pass_sparse_ledger,
            ),
        }
    }

    pub fn get_staged_ledger_sync<L, F, ApplyFirst, ApplySecond, ApplyFirstSparse>(
        &self,
        ledger: &mut L,
        get_protocol_state: F,
        apply_first_pass: ApplyFirst,
        apply_second_pass: ApplySecond,
        apply_first_pass_sparse_ledger: ApplyFirstSparse,
    ) -> Result<Pass, String>
    where
        L: LedgerIntf + Clone,
        F: Fn(Fp) -> Result<MinaStateProtocolStateValueStableV2, String>,
        ApplyFirst: Fn(
            Slot,
            &ProtocolStateView,
            &mut L,
            &Transaction,
        ) -> Result<TransactionPartiallyApplied<L>, String>,
        ApplySecond:
            Fn(&mut L, TransactionPartiallyApplied<L>) -> Result<TransactionApplied, String>,
        ApplyFirstSparse:
            Fn(
                Slot,
                &ProtocolStateView,
                &mut SparseLedger<AccountId, Account>,
                &Transaction,
            )
                -> Result<TransactionPartiallyApplied<SparseLedger<AccountId, Account>>, String>,
    {
        let staged_transactions_with_state_hash = self.staged_transactions();
        Self::apply_ordered_txns_sync(
            None,
            staged_transactions_with_state_hash,
            ledger,
            get_protocol_state,
            apply_first_pass,
            apply_second_pass,
            apply_first_pass_sparse_ledger,
        )
    }

    pub fn free_space(&self) -> u64 {
        self.scan_state.free_space()
    }

    fn all_jobs(&self) -> Vec<Vec<AvailableJob>> {
        self.scan_state.all_jobs()
    }

    pub fn next_on_new_tree(&self) -> bool {
        self.scan_state.next_on_new_tree()
    }

    pub fn base_jobs_on_latest_tree(&self) -> impl Iterator<Item = TransactionWithWitness> {
        self.scan_state.base_jobs_on_latest_tree()
    }

    pub fn base_jobs_on_earlier_tree(
        &self,
        index: usize,
    ) -> impl Iterator<Item = TransactionWithWitness> {
        self.scan_state.base_jobs_on_earlier_tree(index)
    }

    pub fn partition_if_overflowing(&self) -> SpacePartition {
        let bundle_count = |work_count: u64| (work_count + 1) / 2;

        // slots: current tree space
        // job_count: work count on current tree
        let SpacePartition {
            first: (slots, job_count),
            second,
        } = self.scan_state.partition_if_overflowing();

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
                first_pass_ledger_witness: d.first_pass_ledger_witness,
                second_pass_ledger_witness: d.second_pass_ledger_witness,
                init_stack: d.init_stack,
                block_global_slot: d.block_global_slot,
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
        let work_list = self.scan_state.jobs_for_slots(slots);
        work_list
            .iter()
            .flat_map(|works| group_list(works, |job| job.clone()))
            .collect()
    }

    pub fn k_work_pairs_for_new_diff(&self, k: u64) -> Vec<OneOrTwo<AvailableJob>> {
        let work_list = self.scan_state.jobs_for_next_update();
        work_list
            .iter()
            .flat_map(|works| group_list(works, |job| job.clone()))
            .take(k as usize)
            .collect()
    }

    // Always the same pairing of jobs
    pub fn work_statements_for_new_diff(&self) -> Vec<transaction_snark::work::Statement> {
        let work_list = self.scan_state.jobs_for_next_update();

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
                first_pass_ledger_witness,
                second_pass_ledger_witness,
                init_stack,
                block_global_slot,
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
                        protocol_state_body,
                        init_stack,
                        status,
                        first_pass_ledger: first_pass_ledger_witness,
                        second_pass_ledger: second_pass_ledger_witness,
                        block_global_slot,
                    }
                };

                Ok(snark_work::spec::Work::Transition((statement, witness)))
            }
            Extracted::Second(s) => {
                let merged = s.0.statement().merge(&s.1.statement())?;
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

    pub fn fill_work_and_enqueue_transactions(
        &mut self,
        transactions: Vec<TransactionWithWitness>,
        work: Vec<transaction_snark::work::Unchecked>,
    ) -> Result<
        Option<(
            LedgerProof,
            Vec<TransactionsOrdered<(WithStatus<Transaction>, Fp, Slot)>>,
        )>,
        String,
    > {
        {
            use crate::scan_state::transaction_logic::transaction_applied::Varying::*;

            println!("{} transactions added to scan state:", transactions.len());
            println!(
                "- num_fee_transfer={:?}",
                transactions
                    .iter()
                    .filter(|tx| matches!(tx.transaction_with_info.varying, FeeTransfer(_)))
                    .count()
            );

            println!(
                "- num_coinbase={:?}",
                transactions
                    .iter()
                    .filter(|tx| matches!(tx.transaction_with_info.varying, Coinbase(_)))
                    .count()
            );

            println!(
                "- num_user_command={:?}",
                transactions
                    .iter()
                    .filter(|tx| matches!(tx.transaction_with_info.varying, Command(_)))
                    .count()
            );
        }

        let fill_in_transaction_snark_work =
            |works: Vec<transaction_snark::work::Work>| -> Result<Vec<LedgerProofWithSokMessage>, String>
        {
            let next_jobs = self
                .scan_state
                .jobs_for_next_update()
                .into_iter()
                .flatten()
                .take(total_proofs(&works));

            let works = works.into_iter().flat_map(|transaction_snark::work::Work { fee, proofs, prover }| {
                proofs.into_map(|proof| (fee, proof, prover.clone())).into_iter()
            });

            next_jobs.zip(works).map(|(job, work)| completed_work_to_scanable_work(job, work)).collect()
        };

        // get incomplete transactions from previous proof which will be completed in
        // the new proof, if there's one
        let old_proof_and_incomplete_zkapp_updates = self.incomplete_txns_from_recent_proof_tree();
        let work_list = fill_in_transaction_snark_work(work)?;

        let proof_opt = self
            .scan_state
            .update(transactions, work_list, |base| {
                // TODO: This is for logs only, make it cleaner
                match base.transaction_with_info.varying {
                    super::transaction_logic::transaction_applied::Varying::Command(_) => 0,
                    super::transaction_logic::transaction_applied::Varying::FeeTransfer(_) => 1,
                    super::transaction_logic::transaction_applied::Varying::Coinbase(_) => 2,
                }
            })
            .unwrap();

        match proof_opt {
            None => Ok(None),
            Some((LedgerProofWithSokMessage { proof, .. }, txns_with_witnesses)) => {
                let curr_stmt = proof.statement();

                let (prev_stmt, incomplete_zkapp_updates_from_old_proof) =
                    match old_proof_and_incomplete_zkapp_updates {
                        None => (
                            curr_stmt,
                            (vec![], BorderBlockContinuedInTheNextTree(false)),
                        ),
                        Some((proof_with_sok, incomplete_zkapp_updates_from_old_proof)) => {
                            let proof = &proof_with_sok.proof;
                            (proof.statement(), incomplete_zkapp_updates_from_old_proof)
                        }
                    };

                // prev_target is connected to curr_source- Order of the arguments is
                // important here
                let stmts_connect = if prev_stmt == curr_stmt {
                    Ok(())
                } else {
                    prev_stmt.merge(&curr_stmt).map(|_| ())
                };

                match stmts_connect {
                    Ok(()) => {
                        self.previous_incomplete_zkapp_updates =
                            incomplete_zkapp_updates_from_old_proof;

                        // This block is for when there's a proof emitted so Option.
                        // value_exn is safe here
                        // [latest_ledger_proof] generates ordered transactions
                        // appropriately*)
                        let (proof_with_sok, txns) = self.latest_ledger_proof().unwrap();

                        Ok(Some((proof_with_sok.proof.clone(), txns)))
                    }
                    Err(e) => Err(format!(
                        "The new final statement does not connect to the previous \
                                     proof's statement: {:?}",
                        e
                    )),
                }
            }
        }
    }

    fn required_state_hashes(&self) -> HashSet<Fp> {
        self.staged_transactions()
            .into_iter()
            .fold(HashSet::with_capacity(256), |accum, txns| {
                txns.fold(accum, |accum, txn| {
                    accum.insert(txn.state_hash.0);
                    accum
                })
            })
    }

    fn check_required_protocol_states(&self, _protocol_states: ()) {
        todo!() // Not sure what is the type of `protocol_states` here
    }
}

pub fn group_list<'a, F, T, R>(slice: &'a [T], fun: F) -> impl Iterator<Item = OneOrTwo<R>> + '_
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
        statement: Box<Statement<()>>,
        state_hash: (Fp, Fp),
        first_pass_ledger_witness: SparseLedger<AccountId, Account>,
        second_pass_ledger_witness: SparseLedger<AccountId, Account>,
        init_stack: InitStack,
        block_global_slot: Slot,
    },
    Second(Box<(LedgerProof, LedgerProof)>),
}

#[derive(Debug)]
pub struct TransactionsOrdered<T> {
    pub first_pass: Vec<T>,
    pub second_pass: Vec<T>,
    pub previous_incomplete: Vec<T>,
    pub current_incomplete: Vec<T>,
}

impl<T> TransactionsOrdered<T> {
    fn map<B>(self, fun: impl FnMut(T) -> B) -> TransactionsOrdered<B> {
        let Self {
            first_pass,
            second_pass,
            previous_incomplete,
            current_incomplete,
        } = self;

        let conv = |v: Vec<T>| v.into_iter().map(fun).collect();

        TransactionsOrdered::<B> {
            first_pass: conv(first_pass),
            second_pass: conv(second_pass),
            previous_incomplete: conv(previous_incomplete),
            current_incomplete: conv(current_incomplete),
        }
    }

    fn fold<A>(&self, init: A, fun: impl Fn(A, &T) -> A) -> A {
        let Self {
            first_pass,
            second_pass,
            previous_incomplete,
            current_incomplete,
        } = self;

        let init = first_pass.iter().fold(init, fun);
        let init = previous_incomplete.iter().fold(init, fun);
        let init = second_pass.iter().fold(init, fun);
        current_incomplete.iter().fold(init, fun)
    }
}

impl TransactionsOrdered<TransactionWithWitness> {
    fn first_and_second_pass_transactions_per_tree(
        previous_incomplete: Vec<TransactionWithWitness>,
        txns_per_tree: Vec<TransactionWithWitness>,
    ) -> Vec<Self> {
        let complete_and_incomplete_transactions = |txs: Vec<TransactionWithWitness>| -> Option<
            TransactionsOrdered<TransactionWithWitness>,
        > {
            let target_first_pass_ledger = txs.get(0)?.statement.source.first_pass_ledger;
            let first_state_hash = txs.get(0)?.state_hash.0;

            let first_pass_txns = Vec::with_capacity(txns_per_tree.len());
            let second_pass_txns = Vec::with_capacity(txns_per_tree.len());

            let (first_pass_txns, second_pass_txns, target_first_pass_ledger) =
                txs.into_iter().fold(
                    (first_pass_txns, second_pass_txns, target_first_pass_ledger),
                    |(first_pass_txns, second_pass_txns, _old_root), txn_with_witness| {
                        let txn = txn_with_witness.transaction_with_info.transaction();
                        let target_first_pass_ledger =
                            txn_with_witness.statement.target.first_pass_ledger;

                        use crate::scan_state::transaction_logic::UserCommand::*;
                        use Transaction::*;

                        match txn.data {
                            Coinbase(_) | FeeTransfer(_) | Command(SignedCommand(_)) => {
                                first_pass_txns.push(txn_with_witness);
                            }
                            Command(ZkAppCommand(_)) => {
                                first_pass_txns.push(txn_with_witness);
                                second_pass_txns.push(txn_with_witness);
                            }
                        }

                        (first_pass_txns, second_pass_txns, target_first_pass_ledger)
                    },
                );

            let (second_pass_txns, incomplete_txns) = match second_pass_txns.first() {
                None => (vec![], vec![]),
                Some(txn_with_witness) => {
                    if txn_with_witness.statement.source.second_pass_ledger
                        == target_first_pass_ledger
                    {
                        // second pass completed in the same tree
                        (second_pass_txns, vec![])
                    } else {
                        (vec![], second_pass_txns)
                    }
                }
            };

            let previous_incomplete = match previous_incomplete.first() {
                None => vec![],
                Some(tx) => {
                    if tx.state_hash.0 == first_state_hash {
                        // same block
                        previous_incomplete.clone()
                    } else {
                        vec![]
                    }
                }
            };

            Some(Self {
                first_pass: first_pass_txns,
                second_pass: second_pass_txns,
                current_incomplete: incomplete_txns,
                previous_incomplete,
            })
        };

        let txns_by_block = |txns_per_tree: Vec<TransactionWithWitness>| {
            let mut global = Vec::with_capacity(txns_per_tree.len());

            let make_current = || Vec::<TransactionWithWitness>::with_capacity(txns_per_tree.len());
            let mut current = make_current();

            for next in txns_per_tree {
                if current
                    .last()
                    .map(|last| last.state_hash.0 != next.state_hash.0)
                    .unwrap_or(false)
                {
                    global.push(current);
                    current = make_current();
                }

                current.push(next);
            }

            if !current.is_empty() {
                global.push(current);
            }

            global
        };

        txns_by_block(txns_per_tree)
            .into_iter()
            .filter_map(complete_and_incomplete_transactions)
            .collect()
    }

    fn first_and_second_pass_transactions_per_forest(
        scan_state_txns: Vec<Vec<TransactionWithWitness>>,
        previous_incomplete: Vec<TransactionWithWitness>,
    ) -> Vec<Vec<Self>> {
        scan_state_txns
            .into_iter()
            .map(|txns_per_tree| {
                Self::first_and_second_pass_transactions_per_tree(
                    previous_incomplete,
                    txns_per_tree,
                )
            })
            .collect()
    }
}

pub enum Pass {
    FirstPassLedgerHash(Fp),
}
