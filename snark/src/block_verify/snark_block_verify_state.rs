use std::sync::Arc;

use serde::{Deserialize, Serialize};

use openmina_core::{block::BlockHash, requests::PendingRequests};

use crate::{BlockVerifier, VerifierSRS};

use super::{
    SnarkBlockVerifyError, SnarkBlockVerifyId, SnarkBlockVerifyIdType, VerifiableBlockWithHash,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct SnarkBlockVerifyState {
    pub verifier_index: BlockVerifier,
    pub verifier_srs: Arc<VerifierSRS>,
    pub jobs: PendingRequests<SnarkBlockVerifyIdType, SnarkBlockVerifyStatus>,
}

impl SnarkBlockVerifyState {
    pub fn new(verifier_index: BlockVerifier, verifier_srs: Arc<VerifierSRS>) -> Self {
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
        on_success: redux::Callback<BlockHash>,
        on_error: redux::Callback<(BlockHash, SnarkBlockVerifyError)>,
    },
    Pending {
        time: redux::Timestamp,
        block: VerifiableBlockWithHash,
        on_success: redux::Callback<BlockHash>,
        on_error: redux::Callback<(BlockHash, SnarkBlockVerifyError)>,
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
