pub mod staged_ledger_parts;
use staged_ledger_parts::{
    StagedLedgerPartsReceiveProgress, StagedLedgerPartsResponse, StagedLedgerPartsResponseFull,
    StagedLedgerPartsSendProgress,
};

use std::time::Duration;

use binprot_derive::{BinProtRead, BinProtWrite};
use derive_more::From;
use mina_p2p_messages::v2;
use serde::{Deserialize, Serialize};

use crate::P2pTimeouts;

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum P2pStreamingRpcKind {
    StagedLedgerParts,
}

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum P2pStreamingRpcRequest {
    StagedLedgerParts(v2::StateHash),
}

#[derive(Serialize, Deserialize, From, Debug, Clone)]
pub enum P2pStreamingRpcResponseFull {
    StagedLedgerParts(StagedLedgerPartsResponseFull),
}

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, From, Debug, Clone)]
pub enum P2pStreamingRpcResponse {
    StagedLedgerParts(StagedLedgerPartsResponse),
}

#[derive(Serialize, Deserialize, From, Debug, Clone)]
pub enum P2pStreamingRpcSendProgress {
    StagedLedgerParts(StagedLedgerPartsSendProgress),
}

#[derive(Serialize, Deserialize, From, Debug, Clone)]
pub enum P2pStreamingRpcReceiveProgress {
    StagedLedgerParts(StagedLedgerPartsReceiveProgress),
}

impl P2pStreamingRpcKind {
    pub fn timeout(self, _config: &P2pTimeouts) -> Option<Duration> {
        match self {
            // TODO(binier): use config
            Self::StagedLedgerParts => Some(Duration::from_secs(10)),
        }
    }
}

impl P2pStreamingRpcRequest {
    pub fn kind(&self) -> P2pStreamingRpcKind {
        match self {
            Self::StagedLedgerParts(_) => P2pStreamingRpcKind::StagedLedgerParts,
        }
    }
}

impl Default for P2pStreamingRpcRequest {
    fn default() -> Self {
        Self::StagedLedgerParts(v2::StateHash::zero())
    }
}

impl std::fmt::Display for P2pStreamingRpcRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.kind())?;
        match self {
            Self::StagedLedgerParts(block_hash) => write!(f, ", {block_hash}"),
        }
    }
}

impl P2pStreamingRpcResponseFull {
    pub fn kind(&self) -> P2pStreamingRpcKind {
        match self {
            Self::StagedLedgerParts(_) => P2pStreamingRpcKind::StagedLedgerParts,
        }
    }
}

impl P2pStreamingRpcResponse {
    pub fn kind(&self) -> P2pStreamingRpcKind {
        match self {
            Self::StagedLedgerParts(_) => P2pStreamingRpcKind::StagedLedgerParts,
        }
    }
}

impl P2pStreamingRpcSendProgress {
    pub fn kind(&self) -> P2pStreamingRpcKind {
        match self {
            Self::StagedLedgerParts(_) => P2pStreamingRpcKind::StagedLedgerParts,
        }
    }

    pub fn external_data_todo(&self) -> bool {
        match self {
            Self::StagedLedgerParts(v) => {
                matches!(v, StagedLedgerPartsSendProgress::LedgerGetIdle { .. })
            }
        }
    }

    pub fn external_data_pending(&self) -> bool {
        match self {
            Self::StagedLedgerParts(v) => {
                matches!(v, StagedLedgerPartsSendProgress::LedgerGetPending { .. })
            }
        }
    }

    pub fn next_msg(&self) -> Option<P2pStreamingRpcResponse> {
        match self {
            Self::StagedLedgerParts(v) => v.next_msg().map(Into::into),
        }
    }

    pub fn is_done(&self) -> bool {
        match self {
            Self::StagedLedgerParts(s) => {
                matches!(s, StagedLedgerPartsSendProgress::Success { .. })
            }
        }
    }
}

impl P2pStreamingRpcReceiveProgress {
    pub fn kind(&self) -> P2pStreamingRpcKind {
        match self {
            Self::StagedLedgerParts(_) => P2pStreamingRpcKind::StagedLedgerParts,
        }
    }

    pub fn is_done(&self) -> bool {
        match self {
            Self::StagedLedgerParts(s) => {
                matches!(s, StagedLedgerPartsReceiveProgress::Success { .. })
            }
        }
    }

    pub fn update(&mut self, time: redux::Timestamp, resp: &P2pStreamingRpcResponse) -> bool {
        match (self, resp) {
            (
                Self::StagedLedgerParts(progress),
                P2pStreamingRpcResponse::StagedLedgerParts(resp),
            ) => progress.update(time, resp),
            // _ => false,
        }
    }

    pub fn is_part_pending(&self) -> bool {
        match self {
            Self::StagedLedgerParts(progress) => progress.is_part_pending(),
        }
    }

    pub fn set_next_pending(&mut self, time: redux::Timestamp) -> bool {
        match self {
            Self::StagedLedgerParts(progress) => progress.set_next_pending(time),
        }
    }
}
