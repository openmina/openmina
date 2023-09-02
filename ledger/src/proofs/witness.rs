use ark_ec::{
    short_weierstrass_jacobian::{GroupAffine, GroupProjective},
    AffineCurve, ProjectiveCurve, SWModelParameters,
};
use ark_ff::{BigInteger256, Field};
use mina_curves::pasta::{
    Fq, PallasParameters, ProjectivePallas, ProjectiveVesta, VestaParameters,
};
use mina_hasher::Fp;
use mina_p2p_messages::{
    string::ByteString,
    v2::{
        ConsensusGlobalSlotStableV1, ConsensusProofOfStakeDataConsensusStateValueStableV2,
        ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
        ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
        CurrencyAmountStableV1, MinaBaseEpochLedgerValueStableV1, MinaBaseFeeExcessStableV1,
        MinaBaseProtocolConstantsCheckedValueStableV1, MinaNumbersGlobalSlotSinceGenesisMStableV1,
        MinaNumbersGlobalSlotSinceHardForkMStableV1, MinaStateBlockchainStateValueStableV2,
        MinaStateBlockchainStateValueStableV2LedgerProofStatement,
        MinaStateBlockchainStateValueStableV2LedgerProofStatementSource,
        MinaStateBlockchainStateValueStableV2SignedAmount, MinaStateProtocolStateBodyValueStableV2,
        MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1, NonZeroCurvePoint,
        NonZeroCurvePointUncompressedStableV1, SgnStableV1, SignedAmount, TokenFeeExcess,
        UnsignedExtendedUInt32StableV1, UnsignedExtendedUInt64Int64ForVersionTagsStableV1,
    },
};

use crate::{
    scan_state::{
        scan_state::transaction_snark::Statement,
        transaction_logic::protocol_state::{EpochData, EpochLedger},
    },
    staged_ledger::hash::StagedLedgerHash,
};

use super::to_field_elements::ToFieldElements;

#[derive(Debug)]
pub struct Witness<C: FieldWitness> {
    aux: Vec<C>,
}

impl<F: FieldWitness> Witness<F> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            aux: Vec::with_capacity(capacity),
        }
    }

    pub fn push<I: Into<F>>(&mut self, field: I) {
        let field = {
            let field: F = field.into();
            dbg!(field)
        };
        self.aux.push(field)
    }

    pub fn pusher(&mut self) -> impl FnMut(F) -> F + '_ {
        let push = |f: F| -> F {
            self.push(f);
            f
        };
        push
    }

    fn exists<T>(&mut self, data: T) -> T
    where
        T: ToFieldElements<F> + Check<F>,
    {
        data.to_field_elements(&mut self.aux);
        data.check(self);
        data
    }

    pub fn to_field_checked_prime<const NBITS: usize>(&mut self, scalar: F) -> (F, F, F) {
        to_field_checked_prime::<F, NBITS>(scalar, self)
    }
}

pub trait Check<F: FieldWitness> {
    fn check(&self, witnesses: &mut Witness<F>);
}

struct FieldBitsIterator {
    index: usize,
    bigint: BigInteger256,
}

impl Iterator for FieldBitsIterator {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        self.index += 1;

        let limb_index = index / 64;
        let bit_index = index % 64;

        let limb = self.bigint.0.get(limb_index)?;
        Some(limb & (1 << bit_index) != 0)
    }
}

fn field_to_bits<F, const NBITS: usize>(field: F) -> [bool; NBITS]
where
    F: Field + Into<BigInteger256>,
{
    let bigint: BigInteger256 = field.into();
    let mut bits = FieldBitsIterator { index: 0, bigint }.take(NBITS);
    std::array::from_fn(|_| bits.next().unwrap())
}

fn bits_msb<F, const NBITS: usize>(field: F) -> [bool; NBITS]
where
    F: Field + Into<BigInteger256>,
{
    let mut bits = field_to_bits::<_, NBITS>(field);
    bits.reverse();
    bits
}

