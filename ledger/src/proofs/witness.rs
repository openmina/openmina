use std::collections::BTreeMap;

use ark_ec::{
    short_weierstrass_jacobian::{GroupAffine, GroupProjective},
    AffineCurve, ProjectiveCurve, SWModelParameters,
};
use ark_ff::{BigInteger256, Field, FpParameters, PrimeField};
use mina_curves::pasta::{
    Fq, PallasParameters, ProjectivePallas, ProjectiveVesta, VestaParameters,
};
use mina_hasher::Fp;
use mina_p2p_messages::{
    string::ByteString,
    v2::{
        self, ConsensusGlobalSlotStableV1, ConsensusProofOfStakeDataConsensusStateValueStableV2,
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
use mina_signer::CompressedPubKey;

use crate::{
    scan_state::transaction_logic::{
        protocol_state::{EpochData, EpochLedger},
        transaction_union_payload,
    },
    staged_ledger::hash::StagedLedgerHash,
    TokenId,
};

use super::{public_input::plonk_checks::ShiftedValue, to_field_elements::ToFieldElements};

#[derive(Debug)]
pub struct Witness<F: FieldWitness> {
    aux: Vec<F>,
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
            // dbg!(field)
            field
        };
        self.aux.push(field)
    }

    pub fn extend<I: Into<F>, V: Iterator<Item = I>>(&mut self, field: V) {
        let fields = {
            let fields: Vec<F> = field.map(Into::into).collect();
            // eprintln!("extend[{}]={:#?}", fields.len(), fields);
            fields
        };
        self.aux.extend(fields)
    }

    fn exists<T>(&mut self, data: T) -> T
    where
        T: ToFieldElements<F> + Check<F>,
    {
        // data.to_field_elements(&mut self.aux);
        let mut fields = data.to_field_elements_owned();
        eprintln!("w{:?}", &fields);
        self.aux.append(&mut fields);

        data.check(self);
        data
    }

    /// Helper
    pub fn to_field_checked_prime<const NBITS: usize>(&mut self, scalar: F) -> (F, F, F) {
        scalar_challenge::to_field_checked_prime::<F, NBITS>(scalar, self)
    }

    /// Helper
    pub fn add_fast(
        &mut self,
        p1: GroupAffine<F::Parameters>,
        p2: GroupAffine<F::Parameters>,
    ) -> GroupAffine<F::Parameters> {
        add_fast::<F>(p1, p2, None, self)
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

fn bigint_to_bits<const NBITS: usize>(bigint: BigInteger256) -> [bool; NBITS] {
    let mut bits = FieldBitsIterator { index: 0, bigint }.take(NBITS);
    std::array::from_fn(|_| bits.next().unwrap())
}

fn field_to_bits<F, const NBITS: usize>(field: F) -> [bool; NBITS]
where
    F: Field + Into<BigInteger256>,
{
    let bigint: BigInteger256 = field.into();
    bigint_to_bits(bigint)
}

fn bits_msb<F, const NBITS: usize>(field: F) -> [bool; NBITS]
where
    F: Field + Into<BigInteger256>,
{
    let mut bits = field_to_bits::<F, NBITS>(field);
    bits.reverse();
    bits
}

pub fn endos<F>() -> (F, F::Scalar)
where
    F: FieldWitness,
{
    use std::any::TypeId;
    use std::cell::RefCell;

    // Let's keep them in cache since they're used everywhere
    thread_local! {
        static ENDOS: RefCell<BTreeMap<TypeId, [BigInteger256; 2]>> = RefCell::default();
    }

    let type_id = std::any::TypeId::of::<F>();

    ENDOS.with(|cache| {
        let mut cache = cache.borrow_mut();

        if let Some([base, scalar]) = cache.get(&type_id).copied() {
            return (base.into(), scalar.into());
        };

        let (base, scalar) = poly_commitment::srs::endos::<GroupAffine<F::Parameters>>();
        cache.insert(type_id, [base.into(), scalar.into()]);

        (base, scalar)
    })
}

fn make_group<F>(x: F, y: F) -> GroupAffine<F::Parameters>
where
    F: FieldWitness,
{
    GroupAffine::<F::Parameters>::new(x, y, false)
}

pub mod scalar_challenge {
    use super::*;

    // https://github.com/MinaProtocol/mina/blob/357144819e7ce5f61109d23d33da627be28024c7/src/lib/pickles/scalar_challenge.ml#L12
    pub fn to_field_checked_prime<F, const NBITS: usize>(scalar: F, w: &mut Witness<F>) -> (F, F, F)
    where
        F: FieldWitness,
    {
        let zero = F::zero();
        let one = F::one();
        let neg_one = one.neg();

        let a_array = [zero, zero, neg_one, one];
        let a_func = |n: u64| a_array[n as usize];

        let b_array = [neg_one, one, zero, zero];
        let b_func = |n: u64| b_array[n as usize];

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
                .map(|j| w.exists(F::from(nybbles_by_row[i][j])))
                .collect();

            let n8: F = w.exists(xs.iter().fold(n0, |accum, x| accum.double().double() + x));

            let a8: F = w.exists(
                nybbles_by_row[i]
                    .iter()
                    .fold(a0, |accum, x| accum.double() + a_func(*x)),
            );

            let b8: F = w.exists(
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

    pub fn endo<F, const NBITS: usize>(
        t: GroupAffine<F::Parameters>,
        scalar: F,
        w: &mut Witness<F>,
    ) -> GroupAffine<F::Parameters>
    where
        F: FieldWitness,
    {
        let bits: [bool; NBITS] = bits_msb::<F, NBITS>(scalar);

        let bits_per_row = 4;
        let rows = NBITS / bits_per_row;

        let GroupAffine { x: xt, y: yt, .. } = t;
        let (endo_base, _) = endos::<F>();

        let mut acc = {
            // The `exists` call is made by the `seal` call in OCaml
            let tmp = w.exists(xt * endo_base);
            let p = w.add_fast(t, make_group::<F>(tmp, yt));
            w.add_fast(p, p)
        };

        let mut n_acc = F::zero();
        for i in 0..rows {
            let n_acc_prev = n_acc;
            let b1 = w.exists(F::from(bits[i * bits_per_row]));
            let b2 = w.exists(F::from(bits[(i * bits_per_row) + 1]));
            let b3 = w.exists(F::from(bits[(i * bits_per_row) + 2]));
            let b4 = w.exists(F::from(bits[(i * bits_per_row) + 3]));

            let GroupAffine { x: xp, y: yp, .. } = acc;
            let xq1 = w.exists((F::one() + ((endo_base - F::one()) * b1)) * xt);
            let yq1 = w.exists((b2.double() - F::one()) * yt);
            let s1 = w.exists((yq1 - yp) / (xq1 - xp));
            let s1_squared = w.exists(s1.square());
            let s2 = w.exists((yp.double() / (xp.double() + xq1 - s1_squared)) - s1);
            let xr = w.exists(xq1 + s2.square() - s1_squared);
            let yr = w.exists(((xp - xr) * s2) - yp);
            let xq2 = w.exists((F::one() + ((endo_base - F::one()) * b3)) * xt);
            let yq2 = w.exists((b4.double() - F::one()) * yt);
            let s3 = w.exists((yq2 - yr) / (xq2 - xr));
            let s3_squared = w.exists(s3.square());
            let s4 = w.exists((yr.double() / (xr.double() + xq2 - s3_squared)) - s3);
            let xs = w.exists(xq2 + s4.square() - s3_squared);
            let ys = w.exists(((xr - xs) * s4) - yr);

            acc = make_group::<F>(xs, ys);
            n_acc =
                w.exists((((n_acc_prev.double() + b1).double() + b2).double() + b3).double() + b4);
        }

        acc
    }
}

fn add_fast<F>(
    p1: GroupAffine<F::Parameters>,
    p2: GroupAffine<F::Parameters>,
    check_finite: Option<bool>,
    w: &mut Witness<F>,
) -> GroupAffine<F::Parameters>
where
    F: FieldWitness,
{
    let GroupAffine { x: x1, y: y1, .. } = p1;
    let GroupAffine { x: x2, y: y2, .. } = p2;
    let check_finite = check_finite.unwrap_or(true);

    let bool_to_field = |b: bool| if b { F::one() } else { F::zero() };

    let same_x_bool = x1 == x2;
    let _same_x = w.exists(bool_to_field(same_x_bool));

    let _inf = if check_finite {
        F::zero()
    } else {
        w.exists(bool_to_field(same_x_bool && y1 != y2))
    };

    let _inf_z = w.exists({
        if y1 == y2 {
            F::zero()
        } else if same_x_bool {
            (y2 - y1).inverse().unwrap()
        } else {
            F::zero()
        }
    });

    let _x21_inv = w.exists({
        if same_x_bool {
            F::zero()
        } else {
            (x2 - x1).inverse().unwrap()
        }
    });

    let s = w.exists({
        if same_x_bool {
            let x1_squared = x1.square();
            (x1_squared + x1_squared + x1_squared) / (y1 + y1)
        } else {
            (y2 - y1) / (x2 - x1)
        }
    });

    let x3 = w.exists(s.square() - (x1 + x2));
    let y3 = w.exists(s * (x1 - x3) - y1);

    make_group::<F>(x3, y3)
}

fn fold_map<T, Acc, U>(
    iter: impl Iterator<Item = T>,
    init: Acc,
    mut fun: impl FnMut(Acc, T) -> (Acc, U),
) -> (Acc, Vec<U>) {
    let mut acc = Some(init);
    let result = iter
        .map(|x| {
            let (new_acc, y) = fun(acc.take().unwrap(), x);
            acc = Some(new_acc);
            y
        })
        .collect::<Vec<_>>();
    (acc.unwrap(), result)
}

mod plonk_curve_ops {
    use super::*;

    const BITS_PER_CHUNK: usize = 5;

    pub fn scale_fast<F, const NBITS: usize>(
        base: GroupAffine<F::Parameters>,
        shifted_value: ShiftedValue<F>,
        w: &mut Witness<F>,
    ) where
        F: FieldWitness,
    {
        scale_fast_unpack::<F, NBITS>(base, shifted_value, w)
    }

    // https://github.com/openmina/mina/blob/8f83199a92faa8ff592b7ae5ad5b3236160e8c20/src/lib/pickles/plonk_curve_ops.ml#L140
    fn scale_fast_unpack<F, const NBITS: usize>(
        base: GroupAffine<F::Parameters>,
        ShiftedValue { shifted: scalar }: ShiftedValue<F>,
        w: &mut Witness<F>,
    ) where
        F: FieldWitness,
    {
        let GroupAffine {
            x: x_base,
            y: y_base,
            ..
        } = base;

        let chunks: usize = NBITS / BITS_PER_CHUNK;
        assert_eq!(NBITS % BITS_PER_CHUNK, 0);

        let bits_msb: [bool; NBITS] = bits_msb::<F, NBITS>(scalar);
        let acc = w.add_fast(base, base);
        let mut n_acc = F::zero();

        for chunk in 0..chunks {
            let bs: [bool; BITS_PER_CHUNK] =
                std::array::from_fn(|i| bits_msb[(chunk * BITS_PER_CHUNK) + i]);

            let n_acc_prev = n_acc;

            n_acc = w.exists(
                bs.iter()
                    .fold(n_acc_prev, |acc, b| acc.double() + F::from(*b)),
            );

            let _ = fold_map(bs.iter(), acc, |acc, b| {
                let GroupAffine {
                    x: x_acc, y: y_acc, ..
                } = acc;
                let b: F = F::from(*b);

                let s1 = w.exists((y_acc - (y_base * (b.double() - F::one()))) / (x_acc - x_base));
                let s1_squared = w.exists(s1.square());
                let s2 = w.exists((y_acc.double() / (x_acc.double() + x_base - s1_squared)) - s1);

                let x_res = w.exists(x_base + s2.square() - s1_squared);
                let y_res = w.exists(((x_acc - x_res) * s2) - y_acc);
                let acc = make_group(x_res, y_res);

                (acc, (acc, s1))
            });

            // TODO: Rest of the code doesn't touch the witness
        }
    }
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

impl<F: FieldWitness> ToFieldElements<F> for StagedLedgerHash<F> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self {
            non_snark,
            pending_coinbase_hash,
        } = self;

        let non_snark_digest = non_snark.digest();

        const BITS: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];
        fields.extend(
            non_snark_digest
                .iter()
                .flat_map(|byte| BITS.iter().map(|bit| F::from((*byte & bit != 0) as u64))),
        );

        fields.push(*pending_coinbase_hash);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for ByteString {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let slice: &[u8] = self;
        slice.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for &'_ [u8] {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        const BITS: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];
        fields.extend(
            self.iter()
                .flat_map(|byte| BITS.iter().map(|bit| F::from((*byte & bit != 0) as u64))),
        );
    }
}

impl<F: FieldWitness> ToFieldElements<F> for &'_ [bool] {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        fields.reserve(self.len());
        fields.extend(self.iter().copied().map(F::from))
    }
}

impl<F: FieldWitness, const NBITS: usize> ToFieldElements<F> for &'_ [bool; NBITS] {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        fields.reserve(self.len());
        fields.extend(self.iter().copied().map(F::from))
    }
}

