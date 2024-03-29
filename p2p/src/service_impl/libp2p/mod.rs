mod behavior;
pub use behavior::Event as BehaviourEvent;
pub use behavior::*;

use mina_p2p_messages::rpc::GetSomeInitialPeersV1ForV2;

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use mina_p2p_messages::binprot::{self, BinProtRead, BinProtWrite};
use mina_p2p_messages::v2::NetworkPoolSnarkPoolDiffVersionedStableV2;
use openmina_core::channels::mpsc;
use openmina_core::snark::Snark;

use libp2p::core::muxing::StreamMuxerBox;
use libp2p::core::transport;
use libp2p::futures::{select, FutureExt, StreamExt};
use libp2p::gossipsub::{
    Behaviour as Gossipsub, ConfigBuilder as GossipsubConfigBuilder, Event as GossipsubEvent,
    IdentTopic, MessageAcceptance, MessageAuthenticity,
};
use libp2p::identity::Keypair;
use libp2p::kad::record::store::MemoryStore;
use libp2p::kad::Mode;
use libp2p::pnet::{PnetConfig, PreSharedKey};
use libp2p::swarm::dial_opts::DialOpts;
use libp2p::swarm::SwarmEvent;
use libp2p::{identify, kad};
use libp2p::{noise, StreamProtocol};
use libp2p::{Multiaddr, PeerId, Swarm, Transport};
pub use mina_p2p_messages::gossip::GossipNetMessageV2 as GossipNetMessage;

use libp2p_rpc_behaviour::{BehaviourBuilder, Event as RpcBehaviourEvent, StreamId};

use crate::channels::best_tip::BestTipPropagationChannelMsg;
use crate::channels::rpc::{
    BestTipWithProof, P2pRpcRequest, P2pRpcResponse, RpcChannelMsg,
    StagedLedgerAuxAndPendingCoinbases,
};
use crate::channels::ChannelMsg;
use crate::connection::outgoing::{
    P2pConnectionOutgoingInitLibp2pOpts, P2pConnectionOutgoingInitOpts,
};
use crate::identity::SecretKey;
use crate::{P2pChannelEvent, P2pConnectionEvent, P2pDiscoveryEvent, P2pEvent, P2pListenEvent};

use super::TaskSpawner;

/// Type alias for libp2p transport
pub type P2PTransport = (PeerId, StreamMuxerBox);
/// Type alias for boxed libp2p transport
pub type BoxedP2PTransport = transport::Boxed<P2PTransport>;

#[derive(Debug)]
pub enum Cmd {
    Dial(PeerId, Vec<Multiaddr>),
    Disconnect(PeerId),
    SendMessage(PeerId, ChannelMsg),
    SnarkBroadcast(Snark, u32),
    RunDiscovery(Vec<(PeerId, Multiaddr)>),
    FindNode(PeerId),
}

pub struct Libp2pService {
    cmd_sender: mpsc::UnboundedSender<Cmd>,
}

async fn determine_own_ip() -> BTreeSet<IpAddr> {
    use std::net::{Ipv4Addr, Ipv6Addr};

    let local_addresses = [
        IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        IpAddr::V6(Ipv6Addr::UNSPECIFIED),
    ];
    let services = [
        "https://ifconfig.co/ip",
        "https://bot.whatismyipaddress.com",
        "https://api.ipify.org",
    ];

    let clients = local_addresses.into_iter().filter_map(|addr| {
        reqwest::ClientBuilder::new()
            .local_address(addr)
            .timeout(Duration::from_secs(20))
            .build()
            .ok()
    });

    let (tx, mut rx) = mpsc::unbounded_channel();
    for client in clients {
        for service in services {
            let tx = tx.clone();
            let client = client.clone();
            tokio::spawn(async move {
                let addr = {
                    client
                        .get(service)
                        .send()
                        .await
                        .ok()?
                        .text()
                        .await
                        .ok()?
                        .trim_end_matches('\n')
                        .parse::<IpAddr>()
                        .ok()?
                };
                tx.send(addr).unwrap_or_default();

                Some(())
            });
        }
    }
    drop(tx);

    let mut addresses = BTreeSet::new();
    while let Some(addr) = rx.recv().await {
        addresses.insert(addr);
    }

    addresses
}

#[allow(dead_code)]
async fn determine_own_ip_stun(stun_addr: SocketAddr) -> io::Result<IpAddr> {
    use faster_stun::{attribute, Decoder, Kind, Method, Payload};
    use tokio::net::UdpSocket;

    let socket = UdpSocket::bind(SocketAddr::from(([0; 4], 0))).await?;
    tokio::time::timeout(Duration::from_secs(10), async move {
        loop {
            let mut request =
                *b"\x00\x01\x00\x00\x21\x12\xa4\x42\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
            request[8..].clone_from_slice(&rand::random::<[u8; 12]>());
            socket.send_to(&request, stun_addr).await?;
            let mut buf = [0; 0x10000];
            let (_, remote_addr) = socket.recv_from(&mut buf).await?;
            let mut decoder = Decoder::new();
            if let Ok(Payload::Message(msg)) = decoder.decode(&buf) {
                if msg.method == Method::Binding(Kind::Response)
                    && remote_addr == stun_addr
                    && msg.token == &request[8..]
                {
                    if let Some(addr) = msg.get::<attribute::XorMappedAddress>() {
                        return Ok(addr.ip());
                    }
                }
            }
        }
    }).await.map_err(|_| io::Error::from(io::ErrorKind::TimedOut)).and_then(|x| x)
}

