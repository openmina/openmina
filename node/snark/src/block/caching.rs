use std::{collections::HashMap, sync::Arc};

use ark_ec::{short_weierstrass_jacobian::GroupAffine, AffineCurve, ModelParameters};
use ark_poly::{univariate::DensePolynomial, Radix2EvaluationDomain};
use commitment_dlog::{commitment::CommitmentCurve, srs::SRS, PolyComm};
use kimchi::{
    alphas::Alphas,
    circuits::{
        argument::{Argument, ArgumentType},
        expr::{Linearization, PolishToken},
        gate::GateType,
        polynomials::{permutation, range_check, varbasemul::VarbaseMul},
        wires::{COLUMNS, PERMUTS},
    },
    mina_curves::pasta::Pallas,
    verifier_index::LookupVerifierIndex,
};
use mina_curves::pasta::Fq;
use mina_p2p_messages::bigint::BigInt;
use num_bigint::BigUint;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

use crate::VerifierIndex;

fn into<'a, U, T>(slice: &'a [U]) -> Vec<T>
where
    T: From<&'a U>,
{
    slice.iter().map(T::from).collect()
}

// Make it works with other containers, and non-From types
fn into_with<U, T, F, C, R>(container: C, fun: F) -> R
where
    F: Fn(U) -> T,
    C: IntoIterator<Item = U>,
    R: std::iter::FromIterator<T>,
{
    container.into_iter().map(fun).collect()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Radix2EvaluationDomainCached {
    size: u64,
    log_size_of_group: u32,
    size_as_field_element: BigInt,
    size_inv: BigInt,
    group_gen: BigInt,
    group_gen_inv: BigInt,
    generator_inv: BigInt,
}

impl From<&Radix2EvaluationDomainCached> for Radix2EvaluationDomain<Fq> {
    fn from(domain: &Radix2EvaluationDomainCached) -> Self {
        Self {
            size: domain.size,
            log_size_of_group: domain.log_size_of_group,
            size_as_field_element: domain.size_as_field_element.to_field(),
            size_inv: domain.size_inv.to_field(),
            group_gen: domain.group_gen.to_field(),
            group_gen_inv: domain.group_gen_inv.to_field(),
            generator_inv: domain.generator_inv.to_field(),
        }
    }
}

impl From<&Radix2EvaluationDomain<Fq>> for Radix2EvaluationDomainCached {
    fn from(domain: &Radix2EvaluationDomain<Fq>) -> Self {
        Self {
            size: domain.size,
            log_size_of_group: domain.log_size_of_group,
            size_as_field_element: domain.size_as_field_element.into(),
            size_inv: domain.size_inv.into(),
            group_gen: domain.group_gen.into(),
            group_gen_inv: domain.group_gen_inv.into(),
            generator_inv: domain.generator_inv.into(),
        }
    }
}

// Note: This should be an enum but bincode encode the discriminant in 8 bytes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupAffineCached {
    x: BigInt,
    y: BigInt,
    infinity: bool,
}

impl<'a, T> From<&'a GroupAffine<T>> for GroupAffineCached
where
    T: ark_ec::SWModelParameters,
    BigInt: From<&'a <T as ModelParameters>::BaseField>,
{
    fn from(pallas: &'a GroupAffine<T>) -> Self {
        Self {
            x: (&pallas.x).into(),
            y: (&pallas.y).into(),
            infinity: pallas.infinity,
        }
    }
}

