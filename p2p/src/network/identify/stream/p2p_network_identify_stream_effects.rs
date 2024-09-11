use std::net::SocketAddr;

use multiaddr::Multiaddr;
use openmina_core::{bug_condition, error, fuzzed_maybe, log::system_time, warn};
use redux::ActionMeta;

use super::{
    super::{pb, P2pNetworkIdentify},
    P2pNetworkIdentifyStreamAction, P2pNetworkIdentifyStreamState,
};
use crate::{
    identify::P2pIdentifyAction, network::identify::stream::P2pNetworkIdentifyStreamError, token,
    Data, P2pNetworkSchedulerAction, P2pNetworkService, P2pNetworkYamuxAction, YamuxFlags,
};

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
        if let P2pNetworkIdentifyStreamAction::Prune { .. } = self {
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
            P2pNetworkIdentifyStreamAction::New {
                addr,
                peer_id,
                incoming: true,
                stream_id,
            } => {
                let P2pNetworkIdentifyStreamState::SendIdentify = state else {
                    bug_condition!("Invalid state {:?} for action {:?}", state, self);
                    return Ok(());
                };

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

                let mut protocols = vec![
                    token::StreamKind::Identify(token::IdentifyAlgorithm::Identify1_0_0),
                    token::StreamKind::Broadcast(token::BroadcastAlgorithm::Meshsub1_1_0),
                    token::StreamKind::Rpc(token::RpcAlgorithm::Rpc0_0_1),
                ];
                if store.state().network.scheduler.discovery_state.is_some() {
                    protocols.push(token::StreamKind::Discovery(
                        token::DiscoveryAlgorithm::Kademlia1_0_0,
                    ));
                }
                let identify_msg = P2pNetworkIdentify {
                    protocol_version: Some("ipfs/0.1.0".to_string()),
                    // TODO: include build info from GlobalConfig (?)
                    agent_version: Some("openmina".to_owned()),
                    public_key,
                    listen_addrs,
                    // TODO: other peers seem to report inaccurate information, should we implement this?
                    observed_addr: None,
                    protocols,
                };

                //println!("{:?}", identify_msg);

                let mut out = Vec::new();
                let identify_msg_proto: pb::Identify = match (&identify_msg).try_into() {
                    Ok(identify_msg_proto) => identify_msg_proto,
                    Err(err) => {
                        bug_condition!("error encoding message {:?}", err);
                        return Err(err.to_string());
                    }
                };

                if let Err(err) =
                    prost::Message::encode_length_delimited(&identify_msg_proto, &mut out)
                {
                    bug_condition!("error serializing message {:?}", err);
                    return Err(err.to_string());
                }

                let data = fuzzed_maybe!(
                    Data(out.into_boxed_slice()),
                    crate::fuzzer::mutate_identify_msg
                );

                let flags = fuzzed_maybe!(Default::default(), crate::fuzzer::mutate_yamux_flags);

                store.dispatch(P2pNetworkYamuxAction::OutgoingData {
                    addr,
                    stream_id,
                    data,
                    flags,
                });

                store.dispatch(P2pNetworkIdentifyStreamAction::Close {
                    addr,
                    peer_id,
                    stream_id,
                });

                Ok(())
            }
            P2pNetworkIdentifyStreamAction::New {
                incoming: false, ..
            } => {
                if !matches!(state, P2pNetworkIdentifyStreamState::RecvIdentify) {
                    bug_condition!("Invalid state {:?} for action {:?}", state, self);
                }

                Ok(())
            }
            P2pNetworkIdentifyStreamAction::IncomingData {
                addr,
                peer_id,
                stream_id,
                ..
            } => match state {
                P2pNetworkIdentifyStreamState::IncomingPartialData { .. } => Ok(()),
                P2pNetworkIdentifyStreamState::IdentifyReceived { data } => {
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
                P2pNetworkIdentifyStreamState::Error(err) => {
                    warn!(meta.time(); summary = "error handling Identify action", error = display(err));
                    let error = P2pNetworkIdentifyStreamError::from(err.clone()).into();

                    store.dispatch(P2pNetworkSchedulerAction::Error { addr, error });
                    Ok(())
                }
                _ => unimplemented!(),
            },
            P2pNetworkIdentifyStreamAction::Close {
                addr,
                peer_id,
                stream_id,
            } => {
                match state {
                    P2pNetworkIdentifyStreamState::RecvIdentify
                    | P2pNetworkIdentifyStreamState::IncomingPartialData { .. }
                    | P2pNetworkIdentifyStreamState::IdentifyReceived { .. }
                    | P2pNetworkIdentifyStreamState::SendIdentify => {
                        // send FIN to the network
                        store.dispatch(P2pNetworkYamuxAction::OutgoingData {
                            addr,
                            stream_id,
                            data: Data(Box::new([])),
                            flags: YamuxFlags::FIN,
                        });
                        store.dispatch(P2pNetworkIdentifyStreamAction::Prune {
                            addr,
                            peer_id,
                            stream_id,
                        });
                        Ok(())
                    }
                    _ => Err(format!("incorrect state {state:?} for action {self:?}")),
                }
            }
            P2pNetworkIdentifyStreamAction::RemoteClose {
                addr,
                peer_id,
                stream_id,
            } => {
                match state {
                    P2pNetworkIdentifyStreamState::RecvIdentify
                    | P2pNetworkIdentifyStreamState::IncomingPartialData { .. }
                    | P2pNetworkIdentifyStreamState::IdentifyReceived { .. } => {
                        // send FIN to the network
                        store.dispatch(P2pNetworkYamuxAction::OutgoingData {
                            addr,
                            stream_id,
                            data: Data(Box::new([])),
                            flags: YamuxFlags::FIN,
                        });
                        store.dispatch(P2pNetworkIdentifyStreamAction::Prune {
                            addr,
                            peer_id,
                            stream_id,
                        });
                        Ok(())
                    }
                    P2pNetworkIdentifyStreamState::SendIdentify => {
                        store.dispatch(P2pNetworkIdentifyStreamAction::Prune {
                            addr,
                            peer_id,
                            stream_id,
                        });
                        Ok(())
                    }
                    _ => Err(format!("incorrect state {state:?} for action {self:?}")),
                }
            }
            P2pNetworkIdentifyStreamAction::Prune { .. } => {
                bug_condition!("Invalid state {:?} for action {:?}", state, self);
                Ok(())
            }
        }
    }
}
