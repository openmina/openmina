use openmina_core::block::BlockWithHash;

use redux::ActionMeta;

use crate::{channels::P2pChannelsService, peer::P2pPeerAction, P2pNetworkRpcAction};

use super::{P2pChannelsRpcAction, P2pRpcResponse};

// TODO: support for webrtc peers
impl P2pChannelsRpcAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        match self {
            P2pChannelsRpcAction::Init { peer_id } => {
                // TODO: webrtc init?
                // TODO(akoptelov): open a new stream, if we decide not to forcibly do that on connection established
                store.dispatch(P2pChannelsRpcAction::Pending { peer_id });
            }
            P2pChannelsRpcAction::RequestSend {
                peer_id,
                id,
                request,
            } => {
                if let Some((query, data)) = super::internal_request_into_libp2p(request, id) {
                    store.dispatch(P2pNetworkRpcAction::OutgoingQuery {
                        peer_id,
                        query,
                        data,
                    });
                }
            }
            P2pChannelsRpcAction::ResponseReceived {
                peer_id, response, ..
            } => {
                if let Some(P2pRpcResponse::BestTipWithProof(resp)) = response {
                    store.dispatch(P2pPeerAction::BestTipUpdate {
                        peer_id,
                        best_tip: BlockWithHash::new(resp.best_tip.clone()),
                    });
                }
            }
            P2pChannelsRpcAction::ResponseSend {
                peer_id,
                id,
                response,
            } => {
                if let Some(response) = response {
                    if let Some((response, data)) =
                        super::internal_response_into_libp2p(response, id)
                    {
                        store.dispatch(P2pNetworkRpcAction::OutgoingResponse {
                            peer_id,
                            response,
                            data,
                        });
                    }
                }
            }
            P2pChannelsRpcAction::Pending { .. }
            | P2pChannelsRpcAction::Ready { .. }
            | P2pChannelsRpcAction::Timeout { .. }
            | P2pChannelsRpcAction::RequestReceived { .. } => {}
        }
    }
}
