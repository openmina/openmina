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
    mina_curves::pasta::Pallas,
    verifier_index::VerifierIndex,
};
use mina_curves::pasta::Fq;

use once_cell::sync::OnceCell;

use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};

use crate::{
    proofs::{field::GroupAffine, BACKEND_TOCK_ROUNDS_N},
    VerificationKey,
};

use super::{
    caching::{verifier_index_from_bytes, verifier_index_to_bytes},
    transaction::InnerCurve,
};
use super::{
    transaction::endos,
    wrap::{Domain, Domains},
};

pub enum VerifierKind {
    Blockchain,
    Transaction,
}

use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::Context;
use openmina_core::{info, log::system_time, warn};

fn openmina_cache_path<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache/openmina").join(path))
}

fn read_index(path: &Path, digest: &[u8]) -> anyhow::Result<VerifierIndex<Pallas>> {
    let mut buf = Vec::with_capacity(5700000);
    let mut file = File::open(path).context("opening cache file")?;
    let mut d = [0; 32];
    // source digest
    file.read_exact(&mut d).context("reading source digest")?;
    if &d != digest {
        anyhow::bail!("source digest verification failed");
    }

    // index digest
    file.read_exact(&mut d).context("reading index digest")?;
    // index
    file.read_to_end(&mut buf)
        .context("reading verifier index from cache file")?;

    let mut hasher = Sha256::new();
    hasher.update(&buf);
    let digest = hasher.finalize();
    if &d != digest.as_slice() {
        anyhow::bail!("verifier index digest verification failed");
    }
    Ok(verifier_index_from_bytes(&buf))
}

fn write_index(path: &Path, index: &VerifierIndex<Pallas>, digest: &[u8]) -> anyhow::Result<()> {
    let bytes = verifier_index_to_bytes(index);
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let Some(parent) = path.parent() else {
        anyhow::bail!("cannot get parent for {path:?}");
    };
    std::fs::create_dir_all(parent).context("creating cache file parent directory")?;
    let mut file = File::create(path).context("creating cache file")?;
    file.write_all(digest).context("storing source digest")?;
    file.write_all(&hasher.finalize())
        .context("storing verifier index digest")?;
    file.write_all(&bytes)
        .context("storing verifier index into cache file")?;
    Ok(())
}

#[cfg(target_family = "wasm")]
fn make_with_ext_cache(data: &str, cache: &str) -> VerifierIndex<Pallas> {
    let verifier_index: kimchi::verifier_index::VerifierIndex<GroupAffine<Fp>> =
        serde_json::from_str(data).unwrap();
    make_verifier_index(verifier_index)
}

#[cfg(not(target_family = "wasm"))]
fn make_with_ext_cache(data: &str, cache: &str) -> VerifierIndex<Pallas> {
    let verifier_index: kimchi::verifier_index::VerifierIndex<GroupAffine<Fp>> =
        serde_json::from_str(data).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(data);
    let src_index_digest = hasher.finalize();

    if let Some(path) = openmina_cache_path(cache) {
        match read_index(&path, &src_index_digest) {
            Ok(verifier_index) => {
                info!(system_time(); "Block verifier index is loaded from {path:?}");
                verifier_index
            }
            Err(err) => {
                warn!(system_time(); "Cannot load verifier index from cache file {path:?}: {err}");
                let index = make_verifier_index(verifier_index);
                if let Err(err) = write_index(&path, &index, &src_index_digest) {
                    warn!(system_time(); "Cannot store verifier index to cache file {path:?}: {err}");
                } else {
                    info!(system_time(); "Stored block verifier index to cache file {path:?}");
                }
                index
            }
        }
    } else {
        warn!(system_time(); "Cannot determine cache path for verifier index");
        make_verifier_index(verifier_index)
    }
}

pub fn get_verifier_index(kind: VerifierKind) -> VerifierIndex<Pallas> {
    match kind {
        VerifierKind::Blockchain => {
            cache_one!(VerifierIndex<Pallas>, {
                make_with_ext_cache(
                    include_str!("data/blockchain_verifier_index.json"),
                    "block_verifier_index.bin",
                )
            })
        }
        VerifierKind::Transaction => {
            cache_one!(VerifierIndex<Pallas>, {
                make_with_ext_cache(
                    include_str!("data/transaction_verifier_index.json"),
                    "transaction_verifier_index.bin",
                )
            })
        }
    }
}

fn make_verifier_index(
    index: kimchi::verifier_index::VerifierIndex<GroupAffine<Fp>>,
) -> VerifierIndex<Pallas> {
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
pub fn make_zkapp_verifier_index(vk: &VerificationKey) -> VerifierIndex<Pallas> {
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
        sigma_comm: vk.wrap_index.sigma.each_ref().map(make_poly),
        coefficients_comm: vk.wrap_index.coefficients.each_ref().map(make_poly),
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