impl<F: FieldWitness, const NBITS: usize> ToFieldElements<F> for [bool; NBITS] {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        fields.reserve(self.len());
        fields.extend(self.iter().copied().map(F::from))
    }
}

impl<F: FieldWitness> ToFieldElements<F> for EpochData<F> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
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

impl<F: FieldWitness> ToFieldElements<F> for NonZeroCurvePoint {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let NonZeroCurvePointUncompressedStableV1 { x, is_odd } = self.inner();

        fields.push(x.to_field());
        fields.push((*is_odd).into());
    }
}

impl<F: FieldWitness> ToFieldElements<F> for ConsensusProofOfStakeDataConsensusStateValueStableV2 {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
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

        let staking_epoch_data: EpochData<F> = staking_epoch_data.into();
        let next_epoch_data: EpochData<F> = next_epoch_data.into();

        fields.push(blockchain_length.as_u32().into());
        fields.push(epoch_count.as_u32().into());
        fields.push(min_window_density.as_u32().into());
        fields.extend(sub_window_densities.iter().map(|w| F::from(w.as_u32())));

        {
            let vrf: &[u8] = last_vrf_output.as_ref();
            (&vrf[..31]).to_field_elements(fields);
            // Ignore the last 3 bits
            let last_byte = vrf[31];
            for bit in [1, 2, 4, 8, 16] {
                fields.push(F::from(last_byte & bit != 0))
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

impl<F: FieldWitness> ToFieldElements<F> for MinaBaseProtocolConstantsCheckedValueStableV1 {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
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

impl<F: FieldWitness> ToFieldElements<F> for MinaStateBlockchainStateValueStableV2 {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self {
            staged_ledger_hash,
            genesis_ledger_hash,
            ledger_proof_statement,
            timestamp,
            body_reference,
        } = self;

        let staged_ledger_hash: StagedLedgerHash<F> = staged_ledger_hash.into();

        staged_ledger_hash.to_field_elements(fields);
        fields.push(genesis_ledger_hash.inner().to_field());
        ledger_proof_statement.to_field_elements(fields);
        fields.push(timestamp.as_u64().into());
        body_reference.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for MinaStateProtocolStateBodyValueStableV2 {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
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

impl ToFieldElements<Fp> for TokenId {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self(token_id) = self;
        fields.push(*token_id);
    }
}

impl ToFieldElements<Fp> for CompressedPubKey {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self { x, is_odd } = self;
        fields.push(*x);
        fields.push(Fp::from(*is_odd));
    }
}

impl ToFieldElements<Fp> for mina_signer::Signature {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self { rx, s } = self;

        fields.push(*rx);
        let bits = field_to_bits::<_, 255>(*s);
        bits.to_field_elements(fields);
    }
}

impl ToFieldElements<Fp> for mina_signer::PubKey {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let GroupAffine { x, y, .. } = self.point();
        fields.push(*x);
        fields.push(*y);
    }
}

impl ToFieldElements<Fp> for transaction_union_payload::TransactionUnion {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        use transaction_union_payload::{Body, Common, TransactionUnionPayload};

        let Self {
            payload:
                TransactionUnionPayload {
                    common:
                        Common {
                            fee,
                            fee_token,
                            fee_payer_pk,
                            nonce,
                            valid_until,
                            memo,
                        },
                    body:
                        Body {
                            tag,
                            source_pk,
                            receiver_pk,
                            token_id,
                            amount,
                        },
                },
            signer,
            signature,
        } = self;

        fields.push(fee.as_u64().into());
        fee_token.to_field_elements(fields);
        fee_payer_pk.to_field_elements(fields);
        fields.push(nonce.as_u32().into());
        fields.push(valid_until.as_u32().into());
        memo.as_slice().to_field_elements(fields);
        tag.to_untagged_bits().to_field_elements(fields);
        source_pk.to_field_elements(fields);
        receiver_pk.to_field_elements(fields);
        token_id.to_field_elements(fields);
        fields.push(amount.as_u64().into());
        signer.to_field_elements(fields);
        signature.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for MinaNumbersGlobalSlotSinceGenesisMStableV1 {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        fields.push(self.as_u32().into())
    }
}

impl<F: FieldWitness> ToFieldElements<F> for v2::MinaBasePendingCoinbaseStackVersionedStableV1 {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self {
            data,
            state: v2::MinaBasePendingCoinbaseStateStackStableV1 { init, curr },
        } = self;

        fields.push(data.to_field());
        fields.push(init.to_field());
        fields.push(curr.to_field());
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
        scalar_challenge::to_field_checked_prime::<F, NBITS>(amount, witnesses);
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
        scalar_challenge::to_field_checked_prime::<F, NBITS>(number, witnesses);
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
        scalar_challenge::to_field_checked_prime::<F, NBITS>(number, witnesses);
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
    Self: Field
        + Send
        + Sync
        + Into<BigInteger256>
        + From<BigInteger256>
        + Into<mina_p2p_messages::bigint::BigInt>
        + From<BigInteger256>
        + From<i64>
        + ToFieldElements<Self>
        + Check<Self>
        + FromFpFq
        + PrimeField
        + std::fmt::Debug
        + 'static,
{
    type Scalar: FieldWitness;
    type Affine: AffineCurve<Projective = Self::Projective, BaseField = Self, ScalarField = Self::Scalar>
        + Into<GroupAffine<Self::Parameters>>
        + std::fmt::Debug;
    type Projective: ProjectiveCurve<Affine = Self::Affine, BaseField = Self, ScalarField = Self::Scalar>
        + From<GroupProjective<Self::Parameters>>
        + std::fmt::Debug;
    type Parameters: SWModelParameters<BaseField = Self, ScalarField = Self::Scalar>
        + Clone
        + std::fmt::Debug;
    const PARAMS: Params<Self>;
    const SIZE: BigInteger256;
}

pub struct Params<F> {
    a: F,
    b: F,
}

impl FieldWitness for Fp {
    type Scalar = Fq;
    type Parameters = PallasParameters;
    type Affine = GroupAffine<Self::Parameters>;
    type Projective = ProjectivePallas;

    /// https://github.com/openmina/mina/blob/46b6403cb7f158b66a60fc472da2db043ace2910/src/lib/crypto/kimchi_backend/pasta/basic/kimchi_pasta_basic.ml#L107
    const PARAMS: Params<Self> = Params::<Self> {
        a: ark_ff::field_new!(Fp, "0"),
        b: ark_ff::field_new!(Fp, "5"),
    };
    const SIZE: BigInteger256 = mina_curves::pasta::fields::FpParameters::MODULUS;
}

impl FieldWitness for Fq {
    type Scalar = Fp;
    type Parameters = VestaParameters;
    type Affine = GroupAffine<Self::Parameters>;
    type Projective = ProjectiveVesta;

    /// https://github.com/openmina/mina/blob/46b6403cb7f158b66a60fc472da2db043ace2910/src/lib/crypto/kimchi_backend/pasta/basic/kimchi_pasta_basic.ml#L95
    const PARAMS: Params<Self> = Params::<Self> {
        a: ark_ff::field_new!(Fq, "0"),
        b: ark_ff::field_new!(Fq, "5"),
    };
    const SIZE: BigInteger256 = mina_curves::pasta::fields::FqParameters::MODULUS;
}

/// Trait helping converting generics into concrete types
pub trait FromFpFq {
    fn from_fp(fp: Fp) -> Self;
    fn from_fq(fp: Fq) -> Self;
}

impl FromFpFq for Fp {
    fn from_fp(fp: Fp) -> Self {
        fp
    }
    fn from_fq(fq: Fq) -> Self {
        let bigint: BigInteger256 = fq.into();
        bigint.into()
    }
}

impl FromFpFq for Fq {
    fn from_fp(fp: Fp) -> Self {
        let bigint: BigInteger256 = fp.into();
        bigint.into()
    }
    fn from_fq(fq: Fq) -> Self {
        fq
    }
}

/// Trait helping converting concrete types into generics
pub trait IntoGeneric<F: FieldWitness> {
    fn into_gen(self) -> F;
}

impl<F: FieldWitness> IntoGeneric<F> for Fp {
    fn into_gen(self) -> F {
        F::from_fp(self)
    }
}

impl<F: FieldWitness> IntoGeneric<F> for Fq {
    fn into_gen(self) -> F {
        F::from_fq(self)
    }
}

/// Rust calls:
/// https://github.com/openmina/mina/blob/8f83199a92faa8ff592b7ae5ad5b3236160e8c20/src/lib/crypto/kimchi_bindings/stubs/src/projective.rs
/// Conversion to/from OCaml:
/// https://github.com/openmina/mina/blob/8f83199a92faa8ff592b7ae5ad5b3236160e8c20/src/lib/crypto/kimchi_bindings/stubs/src/arkworks/group_projective.rs
/// Typ:
/// https://github.com/o1-labs/snarky/blob/7edf13628872081fd7cad154de257dad8b9ba621/snarky_curve/snarky_curve.ml#L219-L229
///
#[derive(
    Clone, derive_more::Add, derive_more::Sub, derive_more::Neg, derive_more::Mul, derive_more::Div,
)]
pub struct InnerCurve<F: FieldWitness> {
    // ProjectivePallas
    // ProjectiveVesta
    inner: F::Projective,
}

impl<F: FieldWitness> std::fmt::Debug for InnerCurve<F> {
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

impl<F: FieldWitness> InnerCurve<F> {
    fn one() -> Self {
        let inner = F::Projective::prime_subgroup_generator();
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

    fn add_fast(&self, other: Self, w: &mut Witness<F>) -> Self {
        let result = w.add_fast(self.to_affine(), other.to_affine());
        Self::of_affine(result)
    }

    fn to_affine(&self) -> GroupAffine<F::Parameters> {
        // Both `affine` below are the same type, but we use `into()` to make it non-generic
        let affine: F::Affine = self.inner.into_affine();
        let affine: GroupAffine<F::Parameters> = affine.into();
        // OCaml panics on infinity
        // https://github.com/MinaProtocol/mina/blob/3e58e92ea9aeddb41ad3b6e494279891c5f9aa09/src/lib/crypto/kimchi_backend/common/curve.ml#L180
        assert!(!affine.infinity);
        affine
    }

    fn of_affine(affine: GroupAffine<F::Parameters>) -> Self {
        // Both `inner` below are the same type, but we use `into()` to make it generic
        let inner: GroupProjective<F::Parameters> = affine.into_projective();
        let inner: F::Projective = inner.into();
        Self { inner }
    }

    fn fake_random() -> Self {
        static SEED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

        let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(
            SEED.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        );
        let proj: GroupProjective<F::Parameters> = ark_ff::UniformRand::rand(&mut rng);
        let proj: F::Projective = proj.into();
        Self { inner: proj }
    }

    fn random() -> Self {
        Self::fake_random()
        // // Both `proj` below are the same type, but we use `into()` to make it generic
        // let rng = &mut rand::rngs::OsRng;
        // let proj: GroupProjective<F::Parameters> = ark_ff::UniformRand::rand(rng);
        // let proj: F::Projective = proj.into();

        // Self { inner: proj }
    }
}

/// https://github.com/openmina/mina/blob/45c195d72aa8308fcd9fc1c7bc5da36a0c3c3741/src/lib/snarky_curves/snarky_curves.ml#L267
fn create_shifted_inner_curve<F>(w: &mut Witness<F>) -> InnerCurve<F>
where
    F: FieldWitness,
{
    w.exists(InnerCurve::<F>::random())
}

impl<F: FieldWitness> ToFieldElements<F> for InnerCurve<F> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let GroupAffine { x, y, .. } = self.to_affine();
        fields.push(x);
        fields.push(y);
    }
}

impl<F: FieldWitness> Check<F> for InnerCurve<F> {
    // https://github.com/openmina/mina/blob/8f83199a92faa8ff592b7ae5ad5b3236160e8c20/src/lib/snarky_curves/snarky_curves.ml#L167
    fn check(&self, witnesses: &mut Witness<F>) {
        let GroupAffine { x, y: _, .. } = self.to_affine();
        let x2 = field::square(x, witnesses);
        let _x3 = field::mul(x2, x, witnesses);
        // TODO: Rest of the function doesn't modify witness
    }
}

impl<F: FieldWitness> Check<F> for transaction_union_payload::Tag {
    fn check(&self, _witnesses: &mut Witness<F>) {
        // Does not modify the witness
        // Note: For constraints we need to convert to unpacked union
        // https://github.com/openmina/mina/blob/45c195d72aa8308fcd9fc1c7bc5da36a0c3c3741/src/lib/mina_base/transaction_union_tag.ml#L177
    }
}

impl<F: FieldWitness> Check<F> for transaction_union_payload::TransactionUnion {
    fn check(&self, witnesses: &mut Witness<F>) {
        use transaction_union_payload::{Body, Common, TransactionUnionPayload};

        let Self {
            payload:
                TransactionUnionPayload {
                    common:
                        Common {
                            fee,
                            fee_token: _,
                            fee_payer_pk: _,
                            nonce,
                            valid_until,
                            memo: _,
                        },
                    body:
                        Body {
                            tag,
                            source_pk: _,
                            receiver_pk: _,
                            token_id: _,
                            amount,
                        },
                },
            signer: _,
            signature: _,
        } = self;

        fee.check(witnesses);
        nonce.check(witnesses);
        valid_until.check(witnesses);
        tag.check(witnesses);
        amount.check(witnesses);
    }
}

impl<F: FieldWitness> Check<F> for v2::MinaBasePendingCoinbaseStackVersionedStableV1 {
    fn check(&self, _witnesses: &mut Witness<F>) {
        // Does not modify the witness
    }
}

impl<F: FieldWitness, const NBITS: usize> Check<F> for [bool; NBITS] {
    fn check(&self, _witnesses: &mut Witness<F>) {
        // Does not modify the witness
    }
}

mod field {
    use super::*;

