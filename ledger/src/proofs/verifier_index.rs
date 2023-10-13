use std::{array, sync::Arc};

use ark_ec::short_weierstrass_jacobian::GroupAffine;
use ark_ec::AffineCurve;

use mina_hasher::Fp;
use poly_commitment::{commitment::CommitmentCurve, srs::SRS, PolyComm};

use kimchi::{
    circuits::{
        constraints::FeatureFlags,
        expr::Linearization,
        lookup::lookups::{LookupFeatures, LookupPatterns},
        polynomials::permutation::{zk_polynomial, zk_w3},
    },
    curve::KimchiCurve,
    linearization::expr_linearization,
    verifier_index::LookupVerifierIndex,
};
use mina_curves::pasta::{Fq, Pallas, VestaParameters};

use once_cell::sync::OnceCell;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};

use crate::{proofs::BACKEND_TOCK_ROUNDS_N, CurveAffine, VerificationKey};

use super::public_input::scalars::field_from_hex;
use super::VerifierIndex;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct VerifierIndexOcaml<G: CommitmentCurve + KimchiCurve + AffineCurve> {
    #[serde(bound = "PolyComm<G>: Serialize + DeserializeOwned")]
    index: Index<G>,
    data: DataOcaml,
}

const PERMUTS: usize = 7;
const COLUMNS: usize = 15;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Evals {
    sigma_comm: [PolynomialCommitment; PERMUTS],
    coefficients_comm: [PolynomialCommitment; COLUMNS],
    generic_comm: PolynomialCommitment,
    psm_comm: PolynomialCommitment,
    complete_add_comm: PolynomialCommitment,
    mul_comm: PolynomialCommitment,
    emul_comm: PolynomialCommitment,
    endomul_scalar_comm: PolynomialCommitment,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Index<G: KimchiCurve> {
    domain: DomainOcaml,
    max_poly_size: usize,
    public: usize,
    prev_challenges: usize,
    #[serde(skip)]
    srs: OnceCell<Arc<SRS<G>>>,
    shifts: [String; PERMUTS],
    #[serde(bound = "PolyComm<G>: Serialize + DeserializeOwned")]
    lookup_index: Option<LookupVerifierIndex<G>>,

    evals: Evals,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DomainOcaml {
    log_size_of_group: u32,
    group_gen: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DataOcaml {
    constraints: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
enum FieldType {
    Single(String),
    Pair(String, String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PolynomialCommitment {
    unshifted: Vec<Vec<FieldType>>,
    shifted: Option<FieldType>,
}

pub enum VerifierKind {
    Blockchain,
    Transaction,
}

pub fn get_verifier_index(kind: VerifierKind) -> VerifierIndex {
    let verifier_index_str = match kind {
        VerifierKind::Blockchain => include_str!("data/blockchain_verifier_index.json"),
        VerifierKind::Transaction => include_str!("data/transaction_verifier_index.json"),
    };

    let verifier_index_json: VerifierIndexOcaml<Pallas> =
        serde_json::from_str(verifier_index_str).unwrap();

    make_verifier_index(&verifier_index_json)
}

fn make_verifier_index(index: &VerifierIndexOcaml<Pallas>) -> VerifierIndex {
    let make_poly = |poly: &PolynomialCommitment| {
        let unshifted = &poly.unshifted[0][1];
        let FieldType::Pair(x, y) = &unshifted else {
            panic!()
        };

        PolyComm {
            unshifted: vec![Pallas::of_coordinates(field_from_hex(x), field_from_hex(y))],
            shifted: None,
        }
    };

    let evals: &Evals = &index.index.evals;

    let sigma_comm: [PolyComm<Pallas>; 7] = array::from_fn(|i| make_poly(&evals.sigma_comm[i]));
    let coefficients_comm: [PolyComm<Pallas>; 15] =
        array::from_fn(|i| make_poly(&evals.coefficients_comm[i]));

    let generic_comm: PolyComm<Pallas> = make_poly(&evals.generic_comm);
    let psm_comm: PolyComm<Pallas> = make_poly(&evals.psm_comm);
    let complete_add_comm: PolyComm<Pallas> = make_poly(&evals.complete_add_comm);
    let mul_comm: PolyComm<Pallas> = make_poly(&evals.mul_comm);
    let emul_comm: PolyComm<Pallas> = make_poly(&evals.emul_comm);
    let endomul_scalar_comm: PolyComm<Pallas> = make_poly(&evals.endomul_scalar_comm);

    let domain: Radix2EvaluationDomain<Fq> =
        Radix2EvaluationDomain::new(index.data.constraints).unwrap();

    let max_poly_size: usize = index.index.max_poly_size;
    let prev_challenges: usize = index.index.prev_challenges;

    let shift = array::from_fn(|i| {
        let shift = index.index.shifts[i].as_str();
        field_from_hex(shift)
    });

    let (endo, _) = kimchi::poly_commitment::srs::endos::<GroupAffine<VestaParameters>>();

    let feature_flags = FeatureFlags {
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
    };

    let (mut linearization, powers_of_alpha) = expr_linearization(Some(&feature_flags), true);

    let linearization = Linearization {
        constant_term: linearization.constant_term,
        index_terms: {
            // Make the verifier index deterministic
            linearization
                .index_terms
                .sort_by_key(|&(columns, _)| columns);
            linearization.index_terms
        },
    };

    let public: usize = index.index.public;

    // https://github.com/o1-labs/proof-systems/blob/2702b09063c7a48131173d78b6cf9408674fd67e/kimchi/src/verifier_index.rs#L310-L314
    let srs = {
        let mut srs = SRS::create(max_poly_size);
        srs.add_lagrange_basis(domain);
        Arc::new(srs)
    };

    // https://github.com/o1-labs/proof-systems/blob/2702b09063c7a48131173d78b6cf9408674fd67e/kimchi/src/verifier_index.rs#L319
    let zkpm = zk_polynomial(domain);

    // https://github.com/o1-labs/proof-systems/blob/2702b09063c7a48131173d78b6cf9408674fd67e/kimchi/src/verifier_index.rs#L324
    let w = zk_w3(domain);

    VerifierIndex {
        domain,
        max_poly_size,
        srs: OnceCell::from(srs),
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
        foreign_field_add_comm: None,
        xor_comm: None,
        shift,
        zkpm: OnceCell::from(zkpm),
        w: OnceCell::from(w),
        endo,
        lookup_index: None,
        linearization,
        powers_of_alpha,
        range_check0_comm: None,
        range_check1_comm: None,
        foreign_field_mul_comm: None,
        rot_comm: None,
    }
}

/// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/crypto/kimchi_bindings/stubs/src/pasta_fq_plonk_verifier_index.rs#L213
/// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/pickles/common.ml#L16C1-L25C58
pub fn make_shifts(
    domain: &Radix2EvaluationDomain<Fq>,
) -> kimchi::circuits::polynomials::permutation::Shifts<Fq> {
    // let value = 1 << log2_size;
    // let domain = Domain::<Fq>::new(value).unwrap();
    kimchi::circuits::polynomials::permutation::Shifts::new(domain)
}

/// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/pickles/side_loaded_verification_key.ml#L206
pub fn make_zkapp_verifier_index(vk: &VerificationKey) -> VerifierIndex {
    // https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/pickles/common.ml#L27
    let wrap_domains = |proofs_verified: usize| -> usize {
        match proofs_verified {
            0 => 13,
            1 => 14,
            2 => 15,
            _ => unreachable!(),
        }
    };

    let d = wrap_domains(vk.actual_wrap_domain_size.to_int());
    let log2_size = d;

    let public = 52; // Is that constant ?

    let domain: Radix2EvaluationDomain<Fq> =
        Radix2EvaluationDomain::new(1 << log2_size as u64).unwrap();

    let srs = {
        use mina_curves::pasta::Vesta;
        let degree = 1 << BACKEND_TOCK_ROUNDS_N;
        let mut srs = SRS::<<Vesta as KimchiCurve>::OtherCurve>::create(degree);
        srs.add_lagrange_basis(domain);
        srs
    };

    let make_poly = |poly: &CurveAffine<Fp>| {
        let CurveAffine(x, y) = poly;
        poly_commitment::PolyComm {
            unshifted: vec![Pallas::of_coordinates(*x, *y)],
            shifted: None,
        }
    };

    let feature_flags = FeatureFlags {
        range_check0: false,
        range_check1: false,
        foreign_field_add: false,
        foreign_field_mul: false,
        rot: false,
        xor: false,
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
    };

    let (endo_q, _endo_r) = poly_commitment::srs::endos::<mina_curves::pasta::Vesta>();
    let (linearization, powers_of_alpha) = expr_linearization(Some(&feature_flags), true);

    let shift = make_shifts(&domain);

    // Note: Verifier index is converted from OCaml here:
    // https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/crypto/kimchi_bindings/stubs/src/pasta_fq_plonk_verifier_index.rs#L58

    VerifierIndex {
        domain,
        max_poly_size: 1 << BACKEND_TOCK_ROUNDS_N,
        srs: once_cell::sync::OnceCell::with_value(Arc::new(srs)),
        public,
        prev_challenges: 2,
        sigma_comm: std::array::from_fn(|i| make_poly(&vk.wrap_index.sigma[i])),
        coefficients_comm: std::array::from_fn(|i| make_poly(&vk.wrap_index.coefficients[i])),
        generic_comm: make_poly(&vk.wrap_index.generic),
        psm_comm: make_poly(&vk.wrap_index.psm),
        complete_add_comm: make_poly(&vk.wrap_index.complete_add),
        mul_comm: make_poly(&vk.wrap_index.mul),
        emul_comm: make_poly(&vk.wrap_index.emul),
        endomul_scalar_comm: make_poly(&vk.wrap_index.endomul_scalar),
        range_check0_comm: None,
        range_check1_comm: None,
        foreign_field_add_comm: None,
        foreign_field_mul_comm: None,
        xor_comm: None,
        rot_comm: None,
        shift: *shift.shifts(),
        zkpm: { OnceCell::with_value(zk_polynomial(domain)) },
        w: { OnceCell::with_value(zk_w3(domain)) },
        endo: endo_q,
        lookup_index: None,
        linearization,
        powers_of_alpha,
    }
}
