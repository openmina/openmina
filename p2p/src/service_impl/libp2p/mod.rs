mod behavior;
pub use behavior::Event as BehaviourEvent;
pub use behavior::*;
use mina_p2p_messages::rpc::GetSomeInitialPeersV1ForV2;

mod discovery;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use mina_p2p_messages::binprot::{self, BinProtRead, BinProtWrite};
use mina_p2p_messages::v2::NetworkPoolSnarkPoolDiffVersionedStableV2;
use multihash::{Blake2b256, Hasher};
use openmina_core::channels::mpsc;
use openmina_core::snark::Snark;

use libp2p::core::muxing::StreamMuxerBox;
use libp2p::core::transport;
use libp2p::core::transport::upgrade;
use libp2p::futures::{select, FutureExt, StreamExt};
use libp2p::gossipsub::{
    Behaviour as Gossipsub, ConfigBuilder as GossipsubConfigBuilder, Event as GossipsubEvent,
    IdentTopic, MessageAcceptance, MessageAuthenticity,
};
use libp2p::identify;
use libp2p::identity::Keypair;
use libp2p::noise;
use libp2p::pnet::{PnetConfig, PreSharedKey};
use libp2p::swarm::dial_opts::DialOpts;
use libp2p::swarm::{SwarmBuilder, SwarmEvent};
use libp2p::yamux::YamuxConfig;
use libp2p::{PeerId, Swarm, Transport};

pub use mina_p2p_messages::gossip::GossipNetMessageV2 as GossipNetMessage;

use libp2p_rpc_behaviour::{BehaviourBuilder, Event as RpcBehaviourEvent, StreamId};

use crate::channels::best_tip::BestTipPropagationChannelMsg;
use crate::channels::rpc::{
    BestTipWithProof, P2pRpcRequest, P2pRpcResponse, RpcChannelMsg,
    StagedLedgerAuxAndPendingCoinbases,
};
use crate::channels::{ChannelId, ChannelMsg};
use crate::identity::SecretKey;
use crate::{P2pChannelEvent, P2pConnectionEvent, P2pEvent};

use super::TaskSpawner;

/// Type alias for libp2p transport
pub type P2PTransport = (PeerId, StreamMuxerBox);
/// Type alias for boxed libp2p transport
pub type BoxedP2PTransport = transport::Boxed<P2PTransport>;

#[derive(Debug)]
pub enum Cmd {
    Dial(DialOpts),
    Disconnect(PeerId),
    SendMessage(PeerId, ChannelMsg),
    SnarkBroadcast(Snark, u32),
}

pub struct Libp2pService {
    cmd_sender: mpsc::UnboundedSender<Cmd>,
}

impl Libp2pService {
    const GOSSIPSUB_TOPIC: &'static str = "coda/consensus-messages/0.0.1";