    // https://github.com/o1-labs/snarky/blob/7edf13628872081fd7cad154de257dad8b9ba621/src/base/utils.ml#L99
    pub fn square<F>(field: F, w: &mut Witness<F>) -> F
    where
        F: FieldWitness,
    {
        w.exists(field.square())
        // TODO: Rest of the function doesn't modify witness
    }

    pub fn mul<F>(x: F, y: F, w: &mut Witness<F>) -> F
    where
        F: FieldWitness,
    {
        w.exists(x * y)
    }
}

#[allow(unused)]
fn dummy_constraints<F>(w: &mut Witness<F>)
where
    F: FieldWitness,
{
    let x: F = w.exists(F::from(3u64));
    let g: InnerCurve<F> = w.exists(InnerCurve::<F>::one());

    let _ = w.to_field_checked_prime::<16>(x);

    plonk_curve_ops::scale_fast::<F, 5>(g.to_affine(), ShiftedValue { shifted: x }, w);
    plonk_curve_ops::scale_fast::<F, 5>(g.to_affine(), ShiftedValue { shifted: x }, w);
    scalar_challenge::endo::<F, 4>(g.to_affine(), x, w);

    // dbg!(w);
}

pub mod legacy_input {
    use crate::scan_state::transaction_logic::transaction_union_payload::{
        Body, Common, TransactionUnionPayload,
    };

