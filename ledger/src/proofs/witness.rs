use std::{collections::HashMap, str::FromStr, sync::Arc};

use ark_ec::{
    short_weierstrass_jacobian::GroupProjective, AffineCurve, ProjectiveCurve, SWModelParameters,
};
use ark_ff::{BigInteger256, FftField, Field, FpParameters, PrimeField, SquareRootField};
use kimchi::{
    circuits::{gate::CircuitGate, wires::COLUMNS},
    proof::RecursionChallenge,
    prover_index::ProverIndex,
};
use kimchi::{curve::KimchiCurve, proof::ProofEvaluations};
use mina_curves::pasta::Pallas;
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
use mina_poseidon::{constants::PlonkSpongeConstantsKimchi, sponge::DefaultFqSponge};
use mina_signer::{CompressedPubKey, PubKey};

use crate::{
    decompress_pk, gen_keypair,
    proofs::{
        constants::{RegularTransactionProof, WrapProof},
        unfinalized::AllEvals,
    },
    scan_state::{
        currency::{self, Sgn},
        fee_excess::FeeExcess,
        pending_coinbase,
        scan_state::transaction_snark::{Registers, SokDigest, SokMessage, Statement},
        transaction_logic::{
            local_state::LocalState,
            protocol_state::{EpochData, EpochLedger},
            transaction_union_payload,
        },
    },
    staged_ledger::hash::StagedLedgerHash,
    verifier::get_srs,
    Account, MyCow, ReceiptChainHash, SpongeParamsForField, TimingAsRecord, TokenId, TokenSymbol,
    VotingFor,
};

use super::{
    constants::ProofConstants,
    numbers::currency::{CheckedCurrency, CheckedSigned},
    public_input::{
        messages::{dummy_ipa_step_sg, MessagesForNextWrapProof},
        plonk_checks::{self, ShiftedValue},
    },
    to_field_elements::ToFieldElements,
    unfinalized::{EvalsWithPublicInput, Unfinalized},
    BACKEND_TICK_ROUNDS_N, BACKEND_TOCK_ROUNDS_N,
};

pub type GroupAffine<F> =
    ark_ec::short_weierstrass_jacobian::GroupAffine<<F as FieldWitness>::Parameters>;

#[derive(Debug)]
pub struct Witness<F: FieldWitness> {
    pub primary: Vec<F>,
    pub(super) aux: Vec<F>,
    // Following fields are used to compare our witness with OCaml
    pub ocaml_aux: Vec<F>,
    ocaml_aux_index: usize,
}

impl<F: FieldWitness> Witness<F> {
    pub fn new<C: ProofConstants>() -> Self {
        Self {
            primary: Vec::with_capacity(C::PRIMARY_LEN),
            aux: Vec::with_capacity(C::AUX_LEN),
            ocaml_aux: Vec::new(),
            ocaml_aux_index: 0,
        }
    }

    pub fn empty() -> Self {
        Self {
            primary: Vec::new(),
            aux: Vec::new(),
            ocaml_aux: Vec::new(),
            ocaml_aux_index: 0,
        }
    }

    pub fn push<I: Into<F>>(&mut self, field: I) {
        let field = {
            let field: F = field.into();
            // dbg!(field)
            field
        };
        self.assert_ocaml_aux(&[field]);
        self.aux.push(field);
    }

    pub fn extend<I: Into<F>, V: Iterator<Item = I>>(&mut self, field: V) {
        let fields = {
            let fields: Vec<F> = field.map(Into::into).collect();
            self.assert_ocaml_aux(&fields);
            // eprintln!("extend[{}]={:#?}", fields.len(), fields);
            fields
        };
        self.aux.extend(fields)
    }

    pub fn exists<T>(&mut self, data: T) -> T
    where
        T: ToFieldElements<F> + Check<F>,
    {
        // data.to_field_elements(&mut self.aux);
        let mut fields = data.to_field_elements_owned();
        self.assert_ocaml_aux(&fields);

        // eprintln!("index={:?} w{:?}", self.aux.len() + 67, &fields);
        eprintln!(
            "index={:?} w{:?}",
            self.aux.len() + self.primary.capacity(),
            &fields
        );
        self.aux.append(&mut fields);

        data.check(self);
        data
    }

    pub fn exists_no_check<T>(&mut self, data: T) -> T
    where
        T: ToFieldElements<F>,
    {
        // data.to_field_elements(&mut self.aux);
        let mut fields = data.to_field_elements_owned();
        self.assert_ocaml_aux(&fields);

        // eprintln!("index={:?} w{:?}", self.aux.len() + 67, &fields);
        eprintln!(
            "index={:?} w{:?}",
            self.aux.len() + self.primary.capacity(),
            &fields
        );
        self.aux.append(&mut fields);

        data
    }

    /// Compare our witness with OCaml
    fn assert_ocaml_aux(&mut self, new_fields: &[F]) {
        if self.ocaml_aux.is_empty() {
            return;
        }

        // let len = new_fields.len();
        // let before = self.aux.len();
        // let ocaml = &self.ocaml_aux[before..before + len];
        // eprintln!("w{:?} ocaml{:?} {:?}", new_fields, ocaml, new_fields == ocaml);

        let len = new_fields.len();
        let before = self.aux.len();
        assert_eq!(before, self.ocaml_aux_index);
        assert_eq!(new_fields, &self.ocaml_aux[before..before + len]);

        self.ocaml_aux_index += len;
    }

    /// Helper
    pub fn to_field_checked_prime<const NBITS: usize>(&mut self, scalar: F) -> (F, F, F) {
        scalar_challenge::to_field_checked_prime::<F, NBITS>(scalar, self)
    }

    /// Helper
    pub fn add_fast(&mut self, p1: GroupAffine<F>, p2: GroupAffine<F>) -> GroupAffine<F> {
        add_fast::<F>(p1, p2, None, self)
    }
}

pub trait Check<F: FieldWitness> {
    fn check(&self, w: &mut Witness<F>);
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

pub fn field_to_bits<F, const NBITS: usize>(field: F) -> [bool; NBITS]
where
    F: Field + Into<BigInteger256>,
{
    let bigint: BigInteger256 = field.into();
    bigint_to_bits(bigint)
}

/// Difference with `bigint_to_bits`: the number of bits isn't a constant
fn bigint_to_bits2(bigint: BigInteger256, nbits: usize) -> Box<[bool]> {
    FieldBitsIterator { index: 0, bigint }.take(nbits).collect()
}

/// Difference with `field_to_bits`: the number of bits isn't a constant
pub fn field_to_bits2<F>(field: F, nbits: usize) -> Box<[bool]>
where
    F: Field + Into<BigInteger256>,
{
    let bigint: BigInteger256 = field.into();
    bigint_to_bits2(bigint, nbits)
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
    use poly_commitment::srs::endos;

    // Let's keep them in cache since they're used everywhere
    cache!((F, F::Scalar), endos::<GroupAffine<F>>())
}

pub fn make_group<F>(x: F, y: F) -> GroupAffine<F>
where
    F: FieldWitness,
{
    GroupAffine::<F>::new(x, y, false)
}

pub mod scalar_challenge {
    use super::*;

    // TODO: `scalar` might be a `F::Scalar` here
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

        for nybbles_by_row in nybbles_by_row.iter().take(rows) {
            let n0 = n;
            let a0 = a;
            let b0 = b;

            let xs: Vec<F> = (0..nybbles_per_row)
                .map(|j| w.exists(F::from(nybbles_by_row[j])))
                .collect();

            let n8: F = w.exists(xs.iter().fold(n0, |accum, x| accum.double().double() + x));

            let a8: F = w.exists(
                nybbles_by_row
                    .iter()
                    .fold(a0, |accum, x| accum.double() + a_func(*x)),
            );

            let b8: F = w.exists(
                nybbles_by_row
                    .iter()
                    .fold(b0, |accum, x| accum.double() + b_func(*x)),
            );

            n = n8;
            a = a8;
            b = b8;
        }

        (a, b, n)
    }

    // TODO: `scalar` might be a `F::Scalar` here
    pub fn to_field_checked<F, const NBITS: usize>(scalar: F, endo: F, w: &mut Witness<F>) -> F
    where
        F: FieldWitness,
    {
        let (a, b, _n) = to_field_checked_prime::<F, NBITS>(scalar, w);
        (a * endo) + b
    }

    // TODO: Use `F::Scalar` instead of `F2`
    pub fn endo<F, F2, const NBITS: usize>(
        t: GroupAffine<F>,
        scalar: F2,
        w: &mut Witness<F>,
    ) -> GroupAffine<F>
    where
        F: FieldWitness,
        F2: FieldWitness,
    {
        let bits: [bool; NBITS] = bits_msb::<F2, NBITS>(scalar);

        let bits_per_row = 4;
        let rows = NBITS / bits_per_row;

        let GroupAffine::<F> { x: xt, y: yt, .. } = t;
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

            let GroupAffine::<F> { x: xp, y: yp, .. } = acc;
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

    // TODO: Use `F::Scalar` for `chal`
    pub fn endo_inv<F, F2, const NBITS: usize>(
        t: GroupAffine<F>,
        chal: F2,
        w: &mut Witness<F>,
    ) -> GroupAffine<F>
    where
        F: FieldWitness,
        F2: FieldWitness,
    {
        use crate::proofs::public_input::scalar_challenge::ScalarChallenge;
        use ark_ff::One;

        let (_, e) = endos::<F>();

        let res = w.exists({
            let chal = ScalarChallenge::from(chal).to_field(&e);
            InnerCurve::<F>::of_affine(t).scale(<F::Scalar>::one() / chal)
        });
        let _ = endo::<F, F2, NBITS>(res.to_affine(), chal, w);
        res.to_affine()
    }
}

pub fn add_fast<F>(
    p1: GroupAffine<F>,
    p2: GroupAffine<F>,
    check_finite: Option<bool>,
    w: &mut Witness<F>,
) -> GroupAffine<F>
where
    F: FieldWitness,
{
    let GroupAffine::<F> { x: x1, y: y1, .. } = p1;
    let GroupAffine::<F> { x: x2, y: y2, .. } = p2;
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

pub mod plonk_curve_ops {
    use crate::proofs::public_input::plonk_checks::ShiftingValue;

    use super::*;

    const BITS_PER_CHUNK: usize = 5;

    // TODO: `scalar` is a `F::Scalar` here
    pub fn scale_fast<F, F2, const NBITS: usize>(
        base: GroupAffine<F>,
        shifted_value: F2::Shifting,
        w: &mut Witness<F>,
    ) -> GroupAffine<F>
    where
        F: FieldWitness,
        F2: FieldWitness,
    {
        let (r, _bits) = scale_fast_unpack::<F, F2, NBITS>(base, shifted_value, w);
        r
    }

    // TODO: `scalar` is a `F::Scalar` here
    // https://github.com/openmina/mina/blob/8f83199a92faa8ff592b7ae5ad5b3236160e8c20/src/lib/pickles/plonk_curve_ops.ml#L140
    pub fn scale_fast_unpack<F, F2, const NBITS: usize>(
        base: GroupAffine<F>,
        shifted: F2::Shifting,
        w: &mut Witness<F>,
    ) -> (GroupAffine<F>, [bool; NBITS])
    where
        F: FieldWitness,
        F2: FieldWitness,
    {
        let scalar = shifted.shifted_raw();
        let GroupAffine::<F> {
            x: x_base,
            y: y_base,
            ..
        } = base;

        let chunks: usize = NBITS / BITS_PER_CHUNK;
        assert_eq!(NBITS % BITS_PER_CHUNK, 0);

        let bits_msb: [bool; NBITS] = w.exists(bits_msb::<F2, NBITS>(scalar));
        let mut acc = w.add_fast(base, base);
        let mut n_acc = F::zero();

        for chunk in 0..chunks {
            let bs: [bool; BITS_PER_CHUNK] =
                std::array::from_fn(|i| bits_msb[(chunk * BITS_PER_CHUNK) + i]);

            let n_acc_prev = n_acc;

            n_acc = w.exists(
                bs.iter()
                    .fold(n_acc_prev, |acc, b| acc.double() + F::from(*b)),
            );

            let (_, v) = fold_map(bs.iter(), acc, |acc, b| {
                let GroupAffine::<F> {
                    x: x_acc, y: y_acc, ..
                } = acc;
                let b: F = F::from(*b);

                let s1: F =
                    w.exists((y_acc - (y_base * (b.double() - F::one()))) / (x_acc - x_base));
                let s1_squared = w.exists(s1.square());
                let s2 = w.exists((y_acc.double() / (x_acc.double() + x_base - s1_squared)) - s1);

                let x_res = w.exists(x_base + s2.square() - s1_squared);
                let y_res = w.exists(((x_acc - x_res) * s2) - y_acc);
                let acc = make_group(x_res, y_res);

                (acc, (acc, s1))
            });

            let (mut accs, _slopes): (Vec<_>, Vec<_>) = v.into_iter().unzip();

            accs.insert(0, acc);
            acc = accs.last().cloned().unwrap();
        }

        let bits_lsb = {
            let mut bits_msb = bits_msb.clone();
            bits_msb.reverse();
            bits_msb
        };

        (acc, bits_lsb)
    }
}

impl<F: FieldWitness, T: ToFieldElements<F>> ToFieldElements<F> for Vec<T> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        self.iter().for_each(|v| v.to_field_elements(fields));
    }
}

impl<F: FieldWitness, T: ToFieldElements<F>> ToFieldElements<F> for Box<[T]> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        self.iter().for_each(|v| v.to_field_elements(fields));
    }
}

impl<F: FieldWitness> ToFieldElements<F> for Fp {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        fields.push(self.into_gen());
    }
}

// pack
pub fn field_of_bits<F: FieldWitness, const N: usize>(bs: &[bool; N]) -> F {
    bs.iter().rev().fold(F::zero(), |acc, b| {
        let acc = acc + acc;
        if *b {
            acc + F::one()
        } else {
            acc
        }
    })
}

impl<F: FieldWitness> ToFieldElements<F> for Fq {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        use std::any::TypeId;

        // TODO: Refactor when specialization is stable
        if TypeId::of::<F>() == TypeId::of::<Fq>() {
            fields.push(self.into_gen());
        } else {
            // `Fq` is larger than `Fp` so we have to split the field (low & high bits)
            // See:
            // https://github.com/MinaProtocol/mina/blob/e85cf6969e42060f69d305fb63df9b8d7215d3d7/src/lib/pickles/impls.ml#L94C1-L105C45

            let to_high_low = |fq: Fq| {
                let [low, high @ ..] = field_to_bits::<Fq, 255>(fq);
                [field_of_bits(&high), F::from(low)]
            };
            fields.extend(to_high_low(*self));
        }
    }
}

impl<F: FieldWitness, T: ToFieldElements<F>, const N: usize> ToFieldElements<F> for [T; N] {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        self.iter().for_each(|v| v.to_field_elements(fields));
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

impl<F: FieldWitness> ToFieldElements<F> for GroupAffine<F> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self {
            x, y, infinity: _, ..
        } = self;
        y.to_field_elements(fields);
        x.to_field_elements(fields);
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

impl<F: FieldWitness> ToFieldElements<F> for bool {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        F::from(*self).to_field_elements(fields)
    }
}

impl<F: FieldWitness> ToFieldElements<F> for ProofEvaluations<[F; 2]> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self {
            w,
            z,
            s,
            coefficients,
            generic_selector,
            poseidon_selector,
            complete_add_selector,
            mul_selector,
            emul_selector,
            endomul_scalar_selector,
            range_check0_selector,
            range_check1_selector,
            foreign_field_add_selector,
            foreign_field_mul_selector,
            xor_selector,
            rot_selector,
            lookup_aggregation,
            lookup_table,
            lookup_sorted,
            runtime_lookup_table,
            runtime_lookup_table_selector,
            xor_lookup_selector,
            lookup_gate_lookup_selector,
            range_check_lookup_selector,
            foreign_field_mul_lookup_selector,
        } = self;

        let mut push = |[a, b]: &[F; 2]| {
            a.to_field_elements(fields);
            b.to_field_elements(fields);
        };

        w.iter().for_each(&mut push);
        coefficients.iter().for_each(&mut push);
        push(z);
        s.iter().for_each(&mut push);
        push(generic_selector);
        push(poseidon_selector);
        push(complete_add_selector);
        push(mul_selector);
        push(emul_selector);
        push(endomul_scalar_selector);
        range_check0_selector.as_ref().map(&mut push);
        range_check1_selector.as_ref().map(&mut push);
        foreign_field_add_selector.as_ref().map(&mut push);
        foreign_field_mul_selector.as_ref().map(&mut push);
        xor_selector.as_ref().map(&mut push);
        rot_selector.as_ref().map(&mut push);
        lookup_aggregation.as_ref().map(&mut push);
        lookup_table.as_ref().map(&mut push);
        lookup_sorted.iter().for_each(|v| {
            v.as_ref().map(&mut push);
        });
        runtime_lookup_table.as_ref().map(&mut push);
        runtime_lookup_table_selector.as_ref().map(&mut push);
        xor_lookup_selector.as_ref().map(&mut push);
        lookup_gate_lookup_selector.as_ref().map(&mut push);
        range_check_lookup_selector.as_ref().map(&mut push);
        foreign_field_mul_lookup_selector.as_ref().map(&mut push);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for AllEvals<F> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self {
            ft_eval1,
            evals:
                EvalsWithPublicInput {
                    evals,
                    public_input,
                },
        } = self;

        public_input.to_field_elements(fields);
        evals.to_field_elements(fields);
        ft_eval1.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for &[AllEvals<F>] {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        self.iter().for_each(|e| e.to_field_elements(fields))
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

impl<F: FieldWitness> ToFieldElements<F> for TokenId {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self(token_id) = self;
        fields.push(token_id.into_gen());
    }
}

