use std::sync::Arc;

use mina_hasher::Fp;
use poly_commitment::srs::SRS;

use kimchi::{
    circuits::{
        constraints::FeatureFlags,
        expr::Linearization,
        lookup::lookups::{LookupFeatures, LookupPatterns},
        polynomials::permutation::{zk_polynomial, zk_w3},
    },
    curve::KimchiCurve,
    linearization::expr_linearization,
};
use mina_curves::pasta::Fq;

use once_cell::sync::OnceCell;

use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};

use crate::{
    proofs::{witness::GroupAffine, BACKEND_TOCK_ROUNDS_N},
    VerificationKey,
};

use super::{
    witness::endos,
    wrap::{Domain, Domains},
};
use super::{witness::InnerCurve, VerifierIndex};

const PERMUTS: usize = 7;
const COLUMNS: usize = 15;

pub enum VerifierKind {
    Blockchain,
    Transaction,
}

pub fn get_verifier_index(kind: VerifierKind) -> VerifierIndex {
    let make = |data: &str| {
        let verifier_index: kimchi::verifier_index::VerifierIndex<GroupAffine<Fp>> =
            serde_json::from_str(data).unwrap();
        make_verifier_index(verifier_index)
    };

    match kind {
        VerifierKind::Blockchain => {
            cache_one!(VerifierIndex, {
                make(include_str!("data/blockchain_verifier_index.json"))
            })
        }
        VerifierKind::Transaction => {
            cache_one!(VerifierIndex, {
                make(include_str!("data/transaction_verifier_index.json"))
            })
        }
    }
}

fn make_verifier_index(
    index: kimchi::verifier_index::VerifierIndex<GroupAffine<Fp>>,
) -> VerifierIndex {
    let domain = index.domain;
    let max_poly_size: usize = index.max_poly_size;
    let (endo, _) = endos::<Fq>();

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
        srs: OnceCell::from(srs),
        zkpm: OnceCell::from(zkpm),
        w: OnceCell::from(w),
        endo,
        linearization,
        powers_of_alpha,
        ..index
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

// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/pickles/common.ml#L27
pub fn wrap_domains(proofs_verified: usize) -> Domains {
    let h = match proofs_verified {
        0 => 13,
        1 => 14,
        2 => 15,
        _ => unreachable!(),
    };

    Domains {
        h: Domain::Pow2RootsOfUnity(h),
    }
}

/// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/pickles/side_loaded_verification_key.ml#L206
pub fn make_zkapp_verifier_index(vk: &VerificationKey) -> VerifierIndex {
    let d = wrap_domains(vk.actual_wrap_domain_size.to_int());
    let log2_size = d.h.log2_size();

    let public = 40; // Is that constant ?

    let domain: Radix2EvaluationDomain<Fq> =
        Radix2EvaluationDomain::new(1 << log2_size as u64).unwrap();

    let srs = {
        use mina_curves::pasta::Vesta;
        let degree = 1 << BACKEND_TOCK_ROUNDS_N;
        let mut srs = SRS::<<Vesta as KimchiCurve>::OtherCurve>::create(degree);
        srs.add_lagrange_basis(domain);
        srs
    };

    let make_poly = |poly: &InnerCurve<Fp>| poly_commitment::PolyComm {
        unshifted: vec![poly.to_affine()],
        shifted: None,
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

    let (endo_q, _endo_r) = endos::<Fq>();
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
