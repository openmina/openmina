use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use openmina_core::requests::PendingRequests;
use openmina_core::snark::Snark;

use crate::{VerifierIndex, VerifierSRS};

use super::{SnarkWorkVerifyError, SnarkWorkVerifyId, SnarkWorkVerifyIdType};

#[derive(Serialize, Deserialize, Clone)]
pub struct SnarkWorkVerifyState {
    pub verifier_index: Arc<VerifierIndex>,
    pub verifier_srs: Arc<Mutex<VerifierSRS>>,
    pub jobs: PendingRequests<SnarkWorkVerifyIdType, SnarkWorkVerifyStatus>,
}

impl SnarkWorkVerifyState {
    pub fn new(verifier_index: Arc<VerifierIndex>, verifier_srs: Arc<Mutex<VerifierSRS>>) -> Self {
        Self {
            verifier_index,
            verifier_srs,
            jobs: Default::default(),
        }
    }

    pub fn next_req_id(&self) -> SnarkWorkVerifyId {
        self.jobs.next_req_id()
    }
}

impl std::fmt::Debug for SnarkWorkVerifyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SnarkWorkVerifyState")
            // TODO(binier): display hashes instead.
            .field("verifier_index", &"<content too big>")
            .field("verifier_srs", &"<content too big>")
            .field("jobs", &self.jobs)
            .finish()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnarkWorkVerifyStatus {
    Init {
        time: redux::Timestamp,
        batch: Vec<Snark>,
        // TODO(binier): move p2p/src/identity to shared crate and use
        // `PeerId` here.
        sender: String,
    },
    Pending {
        time: redux::Timestamp,
        batch: Vec<Snark>,
        sender: String,
    },
    Error {
        time: redux::Timestamp,
        batch: Vec<Snark>,
        sender: String,
        error: SnarkWorkVerifyError,
    },
    Success {
        time: redux::Timestamp,
        batch: Vec<Snark>,
        sender: String,
    },
}

impl SnarkWorkVerifyStatus {
    pub fn is_init(&self) -> bool {
        matches!(self, Self::Init { .. })
    }

    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending { .. })
    }

    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Error { .. } | Self::Success { .. })
    }

    pub fn batch(&self) -> &[Snark] {
        match self {
            Self::Init { batch, .. } => batch,
            Self::Pending { batch, .. } => batch,
            Self::Error { batch, .. } => batch,
            Self::Success { batch, .. } => batch,
        }
    }

    pub fn sender(&self) -> &str {
        match self {
            Self::Init { sender, .. } => sender,
            Self::Pending { sender, .. } => sender,
            Self::Error { sender, .. } => sender,
            Self::Success { sender, .. } => sender,
        }
    }
}
