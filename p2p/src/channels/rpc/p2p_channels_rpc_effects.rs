use openmina_core::block::BlockWithHash;

use redux::ActionMeta;

use crate::{
    channels::{ChannelId, MsgId, P2pChannelsService},
    peer::P2pPeerAction,
    P2pNetworkRpcOutgoingQueryAction, P2pNetworkRpcOutgoingResponseAction,
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
                store.dispatch(P2pChannelsRpcAction::Pending { peer_id });
            }
            P2pChannelsRpcAction::RequestSend {
                peer_id,
                id,
                request,
            } => {
                if cfg!(feature = "p2p-libp2p") {
                    let msg = RpcChannelMsg::Request(id, request);
                    store
                        .service()
                        .channel_send(peer_id, MsgId::first(), msg.into());
                } else if let Some((query, data)) = super::internal_request_into_libp2p(request, id)
                {
                    store.dispatch(P2pNetworkRpcOutgoingQueryAction {
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
                if cfg!(feature = "p2p-libp2p") {
                    let msg = RpcChannelMsg::Response(id, response);
                    store
                        .service()
                        .channel_send(peer_id, MsgId::first(), msg.into());
                } else if let Some(response) = response {
                    if let Some((response, data)) =
                        super::internal_response_into_libp2p(response, id)
                    {
                        store.dispatch(P2pNetworkRpcOutgoingResponseAction {
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
