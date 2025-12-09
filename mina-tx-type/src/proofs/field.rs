use ark_ec::{short_weierstrass::Projective, AffineRepr, CurveGroup};
use ark_ff::{BigInteger256, FftField, Field, One, PrimeField};
use kimchi::curve::KimchiCurve;
use mina_curves::pasta::{
    fields::fft::FpParameters as _, Fp, Fq, PallasParameters, ProjectivePallas, ProjectiveVesta,
    VestaParameters,
};
use mina_poseidon::{constants::PlonkSpongeConstantsKimchi, sponge::DefaultFqSponge};

use poseidon::SpongeParamsForField;

use super::{
    to_field_elements::ToFieldElements,
    witness::{Check, Witness},
};

pub const BACKEND_TICK_ROUNDS_N: usize = 16;
pub const BACKEND_TOCK_ROUNDS_N: usize = 17;

pub type GroupAffine<F> = ark_ec::short_weierstrass::Affine<<F as FieldWitness>::Parameters>;

/// All the generics we need during witness generation
pub trait FieldWitness
where
    Self: Field
        + Send
        + Sync
        + Into<BigInteger256>
        + From<BigInteger256>
        + Into<mina_p2p_messages::bigint::BigInt>
        + From<i64>
        + From<i32>
        + ToFieldElements<Self>
        + Check<Self>
        + FromFpFq
        + PrimeField
        + FftField
        + SpongeParamsForField<Self>
        + std::fmt::Debug
        + 'static,
{
    type Scalar: FieldWitness<Scalar = Self>;
    type Affine: AffineRepr<
            Group = Self::Projective,
            BaseField = Self,
            ScalarField = <Self as FieldWitness>::Scalar,
        > + Into<GroupAffine<Self>>
        + KimchiCurve
        + std::fmt::Debug;
    type Projective: CurveGroup<
            Affine = Self::Affine,
            BaseField = Self,
            ScalarField = <Self as FieldWitness>::Scalar,
        > + From<Projective<Self::Parameters>>
        + std::fmt::Debug;
    type Parameters: ark_ec::short_weierstrass::SWCurveConfig<
            BaseField = Self,
            ScalarField = <Self as FieldWitness>::Scalar,
        > + Clone
        + std::fmt::Debug;
    type Shifting: ShiftingValue<Self> + Clone + std::fmt::Debug;
    type OtherCurve: KimchiCurve<ScalarField = Self, BaseField = <Self as FieldWitness>::Scalar>;
    type FqSponge: Clone
        + mina_poseidon::FqSponge<<Self as FieldWitness>::Scalar, Self::OtherCurve, Self>;

    const PARAMS: Params<Self>;
    const SIZE: BigInteger256;
    const NROUNDS: usize;
    const SRS_DEPTH: usize;
}

pub struct Params<F> {
    pub a: F,
    pub b: F,
}

// Implement Check for Fp and Fq
impl Check<Fp> for Fp {
    fn check(&self, _w: &mut Witness<Fp>) {
        // Does not modify the witness
    }
}

impl Check<Fq> for Fq {
    fn check(&self, _w: &mut Witness<Fq>) {
        // Does not modify the witness
    }
}

// Implement Check for arrays
impl<F: FieldWitness, const N: usize> Check<F> for [F; N] {
    fn check(&self, _w: &mut Witness<F>) {
        // Does not modify the witness
    }
}

impl FieldWitness for Fp {
    type Scalar = Fq;
    type Parameters = PallasParameters;
    type Affine = GroupAffine<Self>;
    type Projective = ProjectivePallas;
    type Shifting = ShiftedValue<Fp>;
    type OtherCurve = GroupAffine<Fq>;
    type FqSponge = DefaultFqSponge<VestaParameters, PlonkSpongeConstantsKimchi>;

