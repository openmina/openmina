use super::{LedgerReadAction, LedgerReadActionWithMetaRef, LedgerReadState};

impl LedgerReadState {
    pub fn reducer(&mut self, action: LedgerReadActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            LedgerReadAction::FindTodos => {}
            LedgerReadAction::Init { .. } => {}
            LedgerReadAction::Pending { request, .. } => {
                self.add(meta.time(), request.clone());
            }
            LedgerReadAction::Success { id, response } => {
                self.add_response(*id, meta.time(), response.clone());
            }
            LedgerReadAction::Prune { id } => {
                self.remove(*id);
            }
        }
    }
}