// TODO: This function is incomplete (compare to OCaml), here it only push witness values
// https://github.com/MinaProtocol/mina/blob/357144819e7ce5f61109d23d33da627be28024c7/src/lib/pickles/scalar_challenge.ml#L12
pub fn to_field_checked_prime<F, const NBITS: usize>(
    scalar: F,
    witnesses: &mut Witness<F>,
) -> (F, F, F)
where
    F: FieldWitness,
{
    let mut push = witnesses.pusher();

    let neg_one = F::one().neg();

    let a_func = |n: u64| match n {
        0 => F::zero(),
        1 => F::zero(),
        2 => neg_one,
        3 => F::one(),
        _ => panic!("invalid argument"),
    };

    let b_func = |n: u64| match n {
        0 => neg_one,
        1 => F::one(),
        2 => F::zero(),
        3 => F::zero(),
        _ => panic!("invalid argument"),
    };

    let bits_msb: [bool; NBITS] = bits_msb::<_, NBITS>(scalar);

    let nybbles_per_row = 8;
    let bits_per_row = 2 * nybbles_per_row;
    assert_eq!((NBITS % bits_per_row), 0);
    let rows = NBITS / bits_per_row;

    // TODO: Use arrays when const feature allows it
    // https://github.com/rust-lang/rust/issues/76560
    let nybbles_by_row: Vec<Vec<u64>> = (0..rows)
        .map(|i| {
            (0..nybbles_per_row)
                .map(|j| {
                    let bit = (bits_per_row * i) + (2 * j);
                    let b0 = bits_msb[bit + 1] as u64;
                    let b1 = bits_msb[bit] as u64;
                    b0 + (2 * b1)
                })
                .collect()
        })
        .collect();

    let two: F = 2u64.into();
    let mut a = two;
    let mut b = two;
    let mut n = F::zero();

    for i in 0..rows {
        let n0 = n;
        let a0 = a;
        let b0 = b;

        let xs: Vec<F> = (0..nybbles_per_row)
            .map(|j| push(F::from(nybbles_by_row[i][j])))
            .collect();

        let n8: F = push(xs.iter().fold(n0, |accum, x| accum.double().double() + x));

        let a8: F = push(
            nybbles_by_row[i]
                .iter()
                .fold(a0, |accum, x| accum.double() + a_func(*x)),
        );

        let b8: F = push(
            nybbles_by_row[i]
                .iter()
                .fold(b0, |accum, x| accum.double() + b_func(*x)),
        );

        n = n8;
        a = a8;
        b = b8;
    }

    (a, b, n)
}

impl ToFieldElements<Fp> for Fp {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        fields.push(*self);
    }
}

impl ToFieldElements<Fq> for Fq {
    fn to_field_elements(&self, fields: &mut Vec<Fq>) {
        fields.push(*self);
    }
}

impl ToFieldElements<Fq> for Fp {
    fn to_field_elements(&self, fields: &mut Vec<Fq>) {
        let field: BigInteger256 = (*self).into();
        fields.push(field.into());
    }
}

impl ToFieldElements<Fp> for Fq {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let field: BigInteger256 = (*self).into();
        fields.push(field.into());
    }
}

impl ToFieldElements<Fp> for StagedLedgerHash {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self {
            non_snark,
            pending_coinbase_hash,
        } = self;

        let non_snark_digest = non_snark.digest();

        const BITS: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];
        fields.extend(
            non_snark_digest
                .iter()
                .flat_map(|byte| BITS.iter().map(|bit| Fp::from((*byte & bit != 0) as u64))),
        );

        fields.push(*pending_coinbase_hash);
    }
}

impl ToFieldElements<Fp> for ByteString {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let slice: &[u8] = self;
        slice.to_field_elements(fields);
    }
}

impl ToFieldElements<Fp> for &'_ [u8] {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        const BITS: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];
        fields.extend(
            self.iter()
                .flat_map(|byte| BITS.iter().map(|bit| Fp::from((*byte & bit != 0) as u64))),
        );
    }
}

impl ToFieldElements<Fp> for EpochData {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self {
            ledger:
                EpochLedger {
                    hash,
                    total_currency,
                },
            seed,
            start_checkpoint,
            lock_checkpoint,
            epoch_length,
        } = self;

        fields.push(*hash);
        fields.push(total_currency.as_u64().into());
        fields.push(*seed);
        fields.push(*start_checkpoint);
        fields.push(*lock_checkpoint);
        fields.push(epoch_length.as_u32().into());
    }
}

