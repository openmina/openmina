use mina_hasher::Fp;

use super::{
    merge::{Basic, FeatureFlags, ForStep, ForStepKind, OptFlag},
    witness::CircuitPlonkVerificationKeyEvals,
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

pub struct StepZkappProofProof {}
pub struct StepZkappOptSignedProof {}
pub struct StepZkappOptSignedOptSignedProof {}
pub struct WrapZkappProof {}
pub struct WrapZkappOptSignedProof {}

impl ProofConstants for StepZkappOptSignedOptSignedProof {
    const PRIMARY_LEN: usize = 67;
    const AUX_LEN: usize = 104744;
    const PREVIOUS_CHALLENGES: usize = 0;
    const ROWS: usize = 18590;
}

impl ProofConstants for StepZkappOptSignedProof {
    const PRIMARY_LEN: usize = 67;
    const AUX_LEN: usize = 71779;
    const PREVIOUS_CHALLENGES: usize = 0;
    const ROWS: usize = 11298;
}

impl ProofConstants for StepZkappProofProof {
    const PRIMARY_LEN: usize = 67;
    const AUX_LEN: usize = 210282;
    const PREVIOUS_CHALLENGES: usize = 1;
    const ROWS: usize = 19980;
}

// Same values than `WrapTransactionProof`
impl ProofConstants for WrapZkappProof {
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
    const AUX_LEN: usize = 94386;
    const PREVIOUS_CHALLENGES: usize = 0;
    const ROWS: usize = 17794;
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
    const AUX_LEN: usize = 179248;
    const PREVIOUS_CHALLENGES: usize = 2;
    const ROWS: usize = 14657;
}

impl ProofConstants for StepMergeProof {
    const PRIMARY_LEN: usize = 67;
    const AUX_LEN: usize = 352536;
    const PREVIOUS_CHALLENGES: usize = 2;
    const ROWS: usize = 29010;
}

impl ProofConstants for StepBlockProof {
    const PRIMARY_LEN: usize = 67;
    const AUX_LEN: usize = 339034;
    const PREVIOUS_CHALLENGES: usize = 2;
    const ROWS: usize = 34811;
}

pub trait ForWrapData {
    fn wrap_data() -> WrapData;
}

impl ForWrapData for WrapTransactionProof {
    fn wrap_data() -> WrapData {
        make_wrap_transaction_data()
    }
}

impl ForWrapData for WrapZkappProof {
    fn wrap_data() -> WrapData {
        make_wrap_zkapp_data()
    }
}

impl ForWrapData for WrapZkappOptSignedProof {
    fn wrap_data() -> WrapData {
        make_wrap_zkapp_opt_signed_data()
    }
}

impl ForWrapData for WrapBlockProof {
    fn wrap_data() -> WrapData {
        make_wrap_block_data()
    }
}

impl ForWrapData for WrapMergeProof {
    fn wrap_data() -> WrapData {
        make_wrap_merge_data()
    }
}

pub fn make_step_block_data(wrap_key: &CircuitPlonkVerificationKeyEvals<Fp>) -> ForStep {
    let basic = Basic {
        proof_verifieds: vec![2],
        wrap_domain: Domains {
            h: Domain::Pow2RootsOfUnity(14),
        },
        step_domains: vec![Domains {
            h: Domain::Pow2RootsOfUnity(16),
        }],
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

pub fn make_step_transaction_data(wrap_key: &CircuitPlonkVerificationKeyEvals<Fp>) -> ForStep {
    let basic = Basic {
        proof_verifieds: vec![0, 2, 0, 0, 1],
        wrap_domain: Domains {
            h: Domain::Pow2RootsOfUnity(14),
        },
        step_domains: vec![
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
        ],
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
        step_domains: vec![
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
        ],
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
        wrap_domain: ForStepKind::SideLoaded,
        step_domains: ForStepKind::SideLoaded,
        feature_flags: basic.feature_flags,
    };

    self_data
}

pub struct WrapData {
    pub which_index: u64,
    pub pi_branches: u64,
    pub step_widths: Box<[u64]>, // TODO: Use array with size=pi_branches
    pub step_domains: Box<[Domains]>, // Here too
}

fn make_wrap_block_data() -> WrapData {
    WrapData {
        which_index: 0,
        pi_branches: 1,
        step_widths: Box::new([2]),
        step_domains: Box::new([Domains {
            h: Domain::Pow2RootsOfUnity(16),
        }]),
    }
}

fn make_wrap_merge_data() -> WrapData {
    WrapData {
        which_index: 1,
        pi_branches: 5,
        step_widths: Box::new([0, 2, 0, 0, 1]),
        step_domains: Box::new([
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
        ]),
    }
}

fn make_wrap_transaction_data() -> WrapData {
    WrapData {
        which_index: 0,
        pi_branches: 5,
        step_widths: Box::new([0, 2, 0, 0, 1]),
        step_domains: Box::new([
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
        ]),
    }
}

fn make_wrap_zkapp_data() -> WrapData {
    WrapData {
        which_index: 2,
        pi_branches: 5,
        step_widths: Box::new([0, 2, 0, 0, 1]),
        step_domains: Box::new([
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
        ]),
    }
}

fn make_wrap_zkapp_opt_signed_data() -> WrapData {
    WrapData {
        which_index: 3,
        pi_branches: 5,
        step_widths: Box::new([0, 2, 0, 0, 1]),
        step_domains: Box::new([
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
        ]),
    }
}