    use super::*;

    pub struct BitsIterator<const N: usize> {
        pub index: usize,
        pub number: [u8; N],
    }

    impl<const N: usize> Iterator for BitsIterator<N> {
        type Item = bool;

        fn next(&mut self) -> Option<Self::Item> {
            let index = self.index;
            self.index += 1;

            let limb_index = index / 8;
            let bit_index = index % 8;

            let limb = self.number.get(limb_index)?;
            Some(limb & (1 << bit_index) != 0)
        }
    }

    pub fn bits_iter<N: Into<u64>, const NBITS: usize>(number: N) -> impl Iterator<Item = bool> {
        let number: u64 = number.into();
        BitsIterator {
            index: 0,
            number: number.to_ne_bytes(),
        }
        .take(NBITS)
    }

    pub fn to_bits<N: Into<u64>, const NBITS: usize>(number: N) -> [bool; NBITS] {
        let mut iter = bits_iter::<N, NBITS>(number);
        std::array::from_fn(|_| iter.next().unwrap())
    }

    pub trait CheckedLegacyInput<F: FieldWitness> {
        fn to_checked_legacy_input(&self, inputs: &mut LegacyInput<F>, w: &mut Witness<F>);

        fn to_checked_legacy_input_owned(&self, w: &mut Witness<F>) -> LegacyInput<F> {
            let mut inputs = LegacyInput::new();
            self.to_checked_legacy_input(&mut inputs, w);
            inputs
        }
    }

