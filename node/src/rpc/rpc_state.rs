use std::collections::BTreeMap;

use mina_p2p_messages::v2;
use openmina_core::block::ArcBlockWithHash;
use serde::{Deserialize, Serialize};

use super::{RpcId, RpcRequest};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcRequestState {
    pub req: RpcRequest,
    pub status: RpcRequestStatus,
    /// Extra data for the request.
    pub data: RpcRequestExtraData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RpcRequestStatus {
    Init {
        time: redux::Timestamp,
    },
    Pending {
        time: redux::Timestamp,
    },
    Error {
        time: redux::Timestamp,
        error: String,
    },
    Success {
        time: redux::Timestamp,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RpcRequestExtraData {
    None,
    FullBlockOpt(Option<ArcBlockWithHash>),
}

impl RpcRequestStatus {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcState {
    pub requests: BTreeMap<RpcId, RpcRequestState>,
}

impl RpcState {
    pub fn new() -> Self {
        Self {
            requests: Default::default(),
        }
    }

    pub fn scan_state_summary_rpc_ids(
        &self,
    ) -> impl Iterator<Item = (RpcId, &v2::LedgerHash, &RpcRequestStatus)> {
        self.requests
            .iter()
            .filter(|(_, req)| matches!(req.req, RpcRequest::ScanStateSummaryGet(_)))
            .filter_map(|(id, req)| {
                let block = match &req.data {
                    RpcRequestExtraData::FullBlockOpt(block) => block.as_ref()?,
                    _ => return None,
                };
                Some((*id, block.staged_ledger_hash(), &req.status))
            })
    }
}

impl Default for RpcRequestExtraData {
    fn default() -> Self {
        Self::None
    }
}
