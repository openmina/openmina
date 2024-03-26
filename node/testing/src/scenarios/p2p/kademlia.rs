use std::{net::Ipv4Addr, time::Duration};

use libp2p::{
    futures::StreamExt,
    identity::Keypair,
    pnet::PreSharedKey,
    swarm::{NetworkBehaviour, SwarmEvent},
    Transport,
};
use multiaddr::{multiaddr, Multiaddr};
use node::p2p::{
    connection::outgoing::{P2pConnectionOutgoingInitLibp2pOpts, P2pConnectionOutgoingInitOpts},
    identity::SecretKey,
    webrtc::Host,
};

use crate::{
    node::RustNodeTestingConfig,
    scenario::ListenerNode,
    scenarios::{trace_steps, wait_for_nodes_listening_on_localhost, ClusterRunner, Driver},
};

const LOCALHOST: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

/// Incoming FIND_NODE request test.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct IncomingFindNode;

impl IncomingFindNode {
    pub async fn run<'cluster>(self, runner: ClusterRunner<'cluster>) {
        let mut driver = Driver::new(runner);
        let (node1, peer_id1) = driver.add_rust_node(
            RustNodeTestingConfig::berkeley_default().initial_peers(
                (0..100)
                    .map(|n| {
                        let peer_id = SecretKey::rand().public_key().peer_id();
                        let port = 12000 + n;
                        let host = Host::Ipv4([127, 0, 0, 1].into());
                        ListenerNode::Custom(P2pConnectionOutgoingInitOpts::LibP2P(
                            P2pConnectionOutgoingInitLibp2pOpts {
                                peer_id,
                                host,
                                port,
                            },
                        ))
                    })
                    .collect(),
            ),
        );

        let addr = format!(
            "/ip4/127.0.0.1/tcp/{}/p2p/{}",
            driver
                .inner()
                .node(node1)
                .unwrap()
                .state()
                .p2p
                .config
                .libp2p_port
                .unwrap(),
            peer_id1.to_libp2p_string(),
        )
        .parse::<Multiaddr>()
        .unwrap();

        assert!(
            wait_for_nodes_listening_on_localhost(&mut driver, Duration::from_secs(30), [node1])
                .await
                .unwrap(),
            "node should be listening"
        );

        let identity_key =
            Keypair::ed25519_from_bytes([0xba; 32]).expect("secret key bytes must be valid");

        let handle = tokio::spawn(async move {
            let mut fake_peer = fake_kad_peer(identity_key, None).unwrap();
            fake_peer.dial(addr.clone()).unwrap();
            fake_peer
                .behaviour_mut()
                .kademlia
                .add_address(&peer_id1.clone().into(), addr);
            loop {
                let next = fake_peer.next().await.unwrap();
                println!("<<< {next:?}");
                match next {
                    SwarmEvent::ConnectionEstablished { peer_id: _, .. } => {
                        fake_peer.behaviour_mut().kademlia.bootstrap().unwrap();
                    }
                    SwarmEvent::Behaviour(Event::Kademlia(
                        libp2p::kad::Event::OutboundQueryProgressed { stats, .. },
                    )) => {
                        return (stats.num_successes() >= 1)
                            .then_some(())
                            .ok_or(format!("incorrect query stats: {stats:?}"));
                    }
                    _ => {}
                }
            }
        });

        tokio::select! {
            res = trace_steps(driver.inner_mut()) => {
                panic!("statemachine finished unexpectedly: {res:?}");
            }
            res = tokio::time::timeout(Duration::from_secs(20), handle) => {
                let res = res.expect("timeout waiting for kad query result");
                if let Err(err) = res {
                    panic!("error from peer: {err}");
                }
            }
        }
    }
}

/// Kademlia bootstrap test.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct KademliaBootstrap;

