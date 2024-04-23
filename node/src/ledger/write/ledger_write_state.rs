use serde::{Deserialize, Serialize};

use super::{LedgerWriteRequest, LedgerWriteResponse};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LedgerWriteState {
    Idle,
    Init {
        time: redux::Timestamp,
        request: LedgerWriteRequest,
    },
    Pending {
        time: redux::Timestamp,
        request: LedgerWriteRequest,
    },
    Success {
        time: redux::Timestamp,
        request: LedgerWriteRequest,
        response: LedgerWriteResponse,
    },
}

impl LedgerWriteState {
    pub fn request(&self) -> Option<&LedgerWriteRequest> {
        match self {
            Self::Idle => None,
            Self::Init { request, .. }
            | Self::Pending { request, .. }
            | Self::Success { request, .. } => Some(request),
        }
    }
}

impl Default for LedgerWriteState {
    fn default() -> Self {
        Self::Idle
    }
}
