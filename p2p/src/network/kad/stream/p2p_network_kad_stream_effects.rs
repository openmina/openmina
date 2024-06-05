use openmina_core::warn;
use redux::ActionMeta;

use crate::{
    stream::{P2pNetworkKadIncomingStreamError, P2pNetworkKadOutgoingStreamError},
    Data, P2pNetworkKademliaAction, P2pNetworkSchedulerAction, P2pNetworkYamuxAction, YamuxFlags,
};

use super::{
    super::{P2pNetworkKademliaRpcReply, P2pNetworkKademliaRpcRequest},
    P2pNetworkKademliaStreamAction,
};

impl P2pNetworkKademliaStreamAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store) -> Result<(), String>
    where
        Store: crate::P2pStore<S>,
    {
        use super::P2pNetworkKadIncomingStreamState as I;
        use super::P2pNetworkKadOutgoingStreamState as O;
        use super::P2pNetworkKadStreamState as D;
        use P2pNetworkKademliaStreamAction as A;

        if let A::Prune { .. } = self {
            return Ok(());
        }

        let state = store
            .state()
            .network
            .scheduler
            .discovery_state
            .as_ref()
            .ok_or_else(|| String::from("peer discovery not configured"))?
            .find_kad_stream_state(self.peer_id(), self.stream_id())
            .ok_or_else(|| format!("stream not found for action {self:?}"))?;

        match (self, state) {
            (
                A::New { .. },
                D::Outgoing(O::WaitingForRequest { .. }) | D::Incoming(I::WaitingForRequest { .. }),
            ) => Ok(()),
            (
                A::IncomingData {
                    addr,
                    peer_id,
                    stream_id,
                    ..
                },
                D::Incoming(I::RequestIsReady {
                    data: P2pNetworkKademliaRpcRequest::FindNode { key },
                }),
            ) => {
                let key = *key;
                store.dispatch(A::WaitOutgoing {
                    addr,
                    peer_id,
                    stream_id,
                });
                store.dispatch(P2pNetworkKademliaAction::AnswerFindNodeRequest {
                    addr,
                    peer_id,
                    stream_id,
                    key,
                });
                Ok(())
            }
            (
                A::IncomingData {
                    addr,
                    peer_id,
                    stream_id,
                    ..
                },
                D::Outgoing(O::ResponseIsReady {
                    data: P2pNetworkKademliaRpcReply::FindNode { closer_peers },
                }),
            ) => {
                let closest_peers = closer_peers.clone();
                store.dispatch(A::WaitOutgoing {
                    addr,
                    peer_id,
                    stream_id,
                });
                store.dispatch(P2pNetworkKademliaAction::UpdateFindNodeRequest {
                    addr,
                    peer_id,
                    stream_id,
                    closest_peers,
                });
                Ok(())
            }
            (
                A::WaitOutgoing { .. },
                D::Incoming(I::WaitingForReply { .. }) | D::Outgoing(O::WaitingForRequest { .. }),
            ) => Ok(()),
            (
                A::SendRequest {
                    addr,
                    peer_id,
                    stream_id,
                    ..
                }
                | A::SendResponse {
                    addr,
                    peer_id,
                    stream_id,
                    ..
                },
                D::Incoming(I::ResponseBytesAreReady { bytes })
                | D::Outgoing(O::RequestBytesAreReady { bytes }),
            ) => {
                // send data to the network
                store.dispatch(P2pNetworkYamuxAction::OutgoingData {
                    addr,
                    stream_id,
                    data: bytes.clone().into(),
                    flags: Default::default(),
                });
                store.dispatch(A::WaitIncoming {
                    addr,
                    peer_id,
                    stream_id,
                });
                Ok(())
            }
            (
                A::WaitIncoming { .. },
                D::Incoming(I::WaitingForRequest { .. }) | D::Outgoing(O::WaitingForReply),
            ) => Ok(()),
            (
                A::Close {
                    addr, stream_id, ..
                },
                D::Incoming(I::ResponseBytesAreReady { bytes })
                | D::Outgoing(O::RequestBytesAreReady { bytes }),
            ) if bytes.is_empty() => {
                // send FIN to the network
                store.dispatch(P2pNetworkYamuxAction::OutgoingData {
                    addr,
                    stream_id,
                    data: Data(Box::new([0; 0])),
                    flags: YamuxFlags::FIN,
                });
                Ok(())
            }
            (
                A::RemoteClose {
                    addr,
                    peer_id,
                    stream_id,
                },
                D::Incoming(I::Closing),
            ) => {
                // send FIN to the network
                store.dispatch(P2pNetworkYamuxAction::OutgoingData {
                    addr,
                    stream_id,
                    data: Data(Box::new([])),
                    flags: YamuxFlags::FIN,
                });
                store.dispatch(A::Prune {
                    addr,
                    peer_id,
                    stream_id,
                });
                Ok(())
            }
            (
                A::RemoteClose {
                    addr,
                    peer_id,
                    stream_id,
                },
                D::Outgoing(O::Closing),
            ) => {
                store.dispatch(A::Prune {
                    addr,
                    peer_id,
                    stream_id,
                });
                Ok(())
            }
            (action, D::Incoming(I::Error(err)) | D::Outgoing(O::Error(err))) => {
                warn!(meta.time(); summary = "error handling kademlia action", error = display(err));
                let error = match state {
                    D::Incoming(_) => P2pNetworkKadIncomingStreamError::from(err.clone()).into(),
                    D::Outgoing(_) => P2pNetworkKadOutgoingStreamError::from(err.clone()).into(),
                };
                store.dispatch(P2pNetworkSchedulerAction::Error {
                    addr: *action.addr(),
                    error,
                });
                Ok(())
            }
            (action, _) => Err(format!("incorrect state {state:?} for action {action:?}")),
        }
    }
}
