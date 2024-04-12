use super::{super::*, *};

use super::p2p_network_pnet_state::Half;

impl P2pNetworkPnetAction {
    pub fn effects<Store, S>(self, _meta: &redux::ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pMioService,
    {
        let (state, service) = store.state_and_service();
        let connections = &state.network.scheduler.connections;
        let Some(state) = connections.get(self.addr()) else {
            return;
        };
        let state = &state.pnet;
        match self {
            P2pNetworkPnetAction::IncomingData { addr, .. } => match &state.incoming {
                Half::Done { to_send, .. } if !to_send.is_empty() => {
                    let data = to_send.clone().into();
                    store.dispatch(P2pNetworkSelectAction::IncomingData {
                        addr,
                        kind: SelectKind::Authentication,
                        data,
                        fin: false,
                    });
                }
                _ => {}
            },
            P2pNetworkPnetAction::OutgoingData { addr, .. } => match &state.outgoing {
                Half::Done { to_send, .. } if !to_send.is_empty() => {
                    service.send_mio_cmd(crate::MioCmd::Send(
                        addr,
                        to_send.clone().into_boxed_slice(),
                    ));
                }
                _ => {}
            },
            P2pNetworkPnetAction::SetupNonce {
                addr,
                nonce,
                incoming,
            } => {
                service.send_mio_cmd(crate::MioCmd::Send(addr, nonce.to_vec().into_boxed_slice()));
                store.dispatch(P2pNetworkSelectAction::Init {
                    addr,
                    kind: SelectKind::Authentication,
                    incoming,
                    send_handshake: true,
                });
            }
        }
    }
}