impl ToFieldElements<Fp> for NonZeroCurvePoint {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let NonZeroCurvePointUncompressedStableV1 { x, is_odd } = self.inner();

        fields.push(x.to_field());
        fields.push((*is_odd).into());
    }
}

impl ToFieldElements<Fp> for ConsensusProofOfStakeDataConsensusStateValueStableV2 {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let ConsensusProofOfStakeDataConsensusStateValueStableV2 {
            blockchain_length,
            epoch_count,
            min_window_density,
            sub_window_densities,
            last_vrf_output,
            total_currency,
            curr_global_slot:
                ConsensusGlobalSlotStableV1 {
                    slot_number,
                    slots_per_epoch,
                },
            global_slot_since_genesis,
            staking_epoch_data,
            next_epoch_data,
            has_ancestor_in_same_checkpoint_window,
            block_stake_winner,
            block_creator,
            coinbase_receiver,
            supercharge_coinbase,
        } = self;

        let staking_epoch_data: EpochData = staking_epoch_data.into();
        let next_epoch_data: EpochData = next_epoch_data.into();

        fields.push(blockchain_length.as_u32().into());
        fields.push(epoch_count.as_u32().into());
        fields.push(min_window_density.as_u32().into());
        fields.extend(sub_window_densities.iter().map(|w| Fp::from(w.as_u32())));

        {
            let vrf: &[u8] = last_vrf_output.as_ref();
            (&vrf[..31]).to_field_elements(fields);
            // Ignore the last 3 bits
            let last_byte = vrf[31];
            for bit in [1, 2, 4, 8, 16] {
                fields.push(Fp::from(last_byte & bit != 0))
            }
        }

        fields.push(total_currency.as_u64().into());
        fields.push(slot_number.as_u32().into());
        fields.push(slots_per_epoch.as_u32().into());
        fields.push(global_slot_since_genesis.as_u32().into());
        staking_epoch_data.to_field_elements(fields);
        next_epoch_data.to_field_elements(fields);
        fields.push((*has_ancestor_in_same_checkpoint_window).into());
        block_stake_winner.to_field_elements(fields);
        block_creator.to_field_elements(fields);
        coinbase_receiver.to_field_elements(fields);
        fields.push((*supercharge_coinbase).into());
    }
}

impl ToFieldElements<Fp> for MinaBaseProtocolConstantsCheckedValueStableV1 {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self {
            k,
            slots_per_epoch,
            slots_per_sub_window,
            delta,
            genesis_state_timestamp,
        } = self;

        fields.push(k.as_u32().into());
        fields.push(slots_per_epoch.as_u32().into());
        fields.push(slots_per_sub_window.as_u32().into());
        fields.push(delta.as_u32().into());
        fields.push(genesis_state_timestamp.as_u64().into());
    }
}

impl ToFieldElements<Fp> for MinaStateBlockchainStateValueStableV2 {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self {
            staged_ledger_hash,
            genesis_ledger_hash,
            ledger_proof_statement,
            timestamp,
            body_reference,
        } = self;

        let staged_ledger_hash: StagedLedgerHash = staged_ledger_hash.into();
        let ledger_proof_statement: Statement<()> = ledger_proof_statement.into();

        staged_ledger_hash.to_field_elements(fields);
        fields.push(genesis_ledger_hash.inner().to_field());
        ledger_proof_statement.to_field_elements(fields);
        fields.push(timestamp.as_u64().into());
        body_reference.to_field_elements(fields);
    }
}

impl ToFieldElements<Fp> for MinaStateProtocolStateBodyValueStableV2 {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let MinaStateProtocolStateBodyValueStableV2 {
            genesis_state_hash,
            blockchain_state,
            consensus_state,
            constants,
        } = self;

        fields.push(genesis_state_hash.inner().to_field());
        blockchain_state.to_field_elements(fields);
        consensus_state.to_field_elements(fields);
        constants.to_field_elements(fields);
    }
}

impl<F: FieldWitness> Check<F> for SgnStableV1 {
    fn check(&self, _witnesses: &mut Witness<F>) {
        // Does not modify the witness
    }
}

impl<F: FieldWitness> Check<F> for bool {
    fn check(&self, _witnesses: &mut Witness<F>) {
        // Does not modify the witness
    }
}

