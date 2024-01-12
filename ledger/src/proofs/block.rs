use std::{path::Path, rc::Rc, str::FromStr, sync::Arc};

use kimchi::{proof::RecursionChallenge, verifier_index::VerifierIndex};
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use mina_p2p_messages::v2;

use crate::{
    dummy,
    proofs::{
        block::consensus::ConsensusState,
        constants::{
            make_step_block_data, make_step_transaction_data, StepBlockProof, WrapBlockProof,
        },
        merge::{
            extract_recursion_challenges, verify_one, ForStep, ForStepKind, PerProofWitness,
            StatementProofState,
        },
        numbers::{
            currency::CheckedSigned,
            nat::{CheckedNat, CheckedSlot},
        },
        unfinalized::{evals_from_p2p, AllEvals, EvalsWithPublicInput, Unfinalized},
        util::{sha256_sum, u64_to_field},
        verifier_index::wrap_domains,
        witness::{
            create_proof, get_messages_for_next_wrap_proof_padding, Boolean, InnerCurve,
            MessagesForNextStepProof, ReducedMessagesForNextStepProof, ToFieldElementsDebug,
        },
        wrap::{wrap, wrap_verifier, CircuitVar, Domain, WrapParams},
    },
    scan_state::{
        fee_excess::{self, FeeExcess},
        pending_coinbase::{PendingCoinbase, PendingCoinbaseWitness, Stack},
        protocol_state::MinaHash,
        scan_state::{
            transaction_snark::{
                validate_ledgers_at_merge_checked, Registers, SokDigest, Statement,
                StatementLedgers,
            },
            ForkConstants,
        },
        transaction_logic::protocol_state::EpochLedger,
    },
    verifier::get_srs,
    Inputs, ToInputs,
};

use super::{
    constants::ProofConstants,
    merge::{ExpandedProof, InductiveRule, OptFlag, PreviousProofStatement},
    numbers::{
        currency::CheckedAmount,
        nat::{CheckedBlockTime, CheckedBlockTimeSpan, CheckedLength},
    },
    to_field_elements::ToFieldElements,
    witness::{
        field,
        transaction_snark::{checked_hash, CONSTRAINT_CONSTANTS},
        Check, CircuitPlonkVerificationKeyEvals, GroupAffine, Prover, StepStatement, Witness,
    },
    wrap::WrapProof,
};

fn read_witnesses() -> Vec<Fp> {
    let f = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("rampup4")
            .join("block_fps.txt"),
    )
    .unwrap();

    let fps = f
        .lines()
        .filter(|s| !s.is_empty())
        .map(|s| Fp::from_str(s).unwrap())
        .collect::<Vec<_>>();

    fps
}

impl ToFieldElements<Fp> for v2::MinaStateSnarkTransitionValueStableV2 {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self {
            blockchain_state,
            consensus_transition,
            pending_coinbase_update:
                v2::MinaBasePendingCoinbaseUpdateStableV1 {
                    action,
                    coinbase_amount,
                },
        } = self;

        blockchain_state.to_field_elements(fields);
        fields.push(Fp::from(consensus_transition.as_u32()));

        // https://github.com/MinaProtocol/mina/blob/f6fb903bef974b191776f393a2f9a1e6360750fe/src/lib/mina_base/pending_coinbase.ml#L420
        use v2::MinaBasePendingCoinbaseUpdateActionStableV1::*;
        let bits = match action {
            UpdateNone => [Boolean::False, Boolean::False],
            UpdateOne => [Boolean::True, Boolean::False],
            UpdateTwoCoinbaseInFirst => [Boolean::False, Boolean::True],
            UpdateTwoCoinbaseInSecond => [Boolean::True, Boolean::True],
        };
        fields.extend(bits.into_iter().map(Boolean::to_field::<Fp>));
        fields.push(Fp::from(coinbase_amount.as_u64()));
    }
}

impl Check<Fp> for v2::MinaStateSnarkTransitionValueStableV2 {
    fn check(&self, w: &mut Witness<Fp>) {
        let Self {
            blockchain_state,
            consensus_transition,
            pending_coinbase_update:
                v2::MinaBasePendingCoinbaseUpdateStableV1 {
                    action: _,
                    coinbase_amount,
                },
        } = self;

        blockchain_state.check(w);
        consensus_transition.check(w);
        coinbase_amount.check(w);
    }
}

impl ToFieldElements<Fp> for EpochLedger<Fp> {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self {
            hash,
            total_currency,
        } = self;
        hash.to_field_elements(fields);
        total_currency.to_field_elements(fields);
    }
}

impl ToFieldElements<Fp> for v2::MinaStateProtocolStateValueStableV2 {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self {
            previous_state_hash,
            body,
        } = self;

        previous_state_hash
            .to_field::<Fp>()
            .to_field_elements(fields);
        body.to_field_elements(fields);
    }
}

impl Check<Fp> for v2::MinaStateProtocolStateValueStableV2 {
    fn check(&self, w: &mut Witness<Fp>) {
        let Self {
            previous_state_hash: _,
            body,
        } = self;

        body.check(w);
    }
}

fn ledger_proof_opt(
    proof: Option<&v2::LedgerProofProdStableV2>,
    next_state: &v2::MinaStateProtocolStateValueStableV2,
) -> (Statement<SokDigest>, Arc<v2::TransactionSnarkProofStableV2>) {
    match proof {
        Some(proof) => {
            let statement: Statement<SokDigest> = (&proof.0.statement).into();
            let p: &v2::TransactionSnarkProofStableV2 = &proof.0.proof;
            // TODO: Don't clone the proof here
            (statement, Arc::new(p.clone()))
        }
        None => {
            let statement: Statement<()> =
                (&next_state.body.blockchain_state.ledger_proof_statement).into();
            let statement = statement.with_digest(SokDigest::default());
            let p = dummy::dummy_transaction_proof();
            (statement, p)
        }
    }
}

fn checked_hash_protocol_state(
    state: &v2::MinaStateProtocolStateValueStableV2,
    w: &mut Witness<Fp>,
) -> (Fp, Fp) {
    let v2::MinaStateProtocolStateValueStableV2 {
        previous_state_hash,
        body,
    } = state;

    let mut inputs = Inputs::new();
    body.to_inputs(&mut inputs);
    let body_hash = checked_hash("MinaProtoStateBody", &inputs.to_fields(), w);

    let mut inputs = Inputs::new();
    inputs.append_field(previous_state_hash.to_field());
    inputs.append_field(body_hash);
    let hash = checked_hash("MinaProtoState", &inputs.to_fields(), w);

    (hash, body_hash)
}

// Checked version
fn checked_hash_protocol_state2(state: &ProtocolState, w: &mut Witness<Fp>) -> (Fp, Fp) {
    let ProtocolState {
        previous_state_hash,
        body,
    } = state;

    let mut inputs = Inputs::new();
    body.to_inputs(&mut inputs);
    let body_hash = checked_hash("MinaProtoStateBody", &inputs.to_fields(), w);

    let mut inputs = Inputs::new();
    inputs.append_field(*previous_state_hash);
    inputs.append_field(body_hash);
    let hash = checked_hash("MinaProtoState", &inputs.to_fields(), w);

    (hash, body_hash)
}

fn non_pc_registers_equal_var(t1: &Registers, t2: &Registers, w: &mut Witness<Fp>) -> Boolean {
    let alls = [
        // t1.pending_coinbase_stack.equal_var(&t2.pending_coinbase_stack, w),
        field::equal(t1.first_pass_ledger, t2.first_pass_ledger, w),
        field::equal(t1.second_pass_ledger, t2.second_pass_ledger, w),
    ]
    .into_iter()
    .chain(t1.local_state.checked_equal_prime(&t2.local_state, w))
    .collect::<Vec<_>>();

    Boolean::all(&alls, w)
}

fn txn_statement_ledger_hashes_equal(
    s1: &Statement<()>,
    s2: &Statement<()>,
    w: &mut Witness<Fp>,
) -> Boolean {
    let source_eq = non_pc_registers_equal_var(&s1.source, &s2.source, w);
    let target_eq = non_pc_registers_equal_var(&s1.target, &s2.target, w);
    let left_ledger_eq = field::equal(s1.connecting_ledger_left, s2.connecting_ledger_left, w);
    let right_ledger_eq = field::equal(s1.connecting_ledger_right, s2.connecting_ledger_right, w);
    let supply_increase_eq = s1
        .supply_increase
        .to_checked()
        .equal(&s2.supply_increase.to_checked(), w);

    Boolean::all(
        &[
            source_eq,
            target_eq,
            left_ledger_eq,
            right_ledger_eq,
            supply_increase_eq,
        ],
        w,
    )
}

mod floating_point {
    use ark_ff::BigInteger256;
    use num_bigint::BigUint;

