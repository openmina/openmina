use openmina_core::requests::{PendingRequests, RequestId, RequestIdType};
use serde::{Deserialize, Serialize};

use super::{LedgerReadRequest, LedgerReadResponse};

const MAX_TOTAL_COST: usize = 256;

pub struct LedgerReadIdType;
impl RequestIdType for LedgerReadIdType {
    fn request_id_type() -> &'static str {
        "LedgerReadId"
    }
}

pub type LedgerReadId = RequestId<LedgerReadIdType>;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct LedgerReadState {
    pending: PendingRequests<LedgerReadIdType, LedgerReadRequestState>,
    /// Total cost of currently pending requests.
    total_cost: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LedgerReadRequestState {
    Pending {
        time: redux::Timestamp,
        request: LedgerReadRequest,
    },
    Success {
        time: redux::Timestamp,
        request: LedgerReadRequest,
        response: LedgerReadResponse,
    },
}

impl LedgerReadState {
    pub fn contains(&self, id: LedgerReadId) -> bool {
        self.pending.contains(id)
    }

    pub fn get(&self, id: LedgerReadId) -> Option<&LedgerReadRequestState> {
        self.pending.get(id)
    }

    pub fn get_mut(&mut self, id: LedgerReadId) -> Option<&mut LedgerReadRequestState> {
        self.pending.get_mut(id)
    }

    pub fn is_total_cost_under_limit(&self) -> bool {
        self.total_cost < MAX_TOTAL_COST
    }

    pub fn next_req_id(&self) -> LedgerReadId {
        self.pending.next_req_id()
    }

    pub fn add(&mut self, time: redux::Timestamp, request: LedgerReadRequest) -> LedgerReadId {
        self.total_cost = self.total_cost.saturating_add(request.cost());
        self.pending
            .add(LedgerReadRequestState::Pending { time, request })
    }

    pub fn remove(&mut self, id: LedgerReadId) -> Option<LedgerReadRequestState> {
        let req = self.pending.remove(id)?;
        self.total_cost = self.total_cost.saturating_sub(req.request().cost());
        Some(req)
    }

    pub fn add_response(
        &mut self,
        id: LedgerReadId,
        time: redux::Timestamp,
        response: LedgerReadResponse,
    ) {
        self.pending.update(id, move |req| match req {
            LedgerReadRequestState::Pending { request, .. } => LedgerReadRequestState::Success {
                time,
                request,
                response,
            },
            LedgerReadRequestState::Success { .. } => {
                unreachable!("must be prevented by enabling condition")
            }
        });
    }

    pub fn has_same_request(&self, req: &LedgerReadRequest) -> bool {
        self.pending
            .iter()
            .any(|(_, pending)| pending.request() == req)
    }
}

impl LedgerReadRequestState {
    pub fn request(&self) -> &LedgerReadRequest {
        match self {
            Self::Pending { request, .. } | Self::Success { request, .. } => request,
        }
    }
}