    /// <https://github.com/openmina/mina/blob/46b6403cb7f158b66a60fc472da2db043ace2910/src/lib/crypto/kimchi_backend/pasta/basic/kimchi_pasta_basic.ml#L107>
    const PARAMS: Params<Self> = Params::<Self> {
        a: ark_ff::MontFp!("0"),
        b: ark_ff::MontFp!("5"),
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
    type OtherCurve = GroupAffine<Fp>;
    type FqSponge = DefaultFqSponge<PallasParameters, PlonkSpongeConstantsKimchi>;

    /// <https://github.com/openmina/mina/blob/46b6403cb7f158b66a60fc472da2db043ace2910/src/lib/crypto/kimchi_backend/pasta/basic/kimchi_pasta_basic.ml#L95>
    const PARAMS: Params<Self> = Params::<Self> {
        a: ark_ff::MontFp!("0"),
        b: ark_ff::MontFp!("5"),
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
    fn from_fq(_fq: Fq) -> Self {
        // `Fq` cannot be converted into `Fp`.
        // Caller must first split `Fq` into 2 parts (high & low bits)
        // See `impl<F: FieldWitness> ToFieldElements<F> for Fq`
        panic!("Attempt to convert a `Fq` into `Fp`")
    }
}

impl FromFpFq for Fq {
    fn from_fp(fp: Fp) -> Self {
        // `Fp` is smaller than `Fq`, so the conversion is fine
        let bigint: BigInteger256 = fp.into();
        Self::from(bigint)
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

#[allow(clippy::module_inception)]
pub mod field {
    use super::*;

    // <https://github.com/o1-labs/snarky/blob/7edf13628872081fd7cad154de257dad8b9ba621/src/base/utils.ml#L99>
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

    // Note: compare and assert_lt functions are not included here as they
    // depend on field_to_bits2 from the transaction module. They remain
    // in the ledger crate.
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
    True = 1,
    False = 0,
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
    pub fn assert_non_zero<F: FieldWitness>(v: F, w: &mut Witness<F>) {
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

    pub fn var(&self) -> CircuitVar<Boolean> {
        CircuitVar::Var(*self)
    }

    pub fn constant(&self) -> CircuitVar<Boolean> {
        CircuitVar::Constant(*self)
    }
}

impl<F: FieldWitness> Check<F> for Boolean {
    fn check(&self, _w: &mut Witness<F>) {
        // Does not modify the witness
    }
}

impl<F: FieldWitness> Check<F> for CircuitVar<Boolean> {
    fn check(&self, _w: &mut Witness<F>) {
        // Does not modify the witness
    }
}

/// Our implementation of cvars (compare to OCaml) is incomplete, but sufficient
/// for now (for witness generation).
/// It's also sometimes not used correctly (`CircuitVar<GroupAffine<F>>` should be
/// `GroupAffine<CircuitVar<F>>` instead)
///
/// Note that our implementation of `CircuitVar<Boolean>` is complete.
#[derive(Clone, Copy, Debug)]
pub enum CircuitVar<F> {
    Var(F),
    Constant(F),
}

impl<F: FieldWitness> CircuitVar<F> {
    pub fn as_field(&self) -> F {
        match self {
            CircuitVar::Var(f) => *f,
            CircuitVar::Constant(f) => *f,
        }
    }

    fn scale(x: CircuitVar<F>, s: F) -> CircuitVar<F> {
        use CircuitVar::*;

        if s.is_zero() {
            Constant(F::zero())
        } else if s.is_one() {
            x
        } else {
            match x {
                Constant(x) => Constant(x * s),
                Var(x) => Var(x * s),
            }
        }
    }

    fn mul(&self, other: &Self, w: &mut Witness<F>) -> CircuitVar<F> {
        use CircuitVar::*;

        let (x, y) = (*self, *other);
        match (x, y) {
            (Constant(x), Constant(y)) => Constant(x * y),
            (Constant(x), _) => Self::scale(y, x),
            (_, Constant(y)) => Self::scale(x, y),
            (Var(x), Var(y)) => Var(w.exists(x * y)),
        }
    }

    fn equal(x: &Self, y: &Self, w: &mut Witness<F>) -> CircuitVar<Boolean> {
        match (x, y) {
            (CircuitVar::Constant(x), CircuitVar::Constant(y)) => {
                let eq = if x == y {
                    Boolean::True
                } else {
                    Boolean::False
                };
                CircuitVar::Constant(eq)
            }
            _ => {
                let x = x.as_field();
                let y = y.as_field();
                CircuitVar::Var(field::equal(x, y, w))
            }
        }
    }
}

impl<T> CircuitVar<T> {
    pub fn is_const(&self) -> bool {
        match self {
            CircuitVar::Var(_) => false,
            CircuitVar::Constant(_) => true,
        }
    }

    pub fn value(&self) -> &T {
        match self {
            CircuitVar::Var(v) => v,
            CircuitVar::Constant(v) => v,
        }
    }

    pub fn map<Fun, V>(&self, fun: Fun) -> CircuitVar<V>
    where
        Fun: Fn(&T) -> V,
    {
        match self {
            CircuitVar::Var(v) => CircuitVar::Var(fun(v)),
            CircuitVar::Constant(v) => CircuitVar::Constant(fun(v)),
        }
    }
}

impl CircuitVar<Boolean> {
    pub fn as_boolean(&self) -> Boolean {
        match self {
            CircuitVar::Var(b) => *b,
            CircuitVar::Constant(b) => *b,
        }
    }

    fn as_cvar<F: FieldWitness>(&self) -> CircuitVar<F> {
        self.map(|b| b.to_field::<F>())
    }

    pub fn of_cvar<F: FieldWitness>(cvar: CircuitVar<F>) -> Self {
        cvar.map(|b| {
            // TODO: Should we check for `is_one` or `is_zero` here ? To match OCaml behavior
            if b.is_one() {
                Boolean::True
            } else {
                Boolean::False
            }
        })
    }

    pub fn and<F: FieldWitness>(&self, other: &Self, w: &mut Witness<F>) -> Self {
        self.as_cvar().mul(&other.as_cvar(), w).map(|v| {
            // TODO: Should we check for `is_one` or `is_zero` here ? To match OCaml behavior
            if v.is_one() {
                Boolean::True
            } else {
                Boolean::False
            }
        })
    }

    fn boolean_sum<F: FieldWitness>(x: &[Self]) -> CircuitVar<F> {
        let sum = x.iter().fold(0u64, |acc, b| {
            acc + match b.as_boolean() {
                Boolean::True => 1,
                Boolean::False => 0,
            }
        });
        if x.iter().all(|x| matches!(x, CircuitVar::Constant(_))) {
            CircuitVar::Constant(F::from(sum))
        } else {
            CircuitVar::Var(F::from(sum))
        }
    }

    pub fn any<F: FieldWitness>(x: &[Self], w: &mut Witness<F>) -> CircuitVar<Boolean> {
        match x {
            [] => CircuitVar::Constant(Boolean::False),
            [b1] => *b1,
            [b1, b2] => b1.or(b2, w),
            bs => {
                let sum = Self::boolean_sum(bs);
                CircuitVar::equal(&sum, &CircuitVar::Constant(F::zero()), w).map(Boolean::neg)
            }
        }
    }

    pub fn all<F: FieldWitness>(x: &[Self], w: &mut Witness<F>) -> CircuitVar<Boolean> {
        match x {
            [] => CircuitVar::Constant(Boolean::True),
            [b1] => *b1,
            [b1, b2] => b1.and(b2, w),
            bs => {
                let sum = Self::boolean_sum(bs);
                let len = F::from(bs.len() as u64);
                CircuitVar::equal(&CircuitVar::Constant(len), &sum, w)
            }
        }
    }

    pub fn neg(&self) -> Self {
        self.map(Boolean::neg)
    }

    pub fn or<F: FieldWitness>(&self, other: &Self, w: &mut Witness<F>) -> Self {
        let both_false = self.neg().and(&other.neg(), w);
        both_false.neg()
    }

    pub fn lxor<F: FieldWitness>(&self, other: &Self, w: &mut Witness<F>) -> Self {
        match (self, other) {
            (CircuitVar::Var(a), CircuitVar::Var(b)) => CircuitVar::Var(a.lxor(b, w)),
            (CircuitVar::Constant(a), CircuitVar::Constant(b)) => {
                CircuitVar::Constant(a.const_lxor(b))
            }
            (a, b) => CircuitVar::Var(a.as_boolean().const_lxor(&b.as_boolean())),
        }
    }

    pub fn equal_bool<F: FieldWitness>(&self, other: &Self, w: &mut Witness<F>) -> Self {
        self.lxor(other, w).neg()
    }

    pub fn assert_any<F: FieldWitness>(bs: &[Self], w: &mut Witness<F>) {
        let num_true = bs.iter().fold(0u64, |acc, b| {
            acc + match b.as_boolean() {
                Boolean::True => 1,
                Boolean::False => 0,
            }
        });
        Boolean::assert_non_zero::<F>(F::from(num_true), w)
    }
}

// ShiftingValue trait and related types

pub trait ShiftingValue<F: Field> {
    type MyShift;
    fn shift() -> Self::MyShift;
    fn of_field(field: F) -> Self;
    fn shifted_to_field(&self) -> F;
    fn shifted_raw(&self) -> F;
    fn of_raw(shifted: F) -> Self;
}

#[derive(Clone, Debug)]
pub struct Shift<F: Field> {
    pub c: F,
    pub scale: F,
}

impl<F> Shift<F>
where
    F: Field + From<i32>,
{
    /// <https://github.com/MinaProtocol/mina/blob/0b63498e271575dbffe2b31f3ab8be293490b1ac/src/lib/pickles_types/shifted_value.ml#L121>
    pub fn create() -> Self {
        let c = (0..255).fold(F::one(), |accum, _| accum + accum) + F::one();

        let scale: F = 2.into();
        let scale = scale.inverse().unwrap();

        Self { c, scale } // TODO: This can be a constant
    }
}

#[derive(Clone, Debug)]
pub struct ShiftedValue<F: Field> {
    pub shifted: F,
}

impl<F: FieldWitness, F2: FieldWitness + ToFieldElements<F>> ToFieldElements<F>
    for ShiftedValue<F2>
{
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self { shifted } = self;
        shifted.to_field_elements(fields);
    }
}

impl<F> ShiftedValue<F>
where
    F: Field,
{
    /// Creates without shifting
    pub fn new(field: F) -> Self {
        Self { shifted: field }
    }
}

#[derive(Clone, Debug)]
pub struct ShiftFq {
    pub shift: Fq,
}

impl ShiftingValue<Fp> for ShiftedValue<Fp> {
    type MyShift = Shift<Fp>;

    fn shift() -> Self::MyShift {
        let c = (0..255).fold(Fp::one(), |accum, _| accum + accum) + Fp::one();

        let scale: Fp = 2.into();
        let scale = scale.inverse().unwrap();

        Shift { c, scale }
    }

    fn of_field(field: Fp) -> Self {
        let shift = Self::shift();
        Self {
            shifted: (field - shift.c) * shift.scale,
        }
    }

    fn shifted_to_field(&self) -> Fp {
        let shift = Self::shift();
        self.shifted + self.shifted + shift.c
    }

    fn shifted_raw(&self) -> Fp {
        self.shifted
    }

    fn of_raw(shifted: Fp) -> Self {
        Self { shifted }
    }
}

impl ShiftingValue<Fq> for ShiftedValue<Fq> {
    type MyShift = ShiftFq;

    fn shift() -> Self::MyShift {
        ShiftFq {
            shift: (0..255).fold(Fq::one(), |accum, _| accum + accum),
        }
    }

    fn of_field(field: Fq) -> Self {
        let shift = Self::shift();
        Self {
            shifted: field - shift.shift,
        }
    }

    fn shifted_to_field(&self) -> Fq {
        let shift = Self::shift();
        self.shifted + shift.shift
    }

    fn shifted_raw(&self) -> Fq {
        self.shifted
    }

    fn of_raw(shifted: Fq) -> Self {
        Self { shifted }
    }
}