    use crate::{
        proofs::{
            to_field_elements::field_of_bits,
            witness::{field_to_bits2, FieldWitness},
        },
        scan_state::currency::{Amount, Balance, Sgn},
    };

    use super::*;

    pub enum CoeffIntegerPart {
        Zero,
        One,
    }

    const COEFFICIENTS: [(Sgn, BigInteger256); 11] = [
        (Sgn::Pos, BigInteger256::new([405058, 0, 0, 0])),
        (Sgn::Neg, BigInteger256::new([1007582, 0, 0, 0])),
        (Sgn::Pos, BigInteger256::new([465602, 0, 0, 0])),
        (Sgn::Neg, BigInteger256::new([161365, 0, 0, 0])),
        (Sgn::Pos, BigInteger256::new([44739, 0, 0, 0])),
        (Sgn::Neg, BigInteger256::new([10337, 0, 0, 0])),
        (Sgn::Pos, BigInteger256::new([2047, 0, 0, 0])),
        (Sgn::Neg, BigInteger256::new([354, 0, 0, 0])),
        (Sgn::Pos, BigInteger256::new([54, 0, 0, 0])),
        (Sgn::Neg, BigInteger256::new([7, 0, 0, 0])),
        (Sgn::Pos, BigInteger256::new([0, 0, 0, 0])),
    ];

    pub struct Params {
        pub total_precision: usize,
        pub per_term_precision: usize,
        pub terms_needed: usize,
        pub coefficients: [(Sgn, BigInteger256); 11],
        pub linear_term_integer_part: CoeffIntegerPart,
    }

    pub const PARAMS: Params = Params {
        total_precision: 16,
        per_term_precision: 20,
        terms_needed: 11,
        coefficients: COEFFICIENTS,
        linear_term_integer_part: CoeffIntegerPart::One,
    };

    #[derive(Clone)]
    pub enum Interval {
        Constant(BigUint),
        LessThan(BigUint),
    }

    fn bits_needed(v: BigUint) -> u64 {
        v.bits().checked_sub(1).unwrap()
    }

    impl Interval {
        fn scale(&self, x: BigUint) -> Self {
            match self {
                Interval::Constant(v) => Self::Constant(v * x),
                Interval::LessThan(v) => Self::LessThan(v * x),
            }
        }

        fn bits_needed(&self) -> u64 {
            match self {
                Interval::Constant(x) => bits_needed(x + BigUint::from(1u64)),
                Interval::LessThan(x) => bits_needed(x.clone()),
            }
        }

        fn quotient(&self, b: &Self) -> Self {
            use Interval::*;

            match (self, b) {
                (Constant(a), Constant(b)) => Constant(a / b),
                (LessThan(a), Constant(b)) => LessThan((a / b) + 1u64),
                (Constant(a), LessThan(_)) => LessThan(a + 1u64),
                (LessThan(a), LessThan(_)) => LessThan(a.clone()),
            }
        }
    }

    pub struct SnarkyInteger<F: FieldWitness> {
        pub value: F,
        pub interval: Interval,
        pub bits: Option<Box<[bool]>>,
    }

    impl<F: FieldWitness> SnarkyInteger<F> {
        pub fn create(value: F, upper_bound: BigUint) -> Self {
            Self {
                value,
                interval: Interval::LessThan(upper_bound),
                bits: None,
            }
        }

        fn to_field(&self) -> F {
            self.value
        }

        fn shift_left(&self, k: usize) -> Self {
            let Self {
                value,
                interval,
                bits,
            } = self;

            let two_to_k = BigUint::new(vec![1, 0, 0, 0]) << k;

            Self {
                value: *value * F::from(two_to_k.clone()),
                interval: interval.scale(two_to_k),
                bits: bits.as_ref().map(|_| todo!()),
            }
        }

        fn div_mod(&self, b: &Self, w: &mut Witness<F>) -> (Self, Self) {
            let (q, r) = w.exists({
                let a: BigUint = self.value.into();
                let b: BigUint = b.value.into();
                (F::from(&a / &b), F::from(&a % &b))
            });

            let q_bit_length = self.interval.bits_needed();
            let b_bit_length = b.interval.bits_needed();

            let q_bits = w.exists(field_to_bits2(q, q_bit_length as usize));
            let r_bits = w.exists(field_to_bits2(r, b_bit_length as usize));

            field::assert_lt(b_bit_length, r, b.value, w);

            (
                Self {
                    value: q,
                    interval: self.interval.quotient(&b.interval),
                    bits: Some(q_bits),
                },
                Self {
                    value: r,
                    interval: b.interval.clone(),
                    bits: Some(r_bits),
                },
            )
        }
    }

    pub fn balance_upper_bound() -> BigUint {
        BigUint::new(vec![1, 0, 0, 0]) << Balance::NBITS as u32
    }

    pub fn amount_upper_bound() -> BigUint {
        BigUint::new(vec![1, 0, 0, 0]) << Amount::NBITS as u32
    }

    #[derive(Clone, Debug)]
    pub struct Point<F: FieldWitness> {
        pub value: F,
        pub precision: usize,
    }

    pub fn of_quotient<F: FieldWitness>(
        precision: usize,
        top: SnarkyInteger<F>,
        bottom: SnarkyInteger<F>,
        w: &mut Witness<F>,
    ) -> Point<F> {
        let (q, _r) = top.shift_left(precision).div_mod(&bottom, w);
        Point {
            value: q.to_field(),
            precision,
        }
    }

    impl<F: FieldWitness> Point<F> {
        pub fn mul(&self, y: &Self, w: &mut Witness<F>) -> Self {
            let new_precision = self.precision + y.precision;
            Self {
                value: field::mul(self.value, y.value, w),
                precision: new_precision,
            }
        }

        pub fn const_mul(&self, y: &Self) -> Self {
            let new_precision = self.precision + y.precision;
            Self {
                value: self.value * y.value,
                precision: new_precision,
            }
        }

        pub fn powers(&self, n: usize, w: &mut Witness<F>) -> Vec<Self> {
            let mut res = vec![self.clone(); n];
            for i in 1..n {
                res[i] = self.mul(&res[i - 1], w);
            }
            res
        }

        pub fn constant(value: &BigInteger256, precision: usize) -> Self {
            Self {
                value: (*value).into(),
                precision,
            }
        }

        pub fn add_signed(&self, (sgn, t2): (Sgn, &Self)) -> Self {
            let precision = self.precision.max(t2.precision);
            let (t1, t2) = if self.precision < t2.precision {
                (self, t2)
            } else {
                (t2, self)
            };

            let powed2 = (0..(t2.precision - t1.precision)).fold(F::one(), |acc, _| acc.double());

            let value = match sgn {
                Sgn::Pos => (powed2 * t1.value) + t2.value,
                Sgn::Neg => (powed2 * t1.value) - t2.value,
            };

            Self { value, precision }
        }

        pub fn add(&self, t2: &Self) -> Self {
            self.add_signed((Sgn::Pos, t2))
        }

        pub fn of_bits<const N: usize>(bits: &[bool; N], precision: usize) -> Self {
            Self {
                value: field_of_bits(bits),
                precision,
            }
        }

        pub fn le(&self, t2: &Self, w: &mut Witness<F>) -> Boolean {
            let precision = self.precision.max(t2.precision);

            let padding = {
                let k = precision - self.precision.min(t2.precision);
                (0..k).fold(F::one(), |acc, _| acc.double())
            };

            let (x1, x2) = {
                let (x1, x2) = (self.value, t2.value);
                if self.precision < t2.precision {
                    (padding * x1, x2)
                } else if t2.precision < self.precision {
                    (x1, padding * x2)
                } else {
                    (x1, x2)
                }
            };

            field::compare(precision as u64, x1, x2, w).1
        }
    }
}

mod snarky_taylor {
    use crate::{proofs::witness::FieldWitness, scan_state::currency::Sgn};

    use super::*;
    use floating_point::*;

    fn taylor_sum<F: FieldWitness>(
        x_powers: Vec<Point<F>>,
        coefficients: impl Iterator<Item = (Sgn, floating_point::Point<F>)>,
        linear_term_integer_part: &CoeffIntegerPart,
    ) -> Point<F> {
        let acc = coefficients
            .zip(&x_powers)
            .fold(None, |sum, ((sgn, ci), xi)| {
                let term = ci.const_mul(xi);
                match sum {
                    None => Some(term),
                    Some(s) => Some(s.add_signed((sgn, &term))),
                }
            })
            .unwrap();

        match linear_term_integer_part {
            CoeffIntegerPart::Zero => acc,
            CoeffIntegerPart::One => acc.add(&x_powers[0]),
        }
    }

