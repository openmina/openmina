use std::convert::Infallible;
use std::time::Duration;

use wasm_bindgen_futures::spawn_local;

use libp2p::core::muxing::StreamMuxerBox;
use libp2p::core::transport;
use libp2p::core::transport::upgrade;
use libp2p::futures::channel::mpsc;
use libp2p::futures::{select, SinkExt, StreamExt};
use libp2p::gossipsub::{
    Gossipsub, GossipsubConfigBuilder, GossipsubEvent, IdentTopic, MessageAuthenticity,
};
use libp2p::identity::{self, Keypair};
use libp2p::swarm::dial_opts::DialOpts;
use libp2p::swarm::{SwarmBuilder, SwarmEvent};
use libp2p::wasm_ext::ffi::ManualConnector as JsManualConnector;
use libp2p::yamux::YamuxConfig;
use libp2p::{build_multiaddr, PeerId, Swarm, Transport};

pub use mina_p2p_messages::gossip::GossipNetMessageV2 as GossipNetMessage;

use lib::event_source::{Event, P2pConnectionEvent, P2pEvent, P2pPubsubEvent};
use lib::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;
use lib::p2p::pubsub::PubsubTopic;
use lib::p2p::rpc::{P2pRpcId, P2pRpcRequest};
use lib::service::{
    P2pConnectionService, P2pDisconnectionService, P2pPubsubService, P2pRpcService,
};

mod behavior;
pub use behavior::Event as BehaviourEvent;
pub use behavior::*;

pub mod rpc;

use crate::NodeWasmService;

use self::rpc::RpcBehaviour;

/// Type alias for libp2p transport
pub type P2PTransport = (PeerId, StreamMuxerBox);
/// Type alias for boxed libp2p transport
pub type BoxedP2PTransport = transport::Boxed<P2PTransport>;

#[derive(Debug)]
pub enum Cmd {
    SendMessage(CmdSendMessage),
    Dial(DialOpts),
    Disconnect(PeerId),
}

#[derive(Debug)]
pub enum CmdSendMessage {
    Gossipsub(PubsubTopic, Vec<u8>),
    RpcRequest(PeerId, P2pRpcId, P2pRpcRequest),
}

pub struct Libp2pService {
    cmd_sender: mpsc::Sender<Cmd>,
}

impl Libp2pService {
    async fn build_transport(
        identity_keys: Keypair,
    ) -> Result<(BoxedP2PTransport, PeerId, JsManualConnector), std::io::Error> {
        let peer_id = identity_keys.public().to_peer_id();
        let (transport, manual_connector) = {
            cfg_if::cfg_if! {
                if #[cfg(target_arch = "wasm32")] {
                    use libp2p::wasm_ext;

                    // Note that DNS has been implictly supported in the extended javascript code,
                    // and TCP is not feasible in browsers
                    // const transport = match JsFuture::from(wasm_ext::ffi::webrtc_transport()).await {
                    //     Ok(v) => v.,
                    //     Err(err) => panic!(err),
                    // };

                    let transport = wasm_ext::webrtc_transport(identity_keys).await;
                    let manual_connector = transport.manual_connector();
                    (wasm_ext::ExtTransport::new(transport), manual_connector)
                } else {
                    todo!()
                }
            }
        };

        let yamux_config = {
            let mut c = YamuxConfig::default();
            c.set_protocol_name(b"/coda/yamux/1.0.0");
            c
        };

