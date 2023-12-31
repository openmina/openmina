use openmina_core::block::BlockWithHash;
use redux::ActionMeta;

use crate::{
    channels::{rpc::P2pRpcRequest, ChannelId, MsgId, P2pChannelsService},
    peer::P2pPeerBestTipUpdateAction,
    P2pNetworkRpcOutgoingQueryAction,
};

use super::{P2pChannelsRpcAction, P2pRpcResponse, RpcChannelMsg};

impl P2pChannelsRpcAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
        P2pPeerBestTipUpdateAction: redux::EnablingCondition<S>,
        Self: redux::EnablingCondition<S>,
        P2pNetworkRpcOutgoingQueryAction: redux::EnablingCondition<S>,
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
                } else {
                    use binprot::BinProtWrite;
                    use mina_p2p_messages::{
                        rpc,
                        rpc_kernel::{NeedsLength, QueryHeader, QueryPayload, RpcMethod},
                    };

                    match request {
                        P2pRpcRequest::BestTipWithProof => {
                            type Method = rpc::GetBestTipV2;
                            type Payload = QueryPayload<<Method as RpcMethod>::Query>;

                            let mut v = vec![];
                            <Payload as BinProtWrite>::binprot_write(&NeedsLength(()), &mut v)
                                .unwrap_or_default();
                            store.dispatch(P2pNetworkRpcOutgoingQueryAction {
                                peer_id,
                                query: QueryHeader {
                                    tag: Method::NAME.into(),
                                    version: Method::VERSION,
                                    id: id as _,
                                },
                                data: v.into(),
                            });
                        }
                        P2pRpcRequest::LedgerQuery(hash, q) => {
                            type Method = rpc::AnswerSyncLedgerQueryV2;
                            type Payload = QueryPayload<<Method as RpcMethod>::Query>;

                            let mut v = vec![];
                            <Payload as BinProtWrite>::binprot_write(
                                &NeedsLength((hash.0.clone(), q)),
                                &mut v,
                            )
                            .unwrap_or_default();
                            store.dispatch(P2pNetworkRpcOutgoingQueryAction {
                                peer_id,
                                query: QueryHeader {
                                    tag: Method::NAME.into(),
                                    version: Method::VERSION,
                                    id: id as _,
                                },
                                data: v.into(),
                            });
                        }
                        P2pRpcRequest::StagedLedgerAuxAndPendingCoinbasesAtBlock(hash) => {
                            type Method = rpc::GetStagedLedgerAuxAndPendingCoinbasesAtHashV2;
                            type Payload = QueryPayload<<Method as RpcMethod>::Query>;

                            let mut v = vec![];
                            <Payload as BinProtWrite>::binprot_write(
                                &NeedsLength(hash.0.clone()),
                                &mut v,
                            )
                            .unwrap_or_default();
                            store.dispatch(P2pNetworkRpcOutgoingQueryAction {
                                peer_id,
                                query: QueryHeader {
                                    tag: Method::NAME.into(),
                                    version: Method::VERSION,
                                    id: id as _,
                                },
                                data: v.into(),
                            });
                        }
                        P2pRpcRequest::Block(hash) => {
                            type Method = rpc::GetTransitionChainV2;
                            type Payload = QueryPayload<<Method as RpcMethod>::Query>;

                            let mut v = vec![];
                            <Payload as BinProtWrite>::binprot_write(
                                &NeedsLength(vec![hash.0.clone()]),
                                &mut v,
                            )
                            .unwrap_or_default();
                            store.dispatch(P2pNetworkRpcOutgoingQueryAction {
                                peer_id,
                                query: QueryHeader {
                                    tag: Method::NAME.into(),
                                    version: Method::VERSION,
                                    id: id as _,
                                },
                                data: v.into(),
                            });
                        }
                        P2pRpcRequest::Snark(hash) => {
                            let _ = hash;
                            // libp2p cannot fulfill this request
                        }
                        P2pRpcRequest::InitialPeers => {
                            type Method = rpc::GetSomeInitialPeersV1ForV2;
                            type Payload = QueryPayload<<Method as RpcMethod>::Query>;

                            let mut v = vec![];
                            <Payload as BinProtWrite>::binprot_write(&NeedsLength(()), &mut v)
                                .unwrap_or_default();
                            store.dispatch(P2pNetworkRpcOutgoingQueryAction {
                                peer_id,
                                query: QueryHeader {
                                    tag: Method::NAME.into(),
                                    version: Method::VERSION,
                                    id: id as _,
                                },
                                data: v.into(),
                            });
                        }
                    }
                }
            }
            P2pChannelsRpcAction::ResponseReceived {
                peer_id, response, ..
            } => {
                if let Some(P2pRpcResponse::BestTipWithProof(resp)) = response {
                    store.dispatch(P2pPeerBestTipUpdateAction {
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
                let msg = RpcChannelMsg::Response(id, response);
                store
                    .service()
                    .channel_send(peer_id, MsgId::first(), msg.into());
            }
            P2pChannelsRpcAction::Pending { .. }
            | P2pChannelsRpcAction::Ready { .. }
            | P2pChannelsRpcAction::Timeout { .. }
            | P2pChannelsRpcAction::RequestReceived { .. } => {}
        }
    }
}
