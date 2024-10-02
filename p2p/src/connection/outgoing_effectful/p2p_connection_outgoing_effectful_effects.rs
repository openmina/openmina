use openmina_core::bug_condition;
use redux::ActionMeta;

use crate::{
    connection::{
        outgoing::{
            P2pConnectionOutgoingAction, P2pConnectionOutgoingInitOpts, P2pConnectionOutgoingState,
        },
        P2pConnectionService, P2pConnectionState,
    },
    webrtc, P2pPeerStatus,
};

use super::P2pConnectionOutgoingEffectfulAction;

impl P2pConnectionOutgoingEffectfulAction {
    pub fn effects<Store, S>(self, _meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
    {
        match self {
            P2pConnectionOutgoingEffectfulAction::RandomInit => {
                let peers = store.state().disconnected_peers().collect::<Vec<_>>();
                let picked_peer = store.service().random_pick(&peers);
                if let Some(picked_peer) = picked_peer {
                    store.dispatch(P2pConnectionOutgoingAction::Reconnect {
                        opts: picked_peer,
                        rpc_id: None,
                    });
                } else {
                    bug_condition!("Picked peer is None");
                }
            }
            P2pConnectionOutgoingEffectfulAction::Init { opts, .. } => {
                let peer_id = *opts.peer_id();
                store.service().outgoing_init(opts);
                store.dispatch(P2pConnectionOutgoingAction::OfferSdpCreatePending { peer_id });
            }
            P2pConnectionOutgoingEffectfulAction::OfferSend { peer_id, offer } => {
                let (state, service) = store.state_and_service();
                let Some(peer) = state.peers.get(&peer_id) else {
                    return;
                };
                let P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionOutgoingState::OfferReady { opts, .. },
                )) = &peer.status
                else {
                    return;
                };
                let signaling_method = match opts {
                    P2pConnectionOutgoingInitOpts::WebRTC { signaling, .. } => signaling,
                    #[allow(unreachable_patterns)]
                    _ => return,
                };
                match signaling_method {
                    webrtc::SignalingMethod::Http(_) | webrtc::SignalingMethod::Https(_) => {
                        let Some(url) = signaling_method.http_url() else {
                            return;
                        };
                        service.http_signaling_request(url, *offer);
                    }
                }
                store.dispatch(P2pConnectionOutgoingAction::OfferSendSuccess { peer_id });
            }
            P2pConnectionOutgoingEffectfulAction::AnswerSet { peer_id, answer } => {
                store.service().set_answer(peer_id, *answer);
                store.dispatch(P2pConnectionOutgoingAction::FinalizePending { peer_id });
            }
        }
    }
}
