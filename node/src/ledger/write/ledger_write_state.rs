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

    pub fn pending_requests(
        &self,
    ) -> impl Iterator<Item = (&LedgerWriteRequest, redux::Timestamp)> {
        std::iter::once(match self {
            Self::Pending { time, request } => Some((request, *time)),
            _ => None,
        })
        .flatten()
    }
}

impl Default for LedgerWriteState {
    fn default() -> Self {
        Self::Idle
    }
}
