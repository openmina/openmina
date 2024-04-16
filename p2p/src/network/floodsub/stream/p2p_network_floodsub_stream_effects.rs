use super::P2pNetworkFloodsubStreamAction;
use crate::{Data, P2pNetworkYamuxAction};
use openmina_core::warn;
use redux::ActionMeta;

impl P2pNetworkFloodsubStreamAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store) -> Result<(), String>
    where
        Store: crate::P2pStore<S>,
    {
        use super::P2pNetworkFloodsubStreamState as S;
        use P2pNetworkFloodsubStreamAction as A;

        if let A::Prune { .. } = self {
            return Ok(());
        }

        let state = store
            .state()
            .network
            .scheduler
            .floodsub_state
            .find_floodsub_stream_state(self.peer_id(), self.stream_id())
            .ok_or_else(|| format!("stream not found for action {self:?}"))?;

        println!("FloodsubStreamAction effects state: {:?}", state);
        match self {
            A::New { incoming: true, .. } => {
                if let S::WaitForInput = state {
                    Ok(())
                } else {
                    unreachable!()
                }
            }
            A::New {
                incoming: false, ..
            } => {
                if let S::SendSubscriptions = state {
                    println!("=== TODO: should send subscriptions");
                    Ok(()) // TODO
                } else {
                    unreachable!()
                }
            }
            A::IncomingData {
                addr,
                peer_id,
                stream_id,
                ..
            } => match state {
                S::IncomingPartialData { .. } => Ok(()),
                S::MessageReceived { data } => {
                    println!("FLOODSUB recv: {:?}", data);
                    // store.dispatch(P2pFloodsubAction::UpdatePeerInformation {
                    //     peer_id,
                    //     info: data.clone(),
                    // });
                    Ok(())
                }
                S::Error(err) => {
                    warn!(meta.time(); summary = "error handling Floodsub action", error = err, action = format!("{self:?}"));
                    Ok(())
                }
                _ => unimplemented!(),
            },
            A::Close {
                addr,
                peer_id,
                stream_id,
            } => {
                match state {
                    S::WaitForInput
                    | S::IncomingPartialData { .. }
                    | S::MessageReceived { .. }
                    | S::SendSubscriptions => {
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
                    _ => Err(format!("incorrect state {state:?} for action {self:?}")),
                }
            }
            A::RemoteClose {
                addr,
                peer_id,
                stream_id,
            } => {
                match state {
                    S::WaitForInput
                    | S::IncomingPartialData { .. }
                    | S::MessageReceived { .. }
                    | S::SendSubscriptions => {
                        println!("FLOODSUB: RemoteClose {:?} {:?}", peer_id, stream_id);
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
                    _ => Err(format!("incorrect state {state:?} for action {self:?}")),
                }
            }
            A::Prune { .. } => unreachable!(), // handled before match
        }
    }
}
