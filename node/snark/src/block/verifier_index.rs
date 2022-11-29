use std::{array, sync::Arc};

use ark_ec::short_weierstrass_jacobian::GroupAffine;
use ark_ec::AffineCurve;

use commitment_dlog::{commitment::CommitmentCurve, srs::SRS, PolyComm};

use kimchi::{
    curve::KimchiCurve, linearization::expr_linearization, verifier_index::LookupVerifierIndex,
};
use mina_curves::pasta::{Fq, Pallas, VestaParameters};

use once_cell::sync::OnceCell;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};

use super::VerifierIndex;
use crate::public_input::scalars::field_from_hex;

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
    chacha_comm: Option<[PolynomialCommitment; 4]>,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Index<G: KimchiCurve> {
    domain: DomainOcaml,
    max_poly_size: usize,
    max_quot_size: usize,
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

pub fn get_verifier_index() -> VerifierIndex {
    let verifier_index_str = include_str!("../data/verifier_index.json");

    let verifier_index_json: VerifierIndexOcaml<Pallas> =
        serde_json::from_str(verifier_index_str).unwrap();

    make_verifier_index(&verifier_index_json)
}

fn make_verifier_index(index: &VerifierIndexOcaml<Pallas>) -> VerifierIndex {
    let make_poly = |poly: &PolynomialCommitment| {
        let unshifted = &poly.unshifted[0][1];
        let FieldType::Pair(x, y) = &unshifted else { panic!() };

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
    let chacha_comm: Option<[PolyComm<Pallas>; 4]> = evals
        .chacha_comm
        .as_ref()
        .map(|chacha| array::from_fn(|i| make_poly(&chacha[i])));

    let domain: Radix2EvaluationDomain<Fq> =
        Radix2EvaluationDomain::new(index.data.constraints).unwrap();

    let max_poly_size: usize = index.index.max_poly_size;
    let max_quot_size: usize = index.index.max_quot_size;
    let prev_challenges: usize = index.index.prev_challenges;

    let shift = array::from_fn(|i| {
        let shift = index.index.shifts[i].as_str();
        field_from_hex(shift)
    });

    let (endo, _) = kimchi::commitment_dlog::srs::endos::<GroupAffine<VestaParameters>>();

    let (linearization, powers_of_alpha) = expr_linearization(false, false, None, false, false);

    let public: usize = index.index.public;

    VerifierIndex {
        domain,
        max_poly_size,
        max_quot_size,
        srs: Default::default(),
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
        chacha_comm,
        range_check_comm: None,
        foreign_field_modulus: None,
        foreign_field_add_comm: None,
        xor_comm: None,
        shift,
        zkpm: Default::default(),
        w: Default::default(),
        endo,
        lookup_index: None,
        linearization,
        powers_of_alpha,
    }
}
