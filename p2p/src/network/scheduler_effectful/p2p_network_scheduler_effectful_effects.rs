use redux::ActionMeta;
use std::net::SocketAddr;

use crate::{MioCmd, P2pCryptoService, P2pMioService};

use super::{super::*, *};

impl P2pNetworkSchedulerEffectfulAction {
    pub fn effects<Store, S>(self, _meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pMioService + P2pCryptoService,
    {
        match self {
            P2pNetworkSchedulerEffectfulAction::InterfaceDetected { ip, port } => {
                store
                    .service()
                    .send_mio_cmd(MioCmd::ListenOn(SocketAddr::new(ip, port)));
            }
            P2pNetworkSchedulerEffectfulAction::IncomingConnectionIsReady { listener, .. } => {
                let state = store.state();
                if state.network.scheduler.connections.len()
                    >= state.config.limits.max_connections()
                {
                    store.service().send_mio_cmd(MioCmd::Refuse(listener));
                } else {
                    store.service().send_mio_cmd(MioCmd::Accept(listener));
                }
            }
            P2pNetworkSchedulerEffectfulAction::IncomingDidAccept { addr, .. } => {
                let nonce = store.service().generate_random_nonce();
                store.dispatch(P2pNetworkPnetAction::SetupNonce {
                    addr,
                    nonce: nonce.to_vec().into(),
                    incoming: true,
                });
            }
            P2pNetworkSchedulerEffectfulAction::OutgoingConnect { addr } => {
                store.service().send_mio_cmd(MioCmd::Connect(addr));
            }
            P2pNetworkSchedulerEffectfulAction::OutgoingDidConnect { addr } => {
                let nonce = store.service().generate_random_nonce();
                store.dispatch(P2pNetworkPnetAction::SetupNonce {
                    addr,
                    nonce: nonce.to_vec().into(),
                    incoming: false,
                });
            }
            P2pNetworkSchedulerEffectfulAction::IncomingDataIsReady { addr, limit } => {
                store
                    .service()
                    .send_mio_cmd(MioCmd::Recv(addr, vec![0; limit].into_boxed_slice()));
            }
            P2pNetworkSchedulerEffectfulAction::NoiseSelectDone { addr, incoming } => {
                let ephemeral_sk = Sk::from_random(store.service().ephemeral_sk());
                let static_sk = Sk::from_random(store.service().static_sk());
                let static_pk = static_sk.pk();

                let signature = store.service().sign_key(static_pk.0.as_bytes()).into();

                store.dispatch(P2pNetworkNoiseAction::Init {
                    addr,
                    incoming,
                    ephemeral_sk,
                    static_sk,
                    signature,
                });
            }
            // TODO: remove state access
            P2pNetworkSchedulerEffectfulAction::SelectError { addr, .. }
            | P2pNetworkSchedulerEffectfulAction::Disconnect { addr, .. }
            | P2pNetworkSchedulerEffectfulAction::Error { addr, .. } => {
                if let Some(conn_state) = store.state().network.scheduler.connections.get(&addr) {
                    if let Some(reason) = conn_state.closed.clone() {
                        store.service().send_mio_cmd(MioCmd::Disconnect(addr));
                        store.dispatch(P2pNetworkSchedulerAction::Disconnected { addr, reason });
                    }
                }
            }
        }
    }
}
