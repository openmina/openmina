use redux::ActionMeta;

use crate::channels::{ChannelId, MsgId, P2pChannelsService};

use super::{P2pChannelsStreamingRpcAction, StreamingRpcChannelMsg};

impl P2pChannelsStreamingRpcAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        match self {
            P2pChannelsStreamingRpcAction::Init { peer_id } => {
                store
                    .service()
                    .channel_open(peer_id, ChannelId::StreamingRpc);
                store.dispatch(P2pChannelsStreamingRpcAction::Pending { peer_id });
            }
            P2pChannelsStreamingRpcAction::RequestSend {
                peer_id,
                id,
                request,
            } => {
                let msg = StreamingRpcChannelMsg::Request(id, *request);
                store
                    .service()
                    .channel_send(peer_id, MsgId::first(), msg.into());
            }
            P2pChannelsStreamingRpcAction::ResponseNextPartGet { peer_id, id } => {
                let msg = StreamingRpcChannelMsg::Next(id);
                store
                    .service()
                    .channel_send(peer_id, MsgId::first(), msg.into());
            }
            P2pChannelsStreamingRpcAction::ResponsePartReceived { peer_id, id, .. } => {
                let Some(peer) = store.state().get_ready_peer(&peer_id) else {
                    return;
                };

                if let Some(response) = peer.channels.streaming_rpc.local_done_response() {
                    store.dispatch(P2pChannelsStreamingRpcAction::ResponseReceived {
                        peer_id,
                        id,
                        response: Some(response),
                    });
                    return;
                }
                store.dispatch(P2pChannelsStreamingRpcAction::ResponseNextPartGet { peer_id, id });
            }
            P2pChannelsStreamingRpcAction::ResponseReceived { .. } => {}
            P2pChannelsStreamingRpcAction::RequestReceived { .. } => {}
            P2pChannelsStreamingRpcAction::ResponsePending { .. } => {}
            P2pChannelsStreamingRpcAction::ResponseSendInit {
                peer_id,
                id,
                response,
                ..
            } => {
                if response.is_none() {
                    let msg = StreamingRpcChannelMsg::Response(id, None);
                    store
                        .service()
                        .channel_send(peer_id, MsgId::first(), msg.into());

                    store.dispatch(P2pChannelsStreamingRpcAction::ResponseSent { peer_id, id });
                    return;
                }
                store.dispatch(P2pChannelsStreamingRpcAction::ResponsePartNextSend { peer_id, id });
            }
            P2pChannelsStreamingRpcAction::ResponsePartNextSend { peer_id, id } => {
                let Some(response) = None.or_else(|| {
                    let peer = store.state().get_ready_peer(&peer_id)?;
                    peer.channels.streaming_rpc.remote_next_msg().map(Box::new)
                }) else {
                    return;
                };

                store.dispatch(P2pChannelsStreamingRpcAction::ResponsePartSend {
                    peer_id,
                    id,
                    response,
                });
            }
            P2pChannelsStreamingRpcAction::ResponsePartSend {
                peer_id,
                id,
                response,
            } => {
                let msg = StreamingRpcChannelMsg::Response(id, Some(*response));
                store
                    .service()
                    .channel_send(peer_id, MsgId::first(), msg.into());
                store.dispatch(P2pChannelsStreamingRpcAction::ResponseSent { peer_id, id });
            }
            P2pChannelsStreamingRpcAction::ResponseSent { .. } => {}
            P2pChannelsStreamingRpcAction::Pending { .. }
            | P2pChannelsStreamingRpcAction::Ready { .. }
            | P2pChannelsStreamingRpcAction::Timeout { .. } => {}
        }
    }
}