impl<F: FieldWitness> Check<F> for Fp {
    fn check(&self, _witnesses: &mut Witness<F>) {
        // Does not modify the witness
    }
}

impl<F: FieldWitness> Check<F> for Fq {
    fn check(&self, _witnesses: &mut Witness<F>) {
        // Does not modify the witness
    }
}

impl<F: FieldWitness> Check<F> for CurrencyAmountStableV1 {
    fn check(&self, witnesses: &mut Witness<F>) {
        // eprintln!("check CurrencyAmountStableV1 START");
        const NBITS: usize = u64::BITS as usize;

        let amount: u64 = self.as_u64();
        assert_eq!(NBITS, std::mem::size_of_val(&amount) * 8);

        let amount: F = amount.into();
        to_field_checked_prime::<F, NBITS>(amount, witnesses);
        // eprintln!("check CurrencyAmountStableV1 DONE");
    }
}

impl<F: FieldWitness> Check<F> for SignedAmount {
    fn check(&self, witnesses: &mut Witness<F>) {
        let Self { magnitude, sgn } = self;
        magnitude.check(witnesses);
        sgn.check(witnesses);
    }
}

impl<F: FieldWitness> Check<F> for MinaStateBlockchainStateValueStableV2SignedAmount {
    fn check(&self, witnesses: &mut Witness<F>) {
        let Self { magnitude, sgn } = self;
        magnitude.check(witnesses);
        sgn.check(witnesses);
    }
}

impl<F: FieldWitness> Check<F> for UnsignedExtendedUInt32StableV1 {
    fn check(&self, witnesses: &mut Witness<F>) {
        // eprintln!("check UnsignedExtendedUInt32StableV1 START");
        const NBITS: usize = u32::BITS as usize;

        let number: u32 = self.as_u32();
        assert_eq!(NBITS, std::mem::size_of_val(&number) * 8);

        let number: F = number.into();
        to_field_checked_prime::<F, NBITS>(number, witnesses);
        // eprintln!("check UnsignedExtendedUInt32StableV1 DONE");
    }
}

impl<F: FieldWitness> Check<F> for MinaStateBlockchainStateValueStableV2LedgerProofStatementSource {
    fn check(&self, witnesses: &mut Witness<F>) {
        let Self {
            first_pass_ledger: _,
            second_pass_ledger: _,
            pending_coinbase_stack: _,
            local_state:
                MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1 {
                    stack_frame: _,
                    call_stack: _,
                    transaction_commitment: _,
                    full_transaction_commitment: _,
                    excess,
                    supply_increase,
                    ledger: _,
                    success,
                    account_update_index,
                    failure_status_tbl: _,
                    will_succeed,
                },
        } = self;

        excess.check(witnesses);
        supply_increase.check(witnesses);
        success.check(witnesses);
        account_update_index.check(witnesses);
        will_succeed.check(witnesses);
    }
}

impl<F: FieldWitness> Check<F> for MinaBaseFeeExcessStableV1 {
    fn check(&self, witnesses: &mut Witness<F>) {
        let Self(
            TokenFeeExcess {
                token: _fee_token_l,
                amount: fee_excess_l,
            },
            TokenFeeExcess {
                token: _fee_token_r,
                amount: fee_excess_r,
            },
        ) = self;

        fee_excess_l.check(witnesses);
        fee_excess_r.check(witnesses);
    }
}

impl<F: FieldWitness> Check<F> for UnsignedExtendedUInt64Int64ForVersionTagsStableV1 {
    fn check(&self, witnesses: &mut Witness<F>) {
        // eprintln!("check UnsignedExtendedUInt64Int64ForVersionTagsStableV1 START");
        const NBITS: usize = u64::BITS as usize;

        let number: u64 = self.as_u64();
        assert_eq!(NBITS, std::mem::size_of_val(&number) * 8);

        let number: F = number.into();
        to_field_checked_prime::<F, NBITS>(number, witnesses);
        // eprintln!("check UnsignedExtendedUInt64Int64ForVersionTagsStableV1 DONE");
    }
}

impl<F: FieldWitness> Check<F> for MinaNumbersGlobalSlotSinceGenesisMStableV1 {
    fn check(&self, witnesses: &mut Witness<F>) {
        let Self::SinceGenesis(global_slot) = self;
        global_slot.check(witnesses);
    }
}

