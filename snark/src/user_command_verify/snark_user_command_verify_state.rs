use std::sync::Arc;

use ledger::scan_state::transaction_logic::{verifiable, WithStatus};
use mina_p2p_messages::{list::List, v2};
use serde::{Deserialize, Serialize};

use openmina_core::requests::PendingRequests;

use crate::{TransactionVerifier, VerifierSRS};

use super::{SnarkUserCommandVerifyError, SnarkUserCommandVerifyId, SnarkUserCommandVerifyIdType};

#[derive(Serialize, Deserialize, Clone)]
pub struct SnarkUserCommandVerifyState {
    pub verifier_index: TransactionVerifier,
    pub verifier_srs: Arc<VerifierSRS>,
    pub jobs: PendingRequests<SnarkUserCommandVerifyIdType, SnarkUserCommandVerifyStatus>,
}

impl SnarkUserCommandVerifyState {
    pub fn new(verifier_index: TransactionVerifier, verifier_srs: Arc<VerifierSRS>) -> Self {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnarkUserCommandVerifyStatus {
    Init {
        time: redux::Timestamp,
        commands: List<v2::MinaBaseUserCommandStableV2>,
    },
    Pending {
        time: redux::Timestamp,
        commands: Vec<WithStatus<verifiable::UserCommand>>,
    },
    Error {
        time: redux::Timestamp,
        commands: Vec<WithStatus<verifiable::UserCommand>>,
        error: SnarkUserCommandVerifyError,
    },
    Success {
        time: redux::Timestamp,
        commands: Vec<WithStatus<verifiable::UserCommand>>,
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
}