impl Libp2pService {
    const GOSSIPSUB_TOPIC: &'static str = "coda/consensus-messages/0.0.1";

    pub fn mocked() -> (Self, mpsc::UnboundedReceiver<Cmd>) {
        let (cmd_sender, rx) = mpsc::unbounded_channel();
        (Self { cmd_sender }, rx)
    }

    pub fn run<E, S>(
        libp2p_port: Option<u16>,
        secret_key: SecretKey,
        chain_id: String,
        event_source_sender: mpsc::UnboundedSender<E>,
        spawner: S,
    ) -> Self
    where
        E: 'static + Send + From<P2pEvent>,
        S: TaskSpawner,
    {
        let topics_iter = IntoIterator::into_iter([
            Self::GOSSIPSUB_TOPIC,
            "mina/block/1.0.0",
            "mina/tx/1.0.0",
            "mina/snark-work/1.0.0",
        ]);

        let identity_keys = Keypair::ed25519_from_bytes(secret_key.to_bytes())
            .expect("secret key bytes must be valid");

        let message_authenticity = MessageAuthenticity::Signed(identity_keys.clone());
        let gossipsub_config = GossipsubConfigBuilder::default()
            .max_transmit_size(1024 * 1024 * 32)
            .validate_messages()
            .build()
            .unwrap();
        let mut gossipsub: Gossipsub =
            Gossipsub::new(message_authenticity, gossipsub_config).unwrap();
        topics_iter
            .map(|v| IdentTopic::new(v))
            .for_each(|topic| assert!(gossipsub.subscribe(&topic).unwrap()));

        let identify = identify::Behaviour::new(identify::Config::new(
            "ipfs/0.1.0".to_string(),
            identity_keys.public(),
        ));

        let peer_id = identity_keys.public().to_peer_id();
        let kad_config = {
            let mut c = kad::Config::default();
            c.set_protocol_names(vec![StreamProtocol::new("/coda/kad/1.0.0")]);
            c
        };
        let kademlia = kad::Behaviour::with_config(peer_id, MemoryStore::new(peer_id), kad_config);

        let behaviour = Behaviour {
            gossipsub,
            rpc: {
                use mina_p2p_messages::rpc::{
                    AnswerSyncLedgerQueryV2, GetAncestryV2, GetBestTipV2,
                    GetStagedLedgerAuxAndPendingCoinbasesAtHashV2, GetTransitionChainProofV1ForV2,
                    GetTransitionChainV2,
                };

                BehaviourBuilder::default()
                    .register_method::<GetBestTipV2>()
                    .register_method::<GetAncestryV2>()
                    .register_method::<GetStagedLedgerAuxAndPendingCoinbasesAtHashV2>()
                    .register_method::<AnswerSyncLedgerQueryV2>()
                    .register_method::<GetTransitionChainV2>()
                    .register_method::<GetTransitionChainProofV1ForV2>()
                    .build()
            },
            identify,
            kademlia,
            chain_id,
            event_source_sender,
            ongoing: BTreeMap::default(),
            ongoing_incoming: BTreeMap::default(),
        };

        let (cmd_sender, mut cmd_receiver) = mpsc::unbounded_channel();
        let psk = PreSharedKey::new(openmina_core::preshared_key(&behaviour.chain_id));

        let fut = async move {
            let mut swarm = libp2p::SwarmBuilder::with_existing_identity(identity_keys)
                .with_tokio()
                .with_other_transport(|key| {
                    let noise_config = noise::Config::new(key).unwrap();
                    let mut yamux_config = libp2p::yamux::Config::default();

                    yamux_config.set_protocol_name("/coda/yamux/1.0.0");

                    let base_transport = libp2p::tcp::tokio::Transport::new(
                        libp2p::tcp::Config::default()
                            .nodelay(true)
                            .port_reuse(true),
                    );

                    base_transport
                        .and_then(move |socket, _| PnetConfig::new(psk).handshake(socket))
                        .upgrade(libp2p::core::upgrade::Version::V1)
                        .authenticate(noise_config)
                        .multiplex(yamux_config)
                        .timeout(Duration::from_secs(60))
                })?
                .with_dns()?
                .with_behaviour(|_| behaviour)?
                .build();

            swarm.behaviour_mut().kademlia.set_mode(Some(Mode::Server));

            if let Some(port) = libp2p_port {
                for ip in determine_own_ip().await {
                    let mut addr = Multiaddr::from(ip);
                    addr.push(libp2p::multiaddr::Protocol::Tcp(port));
                    swarm.add_external_address(addr);
                }

                if let Err(err) = swarm.listen_on(format!("/ip6/::/tcp/{port}").parse().unwrap()) {
                    openmina_core::log::error!(
                        openmina_core::log::system_time();
                        kind = "Libp2pListenError",
                        summary = format!("libp2p failed to start listener on ipv6 at port: {port}. error: {err:?}"),
                    );
                }
                if let Err(err) =
                    swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{port}").parse().unwrap())
                {
                    openmina_core::log::error!(
                        openmina_core::log::system_time();
                        kind = "Libp2pListenError",
                        summary = format!("libp2p failed to start listener on ipv4 at port: {port}. error: {err:?}"),
                    );
                }
            }

            loop {
                select! {
                    event = swarm.next() => match event {
                        Some(event) => Self::handle_event(&mut swarm, event).await,
                        None => break,
                    },
                    cmd = cmd_receiver.recv().fuse() => match cmd {
                        Some(cmd) => Self::handle_cmd(&mut swarm, cmd).await,
                        None => break,
                    }
                }
            }

            // FIXME: keeping the compiler happy but we need proper handling
            Result::<(), Box<dyn std::error::Error>>::Ok(())
        };

        spawner.spawn_main("libp2p", fut);

        Self { cmd_sender }
    }