impl<F: FieldWitness> Check<F> for MinaNumbersGlobalSlotSinceHardForkMStableV1 {
    fn check(&self, witnesses: &mut Witness<F>) {
        let Self::SinceHardFork(global_slot) = self;
        global_slot.check(witnesses);
    }
}

impl<F: FieldWitness> Check<F>
    for ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1
{
    fn check(&self, witnesses: &mut Witness<F>) {
        let Self {
            ledger:
                MinaBaseEpochLedgerValueStableV1 {
                    hash: _,
                    total_currency,
                },
            seed: _,
            start_checkpoint: _,
            lock_checkpoint: _,
            epoch_length,
        } = self;

        total_currency.check(witnesses);
        epoch_length.check(witnesses);
    }
}

impl<F: FieldWitness> Check<F>
    for ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1
{
    fn check(&self, witnesses: &mut Witness<F>) {
        let Self {
            ledger:
                MinaBaseEpochLedgerValueStableV1 {
                    hash: _,
                    total_currency,
                },
            seed: _,
            start_checkpoint: _,
            lock_checkpoint: _,
            epoch_length,
        } = self;

        total_currency.check(witnesses);
        epoch_length.check(witnesses);
    }
}

impl<F: FieldWitness> Check<F> for ConsensusGlobalSlotStableV1 {
    fn check(&self, witnesses: &mut Witness<F>) {
        let Self {
            slot_number,
            slots_per_epoch,
        } = self;

        slot_number.check(witnesses);
        slots_per_epoch.check(witnesses);
    }
}

impl<F: FieldWitness> Check<F> for MinaStateProtocolStateBodyValueStableV2 {
    fn check(&self, witnesses: &mut Witness<F>) {
        let MinaStateProtocolStateBodyValueStableV2 {
            genesis_state_hash: _,
            blockchain_state:
                MinaStateBlockchainStateValueStableV2 {
                    staged_ledger_hash: _,
                    genesis_ledger_hash: _,
                    ledger_proof_statement:
                        MinaStateBlockchainStateValueStableV2LedgerProofStatement {
                            source,
                            target,
                            connecting_ledger_left: _,
                            connecting_ledger_right: _,
                            supply_increase,
                            fee_excess,
                            sok_digest: _,
                        },
                    timestamp,
                    body_reference: _,
                },
            consensus_state:
                ConsensusProofOfStakeDataConsensusStateValueStableV2 {
                    blockchain_length,
                    epoch_count,
                    min_window_density,
                    sub_window_densities,
                    last_vrf_output: _,
                    total_currency,
                    curr_global_slot,
                    global_slot_since_genesis,
                    staking_epoch_data,
                    next_epoch_data,
                    has_ancestor_in_same_checkpoint_window,
                    block_stake_winner: _,
                    block_creator: _,
                    coinbase_receiver: _,
                    supercharge_coinbase,
                },
            constants:
                MinaBaseProtocolConstantsCheckedValueStableV1 {
                    k,
                    slots_per_epoch,
                    slots_per_sub_window,
                    delta,
                    genesis_state_timestamp,
                },
        } = self;

        source.check(witnesses);
        target.check(witnesses);
        supply_increase.check(witnesses);
        fee_excess.check(witnesses);
        timestamp.check(witnesses);
        blockchain_length.check(witnesses);
        epoch_count.check(witnesses);
        min_window_density.check(witnesses);
        // TODO: Check/assert that length equal `constraint_constants.sub_windows_per_window`
        for sub_window_density in sub_window_densities {
            sub_window_density.check(witnesses);
        }
        total_currency.check(witnesses);
        curr_global_slot.check(witnesses);
        global_slot_since_genesis.check(witnesses);
        staking_epoch_data.check(witnesses);
        next_epoch_data.check(witnesses);
        has_ancestor_in_same_checkpoint_window.check(witnesses);
        supercharge_coinbase.check(witnesses);
        k.check(witnesses);
        slots_per_epoch.check(witnesses);
        slots_per_sub_window.check(witnesses);
        delta.check(witnesses);
        genesis_state_timestamp.check(witnesses);
    }
}

