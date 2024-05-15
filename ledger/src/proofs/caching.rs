use std::{collections::HashMap, sync::Arc};

use ark_ec::{short_weierstrass_jacobian::GroupAffine, AffineCurve, ModelParameters};
use ark_poly::{univariate::DensePolynomial, Radix2EvaluationDomain};
use kimchi::{
    alphas::Alphas,
    circuits::{
        argument::{Argument, ArgumentType},
        expr::{Linearization, PolishToken},
        gate::GateType,
        polynomials::{permutation, varbasemul::VarbaseMul},
        wires::{COLUMNS, PERMUTS},
    },
    mina_curves::pasta::Pallas,
    verifier_index::{LookupVerifierIndex, VerifierIndex},
};
use mina_curves::pasta::Fq;
use mina_p2p_messages::bigint::BigInt;
use once_cell::sync::OnceCell;
use poly_commitment::{commitment::CommitmentCurve, srs::SRS, PolyComm};
use serde::{Deserialize, Serialize};

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
struct PolyCommCached {
    unshifted: Vec<GroupAffineCached>,
    shifted: Option<GroupAffineCached>,
}

impl<'a, A> From<&'a PolyComm<A>> for PolyCommCached
where
    GroupAffineCached: From<&'a A>,
{
    fn from(value: &'a PolyComm<A>) -> Self {
        let PolyComm { unshifted, shifted } = value;

        Self {
            unshifted: into(unshifted),
            shifted: shifted.as_ref().map(Into::into),
        }
    }
}