        Ok((
            transport
                // .and_then(move |socket, _| self.pnet_config.handshake(socket))
                // .and_then(|c, _| {
                //     let result = c.remote_peer_id()
                //         .map(|peer_id| (peer_id.clone(), c))
                //         .ok_or(std::io::Error::new(std::io::ErrorKind::Other, "not preauthenticated"));
                //     std::future::ready(result)
                // })
                .and_then(|c, _| {
                    std::future::ready(Result::<_, Infallible>::Ok((c.remote_peer_id().clone(), c)))
                })
                .upgrade(upgrade::Version::V1)
                .as_authenticated()
                .multiplex(yamux_config)
                .timeout(Duration::from_secs(60))
                .boxed(),
            peer_id,
            manual_connector,
        ))
    }

    pub async fn run(event_source_sender: mpsc::Sender<Event>) -> (Self, JsManualConnector) {
        let gossipsub_topic = "coda/consensus-messages/0.0.1";
        let topics_iter = IntoIterator::into_iter([
            gossipsub_topic,
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
            // identify: Identify::new(IdentifyConfig::new(
            //     "/ipfs/id/1.0.0".into(),
            //     config.keypair.public(),
            // )),
            gossipsub,
            rpc: RpcBehaviour::new(),

            event_source_sender,
        };

        let (transport, id, manual_connector) = Self::build_transport(identity_keys).await.unwrap();

        let mut swarm = SwarmBuilder::new(transport, behaviour, id).build();

        let (cmd_sender, mut cmd_receiver) = mpsc::channel(128);

        swarm.listen_on(build_multiaddr!(P2pWebRtcDirect));
        spawn_local(async move {
            loop {
                select! {
                    event = swarm.next() => match event {
                        Some(event) => Self::handle_event(&mut swarm, event).await,
                        None => break,
                    },
                    cmd = cmd_receiver.next() => {
                        match cmd {
                            Some(Cmd::Dial(maddr)) => {
                                swarm.dial(maddr).unwrap();
                            }
                            Some(Cmd::Disconnect(peer_id)) => {
                                let _ = swarm.disconnect_peer_id(peer_id);
                            }
                            Some(Cmd::SendMessage(msg)) => match msg {
                                CmdSendMessage::Gossipsub(topic, msg) => {
                                    swarm.behaviour_mut().gossipsub.publish(topic, msg).unwrap();
                                }
                                CmdSendMessage::RpcRequest(peer_id, id, req) => {
                                    // TODO(binier): handle if is_some
                                    swarm.behaviour_mut().rpc.send_request(peer_id, id, req);
                                }
                            }
                            None => break,
                        }
                    }
                }
            }
        });

        (Self { cmd_sender }, manual_connector)
    }

    async fn handle_event<E: std::error::Error>(
        swarm: &mut Swarm<Behaviour>,
        event: SwarmEvent<BehaviourEvent, E>,
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
                    Event::P2p(P2pEvent::Connection(P2pConnectionEvent::OutgoingInit(
                        peer_id,
                        Ok(()),
                    )))
                } else {
                    // TODO(binier): connected incoming
                    return;
                };
                swarm.behaviour_mut().event_source_sender.send(event).await;
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                let event = Event::P2p(P2pEvent::Connection(P2pConnectionEvent::Closed(peer_id)));
                swarm.behaviour_mut().event_source_sender.send(event).await;

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
                let event = Event::P2p(P2pEvent::Connection(P2pConnectionEvent::OutgoingInit(
                    peer_id,
                    Err(error.to_string()),
                )));
                swarm.behaviour_mut().event_source_sender.send(event).await;
            }
            SwarmEvent::IncomingConnectionError {
                send_back_addr,
                error,
                ..
            } => {
                shared::log::info!(
                    shared::log::system_time();
                    kind = "PeerConnectionIncomingError",
                    summary = format!("peer_addr: {}", send_back_addr.to_string())
                );
                // TODO(binier)
            }
            SwarmEvent::Behaviour(event) => match event {
                BehaviourEvent::Gossipsub(GossipsubEvent::Message {
                    propagation_source,
                    message_id,
                    message,
                }) => {
                    let event = Event::P2p(P2pEvent::Pubsub(P2pPubsubEvent::BytesReceived {
                        author: message.source.unwrap(),
                        sender: propagation_source,
                        topic: message.topic.as_str().parse().unwrap(),
                        bytes: message.data,
                    }));
                    swarm.behaviour_mut().event_source_sender.send(event).await;
                }
                BehaviourEvent::Rpc(event) => {
                    let event = Event::P2p(P2pEvent::Rpc(event));
                    swarm.behaviour_mut().event_source_sender.send(event).await;
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
}

impl P2pConnectionService for NodeWasmService {
    fn outgoing_init(&mut self, opts: P2pConnectionOutgoingInitOpts) {
        let opts = DialOpts::peer_id(opts.peer_id)
            .addresses(opts.addrs)
            .build();
        let cmd = Cmd::Dial(opts);
        let mut tx = self.libp2p.cmd_sender.clone();
        spawn_local(async move {
            tx.send(cmd).await;
        });
    }

    fn random_pick(
        &mut self,
        list: &[P2pConnectionOutgoingInitOpts],
    ) -> P2pConnectionOutgoingInitOpts {
        use rand::seq::SliceRandom;
        list.choose(&mut self.rng).unwrap().clone()
    }
}

impl P2pDisconnectionService for NodeWasmService {
    fn disconnect(&mut self, peer_id: PeerId) {
        let cmd = Cmd::Disconnect(peer_id);
        let mut tx = self.libp2p.cmd_sender.clone();
        spawn_local(async move {
            tx.send(cmd).await;
        });
    }
}

impl P2pPubsubService for NodeWasmService {
    fn publish(&mut self, topic: PubsubTopic, bytes: Vec<u8>) {
        let mut tx = self.libp2p.cmd_sender.clone();
        spawn_local(async move {
            tx.send(Cmd::SendMessage(CmdSendMessage::Gossipsub(topic, bytes)))
                .await;
        });
    }
}

impl P2pRpcService for NodeWasmService {
    fn outgoing_init(&mut self, peer_id: PeerId, id: P2pRpcId, req: P2pRpcRequest) {
        let mut tx = self.libp2p.cmd_sender.clone();
        spawn_local(async move {
            tx.send(Cmd::SendMessage(CmdSendMessage::RpcRequest(
                peer_id, id, req,
            )))
            .await;
        });
    }
}
