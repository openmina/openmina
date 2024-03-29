use openmina_core::warn;
use redux::ActionMeta;

use crate::{Data, P2pNetworkKademliaAction, P2pNetworkYamuxAction};

use super::{
    super::{P2pNetworkKademliaRpcReply, P2pNetworkKademliaRpcRequest},
    P2pNetworkKadStreamKind, P2pNetworkKademliaStreamAction,
};

impl P2pNetworkKademliaStreamAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store) -> Result<(), String>
    where
        Store: crate::P2pStore<S>,
    {
        use super::P2pNetworkKadStreamState as S;
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
                S::WaitingOutgoing {
                    kind: P2pNetworkKadStreamKind::Outgoing,
                    ..
                }
                | S::WaitingIncoming {
                    kind: P2pNetworkKadStreamKind::Incoming,
                    ..
                },
            ) => Ok(()),
            (
                A::IncomingData {
                    addr,
                    peer_id,
                    stream_id,
                    ..
                },
                S::IncomingRequest {
                    data: P2pNetworkKademliaRpcRequest::FindNode { key },
                },
            ) => {
                store.dispatch(P2pNetworkKademliaAction::AnswerFindNodeRequest {
                    addr,
                    peer_id,
                    stream_id,
                    key: key.clone(),
                });
                store.dispatch(A::WaitOutgoing {
                    addr,
                    peer_id,
                    stream_id,
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
                S::IncomingReply {
                    data: P2pNetworkKademliaRpcReply::FindNode { closer_peers },
                },
            ) => {
                store.dispatch(P2pNetworkKademliaAction::UpdateFindNodeRequest {
                    addr,
                    peer_id,
                    stream_id,
                    closest_peers: closer_peers.clone(),
                });
                store.dispatch(A::WaitOutgoing {
                    addr,
                    peer_id,
                    stream_id,
                });
                Ok(())
            }
            (A::WaitOutgoing { .. }, S::WaitingOutgoing { .. }) => Ok(()),
            (
                A::SendRequest {
                    addr,
                    peer_id,
                    stream_id,
                    ..
                }
                | A::SendReply {
                    addr,
                    peer_id,
                    stream_id,
                    ..
                },
                S::OutgoingBytes { bytes, .. },
            ) => {
                // send data to the network
                store.dispatch(P2pNetworkYamuxAction::OutgoingData {
                    addr,
                    stream_id,
                    data: bytes.clone().into(),
                    fin: false,
                });
                store.dispatch(A::WaitIncoming {
                    addr,
                    peer_id,
                    stream_id,
                });
                Ok(())
            }
            (A::WaitIncoming { .. }, S::WaitingIncoming { .. }) => Ok(()),
            (
                A::Close {
                    addr, stream_id, ..
                },
                S::OutgoingBytes { .. },
            ) => {
                // send FIN to the network
                store.dispatch(P2pNetworkYamuxAction::OutgoingData {
                    addr,
                    stream_id,
                    data: Data(Box::new([0; 0])),
                    fin: true,
                });
                Ok(())
            }
            (
                A::RemoteClose {
                    addr,
                    peer_id,
                    stream_id,
                },
                S::WaitingIncoming {
                    kind: P2pNetworkKadStreamKind::Incoming,
                    ..
                },
            ) => {
                // send FIN to the network
                store.dispatch(P2pNetworkYamuxAction::OutgoingData {
                    addr,
                    stream_id,
                    data: Data(Box::new([])),
                    fin: true,
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
                S::WaitingIncoming {
                    kind: P2pNetworkKadStreamKind::Outgoing,
                    ..
                },
            ) => {
                store.dispatch(A::Prune {
                    addr,
                    peer_id,
                    stream_id,
                });
                Ok(())
            }
            (action, S::Error(err)) => {
                warn!(meta.time(); summary = "error handling kademlia action", error = err, action = format!("{action:?}"));
                Ok(())
            }
            (action, _) => Err(format!("incorrect state {state:?} for action {action:?}")),
        }
    }
}