    #[derive(Clone, Debug)]
    pub struct LegacyInput<F: FieldWitness> {
        fields: Vec<F>,
        bits: Vec<bool>,
    }

    impl<F: FieldWitness> LegacyInput<F> {
        pub fn new() -> Self {
            Self {
                fields: Vec::with_capacity(256),
                bits: Vec::with_capacity(1024),
            }
        }

        pub fn append_bit(&mut self, bit: bool) {
            self.bits.push(bit);
        }

        pub fn append_bits(&mut self, bits: &[bool]) {
            self.bits.extend(bits);
        }

        pub fn append_field(&mut self, field: F) {
            self.fields.push(field);
        }

        pub fn to_fields(mut self) -> Vec<F> {
            const NBITS: usize = 255 - 1;

            self.fields.reserve(self.bits.len() / NBITS);
            self.fields.extend(self.bits.chunks(NBITS).map(|bits| {
                assert!(bits.len() <= NBITS);

                let mut field = [0u64; 4];

                for (index, bit) in bits.iter().enumerate() {
                    let limb_index = index / 64;
                    let bit_index = index % 64;
                    field[limb_index] |= (*bit as u64) << bit_index;
                }

                F::from(BigInteger256::new(field))
            }));
            self.fields
        }
    }

    const LEGACY_DEFAULT_TOKEN: [bool; 64] = {
        let mut default = [false; 64];
        default[0] = true;
        default
    };

    impl<F: FieldWitness> CheckedLegacyInput<F> for TransactionUnionPayload {
        fn to_checked_legacy_input(&self, inputs: &mut LegacyInput<F>, w: &mut Witness<F>) {
            let Self {
                common:
                    Common {
                        fee,
                        fee_payer_pk,
                        nonce,
                        valid_until,
                        memo,
                        fee_token: _,
                    },
                body:
                    Body {
                        tag,
                        source_pk,
                        receiver_pk,
                        token_id: _,
                        amount,
                    },
            } = self;

            let fee_token = &LEGACY_DEFAULT_TOKEN;

            // Common
            let nonce = w.exists(nonce.to_bits());
            let valid_until = w.exists(valid_until.to_bits());
            let fee = w.exists(fee.to_bits());
            inputs.append_bits(&fee);
            inputs.append_bits(fee_token);
            inputs.append_field(fee_payer_pk.x.into_gen());
            inputs.append_bit(fee_payer_pk.is_odd);
            inputs.append_bits(&nonce);
            inputs.append_bits(&valid_until);
            inputs.append_bits(&memo.to_bits());

            // Body
            let amount = w.exists(amount.to_bits());
            inputs.append_bits(&tag.to_bits());
            inputs.append_field(source_pk.x.into_gen());
            inputs.append_bit(source_pk.is_odd);
            inputs.append_field(receiver_pk.x.into_gen());
            inputs.append_bit(receiver_pk.is_odd);
            inputs.append_bits(fee_token);
            inputs.append_bits(&amount);
            inputs.append_bit(false);
        }
    }
}

pub mod poseidon {
    use std::marker::PhantomData;

    use mina_poseidon::constants::SpongeConstants;
    use mina_poseidon::poseidon::{ArithmeticSpongeParams, SpongeState};

    use super::*;

    pub struct Sponge<F: FieldWitness, C: SpongeConstants> {
        state: [F; 3],
        sponge_state: SpongeState,
        params: &'static ArithmeticSpongeParams<F>,
        _constants: PhantomData<C>,
    }