/// All the generics we need during witness generation
pub trait FieldWitness
where
    Self: Field + Send + Sync + Into<BigInteger256> + ToFieldElements<Self> + std::fmt::Debug,
{
    type Scalar: Field + std::fmt::Debug;
    type Affine: AffineCurve<Projective = Self::Projective, BaseField = Self, ScalarField = Self::Scalar>
        + Into<GroupAffine<Self::Parameters>>
        + std::fmt::Debug;
    type Projective: ProjectiveCurve<Affine = Self::Affine, BaseField = Self, ScalarField = Self::Scalar>
        + From<GroupProjective<Self::Parameters>>
        + std::fmt::Debug;
    type Parameters: SWModelParameters<BaseField = Self, ScalarField = Self::Scalar>
        + std::fmt::Debug;
}

impl FieldWitness for Fp {
    type Scalar = Fq;
    type Parameters = PallasParameters;
    type Affine = GroupAffine<Self::Parameters>;
    type Projective = ProjectivePallas;
}

impl FieldWitness for Fq {
    type Scalar = Fp;
    type Parameters = VestaParameters;
    type Affine = GroupAffine<Self::Parameters>;
    type Projective = ProjectiveVesta;
}

/// Rust calls:
/// https://github.com/openmina/mina/blob/8f83199a92faa8ff592b7ae5ad5b3236160e8c20/src/lib/crypto/kimchi_bindings/stubs/src/projective.rs
/// Conversion to/from OCaml:
/// https://github.com/openmina/mina/blob/8f83199a92faa8ff592b7ae5ad5b3236160e8c20/src/lib/crypto/kimchi_bindings/stubs/src/arkworks/group_projective.rs
/// Typ:
/// https://github.com/o1-labs/snarky/blob/7edf13628872081fd7cad154de257dad8b9ba621/snarky_curve/snarky_curve.ml#L219-L229
///
#[derive(
    derive_more::Add, derive_more::Sub, derive_more::Neg, derive_more::Mul, derive_more::Div,
)]
pub struct InnerCurve<C: FieldWitness> {
    // ProjectivePallas
    // ProjectiveVesta
    inner: C::Projective,
}

impl<C: FieldWitness> std::fmt::Debug for InnerCurve<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // OCaml uses `to_affine_exn` when those are printed using `sexp`
        // https://github.com/openmina/mina/blob/8f83199a92faa8ff592b7ae5ad5b3236160e8c20/src/lib/snark_params/snark_params.ml#L149
        let GroupAffine { x, y, .. } = self.to_affine();
        f.debug_struct("InnerCurve")
            .field("x", &x)
            .field("y", &y)
            .finish()
    }
}

impl<C: FieldWitness> InnerCurve<C> {
    fn one() -> Self {
        let inner = C::Projective::prime_subgroup_generator();
        Self { inner }
    }

    fn double(&self) -> Self {
        Self {
            inner: self.inner.double(),
        }
    }

    fn scale<S>(&self, scale: S) -> Self
    where
        S: Into<BigInteger256>,
    {
        let scale: BigInteger256 = scale.into();
        Self {
            inner: self.inner.mul(scale),
        }
    }

    fn to_affine(&self) -> GroupAffine<C::Parameters> {
        // Both `affine` below are the same type, but we use `into()` to make it non-generic
        let affine: C::Affine = self.inner.into_affine();
        let affine: GroupAffine<C::Parameters> = affine.into();
        // OCaml panics on infinity
        // https://github.com/MinaProtocol/mina/blob/3e58e92ea9aeddb41ad3b6e494279891c5f9aa09/src/lib/crypto/kimchi_backend/common/curve.ml#L180
        assert!(!affine.infinity);
        affine
    }

    fn of_affine(affine: GroupAffine<C::Parameters>) -> Self {
        // Both `inner` below are the same type, but we use `into()` to make it generic
        let inner: GroupProjective<C::Parameters> = affine.into_projective();
        let inner: C::Projective = inner.into();
        Self { inner }
    }
}

impl<F: FieldWitness> ToFieldElements<F> for InnerCurve<F> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let GroupAffine { x, y, .. } = self.to_affine();
        fields.push(x);
        fields.push(y);
    }
}

