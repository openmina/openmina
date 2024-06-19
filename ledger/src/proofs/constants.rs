use ark_ff::{One, Zero};
use mina_curves::pasta::Fq;
use mina_hasher::Fp;

use super::{
    field::Boolean,
    step::{Basic, FeatureFlags, ForStep, ForStepKind, OptFlag},
    transaction::CircuitPlonkVerificationKeyEvals,
    wrap::{Domain, Domains},
};

pub trait ProofConstants {
    /// Number of public inputs
    const PRIMARY_LEN: usize;
    /// Number of witness values
    const AUX_LEN: usize;
    const PREVIOUS_CHALLENGES: usize;
    const ROWS: usize;
}

pub struct StepTransactionProof {}
pub struct WrapTransactionProof {}

pub struct StepBlockProof {}
pub struct WrapBlockProof {}

pub struct StepMergeProof {}
pub struct WrapMergeProof {}

pub struct StepZkappProvedProof {}
pub struct StepZkappOptSignedProof {}
pub struct StepZkappOptSignedOptSignedProof {}
/// Using signature authorization
pub struct WrapZkappProof {}
/// Using proof authorization
pub struct WrapZkappProvedProof {}
pub struct WrapZkappOptSignedProof {}

impl ProofConstants for StepZkappOptSignedOptSignedProof {
    const PRIMARY_LEN: usize = 67;
    const AUX_LEN: usize = 104914;
    const PREVIOUS_CHALLENGES: usize = 0;
    const ROWS: usize = 18655;
}

impl ProofConstants for StepZkappOptSignedProof {
    const PRIMARY_LEN: usize = 67;
    const AUX_LEN: usize = 71842;
    const PREVIOUS_CHALLENGES: usize = 0;
    const ROWS: usize = 11332;
}

impl ProofConstants for StepZkappProvedProof {
    const PRIMARY_LEN: usize = 67;
    const AUX_LEN: usize = 210350;
    const PREVIOUS_CHALLENGES: usize = 1;
    const ROWS: usize = 20023;
}

// Same values than `WrapTransactionProof`
impl ProofConstants for WrapZkappProof {
    const PRIMARY_LEN: usize = WrapTransactionProof::PRIMARY_LEN;
    const AUX_LEN: usize = WrapTransactionProof::AUX_LEN;
    const PREVIOUS_CHALLENGES: usize = WrapTransactionProof::PREVIOUS_CHALLENGES;
    const ROWS: usize = WrapTransactionProof::ROWS;
}

// Same values than `WrapTransactionProof`
impl ProofConstants for WrapZkappProvedProof {
    const PRIMARY_LEN: usize = WrapTransactionProof::PRIMARY_LEN;
    const AUX_LEN: usize = WrapTransactionProof::AUX_LEN;
    const PREVIOUS_CHALLENGES: usize = WrapTransactionProof::PREVIOUS_CHALLENGES;
    const ROWS: usize = WrapTransactionProof::ROWS;
}

// Same values than `WrapTransactionProof`
impl ProofConstants for WrapZkappOptSignedProof {
    const PRIMARY_LEN: usize = WrapTransactionProof::PRIMARY_LEN;
    const AUX_LEN: usize = WrapTransactionProof::AUX_LEN;
    const PREVIOUS_CHALLENGES: usize = WrapTransactionProof::PREVIOUS_CHALLENGES;
    const ROWS: usize = WrapTransactionProof::ROWS;
}

impl ProofConstants for StepTransactionProof {
    const PRIMARY_LEN: usize = 67;
    const AUX_LEN: usize = 94478;
    const PREVIOUS_CHALLENGES: usize = 0;
    const ROWS: usize = 17806;
}

impl ProofConstants for WrapTransactionProof {
    const PRIMARY_LEN: usize = 40;
    const AUX_LEN: usize = 179491;
    const PREVIOUS_CHALLENGES: usize = 2;
    const ROWS: usize = 15122;
}

// Same values than `WrapTransactionProof`
impl ProofConstants for WrapMergeProof {
    const PRIMARY_LEN: usize = WrapTransactionProof::PRIMARY_LEN;
    const AUX_LEN: usize = WrapTransactionProof::AUX_LEN;
    const PREVIOUS_CHALLENGES: usize = WrapTransactionProof::PREVIOUS_CHALLENGES;
    const ROWS: usize = WrapTransactionProof::ROWS;
}