impl<F: FieldWitness> ToFieldElements<F> for CompressedPubKey {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self { x, is_odd } = self;
        fields.push(x.into_gen());
        fields.push(F::from(*is_odd));
    }
}

impl<F: FieldWitness> ToFieldElements<F> for mina_signer::Signature {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self { rx, s } = self;

        fields.push(rx.into_gen());
        let s_bits = field_to_bits::<_, 255>(*s);
        s_bits.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for mina_signer::PubKey {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let GroupAffine::<Fp> { x, y, .. } = self.point();
        fields.push(x.into_gen());
        fields.push(y.into_gen());
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

impl<F: FieldWitness> ToFieldElements<F> for pending_coinbase::Stack {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self {
            data,
            state: pending_coinbase::StateStack { init, curr },
        } = self;

        fields.push(data.0.into_gen());
        fields.push(init.into_gen());
        fields.push(curr.into_gen());
    }
}

impl<F: FieldWitness> ToFieldElements<F> for TokenSymbol {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let field: F = self.to_field();
        field.to_field_elements(fields);
    }
}

// TODO: De-deduplicate with ToInputs
impl<F: FieldWitness> ToFieldElements<F> for crate::Timing {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let TimingAsRecord {
            is_timed,
            initial_minimum_balance,
            cliff_time,
            cliff_amount,
            vesting_period,
            vesting_increment,
        } = self.to_record();

        fields.push(F::from(is_timed));
        fields.push(F::from(initial_minimum_balance.as_u64()));
        fields.push(F::from(cliff_time.as_u32()));
        fields.push(F::from(cliff_amount.as_u64()));
        fields.push(F::from(vesting_period.as_u32()));
        fields.push(F::from(vesting_increment.as_u64()));
    }
}

impl<F: FieldWitness> ToFieldElements<F> for crate::Permissions<crate::AuthRequired> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        self.iter_as_bits(|bit| {
            fields.push(F::from(bit));
        });
    }
}

impl<F: FieldWitness> ToFieldElements<F> for Box<Account> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Account {
            public_key,
            token_id: TokenId(token_id),
            token_symbol,
            balance,
            nonce,
            receipt_chain_hash: ReceiptChainHash(receipt_chain_hash),
            delegate,
            voting_for: VotingFor(voting_for),
            timing,
            permissions,
            zkapp,
        } = &**self;

        public_key.to_field_elements(fields);
        token_id.to_field_elements(fields);
        token_symbol.to_field_elements(fields);
        balance.to_field_elements(fields);
        nonce.to_field_elements(fields);
        receipt_chain_hash.to_field_elements(fields);
        let delegate = MyCow::borrow_or_else(delegate, CompressedPubKey::empty);
        delegate.to_field_elements(fields);
        voting_for.to_field_elements(fields);
        timing.to_field_elements(fields);
        permissions.to_field_elements(fields);

        let zkapp: F = {
            let zkapp = MyCow::borrow_or_default(zkapp);
            zkapp.hash().into_gen()
        };
        zkapp.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for crate::MerklePath {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        self.hash().to_field_elements(fields);
    }
}

impl<F: FieldWitness, A: ToFieldElements<F>, B: ToFieldElements<F>> ToFieldElements<F> for (A, B) {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let (a, b) = self;
        a.to_field_elements(fields);
        b.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for ReceiptChainHash {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self(receipt_chain_hash) = self;
        receipt_chain_hash.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for Sgn {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let field: F = self.to_field();
        field.to_field_elements(fields)
    }
}

impl<F: FieldWitness, T: currency::Magnitude + ToFieldElements<F>> ToFieldElements<F>
    for currency::Signed<T>
{
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self { magnitude, sgn } = self;

        magnitude.to_field_elements(fields);
        sgn.to_field_elements(fields);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlonkVerificationKeyEvals<F: FieldWitness> {
    pub sigma: [InnerCurve<F>; 7],
    pub coefficients: [InnerCurve<F>; 15],
    pub generic: InnerCurve<F>,
    pub psm: InnerCurve<F>,
    pub complete_add: InnerCurve<F>,
    pub mul: InnerCurve<F>,
    pub emul: InnerCurve<F>,
    pub endomul_scalar: InnerCurve<F>,
}

impl PlonkVerificationKeyEvals<Fp> {
    /// For debugging
    fn to_string(&self) -> String {
        let Self {
            sigma,
            coefficients,
            generic,
            psm,
            complete_add,
            mul,
            emul,
            endomul_scalar,
        } = self;

        let mut string = String::with_capacity(1_000);

        use crate::util::FpExt;

        let mut inner_to_s = |c: &InnerCurve<Fp>| {
            let GroupAffine::<Fp> { x, y, .. } = c.to_affine();
            string.push_str(&format!("{}\n", x.to_decimal()));
            string.push_str(&format!("{}\n", y.to_decimal()));
        };

        sigma.iter().for_each(|c| inner_to_s(c));
        coefficients.iter().for_each(|c| inner_to_s(c));
        inner_to_s(generic);
        inner_to_s(psm);
        inner_to_s(complete_add);
        inner_to_s(mul);
        inner_to_s(emul);
        inner_to_s(endomul_scalar);

        string.trim().to_string()
    }

    /// For debugging
    fn from_string(s: &str) -> Self {
        let mut s = s.lines();

        let mut to_inner = || {
            let a = s.next().unwrap();
            let b = s.next().unwrap();

            let a = Fp::from_str(a).unwrap();
            let b = Fp::from_str(b).unwrap();

            InnerCurve::<Fp>::of_affine(make_group(a, b))
        };

        Self {
            sigma: std::array::from_fn(|_| to_inner()),
            coefficients: std::array::from_fn(|_| to_inner()),
            generic: to_inner(),
            psm: to_inner(),
            complete_add: to_inner(),
            mul: to_inner(),
            emul: to_inner(),
            endomul_scalar: to_inner(),
        }
    }

    pub fn rand() -> Self {
        Self {
            sigma: [
                InnerCurve::rand(),
                InnerCurve::rand(),
                InnerCurve::rand(),
                InnerCurve::rand(),
                InnerCurve::rand(),
                InnerCurve::rand(),
                InnerCurve::rand(),
            ],
            coefficients: [
                InnerCurve::rand(),
                InnerCurve::rand(),
                InnerCurve::rand(),
                InnerCurve::rand(),
                InnerCurve::rand(),
                InnerCurve::rand(),
                InnerCurve::rand(),
                InnerCurve::rand(),
                InnerCurve::rand(),
                InnerCurve::rand(),
                InnerCurve::rand(),
                InnerCurve::rand(),
                InnerCurve::rand(),
                InnerCurve::rand(),
                InnerCurve::rand(),
            ],
            generic: InnerCurve::rand(),
            psm: InnerCurve::rand(),
            complete_add: InnerCurve::rand(),
            mul: InnerCurve::rand(),
            emul: InnerCurve::rand(),
            endomul_scalar: InnerCurve::rand(),
        }
    }
}

impl crate::ToInputs for PlonkVerificationKeyEvals<Fp> {
    fn to_inputs(&self, inputs: &mut crate::Inputs) {
        let Self {
            sigma,
            coefficients,
            generic,
            psm,
            complete_add,
            mul,
            emul,
            endomul_scalar,
        } = self;

        let mut to_input = |v: &InnerCurve<Fp>| {
            let GroupAffine::<Fp> { x, y, .. } = v.to_affine();
            inputs.append(&x);
            inputs.append(&y);
        };

        sigma.iter().for_each(|c| to_input(c));
        coefficients.iter().for_each(|c| to_input(c));
        to_input(generic);
        to_input(psm);
        to_input(complete_add);
        to_input(mul);
        to_input(emul);
        to_input(endomul_scalar);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for PlonkVerificationKeyEvals<F> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self {
            sigma,
            coefficients,
            generic,
            psm,
            complete_add,
            mul,
            emul,
            endomul_scalar,
        } = self;

        sigma.iter().for_each(|s| s.to_field_elements(fields));
        coefficients
            .iter()
            .for_each(|c| c.to_field_elements(fields));
        generic.to_field_elements(fields);
        psm.to_field_elements(fields);
        complete_add.to_field_elements(fields);
        mul.to_field_elements(fields);
        emul.to_field_elements(fields);
        endomul_scalar.to_field_elements(fields);
    }
}

// Implementation for references
impl<F: FieldWitness, T: ToFieldElements<F>> ToFieldElements<F> for &T {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        (*self).to_field_elements(fields);
    }
}

// Implementation for references
impl<F: FieldWitness, T: Check<F>> Check<F> for &T {
    fn check(&self, w: &mut Witness<F>) {
        (*self).check(w)
    }
}

impl<F: FieldWitness> Check<F> for PlonkVerificationKeyEvals<F> {
    fn check(&self, w: &mut Witness<F>) {
        let Self {
            sigma,
            coefficients,
            generic,
            psm,
            complete_add,
            mul,
            emul,
            endomul_scalar,
        } = self;

        sigma.iter().for_each(|s| s.check(w));
        coefficients.iter().for_each(|c| c.check(w));
        generic.check(w);
        psm.check(w);
        complete_add.check(w);
        mul.check(w);
        emul.check(w);
        endomul_scalar.check(w);
    }
}

impl<F: FieldWitness, T: CheckedCurrency<F>> ToFieldElements<F> for CheckedSigned<F, T> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        self.sgn.to_field_elements(fields);
        self.magnitude.to_field_elements(fields);
    }
}

impl<F: FieldWitness> Check<F> for SgnStableV1 {
    fn check(&self, _w: &mut Witness<F>) {
        // Does not modify the witness
    }
}

impl<F: FieldWitness> Check<F> for bool {
    fn check(&self, _w: &mut Witness<F>) {
        // Does not modify the witness
    }
}

impl<F: FieldWitness> Check<F> for Fp {
    fn check(&self, _w: &mut Witness<F>) {
        // Does not modify the witness
    }
}

impl<F: FieldWitness> Check<F> for Fq {
    fn check(&self, _w: &mut Witness<F>) {
        // Does not modify the witness
    }
}

impl<F: FieldWitness, T: Check<F>, const N: usize> Check<F> for [T; N] {
    fn check(&self, w: &mut Witness<F>) {
        self.iter().for_each(|v| v.check(w));
    }
}

impl<F: FieldWitness> Check<F> for CurrencyAmountStableV1 {
    fn check(&self, w: &mut Witness<F>) {
        const NBITS: usize = u64::BITS as usize;

        let amount: u64 = self.as_u64();
        assert_eq!(NBITS, std::mem::size_of_val(&amount) * 8);

        let amount: F = amount.into();
        scalar_challenge::to_field_checked_prime::<F, NBITS>(amount, w);
    }
}

impl<F: FieldWitness> Check<F> for SignedAmount {
    fn check(&self, w: &mut Witness<F>) {
        let Self { magnitude, sgn } = self;
        magnitude.check(w);
        sgn.check(w);
    }
}

impl<F: FieldWitness, T: currency::Magnitude + Check<F>> Check<F> for currency::Signed<T> {
    fn check(&self, w: &mut Witness<F>) {
        let Self { magnitude, sgn } = self;
        magnitude.check(w);
        sgn.check(w);
    }
}

impl<F: FieldWitness> Check<F> for MinaStateBlockchainStateValueStableV2SignedAmount {
    fn check(&self, w: &mut Witness<F>) {
        let Self { magnitude, sgn } = self;
        magnitude.check(w);
        sgn.check(w);
    }
}

impl<F: FieldWitness> Check<F> for UnsignedExtendedUInt32StableV1 {
    fn check(&self, w: &mut Witness<F>) {
        const NBITS: usize = u32::BITS as usize;

        let number: u32 = self.as_u32();
        assert_eq!(NBITS, std::mem::size_of_val(&number) * 8);

        let number: F = number.into();
        scalar_challenge::to_field_checked_prime::<F, NBITS>(number, w);
    }
}

impl<F: FieldWitness> Check<F> for MinaStateBlockchainStateValueStableV2LedgerProofStatementSource {
    fn check(&self, w: &mut Witness<F>) {
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

        excess.check(w);
        supply_increase.check(w);
        success.check(w);
        account_update_index.check(w);
        will_succeed.check(w);
    }
}