impl<F: FieldWitness> Check<F> for InnerCurve<F> {
    // TODO: This function is incomplete (compare to OCaml), here it only push witness values
    // https://github.com/openmina/mina/blob/8f83199a92faa8ff592b7ae5ad5b3236160e8c20/src/lib/snarky_curves/snarky_curves.ml#L167
    fn check(&self, witnesses: &mut Witness<F>) {
        let GroupAffine { x, y, .. } = self.to_affine();
        let x2 = field::square(x, witnesses);
        let _x3 = field::mul(x2, x, witnesses);
        // TODO
    }
}

mod field {
    use super::*;

    // TODO: This function is incomplete (compare to OCaml), here it only push witness values
    // https://github.com/o1-labs/snarky/blob/7edf13628872081fd7cad154de257dad8b9ba621/src/base/utils.ml#L99
    pub fn square<F>(field: F, witnesses: &mut Witness<F>) -> F
    where
        F: FieldWitness,
    {
        let mut push = witnesses.pusher();
        push(field.square())
    }

    pub fn mul<F>(x: F, y: F, witnesses: &mut Witness<F>) -> F
    where
        F: FieldWitness,
    {
        let mut push = witnesses.pusher();
        push(x * y)
    }
}

// TODO: This function is incomplete (compare to OCaml), here it only push witness values
#[allow(unused)]
fn dummy_constraints<F>(w: &mut Witness<F>)
where
    F: FieldWitness + Check<F>,
{
    let x: F = w.exists(F::from(3u64));
    let g: InnerCurve<F> = w.exists(InnerCurve::<F>::one());

    let _ = w.to_field_checked_prime::<16>(x);

    dbg!(w);

    // dbg!(&g);
    // let a: GroupAffine<F::Parameters> = g.to_affine();

    // let x: F = a.x;
    // let y: F = a.y;

    // let a: ProjectivePallas = ProjectivePallas::prime_subgroup_generator();
    // dbg!(a);

    // // use mina_curves::pasta::Pallas;
    // // let proj = Into::<Pallas>::into(a).into_projective();
    // // dbg!(a);

    // let affine: GroupAffine<PallasParameters> = a.into_affine();
    // dbg!(affine);

    // let b = ProjectiveVesta::prime_subgroup_generator();
    // dbg!(b);
}

// (* Currently, a circuit must have at least 1 of every type of constraint. *)
// let dummy_constraints () =
//   make_checked
//     Impl.(
//       fun () ->
//         let x =
//           exists Field.typ ~compute:(fun () -> Field.Constant.of_int 3)
//         in
//         let g = exists Inner_curve.typ ~compute:(fun _ -> Inner_curve.one) in
//         ignore
//           ( Pickles.Scalar_challenge.to_field_checked'
//               (module Impl)
//               ~num_bits:16
//               (Kimchi_backend_common.Scalar_challenge.create x)
//             : Field.t * Field.t * Field.t ) ;
//         ignore
//           ( Pickles.Step_main_inputs.Ops.scale_fast g ~num_bits:5
//               (Shifted_value x)
//             : Pickles.Step_main_inputs.Inner_curve.t ) ;
//         ignore
//           ( Pickles.Step_main_inputs.Ops.scale_fast g ~num_bits:5
//               (Shifted_value x)
//             : Pickles.Step_main_inputs.Inner_curve.t ) ;
//         ignore
//           ( Pickles.Step_verifier.Scalar_challenge.endo g ~num_bits:4
//               (Kimchi_backend_common.Scalar_challenge.create x)
//             : Field.t * Field.t ))

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use mina_hasher::Fp;
    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use super::*;

    #[test]
    fn test_to_field_checked() {
        let mut witness = Witness::with_capacity(32);
        let f = Fp::from_str("1866").unwrap();

        let res = to_field_checked_prime::<_, 32>(f, &mut witness);

        assert_eq!(res, (131085.into(), 65636.into(), 1866.into()));
        assert_eq!(
            witness.aux,
            &[
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                512.into(),
                257.into(),
                0.into(),
                0.into(),
                1.into(),
                3.into(),
                1.into(),
                0.into(),
                2.into(),
                2.into(),
                1866.into(),
                131085.into(),
                65636.into(),
            ]
        );
    }
}