    impl<F, C> Sponge<F, C>
    where
        F: FieldWitness,
        C: SpongeConstants,
    {
        pub fn new_with_state(state: [F; 3], params: &'static ArithmeticSpongeParams<F>) -> Self {
            Self {
                state,
                sponge_state: SpongeState::Absorbed(0),
                params,
                _constants: PhantomData,
            }
        }

        pub fn new(params: &'static ArithmeticSpongeParams<F>) -> Self {
            Self::new_with_state([F::zero(); 3], params)
        }

        pub fn absorb(&mut self, x: &[F], w: &mut Witness<F>) {
            // Hack to know when to ignore witness
            // That should be removed once we use `cvar`
            let mut first = true;

            for x in x.iter() {
                match self.sponge_state {
                    SpongeState::Absorbed(n) => {
                        if n == C::SPONGE_RATE {
                            self.poseidon_block_cipher(first, w);
                            self.sponge_state = SpongeState::Absorbed(1);
                            self.state[0].add_assign(x);
                            w.exists(self.state[0]); // Good
                            first = false;
                        } else {
                            self.sponge_state = SpongeState::Absorbed(n + 1);
                            self.state[n].add_assign(x);
                            w.exists(self.state[n]); // Good
                        }
                    }
                    SpongeState::Squeezed(_n) => {
                        self.state[0].add_assign(x);
                        w.exists(self.state[0]); // Unknown
                        self.sponge_state = SpongeState::Absorbed(1);
                    }
                }
            }
        }

        pub fn squeeze(&mut self, w: &mut Witness<F>) -> F {
            match self.sponge_state {
                SpongeState::Squeezed(n) => {
                    if n == C::SPONGE_RATE {
                        self.poseidon_block_cipher(false, w);
                        self.sponge_state = SpongeState::Squeezed(1);
                        self.state[0]
                    } else {
                        self.sponge_state = SpongeState::Squeezed(n + 1);
                        self.state[n]
                    }
                }
                SpongeState::Absorbed(_n) => {
                    self.poseidon_block_cipher(false, w);
                    self.sponge_state = SpongeState::Squeezed(1);
                    self.state[0]
                }
            }
        }

        pub fn poseidon_block_cipher(&mut self, first: bool, w: &mut Witness<F>) {
            if C::PERM_HALF_ROUNDS_FULL == 0 {
                if C::PERM_INITIAL_ARK {
                    for (i, x) in self.params.round_constants[0].iter().enumerate() {
                        self.state[i].add_assign(x);
                    }
                    w.exists(self.state[0]); // Good
                    w.exists(self.state[1]); // Good
                    if !first {
                        w.exists(self.state[2]); // Good
                    }
                    // dbg!(&state, &params.round_constants[0]);
                    for r in 0..C::PERM_ROUNDS_FULL {
                        self.full_round(r + 1, first && r == 0, w);
                    }
                } else {
                    for r in 0..C::PERM_ROUNDS_FULL {
                        self.full_round(r, first, w);
                    }
                }
            } else {
                unimplemented!()
            }
        }

        pub fn full_round(&mut self, r: usize, first: bool, w: &mut Witness<F>) {
            for (index, state_i) in self.state.iter_mut().enumerate() {
                let push_witness = !(first && index == 2);
                *state_i = sbox::<F>(*state_i, push_witness, w);
            }
            self.state = apply_mds_matrix::<F, C>(self.params, &self.state);
            for (i, x) in self.params.round_constants[r].iter().enumerate() {
                self.state[i].add_assign(x);
                w.exists(self.state[i]); // Good
            }
        }
    }

    pub fn sbox<F: FieldWitness>(x: F, push_witness: bool, w: &mut Witness<F>) -> F {
        let res = x;
        let res = res * res;
        if push_witness {
            w.exists(res); // Good
        }
        let res = res * res;
        if push_witness {
            w.exists(res); // Good
        }
        let res = res * x;
        if push_witness {
            w.exists(res); // Good
        }
        res
    }