    pub fn one_minus_exp<F: FieldWitness>(
        params: &Params,
        x: Point<F>,
        w: &mut Witness<F>,
    ) -> Point<F> {
        let floating_point::Params {
            total_precision: _,
            per_term_precision,
            terms_needed,
            coefficients,
            linear_term_integer_part,
        } = params;

        let powers = x.powers(*terms_needed, w);
        let coefficients = coefficients
            .iter()
            .map(|(sgn, c)| (*sgn, Point::<F>::constant(c, *per_term_precision)));
        taylor_sum(powers, coefficients, linear_term_integer_part)
    }
}

mod vrf {
    use std::ops::Neg;

    use mina_signer::{CompressedPubKey, PubKey};

    use crate::{
        checked_verify_merkle_path,
        proofs::{
            numbers::nat::{CheckedNat, CheckedSlot},
            witness::{
                decompress_var, field_to_bits, legacy_input::to_bits, scale_known,
                scale_non_constant, GroupAffine, InnerCurve,
            },
        },
        scan_state::{
            currency::{Amount, Balance},
            transaction_logic::protocol_state::EpochLedger,
        },
        sparse_ledger::SparseLedger,
        AccountIndex, Address,
    };

    use super::*;

    struct Message<'a> {
        global_slot: &'a CheckedSlot<Fp>,
        seed: Fp,
        delegator: AccountIndex,
        delegator_bits: [bool; 35],
    }

    impl<'a> ToInputs for Message<'a> {
        fn to_inputs(&self, inputs: &mut Inputs) {
            let Self {
                global_slot,
                seed,
                delegator: _,
                delegator_bits,
            } = self;

            inputs.append(seed);
            inputs.append_u32(global_slot.to_inner().as_u32());
            for b in delegator_bits {
                inputs.append_bool(*b)
            }
        }
    }

    fn hash_to_group(m: &Message, w: &mut Witness<Fp>) -> GroupAffine<Fp> {
        let inputs = m.to_inputs_owned().to_fields();
        let hash = checked_hash("MinaVrfMessage", &inputs, w);
        crate::proofs::group_map::to_group(hash, w)
    }

    fn scale_generator(
        s: &[bool; 255],
        init: &InnerCurve<Fp>,
        w: &mut Witness<Fp>,
    ) -> GroupAffine<Fp> {
        scale_known(InnerCurve::<Fp>::one().to_affine(), s, init, w)
    }

    fn eval(
        m: &InnerCurve<Fp>,
        private_key: &[bool; 255],
        message: &Message,
        w: &mut Witness<Fp>,
    ) -> Fp {
        let h = hash_to_group(message, w);

        let u = {
            let _ = w.exists_no_check(h);
            let u = scale_non_constant(h, private_key, m, w);

            // unshift_nonzero
            let neg = m.to_affine().neg();
            w.exists(neg.y);
            w.add_fast(neg, u)
        };

        let GroupAffine::<Fp> { x, y, .. } = u;

        let mut inputs = message.to_inputs_owned();
        inputs.append_field(x);
        inputs.append_field(y);

        checked_hash("MinaVrfOutput", &inputs.to_fields(), w)
    }

    fn eval_and_check_public_key(
        m: &InnerCurve<Fp>,
        private_key: &[bool; 255],
        public_key: &PubKey,
        message: Message,
        w: &mut Witness<Fp>,
    ) -> Fp {
        let _ = {
            let _public_key_shifted = w.add_fast(m.to_affine(), *public_key.point());
            scale_generator(private_key, m, w)
        };

        eval(m, private_key, &message, w)
    }

    fn get_vrf_evaluation(
        m: &InnerCurve<Fp>,
        message: Message,
        prover_state: &v2::ConsensusStakeProofStableV2,
        w: &mut Witness<Fp>,
    ) -> (Fp, Box<crate::Account>) {
        let private_key = prover_state.producer_private_key.to_field::<Fq>();
        let private_key = w.exists(field_to_bits::<Fq, 255>(private_key));

        let account = {
            let mut ledger: SparseLedger = (&prover_state.ledger).into();

            let staker_addr = message.delegator.clone();
            let staker_addr =
                Address::from_index(staker_addr, CONSTRAINT_CONSTANTS.ledger_depth as usize);

            let account = ledger.get_exn(&staker_addr);
            let path = ledger.path_exn(staker_addr.clone());

            let (account, path) = w.exists((account, path));
            checked_verify_merkle_path(&account, &path, w);

            account
        };

        let delegate = {
            let delegate = account.delegate.as_ref().unwrap();
            decompress_var(delegate, w)
        };

        let evaluation = eval_and_check_public_key(m, &private_key, &delegate, message, w);

        (evaluation, account)
    }

    fn is_satisfied(
        my_stake: Balance,
        total_stake: Amount,
        vrf_output: &[bool; VRF_OUTPUT_NBITS],
        w: &mut Witness<Fp>,
    ) -> Boolean {
        use floating_point::*;

        let top = SnarkyInteger::create(my_stake.to_field::<Fp>(), balance_upper_bound());
        let bottom = SnarkyInteger::create(total_stake.to_field::<Fp>(), amount_upper_bound());
        let precision = PARAMS.per_term_precision;

        let point = floating_point::of_quotient(precision, top, bottom, w);
        let rhs = snarky_taylor::one_minus_exp(&PARAMS, point, w);

        let lhs = vrf_output;
        Point::of_bits(lhs, VRF_OUTPUT_NBITS).le(&rhs, w)
    }

    pub const VRF_OUTPUT_NBITS: usize = 253;

    fn truncate_vrf_output(output: Fp, w: &mut Witness<Fp>) -> Box<[bool; VRF_OUTPUT_NBITS]> {
        let output = w.exists(field_to_bits::<_, 255>(output));
        Box::new(std::array::from_fn(|i| output[i]))
    }

    pub fn check(
        m: &InnerCurve<Fp>,
        epoch_ledger: &EpochLedger<Fp>,
        global_slot: &CheckedSlot<Fp>,
        _block_stake_winner: &CompressedPubKey,
        _block_creator: &CompressedPubKey,
        seed: Fp,
        prover_state: &v2::ConsensusStakeProofStableV2,
        w: &mut Witness<Fp>,
    ) -> (
        Boolean,
        Fp,
        Box<[bool; VRF_OUTPUT_NBITS]>,
        Box<crate::Account>,
    ) {
        let (winner_addr, winner_addr_bits) = {
            const LEDGER_DEPTH: usize = 35;
            assert_eq!(CONSTRAINT_CONSTANTS.ledger_depth, LEDGER_DEPTH as u64);

            let account_index = prover_state.delegator.as_u64();
            let bits = w.exists(to_bits::<_, LEDGER_DEPTH>(account_index));
            (AccountIndex(account_index), bits)
        };

        let (result, winner_account) = get_vrf_evaluation(
            m,
            Message {
                global_slot,
                seed,
                delegator: winner_addr,
                delegator_bits: winner_addr_bits,
            },
            prover_state,
            w,
        );

        let my_stake = winner_account.balance;
        let truncated_result = truncate_vrf_output(result, w);
        let satisifed = is_satisfied(my_stake, epoch_ledger.total_currency, &truncated_result, w);

        (satisifed, result, truncated_result, winner_account)
    }
}

pub mod consensus {
    use ark_ff::Zero;
    use mina_signer::CompressedPubKey;

    use super::{vrf::VRF_OUTPUT_NBITS, *};
    use crate::{
        decompress_pk,
        proofs::{
            numbers::{
                currency::CheckedCurrency,
                nat::{CheckedN, CheckedN32, CheckedNat, CheckedSlot, CheckedSlotSpan},
            },
            witness::{compress_var, create_shifted_inner_curve, CompressedPubKeyVar},
        },
        scan_state::{
            currency::{Amount, Length},
            transaction_logic::protocol_state::{EpochData, EpochLedger},
        },
        Account,
    };

    pub struct ConsensusConstantsChecked {
        pub k: CheckedLength<Fp>,
        pub delta: CheckedLength<Fp>,
        pub block_window_duration_ms: CheckedBlockTimeSpan<Fp>,
        pub slots_per_sub_window: CheckedLength<Fp>,
        pub slots_per_window: CheckedLength<Fp>,
        pub sub_windows_per_window: CheckedLength<Fp>,
        pub slots_per_epoch: CheckedLength<Fp>,
        pub grace_period_end: CheckedLength<Fp>,
        pub slot_duration_ms: CheckedBlockTimeSpan<Fp>,
        pub epoch_duration: CheckedBlockTimeSpan<Fp>,
        pub checkpoint_window_slots_per_year: CheckedLength<Fp>,
        pub checkpoint_window_size_in_slots: CheckedLength<Fp>,
        pub delta_duration: CheckedBlockTimeSpan<Fp>,
        pub genesis_state_timestamp: CheckedBlockTime<Fp>,
    }

    /// Number of millisecond per day
    const N_MILLIS_PER_DAY: u64 = 86400000;