impl<'a, A> From<&'a PolyCommCached> for PolyComm<A>
where
    A: From<&'a GroupAffineCached>,
{
    fn from(value: &'a PolyCommCached) -> Self {
        let PolyCommCached { unshifted, shifted } = value;

        Self {
            unshifted: into(unshifted),
            shifted: shifted.as_ref().map(Into::into),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SRSCached {
    g: Vec<GroupAffineCached>,
    h: GroupAffineCached,
    lagrange_bases: HashMap<usize, Vec<PolyCommCached>>,
}

impl<'a, G> From<&'a SRS<G>> for SRSCached
where
    G: CommitmentCurve,
    GroupAffineCached: From<&'a G>,
    PolyCommCached: From<&'a PolyComm<G>>,
    BigInt: From<&'a <G as AffineCurve>::ScalarField>,
    BigInt: From<&'a <G as AffineCurve>::BaseField>,
{
    fn from(srs: &'a SRS<G>) -> Self {
        Self {
            g: into(&srs.g),
            h: (&srs.h).into(),
            lagrange_bases: into_with(&srs.lagrange_bases, |(key, value)| (*key, into(value))),
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
    range_check0_comm: Option<PolyComm<Pallas>>,
    range_check1_comm: Option<PolyComm<Pallas>>,
    foreign_field_add_comm: Option<PolyComm<Pallas>>,
    foreign_field_mul_comm: Option<PolyComm<Pallas>>,
    xor_comm: Option<PolyComm<Pallas>>,
    rot_comm: Option<PolyComm<Pallas>>,
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
        PolishToken::SkipIf(flags, int) => PolishToken::SkipIf(*flags, *int),
        PolishToken::SkipIfNot(flags, int) => PolishToken::SkipIfNot(*flags, *int),
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

impl From<&VerifierIndex<Pallas>> for VerifierIndexCached {
    fn from(v: &VerifierIndex<Pallas>) -> Self {
        let VerifierIndex {
            domain,
            max_poly_size,
            srs,
            public,
            prev_challenges,
            sigma_comm,
            coefficients_comm,
            generic_comm,
            psm_comm,
            complete_add_comm,
            mul_comm,
            emul_comm,
            endomul_scalar_comm,
            range_check0_comm,
            range_check1_comm,
            foreign_field_add_comm,
            foreign_field_mul_comm,
            xor_comm,
            rot_comm,
            shift,
            zkpm,
            w,
            endo,
            lookup_index,
            linearization,
            powers_of_alpha: _, // ignored
        } = v;

        Self {
            domain: domain.into(),
            max_poly_size: *max_poly_size,
            srs: (&**srs.get().unwrap()).into(),
            public: *public,
            prev_challenges: *prev_challenges,
            sigma_comm: sigma_comm.clone(),
            coefficients_comm: coefficients_comm.clone(),
            generic_comm: generic_comm.clone(),
            psm_comm: psm_comm.clone(),
            complete_add_comm: complete_add_comm.clone(),
            mul_comm: mul_comm.clone(),
            emul_comm: emul_comm.clone(),
            endomul_scalar_comm: endomul_scalar_comm.clone(),
            range_check0_comm: range_check0_comm.clone(),
            range_check1_comm: range_check1_comm.clone(),
            foreign_field_add_comm: foreign_field_add_comm.clone(),
            foreign_field_mul_comm: foreign_field_mul_comm.clone(),
            xor_comm: xor_comm.clone(),
            rot_comm: rot_comm.clone(),
            shift: shift.each_ref().map(|s| s.into()),
            zkpm: zkpm.get().unwrap().into(),
            w: (*w.get().unwrap()).into(),
            endo: endo.into(),
            lookup_index: lookup_index.clone(),
            linearization: conv_linearization(&linearization),
        }
    }
}

impl From<&VerifierIndexCached> for VerifierIndex<Pallas> {
    fn from(v: &VerifierIndexCached) -> Self {
        let VerifierIndexCached {
            domain,
            max_poly_size,
            srs,
            public,
            prev_challenges,
            sigma_comm,
            coefficients_comm,
            generic_comm,
            psm_comm,
            complete_add_comm,
            mul_comm,
            emul_comm,
            endomul_scalar_comm,
            range_check0_comm,
            range_check1_comm,
            foreign_field_add_comm,
            foreign_field_mul_comm,
            xor_comm,
            rot_comm,
            shift,
            zkpm,
            w,
            endo,
            lookup_index,
            linearization,
        } = v;

        Self {
            domain: domain.into(),
            max_poly_size: *max_poly_size,
            srs: OnceCell::with_value(Arc::new(srs.into())),
            public: *public,
            prev_challenges: *prev_challenges,
            sigma_comm: sigma_comm.clone(),
            coefficients_comm: coefficients_comm.clone(),
            generic_comm: generic_comm.clone(),
            psm_comm: psm_comm.clone(),
            complete_add_comm: complete_add_comm.clone(),
            mul_comm: mul_comm.clone(),
            emul_comm: emul_comm.clone(),
            endomul_scalar_comm: endomul_scalar_comm.clone(),
            foreign_field_add_comm: foreign_field_add_comm.clone(),
            xor_comm: xor_comm.clone(),
            shift: shift.each_ref().map(|s| s.to_field()),
            zkpm: OnceCell::with_value(zkpm.into()),
            w: OnceCell::with_value(w.to_field()),
            endo: endo.to_field(),
            lookup_index: lookup_index.clone(),
            linearization: conv_linearization(&linearization),
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
            range_check0_comm: range_check0_comm.clone(),
            range_check1_comm: range_check1_comm.clone(),
            foreign_field_mul_comm: foreign_field_mul_comm.clone(),
            rot_comm: rot_comm.clone(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Error writing verifier index to bytes: {0}")]
pub struct VerifierIndexToBytesError(#[from] serde_cbor::Error);

pub fn verifier_index_to_bytes(
    verifier: &VerifierIndex<Pallas>,
) -> Result<Vec<u8>, VerifierIndexToBytesError> {
    let verifier: VerifierIndexCached = verifier.into();
    Ok(serde_cbor::to_vec(&verifier)?)
}

#[derive(Debug, thiserror::Error)]
#[error("Error reading verifier index from bytes: {0}")]
pub struct VerifierIndexFromBytesError(#[from] serde_cbor::Error);

pub fn verifier_index_from_bytes(
    bytes: &[u8],
) -> Result<VerifierIndex<Pallas>, VerifierIndexFromBytesError> {
    let verifier: VerifierIndexCached = serde_cbor::from_slice(bytes)?;
    Ok((&verifier).into())
}

pub fn srs_to_bytes<'a, G>(srs: &'a SRS<G>) -> Vec<u8>
where
    G: CommitmentCurve,
    GroupAffineCached: From<&'a G>,
    BigInt: From<&'a <G as AffineCurve>::ScalarField>,
    BigInt: From<&'a <G as AffineCurve>::BaseField>,
{
    let srs: SRSCached = srs.into();

    serde_cbor::to_vec(&srs).unwrap()
}

pub fn srs_from_bytes<G>(bytes: &[u8]) -> SRS<G>
where
    G: CommitmentCurve,
    G: for<'a> From<&'a GroupAffineCached>,
{
    let srs: SRSCached = serde_cbor::from_slice(bytes).unwrap();
    (&srs).into()
}
