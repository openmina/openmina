#![allow(unused)]

use std::{path::Path, str::FromStr, sync::Arc};

use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use mina_p2p_messages::v2;

use crate::{
    dummy,
    proofs::{numbers::currency::CheckedSigned, witness::Boolean},
    scan_state::{
        protocol_state::MinaHash,
        scan_state::transaction_snark::{Registers, SokDigest, Statement},
    },
    Inputs, ToInputs,
};

use super::{
    numbers::{
        currency::CheckedAmount,
        nat::{CheckedBlockTime, CheckedBlockTimeSpan, CheckedLength},
    },
    to_field_elements::ToFieldElements,
    witness::{
        field,
        transaction_snark::{checked_hash, CONSTRAINT_CONSTANTS},
        Check, Prover, Witness,
    },
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
    use ark_ff::{BigInteger, BigInteger256, One, Zero};
    use num_bigint::BigUint;

    use crate::{
        proofs::witness::{field_to_bits, field_to_bits2, FieldWitness},
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
    }

    pub struct SnarkyInteger<F: FieldWitness> {
        pub value: F,
        pub interval: Interval,
        pub bits: Option<Vec<Boolean>>,
    }

    impl<F: FieldWitness> SnarkyInteger<F> {
        pub fn create(value: F, upper_bound: BigUint) -> Self {
            Self {
                value,
                interval: Interval::LessThan(upper_bound),
                bits: None,
            }
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

        fn div_mod(&self, b: &Self, w: &mut Witness<F>) {
            let (q, r) = w.exists({
                let a: BigUint = self.value.into();
                let b: BigUint = b.value.into();
                (F::from(&a / &b), F::from(&a % &b))
            });

            let q_bit_length = self.interval.bits_needed();
            let b_bit_length = b.interval.bits_needed();

            let q_bits = w.exists(field_to_bits2(q, q_bit_length as usize));
            let b_bits = w.exists(field_to_bits2(r, b_bit_length as usize));

            dbg!(q_bit_length, b_bit_length);
        }
    }

    //   Field.Assert.lt ~bit_length:b_bit_length r b.value ;
    //   (* This assertion checkes that the multiplication q * b is safe. *)
    //   assert (q_bit_length + b_bit_length + 1 < Field.Constant.size_in_bits) ;
    //   assert_r1cs q b.value Field.(a.value - r) ;
    //   ( { value = q
    //     ; interval = Interval.quotient a.interval b.interval
    //     ; bits = Some q_bits
    //     }
    //   , { value = r; interval = b.interval; bits = Some r_bits } )

    pub fn balance_upper_bound() -> BigUint {
        BigUint::new(vec![1, 0, 0, 0]) << Balance::NBITS as u32
    }

    pub fn amount_upper_bound() -> BigUint {
        BigUint::new(vec![1, 0, 0, 0]) << Amount::NBITS as u32
    }

    pub fn of_quotient(
        precision: usize,
        top: SnarkyInteger<Fp>,
        bottom: SnarkyInteger<Fp>,
        w: &mut Witness<Fp>,
    ) {
        top.shift_left(precision).div_mod(&bottom, w);

        // let of_quotient ~m ~precision ~top ~bottom ~top_is_less_than_bottom:() =
        //   let q, _r = Integer.(div_mod ~m (shift_left ~m top precision) bottom) in
        //   { value = Integer.to_field q; precision }
    }
}

mod vrf {
    use std::ops::Neg;

    use ark_ff::{BigInteger, BigInteger256};
    use mina_signer::{CompressedPubKey, PubKey};

    use crate::{
        checked_verify_merkle_path,
        proofs::{
            numbers::nat::{CheckedNat, CheckedSlot},
            witness::{
                decompress_var, field_to_bits, legacy_input::to_bits, scale_known,
                scale_non_constant, FieldWitness, GroupAffine, InnerCurve,
            },
        },
        scan_state::{
            currency::{Amount, Balance, Sgn},
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
        truncated_result: &[bool; VRF_OUTPUT_NBITS],
        w: &mut Witness<Fp>,
    ) {
        use floating_point::*;

        let top = SnarkyInteger::create(my_stake.to_field::<Fp>(), balance_upper_bound());
        let bottom = SnarkyInteger::create(total_stake.to_field::<Fp>(), amount_upper_bound());
        let precision = PARAMS.per_term_precision;

        floating_point::of_quotient(precision, top, bottom, w);
    }

    const VRF_OUTPUT_NBITS: usize = 253;

    fn truncate_vrf_output(output: Fp, w: &mut Witness<Fp>) -> [bool; VRF_OUTPUT_NBITS] {
        let output = w.exists(field_to_bits::<_, 255>(output));
        std::array::from_fn(|i| output[i])
    }

    pub fn check(
        m: &InnerCurve<Fp>,
        epoch_ledger: &EpochLedger<Fp>,
        global_slot: &CheckedSlot<Fp>,
        block_stake_winner: &CompressedPubKey,
        block_creator: &CompressedPubKey,
        seed: Fp,
        prover_state: &v2::ConsensusStakeProofStableV2,
        w: &mut Witness<Fp>,
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

        epoch_ledger.total_currency;

        is_satisfied(my_stake, epoch_ledger.total_currency, &truncated_result, w);

        // let my_stake = winner_account.balance in
        // let%bind truncated_result = Output.Checked.truncate result in
        // let%map satisifed =
        //   Threshold.Checked.is_satisfied ~my_stake
        //     ~total_stake:epoch_ledger.total_currency truncated_result
        // in
        // (satisifed, result, truncated_result, winner_account)
    }
}

mod consensus {
    use mina_signer::CompressedPubKey;

    use super::*;
    use crate::{
        decompress_pk,
        proofs::{
            numbers::nat::{CheckedN, CheckedNat, CheckedSlot, CheckedSlotSpan},
            witness::{compress_var, create_shifted_inner_curve, CompressedPubKeyVar, InnerCurve},
        },
        scan_state::transaction_logic::protocol_state::EpochData,
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

    #[derive(Debug)]
    pub struct GlobalSlot {
        slot_number: CheckedSlot<Fp>,
        slots_per_epoch: CheckedLength<Fp>,
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

    pub fn next_state_checked(
        prev_state: &v2::MinaStateProtocolStateValueStableV2,
        prev_state_hash: Fp,
        transition: &v2::MinaStateSnarkTransitionValueStableV2,
        supply_increase: CheckedSigned<Fp, CheckedAmount<Fp>>,
        prover_state: &v2::ConsensusStakeProofStableV2,
        w: &mut Witness<Fp>,
    ) {
        let _previous_blockchain_state_ledger_hash = prev_state
            .body
            .blockchain_state
            .ledger_proof_statement
            .target
            .first_pass_ledger
            .to_field::<Fp>();
        let _genesis_ledger_hash = prev_state
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
        let (prev_epoch, prev_slot) = prev_global_slot.to_epoch_and_slot(w);

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

        {
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
            );
        }
    }
}

pub struct ProverExtendBlockchainInputStableV22 {
    pub chain: v2::BlockchainSnarkBlockchainStableV2,
    pub next_state: v2::MinaStateProtocolStateValueStableV2,
    pub block: v2::MinaStateSnarkTransitionValueStableV2,
    pub ledger_proof: Option<v2::LedgerProofProdStableV2>,
    pub prover_state: v2::ConsensusStakeProofStableV2,
    pub pending_coinbase: v2::MinaBasePendingCoinbaseWitnessStableV2,
}

struct BlockProofParams<'a> {
    transition: &'a v2::MinaStateSnarkTransitionValueStableV2,
    prev_state: &'a v2::MinaStateProtocolStateValueStableV2,
    prev_state_proof: &'a v2::MinaBaseProofStableV2,
    txn_snark: &'a Statement<SokDigest>,
    txn_snark_proof: &'a v2::TransactionSnarkProofStableV2,
}

pub fn generate_block_proof(
    input: &v2::ProverExtendBlockchainInputStableV2,
    block_prover: &Prover<Fp>,
    wrap_prover: &Prover<Fq>,
    w: &mut Witness<Fp>,
) {
    w.ocaml_aux = read_witnesses();

    let v2::ProverExtendBlockchainInputStableV2 {
        chain,
        next_state,
        block,
        ledger_proof,
        prover_state,
        pending_coinbase,
    } = input;

    let (txn_snark_statement, txn_snark_proof) =
        ledger_proof_opt(ledger_proof.as_ref(), next_state);

    let params = BlockProofParams {
        transition: block,
        prev_state: &chain.state,
        prev_state_proof: &chain.proof,
        txn_snark: &txn_snark_statement,
        txn_snark_proof: &txn_snark_proof,
    };

    let BlockProofParams {
        transition,
        prev_state,
        prev_state_proof,
        txn_snark,
        txn_snark_proof,
    } = params;

    let new_state_hash = w.exists(MinaHash::hash(next_state));
    w.exists(transition);
    w.exists(txn_snark);

    let (
        previous_state,
        previous_state_hash,
        previous_blockchain_proof_input,
        previous_state_body_hash,
    ) = {
        w.exists(prev_state);

        let (previous_state_hash, body) = checked_hash_protocol_state(prev_state, w);

        (prev_state, previous_state_hash, (), body)
    };

    let txn_stmt_ledger_hashes_didn_t_change = {
        let s1: Statement<()> =
            (&previous_state.body.blockchain_state.ledger_proof_statement).into();
        let s2: Statement<()> = txn_snark.clone().without_digest();
        txn_statement_ledger_hashes_equal(&s1, &s2, w)
    };

    eprintln!("AAA");

    let supply_increase = w.exists_no_check(match txn_stmt_ledger_hashes_didn_t_change {
        Boolean::True => CheckedSigned::zero(),
        Boolean::False => txn_snark.supply_increase.to_checked(),
    });

    consensus::next_state_checked(
        previous_state,
        previous_state_hash,
        transition,
        supply_increase,
        prover_state,
        w,
    );

    // let%bind supply_increase =
    //   (* only increase the supply if the txn statement represents a new ledger transition *)
    //   Currency.Amount.(
    //     Signed.Checked.if_ txn_stmt_ledger_hashes_didn't_change
    //       ~then_:
    //         (Signed.create_var ~magnitude:(var_of_t zero) ~sgn:Sgn.Checked.pos)
    //       ~else_:txn_snark.supply_increase)
    // in
    // let%bind `Success updated_consensus_state, consensus_state =
    //   with_label __LOC__ (fun () ->
    //       Consensus_state_hooks.next_state_checked ~constraint_constants
    //         ~prev_state:previous_state ~prev_state_hash:previous_state_hash
    //         transition supply_increase )
    // in
    // let global_slot =
    //   Consensus.Data.Consensus_state.global_slot_since_genesis_var consensus_state
    // in
    // let supercharge_coinbase =
    //   Consensus.Data.Consensus_state.supercharge_coinbase_var consensus_state
    // in
    // let prev_pending_coinbase_root =
    //   previous_state |> Protocol_state.blockchain_state
    //   |> Blockchain_state.staged_ledger_hash
    //   |> Staged_ledger_hash.pending_coinbase_hash_var
    // in
}