impl<T> From<&GroupAffineCached> for GroupAffine<T>
where
    T: ark_ec::SWModelParameters,
{
    fn from(pallas: &GroupAffineCached) -> Self {
        Self::new(pallas.x.to_field(), pallas.y.to_field(), pallas.infinity)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SRSCached {
    g: Vec<GroupAffineCached>,
    h: GroupAffineCached,
    lagrange_bases: HashMap<usize, Vec<GroupAffineCached>>,
    endo_r: BigInt,
    endo_q: BigInt,
}

impl<'a, G> From<&'a SRS<G>> for SRSCached
where
    G: CommitmentCurve,
    GroupAffineCached: From<&'a G>,
    BigInt: From<&'a <G as AffineCurve>::ScalarField>,
    BigInt: From<&'a <G as AffineCurve>::BaseField>,
{
    fn from(srs: &'a SRS<G>) -> Self {
        Self {
            g: into(&srs.g),
            h: (&srs.h).into(),
            lagrange_bases: into_with(&srs.lagrange_bases, |(key, value)| (*key, into(value))),
            endo_r: (&srs.endo_r).into(),
            endo_q: (&srs.endo_q).into(),
        }
    }
}

impl<'a, G> From<&'a SRSCached> for SRS<G>
where
    G: CommitmentCurve + From<&'a GroupAffineCached>,
{
    fn from(srs: &'a SRSCached) -> Self {
        Self {
            g: into(&srs.g),
            h: (&srs.h).into(),
            lagrange_bases: into_with(&srs.lagrange_bases, |(key, value)| (*key, into(value))),
            endo_r: srs.endo_r.to_field(),
            endo_q: srs.endo_q.to_field(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DensePolynomialCached {
    coeffs: Vec<BigInt>, // Fq
}

impl From<&DensePolynomialCached> for DensePolynomial<Fq> {
    fn from(value: &DensePolynomialCached) -> Self {
        Self {
            coeffs: into(&value.coeffs),
        }
    }
}

impl From<&DensePolynomial<Fq>> for DensePolynomialCached {
    fn from(value: &DensePolynomial<Fq>) -> Self {
        Self {
            coeffs: into(&value.coeffs),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VerifierIndexCached {
    domain: Radix2EvaluationDomainCached,
    max_poly_size: usize,
    max_quot_size: usize,
    srs: SRSCached,
    public: usize,
    prev_challenges: usize,
    sigma_comm: [PolyComm<Pallas>; PERMUTS],
    coefficients_comm: [PolyComm<Pallas>; COLUMNS],
    generic_comm: PolyComm<Pallas>,
    psm_comm: PolyComm<Pallas>,
    complete_add_comm: PolyComm<Pallas>,
    mul_comm: PolyComm<Pallas>,
    emul_comm: PolyComm<Pallas>,
    endomul_scalar_comm: PolyComm<Pallas>,
    chacha_comm: Option<[PolyComm<Pallas>; 4]>,
    range_check_comm: Option<[PolyComm<Pallas>; range_check::gadget::GATE_COUNT]>,
    foreign_field_modulus: Option<BigUint>,
    foreign_field_add_comm: Option<PolyComm<Pallas>>,
    xor_comm: Option<PolyComm<Pallas>>,
    shift: [BigInt; PERMUTS], // Fq
    zkpm: DensePolynomialCached,
    w: BigInt,    // Fq
    endo: BigInt, // Fq
    lookup_index: Option<LookupVerifierIndex<Pallas>>,
    linearization: Linearization<Vec<PolishToken<BigInt>>>, // Fq
}

fn conv_token<'a, T, U>(token: &'a PolishToken<T>) -> PolishToken<U>
where
    T: 'a,
    U: From<&'a T>,
{
    match token {
        PolishToken::Alpha => PolishToken::Alpha,
        PolishToken::Beta => PolishToken::Beta,
        PolishToken::Gamma => PolishToken::Gamma,
        PolishToken::JointCombiner => PolishToken::JointCombiner,
        PolishToken::EndoCoefficient => PolishToken::EndoCoefficient,
        PolishToken::Mds { row, col } => PolishToken::Mds {
            row: *row,
            col: *col,
        },
        PolishToken::ForeignFieldModulus(int) => PolishToken::ForeignFieldModulus(*int),
        PolishToken::Literal(f) => PolishToken::Literal(f.into()),
        PolishToken::Cell(var) => PolishToken::Cell(*var),
        PolishToken::Dup => PolishToken::Dup,
        PolishToken::Pow(int) => PolishToken::Pow(*int),
        PolishToken::Add => PolishToken::Add,
        PolishToken::Mul => PolishToken::Mul,
        PolishToken::Sub => PolishToken::Sub,
        PolishToken::VanishesOnLast4Rows => PolishToken::VanishesOnLast4Rows,
        PolishToken::UnnormalizedLagrangeBasis(int) => PolishToken::UnnormalizedLagrangeBasis(*int),
        PolishToken::Store => PolishToken::Store,
        PolishToken::Load(int) => PolishToken::Load(*int),
    }
}

fn conv_linearization<'a, T, U>(
    linearization: &'a Linearization<Vec<PolishToken<T>>>,
) -> Linearization<Vec<PolishToken<U>>>
where
    T: 'a,
    U: From<&'a T>,
{
    let constant_term = &linearization.constant_term;
    let index_terms = &linearization.index_terms;

    Linearization {
        constant_term: into_with(constant_term, conv_token),
        index_terms: into_with(index_terms, |(col, term)| {
            (*col, into_with(term, conv_token))
        }),
    }
}

impl From<&VerifierIndex> for VerifierIndexCached {
    fn from(v: &VerifierIndex) -> Self {
        Self {
            domain: (&v.domain).into(),
            max_poly_size: v.max_poly_size,
            max_quot_size: v.max_quot_size,
            srs: (&**v.srs.get().unwrap()).into(),
            public: v.public,
            prev_challenges: v.prev_challenges,
            sigma_comm: v.sigma_comm.clone(),
            coefficients_comm: v.coefficients_comm.clone(),
            generic_comm: v.generic_comm.clone(),
            psm_comm: v.psm_comm.clone(),
            complete_add_comm: v.complete_add_comm.clone(),
            mul_comm: v.mul_comm.clone(),
            emul_comm: v.emul_comm.clone(),
            endomul_scalar_comm: v.endomul_scalar_comm.clone(),
            chacha_comm: v.chacha_comm.clone(),
            range_check_comm: v.range_check_comm.clone(),
            foreign_field_modulus: v.foreign_field_modulus.clone(),
            foreign_field_add_comm: v.foreign_field_add_comm.clone(),
            xor_comm: v.xor_comm.clone(),
            shift: std::array::from_fn(|i| v.shift[i].into()),
            zkpm: v.zkpm.get().unwrap().into(),
            w: (*v.w.get().unwrap()).into(),
            endo: v.endo.into(),
            lookup_index: v.lookup_index.clone(),
            linearization: conv_linearization(&v.linearization),
        }
    }
}

impl From<&VerifierIndexCached> for VerifierIndex {
    fn from(v: &VerifierIndexCached) -> Self {
        Self {
            domain: (&v.domain).into(),
            max_poly_size: v.max_poly_size,
            max_quot_size: v.max_quot_size,
            srs: OnceCell::with_value(Arc::new((&v.srs).into())),
            public: v.public,
            prev_challenges: v.prev_challenges,
            sigma_comm: v.sigma_comm.clone(),
            coefficients_comm: v.coefficients_comm.clone(),
            generic_comm: v.generic_comm.clone(),
            psm_comm: v.psm_comm.clone(),
            complete_add_comm: v.complete_add_comm.clone(),
            mul_comm: v.mul_comm.clone(),
            emul_comm: v.emul_comm.clone(),
            endomul_scalar_comm: v.endomul_scalar_comm.clone(),
            chacha_comm: v.chacha_comm.clone(),
            range_check_comm: v.range_check_comm.clone(),
            foreign_field_modulus: v.foreign_field_modulus.clone(),
            foreign_field_add_comm: v.foreign_field_add_comm.clone(),
            xor_comm: v.xor_comm.clone(),
            shift: std::array::from_fn(|i| v.shift[i].to_field()),
            zkpm: OnceCell::with_value((&v.zkpm).into()),
            w: OnceCell::with_value(v.w.to_field()),
            endo: v.endo.to_field(),
            lookup_index: v.lookup_index.clone(),
            linearization: conv_linearization(&v.linearization),
            powers_of_alpha: {
                // `Alphas` contains private data, so we can't de/serialize it.
                // Initializing an `Alphas` is cheap anyway (for block verification).

                // Initialize it like here:
                // https://github.com/o1-labs/proof-systems/blob/a36c088b3e81d17f5720abfff82a49cf9cb1ad5b/kimchi/src/linearization.rs#L31
                let mut powers_of_alpha = Alphas::<Fq>::default();
                powers_of_alpha.register(
                    ArgumentType::Gate(GateType::Zero),
                    VarbaseMul::<Fq>::CONSTRAINTS,
                );
                powers_of_alpha.register(ArgumentType::Permutation, permutation::CONSTRAINTS);
                powers_of_alpha
            },
        }
    }
}

pub fn verifier_index_to_bytes(verifier: &VerifierIndex) -> Vec<u8> {
    const NBYTES: usize = 5328359;

    let verifier: VerifierIndexCached = verifier.into();
    let mut bytes = Vec::with_capacity(NBYTES);
    bincode::serialize_into(&mut bytes, &verifier).unwrap();

    bytes
}

pub fn verifier_index_from_bytes(bytes: &[u8]) -> VerifierIndex {
    let verifier: VerifierIndexCached = bincode::deserialize(bytes).unwrap();
    (&verifier).into()
}

pub fn srs_to_bytes<'a, G>(srs: &'a SRS<G>) -> Vec<u8>
where
    G: CommitmentCurve,
    GroupAffineCached: From<&'a G>,
    BigInt: From<&'a <G as AffineCurve>::ScalarField>,
    BigInt: From<&'a <G as AffineCurve>::BaseField>,
{
    const NBYTES: usize = 5308593;

    let srs: SRSCached = srs.into();
    let mut bytes = Vec::with_capacity(NBYTES);
    bincode::serialize_into(&mut bytes, &srs).unwrap();

    bytes
}

pub fn srs_from_bytes<G>(bytes: &[u8]) -> SRS<G>
where
    G: CommitmentCurve,
    G: for<'a> From<&'a GroupAffineCached>,
{
    let srs: SRSCached = bincode::deserialize(bytes).unwrap();
    (&srs).into()
}
