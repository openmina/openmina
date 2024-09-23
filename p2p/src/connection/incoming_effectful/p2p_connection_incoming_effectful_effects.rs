use redux::ActionMeta;

use super::P2pConnectionIncomingEffectfulAction;
use crate::connection::{incoming::P2pConnectionIncomingAction, P2pConnectionService};

impl P2pConnectionIncomingEffectfulAction {
    pub fn effects<Store, S>(self, _meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
    {
        match self {
            P2pConnectionIncomingEffectfulAction::Init { opts } => {
                let peer_id = opts.peer_id;
                store.service().incoming_init(peer_id, *opts.offer);
                store.dispatch(P2pConnectionIncomingAction::AnswerSdpCreatePending { peer_id });
            }
            P2pConnectionIncomingEffectfulAction::AnswerReady { peer_id, answer } => {
                store.service().set_answer(peer_id, *answer);
            }
        }
    }
}