    fn create_constant_prime(
        prev_state: &v2::MinaStateProtocolStateValueStableV2,
        w: &mut Witness<Fp>,
    ) -> ConsensusConstantsChecked {
        let protocol_constants = &prev_state.body.constants;
        let constraint_constants = &CONSTRAINT_CONSTANTS;

        let of_u64 = |n: u64| CheckedN::<Fp>::from_field(n.into());
        let of_u32 = |n: u32| CheckedN::<Fp>::from_field(n.into());

        let block_window_duration_ms = of_u64(constraint_constants.block_window_duration_ms);

        let k = of_u32(protocol_constants.k.as_u32());
        let delta = of_u32(protocol_constants.delta.as_u32());
        let slots_per_sub_window = of_u32(protocol_constants.slots_per_sub_window.as_u32());
        let sub_windows_per_window = of_u64(constraint_constants.sub_windows_per_window);

        let slots_per_window = slots_per_sub_window.const_mul(&sub_windows_per_window, w);

        let slots_per_epoch = of_u32(protocol_constants.slots_per_epoch.as_u32());

        let slot_duration_ms = block_window_duration_ms.clone();

        let epoch_duration = {
            let size = &slots_per_epoch;
            slot_duration_ms.const_mul(size, w)
        };

        let delta_duration = {
            let delta_plus_one = delta.add(&CheckedN::one(), w);
            slot_duration_ms.const_mul(&delta_plus_one, w)
        };

        let num_days: u64 = 3;
        assert!(num_days < 14);

        let grace_period_end = {
            let slots = {
                let n_days = {
                    let n_days_ms = of_u64(num_days * N_MILLIS_PER_DAY);
                    dbg!(&n_days_ms);
                    n_days_ms.div_mod(&block_window_duration_ms, w).0
                };
                n_days.min(&slots_per_epoch, w)
            };
            match constraint_constants.fork.as_ref() {
                None => slots,
                Some(f) => {
                    let previous_global_slot = of_u32(f.previous_global_slot.as_u32());
                    previous_global_slot.add(&slots, w)
                }
            }
        };

        let to_length = |v: CheckedN<Fp>| CheckedLength::from_field(v.to_field());
        let to_timespan = |v: CheckedN<Fp>| CheckedBlockTimeSpan::from_field(v.to_field());

        ConsensusConstantsChecked {
            k: to_length(k),
            delta: to_length(delta),
            block_window_duration_ms: to_timespan(block_window_duration_ms),
            slots_per_sub_window: to_length(slots_per_sub_window),
            slots_per_window: to_length(slots_per_window),
            sub_windows_per_window: to_length(sub_windows_per_window),
            slots_per_epoch: to_length(slots_per_epoch),
            grace_period_end: to_length(grace_period_end),
            slot_duration_ms: to_timespan(slot_duration_ms),
            epoch_duration: to_timespan(epoch_duration),
            checkpoint_window_slots_per_year: CheckedLength::zero(),
            checkpoint_window_size_in_slots: CheckedLength::zero(),
            delta_duration: to_timespan(delta_duration),
            genesis_state_timestamp: {
                let v = of_u64(protocol_constants.genesis_state_timestamp.as_u64());
                CheckedBlockTime::from_field(v.to_field())
            },
        }
    }

    fn create_constant(
        prev_state: &v2::MinaStateProtocolStateValueStableV2,
        w: &mut Witness<Fp>,
    ) -> ConsensusConstantsChecked {
        let mut constants = create_constant_prime(prev_state, w);

        let (checkpoint_window_slots_per_year, checkpoint_window_size_in_slots) = {
            let of_u64 = |n: u64| CheckedN::<Fp>::from_field(n.into());
            let of_field = |f: Fp| CheckedN::<Fp>::from_field(f);

            let per_year = of_u64(12);
            let slot_duration_ms = of_field(constants.slot_duration_ms.to_field());

            let (slots_per_year, _) = {
                let one_year_ms = of_u64(365 * N_MILLIS_PER_DAY);
                one_year_ms.div_mod(&slot_duration_ms, w)
            };

            let size_in_slots = {
                let (size_in_slots, _rem) = slots_per_year.div_mod(&per_year, w);
                size_in_slots
            };

            let to_length = |v: CheckedN<Fp>| CheckedLength::from_field(v.to_field());
            (to_length(slots_per_year), to_length(size_in_slots))
        };

        constants.checkpoint_window_slots_per_year = checkpoint_window_slots_per_year;
        constants.checkpoint_window_size_in_slots = checkpoint_window_size_in_slots;

        constants
    }

    type CheckedEpoch = CheckedBlockTime<Fp>;

    #[derive(Clone, Debug)]
    pub struct GlobalSlot {
        pub slot_number: CheckedSlot<Fp>,
        pub slots_per_epoch: CheckedLength<Fp>,
    }

    impl From<&v2::ConsensusGlobalSlotStableV1> for GlobalSlot {
        fn from(value: &v2::ConsensusGlobalSlotStableV1) -> Self {
            let v2::ConsensusGlobalSlotStableV1 {
                slot_number,
                slots_per_epoch,
            } = value;

            Self {
                slot_number: CheckedSlot::from_field(slot_number.as_u32().into()),
                slots_per_epoch: CheckedLength::from_field(slots_per_epoch.as_u32().into()),
            }
        }
    }

    impl GlobalSlot {
        fn of_slot_number(
            constants: &ConsensusConstantsChecked,
            slot_number: CheckedSlot<Fp>,
        ) -> Self {
            Self {
                slot_number,
                slots_per_epoch: constants.slots_per_epoch.clone(),
            }
        }

        fn diff_slots(&self, other: &Self, w: &mut Witness<Fp>) -> CheckedSlotSpan<Fp> {
            self.slot_number.diff(&other.slot_number, w)
        }

        fn less_than(&self, other: &Self, w: &mut Witness<Fp>) -> Boolean {
            self.slot_number.less_than(&other.slot_number, w)
        }

        fn to_epoch_and_slot(&self, w: &mut Witness<Fp>) -> (CheckedEpoch, CheckedSlot<Fp>) {
            let (epoch, slot) = self
                .slot_number
                .div_mod(&CheckedSlot::from_field(self.slots_per_epoch.to_field()), w);

            (CheckedEpoch::from_field(epoch.to_field()), slot)
        }
    }

    fn compute_supercharge_coinbase(
        winner_account: &Account,
        global_slot: &CheckedSlot<Fp>,
        w: &mut Witness<Fp>,
    ) -> Boolean {
        let winner_locked = winner_account.has_locked_tokens_checked(global_slot, w);
        winner_locked.neg()
    }

    fn same_checkpoint_window(
        constants: &ConsensusConstantsChecked,
        prev: &GlobalSlot,
        next: &GlobalSlot,
        w: &mut Witness<Fp>,
    ) -> Boolean {
        let slot1 = prev;
        let slot2 = next;

        let slot1 = &slot1.slot_number;
        let checkpoint_window_size_in_slots = &constants.checkpoint_window_size_in_slots;

        let (_q1, r1) = slot1.div_mod(&checkpoint_window_size_in_slots.to_slot(), w);

        let next_window_start =
            { slot1.to_field() - r1.to_field() + checkpoint_window_size_in_slots.to_field() };

        slot2
            .slot_number
            .less_than(&CheckedSlot::from_field(next_window_start), w)
    }

    struct GlobalSubWindow {
        inner: CheckedN32<Fp>,
    }

    impl std::ops::Deref for GlobalSubWindow {
        type Target = CheckedN32<Fp>;

        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }

    impl GlobalSubWindow {
        fn of_global_slot(
            constants: &ConsensusConstantsChecked,
            s: &GlobalSlot,
            w: &mut Witness<Fp>,
        ) -> Self {
            let slot_as_field = s.slot_number.to_field();
            let slot_as_field = CheckedN32::from_field(slot_as_field);

            let (q, _) = slot_as_field.div_mod(
                &CheckedN32::from_field(constants.slots_per_window.to_field()),
                w,
            );
            Self { inner: q }
        }

        fn sub_window(&self, constants: &ConsensusConstantsChecked, w: &mut Witness<Fp>) -> Self {
            let (_, shift) = self.inner.div_mod(
                &CheckedN32::from_field(constants.sub_windows_per_window.to_field()),
                w,
            );
            Self { inner: shift }
        }

        fn equal(&self, other: &Self, w: &mut Witness<Fp>) -> Boolean {
            self.inner.equal(&other.inner, w)
        }

        fn add(&self, b: &CheckedLength<Fp>, w: &mut Witness<Fp>) -> Self {
            let inner = self.inner.add(&CheckedN32::from_field(b.to_field()), w);
            Self { inner }
        }

        fn gte(&self, other: &Self, w: &mut Witness<Fp>) -> Boolean {
            self.inner.gte(&other.inner, w)
        }
    }

    type SubWindow = CheckedN32<Fp>;

