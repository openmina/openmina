use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use once_cell::sync::OnceCell;
use openmina_core::{info, log::system_time, warn};
use serde::{Deserialize, Serialize};
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
    transaction::endos,
    wrap::{Domain, Domains},
};
use super::{transaction::InnerCurve, VerifierIndex};

#[derive(Clone, Copy)]
enum Kind {
    BlockVerifier,
    TransactionVerifier,
}

impl Kind {
    pub fn to_str(self) -> &'static str {
        match self {
            Self::BlockVerifier => "block_verifier_index",
            Self::TransactionVerifier => "transaction_verifier_index",
        }
    }

    pub fn filename(self) -> String {
        format!("{}.postcard", self.to_str())
    }
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

fn cache_filename(kind: Kind) -> PathBuf {
    let circuits_config = openmina_core::NetworkConfig::global().circuits_config;
    Path::new(circuits_config.directory_name).join(kind.filename())
}

#[cfg(not(target_family = "wasm"))]
fn cache_path(kind: Kind) -> Option<PathBuf> {
    super::circuit_blobs::home_base_dir().map(|p| p.join(cache_filename(kind)))
}

async fn read_cache(kind: Kind, digest: &[u8]) -> anyhow::Result<VerifierIndex<Fq>> {
    let data = super::circuit_blobs::fetch(&cache_filename(kind))
        .await
        .context("fetching verifier index failed")?;
    let mut slice = data.as_slice();
    let mut d = [0; 32];
    // source digest
    slice.read_exact(&mut d).context("reading source digest")?;
    if d != digest {
        anyhow::bail!("source digest verification failed");
    }

    // index digest
    slice.read_exact(&mut d).context("reading index digest")?;

    let mut hasher = Sha256::new();
    hasher.update(slice);
    let digest = hasher.finalize();
    if d != digest.as_slice() {
        anyhow::bail!("verifier index digest verification failed");
    }
    Ok(super::caching::verifier_index_from_bytes(slice)?)
}

#[cfg(not(target_family = "wasm"))]
fn write_cache(kind: Kind, index: &VerifierIndex<Fq>, digest: &[u8]) -> anyhow::Result<()> {
    use std::{fs::File, io::Write};

    let path = cache_path(kind)
        .ok_or_else(|| anyhow::anyhow!("$HOME env not set, so can't cache verifier index"))?;
    let bytes = super::caching::verifier_index_to_bytes(index)?;
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

async fn make_with_ext_cache(kind: Kind, data: &str) -> VerifierIndex<Fq> {
    let verifier_index: VerifierIndex<Fq> = serde_json::from_str(data).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(data);
    let src_index_digest = hasher.finalize();

    let cache = read_cache(kind, &src_index_digest).await;

    match cache {
        Ok(verifier_index) => {
            info!(system_time(); "Verifier index is loaded");
            verifier_index
        }
        Err(err) => {
            warn!(system_time(); "Cannot load verifier index: {err}");
            let index = make_verifier_index(verifier_index);
            #[cfg(not(target_family = "wasm"))]
            if let Err(err) = write_cache(kind, &index, &src_index_digest) {
                warn!(system_time(); "Cannot store verifier index to cache file: {err}");
            } else {
                info!(system_time(); "Stored verifier index to cache file");
            }
            index
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockVerifier(Arc<VerifierIndex<Fq>>);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionVerifier(Arc<VerifierIndex<Fq>>);

static BLOCK_VERIFIER: OnceCell<BlockVerifier> = OnceCell::new();
static TX_VERIFIER: OnceCell<TransactionVerifier> = OnceCell::new();

impl BlockVerifier {
    fn kind() -> Kind {
        Kind::BlockVerifier
    }

    fn src_json() -> &'static str {
        let network_name = openmina_core::NetworkConfig::global().name;
        match network_name {
            "mainnet" => include_str!("data/mainnet_blockchain_verifier_index.json"),
            "devnet" => include_str!("data/devnet_blockchain_verifier_index.json"),
            other => panic!("get_verifier_index: unknown network '{other}'"),
        }
    }
}

impl TransactionVerifier {
    fn kind() -> Kind {
        Kind::TransactionVerifier
    }

    fn src_json() -> &'static str {
        let network_name = openmina_core::NetworkConfig::global().name;
        match network_name {
            "mainnet" => include_str!("data/mainnet_transaction_verifier_index.json"),
            "devnet" => include_str!("data/devnet_transaction_verifier_index.json"),
            other => panic!("get_verifier_index: unknown network '{other}'"),
        }
    }

    pub fn get() -> Option<Self> {
        TX_VERIFIER.get().cloned()
    }
}

impl BlockVerifier {
    pub async fn make_async() -> Self {
        if let Some(v) = BLOCK_VERIFIER.get() {
            v.clone()
        } else {
            let verifier = Self(Arc::new(
                make_with_ext_cache(Self::kind(), Self::src_json()).await,
            ));
            BLOCK_VERIFIER.get_or_init(move || verifier).clone()
        }
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn make() -> Self {
        super::provers::block_on(Self::make_async())
    }
}

impl TransactionVerifier {
    pub async fn make_async() -> Self {
        if let Some(v) = TX_VERIFIER.get() {
            v.clone()
        } else {
            let verifier = Self(Arc::new(
                make_with_ext_cache(Self::kind(), Self::src_json()).await,
            ));
            TX_VERIFIER.get_or_init(move || verifier).clone()
        }
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn make() -> Self {
        super::provers::block_on(Self::make_async())
    }
}

impl std::ops::Deref for BlockVerifier {
    type Target = VerifierIndex<Fq>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::Deref for TransactionVerifier {
    type Target = VerifierIndex<Fq>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<BlockVerifier> for Arc<VerifierIndex<Fq>> {
    fn from(value: BlockVerifier) -> Self {
        value.0
    }
}

impl From<TransactionVerifier> for Arc<VerifierIndex<Fq>> {
    fn from(value: TransactionVerifier) -> Self {
        value.0
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
