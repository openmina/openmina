use openmina_core::block::BlockWithHash;

use redux::ActionMeta;

#[cfg(feature = "p2p-libp2p")]
use crate::P2pNetworkRpcAction;
use crate::{
    channels::{ChannelId, MsgId, P2pChannelsService},
    peer::P2pPeerAction,
};

use super::{P2pChannelsRpcAction, P2pRpcResponse, RpcChannelMsg};

impl P2pChannelsRpcAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        match self {
            P2pChannelsRpcAction::Init { peer_id } => {
                store.service().channel_open(peer_id, ChannelId::Rpc);
                // TODO(akoptelov): open a new stream, if we decide not to forcibly do that on connection established
                store.dispatch(P2pChannelsRpcAction::Pending { peer_id });
            }
            P2pChannelsRpcAction::RequestSend {
                peer_id,
                id,
                request,
                on_init,
            } => {
                #[cfg(feature = "p2p-libp2p")]
                if store.state().is_libp2p_peer(&peer_id) {
                    if let Some((query, data)) =
                        super::libp2p::internal_request_into_libp2p(*request, id)
                    {
                        store.dispatch(P2pNetworkRpcAction::OutgoingQuery {
                            peer_id,
                            query,
                            data,
                        });
                    }
                    if let Some(on_init) = on_init {
                        store.dispatch_callback(on_init, (peer_id, id));
                    }
                    return;
                }

                let msg = RpcChannelMsg::Request(id, *request);
                store
                    .service()
                    .channel_send(peer_id, MsgId::first(), msg.into());
                if let Some(on_init) = on_init {
                    store.dispatch_callback(on_init, (peer_id, id));
                }
            }
            P2pChannelsRpcAction::ResponseReceived {
                peer_id, response, ..
            } => {
                if let Some(P2pRpcResponse::BestTipWithProof(resp)) = response.as_deref() {
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
                #[cfg(feature = "p2p-libp2p")]
                if store.state().is_libp2p_peer(&peer_id) {
                    if let Some(response) = response {
                        if let Some((response, data)) =
                            super::libp2p::internal_response_into_libp2p(*response, id)
                        {
                            store.dispatch(P2pNetworkRpcAction::OutgoingResponse {
                                peer_id,
                                response,
                                data,
                            });
                        }
                    }
                    return;
                }
                let msg = RpcChannelMsg::Response(id, response.map(|v| *v));
                store
                    .service()
                    .channel_send(peer_id, MsgId::first(), msg.into());
            }
            P2pChannelsRpcAction::Pending { .. }
            | P2pChannelsRpcAction::Ready { .. }
            | P2pChannelsRpcAction::Timeout { .. }
            | P2pChannelsRpcAction::RequestReceived { .. }
            | P2pChannelsRpcAction::ResponsePending { .. } => {}
        }
    }
}