    struct UpdateMinWindowDensityParams<'a> {
        constants: &'a ConsensusConstantsChecked,
        prev_global_slot: &'a GlobalSlot,
        next_global_slot: &'a GlobalSlot,
        prev_sub_window_densities: Vec<u32>,
        prev_min_window_density: u32,
    }

    fn update_min_window_density(
        params: UpdateMinWindowDensityParams,
        w: &mut Witness<Fp>,
    ) -> (CheckedLength<Fp>, Vec<CheckedLength<Fp>>) {
        let UpdateMinWindowDensityParams {
            constants: c,
            prev_global_slot,
            next_global_slot,
            prev_sub_window_densities,
            prev_min_window_density,
        } = params;

        let prev_global_sub_window = GlobalSubWindow::of_global_slot(c, prev_global_slot, w);
        let next_global_sub_window = GlobalSubWindow::of_global_slot(c, next_global_slot, w);

        let prev_relative_sub_window = prev_global_sub_window.sub_window(c, w);
        let next_relative_sub_window = next_global_sub_window.sub_window(c, w);

        let same_sub_window = prev_global_sub_window.equal(&next_global_sub_window, w);
        let overlapping_window = {
            let x = prev_global_sub_window.add(&c.sub_windows_per_window, w);
            x.gte(&next_global_sub_window, w)
        };

        let current_sub_window_densities = prev_sub_window_densities
            .iter()
            .enumerate()
            .map(|(i, density)| {
                let gt_prev_sub_window =
                    SubWindow::constant(i).greater_than(&prev_relative_sub_window, w);

                // :(
                // This will be removed once we have cvar
                let lt_next_sub_window = if i == 0 {
                    SubWindow::constant(i).const_less_than(&next_relative_sub_window, w)
                } else {
                    SubWindow::constant(i).less_than(&next_relative_sub_window, w)
                };

                let within_range = {
                    let cond = prev_relative_sub_window.less_than(&next_relative_sub_window, w);
                    let on_true = gt_prev_sub_window.and(&lt_next_sub_window, w);
                    let on_false = gt_prev_sub_window.or(&lt_next_sub_window, w);

                    w.exists_no_check(match cond {
                        Boolean::True => on_true,
                        Boolean::False => on_false,
                    })
                };

                let density = Length::from_u32(*density).to_checked::<Fp>();

                let on_true = density.clone();
                let on_false = {
                    let cond = overlapping_window.and(&within_range.neg(), w);
                    w.exists_no_check(match cond {
                        Boolean::True => density,
                        Boolean::False => CheckedLength::zero(),
                    })
                };

                w.exists_no_check(match same_sub_window {
                    Boolean::True => on_true,
                    Boolean::False => on_false,
                })
            })
            .collect::<Vec<_>>();

        let current_window_density = current_sub_window_densities.iter().enumerate().fold(
            CheckedLength::zero(),
            |acc, (i, v)| {
                // :(
                // This will be removed once we have cvar
                if i == 0 {
                    acc.const_add(v, w)
                } else {
                    acc.add(v, w)
                }
            },
        );

        let min_window_density = {
            let in_grace_period = next_global_slot.less_than(
                &GlobalSlot::of_slot_number(c, c.grace_period_end.to_slot()),
                w,
            );
            let prev_min_window_density = Length::from_u32(prev_min_window_density).to_checked();

            let cond = same_sub_window.or(&in_grace_period, w);
            let on_true = prev_min_window_density.clone();
            let on_false = current_window_density.min(&prev_min_window_density, w);

            w.exists_no_check(match cond {
                Boolean::True => on_true,
                Boolean::False => on_false,
            })
        };

        let next_sub_window_densities = current_sub_window_densities
            .iter()
            .enumerate()
            .map(|(i, density)| {
                let is_next_sub_window = SubWindow::constant(i).equal(&next_relative_sub_window, w);

                let on_true = {
                    w.exists_no_check(match same_sub_window {
                        Boolean::True => density.const_succ(),
                        Boolean::False => CheckedLength::zero().const_succ(),
                    })
                };

                w.exists_no_check(match is_next_sub_window {
                    Boolean::True => on_true,
                    Boolean::False => density.clone(),
                })
            })
            .collect::<Vec<_>>();

        (min_window_density, next_sub_window_densities)
    }

    #[derive(Clone)]
    pub struct ConsensusState {
        pub blockchain_length: CheckedLength<Fp>,
        pub epoch_count: CheckedLength<Fp>,
        pub min_window_density: CheckedLength<Fp>,
        pub sub_window_densities: Vec<CheckedLength<Fp>>,
        pub last_vrf_output: Box<[bool; VRF_OUTPUT_NBITS]>,
        pub curr_global_slot: GlobalSlot,
        pub global_slot_since_genesis: CheckedSlot<Fp>,
        pub total_currency: CheckedAmount<Fp>,
        pub staking_epoch_data: EpochData<Fp>,
        pub next_epoch_data: EpochData<Fp>,
        pub has_ancestor_in_same_checkpoint_window: Boolean,
        pub block_stake_winner: CompressedPubKey,
        pub block_creator: CompressedPubKey,
        pub coinbase_receiver: CompressedPubKey,
        pub supercharge_coinbase: Boolean,
    }

    pub fn next_state_checked(
        prev_state: &v2::MinaStateProtocolStateValueStableV2,
        prev_state_hash: Fp,
        transition: &v2::MinaStateSnarkTransitionValueStableV2,
        supply_increase: CheckedSigned<Fp, CheckedAmount<Fp>>,
        prover_state: &v2::ConsensusStakeProofStableV2,
        w: &mut Witness<Fp>,
    ) -> (Boolean, ConsensusState) {
        let previous_blockchain_state_ledger_hash = prev_state
            .body
            .blockchain_state
            .ledger_proof_statement
            .target
            .first_pass_ledger
            .to_field::<Fp>();
        let genesis_ledger_hash = prev_state
            .body
            .blockchain_state
            .genesis_ledger_hash
            .to_field::<Fp>();
        let consensus_transition =
            CheckedSlot::<Fp>::from_field(transition.consensus_transition.as_u32().into());
        let previous_state = &prev_state.body.consensus_state;
        // transiti

        let constants = create_constant(prev_state, w);

        let v2::ConsensusProofOfStakeDataConsensusStateValueStableV2 {
            curr_global_slot: prev_global_slot,
            ..
        } = &prev_state.body.consensus_state;
        let prev_global_slot: GlobalSlot = prev_global_slot.into();

        let next_global_slot = GlobalSlot::of_slot_number(&constants, consensus_transition);

        let slot_diff = next_global_slot.diff_slots(&prev_global_slot, w);

        let _ = {
            let global_slot_increased = prev_global_slot.less_than(&next_global_slot, w);
            let is_genesis = field::equal(
                CheckedSlot::zero().to_field(),
                next_global_slot.slot_number.to_field(),
                w,
            );

            Boolean::assert_any(&[global_slot_increased, is_genesis], w)
        };

        let (next_epoch, next_slot) = next_global_slot.to_epoch_and_slot(w);
        let (prev_epoch, _prev_slot) = prev_global_slot.to_epoch_and_slot(w);

        let global_slot_since_genesis =
            CheckedSlot::<Fp>::from_field(previous_state.global_slot_since_genesis.as_u32().into())
                .add(&CheckedSlot::<Fp>::from_field(slot_diff.to_field()), w);

        let epoch_increased = prev_epoch.less_than(&next_epoch, w);

        let staking_epoch_data = {
            let next_epoch_data: EpochData<Fp> = (&previous_state.next_epoch_data).into();
            let staking_epoch_data: EpochData<Fp> = (&previous_state.staking_epoch_data).into();

            w.exists_no_check(match epoch_increased {
                Boolean::True => next_epoch_data,
                Boolean::False => staking_epoch_data,
            })
        };

        let next_slot_number = next_global_slot.slot_number.clone();

        let block_stake_winner = {
            let delegator_pk: CompressedPubKey = (&prover_state.delegator_pk).into();
            w.exists(delegator_pk)
        };

        let block_creator = {
            // TODO: See why `prover_state.producer_public_key` is compressed
            //       In OCaml it's uncompressed
            let producer_public_key: CompressedPubKey = (&prover_state.producer_public_key).into();
            let pk = decompress_pk(&producer_public_key).unwrap();
            w.exists_no_check(&pk);
            // TODO: Remove this
            let CompressedPubKeyVar { x, is_odd } = compress_var(pk.point(), w);
            CompressedPubKey { x, is_odd }
        };

        let coinbase_receiver = {
            let pk: CompressedPubKey = (&prover_state.coinbase_receiver_pk).into();
            w.exists(pk)
        };

        let (threshold_satisfied, vrf_result, truncated_vrf_result, winner_account) = {
            let m = create_shifted_inner_curve::<Fp>(w);

            vrf::check(
                &m,
                &staking_epoch_data.ledger,
                &next_slot_number,
                &block_stake_winner,
                &block_creator,
                staking_epoch_data.seed,
                prover_state,
                w,
            )
        };

        let supercharge_coinbase =
            compute_supercharge_coinbase(&winner_account, &global_slot_since_genesis, w);

        let (new_total_currency, _overflow) = {
            let total_currency: Amount = previous_state.total_currency.clone().into();
            w.exists(supply_increase.force_value());
            total_currency
                .to_checked()
                .add_signed_flagged(supply_increase, w)
        };

        let has_ancestor_in_same_checkpoint_window =
            same_checkpoint_window(&constants, &prev_global_slot, &next_global_slot, w);

        let in_seed_update_range = next_slot.in_seed_update_range(&constants, w);

        let update_next_epoch_ledger = {
            let snarked_ledger_is_still_genesis = field::equal(
                genesis_ledger_hash,
                previous_blockchain_state_ledger_hash,
                w,
            );
            epoch_increased.and(&snarked_ledger_is_still_genesis.neg(), w)
        };

        fn epoch_seed_update_var(seed: Fp, vrf_result: Fp, w: &mut Witness<Fp>) -> Fp {
            checked_hash("MinaEpochSeed", &[seed, vrf_result], w)
        }

        let next_epoch_data = {
            let seed = {
                let base: Fp = previous_state.next_epoch_data.seed.to_field::<Fp>();
                let updated = epoch_seed_update_var(base, vrf_result, w);
                w.exists_no_check(match in_seed_update_range {
                    Boolean::True => updated,
                    Boolean::False => base,
                })
            };

            let epoch_length = {
                let base = w.exists_no_check(match epoch_increased {
                    Boolean::True => CheckedLength::zero(),
                    Boolean::False => {
                        let length: Length = (&previous_state.next_epoch_data.epoch_length).into();
                        length.to_checked()
                    }
                });
                base.const_succ()
            };

            let ledger = w.exists_no_check(match update_next_epoch_ledger {
                Boolean::True => EpochLedger {
                    hash: previous_blockchain_state_ledger_hash,
                    total_currency: new_total_currency.to_inner(), // TODO: Might overflow ?
                },
                Boolean::False => {
                    let ledger = &previous_state.next_epoch_data.ledger;
                    EpochLedger {
                        hash: ledger.hash.to_field(),
                        total_currency: ledger.total_currency.clone().into(),
                    }
                }
            });

            let start_checkpoint = w.exists_no_check(match epoch_increased {
                Boolean::True => prev_state_hash,
                Boolean::False => previous_state.next_epoch_data.start_checkpoint.to_field(),
            });

            let lock_checkpoint = {
                let base = w.exists_no_check(match epoch_increased {
                    Boolean::True => Fp::zero(),
                    Boolean::False => previous_state.next_epoch_data.lock_checkpoint.to_field(),
                });
                w.exists_no_check(match in_seed_update_range {
                    Boolean::True => prev_state_hash,
                    Boolean::False => base,
                })
            };

            EpochData {
                ledger,
                seed,
                start_checkpoint,
                lock_checkpoint,
                epoch_length: epoch_length.to_inner(), // TODO: Overflow ?
            }
        };

        let blockchain_length = {
            let blockchain_length: Length = (&previous_state.blockchain_length).into();
            blockchain_length.to_checked::<Fp>().const_succ()
        };

        let epoch_count = {
            let epoch_count: Length = (&previous_state.epoch_count).into();
            match epoch_increased {
                Boolean::True => epoch_count.to_checked::<Fp>().const_succ(),
                Boolean::False => epoch_count.to_checked(),
            }
        };

        let (min_window_density, sub_window_densities) = update_min_window_density(
            UpdateMinWindowDensityParams {
                constants: &constants,
                prev_global_slot: &prev_global_slot,
                next_global_slot: &next_global_slot,
                prev_sub_window_densities: previous_state
                    .sub_window_densities
                    .iter()
                    .map(|v| v.as_u32())
                    .collect(),
                prev_min_window_density: previous_state.min_window_density.as_u32(),
            },
            w,
        );

        let state = ConsensusState {
            blockchain_length,
            epoch_count,
            min_window_density,
            sub_window_densities,
            last_vrf_output: truncated_vrf_result,
            curr_global_slot: next_global_slot,
            global_slot_since_genesis,
            total_currency: new_total_currency,
            staking_epoch_data,
            next_epoch_data,
            has_ancestor_in_same_checkpoint_window,
            block_stake_winner,
            block_creator,
            coinbase_receiver,
            supercharge_coinbase,
        };

        (threshold_satisfied, state)
    }
}

