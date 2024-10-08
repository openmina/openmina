use std::net::{IpAddr, SocketAddr};

use multiaddr::Multiaddr;

use crate::{
    bootstrap::P2pNetworkKadBoostrapRequestState,
    connection::outgoing::P2pConnectionOutgoingInitOpts, P2pNetworkKadBootstrapAction,
    P2pNetworkService, P2pPeerAction, SocketAddrTryFromMultiaddrError,
};

use super::P2pNetworkKadEffectfulAction;

fn socket_addr_try_from_multiaddr<Service>(
    service: &mut Service,
    multiaddr: &Multiaddr,
    filter_local: bool,
) -> Result<Option<SocketAddr>, SocketAddrTryFromMultiaddrError>
where
    Service: P2pNetworkService,
{
    let mut iter = multiaddr.iter();
    let ip_addr = match iter.next() {
        Some(multiaddr::Protocol::Ip4(ip4)) => IpAddr::V4(ip4),
        Some(multiaddr::Protocol::Ip6(ip6)) => IpAddr::V6(ip6),
        Some(multiaddr::Protocol::Dns4(hostname) | multiaddr::Protocol::Dns6(hostname)) => {
            service.resolve_name(&hostname)?.first().cloned().ok_or(
                SocketAddrTryFromMultiaddrError::UnsupportedHost(hostname.to_string()),
            )?
        }
        None => return Err(SocketAddrTryFromMultiaddrError::NoHost),
        Some(p) => {
            return Err(SocketAddrTryFromMultiaddrError::UnsupportedHost(
                p.to_string(),
            ))
        }
    };
    let port = match iter.next() {
        Some(multiaddr::Protocol::Tcp(port)) => port,
        None => return Err(SocketAddrTryFromMultiaddrError::NoPort),
        Some(p) => {
            return Err(SocketAddrTryFromMultiaddrError::UnsupportedPort(
                p.to_string(),
            ))
        }
    };
    if let Some(p) = iter.next() {
        return Err(SocketAddrTryFromMultiaddrError::ExtraProtocol(
            p.to_string(),
        ));
    }

    let filter = |addr: &SocketAddr| {
        !filter_local
            || match addr.ip() {
                std::net::IpAddr::V4(ipv4) if ipv4.is_loopback() || ipv4.is_private() => false,
                std::net::IpAddr::V6(ipv6) if ipv6.is_loopback() => false,
                _ => true,
            }
    };

    Ok(Some((ip_addr, port).into()).filter(filter))
}

impl P2pNetworkKadEffectfulAction {
    pub fn effects<Store, S>(self, meta: &redux::ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pNetworkService,
    {
        match self {
            Self::Discovered {
                multiaddr,
                filter_local,
                peer_id,
            } => match socket_addr_try_from_multiaddr(store.service(), &multiaddr, filter_local) {
                Ok(Some(addr)) => {
                    store.dispatch(P2pPeerAction::Discovered {
                        peer_id,
                        dial_opts: Some(P2pConnectionOutgoingInitOpts::LibP2P(
                            (peer_id, addr).into(),
                        )),
                    });
                }
                Ok(None) => {}
                Err(err) => {
                    let _ = err;
                    // TODO: report
                }
            },
            Self::MakeRequest {
                multiaddr,
                filter_local,
                peer_id,
            } => {
                let addrs = multiaddr.iter().filter_map(|multiaddr| {
                    socket_addr_try_from_multiaddr(store.service(), multiaddr, filter_local)
                        .ok()
                        .flatten()
                });
                let addrs_to_use = addrs.collect::<Vec<_>>();
                let request =
                    addrs_to_use
                        .first()
                        .cloned()
                        .map(|addr| P2pNetworkKadBoostrapRequestState {
                            addr,
                            time: meta.time(),
                            addrs_to_use,
                        });
                store.dispatch(P2pNetworkKadBootstrapAction::AppendRequest { request, peer_id });
            }
        }
    }
}
