use std::sync::Arc;

use serde::{Deserialize, Serialize};

use openmina_core::requests::PendingRequests;

use crate::{VerifierIndex, VerifierSRS};

use super::{
    SnarkBlockVerifyError, SnarkBlockVerifyId, SnarkBlockVerifyIdType, VerifiableBlockWithHash,
};

/// This structure contains the state of snark block verification.
/// 
/// The `verifier_index` and `verifier_srs` are set from the
/// `block_verifier_index` and `block_verifier_srs` fields of `SnarkConfig`.
/// 
/// The `jobs` is list of block verification work that can be in any of the
/// following states: Init / Pending / Error / Success. Inside every of these
/// states we can find a `Timestamp` and a `VerifiableBlockWithHash` which
/// wraps either a full block or just the block header. The verifier can later
/// extract the protocol state and protocol state proof from the block header
/// to perform the proof verification.
/// 
#[derive(Serialize, Deserialize, Clone)]
pub struct SnarkBlockVerifyState {
    pub verifier_index: Arc<VerifierIndex>,
    pub verifier_srs: Arc<VerifierSRS>,
    pub jobs: PendingRequests<SnarkBlockVerifyIdType, SnarkBlockVerifyStatus>,
}

impl SnarkBlockVerifyState {
    pub fn new(verifier_index: Arc<VerifierIndex>, verifier_srs: Arc<VerifierSRS>) -> Self {
        Self {
            verifier_index,
            verifier_srs,
            jobs: Default::default(),
        }
    }

    pub fn next_req_id(&self) -> SnarkBlockVerifyId {
        self.jobs.next_req_id()
    }
}

impl std::fmt::Debug for SnarkBlockVerifyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SnarkBlockVerifyState")
            // TODO(binier): display hashes instead.
            .field("verifier_index", &"<content too big>")
            .field("verifier_srs", &"<content too big>")
            .field("jobs", &self.jobs)
            .finish()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnarkBlockVerifyStatus {
    Init {
        time: redux::Timestamp,
        block: VerifiableBlockWithHash,
    },
    Pending {
        time: redux::Timestamp,
        block: VerifiableBlockWithHash,
    },
    Error {
        time: redux::Timestamp,
        block: VerifiableBlockWithHash,
        error: SnarkBlockVerifyError,
    },
    Success {
        time: redux::Timestamp,
        block: VerifiableBlockWithHash,
    },
}

impl SnarkBlockVerifyStatus {
    pub fn is_init(&self) -> bool {
        matches!(self, Self::Init { .. })
    }

    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending { .. })
    }

    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Error { .. } | Self::Success { .. })
    }

    pub fn block(&self) -> &VerifiableBlockWithHash {
        match self {
            Self::Init { block, .. } => block,
            Self::Pending { block, .. } => block,
            Self::Error { block, .. } => block,
            Self::Success { block, .. } => block,
        }
    }
}