    fn apply_mds_matrix<F: Field, C: SpongeConstants>(
        params: &ArithmeticSpongeParams<F>,
        state: &[F; 3],
    ) -> [F; 3] {
        if C::PERM_FULL_MDS {
            std::array::from_fn(|i| {
                state
                    .iter()
                    .zip(params.mds[i].iter())
                    .fold(F::zero(), |x, (s, &m)| m * s + x)
            })
        } else {
            [
                state[0] + state[2],
                state[0] + state[1],
                state[1] + state[2],
            ]
        }
    }
}

fn double_group<F: FieldWitness>(
    group: GroupAffine<F::Parameters>,
    w: &mut Witness<F>,
) -> GroupAffine<F::Parameters> {
    let GroupAffine { x: ax, y: ay, .. } = group;
    let ax: F = ax;
    let ay: F = ay;

    let x_squared = w.exists(ax.square());
    let lambda = w.exists({
        (x_squared + x_squared + x_squared + F::PARAMS.a) * (ay + ay).inverse().unwrap()
    });
    let bx = w.exists(lambda.square() - (ax + ax));
    let by = w.exists((lambda * (ax - bx)) - ay);

    make_group(bx, by)
}

// Used as the _if method
fn group_to_witness<F: FieldWitness>(
    group: GroupAffine<F::Parameters>,
    w: &mut Witness<F>,
) -> GroupAffine<F::Parameters> {
    // We don't want to call `GroupAffine::check` here
    let GroupAffine { x, y, .. } = &group;
    w.exists(*x);
    w.exists(*y);
    group
}

fn scale_non_constant<F: FieldWitness, const N: usize>(
    mut g: GroupAffine<F::Parameters>,
    bits: &[bool; N],
    init: &InnerCurve<F>,
    w: &mut Witness<F>,
) -> GroupAffine<F::Parameters> {
    let mut acc = init.to_affine();

    for b in bits {
        acc = {
            let add_pt = w.add_fast(acc, g);
            let dont_add_pt = acc;
            if *b {
                group_to_witness(add_pt, w)
            } else {
                group_to_witness(dont_add_pt, w)
            }
        };
        g = double_group(g, w);
    }

    acc
}

fn lookup_point<F: FieldWitness>(
    (b0, b1): (bool, bool),
    (t1, t2, t3, t4): (InnerCurve<F>, InnerCurve<F>, InnerCurve<F>, InnerCurve<F>),
    w: &mut Witness<F>,
) -> (F, F) {
    // This doesn't push to the witness, except for the `b0_and_b1`

    let b0_and_b1 = w.exists(F::from(b0 && b1));
    let b0 = F::from(b0);
    let b1 = F::from(b1);
    let lookup_one = |a1: F, a2: F, a3: F, a4: F| -> F {
        a1 + ((a2 - a1) * b0) + ((a3 - a1) * b1) + ((a4 + a1 - a2 - a3) * b0_and_b1)
    };
    let GroupAffine { x: x1, y: y1, .. } = t1.to_affine();
    let GroupAffine { x: x2, y: y2, .. } = t2.to_affine();
    let GroupAffine { x: x3, y: y3, .. } = t3.to_affine();
    let GroupAffine { x: x4, y: y4, .. } = t4.to_affine();

    (lookup_one(x1, x2, x3, x4), lookup_one(y1, y2, y3, y4))
}

fn lookup_single_bit<F: FieldWitness>(b: bool, (t1, t2): (InnerCurve<F>, InnerCurve<F>)) -> (F, F) {
    let lookup_one = |a1: F, a2: F| a1 + (F::from(b) * (a2 - a1));

    let GroupAffine { x: x1, y: y1, .. } = t1.to_affine();
    let GroupAffine { x: x2, y: y2, .. } = t2.to_affine();

    (lookup_one(x1, x2), lookup_one(y1, y2))
}

fn scale_known<F: FieldWitness, const N: usize>(
    t: GroupAffine<F::Parameters>,
    bits: &[bool; N],
    init: &InnerCurve<F>,
    w: &mut Witness<F>,
) -> GroupAffine<F::Parameters> {
    let sigma = InnerCurve::of_affine(t);
    let n = bits.len();
    let sigma_count = (n + 1) / 2;

    let to_term = |two_to_the_i: InnerCurve<F>,
                   two_to_the_i_plus_1: InnerCurve<F>,
                   bits: (bool, bool),
                   w: &mut Witness<F>| {
        let sigma0 = sigma.clone();
        let sigma1 = sigma.clone();
        let sigma2 = sigma.clone();
        let sigma3 = sigma.clone();
        lookup_point(
            bits,
            (
                sigma0,
                (sigma1 + two_to_the_i.clone()),
                (sigma2 + two_to_the_i_plus_1.clone()),
                (sigma3 + two_to_the_i + two_to_the_i_plus_1),
            ),
            w,
        )
    };

    let mut acc = init.to_affine();
    let mut two_to_the_i = sigma.clone();
    for chunk in bits.chunks(2) {
        match chunk {
            [b_i] => {
                let (term_x, term_y) =
                    lookup_single_bit(*b_i, (sigma.clone(), sigma.clone() + two_to_the_i.clone()));
                let term_y = w.exists(term_y);
                let term_x = w.exists(term_x);
                acc = w.add_fast(acc, make_group(term_x, term_y));
            }
            [b_i, b_i_plus_1] => {
                let two_to_the_i_plus_1 = two_to_the_i.double().to_affine();
                let (term_x, term_y) = to_term(
                    two_to_the_i.clone(),
                    InnerCurve::of_affine(two_to_the_i_plus_1),
                    (*b_i, *b_i_plus_1),
                    w,
                );
                let term_y = w.exists(term_y);
                let term_x = w.exists(term_x);
                acc = w.add_fast(acc, make_group(term_x, term_y));
                two_to_the_i = InnerCurve::of_affine(two_to_the_i_plus_1).double();
            }
            _ => unreachable!(), // chunks of 2
        }
    }

    let result_with_shift = acc;
    let unshift = std::ops::Neg::neg(sigma).scale(sigma_count as u64);

    w.add_fast(result_with_shift, unshift.to_affine())
}

#[derive(Clone, Copy, Debug)]
enum Boolean {
    False,
    True,
}

impl Boolean {
    fn not(x: bool) -> Self {
        if x {
            Self::False
        } else {
            Self::True
        }
    }
}

#[derive(Debug)]
enum ExprBinary<T> {
    Lit(T),
    And(T, Box<ExprBinary<T>>),
    Or(T, Box<ExprBinary<T>>),
}

#[derive(Debug)]
enum ExprNary<T> {
    Lit(T),
    And(Vec<ExprNary<T>>),
    Or(Vec<ExprNary<T>>),
}

// let lt_bitstring_value =
//   let module Boolean = Checked.Boolean in
//   let module Expr = struct
//     module Binary = struct
//       type 'a t = Lit of 'a | And of 'a * 'a t | Or of 'a * 'a t
//     end

//     module Nary = struct
//       type 'a t = Lit of 'a | And of 'a t list | Or of 'a t list

//       let rec of_binary : 'a Binary.t -> 'a t = function
//         | Lit x ->
//             Lit x
//         | And (x, And (y, t)) ->
//             And [ Lit x; Lit y; of_binary t ]
//         | Or (x, Or (y, t)) ->
//             Or [ Lit x; Lit y; of_binary t ]
//         | And (x, t) ->
//             And [ Lit x; of_binary t ]
//         | Or (x, t) ->
//             Or [ Lit x; of_binary t ]

//       let rec eval =
//         let open Checked.Let_syntax in
//         function
//         | Lit x ->
//             return x
//         | And xs ->
//             Checked.List.map xs ~f:eval >>= Boolean.all
//         | Or xs ->
//             Checked.List.map xs ~f:eval >>= Boolean.any
//     end
//   end in
//   let rec lt_binary xs ys : Boolean.var Expr.Binary.t =
//     match (xs, ys) with
//     | [], [] ->
//         Lit Boolean.false_
//     | [ _x ], [ false ] ->
//         Lit Boolean.false_
//     | [ x ], [ true ] ->
//         Lit (Boolean.not x)
//     | [ x1; _x2 ], [ true; false ] ->
//         Lit (Boolean.not x1)
//     | [ _x1; _x2 ], [ false; false ] ->
//         Lit Boolean.false_
//     | x :: xs, false :: ys ->
//         And (Boolean.not x, lt_binary xs ys)
//     | x :: xs, true :: ys ->
//         Or (Boolean.not x, lt_binary xs ys)
//     | _ :: _, [] | [], _ :: _ ->
//         failwith "lt_bitstring_value: Got unequal length strings"
//   in
//   fun (xs : Boolean.var Bitstring_lib.Bitstring.Msb_first.t)
//       (ys : bool Bitstring_lib.Bitstring.Msb_first.t) ->
//     let open Expr.Nary in
//     Printf.eprintf "[snark0.lt_bitstring_value] START\n%!" ;
//     let value =
//       of_binary (lt_binary (xs :> Boolean.var list) (ys :> bool list))
//     in
//     Printf.eprintf "[snark0.lt_bitstring_value] 000\n%!" ;
//     let res = eval value in
//     Printf.eprintf "[snark0.lt_bitstring_value] DONE\n%!" ;
//     res

fn lt_binary<F: FieldWitness>(xs: &[bool], ys: &[bool]) -> ExprBinary<Boolean> {
    match (xs, ys) {
        ([], []) => ExprBinary::Lit(Boolean::False),
        ([_x], [false]) => ExprBinary::Lit(Boolean::False),
        ([x], [true]) => ExprBinary::Lit(Boolean::not(*x)),
        ([x1, _x2], [true, false]) => ExprBinary::Lit(Boolean::not(*x1)),
        ([_x1, _x2], [false, false]) => ExprBinary::Lit(Boolean::False),
        ([x, xs @ ..], [false, ys @ ..]) => {
            ExprBinary::And(Boolean::not(*x), Box::new(lt_binary::<F>(xs, ys)))
        }
        ([x, xs @ ..], [true, ys @ ..]) => {
            ExprBinary::Or(Boolean::not(*x), Box::new(lt_binary::<F>(xs, ys)))
        }
        _ => panic!("unequal length"),
    }
}

fn of_binary<F: FieldWitness>(expr: &ExprBinary<Boolean>) -> ExprNary<Boolean> {
    match expr {
        ExprBinary::Lit(x) => ExprNary::Lit(*x),
        ExprBinary::And(x, t) => match &**t {
            ExprBinary::And(y, t) => ExprNary::And(vec![
                ExprNary::Lit(*x),
                ExprNary::Lit(*y),
                of_binary::<F>(&**t),
            ]),
            _ => ExprNary::And(vec![ExprNary::Lit(*x), of_binary::<F>(&**t)]),
        },
        ExprBinary::Or(x, t) => match &**t {
            ExprBinary::Or(y, t) => ExprNary::Or(vec![
                ExprNary::Lit(*x),
                ExprNary::Lit(*y),
                of_binary::<F>(&**t),
            ]),
            _ => ExprNary::Or(vec![ExprNary::Lit(*x), of_binary::<F>(&**t)]),
        },
    }
}

fn lt_bitstring_value<F: FieldWitness>(xs: &[bool; 255], ys: &[bool; 255]) {
    let value = of_binary::<F>(&lt_binary::<F>(xs, ys));
    eprintln!("value={:?}", value);
}

fn is_even<F: FieldWitness>(y: F, w: &mut Witness<F>) {
    let bits_msb = {
        let mut bits = w.exists(field_to_bits::<F, 255>(y));
        bits.reverse(); // msb
        bits
    };

    let size_msb = {
        let mut size = bigint_to_bits::<255>(F::SIZE);
        size.reverse(); // msb
        size
    };

    lt_bitstring_value::<F>(&bits_msb, &size_msb);

    // let%map () =
    //   lt_bitstring_value
    //     (Bitstring.Msb_first.of_lsb_first res)
    //     field_size_bits
    //   >>= Checked.Boolean.Assert.is_true
    // in
    // Printf.eprintf "[snark0] UNPACK_FULL DONE\n%!" ;
    // res

    eprintln!("size={:?}", size_msb);

    // let a: BigInteger256 = mina_curves::pasta::fields::FpParameters::MODULUS;

    // dbg!(mina_curves::pasta::fields::FpParameters::MODULUS.to_string());
    // dbg!(mina_curves::pasta::fields::FqParameters::MODULUS.to_string());
}

mod transaction_snark {
    use std::ops::Neg;

