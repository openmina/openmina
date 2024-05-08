use std::net::SocketAddr;

use multiaddr::Multiaddr;
use openmina_core::{error, log::system_time, warn};
use quick_protobuf::{MessageWrite, Writer};
use redux::ActionMeta;

use super::{
    super::{Identify, P2pNetworkIdentify},
    P2pNetworkIdentifyStreamAction,
};
use crate::{identify::P2pIdentifyAction, token, Data, P2pNetworkService, P2pNetworkYamuxAction};

fn get_addrs<I, S>(addr: &SocketAddr, net_svc: &mut S) -> I
where
    S: P2pNetworkService,
    I: FromIterator<Multiaddr>,
{
    let port = addr.port();
    let ip = addr.ip();
    let is_ipv6 = ip.is_ipv6();
    let ip_addrs = if ip.is_unspecified() {
        match net_svc.detect_local_ip() {
            Err(err) => {
                error!(system_time(); "error getting node addresses: {err}");
                Vec::new()
            }
            Ok(v) => v.into_iter().filter(|ip| ip.is_ipv6() == is_ipv6).collect(),
        }
    } else {
        vec![ip]
    };
    ip_addrs
        .into_iter()
        .map(|addr| Multiaddr::from(addr).with(multiaddr::Protocol::Tcp(port)))
        .collect()
}

impl P2pNetworkIdentifyStreamAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store) -> Result<(), String>
    where
        Store::Service: P2pNetworkService,
        Store: crate::P2pStore<S>,
    {
        use super::P2pNetworkIdentifyStreamState as S;
        use P2pNetworkIdentifyStreamAction as A;

        if let A::Prune { .. } = self {
            return Ok(());
        }

        let state = store
            .state()
            .network
            .scheduler
            .identify_state
            .find_identify_stream_state(self.peer_id(), self.stream_id())
            .ok_or_else(|| format!("stream not found for action {self:?}"))?;

        //println!("IdentifyStreamAction effects state: {:?}", state);
        match self {
            A::New {
                addr,
                peer_id,
                incoming: true,
                stream_id,
            } => {
                if let S::SendIdentify = state {
                    let mut listen_addrs = Vec::new();
                    for addr in store
                        .state()
                        .network
                        .scheduler
                        .listeners
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                    {
                        listen_addrs.extend(get_addrs::<Vec<_>, _>(&addr, store.service()))
                    }

                    let public_key = Some(store.state().config.identity_pub_key.clone());

                    let identify_msg = P2pNetworkIdentify {
                        protocol_version: Some("ipfs/0.1.0".to_string()),
                        // TODO: include build info from GlobalConfig (?)
                        agent_version: Some("openmina".to_owned()),
                        public_key,
                        listen_addrs,
                        // TODO: other peers seem to report inaccurate information, should we implement this?
                        observed_addr: None,
                        protocols: vec![
                            token::StreamKind::Discovery(token::DiscoveryAlgorithm::Kademlia1_0_0),
                            //token::StreamKind::Broadcast(token::BroadcastAlgorithm::Floodsub1_0_0),
                            token::StreamKind::Identify(token::IdentifyAlgorithm::Identify1_0_0),
                            //token::StreamKind::Identify(
                            //    token::IdentifyAlgorithm::IdentifyPush1_0_0,
                            //),
                            //token::StreamKind::Ping(token::PingAlgorithm::Ping1_0_0),
                            //token::StreamKind::Broadcast(token::BroadcastAlgorithm::Meshsub1_0_0),
                            //token::StreamKind::Broadcast(token::BroadcastAlgorithm::Meshsub1_1_0),
                            //token::StreamKind::Bitswap(token::BitswapAlgorithm::MinaBitswap),
                            //token::StreamKind::Bitswap(token::BitswapAlgorithm::MinaBitswap1_0_0),
                            //token::StreamKind::Bitswap(token::BitswapAlgorithm::MinaBitswap1_1_0),
                            //token::StreamKind::Bitswap(token::BitswapAlgorithm::MinaBitswap1_2_0),
                            //token::StreamKind::Status(token::StatusAlgorithm::MinaNodeStatus),
                            token::StreamKind::Rpc(token::RpcAlgorithm::Rpc0_0_1),
                        ],
                    };

                    //println!("{:?}", identify_msg);

                    let mut out = Vec::new();
                    let mut writer = Writer::new(&mut out);
                    let identify_msg_proto: Identify = (&identify_msg).into();

                    if let Err(err) = identify_msg_proto.write_message(&mut writer) {
                        warn!(meta.time(); summary = "error serializing Identify message", error = err.to_string(), action = format!("{self:?}"));
                        return Ok(());
                    }

                    store.dispatch(P2pNetworkYamuxAction::OutgoingData {
                        addr,
                        stream_id,
                        data: Data(out.into_boxed_slice()),
                        fin: false,
                    });

                    store.dispatch(P2pNetworkIdentifyStreamAction::Close {
                        addr,
                        peer_id,
                        stream_id,
                    });

                    Ok(())
                } else {
                    unreachable!()
                }
            }
            A::New {
                incoming: false, ..
            } => {
                if let S::RecvIdentify = state {
                    Ok(())
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
                S::IdentifyReceived { data } => {
                    store.dispatch(P2pIdentifyAction::UpdatePeerInformation {
                        peer_id,
                        info: data.clone(),
                    });
                    store.dispatch(P2pNetworkIdentifyStreamAction::Close {
                        addr,
                        peer_id,
                        stream_id,
                    });
                    Ok(())
                }
                S::Error(err) => {
                    warn!(meta.time(); summary = "error handling Identify action", error = err, action = format!("{self:?}"));
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
                    S::RecvIdentify
                    | S::IncomingPartialData { .. }
                    | S::IdentifyReceived { .. }
                    | S::SendIdentify => {
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
                    S::RecvIdentify
                    | S::IncomingPartialData { .. }
                    | S::IdentifyReceived { .. } => {
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
                    S::SendIdentify => {
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