fn is_genesis_state_var(
    cs: &v2::ConsensusProofOfStakeDataConsensusStateValueStableV2,
    w: &mut Witness<Fp>,
) -> Boolean {
    use crate::scan_state::currency::Slot;

    let curr_global_slot = &cs.curr_global_slot;
    let slot_number = Slot::from_u32(curr_global_slot.slot_number.as_u32()).to_checked::<Fp>();

    CheckedSlot::zero().equal(&slot_number, w)
}

fn is_genesis_state_var2(cs: &ConsensusState, w: &mut Witness<Fp>) -> Boolean {
    let curr_global_slot = &cs.curr_global_slot;
    let slot_number = &curr_global_slot.slot_number;

    CheckedSlot::zero().equal(slot_number, w)
}

fn genesis_state_hash_checked(
    state_hash: Fp,
    state: &v2::MinaStateProtocolStateValueStableV2,
    w: &mut Witness<Fp>,
) -> Fp {
    let is_genesis = is_genesis_state_var(&state.body.consensus_state, w);

    w.exists_no_check(match is_genesis {
        Boolean::True => state_hash,
        Boolean::False => state.body.genesis_state_hash.to_field(),
    })
}

pub struct ProtocolStateBody {
    pub genesis_state_hash: Fp,
    pub blockchain_state: v2::MinaStateBlockchainStateValueStableV2,
    pub consensus_state: ConsensusState,
    pub constants: v2::MinaBaseProtocolConstantsCheckedValueStableV1,
}

pub struct ProtocolState {
    pub previous_state_hash: Fp,
    pub body: ProtocolStateBody,
}

fn protocol_create_var(
    previous_state_hash: Fp,
    genesis_state_hash: Fp,
    blockchain_state: &v2::MinaStateBlockchainStateValueStableV2,
    consensus_state: &ConsensusState,
    constants: &v2::MinaBaseProtocolConstantsCheckedValueStableV1,
) -> ProtocolState {
    ProtocolState {
        previous_state_hash,
        body: ProtocolStateBody {
            genesis_state_hash,
            blockchain_state: blockchain_state.clone(),
            consensus_state: consensus_state.clone(),
            constants: constants.clone(),
        },
    }
}

/// `N_PREVIOUS`: Number of previous proofs.
///  - For block and merge proofs, the number is 2
///  - For zkapp with proof authorization, it's 1
///  - For other proofs, it's 0
pub struct StepParams<'a, const N_PREVIOUS: usize> {
    pub app_state: Rc<dyn ToFieldElementsDebug>,
    pub rule: InductiveRule<'a, N_PREVIOUS>,
    pub for_step_datas: [&'a ForStep; N_PREVIOUS],
    pub indexes: [(
        &'a VerifierIndex<GroupAffine<Fp>>,
        &'a CircuitPlonkVerificationKeyEvals<Fp>,
    ); N_PREVIOUS],
    pub prev_challenge_polynomial_commitments: Vec<RecursionChallenge<GroupAffine<Fq>>>,
    /// TODO: Remove this. See documentation in `PerProofWitness` struct.
    pub hack_feature_flags: OptFlag,
    pub step_prover: &'a Prover<Fp>,
    pub wrap_prover: &'a Prover<Fq>,
}

