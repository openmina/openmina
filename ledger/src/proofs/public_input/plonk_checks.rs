use std::rc::Rc;

use ark_ff::{Field, One};
use ark_poly::Radix2EvaluationDomain;
use kimchi::{
    circuits::expr::RowOffset,
    curve::KimchiCurve,
    proof::{PointEvaluations, ProofEvaluations},
};
use mina_curves::pasta::Fq;
use mina_hasher::Fp;

use crate::{
    proofs::{
        field::{field, Boolean, FieldWitness},
        public_input::plonk_checks::scalars::MinimalForScalar,
        step::step_verifier::PlonkDomain,
        to_field_elements::ToFieldElements,
        witness::Witness,
        wrap::{wrap_verifier::PlonkWithField, AllFeatureFlags},
    },
    scan_state::transaction_logic::local_state::LazyValue,
};

#[derive(Clone, Debug)]
pub struct PlonkMinimal<F: FieldWitness, const NLIMB: usize = 2> {
    pub alpha: F,
    pub beta: F,
    pub gamma: F,
    pub zeta: F,
    pub joint_combiner: Option<F>,
    pub alpha_bytes: [u64; NLIMB],
    pub beta_bytes: [u64; NLIMB],
    pub gamma_bytes: [u64; NLIMB],
    pub zeta_bytes: [u64; NLIMB],
    pub joint_combiner_bytes: Option<[u64; NLIMB]>,
    pub feature_flags: crate::proofs::step::FeatureFlags<bool>,
}

type TwoFields = [Fp; 2];

pub struct ScalarsEnv<F: FieldWitness> {
    pub zk_polynomial: F,
    pub zeta_to_n_minus_1: F,
    pub srs_length_log2: u64,
    pub domain: Rc<dyn PlonkDomain<F>>,
    pub omega_to_minus_zk_rows: F,
    pub feature_flags: Option<AllFeatureFlags<F>>,
    pub unnormalized_lagrange_basis: Option<Box<dyn Fn(RowOffset, &mut Witness<F>) -> F>>,
    pub vanishes_on_zero_knowledge_and_previous_rows: F,
    pub zeta_to_srs_length: LazyValue<F, Witness<F>>,
}

// Result of `plonk_derive`
#[derive(Debug)]
pub struct InCircuit<F: FieldWitness> {
    pub alpha: F,
    pub beta: F,
    pub gamma: F,
    pub zeta: F,
    pub zeta_to_domain_size: F::Shifting,
    pub zeta_to_srs_length: F::Shifting,
    pub perm: F::Shifting,
    pub lookup: Option<F>,
    pub feature_flags: crate::proofs::step::FeatureFlags<bool>,
}

pub trait ShiftingValue<F: Field> {
    type MyShift;
    fn shift() -> Self::MyShift;
    fn of_field(field: F) -> Self;
    fn shifted_to_field(&self) -> F;
    fn shifted_raw(&self) -> F;
    fn of_raw(shifted: F) -> Self;
}

impl ShiftingValue<Fp> for ShiftedValue<Fp> {
    type MyShift = Shift<Fp>;