impl ProofConstants for WrapBlockProof {
    const PRIMARY_LEN: usize = 40;
    const AUX_LEN: usize = 179208;
    const PREVIOUS_CHALLENGES: usize = 2;
    const ROWS: usize = 14657;
}

impl ProofConstants for StepMergeProof {
    const PRIMARY_LEN: usize = 67;
    const AUX_LEN: usize = 352469;
    const PREVIOUS_CHALLENGES: usize = 2;
    const ROWS: usize = 29010;
}

impl ProofConstants for StepBlockProof {
    const PRIMARY_LEN: usize = 67;
    const AUX_LEN: usize = 338873;
    const PREVIOUS_CHALLENGES: usize = 2;
    const ROWS: usize = 34797;
}

pub trait ForWrapData {
    fn wrap_data() -> WrapData;
}

#[derive(Debug)]
pub struct WrapData {
    pub which_index: u64,
    pub pi_branches: u64,
    pub step_widths: Box<[u64]>, // TODO: Use array with size=pi_branches
    pub step_domains: Box<[Domains]>, // Here too
    pub wrap_domain_indices: Box<[Fq; 2]>,
}

impl ForWrapData for WrapTransactionProof {
    fn wrap_data() -> WrapData {
        WrapData {
            which_index: 0,
            pi_branches: 5,
            step_widths: default_step_widths(),
            step_domains: default_step_domains(),
            wrap_domain_indices: default_wrap_domain_indices(),
        }
    }
}

impl ForWrapData for WrapZkappProof {
    fn wrap_data() -> WrapData {
        WrapData {
            which_index: 2,
            pi_branches: 5,
            step_widths: default_step_widths(),
            step_domains: default_step_domains(),
            wrap_domain_indices: default_wrap_domain_indices(),
        }
    }
}

impl ForWrapData for WrapZkappProvedProof {
    fn wrap_data() -> WrapData {
        WrapData {
            which_index: 4,
            pi_branches: 5,
            step_widths: default_step_widths(),
            step_domains: default_step_domains(),
            wrap_domain_indices: Box::new([Fq::one(), Fq::zero()]),
        }
    }
}

impl ForWrapData for WrapZkappOptSignedProof {
    fn wrap_data() -> WrapData {
        WrapData {
            which_index: 3,
            pi_branches: 5,
            step_widths: default_step_widths(),
            step_domains: default_step_domains(),
            wrap_domain_indices: default_wrap_domain_indices(),
        }
    }
}

impl ForWrapData for WrapBlockProof {
    fn wrap_data() -> WrapData {
        WrapData {
            which_index: 0,
            pi_branches: 1,
            step_widths: Box::new([2]),
            step_domains: Box::new([Domains {
                h: Domain::Pow2RootsOfUnity(16),
            }]),
            wrap_domain_indices: default_wrap_domain_indices(),
        }
    }
}

impl ForWrapData for WrapMergeProof {
    fn wrap_data() -> WrapData {
        WrapData {
            which_index: 1,
            pi_branches: 5,
            step_widths: default_step_widths(),
            step_domains: default_step_domains(),
            wrap_domain_indices: default_wrap_domain_indices(),
        }
    }
}

fn default_step_domains() -> Box<[Domains]> {
    Box::new([
        Domains {
            h: Domain::Pow2RootsOfUnity(15),
        },
        Domains {
            h: Domain::Pow2RootsOfUnity(15),
        },
        Domains {
            h: Domain::Pow2RootsOfUnity(15),
        },
        Domains {
            h: Domain::Pow2RootsOfUnity(14),
        },
        Domains {
            h: Domain::Pow2RootsOfUnity(15),
        },
    ])
}

fn default_wrap_domain_indices() -> Box<[Fq; 2]> {
    Box::new([Fq::one(), Fq::one()])
}

fn default_step_widths() -> Box<[u64]> {
    Box::new([0, 2, 0, 0, 1])
}

