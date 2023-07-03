use std::time::Duration;

use binprot::{BinProtRead, BinProtWrite};
use multihash::{Blake2b256, Hasher};
use tokio::sync::mpsc;

use libp2p::core::muxing::StreamMuxerBox;
use libp2p::core::transport;
use libp2p::core::transport::upgrade;
use libp2p::futures::{select, FutureExt, StreamExt};
use libp2p::gossipsub::{
    Behaviour as Gossipsub, ConfigBuilder as GossipsubConfigBuilder, Event as GossipsubEvent,
    IdentTopic, MessageAuthenticity,
};
use libp2p::identity::Keypair;
use libp2p::noise;
use libp2p::pnet::{PnetConfig, PreSharedKey};
use libp2p::swarm::dial_opts::DialOpts;
use libp2p::swarm::{SwarmBuilder, SwarmEvent};
use libp2p::yamux::YamuxConfig;
use libp2p::{PeerId, Swarm, Transport};

pub use mina_p2p_messages::gossip::GossipNetMessageV2 as GossipNetMessage;

mod behavior;
pub use behavior::Event as BehaviourEvent;
pub use behavior::*;
use tokio_util::task::LocalPoolHandle;

pub mod rpc;
use self::rpc::RpcBehaviour;

use crate::channels::best_tip::BestTipPropagationChannelMsg;
use crate::channels::rpc::RpcChannelMsg;
use crate::channels::ChannelMsg;
use crate::{P2pChannelEvent, P2pConnectionEvent, P2pEvent};

/// Type alias for libp2p transport
pub type P2PTransport = (PeerId, StreamMuxerBox);
/// Type alias for boxed libp2p transport
pub type BoxedP2PTransport = transport::Boxed<P2PTransport>;

#[derive(Debug)]
pub enum Cmd {
    SendMessage(PeerId, ChannelMsg),
    Dial(DialOpts),
    Disconnect(PeerId),
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
            tcp::{Config as TcpConfig, TokioTcpTransport},
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

    pub fn run<E>(
        chain_id: String,
        event_source_sender: mpsc::UnboundedSender<E>,
        rt_pool: &LocalPoolHandle,
    ) -> Self
    where
        E: 'static + Send + From<P2pEvent>,
    {
        let topics_iter = IntoIterator::into_iter([
            Self::GOSSIPSUB_TOPIC,
            "mina/block/1.0.0",
            "mina/tx/1.0.0",
            "mina/snark-work/1.0.0",
        ]);

        let identity_keys = Keypair::generate_ed25519();

        let message_authenticity = MessageAuthenticity::Signed(identity_keys.clone());
        let gossipsub_config = GossipsubConfigBuilder::default()
            .max_transmit_size(1024 * 1024 * 32)
            .build()
            .unwrap();
        let mut gossipsub: Gossipsub =
            Gossipsub::new(message_authenticity, gossipsub_config).unwrap();
        topics_iter
            .map(|v| IdentTopic::new(v))
            .for_each(|topic| assert!(gossipsub.subscribe(&topic).unwrap()));

        let behaviour = Behaviour {
            gossipsub,
            rpc: RpcBehaviour::new(),
            event_source_sender,
        };

        let (cmd_sender, mut cmd_receiver) = mpsc::unbounded_channel();

        let fut = async move {
            let (transport, id) = Self::build_transport(chain_id, identity_keys)
                .await
                .unwrap();

            let mut swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, id).build();

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

        rt_pool.spawn_pinned(move || fut);

        Self { cmd_sender }
    }

    fn gossipsub_send<T, E>(swarm: &mut Swarm<Behaviour<E>>, prefix: u8, msg: &T)
    where
        T: BinProtWrite,
        E: From<P2pEvent>,
    {
        let mut encoded = vec![0; 9];
        encoded[8] = prefix;
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
                swarm.dial(maddr).unwrap();
            }
            Cmd::Disconnect(peer_id) => {
                let _ = swarm.disconnect_peer_id(peer_id);
            }
            Cmd::SendMessage(peer_id, msg) => match msg {
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
                        Self::gossipsub_send(swarm, 0, &*block);
                        // TODO(binier): send event: `P2pChannelEvent::Sent`
                    }
                },
                ChannelMsg::Rpc(msg) => match msg {
                    RpcChannelMsg::Request(id, req) => {
                        swarm.behaviour_mut().rpc.send_request(peer_id, id, req);
                    }
                    RpcChannelMsg::Response(_id, _resp) => {
                        // TODO(binier): respond to incoming rpcs.
                    }
                },
            },
        }
    }

    async fn handle_event<E: From<P2pEvent>, Err: std::error::Error>(
        swarm: &mut Swarm<Behaviour<E>>,
        event: SwarmEvent<BehaviourEvent, Err>,
    ) {
        match event {
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                shared::log::info!(
                    shared::log::system_time();
                    kind = "PeerConnected",
                    summary = format!("peer_id: {}", peer_id),
                    peer_id = peer_id.to_string()
                );
                let event = if endpoint.is_dialer() {
                    P2pEvent::Connection(P2pConnectionEvent::Finalized(peer_id.into(), Ok(())))
                } else {
                    // TODO(binier): connected incoming
                    return;
                };
                let _ = swarm.behaviour_mut().event_source_sender.send(event.into());
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                let event = P2pEvent::Connection(P2pConnectionEvent::Closed(peer_id.into()));
                let _ = swarm.behaviour_mut().event_source_sender.send(event.into());

                // TODO(binier): move to log effects
                shared::log::info!(
                    shared::log::system_time();
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
                    message_id: _,
                    message,
                }) => {
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
                        _ => return,
                    };

                    let event = P2pEvent::Channel(P2pChannelEvent::Received(
                        propagation_source.into(),
                        res,
                    ));
                    let _ = swarm.behaviour_mut().event_source_sender.send(event.into());
                }
                BehaviourEvent::Rpc(event) => {
                    let _ = swarm
                        .behaviour_mut()
                        .event_source_sender
                        .send(P2pEvent::from(event).into());
                }
                _ => {
                    shared::log::trace!(
                        shared::log::system_time();
                        kind = "IgnoredLibp2pBehaviorEvent",
                        event = format!("{:?}", event)
                    );
                }
            },
            event => {
                shared::log::trace!(
                    shared::log::system_time();
                    kind = "IgnoredLibp2pSwarmEvent",
                    event = format!("{:?}", event)
                );
            }
        }
    }

    pub fn cmd_sender(&mut self) -> &mut mpsc::UnboundedSender<Cmd> {
        &mut self.cmd_sender
    }
}