    fn shift() -> Self::MyShift {
        type MyShift = Shift<Fp>;

        cache_one! {MyShift, {
            let c = (0..255).fold(Fp::one(), |accum, _| accum + accum) + Fp::one();

            let scale: Fp = 2.into();
            let scale = scale.inverse().unwrap();

            Shift { c, scale }
        }}
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
        cache_one! {ShiftFq, {
            ShiftFq {
                shift: (0..255).fold(Fq::one(), |accum, _| accum + accum),
            }
        }}
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

#[derive(Clone, Debug)]
pub struct ShiftFq {
    shift: Fq,
}

#[derive(Clone, Debug)]
pub struct Shift<F: Field> {
    c: F,
    scale: F,
}

impl<F> Shift<F>
where
    F: Field + From<i32>,
{
    /// https://github.com/MinaProtocol/mina/blob/0b63498e271575dbffe2b31f3ab8be293490b1ac/src/lib/pickles_types/shifted_value.ml#L121
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

// impl<F: Field + FpExt> std::fmt::Debug for ShiftedValue<F> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("ShiftedValue")
//             .field("shifted", &{
//                 let mut bytes = self.shifted.to_bytes();
//                 bytes.reverse();
//                 hex::encode(bytes)
//             })
//             .finish()
//     }
// }

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

    // /// https://github.com/MinaProtocol/mina/blob/0b63498e271575dbffe2b31f3ab8be293490b1ac/src/lib/pickles_types/shifted_value.ml#L127
    // pub fn of_field(field: F, shift: &Shift<F>) -> Self {
    //     Self {
    //         shifted: (field - shift.c) * shift.scale,
    //     }
    // }

    // /// https://github.com/MinaProtocol/mina/blob/0b63498e271575dbffe2b31f3ab8be293490b1ac/src/lib/pickles_types/shifted_value.ml#L131
    // #[allow(unused)]
    // pub fn to_field(&self, shift: &Shift<F>) -> F {
    //     self.shifted + self.shifted + shift.c
    // }
}

/// https://github.com/MinaProtocol/mina/blob/0b63498e271575dbffe2b31f3ab8be293490b1ac/src/lib/pickles/plonk_checks/plonk_checks.ml#L218
pub const PERM_ALPHA0: usize = 21;

pub const NPOWERS_OF_ALPHA: usize = PERM_ALPHA0 + 3;

/// https://github.com/MinaProtocol/mina/blob/0b63498e271575dbffe2b31f3ab8be293490b1ac/src/lib/pickles/plonk_checks/plonk_checks.ml#L141
pub fn powers_of_alpha<F: FieldWitness>(alpha: F) -> Box<[F; NPOWERS_OF_ALPHA]> {
    // The OCaml code computes until alpha^71, but we don't need that much here
    let mut alphas = Box::new([F::one(); NPOWERS_OF_ALPHA]);

    alphas[1] = alpha;
    for i in 2..alphas.len() {
        alphas[i] = alpha * alphas[i - 1];
    }

    alphas
}

pub fn derive_plonk<F: FieldWitness, const NLIMB: usize>(
    env: &ScalarsEnv<F>,
    evals: &ProofEvaluations<PointEvaluations<F>>,
    minimal: &PlonkMinimal<F, NLIMB>,
) -> InCircuit<F> {
    let PlonkMinimal {
        alpha,
        beta,
        gamma,
        joint_combiner,
        feature_flags: actual_feature_flags,
        ..
    } = minimal;

    let zkp = env.zk_polynomial;
    let powers_of_alpha = powers_of_alpha(*alpha);
    let alpha_pow = |i: usize| powers_of_alpha[i];
    let w0 = evals.w.map(|point| point.fst());

    let beta = *beta;
    let gamma = *gamma;

    // https://github.com/MinaProtocol/mina/blob/0b63498e271575dbffe2b31f3ab8be293490b1ac/src/lib/pickles/plonk_checks/plonk_checks.ml#L397
    let perm = evals.s.iter().enumerate().fold(
        evals.z.snd() * beta * alpha_pow(PERM_ALPHA0) * zkp,
        |accum, (index, elem)| accum * (gamma + (beta * elem.fst()) + w0[index]),
    );
    let perm = -perm;

    let zeta_to_domain_size = env.zeta_to_n_minus_1 + F::one();

    let zeta_to_srs_length = {
        let mut unused_w = Witness::empty();
        *env.zeta_to_srs_length.get(&mut unused_w)
    };

    // Shift values
    let shift = |f: F| F::Shifting::of_field(f);

    InCircuit {
        alpha: minimal.alpha,
        beta: minimal.beta,
        gamma: minimal.gamma,
        zeta: minimal.zeta,
        zeta_to_domain_size: shift(zeta_to_domain_size),
        zeta_to_srs_length: shift(zeta_to_srs_length),
        perm: shift(perm),
        lookup: joint_combiner
            .as_ref()
            .map(|joint_combiner| *joint_combiner),
        feature_flags: actual_feature_flags.clone(),
    }
}

// TODO: De-duplicate with `derive_plonk`
pub fn derive_plonk_checked<F: FieldWitness>(
    env: &ScalarsEnv<F>,
    evals: &ProofEvaluations<PointEvaluations<F>>,
    minimal: &PlonkWithField<F>,
    w: &mut Witness<F>,
) -> InCircuit<F> {
    // use kimchi::circuits::gate::GateType;

    let zkp = env.zk_polynomial;
    let powers_of_alpha = powers_of_alpha(minimal.alpha);
    let alpha_pow = |i: usize| powers_of_alpha[i];
    let w0 = evals.w.map(|point| point.fst());

    let beta = minimal.beta;
    let gamma = minimal.gamma;

    let perm = evals.s.iter().enumerate().fold(
        field::muls(&[evals.z.snd(), beta, alpha_pow(PERM_ALPHA0), zkp], w),
        |accum, (index, elem)| {
            // We decompose this way because of OCaml evaluation order
            let beta_elem = field::mul(beta, elem.fst(), w);
            field::mul(accum, gamma + beta_elem + w0[index], w)
        },
    );
    let perm = -perm;

    let zeta_to_domain_size = env.zeta_to_n_minus_1 + F::one();
    // https://github.com/MinaProtocol/mina/blob/0b63498e271575dbffe2b31f3ab8be293490b1ac/src/lib/pickles/plonk_checks/plonk_checks.ml#L46

    // let minimal_for_scalar = MinimalForScalar {
    //     alpha: minimal.alpha,
    //     beta: minimal.beta,
    //     gamma: minimal.gamma,
    // };

    // We decompose this way because of OCaml evaluation order
    // use GateType::{CompleteAdd, EndoMul, EndoMulScalar, VarBaseMul};
    // let endomul_scalar = scalars::compute(Some(EndoMulScalar), &minimal_for_scalar, evals, w);
    // let endomul = scalars::compute(Some(EndoMul), &minimal_for_scalar, evals, w);
    // let complete_add = scalars::compute(Some(CompleteAdd), &minimal_for_scalar, evals, w);
    // let vbmul = scalars::compute(Some(VarBaseMul), &minimal_for_scalar, evals, w);

    let zeta_to_srs_length = *env.zeta_to_srs_length.get(w);

    // Shift values
    let shift = |f: F| F::Shifting::of_field(f);

    InCircuit {
        alpha: minimal.alpha,
        beta: minimal.beta,
        gamma: minimal.gamma,
        zeta: minimal.zeta,
        zeta_to_domain_size: shift(zeta_to_domain_size),
        zeta_to_srs_length: shift(zeta_to_srs_length),
        perm: shift(perm),
        lookup: None,
        feature_flags: crate::proofs::step::FeatureFlags::empty_bool(),
    }
}

pub fn checked<F: FieldWitness>(
    env: &ScalarsEnv<F>,
    evals: &ProofEvaluations<PointEvaluations<F>>,
    plonk: &PlonkWithField<F>,
    w: &mut Witness<F>,
) -> Boolean {
    let actual = derive_plonk_checked(env, evals, plonk, w);

    let list = [
        // field::equal(plonk.vbmul.shifted, actual.vbmul.shifted, w),
        // field::equal(plonk.complete_add.shifted, actual.complete_add.shifted, w),
        // field::equal(plonk.endomul.shifted, actual.endomul.shifted, w),
        field::equal(plonk.perm.shifted, actual.perm.shifted_raw(), w),
    ];

    Boolean::all(&list[..], w)
}

pub fn make_shifts<F: FieldWitness>(
    domain: &Radix2EvaluationDomain<F>,
) -> kimchi::circuits::polynomials::permutation::Shifts<F> {
    // let value = 1 << log2_size;
    // let domain = Domain::<Fq>::new(value).unwrap();
    kimchi::circuits::polynomials::permutation::Shifts::new(domain)
}

pub fn ft_eval0<F: FieldWitness, const NLIMB: usize>(
    env: &ScalarsEnv<F>,
    evals: &ProofEvaluations<PointEvaluations<F>>,
    minimal: &PlonkMinimal<F, NLIMB>,
    p_eval0: &[F],
) -> F {
    const PLONK_TYPES_PERMUTS_MINUS_1_N: usize = 6;

    let e0_s: Vec<_> = evals.s.iter().map(|s| s.fst()).collect();
    let zkp = env.zk_polynomial;
    let powers_of_alpha = powers_of_alpha(minimal.alpha);
    let alpha_pow = |i: usize| powers_of_alpha[i];

    let zeta1m1 = env.zeta_to_n_minus_1;

    let mut unused_w = Witness::empty();
    let p_eval0 = p_eval0
        .iter()
        .copied()
        .rfold(None, |acc, p_eval0| match acc {
            None => Some(p_eval0),
            Some(acc) => {
                let zeta1 = *env.zeta_to_srs_length.get(&mut unused_w);
                Some(p_eval0 + (zeta1 * acc))
            }
        })
        .unwrap(); // Never fail, `p_eval0` is non-empty

    let w0: Vec<_> = evals.w.iter().map(|w| w.fst()).collect();

    let ft_eval0 = {
        let a0 = alpha_pow(PERM_ALPHA0);
        let w_n = w0[PLONK_TYPES_PERMUTS_MINUS_1_N];
        let init = (w_n + minimal.gamma) * evals.z.snd() * a0 * zkp;
        e0_s.iter().enumerate().fold(init, |acc, (i, s)| {
            ((minimal.beta * s) + w0[i] + minimal.gamma) * acc
        })
    };

    let shifts = env.domain.shifts();
    let ft_eval0 = ft_eval0 - p_eval0;

    let ft_eval0 = ft_eval0
        - shifts.iter().enumerate().fold(
            alpha_pow(PERM_ALPHA0) * zkp * evals.z.fst(),
            |acc, (i, s)| acc * (minimal.gamma + (minimal.beta * minimal.zeta * s) + w0[i]),
        );

    let nominator =
        (zeta1m1 * alpha_pow(PERM_ALPHA0 + 1) * (minimal.zeta - env.omega_to_minus_zk_rows)
            + (zeta1m1 * alpha_pow(PERM_ALPHA0 + 2) * (minimal.zeta - F::one())))
            * (F::one() - evals.z.fst());

    let denominator = (minimal.zeta - env.omega_to_minus_zk_rows) * (minimal.zeta - F::one());
    let ft_eval0 = ft_eval0 + (nominator / denominator);

    let minimal = MinimalForScalar {
        alpha: minimal.alpha,
        beta: minimal.beta,
        gamma: minimal.gamma,
        lookup: minimal.joint_combiner,
    };
    let mut w = Witness::empty();
    let constant_term = scalars::compute(None, &minimal, evals, env, &mut w);

    ft_eval0 - constant_term
}

fn get_feature_flag<F: FieldWitness>(
    feature_flags: &AllFeatureFlags<F>,
    feature: &kimchi::circuits::expr::FeatureFlag,
    w: &mut Witness<F>,
) -> Option<Boolean> {
    use kimchi::circuits::expr::FeatureFlag::*;
    use kimchi::circuits::lookup::lookups::LookupPattern;

    match feature {
        RangeCheck0 => Some(feature_flags.features.range_check0),
        RangeCheck1 => Some(feature_flags.features.range_check1),
        ForeignFieldAdd => Some(feature_flags.features.foreign_field_add),
        ForeignFieldMul => Some(feature_flags.features.foreign_field_mul),
        Xor => Some(feature_flags.features.xor),
        Rot => Some(feature_flags.features.rot),
        LookupTables => Some(*feature_flags.lookup_tables.get(w)),
        RuntimeLookupTables => Some(feature_flags.features.runtime_tables),
        TableWidth(3) => Some(*feature_flags.table_width_3.get(w)),
        TableWidth(2) => Some(*feature_flags.table_width_at_least_2.get(w)),
        TableWidth(i) if *i <= 1 => Some(*feature_flags.table_width_at_least_1.get(w)),
        TableWidth(_) => None,
        LookupsPerRow(4) => Some(*feature_flags.lookups_per_row_4.get(w)),
        LookupsPerRow(i) if *i <= 3 => Some(*feature_flags.lookups_per_row_3.get(w)),
        LookupsPerRow(_) => None,
        LookupPattern(LookupPattern::Lookup) => Some(feature_flags.features.lookup),
        LookupPattern(LookupPattern::Xor) => Some(*feature_flags.lookup_pattern_xor.get(w)),
        LookupPattern(LookupPattern::RangeCheck) => {
            Some(*feature_flags.lookup_pattern_range_check.get(w))
        }
        LookupPattern(LookupPattern::ForeignFieldMul) => {
            Some(feature_flags.features.foreign_field_mul)
        }
    }
}

mod scalars {
    use std::collections::BTreeMap;

    use kimchi::{
        circuits::{
            constraints::FeatureFlags,
            expr::{CacheId, Column, ConstantExpr, Constants, Expr, ExprError, Op2, Variable},
            gate::{CurrOrNext, GateType},
            lookup::lookups::{LookupFeatures, LookupPatterns},
        },
        proof::PointEvaluations,
    };

    use crate::proofs::transaction::endos;

    use super::*;

    // This method `Variable::evaluate` is private in proof-systems :(
    fn var_evaluate<F: FieldWitness>(
        v: &Variable,
        evals: &ProofEvaluations<PointEvaluations<F>>,
    ) -> Result<F, ExprError> {
        let point_evaluations = {
            use kimchi::circuits::lookup::lookups::LookupPattern;
            use Column::*;

            match v.col {
                Witness(i) => Ok(evals.w[i]),
                Z => Ok(evals.z),
                LookupSorted(i) => {
                    evals.lookup_sorted[i].ok_or(ExprError::MissingIndexEvaluation(v.col))
                }
                LookupAggreg => evals
                    .lookup_aggregation
                    .ok_or(ExprError::MissingIndexEvaluation(v.col)),
                LookupTable => evals
                    .lookup_table
                    .ok_or(ExprError::MissingIndexEvaluation(v.col)),
                LookupRuntimeTable => evals
                    .runtime_lookup_table
                    .ok_or(ExprError::MissingIndexEvaluation(v.col)),
                Index(GateType::Poseidon) => Ok(evals.poseidon_selector),
                Index(GateType::Generic) => Ok(evals.generic_selector),
                Index(GateType::CompleteAdd) => Ok(evals.complete_add_selector),
                Index(GateType::VarBaseMul) => Ok(evals.mul_selector),
                Index(GateType::EndoMul) => Ok(evals.emul_selector),
                Index(GateType::EndoMulScalar) => Ok(evals.endomul_scalar_selector),
                Index(GateType::RangeCheck0) => evals
                    .range_check0_selector
                    .ok_or(ExprError::MissingIndexEvaluation(v.col)),
                Index(GateType::RangeCheck1) => evals
                    .range_check1_selector
                    .ok_or(ExprError::MissingIndexEvaluation(v.col)),
                Index(GateType::ForeignFieldAdd) => evals
                    .foreign_field_add_selector
                    .ok_or(ExprError::MissingIndexEvaluation(v.col)),
                Index(GateType::ForeignFieldMul) => evals
                    .foreign_field_mul_selector
                    .ok_or(ExprError::MissingIndexEvaluation(v.col)),
                Index(GateType::Xor16) => evals
                    .xor_selector
                    .ok_or(ExprError::MissingIndexEvaluation(v.col)),
                Index(GateType::Rot64) => evals
                    .rot_selector
                    .ok_or(ExprError::MissingIndexEvaluation(v.col)),
                Permutation(i) => Ok(evals.s[i]),
                Coefficient(i) => Ok(evals.coefficients[i]),
                Column::LookupKindIndex(LookupPattern::Xor) => evals
                    .xor_lookup_selector
                    .ok_or(ExprError::MissingIndexEvaluation(v.col)),
                Column::LookupKindIndex(LookupPattern::Lookup) => evals
                    .lookup_gate_lookup_selector
                    .ok_or(ExprError::MissingIndexEvaluation(v.col)),
                Column::LookupKindIndex(LookupPattern::RangeCheck) => evals
                    .range_check_lookup_selector
                    .ok_or(ExprError::MissingIndexEvaluation(v.col)),
                Column::LookupKindIndex(LookupPattern::ForeignFieldMul) => evals
                    .foreign_field_mul_lookup_selector
                    .ok_or(ExprError::MissingIndexEvaluation(v.col)),
                Column::LookupRuntimeSelector => evals
                    .runtime_lookup_table_selector
                    .ok_or(ExprError::MissingIndexEvaluation(v.col)),
                Index(_) => Err(ExprError::MissingIndexEvaluation(v.col)),
            }
        }?;
        match v.row {
            CurrOrNext::Curr => Ok(point_evaluations.zeta),
            CurrOrNext::Next => Ok(point_evaluations.zeta_omega),
        }
    }

    fn pow<F: FieldWitness>(x: F, n: u64, w: &mut Witness<F>) -> F {
        if n == 0 {
            F::one()
        } else if n == 1 {
            x
        } else {
            let y = pow(field::square(x, w), n / 2, w);
            if n % 2 == 0 {
                y
            } else {
                field::mul(x, y, w)
            }
        }
    }

    fn pow_const<F: FieldWitness>(x: F, n: u64) -> F {
        if n == 0 {
            F::one()
        } else if n == 1 {
            x
        } else {
            (0..n - 1).fold(x, |acc, _| x * acc)
        }
    }

    pub struct EvalContext<'a, F: FieldWitness> {
        pub evals: &'a ProofEvaluations<PointEvaluations<F>>,
        pub constants: &'a Constants<F>,
        pub cache: BTreeMap<CacheId, F>,
        pub env: &'a ScalarsEnv<F>,
        pub w: &'a mut Witness<F>,
    }

    // TODO: Use cvar instead
    fn is_const<F: FieldWitness>(e: &Expr<ConstantExpr<F>>) -> bool {
        use ConstantExpr::*;
        match e {
            Expr::Constant(c) => matches!(c, EndoCoefficient | Literal(_) | Mds { .. }),
            Expr::BinOp(_, x, y) => is_const(x) && is_const(y),
            Expr::Pow(x, _) => is_const(x),
            _ => false,
        }
    }

    pub fn eval<F: FieldWitness>(e: &Expr<ConstantExpr<F>>, ctx: &mut EvalContext<F>) -> F {
        use Expr::*;
        match e {
            Double(x) => {
                let v = eval(x, ctx);
                v.double()
            }
            Constant(x) => {
                let v = x.value(ctx.constants);
                if let ConstantExpr::Mul(_, _) = x {
                    ctx.w.exists_no_check(v);
                };
                v
            }
            Pow(x, p) => {
                let p = *p;
                let v = eval(x, ctx);

                if is_const(x) {
                    pow_const(v, p)
                } else {
                    pow(v, p, ctx.w)
                }
            }
            BinOp(Op2::Mul, x, y) => {
                let is_x_const = is_const(x);
                let is_y_const = is_const(y);
                let y = eval(y, ctx);
                let x = eval(x, ctx);
                if is_x_const || is_y_const {
                    x * y
                } else {
                    field::mul(x, y, ctx.w)
                }
            }
            Square(x) => {
                let is_x_const = is_const(x);
                let x = eval(x, ctx);
                if is_x_const {
                    x * x
                } else {
                    field::mul(x, x, ctx.w)
                }
            }
            BinOp(Op2::Add, x, y) => {
                let y = eval(y, ctx);
                let x = eval(x, ctx);
                x + y
            }
            BinOp(Op2::Sub, x, y) => {
                let y = eval(y, ctx);
                let x = eval(x, ctx);
                x - y
            }
            VanishesOnZeroKnowledgeAndPreviousRows => {
                ctx.env.vanishes_on_zero_knowledge_and_previous_rows
            }
            UnnormalizedLagrangeBasis(i) => {
                let unnormalized_lagrange_basis =
                    ctx.env.unnormalized_lagrange_basis.as_ref().unwrap();
                unnormalized_lagrange_basis(*i, ctx.w)
            }
            Cell(v) => {
                var_evaluate(v, ctx.evals).unwrap_or_else(|_| F::zero()) // TODO: Is that correct ?
            }
            Cache(id, _e) => {
                ctx.cache.get(id).copied().unwrap() // Cached values were already computed
            }
            IfFeature(feature, e1, e2) => match ctx.env.feature_flags.as_ref() {
                None => eval(e2, ctx),
                Some(feature_flags) => {
                    let is_feature_enabled = match get_feature_flag(feature_flags, feature, ctx.w) {
                        None => return eval(e2, ctx),
                        Some(enabled) => enabled,
                    };

                    let on_false = eval(e2, ctx);
                    let on_true = eval(e1, ctx);

                    ctx.w.exists_no_check(match is_feature_enabled {
                        Boolean::True => on_true,
                        Boolean::False => on_false,
                    })
                }
            },
        }
    }

    #[derive(Default)]
    pub struct Cached<F: FieldWitness> {
        /// cache may contain their own caches
        expr: BTreeMap<CacheId, (Box<Cached<F>>, Box<Expr<ConstantExpr<F>>>)>,
    }

    #[inline(never)]
    pub fn extract_caches<F: FieldWitness>(e: &Expr<ConstantExpr<F>>, cache: &mut Cached<F>) {
        use Expr::*;
        match e {
            Double(x) => {
                extract_caches(x, cache);
            }
            Constant(_x) => (),
            Pow(x, _p) => {
                extract_caches(x, cache);
            }
            BinOp(Op2::Mul, x, y) => {
                extract_caches(y, cache);
                extract_caches(x, cache);
            }
            Square(x) => {
                extract_caches(x, cache);
            }
            BinOp(Op2::Add, x, y) => {
                extract_caches(y, cache);
                extract_caches(x, cache);
            }
            BinOp(Op2::Sub, x, y) => {
                extract_caches(y, cache);
                extract_caches(x, cache);
            }
            VanishesOnZeroKnowledgeAndPreviousRows => todo!(),
            UnnormalizedLagrangeBasis(_i) => todo!(),
            Cell(_v) => (),
            Cache(id, e) => {
                let mut cached = Cached::default();
                extract_caches(e, &mut cached);
                cache.expr.insert(*id, (Box::new(cached), e.clone()));
            }
            IfFeature(_feature, e1, e2) => {
                if false {
                    extract_caches(e1, cache)
                } else {
                    extract_caches(e2, cache)
                }
            }
        }
    }

    fn eval_cache<F: FieldWitness>(cached_exprs: &Cached<F>, ctx: &mut EvalContext<F>) {
        // Each cached expression may contain their own caches
        for (id, (cache, expr)) in &cached_exprs.expr {
            let mut old_cache = std::mem::take(&mut ctx.cache);
            eval_cache::<F>(cache, ctx);
            old_cache.insert(*id, eval::<F>(expr, ctx));
            ctx.cache = old_cache;
        }
    }

    pub struct MinimalForScalar<F> {
        pub alpha: F,
        pub beta: F,
        pub gamma: F,
        pub lookup: Option<F>,
    }

    pub fn compute<F: FieldWitness>(
        gate: Option<GateType>,
        minimal: &MinimalForScalar<F>,
        evals: &ProofEvaluations<PointEvaluations<F>>,
        env: &ScalarsEnv<F>,
        w: &mut Witness<F>,
    ) -> F {
        let (constant_term, index_terms) = &*{
            type TermsMap<F> = BTreeMap<Column, Expr<ConstantExpr<F>>>;
            type Const<F> = Expr<ConstantExpr<F>>;
            type Terms<F> = Rc<(Const<F>, TermsMap<F>)>;
            cache! {
                Terms::<F>, {
                    // No features for `Fp`:
                    // https://github.com/MinaProtocol/mina/blob/4af0c229548bc96d76678f11b6842999de5d3b0b/src/lib/crypto/kimchi_bindings/stubs/src/linearization.rs
                    let is_fp = std::any::TypeId::of::<F>() == std::any::TypeId::of::<Fp>();

                    let features = if is_fp {
                        None
                    } else {
                        Some(FeatureFlags {
                            range_check0: false,
                            range_check1: false,
                            foreign_field_add: false,
                            foreign_field_mul: false,
                            xor: false,
                            rot: false,
                            lookup_features: LookupFeatures {
                                patterns: LookupPatterns {
                                    xor: false,
                                    lookup: false,
                                    range_check: false,
                                    foreign_field_mul: false,
                                },
                                joint_lookup_used: false,
                                uses_runtime_tables: false,
                            },
                        })
                    };

                    let evaluated_cols =
                        kimchi::linearization::linearization_columns::<F>(features.as_ref());
                    let (linearization, _powers_of_alpha) =
                        kimchi::linearization::constraints_expr::<F>(features.as_ref(), true);

                    let kimchi::circuits::expr::Linearization {
                        constant_term,
                        index_terms,
                    } = linearization.linearize(evaluated_cols).unwrap();

                    let index_terms = index_terms.into_iter().collect::<TermsMap<F>>();
                    Rc::new((constant_term, index_terms))
                }
            }
        };

        let constants = kimchi::circuits::expr::Constants::<F> {
            alpha: minimal.alpha,
            beta: minimal.beta,
            gamma: minimal.gamma,
            joint_combiner: minimal.lookup,
            endo_coefficient: {
                let (base, _) = endos::<F>();
                base
            },
            mds: &<F::OtherCurve>::sponge_params().mds,
            zk_rows: 3,
        };

        let mut ctx = EvalContext {
            evals,
            constants: &constants,
            cache: BTreeMap::new(),
            env,
            w,
        };

        let term = match gate {
            Some(gate) => index_terms.get(&Column::Index(gate)).unwrap(),
            None => constant_term,
        };

        // We evaluate the cached expressions first
        let mut cached_exprs = Cached::default();
        extract_caches(term, &mut cached_exprs);
        eval_cache(&cached_exprs, &mut ctx);

        // Eval the rest
        eval(term, &mut ctx)
    }
}

// TODO: De-duplicate with `ft_eval0`
pub fn ft_eval0_checked<F: FieldWitness, const NLIMB: usize>(
    env: &ScalarsEnv<F>,
    evals: &ProofEvaluations<PointEvaluations<F>>,
    minimal: &PlonkMinimal<F, NLIMB>,
    lookup: Option<F>,
    p_eval0: &[F],
    w: &mut Witness<F>,
) -> F {
    const PLONK_TYPES_PERMUTS_MINUS_1_N: usize = 6;

    let e0_s: Vec<_> = evals.s.iter().map(|s| s.fst()).collect();
    let zkp = env.zk_polynomial;
    let powers_of_alpha = powers_of_alpha(minimal.alpha);
    let alpha_pow = |i: usize| powers_of_alpha[i];

    let zeta1m1 = env.zeta_to_n_minus_1;
    let p_eval0 = p_eval0
        .iter()
        .copied()
        .rfold(None, |acc, p_eval0| match acc {
            None => Some(p_eval0),
            Some(acc) => {
                let zeta1 = *env.zeta_to_srs_length.get(w);
                Some(p_eval0 + field::mul(zeta1, acc, w))
            }
        })
        .unwrap(); // Never fail, `p_eval0` is non-empty
    let w0: Vec<_> = evals.w.iter().map(|w| w.fst()).collect();

    let ft_eval0 = {
        let a0 = alpha_pow(PERM_ALPHA0);
        let w_n = w0[PLONK_TYPES_PERMUTS_MINUS_1_N];
        let init = field::muls(&[(w_n + minimal.gamma), evals.z.snd(), a0, zkp], w);
        e0_s.iter().enumerate().fold(init, |acc, (i, s)| {
            // We decompose this way because of OCaml evaluation order
            let beta_s = field::mul(minimal.beta, *s, w);
            field::mul(beta_s + w0[i] + minimal.gamma, acc, w)
        })
    };

    let shifts = env.domain.shifts();
    let ft_eval0 = ft_eval0 - p_eval0;

    let ft_eval0 = ft_eval0
        - shifts.iter().enumerate().fold(
            field::muls(&[alpha_pow(PERM_ALPHA0), zkp, evals.z.fst()], w),
            |acc, (i, s)| {
                let beta_zeta = field::mul(minimal.beta, minimal.zeta, w);
                field::mul(acc, minimal.gamma + (beta_zeta * s) + w0[i], w)
            },
        );

    // We decompose this way because of OCaml evaluation order
    let a = field::muls(
        &[
            zeta1m1,
            alpha_pow(PERM_ALPHA0 + 2),
            (minimal.zeta - F::one()),
        ],
        w,
    );
    let b = field::muls(
        &[
            zeta1m1,
            alpha_pow(PERM_ALPHA0 + 1),
            (minimal.zeta - env.omega_to_minus_zk_rows),
        ],
        w,
    );
    let nominator = field::mul(a + b, F::one() - evals.z.fst(), w);

    let denominator = field::mul(
        minimal.zeta - env.omega_to_minus_zk_rows,
        minimal.zeta - F::one(),
        w,
    );
    let ft_eval0 = ft_eval0 + field::div_by_inv(nominator, denominator, w);

    let minimal = MinimalForScalar {
        alpha: minimal.alpha,
        beta: minimal.beta,
        gamma: minimal.gamma,
        lookup,
    };
    let constant_term = scalars::compute(None, &minimal, evals, env, w);
    ft_eval0 - constant_term
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::FpExt;

    use super::*;

    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    // #[test]
    // fn test_derive_plonk() {
    //     let f = |s| Fp::from_str(s).unwrap();

    //     let shift = Shift::<Fp>::create();

    //     assert_eq!(
    //         shift.scale.to_decimal(),
    //         "14474011154664524427946373126085988481681528240970780357977338382174983815169"
    //     );
    //     assert_eq!(
    //         shift.c.to_decimal(),
    //         "28948022309329048855892746252171976963271935850878721303774115239606597189632"
    //     );

    //     let env = ScalarsEnv {
    //         zk_polynomial: f(
    //             "14952139847623627632005961777062011953737808795126043460542801649311194043823",
    //         ),
    //         zeta_to_n_minus_1: f(
    //             "19992360331803571005450582443613456929944945612236598221548387723463412653399",
    //         ),
    //         srs_length_log2: 16,
    //     };
    //     let evals = ProofEvaluations {
    //         w: [
    //             [
    //                 f("6289128557598946688693552667439393426405656688717900311656646754749459718720"),
    //                 f("24608814230259595932281883624655184390098983971078865819273833140490521621830")
    //             ],
    //             [
    //                 f("13310290304507948299374922965790520471341407062194282657371827929238607707213"),
    //                 f("3420090439313334404412034509907098823576869881992393349685606692932938435891")
    //             ],
    //             [
    //                 f("1004170231282419143892553232264207846476468710370262861712645500613730936589"),
    //                 f("15358892959463725686065496984540565674655335654104110656651674381972851539319")
    //             ],
    //             [
    //                 f("14235022520036276432047767790226929227376034059765238012530447329202034471184"),
    //                 f("17857867215014615248343791853134492935501050027762182094448303054841389536812")
    //             ],
    //             [
    //                 f("217270972334916519789201407484219134254562938054977741353993356478916733888"),
    //                 f("23568343258930877322113518854070923552760559626728211948238368157503758958395")
    //             ],
    //             [
    //                 f("20724860985472235937562434615223247979133698324833013342416503003953673114269"),
    //                 f("14230270274902068449746409862542021948616341843532980943017138764071240362770")
    //             ],
    //             [
    //                 f("12691818116679147143874935777710847698261799259714916958466576430559681885637"),
    //                 f("22289834256183911112986280165603148282177788815718258800419577474647368509415")
    //             ],
    //             [
    //                 f("24411935077464468269852858502666516692114510902574937532215362000430706390980"),
    //                 f("27001187352494410500619676382138528783481518663974608697374943352528963038629")
    //             ],
    //             [
    //                 f("6360373480154519328489111512851611048061657527472483396338572585132690211046"),
    //                 f("23582754696949565264224763776269416039258289346076169583123362754778051868081")
    //             ],
    //             [
    //                 f("24503282241957787061015546660977633360122189424659418210184685296100040089763"),
    //                 f("1245945804906356625120959596915782469823165927957553537115988479791001371335")
    //             ],
    //             [
    //                 f("9672494201562279236249240210670811621086234303424478665245302624553285994017"),
    //                 f("6320456619637925667340696511492775125069306866183434852057714045546561260366")
    //             ],
    //             [
    //                 f("5210254176721326039284483791950162878174035416676107991148934478856221337173"),
    //                 f("815800705957329676302064392632236617950232059455550504081221846882909342121")
    //             ],
    //             [
    //                 f("19271726941980627641895844001132986442331047173347430774843986312900315111686"),
    //                 f("20110056132616893796657499795094959354081479634362692363043782275259311243305")
    //             ],
    //             [
    //                 f("146495953729640570485828778213514297261651081782921764417219235177307212109"),
    //                 f("13510051022748561152196281174320974475938901566632935888293972337640213044221")
    //             ],
    //             [
    //                 f("3518198406204532554527858244346702174770464435733756876385351085395310209176"),
    //                 f("15550522660048386236921180860694193742171605550559912660860741398328060104279")
    //             ],
    //         ],
    //         z: [
    //             f("21931198523501459183153033195666929989603803804227249746439702940933841410354"),
    //             f("6022590485205496790548288878896488149850531459551034305056017859745758834601")
    //         ],
    //         s: [
    //             [
    //                 f("19085168690544843887043640753077042378743448700110307869084708762711870533936"),
    //                 f("22751786545661378548233843701817138109179015540519447359893734727224759557557")
    //             ],
    //             [
    //                 f("12809849114708285788093252766158738590831386282299128882364744994146092851397"),
    //                 f("7775219585837024399019590773584893518283271570804407455680225353290518221037")
    //             ],
    //             [
    //                 f("1321451461746223492813831865923699791759758119520679872448309408329685991365"),
    //                 f("24250442489736828467922579478325926748516761175781497417627070248911524273876")
    //             ],
    //             [
    //                 f("14126289132628522291939529284539290777701180720493062389536649540269039839059"),
    //                 f("9881670171615426333925133765399382666579015176251728331663175340398678714235")
    //             ],
    //             [
    //                 f("5478696824111960874152427151223870065214252944402491332456328657031528756061"),
    //                 f("1377997571297099342686120389926615964800209763625383648915112029912281716920")
    //             ],
    //             [
    //                 f("80056797119034209825231055487115732036243341031547669733590224790110901245"),
    //                 f("23954132758389233312829853625715975217840278999681825436710260294777706132878")
    //             ],
    //         ],
    //         generic_selector: [
    //             f("21284673669227882794919992407172292293537574509733523059283928016664652610788"),
    //             f("21551467950274947783566469754812261989391861302105687694024368348901050138933")
    //         ],
    //         poseidon_selector: [
    //             f("24296813547347387275700638062514083772319473069038997762814161750820112396767"),
    //             f("7410307958356630828270235926030893251108745125559587079838321820967330039556")
    //         ],
    //         lookup: None,
    //     };

    //     let minimal = PlonkMinimal {
    //         alpha: f(
    //             "27274897876953793245985525242013286410205575357216365244783619058623821516088",
    //         ),
    //         beta: f("325777784653629264882054754458727445360"),
    //         gamma: f("279124633756639571190538225494418257063"),
    //         zeta: f(
    //             "23925312945193845374476523404515906217333838946294378278132006013444902630130",
    //         ),
    //         joint_combiner: None,
    //         alpha_bytes: [0; 2], // unused here
    //         beta_bytes: [0; 2],  // unused here
    //         gamma_bytes: [0; 2], // unused here
    //         zeta_bytes: [0; 2],  // unused here
    //     };

    //     let plonk = derive_plonk(&env, &evals, &minimal);

    //     let InCircuit {
    //         alpha,
    //         beta,
    //         gamma,
    //         zeta,
    //         zeta_to_domain_size,
    //         zeta_to_srs_length,
    //         poseidon_selector,
    //         vbmul,
    //         complete_add,
    //         endomul,
    //         endomul_scalar,
    //         perm,
    //         generic,
    //     } = plonk;

    //     // OCAML RESULTS
    //     assert_eq!(
    //         alpha.to_decimal(),
    //         "27274897876953793245985525242013286410205575357216365244783619058623821516088"
    //     );
    //     assert_eq!(beta.to_decimal(), "325777784653629264882054754458727445360");
    //     assert_eq!(
    //         gamma.to_decimal(),
    //         "279124633756639571190538225494418257063"
    //     );
    //     assert_eq!(
    //         zeta.to_decimal(),
    //         "23925312945193845374476523404515906217333838946294378278132006013444902630130"
    //     );
    //     assert_eq!(
    //         zeta_to_srs_length.shifted.to_decimal(),
    //         "24470191320566309930671664347892716946699561362620499174841813006278375362221"
    //     );
    //     assert_eq!(
    //         zeta_to_domain_size.shifted.to_decimal(),
    //         "24470191320566309930671664347892716946699561362620499174841813006278375362221"
    //     );
    //     assert_eq!(
    //         poseidon_selector.shifted.to_decimal(),
    //         "12148406773673693637850319031257041886205296850050918587497361637781741418736"
    //     );
    //     assert_eq!(
    //         perm.shifted.to_decimal(),
    //         "23996362553795482752052846938731572092036494641475074785875316421561912520951"
    //     );

    //     assert_eq!(
    //         complete_add.shifted.to_decimal(),
    //         "25922772146036107832349768103654536495710983785678446578187694836830607595414",
    //     );
    //     assert_eq!(
    //         vbmul.shifted.to_decimal(),
    //         "28215784507806213626095422113975086465418154941914519667351031525311316574704",
    //     );
    //     assert_eq!(
    //         endomul.shifted.to_decimal(),
    //         "13064695703283280169401378869933492390852015768221327952370116298712909203140",
    //     );
    //     assert_eq!(
    //         endomul_scalar.shifted.to_decimal(),
    //         "28322098634896565337442184680409326841751743190638136318667225694677236113253",
    //     );

    //     let generic_str = generic.map(|f| f.shifted.to_decimal());
    //     assert_eq!(
    //         generic_str,
    //         [
    //             "25116347989278465825406369329672134628495875811368961593709583152878995340915",
    //             "17618575433463997772293149459805685194929916900861150219895942521921398894881",
    //             "6655145152253974149687461482895260235716263846628561034776194726990989073959",
    //             "502085115641209571946276616132103923283794670716551136946603512678550688647",
    //             "7575395148088980464707243648576550930395639510870089332970629398518203372685",
    //             "21591522414682662643970257021199453095415105586384819070332842809147686271113",
    //             "14582646640831982687840973829828098048854370025529688934744615822786127402465",
    //             "10362430492736117968781217307611623989612409477947926377298532264348521777487",
    //             "12100020790372419869711605834757334981877832164856370998628117306965052030192",
    //         ]
    //     );
    // }

    #[test]
    fn test_alphas() {
        let n = Fp::from_str(
            "27274897876953793245985525242013286410205575357216365244783619058623821516088",
        )
        .unwrap();

        let alphas: Box<[Fp; NPOWERS_OF_ALPHA]> = powers_of_alpha(n);
        let alphas_str: Vec<String> = alphas.iter().map(|f| f.to_decimal()).collect();

        const OCAML_RESULT: &[&str] = &[
            "1",
            "27274897876953793245985525242013286410205575357216365244783619058623821516088",
            "5856243499679297994261942705106783326584825647279332525318074626467168425175",
            "26908526253468636093650206549302380737071523922183255477383956748755441012366",
            "21276200075660690362913766498168565850417909161152737384646582509540496229450",
            "3843731251681147173193384676587074004662025496739119332721571982426684387560",
            "12392606098341916760701161625583524765199435768082801118718099569066567820086",
            "5932489972119399045562481763112253944218195162891420406370178296693572483896",
            "1375846522483390900802414356841133463956126287864007136159978297384640659584",
            "5356524575738460513076981906288272723856440519543693179836517878630162813220",
            "23319398249603527452857836680743813857193813763032214190196631633251915644825",
            "10921184148344839491052929288136821627436657352065581423854521247501001908351",
            "13053560967285308651226207033123539702290413328361716005386653453569329750313",
            "8298101552564684053050013414211292674866114224797784754887740268228151928335",
            "715072795965317694491886715913315968459520650830405802156784401283709943505",
            "25198551493059869063561311792478884528738012039746184861146867788131566740666",
            "27161703551928606962685117055547438689494792119791879693135179256422752270728",
            "28799358614011589987311924793640447939591189984280570017428244220659375622447",
            "4488279652568453906961591843014473441709515392753701104095475657832824041646",
            "4641946865609115816676535679719511429699894348223929677606063307711524129548",
            "995093492640264169583875280706844374785298168266651011740457078469635678163",
            "17429526728376789811772110265115435172515921536052154380599101096979177652072",
            "22850194394147425267881995863659224768162140900074856912248813188202263996579",
            "26770317988143355422138083990683491489315652370177339626307684951664128480053",
        ];

        assert_eq!(alphas_str, OCAML_RESULT);
    }
}