    fn gossipsub_send<E>(swarm: &mut Swarm<Behaviour<E>>, msg: &GossipNetMessage)
    where
        E: From<P2pEvent>,
    {
        let mut encoded = vec![0; 8];
        match msg.binprot_write(&mut encoded) {
            Ok(_) => {}
            Err(_err) => {
                // TODO(binier)
                return;
                // log::error!("Failed to encode GossipSub Message: {:?}", err);
                // panic!("{}", err);
            }
        }
        let msg_len = (encoded.len() as u64 - 8).to_le_bytes();
        encoded[..8].clone_from_slice(&msg_len);

        let topic = IdentTopic::new(Self::GOSSIPSUB_TOPIC);
        let _ = swarm.behaviour_mut().gossipsub.publish(topic, encoded);
    }

    async fn handle_cmd<E: From<P2pEvent>>(swarm: &mut Swarm<Behaviour<E>>, cmd: Cmd) {
        match cmd {
            Cmd::Dial(peer_id, addrs) => {
                let opts = DialOpts::peer_id(peer_id.into()).addresses(addrs).build();
                if let Err(e) = swarm.dial(opts) {
                    let peer_id = crate::PeerId::from(peer_id);
                    openmina_core::log::error!(
                        openmina_core::log::system_time();
                        node_id = crate::PeerId::from(swarm.local_peer_id().clone()).to_string(),
                        summary = format!("Cmd::Dial(...)"),
                        peer_id = peer_id.to_string(),
                        error = e.to_string()
                    );
                }
            }
            Cmd::Disconnect(peer_id) => {
                let _ = swarm.disconnect_peer_id(peer_id);
            }
            Cmd::SendMessage(peer_id, msg) => match msg {
                ChannelMsg::SnarkPropagation(_) => {
                    // unsupported. Instead `Cmd::SnarkBroadcast` will be used.
                }
                ChannelMsg::SnarkJobCommitmentPropagation(_) => {
                    // unsupported
                }
                ChannelMsg::BestTipPropagation(msg) => match msg {
                    BestTipPropagationChannelMsg::GetNext => {
                        // TODO(binier): mark that peer can send us
                        // a message now. For now not important as
                        // we send this message right after we see a
                        // message from the peer.
                    }
                    BestTipPropagationChannelMsg::BestTip(block) => {
                        // TODO(binier): for each peer, send message cmd
                        // will be received, yet we are broadcasting to
                        // every peer every time. It's kinda fine because
                        // gossipsub protocol will prevent same message
                        // from being published, but it's still wasteful.
                        Self::gossipsub_send(
                            swarm,
                            &GossipNetMessage::NewState(block.as_ref().clone()),
                        );
                        // TODO(binier): send event: `P2pChannelEvent::Sent`
                    }
                },
                ChannelMsg::Rpc(msg) => {
                    Self::handle_cmd_rpc(swarm, peer_id, msg)
                        .expect("binprot write error must not happen, must send valid msg");
                }
            },
            Cmd::SnarkBroadcast(snark, nonce) => {
                let message = Box::new((snark.statement(), (&snark).into()));
                let message = NetworkPoolSnarkPoolDiffVersionedStableV2::AddSolvedWork(message);
                let nonce = nonce.into();
                Self::gossipsub_send(swarm, &GossipNetMessage::SnarkPoolDiff { message, nonce });
            }
            Cmd::RunDiscovery(peers) => {
                for (peer_id, addr) in peers {
                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                }

                match swarm.behaviour_mut().kademlia.bootstrap() {
                    Ok(_id) => {}
                    Err(err) => {
                        let _ = err;
                        // TODO: log error
                    }
                }
            }
            Cmd::FindNode(peer_id) => {
                let _id = swarm.behaviour_mut().kademlia.get_closest_peers(peer_id);
            }
        }
    }