    use crate::proofs::witness::legacy_input::CheckedLegacyInput;
    use mina_signer::PubKey;

    use crate::scan_state::{
        currency::{Amount, Fee, Slot},
        scan_state::ConstraintConstants,
        transaction_logic::transaction_union_payload::{TransactionUnion, TransactionUnionPayload},
    };
    use mina_signer::Signature;

    use super::{legacy_input::LegacyInput, *};

    // TODO: De-deplicates this constant in the repo
    pub const CONSTRAINT_CONSTANTS: ConstraintConstants = ConstraintConstants {
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

    fn hash(param: &str, inputs: LegacyInput<Fp>, w: &mut Witness<Fp>) -> Fp {
        use mina_poseidon::constants::PlonkSpongeConstantsLegacy as Constants;
        use mina_poseidon::pasta::fp_legacy::static_params;

        // We hash the parameter first, without introducing values to the witness
        let initial_state: [Fp; 3] = {
            use mina_poseidon::poseidon::ArithmeticSponge;
            use mina_poseidon::poseidon::Sponge;

            let mut sponge = ArithmeticSponge::<Fp, Constants>::new(static_params());
            sponge.absorb(&[crate::param_to_field(param)]);
            sponge.squeeze();
            sponge.state.try_into().unwrap()
        };

        let mut sponge =
            poseidon::Sponge::<Fp, Constants>::new_with_state(initial_state, static_params());
        sponge.absorb(&inputs.to_fields(), w);
        sponge.squeeze(w)
    }

    fn hash_checked(
        mut inputs: LegacyInput<Fp>,
        signer: &PubKey,
        signature: &Signature,
        w: &mut Witness<Fp>,
    ) -> [bool; 255] {
        let GroupAffine { x: px, y: py, .. } = signer.point();
        let Signature { rx, s: _ } = signature;

        inputs.append_field(*px);
        inputs.append_field(*py);
        inputs.append_field(*rx);
        let hash = hash("CodaSignature", inputs, w);

        w.exists(field_to_bits::<_, 255>(hash))
    }

    fn check_signature(
        shifted: &InnerCurve<Fp>,
        payload: &TransactionUnionPayload,
        _is_user_command: bool,
        signer: &PubKey,
        signature: &Signature,
        w: &mut Witness<Fp>,
    ) {
        println!("START\n");

        let inputs = payload.to_checked_legacy_input_owned(w);
        let hash = hash_checked(inputs, signer, signature, w);

        // negate
        let public_key = {
            let GroupAffine { x, y, .. } = signer.point();
            let y = w.exists(y.neg()); // This is actually made in the `scale` call below in OCaml
            make_group::<Fp>(*x, y)
        };

        let e_pk = scale_non_constant::<Fp, 255>(public_key, &hash, shifted, w);

        eprintln!("SCALE KNOWN START\n");
        let before = w.aux.len();

        let Signature { rx: _, s } = signature;
        let bits: [bool; 255] = field_to_bits::<_, 255>(*s);
        let one: GroupAffine<_> = InnerCurve::<Fp>::one().to_affine();
        let s_g_e_pk = scale_known(one, &bits, &InnerCurve::of_affine(e_pk), w);
        eprintln!("SCALE KNOWN TOTAL={:?}\n", w.aux.len() - before);

        let GroupAffine { x: rx, y: ry, .. } = {
            let neg_shifted = shifted.to_affine().neg();
            w.exists(neg_shifted.y);
            w.add_fast(neg_shifted, s_g_e_pk)
        };

        is_even(ry, w);
    }

    fn apply_tagged_transaction(
        shifted: &InnerCurve<Fp>,
        _fee_payment_root: Fp,
        _global_slot: Slot,
        _pending_coinbase_init: &v2::MinaBasePendingCoinbaseStackVersionedStableV1,
        _pending_coinbase_stack_before: &v2::MinaBasePendingCoinbaseStackVersionedStableV1,
        _pending_coinbase_stack_after: &v2::MinaBasePendingCoinbaseStackVersionedStableV1,
        _state_body: &MinaStateProtocolStateBodyValueStableV2,
        tx: &TransactionUnion,
        w: &mut Witness<Fp>,
    ) {
        let TransactionUnion {
            payload,
            signer,
            signature,
        } = tx;

        let tag = payload.body.tag.clone();
        let is_user_command = tag.is_user_command();

        check_signature(shifted, payload, is_user_command, signer, signature, w);
    }

    pub fn main(
        statement: &MinaStateBlockchainStateValueStableV2LedgerProofStatement,
        tx_witness: &v2::TransactionWitnessStableV2,
        w: &mut Witness<Fp>,
    ) {
        let tx: crate::scan_state::transaction_logic::Transaction =
            (&tx_witness.transaction).into();
        let tx = transaction_union_payload::TransactionUnion::of_transaction(&tx);

        dummy_constraints(w);
        let shifted = create_shifted_inner_curve(w);

        let state_body = w.exists(tx_witness.protocol_state_body.clone());
        let global_slot = w.exists(tx_witness.block_global_slot.clone());
        let tx = w.exists(tx);
        let pending_coinbase_init = w.exists(tx_witness.init_stack.clone());

        apply_tagged_transaction(
            &shifted,
            statement.source.first_pass_ledger.to_field(),
            Slot::from_u32(global_slot.as_u32()),
            &pending_coinbase_init,
            &statement.source.pending_coinbase_stack,
            &statement.target.pending_coinbase_stack,
            &state_body,
            &tx,
            w,
        );

        // let%bind fee_payment_root_after, fee_excess, supply_increase =
        //   apply_tagged_transaction ~constraint_constants
        //     (module Shifted)
        //     statement.source.first_pass_ledger global_slot pending_coinbase_init
        //     statement.source.pending_coinbase_stack
        //     statement.target.pending_coinbase_stack state_body t
        // in
        // Printf.eprintf "AFTER_TAGGED_TRANSACTION AFTER\n%!" ;
    }
}

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

        let res = scalar_challenge::to_field_checked_prime::<_, 32>(f, &mut witness);

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
