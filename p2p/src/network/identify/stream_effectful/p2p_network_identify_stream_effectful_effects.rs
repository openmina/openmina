use multiaddr::Multiaddr;
use openmina_core::{error, log::system_time};
use redux::ActionMeta;
use std::net::SocketAddr;

use super::P2pNetworkIdentifyStreamEffectfulAction;
use crate::{network::identify::P2pNetworkIdentifyStreamAction, P2pNetworkService};

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
            P2pNetworkIdentifyStreamEffectfulAction::GetListenAddresses {
                addr,
                peer_id,
                stream_id,
                addresses,
            } => {
                let mut listen_addresses = Vec::new();
                for addr in addresses {
                    listen_addresses.extend(get_addrs::<Vec<_>, _>(&addr, store.service()))
                }

                store.dispatch(P2pNetworkIdentifyStreamAction::SendIdentify {
                    addr,
                    peer_id,
                    stream_id,
                    addresses: listen_addresses,
                });
            }
        }
    }
}
