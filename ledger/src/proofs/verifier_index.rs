use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    sync::Arc,
};

use anyhow::Context;
use once_cell::sync::OnceCell;
#[cfg(not(target_family = "wasm"))]
use openmina_core::{info, log::system_time, warn};
use sha2::{Digest, Sha256};

use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};
use kimchi::{
    circuits::{
        constraints::FeatureFlags,
        expr::Linearization,
        lookup::lookups::{LookupFeatures, LookupPatterns},
        polynomials::permutation::{permutation_vanishing_polynomial, zk_w},
    },
    linearization::expr_linearization,
    mina_curves::pasta::Pallas,
};
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use poly_commitment::srs::SRS;

use crate::{proofs::BACKEND_TOCK_ROUNDS_N, VerificationKey};

use super::{
    caching::{verifier_index_from_bytes, verifier_index_to_bytes},
    transaction::InnerCurve,
    VerifierIndex,
};
use super::{
    transaction::endos,
    wrap::{Domain, Domains},
};

pub enum VerifierKind {
    Blockchain,
    Transaction,
}

fn read_index(path: &Path, digest: &[u8]) -> anyhow::Result<VerifierIndex<Fq>> {
    let mut buf = Vec::with_capacity(5700000);
    let mut file = File::open(path).context("opening cache file")?;
    let mut d = [0; 32];
    // source digest
    file.read_exact(&mut d).context("reading source digest")?;
    if d != digest {
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
    if d != digest.as_slice() {
        anyhow::bail!("verifier index digest verification failed");
    }
    Ok(verifier_index_from_bytes(&buf)?)
}

fn write_index(path: &Path, index: &VerifierIndex<Fq>, digest: &[u8]) -> anyhow::Result<()> {
    let bytes = verifier_index_to_bytes(index)?;
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
fn make_with_ext_cache(data: &str, _cache: &str) -> VerifierIndex<Fq> {
    let verifier_index: VerifierIndex<Fq> = serde_json::from_str(data).unwrap();
    make_verifier_index(verifier_index)
}

#[cfg(not(target_family = "wasm"))]
fn make_with_ext_cache(data: &str, cache: &str) -> VerifierIndex<Fq> {
    use super::caching::openmina_cache_path;
    let verifier_index: VerifierIndex<Fq> = serde_json::from_str(data).unwrap();
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

pub fn get_verifier_index(kind: VerifierKind) -> VerifierIndex<Fq> {
    match kind {
        VerifierKind::Blockchain => {
            cache_one!(VerifierIndex<Fq>, {
                let network_name = openmina_core::NetworkConfig::global().name;
                let (json_data, cache_filename) = match network_name {
                    "mainnet" => (
                        include_str!("data/mainnet_blockchain_verifier_index.json"),
                        "mainnet_block_verifier_index.bin",
                    ),
                    "devnet" => (
                        include_str!("data/devnet_blockchain_verifier_index.json"),
                        "devnet_block_verifier_index.bin",
                    ),
                    other => panic!("get_verifier_index: unknown network '{other}'"),
                };
                make_with_ext_cache(json_data, cache_filename)
            })
        }
        VerifierKind::Transaction => {
            cache_one!(VerifierIndex<Fq>, {
                let network_name = openmina_core::NetworkConfig::global().name;
                let (json_data, cache_filename) = match network_name {
                    "mainnet" => (
                        include_str!("data/mainnet_transaction_verifier_index.json"),
                        "mainnet_transaction_verifier_index.bin",
                    ),
                    "devnet" => (
                        include_str!("data/devnet_transaction_verifier_index.json"),
                        "devnet_transaction_verifier_index.bin",
                    ),
                    other => panic!("get_verifier_index: unknown network '{other}'"),
                };
                make_with_ext_cache(json_data, cache_filename)
            })
        }
    }
}

fn make_verifier_index(index: VerifierIndex<Fq>) -> VerifierIndex<Fq> {
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
    let permutation_vanishing_polynomial_m =
        permutation_vanishing_polynomial(domain, index.zk_rows);

    // https://github.com/o1-labs/proof-systems/blob/2702b09063c7a48131173d78b6cf9408674fd67e/kimchi/src/verifier_index.rs#L324
    let w = zk_w(domain, index.zk_rows);

    VerifierIndex::<Fq> {
        srs,
        permutation_vanishing_polynomial_m: OnceCell::from(permutation_vanishing_polynomial_m),
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
pub fn make_zkapp_verifier_index(vk: &VerificationKey) -> VerifierIndex<Fq> {
    let d = wrap_domains(vk.actual_wrap_domain_size.to_int());
    let log2_size = d.h.log2_size();

    let public = 40; // Is that constant ?

    let domain: Radix2EvaluationDomain<Fq> =
        Radix2EvaluationDomain::new(1 << log2_size as u64).unwrap();

    let srs = {
        let degree = 1 << BACKEND_TOCK_ROUNDS_N;
        let mut srs = SRS::<Pallas>::create(degree);
        srs.add_lagrange_basis(domain);
        srs
    };

    let make_poly = |poly: &InnerCurve<Fp>| poly_commitment::PolyComm {
        elems: vec![poly.to_affine()],
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

    // https://github.com/MinaProtocol/mina/blob/047375688f93546d4bdd58c75674394e3faae1f4/src/lib/pickles/side_loaded_verification_key.ml#L232
    let zk_rows = 3;

    // Note: Verifier index is converted from OCaml here:
    // https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/crypto/kimchi_bindings/stubs/src/pasta_fq_plonk_verifier_index.rs#L58

    VerifierIndex::<Fq> {
        domain,
        max_poly_size: 1 << BACKEND_TOCK_ROUNDS_N,
        srs: Arc::new(srs),
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
        permutation_vanishing_polynomial_m: OnceCell::with_value(permutation_vanishing_polynomial(
            domain, zk_rows,
        )),
        w: { OnceCell::with_value(zk_w(domain, zk_rows)) },
        endo: endo_q,
        lookup_index: None,
        linearization,
        powers_of_alpha,
        zk_rows,
    }
}
