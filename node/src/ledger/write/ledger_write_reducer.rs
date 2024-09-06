use super::{LedgerWriteAction, LedgerWriteActionWithMetaRef, LedgerWriteState};

impl LedgerWriteState {
    pub fn reducer(&mut self, action: LedgerWriteActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            LedgerWriteAction::Init {
                request,
                on_init: _,
            } => {
                *self = Self::Init {
                    time: meta.time(),
                    request: request.clone(),
                };
            }
            LedgerWriteAction::Pending => {
                if let Self::Init { request, .. } = self {
                    *self = Self::Pending {
                        time: meta.time(),
                        request: request.clone(),
                    };
                }
            }
            LedgerWriteAction::Success { response } => {
                if let Self::Pending { request, .. } = self {
                    *self = Self::Success {
                        time: meta.time(),
                        request: request.clone(),
                        response: response.clone(),
                    };
                }
            }
        }
    }
}