pub fn step<C: ProofConstants, const N_PREVIOUS: usize>(
    params: StepParams<N_PREVIOUS>,
    w: &mut Witness<Fp>,
) -> (
    StepStatement,
    Vec<AllEvals<Fq>>,
    kimchi::proof::ProverProof<GroupAffine<Fq>>,
) {
    let StepParams {
        app_state,
        rule,
        for_step_datas,
        indexes,
        prev_challenge_polynomial_commitments,
        hack_feature_flags,
        step_prover,
        wrap_prover,
    } = params;

    let dlog_plonk_index = w.exists(super::merge::dlog_plonk_index(wrap_prover));

    let expanded_proofs: [ExpandedProof; N_PREVIOUS] = rule
        .previous_proof_statements
        .iter()
        .zip(indexes)
        .map(|(statement, (verifier_index, dlog_plonk_index))| {
            let PreviousProofStatement {
                public_input,
                proof,
                proof_must_verify,
            } = statement;

            super::merge::expand_proof(
                verifier_index,
                dlog_plonk_index,
                public_input,
                proof,
                40,
                (),
                *proof_must_verify,
                hack_feature_flags,
            )
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let prevs: [&PerProofWitness; N_PREVIOUS] =
        w.exists(std::array::from_fn(|i| &expanded_proofs[i].witness));
    let unfinalized_proofs_unextended: [&Unfinalized; N_PREVIOUS] =
        w.exists(std::array::from_fn(|i| &expanded_proofs[i].unfinalized));

    let messages_for_next_wrap_proof: [Fp; N_PREVIOUS] = {
        let f = u64_to_field::<Fp, 4>;
        std::array::from_fn(|i| {
            f(&expanded_proofs[i]
                .prev_statement_with_hashes
                .proof_state
                .messages_for_next_wrap_proof)
        })
    };

    let messages_for_next_wrap_proof_padded: [Fp; 2] = w.exists({
        let n_padding = 2 - expanded_proofs.len();
        std::array::from_fn(|i| {
            if i < n_padding {
                get_messages_for_next_wrap_proof_padding()
            } else {
                messages_for_next_wrap_proof[i - n_padding]
            }
        })
    });

    let actual_wrap_domains: [usize; N_PREVIOUS] = {
        let all_possible_domains = wrap_verifier::all_possible_domains();

        let actuals_wrap_domain: [u32; N_PREVIOUS] =
            std::array::from_fn(|i| expanded_proofs[i].actual_wrap_domain);

        actuals_wrap_domain.map(|domain_size| {
            let domain_size = domain_size as u64;
            all_possible_domains
                .iter()
                .position(|Domain::Pow2RootsOfUnity(d)| *d == domain_size)
                .unwrap_or(0)
        })
    };

    let prevs: [PerProofWitness; N_PREVIOUS] = rule
        .previous_proof_statements
        .iter()
        .zip(prevs)
        .map(|(stmt, proof)| proof.clone().with_app_state(stmt.public_input.clone()))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let srs = get_srs::<Fq>();
    let mut srs = srs.lock().unwrap();

    let bulletproof_challenges = prevs
        .iter()
        .zip(messages_for_next_wrap_proof)
        .zip(unfinalized_proofs_unextended)
        .zip(&rule.previous_proof_statements)
        .zip(actual_wrap_domains)
        .zip(for_step_datas)
        .map(
            |(
                ((((proof, msg_for_next_wrap_proof), unfinalized), stmt), actual_wrap_domain),
                data,
            )| {
                let PreviousProofStatement {
                    proof_must_verify: should_verify,
                    ..
                } = stmt;

                match data.wrap_domain {
                    ForStepKind::SideLoaded(_) => {} // Nothing
                    ForStepKind::Known(wrap_domain) => {
                        let actual_wrap_domain = wrap_domains(actual_wrap_domain);
                        assert_eq!(actual_wrap_domain.h, wrap_domain);
                    }
                }
                let (chals, _verified) = verify_one(
                    &mut srs,
                    proof,
                    data,
                    msg_for_next_wrap_proof,
                    unfinalized,
                    *should_verify,
                    w,
                );
                chals.try_into().unwrap()
            },
        )
        .collect::<Vec<[Fp; 16]>>();

    let messages_for_next_step_proof = {
        let msg = MessagesForNextStepProof {
            app_state: Rc::clone(&app_state),
            dlog_plonk_index: &dlog_plonk_index,
            challenge_polynomial_commitments: prevs
                .iter()
                .map(|v| InnerCurve::of_affine(v.wrap_proof.proof.sg.clone()))
                .collect(),
            old_bulletproof_challenges: bulletproof_challenges,
        };
        crate::proofs::witness::checked_hash2(&msg.to_fields(), w)
    };

    let unfinalized_proofs = {
        let mut unfinalized_proofs: Vec<_> =
            unfinalized_proofs_unextended.into_iter().cloned().collect();
        while unfinalized_proofs.len() < 2 {
            unfinalized_proofs.insert(0, Unfinalized::dummy());
        }
        unfinalized_proofs
    };

    let statement = crate::proofs::witness::StepMainStatement {
        proof_state: crate::proofs::witness::StepMainProofState {
            unfinalized_proofs,
            messages_for_next_step_proof,
        },
        messages_for_next_wrap_proof: messages_for_next_wrap_proof_padded.to_vec(),
    };

    w.primary = statement.to_field_elements_owned();

    let proof = create_proof::<C, Fp>(step_prover, prev_challenge_polynomial_commitments, w);

    let proofs: [&v2::PicklesProofProofsVerified2ReprStableV2; N_PREVIOUS] =
        std::array::from_fn(|i| rule.previous_proof_statements[i].proof);

    let prev_evals = proofs
        .iter()
        .zip(&expanded_proofs)
        .map(|(p, expanded)| {
            let evals = evals_from_p2p(&p.proof.evaluations);
            let ft_eval1 = p.proof.ft_eval1.to_field();

            AllEvals {
                ft_eval1,
                evals: EvalsWithPublicInput {
                    evals,
                    public_input: expanded.x_hat,
                },
            }
        })
        .collect::<Vec<_>>();

    let challenge_polynomial_commitments = expanded_proofs
        .iter()
        .map(|v| InnerCurve::of_affine(v.sg.clone()))
        .collect();

    let (old_bulletproof_challenges, messages_for_next_wrap_proof): (Vec<_>, Vec<_>) = proofs
        .iter()
        .map(|v| {
            let StatementProofState {
                deferred_values,
                messages_for_next_wrap_proof,
                ..
            } = (&v.statement.proof_state).into();
            (
                deferred_values.bulletproof_challenges,
                messages_for_next_wrap_proof,
            )
        })
        .unzip();

    let old_bulletproof_challenges = old_bulletproof_challenges
        .into_iter()
        .map(|v: [[u64; 2]; 16]| std::array::from_fn(|i| u64_to_field::<Fp, 2>(&v[i])))
        .collect();

    let step_statement = crate::proofs::witness::StepStatement {
        proof_state: crate::proofs::witness::StepProofState {
            unfinalized_proofs: statement.proof_state.unfinalized_proofs,
            messages_for_next_step_proof: ReducedMessagesForNextStepProof {
                app_state: Rc::clone(&app_state),
                challenge_polynomial_commitments,
                old_bulletproof_challenges,
            },
        },
        messages_for_next_wrap_proof,
    };

    (step_statement, prev_evals, proof)
}

fn block_main<'a>(
    params: BlockMainParams<'a>,
    w: &mut Witness<Fp>,
) -> (Fp, [PreviousProofStatement<'a>; 2]) {
    let BlockMainParams {
        transition,
        prev_state,
        prev_state_proof,
        txn_snark,
        txn_snark_proof,
        next_state,
        prover_state,
        pending_coinbase,
    } = params;

    let new_state_hash = w.exists(MinaHash::hash(next_state));
    w.exists(transition);
    w.exists(txn_snark);

    let (
        previous_state,
        previous_state_hash,
        previous_blockchain_proof_input, // TODO: Use hash here
        previous_state_body_hash,
    ) = {
        w.exists(prev_state);

        let (previous_state_hash, body) = checked_hash_protocol_state(prev_state, w);

        (prev_state, previous_state_hash, prev_state, body)
    };

    let txn_stmt_ledger_hashes_didn_t_change = {
        let s1: Statement<()> =
            (&previous_state.body.blockchain_state.ledger_proof_statement).into();
        let s2: Statement<()> = txn_snark.clone().without_digest();
        txn_statement_ledger_hashes_equal(&s1, &s2, w)
    };

    let supply_increase = w.exists_no_check(match txn_stmt_ledger_hashes_didn_t_change {
        Boolean::True => CheckedSigned::zero(),
        Boolean::False => txn_snark.supply_increase.to_checked(),
    });

    let (updated_consensus_state, consensus_state) = consensus::next_state_checked(
        previous_state,
        previous_state_hash,
        transition,
        supply_increase,
        prover_state,
        w,
    );

    let ConsensusState {
        global_slot_since_genesis,
        coinbase_receiver,
        supercharge_coinbase,
        ..
    } = &consensus_state;

    let prev_pending_coinbase_root = previous_state
        .body
        .blockchain_state
        .staged_ledger_hash
        .pending_coinbase_hash
        .to_field::<Fp>();

    let genesis_state_hash = { genesis_state_hash_checked(previous_state_hash, previous_state, w) };

    let (new_state, is_base_case) = {
        let mut t = protocol_create_var(
            previous_state_hash,
            genesis_state_hash,
            &transition.blockchain_state,
            &consensus_state,
            &previous_state.body.constants,
        );
        let is_base_case = CircuitVar::Var(is_genesis_state_var2(&t.body.consensus_state, w));

        let previous_state_hash = match CONSTRAINT_CONSTANTS.fork.as_ref() {
            Some(ForkConstants {
                previous_state_hash: fork_prev,
                ..
            }) => w.exists_no_check(match is_base_case.value() {
                Boolean::True => *fork_prev,
                Boolean::False => t.previous_state_hash,
            }),
            None => t.previous_state_hash,
        };
        t.previous_state_hash = previous_state_hash;
        checked_hash_protocol_state2(&t, w);
        (t, is_base_case)
    };

    let (txn_snark_should_verify, success) = {
        let mut pending_coinbase = PendingCoinbaseWitness {
            is_new_stack: pending_coinbase.is_new_stack,
            pending_coinbase: (&pending_coinbase.pending_coinbases).into(),
        };

        let global_slot = global_slot_since_genesis;

        let (new_pending_coinbase_hash, deleted_stack, no_coinbases_popped) = {
            let (root_after_delete, deleted_stack) = PendingCoinbase::pop_coinbases(
                txn_stmt_ledger_hashes_didn_t_change.neg(),
                &mut pending_coinbase,
                w,
            );

            let no_coinbases_popped =
                field::equal(root_after_delete, prev_pending_coinbase_root, w);

            let new_root = PendingCoinbase::add_coinbase_checked(
                &transition.pending_coinbase_update,
                coinbase_receiver,
                *supercharge_coinbase,
                previous_state_body_hash,
                global_slot,
                &mut pending_coinbase,
                w,
            );

            (new_root, deleted_stack, no_coinbases_popped)
        };

        let current_ledger_statement = &new_state.body.blockchain_state.ledger_proof_statement;
        let pending_coinbase_source_stack = Stack::var_create_with(&deleted_stack);

        let txn_snark_input_correct = {
            fee_excess::assert_equal_checked(&FeeExcess::empty(), &txn_snark.fee_excess, w);

            let previous_ledger_statement =
                &previous_state.body.blockchain_state.ledger_proof_statement;

            let s1 = previous_ledger_statement.into();
            let s2 = current_ledger_statement.into();

            let ledger_statement_valid = validate_ledgers_at_merge_checked(
                &StatementLedgers::of_statement(&s1),
                &StatementLedgers::of_statement(&s2),
                w,
            );

            let a = txn_snark
                .source
                .pending_coinbase_stack
                .equal_var(&pending_coinbase_source_stack, w);
            let b = txn_snark
                .target
                .pending_coinbase_stack
                .equal_var(&deleted_stack, w);

            Boolean::all(&[ledger_statement_valid, a, b], w)
        };

        let nothing_changed = Boolean::all(
            &[txn_stmt_ledger_hashes_didn_t_change, no_coinbases_popped],
            w,
        );

        let correct_coinbase_status = {
            let new_root = transition
                .blockchain_state
                .staged_ledger_hash
                .pending_coinbase_hash
                .to_field::<Fp>();
            field::equal(new_pending_coinbase_hash, new_root, w)
        };

        Boolean::assert_any(&[txn_snark_input_correct, nothing_changed], w);

        let transaction_snark_should_verifiy = CircuitVar::Var(nothing_changed.neg());

        let result = Boolean::all(&[updated_consensus_state, correct_coinbase_status], w);

        (transaction_snark_should_verifiy, result)
    };

    let prev_should_verify = is_base_case.neg();

    Boolean::assert_any(&[*is_base_case.value(), success], w);

    let previous_proof_statements = [
        PreviousProofStatement {
            public_input: Rc::new(MinaHash::hash(previous_blockchain_proof_input)),
            proof: prev_state_proof,
            proof_must_verify: prev_should_verify,
        },
        PreviousProofStatement {
            public_input: Rc::new(txn_snark.clone()),
            proof: txn_snark_proof,
            proof_must_verify: txn_snark_should_verify,
        },
    ];

    (new_state_hash, previous_proof_statements)
}

pub struct ProverExtendBlockchainInputStableV22 {
    pub chain: v2::BlockchainSnarkBlockchainStableV2,
    pub next_state: v2::MinaStateProtocolStateValueStableV2,
    pub block: v2::MinaStateSnarkTransitionValueStableV2,
    pub ledger_proof: Option<v2::LedgerProofProdStableV2>,
    pub prover_state: v2::ConsensusStakeProofStableV2,
    pub pending_coinbase: v2::MinaBasePendingCoinbaseWitnessStableV2,
}

struct BlockMainParams<'a> {
    transition: &'a v2::MinaStateSnarkTransitionValueStableV2,
    prev_state: &'a v2::MinaStateProtocolStateValueStableV2,
    prev_state_proof: &'a v2::MinaBaseProofStableV2,
    txn_snark: &'a Statement<SokDigest>,
    txn_snark_proof: &'a v2::TransactionSnarkProofStableV2,
    next_state: &'a v2::MinaStateProtocolStateValueStableV2,
    prover_state: &'a v2::ConsensusStakeProofStableV2,
    pending_coinbase: &'a v2::MinaBasePendingCoinbaseWitnessStableV2,
}

pub struct BlockParams<'a> {
    pub input: &'a v2::ProverExtendBlockchainInputStableV2,
    pub block_step_prover: &'a Prover<Fp>,
    pub block_wrap_prover: &'a Prover<Fq>,
    pub tx_wrap_prover: &'a Prover<Fq>,
    /// For debugging only
    pub expected_step_proof: Option<&'static str>,
    /// For debugging only
    pub ocaml_wrap_witness: Option<Vec<Fq>>,
}

