use redux::ActionMeta;

use crate::channels::{
    streaming_rpc::{P2pChannelsStreamingRpcAction, StreamingRpcChannelMsg},
    ChannelId, MsgId, P2pChannelsService,
};

use super::P2pChannelsStreamingRpcEffectfulAction;

impl P2pChannelsStreamingRpcEffectfulAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        match self {
            P2pChannelsStreamingRpcEffectfulAction::Init { peer_id } => {
                store
                    .service()
                    .channel_open(peer_id, ChannelId::StreamingRpc);
                store.dispatch(P2pChannelsStreamingRpcAction::Pending { peer_id });
            }
            P2pChannelsStreamingRpcEffectfulAction::RequestSend {
                peer_id,
                id,
                request,
                on_init,
            } => {
                let msg = StreamingRpcChannelMsg::Request(id, *request.clone());
                store
                    .service()
                    .channel_send(peer_id, MsgId::first(), msg.into());
                if let Some(on_init) = on_init {
                    store.dispatch_callback(on_init, (peer_id, id, *request));
                }
            }
            P2pChannelsStreamingRpcEffectfulAction::ResponseNextPartGet { peer_id, id } => {
                let msg = StreamingRpcChannelMsg::Next(id);
                store
                    .service()
                    .channel_send(peer_id, MsgId::first(), msg.into());
            }
            P2pChannelsStreamingRpcEffectfulAction::ResponseSendInit {
                peer_id,
                id,
                response,
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
            P2pChannelsStreamingRpcEffectfulAction::ResponsePartSend {
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
        }
    }
}
