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
            P2pNetworkSchedulerEffectfulAction::IncomingConnectionIsReady {
                listener,
                should_accept,
            } => {
                if should_accept {
                    store.service().send_mio_cmd(MioCmd::Accept(listener));
                } else {
                    store.service().send_mio_cmd(MioCmd::Refuse(listener));
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
                store.service().send_mio_cmd(MioCmd::Recv(addr, limit));
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
            P2pNetworkSchedulerEffectfulAction::Disconnect { addr, reason } => {
                store.service().send_mio_cmd(MioCmd::Disconnect(addr));
                store.dispatch(P2pNetworkSchedulerAction::Disconnected { addr, reason });
            }
        }
    }
}