impl KademliaBootstrap {
    pub async fn run<'cluster>(self, runner: ClusterRunner<'cluster>) {
        const NUM: u8 = 10;
        let identity_key =
            Keypair::ed25519_from_bytes([0xba; 32]).expect("secret key bytes must be valid");
        let peer_id = identity_key.public().to_peer_id();

        let mut fake_peer = fake_kad_peer(identity_key, Some(13000)).unwrap();

        for n in 1..NUM + 1 {
            let mut bytes = [0; 32];
            bytes[0] = n;
            let peer_id = SecretKey::from_bytes(bytes).public_key().peer_id();
            fake_peer.behaviour_mut().kademlia.add_address(
                &peer_id.into(),
                multiaddr!(Ip4(LOCALHOST), Tcp(12000 + (n as u16))),
            );
        }

        let mut driver = Driver::new(runner);

        let (node1, _peer_id1) =
            driver.add_rust_node(RustNodeTestingConfig::berkeley_default().initial_peers(
                FromIterator::from_iter([ListenerNode::Custom(
                    P2pConnectionOutgoingInitOpts::LibP2P(P2pConnectionOutgoingInitLibp2pOpts {
                        peer_id: peer_id.clone().into(),
                        host: Host::Ipv4(LOCALHOST),
                        port: 13000,
                    }),
                )]),
            ));

        assert!(
            wait_for_nodes_listening_on_localhost(&mut driver, Duration::from_secs(30), [node1])
                .await
                .unwrap(),
            "node should be listening"
        );

        // wait for listener to be ready
        tokio::time::timeout(
            Duration::from_secs(1),
            (&mut fake_peer)
                .any(|event| async move { matches!(event, SwarmEvent::NewListenAddr { .. }) }),
        )
        .await
        .expect("should be listening");

        let handle = tokio::spawn(async move {
            loop {
                let next = fake_peer.next().await.unwrap();
                println!("<<< {next:?}");
                match next {
                    SwarmEvent::ConnectionEstablished { .. } => {}
                    SwarmEvent::Behaviour(Event::Kademlia(
                        libp2p::kad::Event::InboundRequest { .. },
                    )) => {
                        // return Result::<_, String>::Ok(());
                    }
                    _ => {}
                }
            }
        });

        tokio::select! {
            res = trace_steps(driver.inner_mut()) => {
                panic!("statemachine finished unexpectedly: {res:?}");
            }
            res = tokio::time::timeout(Duration::from_secs(20000), handle) => {
                let res = res.expect("timeout waiting for kad query result");
                if let Err(err) = res {
                    panic!("error from peer: {err}");
                }
            }
        }
    }
}

fn fake_kad_peer(
    identity_key: Keypair,
    port: Option<u16>,
) -> anyhow::Result<libp2p::Swarm<Behaviour>> {
    let psk = PreSharedKey::new(openmina_core::preshared_key(openmina_core::CHAIN_ID));
    let identify = libp2p::identify::Behaviour::new(libp2p::identify::Config::new(
        "ipfs/0.1.0".to_string(),
        identity_key.public(),
    ));

    let peer_id = identity_key.public().to_peer_id();
    println!("======== peer_id: {peer_id}");
    println!(
        "======== peer_id bytes: {}",
        hex::encode(peer_id.clone().to_bytes())
    );
    let kad_config = {
        let mut c = libp2p::kad::Config::default();
        c.set_protocol_names(vec![libp2p::StreamProtocol::new("/coda/kad/1.0.0")]);
        c
    };
    let mut kademlia = libp2p::kad::Behaviour::with_config(
        peer_id,
        libp2p::kad::store::MemoryStore::new(peer_id),
        kad_config,
    );

    if port.is_some() {
        kademlia.set_mode(Some(libp2p::kad::Mode::Server));
    }

    let behaviour = Behaviour {
        // identify,
        kademlia,
    };

    let swarm = libp2p::SwarmBuilder::with_existing_identity(identity_key)
        .with_tokio()
        .with_other_transport(|key| {
            let noise_config = libp2p::noise::Config::new(key).unwrap();
            let mut yamux_config = libp2p::yamux::Config::default();

            yamux_config.set_protocol_name("/coda/yamux/1.0.0");

            let mut base_transport = libp2p::tcp::tokio::Transport::new(
                libp2p::tcp::Config::default()
                    .nodelay(true)
                    .port_reuse(true),
            );

            if let Some(port) = port {
                base_transport
                    .listen_on(
                        libp2p::core::transport::ListenerId::next(),
                        multiaddr!(Ip4([127, 0, 0, 1]), Tcp(port)),
                    )
                    .expect("listen");
            }

            base_transport
                .and_then(move |socket, _| libp2p::pnet::PnetConfig::new(psk).handshake(socket))
                .upgrade(libp2p::core::upgrade::Version::V1)
                .authenticate(noise_config)
                .multiplex(yamux_config)
                .timeout(Duration::from_secs(60))
        })?
        .with_dns()?
        .with_behaviour(|_| behaviour)?
        .with_swarm_config(|config| {
            config.with_idle_connection_timeout(Duration::from_millis(1000))
        })
        .build();

    //swarm.behaviour_mut().kademlia.set_mode(Some(Mode::Server));

    Ok(swarm)
}

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "Event")]
pub struct Behaviour {
    // pub gossipsub: gossipsub::Behaviour,
    // pub rpc: RpcBehaviour,
    // pub identify: libp2p::identify::Behaviour,
    pub kademlia: libp2p::kad::Behaviour<libp2p::kad::store::MemoryStore>,
}

#[derive(Debug, derive_more::From)]
pub enum Event {
    // Identify(IdentifyEvent),
    // Gossipsub(gossipsub::Event),
    // Rpc((PeerId, RpcEvent)),
    // Identify(libp2p::identify::Event),
    Kademlia(libp2p::kad::Event),
}