pub fn make_step_transaction_data(wrap_key: &CircuitPlonkVerificationKeyEvals<Fp>) -> ForStep {
    let basic = Basic {
        proof_verifieds: vec![0, 2, 0, 0, 1],
        wrap_domain: Domains {
            h: Domain::Pow2RootsOfUnity(14),
        },
        step_domains: default_step_domains(),
        feature_flags: FeatureFlags {
            range_check0: OptFlag::No,
            range_check1: OptFlag::No,
            foreign_field_add: OptFlag::No,
            foreign_field_mul: OptFlag::No,
            xor: OptFlag::No,
            rot: OptFlag::No,
            lookup: OptFlag::No,
            runtime_tables: OptFlag::No,
        },
    };

    let self_branches = 5;
    let max_proofs_verified = 2;
    let self_data = ForStep {
        branches: self_branches,
        max_proofs_verified,
        proof_verifieds: ForStepKind::Known(
            basic
                .proof_verifieds
                .iter()
                .copied()
                .map(Fp::from)
                .collect(),
        ),
        public_input: (),
        wrap_key: wrap_key.clone(), // TODO: Use ref
        wrap_domain: ForStepKind::Known(basic.wrap_domain.h),
        step_domains: ForStepKind::Known(basic.step_domains),
        feature_flags: basic.feature_flags,
    };

    self_data
}

/// Zkapps using proof authorization
pub fn make_step_zkapp_data(wrap_key: &CircuitPlonkVerificationKeyEvals<Fp>) -> ForStep {
    let basic = Basic {
        proof_verifieds: vec![0, 2, 0, 0, 1],
        wrap_domain: Domains {
            h: Domain::Pow2RootsOfUnity(15),
        },
        step_domains: default_step_domains(),
        feature_flags: FeatureFlags {
            range_check0: OptFlag::Maybe,
            range_check1: OptFlag::Maybe,
            foreign_field_add: OptFlag::Maybe,
            foreign_field_mul: OptFlag::Maybe,
            xor: OptFlag::Maybe,
            rot: OptFlag::Maybe,
            lookup: OptFlag::Maybe,
            runtime_tables: OptFlag::Maybe,
        },
    };

    let self_branches = 5; // 8 ?
    let max_proofs_verified = 2;
    let self_data = ForStep {
        branches: self_branches,
        max_proofs_verified,
        proof_verifieds: ForStepKind::Known(
            basic
                .proof_verifieds
                .iter()
                .copied()
                .map(Fp::from)
                .collect(),
        ),
        public_input: (),
        wrap_key: wrap_key.clone(), // TODO: Use ref
        wrap_domain: ForStepKind::SideLoaded(Box::new([
            Boolean::True,
            Boolean::False,
            Boolean::False,
        ])),
        step_domains: ForStepKind::SideLoaded(()),
        feature_flags: basic.feature_flags,
    };

    self_data
}

pub fn make_step_block_data(wrap_key: &CircuitPlonkVerificationKeyEvals<Fp>) -> ForStep {
    let basic = Basic {
        proof_verifieds: vec![2],
        wrap_domain: Domains {
            h: Domain::Pow2RootsOfUnity(14),
        },
        step_domains: Box::new([Domains {
            h: Domain::Pow2RootsOfUnity(16),
        }]),
        feature_flags: FeatureFlags {
            range_check0: OptFlag::No,
            range_check1: OptFlag::No,
            foreign_field_add: OptFlag::No,
            foreign_field_mul: OptFlag::No,
            xor: OptFlag::No,
            rot: OptFlag::No,
            lookup: OptFlag::No,
            runtime_tables: OptFlag::No,
        },
    };

    let self_branches = 1;
    let max_proofs_verified = 2;
    let self_data = ForStep {
        branches: self_branches,
        max_proofs_verified,
        proof_verifieds: ForStepKind::Known(
            basic
                .proof_verifieds
                .iter()
                .copied()
                .map(Fp::from)
                .collect(),
        ),
        public_input: (),
        wrap_key: wrap_key.clone(), // TODO: Use ref
        wrap_domain: ForStepKind::Known(basic.wrap_domain.h),
        step_domains: ForStepKind::Known(basic.step_domains),
        feature_flags: basic.feature_flags,
    };

    self_data
}