    fn handle_cmd_rpc<E: From<P2pEvent>>(
        swarm: &mut Swarm<Behaviour<E>>,
        peer_id: PeerId,
        msg: RpcChannelMsg,
    ) -> Result<(), binprot::Error> {
        use mina_p2p_messages::{
            core::Info,
            rpc::{
                AnswerSyncLedgerQueryV2, GetAncestryV2, GetBestTipV2,
                GetStagedLedgerAuxAndPendingCoinbasesAtHashV2, GetTransitionChainV2,
                ProofCarryingDataStableV1, ProofCarryingDataWithHashV1,
            },
            rpc_kernel::{RpcMethod, RpcResult},
        };

        let b = swarm.behaviour_mut();
        match msg {
            RpcChannelMsg::Request(id, req) => {
                let stream_id = StreamId::Outgoing(0);
                let key = (peer_id, id);

                match req {
                    P2pRpcRequest::BestTipWithProof => {
                        type T = GetBestTipV2;
                        b.ongoing.insert(key, (T::NAME, T::VERSION));
                        b.rpc.query::<T>(peer_id, stream_id, id, ())?;
                    }
                    P2pRpcRequest::LedgerQuery(hash, query) => {
                        type T = AnswerSyncLedgerQueryV2;
                        b.ongoing.insert(key, (T::NAME, T::VERSION));
                        let query = (hash.0.clone(), query);
                        b.rpc.query::<T>(peer_id, stream_id, id, query)?;
                    }
                    P2pRpcRequest::StagedLedgerAuxAndPendingCoinbasesAtBlock(hash) => {
                        type T = GetStagedLedgerAuxAndPendingCoinbasesAtHashV2;
                        b.ongoing.insert(key, (T::NAME, T::VERSION));
                        let query = hash.0.clone();
                        b.rpc.query::<T>(peer_id, stream_id, id, query)?;
                    }
                    P2pRpcRequest::Block(hash) => {
                        type T = GetTransitionChainV2;
                        b.ongoing.insert(key, (T::NAME, T::VERSION));
                        let query = vec![hash.0.clone()];
                        b.rpc.query::<T>(peer_id, stream_id, id, query)?;
                    }
                    P2pRpcRequest::Snark(_) => {}
                    P2pRpcRequest::InitialPeers => {
                        type T = GetSomeInitialPeersV1ForV2;
                        b.ongoing.insert(key, (T::NAME, T::VERSION));
                        b.rpc.query::<T>(peer_id, stream_id, id, ())?;
                    }
                };
            }
            RpcChannelMsg::Response(id, resp) => {
                if let Some((stream_id, tag, version)) = b.ongoing_incoming.remove(&(peer_id, id)) {
                    match resp {
                        None => match (tag.as_bytes(), version) {
                            (GetBestTipV2::NAME, GetBestTipV2::VERSION) => {
                                type T = GetBestTipV2;
                                b.rpc.respond::<T>(peer_id, stream_id, id, Ok(None))?
                            }
                            (GetAncestryV2::NAME, GetAncestryV2::VERSION) => {
                                type T = GetAncestryV2;
                                b.rpc.respond::<T>(peer_id, stream_id, id, Ok(None))?
                            }
                            (AnswerSyncLedgerQueryV2::NAME, AnswerSyncLedgerQueryV2::VERSION) => {
                                // TODO: shouldn't we disable this method in menu?
                                type T = AnswerSyncLedgerQueryV2;
                                b.rpc.respond::<T>(
                                    peer_id,
                                    stream_id,
                                    id,
                                    Ok(RpcResult(Err(Info::from_str("not implemented")))),
                                )?
                            }
                            (
                                GetStagedLedgerAuxAndPendingCoinbasesAtHashV2::NAME,
                                GetStagedLedgerAuxAndPendingCoinbasesAtHashV2::VERSION,
                            ) => {
                                type T = GetStagedLedgerAuxAndPendingCoinbasesAtHashV2;
                                b.rpc.respond::<T>(peer_id, stream_id, id, Ok(None))?
                            }
                            (GetTransitionChainV2::NAME, GetTransitionChainV2::VERSION) => {
                                type T = GetTransitionChainV2;
                                b.rpc.respond::<T>(peer_id, stream_id, id, Ok(None))?
                            }
                            (
                                GetSomeInitialPeersV1ForV2::NAME,
                                GetSomeInitialPeersV1ForV2::VERSION,
                            ) => {
                                type T = GetSomeInitialPeersV1ForV2;
                                b.rpc.respond::<T>(peer_id, stream_id, id, Ok(vec![]))?
                            }
                            _ => {}
                        },
                        Some(P2pRpcResponse::BestTipWithProof(msg)) => {
                            if tag.as_bytes() == GetAncestryV2::NAME {
                                type T = GetAncestryV2;
                                let v = msg.proof.0.iter().map(|x| x.0.clone()).collect();
                                let r = Ok(Some(ProofCarryingDataWithHashV1 {
                                    data: (*msg.best_tip).clone(),
                                    proof: (v, (*msg.proof.1).clone()),
                                }));
                                b.rpc.respond::<T>(peer_id, stream_id, id, r)?;
                            } else {
                                type T = GetBestTipV2;
                                let r = Ok(Some(ProofCarryingDataStableV1 {
                                    data: (*msg.best_tip).clone(),
                                    proof: (msg.proof.0, (*msg.proof.1).clone()),
                                }));
                                b.rpc.respond::<T>(peer_id, stream_id, id, r)?;
                            }
                        }
                        Some(P2pRpcResponse::LedgerQuery(msg)) => {
                            type T = AnswerSyncLedgerQueryV2;
                            let r = Ok(RpcResult(Ok(msg)));
                            b.rpc.respond::<T>(peer_id, stream_id, id, r)?;
                        }
                        Some(P2pRpcResponse::StagedLedgerAuxAndPendingCoinbasesAtBlock(msg)) => {
                            type T = GetStagedLedgerAuxAndPendingCoinbasesAtHashV2;
                            let r = (
                                msg.scan_state.clone(),
                                msg.staged_ledger_hash.0.clone(),
                                msg.pending_coinbase.clone(),
                                msg.needed_blocks.clone(),
                            );
                            let r = Ok(Some(r));
                            b.rpc.respond::<T>(peer_id, stream_id, id, r)?;
                        }
                        Some(P2pRpcResponse::Block(msg)) => {
                            type T = GetTransitionChainV2;
                            let r = Ok(Some(vec![(*msg).clone()]));
                            b.rpc.respond::<T>(peer_id, stream_id, id, r)?;
                        }
                        Some(P2pRpcResponse::Snark(_)) => {}
                        Some(P2pRpcResponse::InitialPeers(peers)) => {
                            type T = GetSomeInitialPeersV1ForV2;
                            let r = Ok(peers
                                .iter()
                                .filter_map(P2pConnectionOutgoingInitOpts::try_into_mina_rpc)
                                .collect());
                            b.rpc.respond::<T>(peer_id, stream_id, id, r)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_event<E: From<P2pEvent>, Err: std::error::Error>(
        swarm: &mut Swarm<Behaviour<E>>,
        event: SwarmEvent<BehaviourEvent, Err>,
    ) {
        match event {
            SwarmEvent::NewListenAddr {
                listener_id,
                address,
                ..
            } => {
                let maddr = format!("{address}/p2p/{}", swarm.local_peer_id());
                let listener_id = format!("{listener_id:?}");
                openmina_core::log::info!(
                    openmina_core::log::system_time();
                    kind = "Libp2pListenStart",
                    summary = format!("libp2p.{listener_id} listening on: {maddr}"),
                    listener_id = listener_id,
                    maddr = maddr,
                );
                let event = P2pEvent::Listen(P2pListenEvent::NewListenAddr {
                    listener_id: listener_id.into(),
                    addr: address,
                });
                let _ = swarm.behaviour_mut().event_source_sender.send(event.into());
            }
            SwarmEvent::ExpiredListenAddr {
                listener_id,
                address,
                ..
            } => {
                let maddr = format!("{address}/p2p/{}", swarm.local_peer_id());
                let listener_id = format!("{listener_id:?}");
                openmina_core::log::info!(
                    openmina_core::log::system_time();
                    kind = "Libp2pListenStart",
                    summary = format!("libp2p.{listener_id} stopped listening on: {maddr}"),
                    listener_id = listener_id,
                    maddr = maddr,
                );
                let event = P2pEvent::Listen(P2pListenEvent::ExpiredListenAddr {
                    listener_id: listener_id.into(),
                    addr: address,
                });
                let _ = swarm.behaviour_mut().event_source_sender.send(event.into());
            }
            SwarmEvent::ListenerError { listener_id, error } => {
                let listener_id = format!("{listener_id:?}");
                openmina_core::log::error!(
                    openmina_core::log::system_time();
                    kind = "Libp2pListenError",
                    summary = format!("libp2p.{listener_id:?} listener error: {error:?}"),
                    listener_id = listener_id,
                );
                let event = P2pEvent::Listen(P2pListenEvent::ListenerError {
                    listener_id: listener_id.into(),
                    error: error.to_string(),
                });
                let _ = swarm.behaviour_mut().event_source_sender.send(event.into());
            }
            SwarmEvent::ListenerClosed {
                listener_id,
                reason,
                ..
            } => {
                let listener_id = format!("{listener_id:?}");
                openmina_core::log::warn!(
                    openmina_core::log::system_time();
                    kind = "Libp2pListenError",
                    summary = format!("libp2p.{listener_id} closed. Reason: {reason:?}"),
                    listener_id = listener_id,
                );
                let event = P2pEvent::Listen(P2pListenEvent::ListenerClosed {
                    listener_id: listener_id.into(),
                    error: reason.err().map(|err| err.to_string()),
                });
                let _ = swarm.behaviour_mut().event_source_sender.send(event.into());
            }
            SwarmEvent::IncomingConnection {
                connection_id,
                local_addr,
                send_back_addr,
            } => {
                let connection_id = format!("{connection_id:?}");
                openmina_core::log::info!(
                    openmina_core::log::system_time();
                    kind = "libp2p::IncomingConnection",
                    summary = format!("libp2p incoming {connection_id} {local_addr} {send_back_addr}"),
                    connection_id = connection_id,
                );
            }
            SwarmEvent::Dialing { peer_id, .. } => {
                let peer_id = peer_id
                    .map(crate::PeerId::from)
                    .as_ref()
                    .map(ToString::to_string);
                let peer_id = peer_id.as_ref().map_or("<unknown>", String::as_str);
                openmina_core::log::info!(
                    openmina_core::log::system_time();
                    node_id = crate::PeerId::from(swarm.local_peer_id().clone()).to_string(),
                    kind = "libp2p::Dialing",
                    summary = format!("peer_id: {peer_id}"),
                    peer_id = peer_id,
                );
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                swarm.behaviour_mut().identify.push(Some(peer_id));
                let peer_id: crate::PeerId = peer_id.into();
                openmina_core::log::info!(
                    openmina_core::log::system_time();
                    node_id = crate::PeerId::from(swarm.local_peer_id().clone()).to_string(),
                    kind = "libp2p::ConnectionEstablished",
                    summary = format!("peer_id: {}", peer_id),
                    peer_id = peer_id.to_string(),
                );
                let event = P2pEvent::Connection(P2pConnectionEvent::Finalized(peer_id, Ok(())));
                let _ = swarm.behaviour_mut().event_source_sender.send(event.into());
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                let peer_id: crate::PeerId = peer_id.into();
                let event = P2pEvent::Connection(P2pConnectionEvent::Closed(peer_id));
                let _ = swarm.behaviour_mut().event_source_sender.send(event.into());

                // TODO(binier): move to log effects
                openmina_core::log::warn!(
                    openmina_core::log::system_time();
                    kind = "PeerDisconnected",
                    summary = format!("peer_id: {}", peer_id),
                    peer_id = peer_id.to_string(),
                    cause = format!("{:?}", cause)
                );
            }
            SwarmEvent::OutgoingConnectionError {
                connection_id: _,
                peer_id,
                error,
            } => {
                let peer_id = match peer_id {
                    Some(v) => v,
                    None => return,
                };
                if peer_id.as_ref().code() == 0x12 {
                    // cannot report about the failure,
                    // because our PeerId cannot represent this peer_id
                    return;
                }
                let event = P2pEvent::Connection(P2pConnectionEvent::Finalized(
                    peer_id.into(),
                    Err(error.to_string()),
                ));
                let _ = swarm.behaviour_mut().event_source_sender.send(event.into());
            }
            SwarmEvent::Behaviour(event) => match event {
                BehaviourEvent::Kademlia(event) => {
                    match event {
                        kad::Event::RoutingUpdated {
                            peer, addresses, ..
                        } => {
                            if peer.as_ref().code() != 0x12 {
                                let event = P2pEvent::Discovery(P2pDiscoveryEvent::AddRoute(
                                    peer.into(),
                                    addresses
                                        .iter()
                                        .filter_map(|a| {
                                            P2pConnectionOutgoingInitLibp2pOpts::try_from(a).ok()
                                        })
                                        .map(P2pConnectionOutgoingInitOpts::LibP2P)
                                        .collect(),
                                ));
                                let _ =
                                    swarm.behaviour_mut().event_source_sender.send(event.into());
                            }
                        }
                        kad::Event::OutboundQueryProgressed { step, result, .. } => {
                            let b = swarm.behaviour_mut();
                            match result {
                                kad::QueryResult::Bootstrap(Ok(_v)) => {
                                    use sha2::digest::{FixedOutput, Update};

                                    if step.last {
                                        let r = sha2::Sha256::default()
                                            .chain(b"/coda/0.0.1/")
                                            .chain(&b.chain_id)
                                            .finalize_fixed();
                                        // TODO(vlad9486): use multihash, remove hardcode
                                        let mut key = vec![18, 32];
                                        key.extend_from_slice(&r);
                                        let key = kad::record::Key::new(&key);

                                        if let Err(_err) = b.kademlia.start_providing(key) {
                                            // memory storage should not return error
                                        }
                                        // initial bootstrap is done
                                        b.event_source_sender
                                            .send(
                                                P2pEvent::Discovery(P2pDiscoveryEvent::Ready)
                                                    .into(),
                                            )
                                            .unwrap_or_default();
                                    }
                                }
                                kad::QueryResult::GetClosestPeers(Ok(v)) => {
                                    let peers = v.peers.into_iter().filter_map(|peer_id| {
                                        if peer_id.as_ref().code() == 0x12 {
                                            return None;
                                        }
                                        Some(peer_id.into())
                                    });
                                    let response = P2pDiscoveryEvent::DidFindPeers(peers.collect());
                                    b.event_source_sender
                                        .send(P2pEvent::Discovery(response).into())
                                        .unwrap_or_default()
                                }
                                kad::QueryResult::GetClosestPeers(Err(err)) => {
                                    let response =
                                        P2pDiscoveryEvent::DidFindPeersError(err.to_string());
                                    b.event_source_sender
                                        .send(P2pEvent::Discovery(response).into())
                                        .unwrap_or_default()
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                BehaviourEvent::Gossipsub(GossipsubEvent::Message {
                    propagation_source,
                    message_id,
                    message,
                }) => {
                    // We will manually publish applied blocks.
                    // TODO(binier): better approach
                    let _ = swarm
                        .behaviour_mut()
                        .gossipsub
                        .report_message_validation_result(
                            &message_id,
                            &propagation_source,
                            MessageAcceptance::Ignore,
                        );

                    let bytes = &message.data;
                    let res = if bytes.len() < 8 {
                        Err("message too short".to_owned())
                    } else {
                        let len = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
                        let data = &bytes[8..];
                        assert_eq!(len, data.len() as u64);
                        GossipNetMessage::binprot_read(&mut &*data)
                            .map_err(|err| format!("{err:?}"))
                    };
                    let res = match res {
                        Err(err) => Err(err),
                        Ok(GossipNetMessage::NewState(block)) => {
                            Ok(ChannelMsg::BestTipPropagation(
                                BestTipPropagationChannelMsg::BestTip(block.into()),
                            ))
                        }
                        Ok(GossipNetMessage::SnarkPoolDiff { message, nonce }) => match message {
                            // TODO(binier): Why empty? Should we error?
                            NetworkPoolSnarkPoolDiffVersionedStableV2::Empty => return,
                            NetworkPoolSnarkPoolDiffVersionedStableV2::AddSolvedWork(work) => {
                                let event =
                                    P2pEvent::Channel(P2pChannelEvent::Libp2pSnarkReceived(
                                        propagation_source.into(),
                                        work.1.into(),
                                        nonce.as_u32(),
                                    ));
                                let _ =
                                    swarm.behaviour_mut().event_source_sender.send(event.into());
                                return;
                            }
                        },
                        _ => return,
                    };

                    let event = P2pEvent::Channel(P2pChannelEvent::Received(
                        propagation_source.into(),
                        res,
                    ));
                    let _ = swarm.behaviour_mut().event_source_sender.send(event.into());
                }
                BehaviourEvent::Rpc((peer_id, event)) => {
                    Self::handle_event_rpc(swarm, peer_id, event);
                }
                BehaviourEvent::Identify(identify::Event::Received { peer_id, info }) => {
                    if let Some(maddr) = info.listen_addrs.first() {
                        swarm
                            .behaviour_mut()
                            .kademlia
                            .add_address(&peer_id, maddr.clone());

                        let mut maddr = maddr.clone();
                        maddr.push(libp2p::multiaddr::Protocol::P2p(peer_id.into()));
                        let _ = swarm
                            .behaviour_mut()
                            .event_source_sender
                            .send(P2pEvent::Libp2pIdentify(peer_id.into(), maddr).into());
                    }
                }
                _ => {
                    openmina_core::log::trace!(
                        openmina_core::log::system_time();
                        kind = "IgnoredLibp2pBehaviorEvent",
                        event = format!("{:?}", event)
                    );
                }
            },
            event => {
                openmina_core::log::trace!(
                    openmina_core::log::system_time();
                    kind = "IgnoredLibp2pSwarmEvent",
                    event = format!("{:?}", event)
                );
            }
        }
    }

    fn handle_event_rpc<E: From<P2pEvent>>(
        swarm: &mut Swarm<Behaviour<E>>,
        peer_id: PeerId,
        event: RpcBehaviourEvent,
    ) {
        let sender = swarm.behaviour_mut().event_source_sender.clone();
        let send = |event: P2pEvent| {
            let _ = sender.send(event.into());
        };
        let send_error = |err: String| {
            let msg = P2pEvent::Channel(P2pChannelEvent::Received(peer_id.into(), Err(err)));
            let _ = sender.send(msg.into());
        };
        match event {
            RpcBehaviourEvent::ConnectionClosed => {
                // send(P2pConnectionEvent::Closed(peer_id.into()).into());
            }
            RpcBehaviourEvent::ConnectionEstablished => {
                // send(P2pConnectionEvent::Finalized(peer_id.into(), Ok(())).into());
            }
            RpcBehaviourEvent::Stream {
                received,
                stream_id,
            } => {
                use libp2p_rpc_behaviour::Received;
                use mina_p2p_messages::{
                    rpc::{
                        AnswerSyncLedgerQueryV2, GetAncestryV2, GetBestTipV2,
                        GetStagedLedgerAuxAndPendingCoinbasesAtHashV2,
                        GetTransitionChainProofV1ForV2, GetTransitionChainV2,
                    },
                    rpc_kernel::{
                        Error as RpcError, NeedsLength, QueryHeader, QueryPayload, ResponseHeader,
                        ResponsePayload, RpcMethod,
                    },
                    v2,
                };

                let ch_send = send;
                let send = |msg: RpcChannelMsg| {
                    ch_send(P2pEvent::Channel(P2pChannelEvent::Received(
                        peer_id.into(),
                        Ok(ChannelMsg::Rpc(msg)),
                    )))
                };

                fn parse_q<M: RpcMethod>(bytes: Vec<u8>) -> Result<M::Query, String> {
                    let mut bytes = bytes.as_slice();
                    <QueryPayload<M::Query> as BinProtRead>::binprot_read(&mut bytes)
                        .map(|NeedsLength(x)| x)
                        .map_err(|err| format!("request {} {}", M::NAME_STR, err))
                }

                fn parse_r<M: RpcMethod>(
                    bytes: Vec<u8>,
                ) -> Result<Result<M::Response, RpcError>, String> {
                    let mut bytes = bytes.as_slice();
                    <ResponsePayload<M::Response> as BinProtRead>::binprot_read(&mut bytes)
                        .map(|x| x.0.map(|NeedsLength(x)| x))
                        .map_err(|err| format!("response {} {}", M::NAME_STR, err))
                }

                match received {
                    Received::Menu(_) => {}
                    Received::HandshakeDone => {}
                    Received::Query {
                        header: QueryHeader { tag, version, id },
                        bytes,
                    } => {
                        let tag = tag.to_string_lossy();

                        swarm
                            .behaviour_mut()
                            .ongoing_incoming
                            .insert((peer_id, id as _), (stream_id, tag.clone(), version));

                        let send =
                            |request: P2pRpcRequest| send(RpcChannelMsg::Request(id as _, request));

                        match (tag.as_bytes(), version) {
                            (GetBestTipV2::NAME, GetBestTipV2::VERSION) => {
                                send(P2pRpcRequest::BestTipWithProof)
                            }
                            (GetAncestryV2::NAME, GetAncestryV2::VERSION) => {
                                match parse_q::<GetAncestryV2>(bytes) {
                                    Ok(query) => {
                                        // TODO (vlad9486): Check query
                                        let _ = query.data; // must be equal current best tip consensus state
                                        let _ = query.hash; // must be equal best_tip.data.header.protocol_state.hash()
                                        send(P2pRpcRequest::BestTipWithProof)
                                    }
                                    Err(err) => send_error(err),
                                };
                            }
                            (AnswerSyncLedgerQueryV2::NAME, AnswerSyncLedgerQueryV2::VERSION) => {
                                match parse_q::<AnswerSyncLedgerQueryV2>(bytes) {
                                    Ok((hash, query)) => send(P2pRpcRequest::LedgerQuery(
                                        v2::MinaBaseLedgerHash0StableV1(hash).into(),
                                        query,
                                    )),
                                    Err(err) => send_error(err),
                                };
                            }
                            (
                                GetStagedLedgerAuxAndPendingCoinbasesAtHashV2::NAME,
                                GetStagedLedgerAuxAndPendingCoinbasesAtHashV2::VERSION,
                            ) => {
                                type T = GetStagedLedgerAuxAndPendingCoinbasesAtHashV2;
                                match parse_q::<T>(bytes) {
                                    Ok(hash) => send(
                                        P2pRpcRequest::StagedLedgerAuxAndPendingCoinbasesAtBlock(
                                            v2::DataHashLibStateHashStableV1(hash).into(),
                                        ),
                                    ),
                                    Err(err) => send_error(err),
                                };
                            }
                            (GetTransitionChainV2::NAME, GetTransitionChainV2::VERSION) => {
                                match parse_q::<GetTransitionChainV2>(bytes) {
                                    Ok(hashes) => {
                                        for hash in hashes {
                                            send(P2pRpcRequest::Block(
                                                v2::DataHashLibStateHashStableV1(hash).into(),
                                            ));
                                        }
                                    }
                                    Err(err) => send_error(err),
                                }
                            }
                            (
                                GetTransitionChainProofV1ForV2::NAME,
                                GetTransitionChainProofV1ForV2::VERSION,
                            ) => swarm
                                .behaviour_mut()
                                .rpc
                                .respond::<GetTransitionChainProofV1ForV2>(
                                    peer_id,
                                    stream_id,
                                    id,
                                    Ok(None),
                                )
                                .unwrap(),
                            (
                                GetSomeInitialPeersV1ForV2::NAME,
                                GetSomeInitialPeersV1ForV2::VERSION,
                            ) => match parse_q::<GetSomeInitialPeersV1ForV2>(bytes) {
                                Ok(()) => {
                                    send(P2pRpcRequest::InitialPeers);
                                }
                                Err(err) => send_error(err),
                            },
                            _ => (),
                        };
                    }
                    Received::Response {
                        header: ResponseHeader { id },
                        bytes,
                    } => {
                        let send = |response: Option<P2pRpcResponse>| {
                            send(RpcChannelMsg::Response(id as _, response))
                        };

                        let Some((tag, version)) =
                            swarm.behaviour_mut().ongoing.remove(&(peer_id, (id as _)))
                        else {
                            return;
                        };

                        match (tag, version) {
                            (GetBestTipV2::NAME, GetBestTipV2::VERSION) => {
                                match parse_r::<GetBestTipV2>(bytes) {
                                    Ok(response) => {
                                        let response = response
                                            .ok()
                                            .flatten()
                                            .map(|resp| BestTipWithProof {
                                                best_tip: resp.data.into(),
                                                proof: (resp.proof.0, resp.proof.1.into()),
                                            })
                                            .map(P2pRpcResponse::BestTipWithProof);
                                        send(response)
                                    }
                                    Err(err) => send_error(err),
                                }
                            }
                            (AnswerSyncLedgerQueryV2::NAME, AnswerSyncLedgerQueryV2::VERSION) => {
                                match parse_r::<AnswerSyncLedgerQueryV2>(bytes) {
                                    Ok(response) => {
                                        let response = response
                                            .ok()
                                            .map(|x| x.0.ok())
                                            .flatten()
                                            .map(P2pRpcResponse::LedgerQuery);
                                        send(response)
                                    }
                                    Err(err) => send_error(err),
                                }
                            }
                            (
                                GetStagedLedgerAuxAndPendingCoinbasesAtHashV2::NAME,
                                GetStagedLedgerAuxAndPendingCoinbasesAtHashV2::VERSION,
                            ) => {
                                type T = GetStagedLedgerAuxAndPendingCoinbasesAtHashV2;
                                match parse_r::<T>(bytes) {
                                    Ok(response) => {
                                        let response = response
                                        .ok()
                                        .flatten()
                                        .map(|(scan_state, hash, pending_coinbase, needed_blocks)| {
                                            let staged_ledger_hash =
                                                v2::MinaBaseLedgerHash0StableV1(hash).into();
                                            Arc::new(StagedLedgerAuxAndPendingCoinbases {
                                                scan_state,
                                                staged_ledger_hash,
                                                pending_coinbase,
                                                needed_blocks,
                                            })
                                        })
                                        .map(P2pRpcResponse::StagedLedgerAuxAndPendingCoinbasesAtBlock);
                                        send(response)
                                    }
                                    Err(err) => send_error(err),
                                }
                            }
                            (GetTransitionChainV2::NAME, GetTransitionChainV2::VERSION) => {
                                match parse_r::<GetTransitionChainV2>(bytes) {
                                    Ok(response) => {
                                        let response = response.ok().flatten().unwrap_or_default();
                                        if response.is_empty() {
                                            send(None)
                                        } else {
                                            for block in response {
                                                send(Some(P2pRpcResponse::Block(Arc::new(block))));
                                            }
                                        }
                                    }
                                    Err(err) => send_error(err),
                                }
                            }
                            (
                                GetSomeInitialPeersV1ForV2::NAME,
                                GetSomeInitialPeersV1ForV2::VERSION,
                            ) => {
                                match parse_r::<GetSomeInitialPeersV1ForV2>(bytes) {
                                    Ok(response) => {
                                        let response = response.ok().unwrap_or_default();
                                        if response.is_empty() {
                                            send(None)
                                        } else {
                                            let peers = response.into_iter().filter_map(P2pConnectionOutgoingInitOpts::try_from_mina_rpc).collect();
                                            send(Some(P2pRpcResponse::InitialPeers(peers)));
                                        }
                                    }
                                    Err(err) => send_error(err),
                                }
                            }
                            _ => send(None),
                        }
                    }
                }
            }
        }
    }

    pub fn cmd_sender(&mut self) -> &mut mpsc::UnboundedSender<Cmd> {
        &mut self.cmd_sender
    }
}