const BLOCK_N_PREVIOUS_PROOFS: usize = 2;

pub fn generate_block_proof(params: BlockParams, w: &mut Witness<Fp>) -> WrapProof {
    w.ocaml_aux = read_witnesses();

    let BlockParams {
        input:
            v2::ProverExtendBlockchainInputStableV2 {
                chain,
                next_state,
                block,
                ledger_proof,
                prover_state,
                pending_coinbase,
            },
        block_step_prover,
        block_wrap_prover,
        tx_wrap_prover,
        expected_step_proof,
        ocaml_wrap_witness,
    } = params;

    let (txn_snark_statement, txn_snark_proof) =
        ledger_proof_opt(ledger_proof.as_ref(), next_state);
    let prev_state_proof = &chain.proof;

    let (new_state_hash, previous_proof_statements) = block_main(
        BlockMainParams {
            transition: block,
            prev_state: &chain.state,
            prev_state_proof,
            txn_snark: &txn_snark_statement,
            txn_snark_proof: &txn_snark_proof,
            next_state,
            prover_state,
            pending_coinbase,
        },
        w,
    );

    let prev_challenge_polynomial_commitments =
        extract_recursion_challenges(&[&prev_state_proof, &txn_snark_proof]);

    let rule = InductiveRule {
        previous_proof_statements,
        public_output: (),
        auxiliary_output: (),
    };

    let dlog_plonk_index = super::merge::dlog_plonk_index(block_wrap_prover);
    let verifier_index = block_wrap_prover.index.verifier_index.as_ref().unwrap();

    let tx_dlog_plonk_index = super::merge::dlog_plonk_index(tx_wrap_prover);
    let tx_verifier_index = tx_wrap_prover.index.verifier_index.as_ref().unwrap();

    let dlog_plonk_index_cvar = dlog_plonk_index.to_cvar(CircuitVar::Var);
    let tx_dlog_plonk_index_cvar = tx_dlog_plonk_index.to_cvar(CircuitVar::Constant);

    let indexes = [
        (verifier_index, &dlog_plonk_index_cvar),
        (tx_verifier_index, &tx_dlog_plonk_index_cvar),
    ];

    let block_data = make_step_block_data(&dlog_plonk_index_cvar);
    let tx_data = make_step_transaction_data(&tx_dlog_plonk_index_cvar);
    let for_step_datas = [&block_data, &tx_data];

    let app_state: Rc<dyn ToFieldElementsDebug> = Rc::new(new_state_hash);

    let (step_statement, prev_evals, proof) = step::<StepBlockProof, BLOCK_N_PREVIOUS_PROOFS>(
        StepParams {
            app_state: Rc::clone(&app_state),
            rule,
            for_step_datas,
            indexes,
            prev_challenge_polynomial_commitments,
            hack_feature_flags: OptFlag::No,
            wrap_prover: block_wrap_prover,
            step_prover: block_step_prover,
        },
        w,
    );

    if let Some(expected) = expected_step_proof {
        let proof_json = serde_json::to_vec(&proof).unwrap();
        assert_eq!(sha256_sum(&proof_json), expected);
    };

    let mut w = Witness::new::<WrapBlockProof>();

    if let Some(ocaml_aux) = ocaml_wrap_witness {
        w.ocaml_aux = ocaml_aux;
    };

    wrap::<WrapBlockProof>(
        WrapParams {
            app_state,
            proof: &proof,
            step_statement,
            prev_evals: &prev_evals,
            dlog_plonk_index: &dlog_plonk_index,
            step_prover_index: &block_step_prover.index,
            wrap_prover: block_wrap_prover,
        },
        &mut w,
    )
}