impl<F: FieldWitness> Check<F> for Registers {
    fn check(&self, w: &mut Witness<F>) {
        let Self {
            first_pass_ledger: _,
            second_pass_ledger: _,
            pending_coinbase_stack: _,
            local_state:
                LocalState {
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

        excess.check(w);
        supply_increase.check(w);
        success.check(w);
        account_update_index.check(w);
        will_succeed.check(w);
    }
}

impl<F: FieldWitness> Check<F> for MinaStateBlockchainStateValueStableV2LedgerProofStatement {
    fn check(&self, w: &mut Witness<F>) {
        let Self {
            source,
            target,
            connecting_ledger_left: _,
            connecting_ledger_right: _,
            supply_increase,
            fee_excess,
            sok_digest: _,
        } = self;

        source.check(w);
        target.check(w);
        supply_increase.check(w);
        fee_excess.check(w);
    }
}

impl<F: FieldWitness, T> Check<F> for Statement<T> {
    fn check(&self, w: &mut Witness<F>) {
        let Self {
            source,
            target,
            connecting_ledger_left: _,
            connecting_ledger_right: _,
            supply_increase,
            fee_excess,
            sok_digest: _,
        } = self;

        source.check(w);
        target.check(w);
        supply_increase.check(w);
        fee_excess.check(w);
    }
}

impl<F: FieldWitness> Check<F> for MinaBaseFeeExcessStableV1 {
    fn check(&self, w: &mut Witness<F>) {
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

        fee_excess_l.check(w);
        fee_excess_r.check(w);
    }
}

impl<F: FieldWitness> Check<F> for FeeExcess {
    fn check(&self, w: &mut Witness<F>) {
        let Self {
            fee_token_l: _,
            fee_excess_l,
            fee_token_r: _,
            fee_excess_r,
        } = self;

        fee_excess_l.check(w);
        fee_excess_r.check(w);
    }
}

impl<F: FieldWitness> Check<F> for UnsignedExtendedUInt64Int64ForVersionTagsStableV1 {
    fn check(&self, w: &mut Witness<F>) {
        const NBITS: usize = u64::BITS as usize;

        let number: u64 = self.as_u64();
        assert_eq!(NBITS, std::mem::size_of_val(&number) * 8);

        let number: F = number.into();
        scalar_challenge::to_field_checked_prime::<F, NBITS>(number, w);
    }
}

impl<F: FieldWitness> Check<F> for MinaNumbersGlobalSlotSinceGenesisMStableV1 {
    fn check(&self, w: &mut Witness<F>) {
        let Self::SinceGenesis(global_slot) = self;
        global_slot.check(w);
    }
}

impl<F: FieldWitness> Check<F> for MinaNumbersGlobalSlotSinceHardForkMStableV1 {
    fn check(&self, w: &mut Witness<F>) {
        let Self::SinceHardFork(global_slot) = self;
        global_slot.check(w);
    }
}

impl<F: FieldWitness> Check<F>
    for ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1
{
    fn check(&self, w: &mut Witness<F>) {
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

        total_currency.check(w);
        epoch_length.check(w);
    }
}

impl<F: FieldWitness> Check<F>
    for ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1
{
    fn check(&self, w: &mut Witness<F>) {
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

        total_currency.check(w);
        epoch_length.check(w);
    }
}

impl<F: FieldWitness> Check<F> for ConsensusGlobalSlotStableV1 {
    fn check(&self, w: &mut Witness<F>) {
        let Self {
            slot_number,
            slots_per_epoch,
        } = self;

        slot_number.check(w);
        slots_per_epoch.check(w);
    }
}

impl<F: FieldWitness> Check<F> for MinaStateBlockchainStateValueStableV2 {
    fn check(&self, w: &mut Witness<F>) {
        let Self {
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
        } = self;

        source.check(w);
        target.check(w);
        supply_increase.check(w);
        fee_excess.check(w);
        timestamp.check(w);
    }
}

impl<F: FieldWitness> Check<F> for MinaStateProtocolStateBodyValueStableV2 {
    fn check(&self, w: &mut Witness<F>) {
        let MinaStateProtocolStateBodyValueStableV2 {
            genesis_state_hash: _,
            blockchain_state,
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

        blockchain_state.check(w);

        blockchain_length.check(w);
        epoch_count.check(w);
        min_window_density.check(w);
        // TODO: Check/assert that length equal `constraint_constants.sub_windows_per_window`
        for sub_window_density in sub_window_densities {
            sub_window_density.check(w);
        }
        total_currency.check(w);
        curr_global_slot.check(w);
        global_slot_since_genesis.check(w);
        staking_epoch_data.check(w);
        next_epoch_data.check(w);
        has_ancestor_in_same_checkpoint_window.check(w);
        supercharge_coinbase.check(w);
        k.check(w);
        slots_per_epoch.check(w);
        slots_per_sub_window.check(w);
        delta.check(w);
        genesis_state_timestamp.check(w);
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
        + From<i32>
        + ToFieldElements<Self>
        + Check<Self>
        + FromFpFq
        + PrimeField
        + SquareRootField
        + FftField
        + SpongeParamsForField<Self>
        + std::fmt::Debug
        + 'static,
{
    type Scalar: FieldWitness<Scalar = Self>;
    type Affine: AffineCurve<Projective = Self::Projective, BaseField = Self, ScalarField = Self::Scalar>
        + Into<GroupAffine<Self>>
        + KimchiCurve
        + std::fmt::Debug;
    type Projective: ProjectiveCurve<Affine = Self::Affine, BaseField = Self, ScalarField = Self::Scalar>
        + From<GroupProjective<Self::Parameters>>
        + std::fmt::Debug;
    type Parameters: SWModelParameters<BaseField = Self, ScalarField = Self::Scalar>
        + Clone
        + std::fmt::Debug;
    type Shifting: plonk_checks::ShiftingValue<Self> + Clone + std::fmt::Debug;
    type OtherCurve: KimchiCurve<
        ScalarField = Self,
        BaseField = Self::Scalar,
        OtherCurve = Self::Affine,
    >;
    type FqSponge: Clone + mina_poseidon::FqSponge<Self::Scalar, Self::OtherCurve, Self>;

    const PARAMS: Params<Self>;
    const SIZE: BigInteger256;
    const NROUNDS: usize;
    const SRS_DEPTH: usize;
}

pub struct Params<F> {
    pub a: F,
    pub b: F,
}

impl FieldWitness for Fp {
    type Scalar = Fq;
    type Parameters = PallasParameters;
    type Affine = GroupAffine<Self>;
    type Projective = ProjectivePallas;
    type Shifting = ShiftedValue<Fp>;
    type OtherCurve = <Self::Affine as KimchiCurve>::OtherCurve;
    type FqSponge = DefaultFqSponge<VestaParameters, PlonkSpongeConstantsKimchi>;

    /// https://github.com/openmina/mina/blob/46b6403cb7f158b66a60fc472da2db043ace2910/src/lib/crypto/kimchi_backend/pasta/basic/kimchi_pasta_basic.ml#L107
    const PARAMS: Params<Self> = Params::<Self> {
        a: ark_ff::field_new!(Fp, "0"),
        b: ark_ff::field_new!(Fp, "5"),
    };
    const SIZE: BigInteger256 = mina_curves::pasta::fields::FpParameters::MODULUS;
    const NROUNDS: usize = BACKEND_TICK_ROUNDS_N;
    const SRS_DEPTH: usize = 32768;
}

impl FieldWitness for Fq {
    type Scalar = Fp;
    type Parameters = VestaParameters;
    type Affine = GroupAffine<Self>;
    type Projective = ProjectiveVesta;
    type Shifting = ShiftedValue<Fq>;
    type OtherCurve = <Self::Affine as KimchiCurve>::OtherCurve;
    type FqSponge = DefaultFqSponge<PallasParameters, PlonkSpongeConstantsKimchi>;

    /// https://github.com/openmina/mina/blob/46b6403cb7f158b66a60fc472da2db043ace2910/src/lib/crypto/kimchi_backend/pasta/basic/kimchi_pasta_basic.ml#L95
    const PARAMS: Params<Self> = Params::<Self> {
        a: ark_ff::field_new!(Fq, "0"),
        b: ark_ff::field_new!(Fq, "5"),
    };
    const SIZE: BigInteger256 = mina_curves::pasta::fields::FqParameters::MODULUS;
    const NROUNDS: usize = BACKEND_TOCK_ROUNDS_N;
    const SRS_DEPTH: usize = 65536;
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
    Clone,
    derive_more::Add,
    derive_more::Sub,
    derive_more::Neg,
    derive_more::Mul,
    derive_more::Div,
    PartialEq,
    Eq,
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
        let GroupAffine::<F> { x, y, .. } = self.to_affine();
        f.debug_struct("InnerCurve")
            .field("x", &x)
            .field("y", &y)
            .finish()
    }
}

impl crate::ToInputs for InnerCurve<Fp> {
    fn to_inputs(&self, inputs: &mut crate::Inputs) {
        let GroupAffine::<Fp> { x, y, .. } = self.to_affine();
        inputs.append_field(x);
        inputs.append_field(y);
    }
}

impl<F: FieldWitness> From<(F, F)> for InnerCurve<F> {
    fn from((x, y): (F, F)) -> Self {
        Self::of_affine(make_group(x, y))
    }
}

impl<F: FieldWitness> InnerCurve<F> {
    pub fn one() -> Self {
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

    pub fn to_affine(&self) -> GroupAffine<F> {
        // Both `affine` below are the same type, but we use `into()` to make it non-generic
        let affine: F::Affine = self.inner.into_affine();
        let affine: GroupAffine<F> = affine.into();
        // OCaml panics on infinity
        // https://github.com/MinaProtocol/mina/blob/3e58e92ea9aeddb41ad3b6e494279891c5f9aa09/src/lib/crypto/kimchi_backend/common/curve.ml#L180
        assert!(!affine.infinity);
        affine
    }

    pub fn of_affine(affine: GroupAffine<F>) -> Self {
        // Both `inner` below are the same type, but we use `into()` to make it generic
        let inner: GroupProjective<F::Parameters> = affine.into_projective();
        let inner: F::Projective = inner.into();
        Self { inner }
    }

    fn fake_random() -> Self {
        // static SEED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

        let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(
            0, // SEED.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        );
        let proj: GroupProjective<F::Parameters> = ark_ff::UniformRand::rand(&mut rng);
        let proj: F::Projective = proj.into();
        Self { inner: proj }
    }

    pub fn random() -> Self {
        Self::fake_random()
        // // Both `proj` below are the same type, but we use `into()` to make it generic
        // let rng = &mut rand::rngs::OsRng;
        // let proj: GroupProjective<F::Parameters> = ark_ff::UniformRand::rand(rng);
        // let proj: F::Projective = proj.into();

        // Self { inner: proj }
    }
}

impl InnerCurve<Fp> {
    // TODO: Remove this
    pub fn rand() -> Self {
        let kp = gen_keypair();
        let point = kp.public.into_point();
        assert!(point.is_on_curve());
        Self::of_affine(point)
    }
}

/// https://github.com/openmina/mina/blob/45c195d72aa8308fcd9fc1c7bc5da36a0c3c3741/src/lib/snarky_curves/snarky_curves.ml#L267
pub fn create_shifted_inner_curve<F>(w: &mut Witness<F>) -> InnerCurve<F>
where
    F: FieldWitness,
{
    w.exists(InnerCurve::<F>::random())
}

impl<F: FieldWitness> ToFieldElements<F> for InnerCurve<F> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let GroupAffine::<F> { x, y, .. } = self.to_affine();
        fields.push(x);
        fields.push(y);
    }
}

impl<F: FieldWitness> Check<F> for InnerCurve<F> {
    // https://github.com/openmina/mina/blob/8f83199a92faa8ff592b7ae5ad5b3236160e8c20/src/lib/snarky_curves/snarky_curves.ml#L167
    fn check(&self, w: &mut Witness<F>) {
        self.to_affine().check(w);
    }
}

impl<F: FieldWitness> Check<F> for GroupAffine<F> {
    // https://github.com/openmina/mina/blob/8f83199a92faa8ff592b7ae5ad5b3236160e8c20/src/lib/snarky_curves/snarky_curves.ml#L167
    fn check(&self, w: &mut Witness<F>) {
        let GroupAffine::<F> { x, y: _, .. } = self;
        let x2 = field::square(*x, w);
        let _x3 = field::mul(x2, *x, w);
        // TODO: Rest of the function doesn't modify witness
    }
}

impl<F: FieldWitness> Check<F> for transaction_union_payload::Tag {
    fn check(&self, _w: &mut Witness<F>) {
        // Does not modify the witness
        // Note: For constraints we need to convert to unpacked union
        // https://github.com/openmina/mina/blob/45c195d72aa8308fcd9fc1c7bc5da36a0c3c3741/src/lib/mina_base/transaction_union_tag.ml#L177
    }
}

impl<F: FieldWitness> Check<F> for transaction_union_payload::TransactionUnion {
    fn check(&self, w: &mut Witness<F>) {
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

        fee.check(w);
        nonce.check(w);
        valid_until.check(w);
        tag.check(w);
        amount.check(w);
    }
}

impl<F: FieldWitness> Check<F> for pending_coinbase::Stack {
    fn check(&self, _w: &mut Witness<F>) {
        // Does not modify the witness
    }
}

impl<F: FieldWitness> Check<F> for TokenSymbol {
    fn check(&self, w: &mut Witness<F>) {
        let field: F = self.to_field();
        scalar_challenge::to_field_checked_prime::<F, 48>(field, w);
    }
}

impl<F: FieldWitness> Check<F> for Box<Account> {
    fn check(&self, w: &mut Witness<F>) {
        let Account {
            public_key: _,
            token_id: _,
            token_symbol,
            balance,
            nonce,
            receipt_chain_hash: _,
            delegate: _,
            voting_for: _,
            timing,
            permissions: _,
            zkapp: _,
        } = &**self;

        token_symbol.check(w);
        balance.check(w);
        nonce.check(w);
        timing.check(w);
    }
}

impl<F: FieldWitness> Check<F> for crate::Timing {
    fn check(&self, w: &mut Witness<F>) {
        let TimingAsRecord {
            is_timed: _,
            initial_minimum_balance,
            cliff_time,
            cliff_amount,
            vesting_period,
            vesting_increment,
        } = self.to_record();

        initial_minimum_balance.check(w);
        cliff_time.check(w);
        cliff_amount.check(w);
        vesting_period.check(w);
        vesting_increment.check(w);
    }
}

impl<F: FieldWitness> Check<F> for crate::MerklePath {
    fn check(&self, _w: &mut Witness<F>) {
        // Does not modify the witness
    }
}

impl<F: FieldWitness, T: Check<F>> Check<F> for Vec<T> {
    fn check(&self, w: &mut Witness<F>) {
        self.iter().for_each(|v| v.check(w))
    }
}

impl<F: FieldWitness, T: Check<F>> Check<F> for Box<[T]> {
    fn check(&self, w: &mut Witness<F>) {
        self.iter().for_each(|v| v.check(w))
    }
}

impl<F: FieldWitness, A: Check<F>, B: Check<F>> Check<F> for (A, B) {
    fn check(&self, w: &mut Witness<F>) {
        let (a, b) = self;
        a.check(w);
        b.check(w);
    }
}

impl<F: FieldWitness> Check<F> for ReceiptChainHash {
    fn check(&self, _w: &mut Witness<F>) {
        // Does not modify the witness
    }
}

impl<F: FieldWitness> Check<F> for Sgn {
    fn check(&self, _w: &mut Witness<F>) {
        // Does not modify the witness
    }
}

impl<F: FieldWitness> Check<F> for CompressedPubKey {
    fn check(&self, _w: &mut Witness<F>) {
        // Does not modify the witness
    }
}

impl<F: FieldWitness> Check<F> for TokenId {
    fn check(&self, _w: &mut Witness<F>) {
        // Does not modify the witness
    }
}

impl<F: FieldWitness> Check<F> for v2::MinaBasePendingCoinbaseStackVersionedStableV1 {
    fn check(&self, _w: &mut Witness<F>) {
        // Does not modify the witness
    }
}

impl<F: FieldWitness> Check<F> for &[AllEvals<F>] {
    fn check(&self, _w: &mut Witness<F>) {
        // Does not modify the witness
    }
}

pub mod field {
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

    pub fn const_mul<F>(x: F, y: F) -> F
    where
        F: FieldWitness,
    {
        x * y
    }

    pub fn muls<F>(xs: &[F], w: &mut Witness<F>) -> F
    where
        F: FieldWitness,
    {
        xs.iter()
            .copied()
            .reduce(|acc, v| w.exists(acc * v))
            .expect("invalid param")
    }

    pub fn div<F>(x: F, y: F, w: &mut Witness<F>) -> F
    where
        F: FieldWitness,
    {
        w.exists(x / y)
    }

    // TODO: Do we need `div` above ?
    pub fn div_by_inv<F>(x: F, y: F, w: &mut Witness<F>) -> F
    where
        F: FieldWitness,
    {
        let y_inv = w.exists(y.inverse().unwrap_or_else(F::zero));
        mul(x, y_inv, w)
    }

    pub fn sub<F>(x: F, y: F, w: &mut Witness<F>) -> F
    where
        F: FieldWitness,
    {
        w.exists(x - y)
    }

    pub fn add<F>(x: F, y: F, w: &mut Witness<F>) -> F
    where
        F: FieldWitness,
    {
        w.exists(x + y)
    }

    pub fn const_add<F>(x: F, y: F) -> F
    where
        F: FieldWitness,
    {
        x + y
    }

    pub fn equal<F: FieldWitness>(x: F, y: F, w: &mut Witness<F>) -> Boolean {
        let z = x - y;

        let (boolean, r, inv) = if x == y {
            (Boolean::True, F::one(), F::zero())
        } else {
            (Boolean::False, F::zero(), z.inverse().unwrap())
        };
        w.exists([r, inv]);

        boolean
    }

    pub fn compare<F: FieldWitness>(bit_length: u64, a: F, b: F, w: &mut Witness<F>) -> (Boolean, Boolean) {
        let two_to_the = |n: usize| {
            (0..n).fold(F::one(), |acc, _| acc.double())
        };

        let bit_length = bit_length as usize;
        let alpha_packed = {
            two_to_the(bit_length) + b - a
        };
        let alpha = w.exists(field_to_bits2(alpha_packed, bit_length + 1));
        let (less_or_equal, prefix) = alpha.split_last().unwrap();

        let less_or_equal = less_or_equal.to_boolean();
        let prefix = prefix.iter().map(|b| b.to_boolean()).collect::<Vec<_>>();

        let not_all_zeros = Boolean::any(&prefix, w);
        let less = less_or_equal.and(&not_all_zeros, w);

        (less, less_or_equal)
    }

    pub fn assert_lt<F: FieldWitness>(bit_length: u64, x: F, y: F, w: &mut Witness<F>) {
        compare(bit_length, x, y, w);
    }
}

#[allow(unused)]
fn dummy_constraints<F>(w: &mut Witness<F>)
where
    F: FieldWitness,
{
    use crate::proofs::public_input::plonk_checks::ShiftingValue;

    let x: F = w.exists(F::from(3u64));
    let g: InnerCurve<F> = w.exists(InnerCurve::<F>::one());

    let _ = w.to_field_checked_prime::<16>(x);

    // TODO: Fix `F, F` below
    plonk_curve_ops::scale_fast::<F, F, 5>(g.to_affine(), F::Shifting::of_raw(x), w);
    plonk_curve_ops::scale_fast::<F, F, 5>(g.to_affine(), F::Shifting::of_raw(x), w);
    scalar_challenge::endo::<F, F, 4>(g.to_affine(), x, w);
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

    impl<F: FieldWitness> Default for LegacyInput<F> {
        fn default() -> Self {
            Self::new()
        }
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

    use mina_poseidon::constants::PlonkSpongeConstantsKimchi;
    use mina_poseidon::constants::SpongeConstants;
    use mina_poseidon::poseidon::{ArithmeticSpongeParams, SpongeState};

    use super::*;

    #[derive(Clone)]
    pub struct Sponge<F: FieldWitness, C: SpongeConstants = PlonkSpongeConstantsKimchi> {
        pub state: [F; 3],
        pub sponge_state: SpongeState,
        params: &'static ArithmeticSpongeParams<F>,
        nabsorb: usize,
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
                nabsorb: 0,
                _constants: PhantomData,
            }
        }

        pub fn new() -> Self {
            Self::new_with_state([F::zero(); 3], F::get_params2())
        }

        pub fn absorb(&mut self, x: &[F], w: &mut Witness<F>) {
            // Hack to know when to ignore witness
            // That should be removed once we use `cvar`
            let mut first = true;

            for x in x.iter() {
                match self.sponge_state {
                    SpongeState::Absorbed(n) => {
                        if n == C::SPONGE_RATE {
                            eprintln!("Sponge::Absorbed_A({})", n);
                            self.poseidon_block_cipher(first, w);
                            self.sponge_state = SpongeState::Absorbed(1);
                            self.state[0].add_assign(x);
                            w.exists(self.state[0]); // Good
                            first = false;
                        } else {
                            eprintln!("Sponge::Absorbed_B({})", n);
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

        pub fn absorb2(&mut self, x: &[F], w: &mut Witness<F>) {
            // Hack to know when to ignore witness
            // That should be removed once we use `cvar`
            let mut first = true;

            for x in x.iter() {
                match self.sponge_state {
                    SpongeState::Absorbed(n) => {
                        if n == C::SPONGE_RATE {
                            eprintln!("Sponge::Absorbed2_A({})", n);
                            self.poseidon_block_cipher(first, w);
                            self.sponge_state = SpongeState::Absorbed(1);
                            self.state[0].add_assign(x);
                            w.exists(self.state[0]); // Good
                            first = false;
                        } else {
                            eprintln!("Sponge::Absorbed2_B({})", n);
                            self.sponge_state = SpongeState::Absorbed(n + 1);
                            self.state[n].add_assign(x);
                            if self.nabsorb > 2 {
                                w.exists(self.state[n]); // Good
                            }
                        }
                    }
                    SpongeState::Squeezed(_n) => {
                        self.state[0].add_assign(x);
                        w.exists(self.state[0]); // Unknown
                        self.sponge_state = SpongeState::Absorbed(1);
                    }
                }
                self.nabsorb += 1;
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
                    // legacy

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
                    // non-legacy

                    w.exists(self.state);
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
                *state_i = sbox::<F, C>(*state_i, push_witness, w);
            }
            self.state = apply_mds_matrix::<F, C>(self.params, &self.state);
            for (i, x) in self.params.round_constants[r].iter().enumerate() {
                self.state[i].add_assign(x);
                if C::PERM_SBOX == 5 {
                    // legacy
                    w.exists(self.state[i]); // Good
                }
            }
            if C::PERM_SBOX == 7 {
                // non-legacy
                w.exists(self.state);
            }
        }
    }

    pub fn sbox<F: FieldWitness, C: SpongeConstants>(
        x: F,
        push_witness: bool,
        w: &mut Witness<F>,
    ) -> F {
        if C::PERM_SBOX == 5 {
            // legacy

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
        } else if C::PERM_SBOX == 7 {
            // non-legacy

            let mut res = x.square();
            res *= x;
            let res = res.square();
            res * x
        } else {
            unimplemented!()
        }
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

fn double_group<F: FieldWitness>(group: GroupAffine<F>, w: &mut Witness<F>) -> GroupAffine<F> {
    let GroupAffine::<F> { x: ax, y: ay, .. } = group;
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
fn group_to_witness<F: FieldWitness>(group: GroupAffine<F>, w: &mut Witness<F>) -> GroupAffine<F> {
    // We don't want to call `GroupAffine::check` here
    let GroupAffine::<F> { x, y, .. } = &group;
    w.exists(*x);
    w.exists(*y);
    group
}

pub fn scale_non_constant<F: FieldWitness, const N: usize>(
    mut g: GroupAffine<F>,
    bits: &[bool; N],
    init: &InnerCurve<F>,
    w: &mut Witness<F>,
) -> GroupAffine<F> {
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
    let GroupAffine::<F> { x: x1, y: y1, .. } = t1.to_affine();
    let GroupAffine::<F> { x: x2, y: y2, .. } = t2.to_affine();
    let GroupAffine::<F> { x: x3, y: y3, .. } = t3.to_affine();
    let GroupAffine::<F> { x: x4, y: y4, .. } = t4.to_affine();

    (lookup_one(x1, x2, x3, x4), lookup_one(y1, y2, y3, y4))
}

fn lookup_single_bit<F: FieldWitness>(b: bool, (t1, t2): (InnerCurve<F>, InnerCurve<F>)) -> (F, F) {
    let lookup_one = |a1: F, a2: F| a1 + (F::from(b) * (a2 - a1));

    let GroupAffine::<F> { x: x1, y: y1, .. } = t1.to_affine();
    let GroupAffine::<F> { x: x2, y: y2, .. } = t2.to_affine();

    (lookup_one(x1, x2), lookup_one(y1, y2))
}

pub fn scale_known<F: FieldWitness, const N: usize>(
    t: GroupAffine<F>,
    bits: &[bool; N],
    init: &InnerCurve<F>,
    w: &mut Witness<F>,
) -> GroupAffine<F> {
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
                let [term_y, term_x] = w.exists([term_y, term_x]);
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
                let [term_y, term_x] = w.exists([term_y, term_x]);
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

pub trait ToBoolean {
    fn to_boolean(&self) -> Boolean;
}

impl ToBoolean for bool {
    fn to_boolean(&self) -> Boolean {
        if *self {
            Boolean::True
        } else {
            Boolean::False
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boolean {
    True,
    False,
}

impl Boolean {
    pub fn of_field<F: FieldWitness>(field: F) -> Self {
        if field.is_zero() {
            Self::False
        } else {
            Self::True
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Boolean::True => true,
            Boolean::False => false,
        }
    }

    pub fn neg(&self) -> Self {
        match self {
            Boolean::False => Boolean::True,
            Boolean::True => Boolean::False,
        }
    }

    pub fn to_field<F: FieldWitness>(self) -> F {
        F::from(self.as_bool())
    }

    fn mul<F: FieldWitness>(&self, other: &Self, w: &mut Witness<F>) -> Self {
        let result: F = self.to_field::<F>() * other.to_field::<F>();
        w.exists(result);
        if result.is_zero() {
            Self::False
        } else {
            assert_eq!(result, F::one());
            Self::True
        }
    }

    pub fn and<F: FieldWitness>(&self, other: &Self, w: &mut Witness<F>) -> Self {
        self.mul(other, w)
    }

    /// Same as `Self::and` but doesn't push values to the witness
    pub fn const_and(&self, other: &Self) -> Self {
        (self.as_bool() && other.as_bool()).to_boolean()
    }

    pub fn or<F: FieldWitness>(&self, other: &Self, w: &mut Witness<F>) -> Self {
        let both_false = self.neg().and(&other.neg(), w);
        both_false.neg()
    }

    /// Same as `Self::or` but doesn't push values to the witness
    pub fn const_or(&self, other: &Self) -> Self {
        (self.as_bool() || other.as_bool()).to_boolean()
    }

    pub fn all<F: FieldWitness>(x: &[Self], w: &mut Witness<F>) -> Self {
        match x {
            [] => Self::True,
            [b1] => *b1,
            [b1, b2] => b1.and(b2, w),
            bs => {
                let len = F::from(bs.len() as u64);
                let sum = bs.iter().fold(0u64, |acc, b| {
                    acc + match b {
                        Boolean::True => 1,
                        Boolean::False => 0,
                    }
                });
                field::equal(len, F::from(sum), w)
            }
        }
    }

    pub fn const_all<F: FieldWitness>(x: &[Self]) -> Self {
        match x {
            [] => Self::True,
            [b1] => *b1,
            [b1, b2] => b1.const_and(b2),
            bs => {
                let len = F::from(bs.len() as u64);
                let sum = bs.iter().fold(0u64, |acc, b| {
                    acc + match b {
                        Boolean::True => 1,
                        Boolean::False => 0,
                    }
                });
                (len == F::from(sum)).to_boolean()
            }
        }
    }

    // For non-constant
    pub fn lxor<F: FieldWitness>(&self, other: &Self, w: &mut Witness<F>) -> Self {
        let result = (self.as_bool() ^ other.as_bool()).to_boolean();
        w.exists(result.to_field::<F>());
        result
    }

    pub fn const_lxor(&self, other: &Self) -> Self {
        (self.as_bool() ^ other.as_bool()).to_boolean()
    }

    pub fn equal<F: FieldWitness>(&self, other: &Self, w: &mut Witness<F>) -> Self {
        self.lxor(other, w).neg()
    }

    pub fn const_equal(&self, other: &Self) -> Self {
        (self.as_bool() == other.as_bool()).to_boolean()
    }

    pub fn const_any<F: FieldWitness>(x: &[Self]) -> Self {
        match x {
            [] => Self::False,
            [b1] => *b1,
            [b1, b2] => b1.const_or(b2),
            bs => {
                let sum = bs.iter().fold(0u64, |acc, b| {
                    acc + match b {
                        Boolean::True => 1,
                        Boolean::False => 0,
                    }
                });
                (F::from(sum) == F::zero()).to_boolean().neg()
            }
        }
    }

    pub fn any<F: FieldWitness>(x: &[Self], w: &mut Witness<F>) -> Self {
        match x {
            [] => Self::False,
            [b1] => *b1,
            [b1, b2] => b1.or(b2, w),
            bs => {
                let sum = bs.iter().fold(0u64, |acc, b| {
                    acc + match b {
                        Boolean::True => 1,
                        Boolean::False => 0,
                    }
                });
                field::equal(F::from(sum), F::zero(), w).neg()
            }
        }
    }

    // Part of utils.inv
    fn assert_non_zero<F: FieldWitness>(v: F, w: &mut Witness<F>) {
        if v.is_zero() {
            w.exists(v);
        } else {
            w.exists(v.inverse().unwrap());
        }
    }

    pub fn assert_any<F: FieldWitness>(bs: &[Self], w: &mut Witness<F>) {
        let num_true = bs.iter().fold(0u64, |acc, b| {
            acc + match b {
                Boolean::True => 1,
                Boolean::False => 0,
            }
        });
        Self::assert_non_zero::<F>(F::from(num_true), w)
    }
}

impl<F: FieldWitness> ToFieldElements<F> for Boolean {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        fields.push(self.to_field::<F>());
    }
}

impl<F: FieldWitness> Check<F> for Boolean {
    fn check(&self, _w: &mut Witness<F>) {
        // Does not modify the witness
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

fn lt_binary<F: FieldWitness>(xs: &[bool], ys: &[bool]) -> ExprBinary<Boolean> {
    match (xs, ys) {
        ([], []) => ExprBinary::Lit(Boolean::False),
        ([_x], [false]) => ExprBinary::Lit(Boolean::False),
        ([x], [true]) => ExprBinary::Lit(x.to_boolean().neg()),
        ([x1, _x2], [true, false]) => ExprBinary::Lit(x1.to_boolean().neg()),
        ([_x1, _x2], [false, false]) => ExprBinary::Lit(Boolean::False),
        ([x, xs @ ..], [false, ys @ ..]) => {
            ExprBinary::And(x.to_boolean().neg(), Box::new(lt_binary::<F>(xs, ys)))
        }
        ([x, xs @ ..], [true, ys @ ..]) => {
            ExprBinary::Or(x.to_boolean().neg(), Box::new(lt_binary::<F>(xs, ys)))
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
                of_binary::<F>(t),
            ]),
            _ => ExprNary::And(vec![ExprNary::Lit(*x), of_binary::<F>(t)]),
        },
        ExprBinary::Or(x, t) => match &**t {
            ExprBinary::Or(y, t) => ExprNary::Or(vec![
                ExprNary::Lit(*x),
                ExprNary::Lit(*y),
                of_binary::<F>(t),
            ]),
            _ => ExprNary::Or(vec![ExprNary::Lit(*x), of_binary::<F>(t)]),
        },
    }
}

impl ExprNary<Boolean> {
    fn eval<F: FieldWitness>(&self, w: &mut Witness<F>) -> Boolean {
        match self {
            ExprNary::Lit(x) => *x,
            ExprNary::And(xs) => {
                let xs: Vec<_> = xs.iter().map(|x| Self::eval::<F>(x, w)).collect();
                Boolean::all::<F>(&xs, w)
            }
            ExprNary::Or(xs) => {
                let xs: Vec<_> = xs.iter().map(|x| Self::eval::<F>(x, w)).collect();
                Boolean::any::<F>(&xs, w)
            }
        }
    }
}

fn lt_bitstring_value<F: FieldWitness>(
    xs: &[bool; 255],
    ys: &[bool; 255],
    w: &mut Witness<F>,
) -> Boolean {
    let value = of_binary::<F>(&lt_binary::<F>(xs, ys));
    value.eval(w)
}

fn unpack_full<F: FieldWitness>(x: F, w: &mut Witness<F>) -> [bool; 255] {
    let bits_lsb = w.exists(field_to_bits::<F, 255>(x));

    let bits_msb = {
        let mut bits = bits_lsb;
        bits.reverse(); // msb
        bits
    };

    let size_msb = {
        let mut size = bigint_to_bits::<255>(F::SIZE);
        size.reverse(); // msb
        size
    };

    lt_bitstring_value::<F>(&bits_msb, &size_msb, w);

    bits_lsb
}

fn is_even<F: FieldWitness>(y: F, w: &mut Witness<F>) -> Boolean {
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

    lt_bitstring_value::<F>(&bits_msb, &size_msb, w)
}

pub struct CompressedPubKeyVar<F: FieldWitness> {
    pub x: F,
    pub is_odd: bool,
}

pub fn compress_var<F: FieldWitness>(
    v: &GroupAffine<F>,
    w: &mut Witness<F>,
) -> CompressedPubKeyVar<F> {
    let GroupAffine::<F> { x, y, .. } = v;

    let is_odd = {
        let bits = unpack_full(*y, w);
        bits[0]
    };

    CompressedPubKeyVar { x: *x, is_odd }
}

pub fn decompress_var(pk: &CompressedPubKey, w: &mut Witness<Fp>) -> PubKey {
    let CompressedPubKey { x, is_odd: _ } = pk;
    let GroupAffine::<Fp> { y, .. } = decompress_pk(pk).unwrap().into_point();

    w.exists(y);

    let point = make_group(*x, y);
    point.check(w);

    let _is_odd2 = {
        let bits = unpack_full(y, w);
        bits[0]
    };
    PubKey::from_point_unsafe(point)
}

pub mod transaction_snark {
    use std::ops::Neg;

    use crate::{
        checked_equal_compressed_key, checked_equal_compressed_key_const_and,
        checked_verify_merkle_path,
        proofs::{
            numbers::{
                currency::{
                    CheckedAmount, CheckedBalance, CheckedCurrency, CheckedFee, CheckedSigned,
                },
                nat::{CheckedNat, CheckedSlot, CheckedSlotSpan},
            },
            witness::legacy_input::CheckedLegacyInput,
        },
        scan_state::{
            currency::Sgn,
            fee_excess::CheckedFeeExcess,
            pending_coinbase,
            transaction_logic::{checked_cons_signed_command_payload, Coinbase},
        },
        sparse_ledger::SparseLedger,
        AccountId, PermissionTo, PermsConst, Timing, TimingAsRecordChecked, ToInputs,
    };
    use ark_ff::Zero;
    use mina_signer::PubKey;

    use crate::scan_state::{
        currency,
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
        coinbase_amount: currency::Amount::from_u64(720000000000),
        supercharged_coinbase_factor: 2,
        account_creation_fee: currency::Fee::from_u64(1000000000),
        fork: None,
    };

    // let res : (a, b, c) Poly.t =
    //   { Poly.k = to_length k
    //   ; delta = to_length delta
    //   ; block_window_duration_ms = to_timespan block_window_duration_ms
    //   ; slots_per_sub_window = to_length slots_per_sub_window
    //   ; slots_per_window = to_length slots_per_window
    //   ; sub_windows_per_window = to_length sub_windows_per_window
    //   ; slots_per_epoch = to_length slots_per_epoch
    //   ; grace_period_end = to_length grace_period_end
    //   ; slot_duration_ms = to_timespan Slot.duration_ms
    //   ; epoch_duration = to_timespan Epoch.duration
    //   ; checkpoint_window_slots_per_year = to_length zero
    //   ; checkpoint_window_size_in_slots = to_length zero
    //   ; delta_duration = to_timespan delta_duration
    //   ; genesis_state_timestamp = protocol_constants.genesis_state_timestamp
    //   }
    // in

    mod user_command_failure {
        use crate::scan_state::{
            currency::Magnitude,
            transaction_logic::{
                timing_error_to_user_command_status, validate_timing, TransactionFailure,
            },
        };

        use super::*;

        const NUM_FIELDS: usize = 8;

        pub struct Failure {
            pub predicate_failed: bool,                 // User commands
            pub source_not_present: bool,               // User commands
            pub receiver_not_present: bool,             // Delegate
            pub amount_insufficient_to_create: bool,    // Payment only
            pub token_cannot_create: bool,              // Payment only, token<>default
            pub source_insufficient_balance: bool,      // Payment only
            pub source_minimum_balance_violation: bool, // Payment only
            pub source_bad_timing: bool,                // Payment only
        }

        impl<F: FieldWitness> ToFieldElements<F> for Failure {
            fn to_field_elements(&self, fields: &mut Vec<F>) {
                let list = self.to_list();
                list.to_field_elements(fields)
            }
        }

        impl<F: FieldWitness> Check<F> for Failure {
            fn check(&self, _w: &mut Witness<F>) {
                // Nothing
            }
        }

        impl Failure {
            fn empty() -> Self {
                Self {
                    predicate_failed: false,
                    source_not_present: false,
                    receiver_not_present: false,
                    amount_insufficient_to_create: false,
                    token_cannot_create: false,
                    source_insufficient_balance: false,
                    source_minimum_balance_violation: false,
                    source_bad_timing: false,
                }
            }

            pub fn to_list(&self) -> [Boolean; NUM_FIELDS] {
                let Self {
                    predicate_failed,
                    source_not_present,
                    receiver_not_present,
                    amount_insufficient_to_create,
                    token_cannot_create,
                    source_insufficient_balance,
                    source_minimum_balance_violation,
                    source_bad_timing,
                } = self;

                [
                    predicate_failed.to_boolean(),
                    source_not_present.to_boolean(),
                    receiver_not_present.to_boolean(),
                    amount_insufficient_to_create.to_boolean(),
                    token_cannot_create.to_boolean(),
                    source_insufficient_balance.to_boolean(),
                    source_minimum_balance_violation.to_boolean(),
                    source_bad_timing.to_boolean(),
                ]
            }
        }

        pub fn compute_as_prover<F: FieldWitness>(
            txn_global_slot: CheckedSlot<F>,
            txn: &TransactionUnion,
            sparse_ledger: &SparseLedger,
            w: &mut Witness<F>,
        ) -> Failure {
            w.exists(compute_as_prover_impl(
                txn_global_slot.to_inner(),
                txn,
                sparse_ledger,
            ))
        }

        /// NOTE: Unchecked computation
        // TODO: Returns errors instead of panics
        fn compute_as_prover_impl(
            txn_global_slot: currency::Slot,
            txn: &TransactionUnion,
            sparse_ledger: &SparseLedger,
        ) -> Failure {
            use transaction_union_payload::Tag::*;

            let _fee_token = &txn.payload.common.fee_token;
            let token = &txn.payload.body.token_id;
            let fee_payer =
                AccountId::create(txn.payload.common.fee_payer_pk.clone(), token.clone());
            let source = AccountId::create(txn.payload.body.source_pk.clone(), token.clone());
            let receiver = AccountId::create(txn.payload.body.receiver_pk.clone(), token.clone());

            let mut fee_payer_account = sparse_ledger.get_account(&fee_payer);
            let source_account = sparse_ledger.get_account(&source);
            let receiver_account = sparse_ledger.get_account(&receiver);

            // compute_unchecked
            let TransactionUnion {
                payload,
                signer: _,
                signature: _,
            } = txn;

            if let FeeTransfer | Coinbase = payload.body.tag {
                return Failure::empty();
            };

            fee_payer_account.balance = fee_payer_account
                .balance
                .sub_amount(currency::Amount::of_fee(&payload.common.fee))
                .unwrap();

            let predicate_failed = if payload.common.fee_payer_pk == payload.body.source_pk {
                false
            } else {
                match payload.body.tag {
                    Payment | StakeDelegation => true,
                    FeeTransfer | Coinbase => panic!(), // Checked above
                }
            };

            match payload.body.tag {
                FeeTransfer | Coinbase => panic!(), // Checked above
                StakeDelegation => {
                    let receiver_account = if receiver == fee_payer {
                        &fee_payer_account
                    } else {
                        &receiver_account
                    };

                    let receiver_not_present = {
                        let id = receiver_account.id();
                        if id.is_empty() {
                            true
                        } else if receiver == id {
                            false
                        } else {
                            panic!("bad receiver account ID")
                        }
                    };

                    let source_account = if source == fee_payer {
                        &fee_payer_account
                    } else {
                        &source_account
                    };

                    let source_not_present = {
                        let id = source_account.id();
                        if id.is_empty() {
                            true
                        } else if source == id {
                            false
                        } else {
                            panic!("bad source account ID")
                        }
                    };

                    Failure {
                        predicate_failed,
                        source_not_present,
                        receiver_not_present,
                        amount_insufficient_to_create: false,
                        token_cannot_create: false,
                        source_insufficient_balance: false,
                        source_minimum_balance_violation: false,
                        source_bad_timing: false,
                    }
                }
                Payment => {
                    let receiver_account = if receiver == fee_payer {
                        &fee_payer_account
                    } else {
                        &receiver_account
                    };

                    let receiver_needs_creating = {
                        let id = receiver_account.id();
                        if id.is_empty() {
                            true
                        } else if id == receiver {
                            false
                        } else {
                            panic!("bad receiver account ID");
                        }
                    };

                    let token_is_default = true;
                    let token_cannot_create = receiver_needs_creating && !token_is_default;

                    let amount_insufficient_to_create = {
                        let creation_amount =
                            currency::Amount::of_fee(&CONSTRAINT_CONSTANTS.account_creation_fee);
                        receiver_needs_creating
                            && payload.body.amount.checked_sub(&creation_amount).is_none()
                    };

                    let fee_payer_is_source = fee_payer == source;
                    let source_account = if fee_payer_is_source {
                        &fee_payer_account
                    } else {
                        &source_account
                    };

                    let source_not_present = {
                        let id = source_account.id();
                        if id.is_empty() {
                            true
                        } else if source == id {
                            false
                        } else {
                            panic!("bad source account ID");
                        }
                    };

                    let source_insufficient_balance = !fee_payer_is_source
                        && if source == receiver {
                            receiver_needs_creating
                        } else {
                            source_account.balance.to_amount() < payload.body.amount
                        };

                    let timing_or_error =
                        validate_timing(source_account, payload.body.amount, &txn_global_slot);

                    let source_minimum_balance_violation = matches!(
                        timing_error_to_user_command_status(timing_or_error.clone()),
                        Err(TransactionFailure::SourceMinimumBalanceViolation),
                    );

                    let source_bad_timing = !fee_payer_is_source
                        && !source_insufficient_balance
                        && timing_or_error.is_err();

                    Failure {
                        predicate_failed,
                        source_not_present,
                        receiver_not_present: false,
                        amount_insufficient_to_create,
                        token_cannot_create,
                        source_insufficient_balance,
                        source_minimum_balance_violation,
                        source_bad_timing,
                    }
                }
            }
        }
    }

    pub fn checked_legacy_hash(param: &str, inputs: LegacyInput<Fp>, w: &mut Witness<Fp>) -> Fp {
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

    pub fn checked_hash(param: &str, inputs: &[Fp], w: &mut Witness<Fp>) -> Fp {
        // We hash the parameter first, without introducing values to the witness
        let initial_state: [Fp; 3] = {
            use crate::{param_to_field, ArithmeticSponge, PlonkSpongeConstantsKimchi, Sponge};

            let mut sponge =
                ArithmeticSponge::<Fp, PlonkSpongeConstantsKimchi>::new(Fp::get_params());
            sponge.absorb(&[param_to_field(param)]);
            sponge.squeeze();
            sponge.state
        };

        // dbg!(inputs);

        let mut sponge = poseidon::Sponge::<Fp>::new_with_state(initial_state, Fp::get_params2());
        sponge.absorb(inputs, w);
        sponge.squeeze(w)
    }

    fn checked_signature_hash(
        mut inputs: LegacyInput<Fp>,
        signer: &PubKey,
        signature: &Signature,
        w: &mut Witness<Fp>,
    ) -> [bool; 255] {
        let GroupAffine::<Fp> { x: px, y: py, .. } = signer.point();
        let Signature { rx, s: _ } = signature;

        inputs.append_field(*px);
        inputs.append_field(*py);
        inputs.append_field(*rx);
        let hash = checked_legacy_hash("CodaSignature", inputs, w);

        w.exists(field_to_bits::<_, 255>(hash))
    }

    fn check_signature(
        shifted: &InnerCurve<Fp>,
        payload: &TransactionUnionPayload,
        is_user_command: Boolean,
        signer: &PubKey,
        signature: &Signature,
        w: &mut Witness<Fp>,
    ) {
        let inputs = payload.to_checked_legacy_input_owned(w);
        let hash = checked_signature_hash(inputs, signer, signature, w);

        // negate
        let public_key = {
            let GroupAffine::<Fp> { x, y, .. } = signer.point();
            let y = w.exists(y.neg()); // This is actually made in the `scale` call below in OCaml
            make_group::<Fp>(*x, y)
        };

        let e_pk = scale_non_constant::<Fp, 255>(public_key, &hash, shifted, w);

        let Signature { rx: _, s } = signature;
        let bits: [bool; 255] = field_to_bits::<_, 255>(*s);
        let one: GroupAffine<Fp> = InnerCurve::<Fp>::one().to_affine();
        let s_g_e_pk = scale_known(one, &bits, &InnerCurve::of_affine(e_pk), w);

        let GroupAffine::<Fp> { x: rx, y: ry, .. } = {
            let neg_shifted = shifted.to_affine().neg();
            w.exists(neg_shifted.y);
            w.add_fast(neg_shifted, s_g_e_pk)
        };

        let y_even = is_even(ry, w);
        let r_correct = field::equal(signature.rx, rx, w);

        let verifies = y_even.and(&r_correct, w);

        Boolean::assert_any(&[is_user_command.neg(), verifies][..], w);
    }

    fn add_burned_tokens<F: FieldWitness>(
        acc_burned_tokens: CheckedAmount<F>,
        amount: CheckedAmount<F>,
        is_coinbase_or_fee_transfer: Boolean,
        update_account: Boolean,
        is_const_add_flagged: bool,
        w: &mut Witness<F>,
    ) -> CheckedAmount<F> {
        let accumulate_burned_tokens =
            Boolean::all(&[is_coinbase_or_fee_transfer, update_account.neg()], w);

        let (amt, overflow) = if is_const_add_flagged {
            acc_burned_tokens.const_add_flagged(&amount, w)
        } else {
            acc_burned_tokens.add_flagged(&amount, w)
        };

        Boolean::assert_any(&[accumulate_burned_tokens.neg(), overflow.neg()], w);

        w.exists_no_check(match accumulate_burned_tokens {
            Boolean::True => amt,
            Boolean::False => acc_burned_tokens,
        })
    }

    fn checked_min_balance_at_slot<F: FieldWitness>(
        global_slot: &CheckedSlot<F>,
        cliff_time: &CheckedSlot<F>,
        cliff_amount: &CheckedAmount<F>,
        vesting_period: &CheckedSlotSpan<F>,
        vesting_increment: &CheckedAmount<F>,
        initial_minimum_balance: &CheckedBalance<F>,
        w: &mut Witness<F>,
    ) -> CheckedBalance<F> {
        let before_cliff = global_slot.less_than(cliff_time, w);

        let else_value = {
            let (_, slot_diff) = global_slot.diff_or_zero(cliff_time, w);

            let cliff_decrement = cliff_amount;
            let min_balance_less_cliff_decrement =
                initial_minimum_balance.sub_amount_or_zero(cliff_decrement, w);

            let (num_periods, _) = slot_diff.div_mod(vesting_period, w);

            let vesting_decrement = CheckedAmount::from_field(field::mul(
                num_periods.to_field(),
                vesting_increment.to_field(),
                w,
            ));

            min_balance_less_cliff_decrement.sub_amount_or_zero(&vesting_decrement, w)
        };

        w.exists_no_check(match before_cliff {
            Boolean::True => initial_minimum_balance.clone(),
            Boolean::False => else_value,
        })
    }

    fn check_timing<F: FieldWitness, Fun>(
        account: &Account,
        txn_amount: Option<&CheckedAmount<F>>,
        txn_global_slot: CheckedSlot<F>,
        timed_balance_check: Fun,
        w: &mut Witness<F>,
    ) -> (CheckedBalance<F>, Timing)
    where
        Fun: Fn(Boolean, &mut Witness<F>),
    {
        let TimingAsRecordChecked {
            is_timed,
            initial_minimum_balance,
            cliff_time,
            cliff_amount,
            vesting_period,
            vesting_increment,
        } = account.timing.to_record_checked::<F>();

        let curr_min_balance = checked_min_balance_at_slot(
            &txn_global_slot,
            &cliff_time,
            &cliff_amount,
            &vesting_period,
            &vesting_increment,
            &initial_minimum_balance,
            w,
        );

        let account_balance = account.balance.to_checked();
        let proposed_balance = match txn_amount {
            Some(txn_amount) => {
                let (proposed_balance, _underflow) =
                    account_balance.sub_amount_flagged(txn_amount, w);
                proposed_balance
            }
            None => account_balance,
        };

        let sufficient_timed_balance = proposed_balance.gte(&curr_min_balance, w);

        {
            let ok = Boolean::any(&[is_timed.neg(), sufficient_timed_balance], w);
            timed_balance_check(ok, w);
        }

        let is_timed_balance_zero = field::equal(curr_min_balance.to_field(), F::zero(), w);

        let is_untimed = is_timed.neg().or(&is_timed_balance_zero, w);

        let timing = w.exists_no_check(match is_untimed {
            Boolean::True => Timing::Untimed,
            Boolean::False => account.timing.clone(),
        });

        (curr_min_balance, timing)
    }

    #[allow(unused)] // TODO: Remove
    fn apply_tagged_transaction(
        shifted: &InnerCurve<Fp>,
        _fee_payment_root: Fp,
        global_slot: currency::Slot,
        pending_coinbase_init: &v2::MinaBasePendingCoinbaseStackVersionedStableV1,
        pending_coinbase_stack_before: &pending_coinbase::Stack,
        pending_coinbase_after: &pending_coinbase::Stack,
        state_body: &MinaStateProtocolStateBodyValueStableV2,
        tx: &TransactionUnion,
        sparse_ledger: &SparseLedger,
        w: &mut Witness<Fp>,
    ) -> (
        Fp,
        CheckedSigned<Fp, CheckedAmount<Fp>>,
        CheckedSigned<Fp, CheckedAmount<Fp>>,
    ) {
        let TransactionUnion {
            payload,
            signer,
            signature,
        } = tx;

        let global_slot = global_slot.to_checked();

        let mut ledger = sparse_ledger.copy_content();

        let tag = payload.body.tag.clone();
        let is_user_command = tag.is_user_command().to_boolean();

        check_signature(shifted, payload, is_user_command, signer, signature, w);

        let _signer_pk = compress_var(signer.point(), w);

        let is_payment = tag.is_payment().to_boolean();
        let is_stake_delegation = tag.is_stake_delegation().to_boolean();
        let is_fee_transfer = tag.is_fee_transfer().to_boolean();
        let is_coinbase = tag.is_coinbase().to_boolean();

        let fee_token = &payload.common.fee_token;
        let fee_token_default = field::equal(fee_token.0, TokenId::default().0, w);

        let token = &payload.body.token_id;
        let token_default = field::equal(token.0, TokenId::default().0, w);

        Boolean::assert_any(
            &[
                fee_token_default,
                is_payment,
                is_stake_delegation,
                is_fee_transfer,
            ],
            w,
        );

        Boolean::assert_any(
            &[
                is_payment,
                is_stake_delegation,
                is_fee_transfer,
                is_coinbase,
            ],
            w,
        );

        let current_global_slot = global_slot;
        let user_command_failure =
            user_command_failure::compute_as_prover(current_global_slot.clone(), tx, &ledger, w);

        let user_command_fails = Boolean::any(&user_command_failure.to_list(), w);
        let fee = payload.common.fee.to_checked();
        let receiver = AccountId::create(payload.body.receiver_pk.clone(), token.clone());
        let source = AccountId::create(payload.body.source_pk.clone(), token.clone());
        let nonce = payload.common.nonce.to_checked();
        let fee_payer = AccountId::create(payload.common.fee_payer_pk.clone(), fee_token.clone());

        fee_payer.checked_equal(&source, w);
        current_global_slot.lte(&payload.common.valid_until.to_checked(), w);

        let state_body_hash = state_body.checked_hash_with_param("MinaProtoStateBody", w);

        let pending_coinbase_init: pending_coinbase::Stack = pending_coinbase_init.into();

        let pending_coinbase_stack_with_state = pending_coinbase_init.checked_push_state(
            state_body_hash,
            current_global_slot.clone(),
            w,
        );

        let computed_pending_coinbase_stack_after = {
            let coinbase = Coinbase {
                receiver: receiver.public_key.clone(),
                amount: payload.body.amount,
                fee_transfer: None,
            };

            let stack_prime = pending_coinbase_stack_with_state.checked_push_coinbase(coinbase, w);

            w.exists(match is_coinbase {
                Boolean::True => stack_prime,
                Boolean::False => pending_coinbase_stack_with_state.clone(),
            })
        };

        let _correct_coinbase_target_stack =
            computed_pending_coinbase_stack_after.equal_var(&pending_coinbase_after, w);

        let _valid_init_state = {
            let equal_source = pending_coinbase_init.equal_var(&pending_coinbase_stack_before, w);

            let equal_source_with_state =
                pending_coinbase_stack_with_state.equal_var(&pending_coinbase_stack_before, w);

            equal_source.or(&equal_source_with_state, w)
        };

        Boolean::assert_any(&[is_user_command, user_command_fails.neg()], w);

        let _predicate_result = {
            let is_own_account = checked_equal_compressed_key(
                &payload.common.fee_payer_pk,
                &payload.body.source_pk,
                w,
            );
            let predicate_result = Boolean::False;

            is_own_account.const_or(&predicate_result)
        };

        let account_creation_amount =
            currency::Amount::of_fee(&CONSTRAINT_CONSTANTS.account_creation_fee).to_checked();
        let is_zero_fee = fee.equal(&CheckedFee::zero(), w);

        let is_coinbase_or_fee_transfer = is_user_command.neg();

        let can_create_fee_payer_account = {
            let fee_may_be_charged = token_default.or(&is_zero_fee, w);
            is_coinbase_or_fee_transfer.and(&fee_may_be_charged, w)
        };

        let mut burned_tokens = CheckedAmount::<Fp>::zero();
        let mut zero_fee = CheckedSigned::zero();
        let mut new_account_fees = zero_fee.clone();

        let root_after_fee_payer_update = {
            let index = ledger.find_index_exn(fee_payer.clone());
            w.exists(index.to_bits());

            let account = ledger.get_exn(&index);
            let path = ledger.path_exn(index.clone());

            let (account, path) = w.exists((account, path));
            checked_verify_merkle_path(&account, &path, w);

            // filter
            let is_empty_and_writeable = {
                let is_writable = can_create_fee_payer_account;
                let account_already_there = account.id().checked_equal(&fee_payer, w);
                let account_not_there = checked_equal_compressed_key_const_and(
                    &account.public_key,
                    &CompressedPubKey::empty(),
                    w,
                );
                let not_there_but_writeable = account_not_there.and(&is_writable, w);
                Boolean::assert_any(&[account_already_there, not_there_but_writeable], w);
                not_there_but_writeable
            };

            // f
            let next = {
                // Why OCaml doesn't push value here ?
                let next_nonce = match is_user_command {
                    Boolean::True => account.nonce.incr().to_checked::<Fp>(),
                    Boolean::False => account.nonce.to_checked(),
                };

                let account_nonce = account.nonce.to_checked();
                let nonce_matches = nonce.equal(&account_nonce, w);
                Boolean::assert_any(&[is_user_command.neg(), nonce_matches], w);

                let current = &account.receipt_chain_hash;
                let r = checked_cons_signed_command_payload(payload, current.clone(), w);
                let receipt_chain_hash = w.exists(match is_user_command {
                    Boolean::True => r,
                    Boolean::False => current.clone(),
                });

                let _permitted_to_access = account.checked_has_permission_to(
                    PermsConst {
                        and_const: false,
                        or_const: false,
                    },
                    Some(is_user_command),
                    PermissionTo::Access,
                    w,
                );
                let permitted_to_increment_nonce = account.checked_has_permission_to(
                    PermsConst {
                        and_const: true,
                        or_const: false,
                    },
                    None,
                    PermissionTo::IncrementNonce,
                    w,
                );
                let permitted_to_send = account.checked_has_permission_to(
                    PermsConst {
                        and_const: true,
                        or_const: false,
                    },
                    None,
                    PermissionTo::Send,
                    w,
                );
                let permitted_to_receive = account.checked_has_permission_to(
                    PermsConst {
                        and_const: true,
                        or_const: true,
                    },
                    None,
                    PermissionTo::Receive,
                    w,
                );

                Boolean::assert_any(&[is_user_command.neg(), permitted_to_increment_nonce], w);
                Boolean::assert_any(&[is_user_command.neg(), permitted_to_send], w);

                let update_account = {
                    let receiving_allowed =
                        Boolean::all(&[is_coinbase_or_fee_transfer, permitted_to_receive], w);
                    Boolean::any(&[is_user_command, receiving_allowed], w)
                };

                let is_empty_and_writeable =
                    Boolean::all(&[is_empty_and_writeable, is_zero_fee.neg()], w);

                let should_pay_to_create = is_empty_and_writeable;

                let amount = {
                    let fee_payer_amount = {
                        let sgn = match is_user_command {
                            Boolean::True => Sgn::Neg,
                            Boolean::False => Sgn::Pos,
                        };
                        CheckedSigned::create(CheckedAmount::of_fee(&fee), sgn)
                    };

                    let account_creation_fee = {
                        let magnitude = if should_pay_to_create.as_bool() {
                            account_creation_amount.clone()
                        } else {
                            CheckedAmount::zero()
                        };
                        CheckedSigned::create(magnitude, Sgn::Neg)
                    };

                    new_account_fees = account_creation_fee.clone();

                    w.exists(fee_payer_amount.value());
                    fee_payer_amount.add(&account_creation_fee, w)
                };

                {
                    let amt = add_burned_tokens::<Fp>(
                        burned_tokens,
                        CheckedAmount::of_fee(&fee),
                        is_coinbase_or_fee_transfer,
                        update_account,
                        true,
                        w,
                    );
                    burned_tokens = amt;
                }

                let txn_global_slot = current_global_slot.clone();
                let timing = {
                    let txn_amount = w.exists_no_check(match amount.sgn {
                        Sgn::Neg => amount.magnitude.clone(),
                        Sgn::Pos => CheckedAmount::zero(),
                    });

                    let timed_balance_check = |_ok: Boolean, _w: &mut Witness<Fp>| {};

                    let (_, timing) = check_timing(
                        &account,
                        Some(&txn_amount),
                        txn_global_slot,
                        timed_balance_check,
                        w,
                    );

                    w.exists_no_check(match update_account {
                        Boolean::True => timing,
                        Boolean::False => account.timing.clone(),
                    })
                };

                let balance = {
                    let account_balance = account.balance.to_checked();
                    let updated_balance = account_balance.add_signed_amount(amount, w);
                    w.exists_no_check(match update_account {
                        Boolean::True => updated_balance,
                        Boolean::False => account_balance,
                    })
                };
                let public_key = w.exists(match is_empty_and_writeable {
                    Boolean::True => fee_payer.public_key.clone(),
                    Boolean::False => account.public_key.clone(),
                });
                let token_id = w.exists(match is_empty_and_writeable {
                    Boolean::True => fee_payer.token_id.clone(),
                    Boolean::False => account.token_id.clone(),
                });
                let delegate = w.exists(match is_empty_and_writeable {
                    Boolean::True => fee_payer.public_key.clone(),
                    Boolean::False => account
                        .delegate
                        .clone()
                        .unwrap_or_else(CompressedPubKey::empty),
                });

                Box::new(Account {
                    public_key,
                    token_id,
                    token_symbol: account.token_symbol,
                    balance: balance.to_inner(),
                    nonce: next_nonce.to_inner(),
                    receipt_chain_hash,
                    delegate: if delegate == CompressedPubKey::empty() {
                        None
                    } else {
                        Some(delegate)
                    },
                    voting_for: account.voting_for,
                    timing,
                    permissions: account.permissions,
                    zkapp: account.zkapp,
                })
            };

            ledger.set_exn(index, next.clone());
            checked_verify_merkle_path(&next, &path, w)
        };

        let receiver_increase = {
            let base_amount = {
                let zero_transfer = is_stake_delegation;
                w.exists_no_check(match zero_transfer {
                    Boolean::True => CheckedAmount::zero(),
                    Boolean::False => payload.body.amount.to_checked(),
                })
            };

            let coinbase_receiver_fee = w.exists_no_check(match is_coinbase {
                Boolean::True => CheckedAmount::of_fee(&fee),
                Boolean::False => CheckedAmount::zero(),
            });

            base_amount.sub(&coinbase_receiver_fee, w)
        };

        let mut receiver_overflow = Boolean::False;
        let mut receiver_balance_update_permitted = Boolean::True;

        let root_after_receiver_update = {
            let index = ledger.find_index_exn(receiver.clone());
            w.exists(index.to_bits());

            let account = ledger.get_exn(&index);
            let path = ledger.path_exn(index.clone());

            let (account, path) = w.exists((account, path));
            checked_verify_merkle_path(&account, &path, w);

            // filter
            let is_empty_and_writeable = {
                let aid = &receiver;
                let account_already_there = account.id().checked_equal(aid, w);
                dbg!(account.public_key.clone(), &CompressedPubKey::empty());
                dbg!(account.public_key.x, &CompressedPubKey::empty().x);
                dbg!(account.public_key.is_odd, &CompressedPubKey::empty().is_odd);
                let account_not_there = checked_equal_compressed_key_const_and(
                    &account.public_key,
                    &CompressedPubKey::empty(),
                    w,
                );

                Boolean::assert_any(&[account_already_there, account_not_there], w);

                account_not_there
            };

            // f
            let next = {
                let permitted_to_access = account.checked_has_permission_to(
                    PermsConst {
                        and_const: true,
                        or_const: true,
                    },
                    Some(Boolean::False),
                    PermissionTo::Access,
                    w,
                );
                let permitted_to_receive = account
                    .checked_has_permission_to(
                        PermsConst {
                            and_const: true,
                            or_const: true,
                        },
                        None,
                        PermissionTo::Receive,
                        w,
                    )
                    .and(&permitted_to_access, w);

                let payment_or_internal_command =
                    Boolean::any(&[is_payment, is_coinbase_or_fee_transfer], w);

                let update_account = Boolean::any(
                    &[payment_or_internal_command.neg(), permitted_to_receive],
                    w,
                )
                .and(&permitted_to_access, w);

                receiver_balance_update_permitted = permitted_to_receive;

                let is_empty_failure = {
                    let must_not_be_empty = is_stake_delegation;
                    is_empty_and_writeable.and(&must_not_be_empty, w)
                };

                // is_empty_failure.equal(&Boolean::from_bool(user_command_failure.receiver_not_present), w);

                let is_empty_and_writeable =
                    Boolean::all(&[is_empty_and_writeable, is_empty_failure.neg()], w);

                let should_pay_to_create = is_empty_and_writeable;

                {
                    let token_should_not_create = should_pay_to_create.and(&token_default.neg(), w);

                    let _token_cannot_create = token_should_not_create.and(&is_user_command, w);
                }

                let balance = {
                    let receiver_amount = {
                        let account_creation_fee = match should_pay_to_create {
                            Boolean::True => account_creation_amount,
                            Boolean::False => CheckedAmount::zero(),
                        };

                        let new_account_fees_total =
                            CheckedSigned::of_unsigned(account_creation_fee.clone())
                                .negate()
                                .add(&new_account_fees, w);
                        new_account_fees = new_account_fees_total;

                        let (amount_for_new_account, underflow) =
                            receiver_increase.sub_flagged(&account_creation_fee, w);

                        w.exists_no_check(match user_command_fails {
                            Boolean::True => CheckedAmount::zero(),
                            Boolean::False => amount_for_new_account,
                        })
                    };

                    let account_balance = account.balance.to_checked();
                    let (balance, overflow) =
                        account_balance.add_amount_flagged(&receiver_amount, w);

                    Boolean::assert_any(&[is_user_command, overflow.neg()], w);

                    w.exists_no_check(match overflow {
                        Boolean::True => account_balance,
                        Boolean::False => balance,
                    })
                };

                {
                    let amt = add_burned_tokens::<Fp>(
                        burned_tokens,
                        receiver_increase,
                        is_coinbase_or_fee_transfer,
                        permitted_to_receive,
                        false,
                        w,
                    );
                    burned_tokens = amt;
                }

                let user_command_fails = receiver_overflow.or(&user_command_fails, w);

                let is_empty_and_writeable = Boolean::all(
                    &[
                        is_empty_and_writeable,
                        user_command_fails.neg(),
                        update_account,
                    ],
                    w,
                );

                let balance = w.exists_no_check(match update_account {
                    Boolean::True => balance,
                    Boolean::False => account.balance.to_checked(),
                });

                let may_delegate = is_empty_and_writeable.and(&token_default, w);

                let delegate = w.exists(match may_delegate {
                    Boolean::True => receiver.public_key.clone(),
                    Boolean::False => account
                        .delegate
                        .clone()
                        .unwrap_or_else(CompressedPubKey::empty),
                });

                let public_key = w.exists(match is_empty_and_writeable {
                    Boolean::True => receiver.public_key.clone(),
                    Boolean::False => account.public_key.clone(),
                });

                let token_id = w.exists(match is_empty_and_writeable {
                    Boolean::True => token.clone(),
                    Boolean::False => account.token_id.clone(),
                });

                Box::new(Account {
                    public_key,
                    token_id,
                    token_symbol: account.token_symbol,
                    balance: balance.to_inner(),
                    nonce: account.nonce,
                    receipt_chain_hash: account.receipt_chain_hash,
                    delegate: if delegate == CompressedPubKey::empty() {
                        None
                    } else {
                        Some(delegate)
                    },
                    voting_for: account.voting_for,
                    timing: account.timing,
                    permissions: account.permissions,
                    zkapp: account.zkapp,
                })
            };

            ledger.set_exn(index, next.clone());
            checked_verify_merkle_path(&next, &path, w)
        };

        let user_command_fails = receiver_overflow.or(&user_command_fails, w);
        let fee_payer_is_source = fee_payer.checked_equal(&source, w);

        let root_after_source_update = {
            let index = ledger.find_index_exn(source.clone());
            w.exists(index.to_bits());

            let account = ledger.get_exn(&index);
            let path = ledger.path_exn(index.clone());

            let (account, path) = w.exists((account, path));
            checked_verify_merkle_path(&account, &path, w);

            // filter
            let is_empty_and_writeable = {
                let is_writable = user_command_failure.source_not_present.to_boolean();
                let account_already_there = account.id().checked_equal(&source, w);
                let account_not_there = checked_equal_compressed_key_const_and(
                    &account.public_key,
                    &CompressedPubKey::empty(),
                    w,
                );
                let not_there_but_writeable = account_not_there.and(&is_writable, w);
                Boolean::assert_any(&[account_already_there, not_there_but_writeable], w);
                not_there_but_writeable
            };

            // f
            let next = {
                let bool_to_field = |b: bool| b.to_boolean().to_field::<Fp>();
                let num_failures = field::const_add(
                    bool_to_field(user_command_failure.source_insufficient_balance),
                    bool_to_field(user_command_failure.source_bad_timing),
                );
                let not_fee_payer_is_source = fee_payer_is_source.neg();

                let permitted_to_access = account.checked_has_permission_to(
                    PermsConst {
                        and_const: false,
                        or_const: false,
                    },
                    Some(is_user_command),
                    PermissionTo::Access,
                    w,
                );
                let permitted_to_update_delegate = account.checked_has_permission_to(
                    PermsConst {
                        and_const: true,
                        or_const: false,
                    },
                    None,
                    PermissionTo::SetDelegate,
                    w,
                );
                let permitted_to_send = account.checked_has_permission_to(
                    PermsConst {
                        and_const: true,
                        or_const: false,
                    },
                    None,
                    PermissionTo::Send,
                    w,
                );
                let permitted_to_receive = account.checked_has_permission_to(
                    PermsConst {
                        and_const: true,
                        or_const: true,
                    },
                    None,
                    PermissionTo::Receive,
                    w,
                );

                let payment_permitted = Boolean::all(
                    &[
                        is_payment,
                        permitted_to_access,
                        permitted_to_send,
                        receiver_balance_update_permitted,
                    ],
                    w,
                );

                let update_account = {
                    let delegation_permitted =
                        Boolean::all(&[is_stake_delegation, permitted_to_update_delegate], w);

                    let fee_receiver_update_permitted =
                        Boolean::all(&[is_coinbase_or_fee_transfer, permitted_to_receive], w);

                    Boolean::any(
                        &[
                            payment_permitted,
                            delegation_permitted,
                            fee_receiver_update_permitted,
                        ],
                        w,
                    )
                    .and(&permitted_to_access, w)
                };

                let amount = w.exists_no_check(match payment_permitted {
                    Boolean::True => payload.body.amount.to_checked(),
                    Boolean::False => CheckedAmount::zero(),
                });

                let txn_global_slot = current_global_slot;

                let timing = {
                    let timed_balance_check = |ok: Boolean, w: &mut Witness<Fp>| {
                        let _not_ok = ok.neg().and(
                            &user_command_failure
                                .source_insufficient_balance
                                .to_boolean(),
                            w,
                        );
                    };

                    let (_, timing) = check_timing(
                        &account,
                        Some(&amount),
                        txn_global_slot,
                        timed_balance_check,
                        w,
                    );

                    w.exists_no_check(match update_account {
                        Boolean::True => timing,
                        Boolean::False => account.timing.clone(),
                    })
                };

                let (balance, _underflow) =
                    account.balance.to_checked().sub_amount_flagged(&amount, w);

                let delegate = {
                    let may_delegate = Boolean::all(&[is_stake_delegation, update_account], w);

                    w.exists(match may_delegate {
                        Boolean::True => receiver.public_key,
                        Boolean::False => account
                            .delegate
                            .clone()
                            .unwrap_or_else(CompressedPubKey::empty),
                    })
                };

                Box::new(Account {
                    public_key: account.public_key,
                    token_id: account.token_id,
                    token_symbol: account.token_symbol,
                    balance: balance.to_inner(),
                    nonce: account.nonce,
                    receipt_chain_hash: account.receipt_chain_hash,
                    delegate: if delegate == CompressedPubKey::empty() {
                        None
                    } else {
                        Some(delegate)
                    },
                    voting_for: account.voting_for,
                    timing,
                    permissions: account.permissions,
                    zkapp: account.zkapp,
                })
            };

            ledger.set_exn(index, next.clone());
            checked_verify_merkle_path(&next, &path, w)
        };

        let fee_excess = {
            let then_value = CheckedSigned::of_unsigned(CheckedAmount::zero());

            let else_value = {
                let amount_fee = CheckedAmount::of_fee(&payload.common.fee.to_checked());

                let user_command_excess = CheckedSigned::of_unsigned(amount_fee.clone());

                let (fee_transfer_excess, fee_transfer_excess_overflowed) = {
                    let (magnitude, overflow) =
                        payload.body.amount.to_checked().add_flagged(&amount_fee, w);
                    (CheckedSigned::create(magnitude, Sgn::Neg), overflow)
                };

                Boolean::assert_any(
                    &[is_fee_transfer.neg(), fee_transfer_excess_overflowed.neg()],
                    w,
                );

                let value = match is_fee_transfer {
                    Boolean::True => fee_transfer_excess,
                    Boolean::False => user_command_excess,
                };
                w.exists_no_check(value.magnitude.clone());
                value
            };

            w.exists_no_check(match is_coinbase {
                Boolean::True => then_value,
                Boolean::False => else_value,
            })
        };

        let supply_increase = {
            dbg!(payload.body.amount);
            let expected_supply_increase = match is_coinbase {
                Boolean::True => CheckedSigned::of_unsigned(payload.body.amount.to_checked()),
                Boolean::False => CheckedSigned::of_unsigned(CheckedAmount::zero()),
            };
            w.exists_no_check(expected_supply_increase.magnitude.clone());
            w.exists_no_check(expected_supply_increase.magnitude.clone());

            let (amt0, overflow0) = expected_supply_increase
                .add_flagged(&CheckedSigned::of_unsigned(burned_tokens).negate(), w);

            let new_account_fees_total = w.exists_no_check(match user_command_fails {
                Boolean::True => zero_fee,
                Boolean::False => new_account_fees,
            });

            w.exists(new_account_fees_total.value()); // Made in the `add_flagged` call
            let (amt, overflow) = amt0.add_flagged(&new_account_fees_total, w);

            amt
        };

        let final_root = w.exists_no_check(match user_command_fails {
            Boolean::True => root_after_fee_payer_update,
            Boolean::False => root_after_source_update,
        });

        (final_root, fee_excess, supply_increase)
    }

    pub fn assert_equal_local_state<F: FieldWitness>(
        t1: &LocalState,
        t2: &LocalState,
        w: &mut Witness<F>,
    ) {
        w.exists_no_check(t1.excess.to_checked::<Fp>().value());
        w.exists_no_check(t2.excess.to_checked::<Fp>().value());

        w.exists_no_check(t1.supply_increase.to_checked::<Fp>().value());
        w.exists_no_check(t2.supply_increase.to_checked::<Fp>().value());
    }

    pub fn main(
        statement_with_sok: &Statement<SokDigest>,
        tx_witness: &v2::TransactionWitnessStableV2,
        w: &mut Witness<Fp>,
    ) {
        let tx: crate::scan_state::transaction_logic::Transaction =
            (&tx_witness.transaction).into();
        let tx = transaction_union_payload::TransactionUnion::of_transaction(&tx);

        dummy_constraints(w);
        let shifted = create_shifted_inner_curve(w);

        let tx = w.exists(tx);
        let pending_coinbase_init = w.exists(tx_witness.init_stack.clone());
        let state_body = w.exists(tx_witness.protocol_state_body.clone());
        let global_slot = w.exists(tx_witness.block_global_slot.clone());

        let sparse_ledger: SparseLedger = (&tx_witness.first_pass_ledger).into();

        let (_fee_payment_root_after, fee_excess, _supply_increase) = apply_tagged_transaction(
            &shifted,
            statement_with_sok.source.first_pass_ledger,
            currency::Slot::from_u32(global_slot.as_u32()),
            &pending_coinbase_init,
            &statement_with_sok.source.pending_coinbase_stack,
            &statement_with_sok.target.pending_coinbase_stack,
            &state_body,
            &tx,
            &sparse_ledger,
            w,
        );

        let _fee_excess = {
            let fee_excess_zero = {
                let fee_excess = w.exists(fee_excess.value());
                field::equal(fee_excess, Fp::zero(), w)
            };

            let fee_token_l = w.exists_no_check(match fee_excess_zero {
                Boolean::True => TokenId::default(),
                Boolean::False => tx.payload.common.fee_token.clone(),
            });

            CheckedFeeExcess {
                fee_token_l,
                fee_excess_l: fee_excess.to_fee(),
                fee_token_r: TokenId::default(),
                fee_excess_r: CheckedSigned::zero(),
            }
        };

        assert_equal_local_state(
            &statement_with_sok.source.local_state,
            &statement_with_sok.target.local_state,
            w,
        );

        // Checked.all_unit
        {
            let supply_increase = statement_with_sok.supply_increase;
            w.exists_no_check(supply_increase.to_checked::<Fp>().value());

            let FeeExcess {
                fee_token_l: _,
                fee_excess_l,
                fee_token_r: _,
                fee_excess_r,
            } = statement_with_sok.fee_excess;

            w.exists_no_check(fee_excess_l.to_checked::<Fp>().value());
            w.exists_no_check(fee_excess_r.to_checked::<Fp>().value());
        }
    }
}

fn get_messages_for_next_wrap_proof_padded() -> Vec<Fp> {
    let msg = MessagesForNextWrapProof {
        challenge_polynomial_commitment: InnerCurve::from(dummy_ipa_step_sg()),
        old_bulletproof_challenges: vec![], // Filled with padding
    };

    let hash = msg.hash();
    let hash = Fp::from(BigInteger256(hash));

    vec![hash, hash]
}

pub fn checked_hash2<F: FieldWitness>(inputs: &[F], w: &mut Witness<F>) -> F {
    let mut sponge = poseidon::Sponge::<F>::new();
    sponge.absorb2(inputs, w);
    sponge.squeeze(w)
}

pub fn checked_hash3<F: FieldWitness>(inputs: &[F], w: &mut Witness<F>) -> F {
    let mut sponge = poseidon::Sponge::<F>::new();
    sponge.absorb(inputs, w);
    sponge.squeeze(w)
}

pub struct StepMainProofState {
    pub unfinalized_proofs: Vec<Unfinalized>,
    pub messages_for_next_step_proof: Fp,
}

pub struct StepMainStatement {
    pub proof_state: StepMainProofState,
    pub messages_for_next_wrap_proof: Vec<Fp>,
}

impl ToFieldElements<Fp> for StepMainStatement {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self {
            proof_state:
                StepMainProofState {
                    unfinalized_proofs,
                    messages_for_next_step_proof,
                },
            messages_for_next_wrap_proof,
        } = self;

        unfinalized_proofs.to_field_elements(fields);
        messages_for_next_step_proof.to_field_elements(fields);
        messages_for_next_wrap_proof.to_field_elements(fields);
    }
}

fn step_main(
    statement_with_sok: &Statement<SokDigest>,
    tx_witness: &v2::TransactionWitnessStableV2,
    dlog_plonk_index: &PlonkVerificationKeyEvals<Fp>,
    w: &mut Witness<Fp>,
) -> StepMainStatement {
    let statement_with_sok = w.exists(statement_with_sok);

    transaction_snark::main(statement_with_sok, tx_witness, w);

    let dlog_plonk_index = w.exists(dlog_plonk_index);

    let messages_for_next_wrap_proof = w.exists(get_messages_for_next_wrap_proof_padded());

    let mut inputs = dlog_plonk_index.to_field_elements_owned();
    statement_with_sok.to_field_elements(&mut inputs);

    let messages_for_next_step_proof = checked_hash2(&inputs, w);

    StepMainStatement {
        proof_state: StepMainProofState {
            unfinalized_proofs: vec![Unfinalized::dummy(); 2],
            messages_for_next_step_proof,
        },
        messages_for_next_wrap_proof,
    }
}

#[derive(Clone, Debug)]
pub struct StepProofState {
    pub unfinalized_proofs: Vec<Unfinalized>,
    pub messages_for_next_step_proof: ReducedMessagesForNextStepProof<Statement<SokDigest>>,
}

#[derive(Debug)]
pub struct StepStatement {
    pub proof_state: StepProofState,
    pub messages_for_next_wrap_proof: Vec<MessagesForNextWrapProof>,
}

#[derive(Debug)]
pub struct StepStatementWithHash {
    pub proof_state: StepProofState,
    pub messages_for_next_wrap_proof: Vec<[u64; 4]>,
}

fn step(
    statement_with_sok: &Statement<SokDigest>,
    tx_witness: &v2::TransactionWitnessStableV2,
    dlog_plonk_index: &PlonkVerificationKeyEvals<Fp>,
    w: &mut Witness<Fp>,
) -> StepStatement {
    let statement = step_main(statement_with_sok, tx_witness, dlog_plonk_index, w);
    w.primary = statement.to_field_elements_owned();

    dbg!(&w.primary);

    let msg = ReducedMessagesForNextStepProof {
        app_state: statement_with_sok.clone(),
        challenge_polynomial_commitments: vec![],
        old_bulletproof_challenges: vec![],
    };

    // let msg = MessagesForNextStepProof {
    //     app_state: &statement_with_sok,
    //     challenge_polynomial_commitments: vec![],
    //     old_bulletproof_challenges: vec![],
    //     dlog_plonk_index,
    // };

    // let hash: [u64; 4] = msg.hash();
    // eprintln!("hash[0]={:?}", hash[0] as i64);
    // eprintln!("hash[1]={:?}", hash[1] as i64);
    // eprintln!("hash[2]={:?}", hash[2] as i64);
    // eprintln!("hash[3]={:?}", hash[3] as i64);

    // assert_eq!(
    //     hash,
    //     [
    //         -7356330309193778536i64 as u64,
    //         9069183817894203571,
    //         -4599336761250751607i64 as u64,
    //         117782671464327204
    //     ]
    // );

    StepStatement {
        proof_state: StepProofState {
            unfinalized_proofs: statement.proof_state.unfinalized_proofs,
            messages_for_next_step_proof: msg,
        },
        messages_for_next_wrap_proof: vec![],
    }
}

#[derive(Clone, Debug)]
pub struct ReducedMessagesForNextStepProof<AppState: ToFieldElements<Fp>> {
    pub app_state: AppState,
    pub challenge_polynomial_commitments: Vec<InnerCurve<Fp>>,
    pub old_bulletproof_challenges: Vec<[Fp; 16]>,
}

#[derive(Clone, Debug)]
pub struct MessagesForNextStepProof<'a, AppState: ToFieldElements<Fp>> {
    pub app_state: &'a AppState,
    pub dlog_plonk_index: &'a PlonkVerificationKeyEvals<Fp>,
    pub challenge_polynomial_commitments: Vec<InnerCurve<Fp>>,
    pub old_bulletproof_challenges: Vec<[Fp; 16]>,
}

impl<AppState> MessagesForNextStepProof<'_, AppState>
where
    AppState: ToFieldElements<Fp>,
{
    /// Implementation of `hash_messages_for_next_step_proof`
    /// https://github.com/MinaProtocol/mina/blob/32a91613c388a71f875581ad72276e762242f802/src/lib/pickles/common.ml#L33
    pub fn hash(&self) -> [u64; 4] {
        let fields: Vec<Fp> = self.to_fields();
        let field: Fp = crate::hash_fields(&fields);

        let bigint: BigInteger256 = field.into_repr();
        bigint.0
    }

    /// Implementation of `to_field_elements`
    /// https://github.com/MinaProtocol/mina/blob/32a91613c388a71f875581ad72276e762242f802/src/lib/pickles/composition_types/composition_types.ml#L493
    pub fn to_fields(&self) -> Vec<Fp> {
        const NFIELDS: usize = 93; // TODO: This is bigger with transactions

        let mut fields = Vec::with_capacity(NFIELDS);

        let push_curve = |fields: &mut Vec<Fp>, curve: &InnerCurve<Fp>| {
            let GroupAffine::<Fp> { x, y, .. } = curve.to_affine();
            fields.push(x);
            fields.push(y);
        };

        // Self::dlog_plonk_index
        // Refactor with `src/account/account.rs`, this is the same code
        {
            let index = &self.dlog_plonk_index;

            for curve in &index.sigma {
                push_curve(&mut fields, curve);
            }
            for curve in &index.coefficients {
                push_curve(&mut fields, curve);
            }
            push_curve(&mut fields, &index.generic);
            push_curve(&mut fields, &index.psm);
            push_curve(&mut fields, &index.complete_add);
            push_curve(&mut fields, &index.mul);
            push_curve(&mut fields, &index.emul);
            push_curve(&mut fields, &index.endomul_scalar);
        }

        self.app_state.to_field_elements(&mut fields);

        let commitments = &self.challenge_polynomial_commitments;
        let old_challenges = &self.old_bulletproof_challenges;
        for (commitments, old) in commitments.iter().zip(old_challenges) {
            push_curve(&mut fields, commitments);
            fields.extend_from_slice(old);
        }

        fields
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum V {
    External(usize),
    Internal(usize),
}

pub type InternalVars<F> = HashMap<usize, (Vec<(F, V)>, Option<F>)>;

pub fn compute_witness<C: ProofConstants, F: FieldWitness>(
    prover: &Prover<F>,
    w: &Witness<F>,
) -> [Vec<F>; COLUMNS] {
    let external_values = |i: usize| {
        if i < C::PRIMARY_LEN {
            w.primary[i]
        } else {
            w.aux[i - C::PRIMARY_LEN]
        }
    };

    let mut internal_values = HashMap::<usize, F>::with_capacity(13_000);

    let public_input_size = C::PRIMARY_LEN;
    let num_rows = C::ROWS;

    let mut res: [_; COLUMNS] = std::array::from_fn(|_| vec![F::zero(); num_rows]);

    // public input
    for i in 0..public_input_size {
        res[0][i] = external_values(i);
    }

    let compute = |(lc, c): &(Vec<(F, V)>, Option<F>), internal_values: &HashMap<_, _>| {
        lc.iter().fold(c.unwrap_or_else(F::zero), |acc, (s, x)| {
            let x = match x {
                V::External(x) => external_values(*x),
                V::Internal(x) => internal_values.get(x).copied().unwrap(),
            };
            acc + (*s * x)
        })
    };

    for (i_after_input, cols) in prover.rows_rev.iter().rev().enumerate() {
        let row_idx = i_after_input + public_input_size;
        for (col_idx, var) in cols.iter().enumerate() {
            // println!("w[{}][{}]", col_idx, row_idx);
            match var {
                None => (),
                Some(V::External(var)) => {
                    res[col_idx][row_idx] = external_values(*var);
                }
                Some(V::Internal(var)) => {
                    let lc = prover.internal_vars.get(var).unwrap();
                    let value = compute(lc, &internal_values);
                    res[col_idx][row_idx] = value;
                    internal_values.insert(*var, value);
                }
            }
        }
    }

    dbg!(internal_values.len());

    res
}

fn make_prover_index<C: ProofConstants, F: FieldWitness>(
    gates: Vec<CircuitGate<F>>,
) -> ProverIndex<F::OtherCurve> {
    use kimchi::circuits::constraints::ConstraintSystem;

    let public = C::PRIMARY_LEN;
    let prev_challenges = C::PREVIOUS_CHALLENGES;

    let cs = ConstraintSystem::<F>::create(gates)
        .public(public as usize)
        .prev_challenges(prev_challenges as usize)
        .build()
        .unwrap();

    let (endo_q, _endo_r) = endos::<F>();

    // TODO: `proof-systems` needs to change how the SRS is used
    let srs: poly_commitment::srs::SRS<F::OtherCurve> = {
        let srs = get_srs::<F>();
        let mut srs = srs.lock().unwrap();
        srs.add_lagrange_basis(cs.domain.d1);
        srs.clone()
    };

    let mut index = ProverIndex::<F::OtherCurve>::create(cs, endo_q, Arc::new(srs));

    // Compute and cache the verifier index digest
    index.compute_verifier_index_digest::<F::FqSponge>();
    index
}

pub fn create_proof<F: FieldWitness>(
    computed_witness: [Vec<F>; COLUMNS],
    prover_index: &ProverIndex<F::OtherCurve>,
    prev_challenges: Vec<RecursionChallenge<F::OtherCurve>>,
) -> kimchi::proof::ProverProof<F::OtherCurve> {
    type EFrSponge<F> = mina_poseidon::sponge::DefaultFrSponge<F, PlonkSpongeConstantsKimchi>;

    let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(0);
    let now = std::time::Instant::now();
    let group_map = kimchi::groupmap::GroupMap::<F::Scalar>::setup();
    let proof = kimchi::proof::ProverProof::create_recursive::<F::FqSponge, EFrSponge<F>>(
        &group_map,
        computed_witness,
        &[],
        prover_index,
        prev_challenges,
        None,
        &mut rng,
    )
    .unwrap();

    eprintln!("proof_elapsed={:?}", now.elapsed());

    proof
}

pub struct Prover<F: FieldWitness> {
    /// Constants to each kind of proof
    pub internal_vars: InternalVars<F>,
    /// Constants to each kind of proof
    pub rows_rev: Vec<Vec<Option<V>>>,
    pub index: ProverIndex<F::OtherCurve>,
}

fn generate_proof(
    statement: &MinaStateBlockchainStateValueStableV2LedgerProofStatement,
    tx_witness: &v2::TransactionWitnessStableV2,
    message: &SokMessage,
    step_prover: &Prover<Fp>,
    wrap_prover: &Prover<Fq>,
    w: &mut Witness<Fp>,
) -> String {
    let statement: Statement<()> = statement.into();
    let sok_digest = message.digest();
    let statement_with_sok = statement.with_digest(sok_digest);

    let dlog_plonk_index =
        { PlonkVerificationKeyEvals::from(wrap_prover.index.verifier_index.as_ref().unwrap()) };

    let now = std::time::Instant::now();
    let step_statement = step(&statement_with_sok, tx_witness, &dlog_plonk_index, w);

    // TODO: Not always dummy
    let prev_evals = vec![AllEvals::dummy(); 2];

    dbg!(w.primary.len());
    dbg!(w.aux.len());
    dbg!(w.ocaml_aux.len());
    // assert_eq!(w.aux.len(), w.ocaml_aux.len());
    // assert_eq!(&w.aux, &w.ocaml_aux);

    eprintln!("witness0_elapsed={:?}", now.elapsed());
    let computed_witness = compute_witness::<RegularTransactionProof, _>(&step_prover, w);
    eprintln!("witness_elapsed={:?}", now.elapsed());

    // let prover_index = make_prover_index(gates);
    let prev_challenges = vec![];
    let proof = create_proof::<Fp>(computed_witness, &step_prover.index, prev_challenges);

    // dbg!(&proof);

    let mut w = Witness::new::<WrapProof>();

    fn read_witnesses_fq() -> std::io::Result<Vec<Fq>> {
        let f = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("/tmp/fqs.txt"),
            // std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fqs_rampup4.txt"),
        )?;

        let fqs = f
            .lines()
            .filter(|s| !s.is_empty())
            .map(|s| Fq::from_str(s).unwrap())
            .collect::<Vec<_>>();

        Ok(fqs)
    }

    // w.ocaml_aux = read_witnesses_fq().unwrap();

    // let wrap_index = make_prover_index_wrap(wrap_gates);

    const WHICH_INDEX: u64 = 0;
    let message = crate::proofs::wrap::wrap(
        &statement_with_sok,
        &proof,
        step_statement,
        &prev_evals,
        &dlog_plonk_index,
        &step_prover.index,
        WHICH_INDEX,
        &mut w,
    );

    let computed_witness = compute_witness::<WrapProof, _>(wrap_prover, &w);

    let prev = message
        .iter()
        .map(|m| RecursionChallenge {
            comm: poly_commitment::PolyComm::<Pallas> {
                unshifted: vec![m.commitment.to_affine()],
                shifted: None,
            },
            chals: m.challenges.to_vec(),
        })
        .collect();

    dbg!(&prev);
    dbg!(&w.primary);
    dbg!(w.primary.len());

    let proof = create_proof::<Fq>(computed_witness, &wrap_prover.index, prev);

    let sum = |s: &[u8]| {
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        hasher.update(s);
        hex::encode(hasher.finalize())
    };

    let proof_json = serde_json::to_vec(&proof).unwrap();
    std::fs::write("/tmp/PROOF_RUST_WRAP.json", &proof_json).unwrap();

    dbg!(w.aux.len(), w.ocaml_aux.len());

    sum(&proof_json)
}

#[cfg(test)]
mod tests_with_wasm {
    use std::str::FromStr;

    use mina_hasher::Fp;

    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use super::*;
    #[test]
    fn test_to_field_checked() {
        let mut witness = Witness::empty();
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

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::Path, str::FromStr};

    use kimchi::circuits::gate::CircuitGate;
    use mina_hasher::Fp;
    use mina_p2p_messages::binprot::{
        self,
        macros::{BinProtRead, BinProtWrite},
    };

    use crate::{
        proofs::{
            block::generate_block_proof,
            constants::{BlockProof, MergeProof},
            merge::generate_merge_proof,
        },
        scan_state::scan_state::transaction_snark::SokMessage,
    };

    use super::*;

    type SnarkWorkSpec =
        mina_p2p_messages::v2::SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Instances;

    /// External worker input.
    #[derive(Debug, BinProtRead, BinProtWrite)]
    pub enum ExternalSnarkWorkerRequest {
        /// Queries worker for readiness, expected reply is `true`.
        AwaitReadiness,
        /// Commands worker to start specified snark job, expected reply is `ExternalSnarkWorkerResult`[ExternalSnarkWorkerResult].
        PerformJob(mina_p2p_messages::v2::SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponse),
    }

    fn read_binprot<T, R>(mut r: R) -> T
    where
        T: binprot::BinProtRead,
        R: std::io::Read,
    {
        use std::io::Read;

        let mut len_buf = [0; std::mem::size_of::<u64>()];
        r.read_exact(&mut len_buf).unwrap();
        let len = u64::from_le_bytes(len_buf);

        let mut buf = Vec::with_capacity(len as usize);
        let mut r = r.take(len);
        r.read_to_end(&mut buf).unwrap();

        let mut read = buf.as_slice();
        T::binprot_read(&mut read).unwrap()
    }

    fn read_witnesses() -> std::io::Result<Vec<Fp>> {
        let f = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("/tmp/fps_rampup4.txt"),
        )?;
        // let f = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("fps.txt"))?;

        let fps = f
            .lines()
            .filter(|s| !s.is_empty())
            .map(|s| Fp::from_str(s).unwrap())
            .collect::<Vec<_>>();

        // TODO: Implement [0..652]
        // Ok(fps.split_off(652))
        Ok(fps)
    }

    fn read_constraints_data<F: FieldWitness>(
        internal_vars_path: &Path,
        rows_rev_path: &Path,
    ) -> Option<(InternalVars<F>, Vec<Vec<Option<V>>>)> {
        use mina_p2p_messages::bigint::BigInt;

        impl From<&VRaw> for V {
            fn from(value: &VRaw) -> Self {
                match value {
                    VRaw::External(index) => Self::External(*index as usize),
                    VRaw::Internal(id) => Self::Internal(*id as usize),
                }
            }
        }

        #[derive(Clone, Debug, PartialEq, BinProtRead, BinProtWrite)]
        enum VRaw {
            External(u32),
            Internal(u32),
        }

        use binprot::BinProtRead;

        type InternalVarsRaw = HashMap<u32, (Vec<(BigInt, VRaw)>, Option<BigInt>)>;

        // ((Fp.t * V.t) list * Fp.t option)
        let Ok(internal_vars) = std::fs::read(internal_vars_path)
        // std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("internal_vars.bin"))
        else {
            return None;
        };
        let internal_vars: InternalVarsRaw =
            HashMap::binprot_read(&mut internal_vars.as_slice()).unwrap();

        // V.t option array list
        let rows_rev = std::fs::read(rows_rev_path).unwrap();
        // std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("rows_rev.bin")).unwrap();
        let rows_rev: Vec<Vec<Option<VRaw>>> = Vec::binprot_read(&mut rows_rev.as_slice()).unwrap();

        let internal_vars: InternalVars<F> = internal_vars
            .into_iter()
            .map(|(id, (list, opt))| {
                let id = id as usize;
                let list: Vec<_> = list
                    .iter()
                    .map(|(n, v)| (n.to_field::<F>(), V::from(v)))
                    .collect();
                let opt = opt.as_ref().map(BigInt::to_field::<F>);
                (id, (list, opt))
            })
            .collect();

        let rows_rev: Vec<_> = rows_rev
            .iter()
            .map(|row| {
                let row: Vec<_> = row.iter().map(|v| v.as_ref().map(V::from)).collect();
                row
            })
            .collect();

        Some((internal_vars, rows_rev))
    }

    #[allow(const_item_mutation)]
    #[test]
    fn test_read_constraints() {
        let internal_vars_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("internal_vars_rampup4.bin");
        let rows_rev_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("rows_rev_rampup4.bin");
        read_constraints_data::<Fp>(&internal_vars_path, &rows_rev_path);
    }

    fn extract_request(
        mut bytes: &[u8],
    ) -> (
        v2::MinaStateSnarkedLedgerStateStableV2,
        v2::TransactionWitnessStableV2,
        SokMessage,
    ) {
        use mina_p2p_messages::v2::*;

        let v: ExternalSnarkWorkerRequest = read_binprot(&mut bytes);

        let ExternalSnarkWorkerRequest::PerformJob(job) = v else {
            panic!()
        };
        let SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponse(Some((a, prover))) = job else {
            panic!()
        };
        let SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Instances::One(single) = a.instances
        else {
            panic!()
        };
        let SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Single::Transition(
            statement,
            tx_witness,
        ) = single
        else {
            panic!()
        };

        let prover: CompressedPubKey = (&prover).into();
        let fee = crate::scan_state::currency::Fee::from_u64(a.fee.as_u64());

        let message = SokMessage { fee, prover };

        (statement, tx_witness, message)
    }

    fn extract_merge(
        mut bytes: &[u8],
    ) -> (
        v2::MinaStateSnarkedLedgerStateStableV2,
        [v2::LedgerProofProdStableV2; 2],
        SokMessage,
    ) {
        use mina_p2p_messages::v2::*;

        let v: ExternalSnarkWorkerRequest = read_binprot(&mut bytes);

        let ExternalSnarkWorkerRequest::PerformJob(job) = v else {
            panic!()
        };
        let SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponse(Some((a, prover))) = job else {
            panic!()
        };
        let SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Instances::One(single) = a.instances
        else {
            panic!()
        };
        let SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Single::Merge(merge) = single else {
            panic!()
        };

        let (statement, p1, p2) = *merge;

        let prover: CompressedPubKey = (&prover).into();
        let fee = crate::scan_state::currency::Fee::from_u64(a.fee.as_u64());

        let message = SokMessage { fee, prover };

        (statement, [p1, p2], message)
    }

    struct Gates {
        gates: Vec<CircuitGate<Fp>>,
        wrap_gates: Vec<CircuitGate<Fq>>,
        merge_gates: Vec<CircuitGate<Fp>>,
        block_gates: Vec<CircuitGate<Fp>>,
        internal_vars: HashMap<usize, (Vec<(Fp, V)>, Option<Fp>)>,
        rows_rev: Vec<Vec<Option<V>>>,
        internal_vars_wrap: HashMap<usize, (Vec<(Fq, V)>, Option<Fq>)>,
        rows_rev_wrap: Vec<Vec<Option<V>>>,
        merge_internal_vars: HashMap<usize, (Vec<(Fp, V)>, Option<Fp>)>,
        merge_rows_rev: Vec<Vec<Option<V>>>,
        block_internal_vars: HashMap<usize, (Vec<(Fp, V)>, Option<Fp>)>,
        block_rows_rev: Vec<Vec<Option<V>>>,
    }

    fn read_gates() -> Gates {
        let base_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

        let internal_vars_path = base_dir.join("internal_vars_rampup4.bin");
        let rows_rev_path = base_dir.join("rows_rev_rampup4.bin");
        let (internal_vars, rows_rev) =
            read_constraints_data::<Fp>(&internal_vars_path, &rows_rev_path).unwrap();

        let internal_vars_path = base_dir.join("internal_vars_wrap_rampup4.bin");
        let rows_rev_path = base_dir.join("rows_rev_wrap_rampup4.bin");
        let (internal_vars_wrap, rows_rev_wrap) =
            read_constraints_data::<Fq>(&internal_vars_path, &rows_rev_path).unwrap();

        let internal_vars_path = base_dir.join("rampup4").join("merge_internal_vars.bin");
        let rows_rev_path = base_dir.join("rampup4").join("merge_rows_rev.bin");
        let (merge_internal_vars, merge_rows_rev) =
            read_constraints_data::<Fp>(&internal_vars_path, &rows_rev_path).unwrap();

        let internal_vars_path = base_dir.join("rampup4").join("block_internal_vars.bin");
        let rows_rev_path = base_dir.join("rampup4").join("block_rows_rev.bin");
        let (block_internal_vars, block_rows_rev) =
            read_constraints_data::<Fp>(&internal_vars_path, &rows_rev_path).unwrap();

        let gates: Vec<CircuitGate<Fp>> = {
            let gates_path = base_dir.join("gates_step_rampup4.json");
            let file = std::fs::File::open(gates_path).unwrap();
            let reader = std::io::BufReader::new(file);
            serde_json::from_reader(reader).unwrap()
        };

        let wrap_gates: Vec<CircuitGate<Fq>> = {
            let gates_path = base_dir.join("gates_wrap_rampup4.json");
            let file = std::fs::File::open(gates_path).unwrap();
            let reader = std::io::BufReader::new(file);
            serde_json::from_reader(reader).unwrap()
        };

        let merge_gates: Vec<CircuitGate<Fp>> = {
            let gates_path = base_dir.join("gates_merge_rampup4.json");
            let file = std::fs::File::open(gates_path).unwrap();
            let reader = std::io::BufReader::new(file);
            serde_json::from_reader(reader).unwrap()
        };

        let block_gates: Vec<CircuitGate<Fp>> = {
            let gates_path = base_dir.join("rampup4").join("block_gates.json");
            let file = std::fs::File::open(gates_path).unwrap();
            let reader = std::io::BufReader::new(file);
            serde_json::from_reader(reader).unwrap()
        };

        Gates {
            gates,
            wrap_gates,
            merge_gates,
            block_gates,
            internal_vars,
            rows_rev,
            internal_vars_wrap,
            rows_rev_wrap,
            merge_internal_vars,
            merge_rows_rev,
            block_internal_vars,
            block_rows_rev,
        }
    }

    struct Provers {
        tx_prover: Prover<Fp>,
        wrap_prover: Prover<Fq>,
        merge_prover: Prover<Fp>,
        block_prover: Prover<Fp>,
    }

    fn make_provers() -> Provers {
        let Gates {
            gates,
            wrap_gates,
            merge_gates,
            block_gates,
            internal_vars,
            rows_rev,
            internal_vars_wrap,
            rows_rev_wrap,
            merge_internal_vars,
            merge_rows_rev,
            block_internal_vars,
            block_rows_rev,
        } = read_gates();
        let tx_prover_index = make_prover_index::<RegularTransactionProof, _>(gates);
        let merge_prover_index = make_prover_index::<MergeProof, _>(merge_gates);
        let wrap_prover_index = make_prover_index::<WrapProof, _>(wrap_gates);
        let block_prover_index = make_prover_index::<BlockProof, _>(block_gates);

        let tx_prover = Prover {
            internal_vars,
            rows_rev,
            index: tx_prover_index,
        };

        let merge_prover = Prover {
            internal_vars: merge_internal_vars,
            rows_rev: merge_rows_rev,
            index: merge_prover_index,
        };

        let wrap_prover = Prover {
            internal_vars: internal_vars_wrap,
            rows_rev: rows_rev_wrap,
            index: wrap_prover_index,
        };

        let block_prover = Prover {
            internal_vars: block_internal_vars,
            rows_rev: block_rows_rev,
            index: block_prover_index,
        };

        Provers {
            tx_prover,
            wrap_prover,
            merge_prover,
            block_prover,
        }
    }

    #[test]
    fn test_protocol_state_body() {
        let Ok(data) =
            // std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("request_signed.bin"))
            std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("rampup4").join("request_payment_1_rampup4.bin"))
            // std::fs::read("/tmp/fee_transfer_1_rampup4.bin")
            // std::fs::read("/tmp/coinbase_1_rampup4.bin")
            // std::fs::read("/tmp/stake_0_rampup4.bin")
        else {
            eprintln!("request not found");
            return;
        };

        let (statement, tx_witness, message) = extract_request(&data);
        let Provers {
            tx_prover,
            wrap_prover,
            merge_prover: _,
            block_prover: _,
        } = make_provers();

        let mut witnesses: Witness<Fp> = Witness::new::<RegularTransactionProof>();
        generate_proof(
            &statement,
            &tx_witness,
            &message,
            &tx_prover,
            &wrap_prover,
            &mut witnesses,
        );
    }

    #[test]
    fn test_merge_proof() {
        let Ok(data) =
            // std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("request_signed.bin"))
            std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("rampup4").join("merge_0_rampup4.bin"))
            // std::fs::read("/tmp/fee_transfer_1_rampup4.bin")
            // std::fs::read("/tmp/coinbase_1_rampup4.bin")
            // std::fs::read("/tmp/stake_0_rampup4.bin")
        else {
            eprintln!("request not found");
            return;
        };

        let (statement, proofs, message) = extract_merge(&data);
        let Provers {
            tx_prover: _,
            wrap_prover,
            merge_prover,
            block_prover: _,
        } = make_provers();

        let mut witnesses: Witness<Fp> = Witness::new::<MergeProof>();
        generate_merge_proof(
            &statement,
            &proofs,
            &message,
            &merge_prover,
            &wrap_prover,
            &mut witnesses,
        );
    }

    #[test]
    fn test_block_proof() {
        let Ok(data) = std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("rampup4")
                .join("block_input_working.bin"),
        ) else {
            eprintln!("request not found");
            return;
        };

        let blockchain_input: v2::ProverExtendBlockchainInputStableV2 =
            read_binprot(&mut data.as_slice());

        // dbg!(blockchain_input);

        let Provers {
            tx_prover: _,
            wrap_prover,
            merge_prover: _,
            block_prover,
        } = make_provers();
        let mut witnesses: Witness<Fp> = Witness::new::<BlockProof>();

        generate_block_proof(
            &blockchain_input,
            &block_prover,
            &wrap_prover,
            &mut witnesses,
        );

        // let blockchain_input = v2::ProverExtendBlockchainInputStableV2::binprot_read(&mut data.as_slice()).unwrap();

        // let (statement, proofs, message) = extract_merge(&data);
        // let (_step_prover, wrap_prover, merge_prover) = make_provers();

        // let mut witnesses: Witness<Fp> = Witness::new::<MergeProof>();
        // generate_merge_proof(
        //     &statement,
        //     &proofs,
        //     &message,
        //     &merge_prover,
        //     &wrap_prover,
        //     &mut witnesses,
        // );
    }

    #[test]
    fn test_proofs() {
        let base_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("rampup4");

        let Provers {
            tx_prover,
            wrap_prover,
            merge_prover,
            block_prover: _,
        } = make_provers();

        // Merge proof
        {
            let data = std::fs::read(base_dir.join("merge_0_rampup4.bin")).unwrap();

            let (statement, proofs, message) = extract_merge(&data);

            let mut witnesses: Witness<Fp> = Witness::new::<MergeProof>();
            generate_merge_proof(
                &statement,
                &proofs,
                &message,
                &merge_prover,
                &wrap_prover,
                &mut witnesses,
            );
        }

        let base_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("rampup4");

        if !base_dir.exists() {
            eprintln!("{:?} not found", base_dir);
            return;
        }

        // Same values than OCaml
        #[rustfmt::skip]
        let requests = [
            ("request_payment_0_rampup4.bin", "c209c2f40caf61b29af5162476748ee7865eef0bc92eb1e6a50e52fc1d391c1e"),
            ("request_payment_1_rampup4.bin", "a5391b8ac8663a06a0a57ee6b6479e3cf4d95dfbb6d0688e439cb8c36cf187f6"),
            ("coinbase_0_rampup4.bin", "a2ce1982938687ca3ba3b1994e5100090a80649aefb1f0d10f736a845dab2812"),
            ("coinbase_1_rampup4.bin", "1120c9fe25078866e0df90fd09a41a2f5870351a01c8a7227d51a19290883efe"),
            ("coinbase_2_rampup4.bin", "7875781e8ea4a7eb9035a5510cd54cfc33229867f46f97e68fbb9a7a6534ec74"),
            ("coinbase_3_rampup4.bin", "12875cb8a182d550eb527e3561ad71458e1ca651ea399ee1878244c9b8f04966"),
            ("coinbase_4_rampup4.bin", "718cdc4b4803afd0f4d6ca38937211b196609f71c393f1195a55ff101d58f843"),
            ("coinbase_5_rampup4.bin", "a0d03705274ee56908a3fad1c260c56a0e07566d58c19bbba5c95cc8a9d11ee0"),
            ("coinbase_6_rampup4.bin", "4b213eeea865b9e6253f3c074017553243420b3183860a7f7720648677c02c54"),
            ("coinbase_7_rampup4.bin", "78fcec79bf2013d4f3d97628b316da7410af3c92a73dc26abc3ea63fbe92372a"),
            ("coinbase_8_rampup4.bin", "169f1ad4739d0a3fe194a66497bcabbca8dd5584cd83d13a5addede4b5a49e9d"),
            ("coinbase_9_rampup4.bin", "dfe50b656e0c0520a9678a1d34dd68af4620ea9909461b39c24bdda69504ed4b"),
            ("fee_transfer_0_rampup4.bin", "58d711bcc6377037e1c6a1334a49d53789b6e9c93aa343bda2f736cfc40d90b3"),
            ("fee_transfer_1_rampup4.bin", "791644dc9b5f17be24cbacab83e8b1f4b2ba7218e09ec718b37f1cd280b6c467"),
            ("fee_transfer_2_rampup4.bin", "ea02567ed5f116191ece0e7f6ac78a3b014079509457d03dd8d654e601404722"),
            ("fee_transfer_3_rampup4.bin", "6048053909b20e57cb104d1838c3aca565462605c69ced184f1a0e31b18c9c05"),
            ("fee_transfer_4_rampup4.bin", "1d6ab348dde0d008691dbb30ddb1412fabd5fe1adca788779c3674e2af412211"),
            ("fee_transfer_5_rampup4.bin", "a326eeeea08778795f35da77b43fc01c0c4b6cbf89cb1bb460c80bfab97d339e"),
            ("fee_transfer_6_rampup4.bin", "6b95aa737e1c8351bbb7a141108a73c808cb92aae9e266ecce13f679d6f6b2df"),
            ("fee_transfer_7_rampup4.bin", "5d97141c3adf576503381e485f5ab20ed856448880658a0a56fb23567225875c"),
            ("fee_transfer_8_rampup4.bin", "e1fa6b5a88b184428a0918cd4bd56952b54f05a5dc175b17e154204533167a78"),
            ("fee_transfer_9_rampup4.bin", "087a07eddedf5de18b2f2bd7ded3cd474d00a0030e9c13d7a5fd2433c72fc7d5"),
        ];

        for (file, expected_sum) in requests {
            let data = std::fs::read(base_dir.join(file)).unwrap();
            let (statement, tx_witness, message) = extract_request(&data);

            let mut witnesses: Witness<Fp> = Witness::new::<RegularTransactionProof>();
            let sum = generate_proof(
                &statement,
                &tx_witness,
                &message,
                &tx_prover,
                &wrap_prover,
                &mut witnesses,
            );

            if sum != expected_sum {
                eprintln!("Wrong proof: {:?}", file);
                eprintln!("got sum:  {:?}", sum);
                eprintln!("expected: {:?}", expected_sum);
                panic!()
            }
        }
    }
}