    async fn build_transport(
        chain_id: String,
        identity_keys: Keypair,
    ) -> Result<(BoxedP2PTransport, PeerId), std::io::Error> {
        let peer_id = identity_keys.public().to_peer_id();

        let yamux_config = {
            let mut c = YamuxConfig::default();
            c.set_protocol_name(b"/coda/yamux/1.0.0");
            c
        };

        use libp2p::{
            dns::TokioDnsConfig as DnsConfig,
            tcp::{tokio::Transport as TokioTcpTransport, Config as TcpConfig},
        };

        let tcp = TcpConfig::new().nodelay(true);
        let transport = DnsConfig::system(TokioTcpTransport::new(tcp))?;

        let pre_shared_key = {
            let mut hasher = Blake2b256::default();
            let rendezvous_string = format!("/coda/0.0.1/{}", chain_id);
            hasher.update(rendezvous_string.as_ref());
            let hash = hasher.finalize();
            let mut psk_fixed: [u8; 32] = Default::default();
            psk_fixed.copy_from_slice(hash.as_ref());
            PreSharedKey::new(psk_fixed)
        };
        let pnet_config = PnetConfig::new(pre_shared_key);

        let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
            .into_authentic(&identity_keys)
            .expect("Signing libp2p-noise static DH keypair failed.");

        Ok((
            transport
                .and_then(move |socket, _| pnet_config.handshake(socket))
                .upgrade(upgrade::Version::V1)
                .authenticate(libp2p::noise::NoiseConfig::xx(noise_keys).into_authenticated())
                .multiplex(yamux_config)
                .timeout(Duration::from_secs(60))
                .boxed(),
            peer_id,
        ))
    }

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
            event_source_sender,
            ongoing: BTreeMap::default(),
            ongoing_incoming: BTreeMap::default(),
        };

        let (cmd_sender, mut cmd_receiver) = mpsc::unbounded_channel();

        let fut = async move {
            let (transport, id) = Self::build_transport(chain_id, identity_keys)
                .await
                .unwrap();

            let mut swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, id).build();
            if let Some(port) = libp2p_port {
                if let Err(err) =
                    swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{port}").parse().unwrap())
                {
                    openmina_core::log::error!(
                        openmina_core::log::system_time();
                        kind = "Libp2pListenError",
                        summary = format!("libp2p failed to start listener at port: {port}. error: {err:?}"),
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
            Cmd::Dial(maddr) => {
                if let Err(e) = swarm.dial(maddr) {
                    openmina_core::log::error!(
                        openmina_core::log::system_time();
                        event = format!("Cmd::Dial(...)"),
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
                let id = id as i64;

                match req {
                    P2pRpcRequest::BestTipWithProof => {
                        type T = GetBestTipV2;
                        b.ongoing.insert(key, (T::NAME.to_string(), T::VERSION));
                        b.rpc.query::<T>(peer_id, stream_id, id, ())?;
                    }
                    P2pRpcRequest::LedgerQuery(hash, query) => {
                        type T = AnswerSyncLedgerQueryV2;
                        b.ongoing.insert(key, (T::NAME.to_string(), T::VERSION));
                        let query = (hash.0.clone(), query);
                        b.rpc.query::<T>(peer_id, stream_id, id, query)?;
                    }
                    P2pRpcRequest::StagedLedgerAuxAndPendingCoinbasesAtBlock(hash) => {
                        type T = GetStagedLedgerAuxAndPendingCoinbasesAtHashV2;
                        b.ongoing.insert(key, (T::NAME.to_string(), T::VERSION));
                        let query = hash.0.clone();
                        b.rpc.query::<T>(peer_id, stream_id, id, query)?;
                    }
                    P2pRpcRequest::Block(hash) => {
                        type T = GetTransitionChainV2;
                        b.ongoing.insert(key, (T::NAME.to_string(), T::VERSION));
                        let query = vec![hash.0.clone()];
                        b.rpc.query::<T>(peer_id, stream_id, id, query)?;
                    }
                    P2pRpcRequest::Snark(_) => {}
                    P2pRpcRequest::InitialPeers => {
                        type T = GetSomeInitialPeersV1ForV2;
                        b.ongoing.insert(key, (T::NAME.to_string(), T::VERSION));
                        b.rpc.query::<T>(peer_id, stream_id, id, ())?;
                    }
                };
            }
            RpcChannelMsg::Response(id, resp) => {
                if let Some((stream_id, tag, version)) = b.ongoing_incoming.remove(&(peer_id, id)) {
                    let id = id as i64;
                    match resp {
                        None => match (tag.as_str(), version) {
                            (GetBestTipV2::NAME, GetBestTipV2::VERSION) => {
                                type T = GetBestTipV2;
                                b.rpc.respond::<T>(peer_id, stream_id, id, Ok(None))?
                            }
                            (GetAncestryV2::NAME, GetAncestryV2::VERSION) => {
                                type T = GetAncestryV2;
                                b.rpc.respond::<T>(peer_id, stream_id, id, Ok(None))?
                            }
                            (AnswerSyncLedgerQueryV2::NAME, AnswerSyncLedgerQueryV2::VERSION) => {
                                type T = AnswerSyncLedgerQueryV2;
                                b.rpc.respond::<T>(
                                    peer_id,
                                    stream_id,
                                    id,
                                    Ok(RpcResult(Err(Info::String(Vec::new().into())))),
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
                            if tag == GetAncestryV2::NAME {
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
                            let r = Ok(peers);
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
            }
            SwarmEvent::ListenerError { listener_id, error } => {
                let listener_id = format!("{listener_id:?}");
                openmina_core::log::error!(
                    openmina_core::log::system_time();
                    kind = "Libp2pListenError",
                    summary = format!("libp2p.{listener_id:?} listener error: {error:?}"),
                    listener_id = listener_id,
                );
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
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                openmina_core::log::info!(
                    openmina_core::log::system_time();
                    kind = "PeerConnected",
                    summary = format!("peer_id: {}", peer_id),
                    peer_id = peer_id.to_string()
                );
                let event =
                    P2pEvent::Connection(P2pConnectionEvent::Finalized(peer_id.into(), Ok(())));
                swarm.behaviour_mut().identify.push(Some(peer_id));
                let _ = swarm.behaviour_mut().event_source_sender.send(event.into());
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                let event = P2pEvent::Connection(P2pConnectionEvent::Closed(peer_id.into()));
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
            SwarmEvent::OutgoingConnectionError { peer_id, error } => {
                let peer_id = match peer_id {
                    Some(v) => v,
                    None => return,
                };
                let event = P2pEvent::Connection(P2pConnectionEvent::Finalized(
                    peer_id.into(),
                    Err(error.to_string()),
                ));
                let _ = swarm.behaviour_mut().event_source_sender.send(event.into());
            }
            SwarmEvent::Behaviour(event) => match event {
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
                        .map_err(|err| format!("request {} {}", M::NAME, err))
                }

                fn parse_r<M: RpcMethod>(
                    bytes: Vec<u8>,
                ) -> Result<Result<M::Response, RpcError>, String> {
                    let mut bytes = bytes.as_slice();
                    <ResponsePayload<M::Response> as BinProtRead>::binprot_read(&mut bytes)
                        .map(|x| x.0.map(|NeedsLength(x)| x))
                        .map_err(|err| format!("response {} {}", M::NAME, err))
                }

                match received {
                    Received::Menu(_) => {}
                    Received::HandshakeDone => {
                        ch_send(
                            P2pChannelEvent::Opened(peer_id.into(), ChannelId::Rpc, Ok(())).into(),
                        );
                    }
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

                        match (tag.as_str(), version) {
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

                        match (tag.as_str(), version) {
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
                            ) => match parse_r::<GetSomeInitialPeersV1ForV2>(bytes) {
                                Ok(response) => {
                                    let response = response.ok().unwrap_or_default();
                                    if response.is_empty() {
                                        send(None)
                                    } else {
                                        send(Some(P2pRpcResponse::InitialPeers(response)));
                                    }
                                }
                                Err(err) => send_error(err),
                            },
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
