use super::P2pChannelsRpcEffectfulAction;
use crate::channels::{
    rpc::{P2pChannelsRpcAction, RpcChannelMsg},
    ChannelId, MsgId, P2pChannelsService,
};
use redux::ActionMeta;

impl P2pChannelsRpcEffectfulAction {
    pub fn effects<Store, S>(self, _meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        match self {
            P2pChannelsRpcEffectfulAction::Init { peer_id } => {
                store.service().channel_open(peer_id, ChannelId::Rpc);
                // TODO(akoptelov): open a new stream, if we decide not to forcibly do that on connection established
                store.dispatch(P2pChannelsRpcAction::Pending { peer_id });
            }
            P2pChannelsRpcEffectfulAction::RequestSend {
                peer_id,
                id,
                request,
                on_init,
            } => {
                let msg = RpcChannelMsg::Request(id, *request.clone());
                store
                    .service()
                    .channel_send(peer_id, MsgId::first(), msg.into());

                if let Some(on_init) = on_init {
                    store.dispatch_callback(on_init, (peer_id, id, *request));
                }
            }
            P2pChannelsRpcEffectfulAction::ResponseSend {
                peer_id,
                id,
                response,
            } => {
                let msg = RpcChannelMsg::Response(id, response.map(|v| *v));
                store
                    .service()
                    .channel_send(peer_id, MsgId::first(), msg.into());
            }
        }
    }
}
