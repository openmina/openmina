use openmina_core::block::BlockWithHash;
use redux::ActionMeta;

use crate::{
    channels::{rpc::P2pRpcRequest, ChannelId, MsgId, P2pChannelsService},
    peer::P2pPeerBestTipUpdateAction,
    P2pNetworkRpcOutgoingQueryAction,
};

use super::{
    P2pChannelsRpcInitAction, P2pChannelsRpcPendingAction, P2pChannelsRpcRequestSendAction,
    P2pChannelsRpcResponseReceivedAction, P2pChannelsRpcResponseSendAction, P2pRpcResponse,
    RpcChannelMsg,
};

impl P2pChannelsRpcInitAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
        P2pChannelsRpcPendingAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        store.service().channel_open(peer_id, ChannelId::Rpc);
        store.dispatch(P2pChannelsRpcPendingAction { peer_id });
    }
}

impl P2pChannelsRpcRequestSendAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
        P2pNetworkRpcOutgoingQueryAction: redux::EnablingCondition<S>,
    {
        if cfg!(feature = "p2p-libp2p") {
            let msg = RpcChannelMsg::Request(self.id, self.request);
            store
                .service()
                .channel_send(self.peer_id, MsgId::first(), msg.into());
        } else {
            use binprot::BinProtWrite;
            use mina_p2p_messages::{
                rpc,
                rpc_kernel::{NeedsLength, QueryHeader, QueryPayload, RpcMethod},
            };

            match self.request {
                P2pRpcRequest::BestTipWithProof => {
                    type Payload = QueryPayload<<rpc::GetBestTipV2 as RpcMethod>::Query>;

                    let mut v = vec![];
                    <Payload as BinProtWrite>::binprot_write(&NeedsLength(()), &mut v)
                        .unwrap_or_default();
                    store.dispatch(P2pNetworkRpcOutgoingQueryAction {
                        peer_id: self.peer_id,
                        query: QueryHeader {
                            tag: rpc::GetBestTipV2::NAME.into(),
                            version: rpc::GetBestTipV2::VERSION,
                            id: self.id as _,
                        },
                        data: v.into(),
                    });
                }
                P2pRpcRequest::LedgerQuery(hash, q) => {
                    type Payload = QueryPayload<<rpc::AnswerSyncLedgerQueryV2 as RpcMethod>::Query>;

                    let mut v = vec![];
                    <Payload as BinProtWrite>::binprot_write(
                        &NeedsLength((hash.0.clone(), q)),
                        &mut v,
                    )
                    .unwrap_or_default();
                    store.dispatch(P2pNetworkRpcOutgoingQueryAction {
                        peer_id: self.peer_id,
                        query: QueryHeader {
                            tag: rpc::AnswerSyncLedgerQueryV2::NAME.into(),
                            version: rpc::AnswerSyncLedgerQueryV2::VERSION,
                            id: self.id as _,
                        },
                        data: v.into(),
                    });
                }
                _ => {}
            }
        }
    }
}

impl P2pChannelsRpcResponseReceivedAction {
    pub fn effects<Store, S>(&self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        P2pPeerBestTipUpdateAction: redux::EnablingCondition<S>,
    {
        match &self.response {
            Some(P2pRpcResponse::BestTipWithProof(resp)) => {
                store.dispatch(P2pPeerBestTipUpdateAction {
                    peer_id: self.peer_id,
                    best_tip: BlockWithHash::new(resp.best_tip.clone()),
                });
            }
            _ => {}
        }
    }
}

impl P2pChannelsRpcResponseSendAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        let msg = RpcChannelMsg::Response(self.id, self.response);
        store
            .service()
            .channel_send(self.peer_id, MsgId::first(), msg.into());
    }
}
