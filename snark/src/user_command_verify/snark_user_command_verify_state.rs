use std::sync::{Arc, Mutex};

use ledger::scan_state::transaction_logic::{verifiable, WithStatus};
use serde::{Deserialize, Serialize};

use openmina_core::requests::PendingRequests;

use crate::{VerifierIndex, VerifierSRS};

use super::{SnarkUserCommandVerifyError, SnarkUserCommandVerifyId, SnarkUserCommandVerifyIdType};

#[derive(Serialize, Deserialize, Clone)]
pub struct SnarkUserCommandVerifyState {
    pub verifier_index: Arc<VerifierIndex>,
    pub verifier_srs: Arc<Mutex<VerifierSRS>>,
    pub jobs: PendingRequests<SnarkUserCommandVerifyIdType, SnarkUserCommandVerifyStatus>,
}

impl SnarkUserCommandVerifyState {
    pub fn new(verifier_index: Arc<VerifierIndex>, verifier_srs: Arc<Mutex<VerifierSRS>>) -> Self {
        Self {
            verifier_index,
            verifier_srs,
            jobs: Default::default(),
        }
    }

    pub fn next_req_id(&self) -> SnarkUserCommandVerifyId {
        self.jobs.next_req_id()
    }
}

impl std::fmt::Debug for SnarkUserCommandVerifyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SnarkUserCommandVerifyState")
            // TODO(binier): display hashes instead.
            .field("verifier_index", &"<content too big>")
            .field("verifier_srs", &"<content too big>")
            .field("jobs", &self.jobs)
            .finish()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SnarkUserCommandVerifyStatus {
    Init {
        time: redux::Timestamp,
        commands: Vec<WithStatus<verifiable::UserCommand>>,
        // TODO(binier): move p2p/src/identity to shared crate and use
        // `PeerId` here.
        sender: String,
    },
    Pending {
        time: redux::Timestamp,
        commands: Vec<WithStatus<verifiable::UserCommand>>,
        sender: String,
    },
    Error {
        time: redux::Timestamp,
        commands: Vec<WithStatus<verifiable::UserCommand>>,
        sender: String,
        error: SnarkUserCommandVerifyError,
    },
    Success {
        time: redux::Timestamp,
        commands: Vec<WithStatus<verifiable::UserCommand>>,
        sender: String,
    },
}

impl SnarkUserCommandVerifyStatus {
    pub fn is_init(&self) -> bool {
        matches!(self, Self::Init { .. })
    }

    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending { .. })
    }

    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Error { .. } | Self::Success { .. })
    }

    pub fn commands(&self) -> &[WithStatus<verifiable::UserCommand>] {
        match self {
            Self::Init { commands, .. } => commands,
            Self::Pending { commands, .. } => commands,
            Self::Error { commands, .. } => commands,
            Self::Success { commands, .. } => commands,
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
