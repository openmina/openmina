use multiaddr::Multiaddr;
use openmina_core::{bug_condition, error, fuzzed_maybe, log::system_time};
use redux::ActionMeta;
use std::net::SocketAddr;

use super::{
    super::{pb, P2pNetworkIdentify},
    P2pNetworkIdentifyStreamEffectfulAction,
};
use crate::{
    network::identify::P2pNetworkIdentifyStreamAction, token, Data, P2pNetworkService,
    P2pNetworkYamuxAction,
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

impl P2pNetworkIdentifyStreamEffectfulAction {
    pub fn effects<Store, S>(self, _meta: &ActionMeta, store: &mut Store)
    where
        Store::Service: P2pNetworkService,
        Store: crate::P2pStore<S>,
    {
        match self {
            P2pNetworkIdentifyStreamEffectfulAction::SendIdentify {
                addr,
                peer_id,
                stream_id,
            } => {
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

                let mut out = Vec::new();
                let identify_msg_proto: pb::Identify = match (&identify_msg).try_into() {
                    Ok(identify_msg_proto) => identify_msg_proto,
                    Err(err) => {
                        bug_condition!("error encoding message {:?}", err);
                        return;
                    }
                };

                if let Err(err) =
                    prost::Message::encode_length_delimited(&identify_msg_proto, &mut out)
                {
                    bug_condition!("error serializing message {:?}", err);
                    return;
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
            }
        }
    }
}
