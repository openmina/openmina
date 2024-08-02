use openmina_core::{fuzzed_maybe, warn};
use redux::ActionMeta;

use crate::{
    stream::{
        P2pNetworkKadIncomingStreamError, P2pNetworkKadOutgoingStreamError,
        P2pNetworkKadStreamState,
    },
    Data, P2pNetworkKademliaAction, P2pNetworkSchedulerAction, P2pNetworkYamuxAction, YamuxFlags,
};

use super::{
    super::{P2pNetworkKademliaRpcReply, P2pNetworkKademliaRpcRequest},
    P2pNetworkKadIncomingStreamState, P2pNetworkKadOutgoingStreamState,
    P2pNetworkKademliaStreamAction,
};

impl P2pNetworkKademliaStreamAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store) -> Result<(), String>
    where
        Store: crate::P2pStore<S>,
    {
        if let P2pNetworkKademliaStreamAction::Prune { .. } = self {
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
                P2pNetworkKademliaStreamAction::New { .. },
                P2pNetworkKadStreamState::Outgoing(
                    P2pNetworkKadOutgoingStreamState::WaitingForRequest { .. },
                )
                | P2pNetworkKadStreamState::Incoming(
                    P2pNetworkKadIncomingStreamState::WaitingForRequest { .. },
                ),
            ) => Ok(()),
            (
                P2pNetworkKademliaStreamAction::IncomingData {
                    addr,
                    peer_id,
                    stream_id,
                    ..
                },
                P2pNetworkKadStreamState::Incoming(
                    P2pNetworkKadIncomingStreamState::RequestIsReady {
                        data: P2pNetworkKademliaRpcRequest::FindNode { key },
                    },
                ),
            ) => {
                let key = *key;
                store.dispatch(P2pNetworkKademliaStreamAction::WaitOutgoing {
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
                P2pNetworkKademliaStreamAction::IncomingData {
                    addr,
                    peer_id,
                    stream_id,
                    ..
                },
                P2pNetworkKadStreamState::Outgoing(
                    P2pNetworkKadOutgoingStreamState::ResponseIsReady {
                        data: P2pNetworkKademliaRpcReply::FindNode { closer_peers },
                    },
                ),
            ) => {
                let closest_peers = closer_peers.clone();
                store.dispatch(P2pNetworkKademliaStreamAction::WaitOutgoing {
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
                P2pNetworkKademliaStreamAction::WaitOutgoing { .. },
                P2pNetworkKadStreamState::Incoming(
                    P2pNetworkKadIncomingStreamState::WaitingForReply { .. },
                )
                | P2pNetworkKadStreamState::Outgoing(
                    P2pNetworkKadOutgoingStreamState::WaitingForRequest { .. },
                ),
            ) => Ok(()),
            (
                P2pNetworkKademliaStreamAction::SendRequest {
                    addr,
                    peer_id,
                    stream_id,
                    ..
                }
                | P2pNetworkKademliaStreamAction::SendResponse {
                    addr,
                    peer_id,
                    stream_id,
                    ..
                },
                P2pNetworkKadStreamState::Incoming(
                    P2pNetworkKadIncomingStreamState::ResponseBytesAreReady { bytes },
                )
                | P2pNetworkKadStreamState::Outgoing(
                    P2pNetworkKadOutgoingStreamState::RequestBytesAreReady { bytes },
                ),
            ) => {
                // send data to the network
                let data = fuzzed_maybe!(bytes.clone().into(), crate::fuzzer::mutate_kad_data);
                let flags = fuzzed_maybe!(Default::default(), crate::fuzzer::mutate_yamux_flags);

                store.dispatch(P2pNetworkYamuxAction::OutgoingData {
                    addr,
                    stream_id,
                    data,
                    flags,
                });
                store.dispatch(P2pNetworkKademliaStreamAction::WaitIncoming {
                    addr,
                    peer_id,
                    stream_id,
                });
                Ok(())
            }
            (
                P2pNetworkKademliaStreamAction::WaitIncoming { .. },
                P2pNetworkKadStreamState::Incoming(
                    P2pNetworkKadIncomingStreamState::WaitingForRequest { .. },
                )
                | P2pNetworkKadStreamState::Outgoing(
                    P2pNetworkKadOutgoingStreamState::WaitingForReply,
                ),
            ) => Ok(()),
            (
                P2pNetworkKademliaStreamAction::Close {
                    addr, stream_id, ..
                },
                P2pNetworkKadStreamState::Incoming(
                    P2pNetworkKadIncomingStreamState::ResponseBytesAreReady { bytes },
                )
                | P2pNetworkKadStreamState::Outgoing(
                    P2pNetworkKadOutgoingStreamState::RequestBytesAreReady { bytes },
                ),
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
                P2pNetworkKademliaStreamAction::RemoteClose {
                    addr,
                    peer_id,
                    stream_id,
                },
                P2pNetworkKadStreamState::Incoming(P2pNetworkKadIncomingStreamState::Closing),
            ) => {
                // send FIN to the network
                store.dispatch(P2pNetworkYamuxAction::OutgoingData {
                    addr,
                    stream_id,
                    data: Data(Box::new([])),
                    flags: YamuxFlags::FIN,
                });
                store.dispatch(P2pNetworkKademliaStreamAction::Prune {
                    addr,
                    peer_id,
                    stream_id,
                });
                Ok(())
            }
            (
                P2pNetworkKademliaStreamAction::RemoteClose {
                    addr,
                    peer_id,
                    stream_id,
                },
                P2pNetworkKadStreamState::Outgoing(P2pNetworkKadOutgoingStreamState::Closing),
            ) => {
                store.dispatch(P2pNetworkKademliaStreamAction::Prune {
                    addr,
                    peer_id,
                    stream_id,
                });
                Ok(())
            }
            (
                action,
                P2pNetworkKadStreamState::Incoming(P2pNetworkKadIncomingStreamState::Error(err))
                | P2pNetworkKadStreamState::Outgoing(P2pNetworkKadOutgoingStreamState::Error(err)),
            ) => {
                warn!(meta.time(); summary = "error handling kademlia action", error = display(err));
                let error = match state {
                    P2pNetworkKadStreamState::Incoming(_) => {
                        P2pNetworkKadIncomingStreamError::from(err.clone()).into()
                    }
                    P2pNetworkKadStreamState::Outgoing(_) => {
                        P2pNetworkKadOutgoingStreamError::from(err.clone()).into()
                    }
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
