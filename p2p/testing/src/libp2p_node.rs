use std::{collections::BTreeMap, error::Error, time::Duration};

use libp2p::{
    gossipsub, identify,
    swarm::{NetworkBehaviour, SwarmEvent, THandlerErr},
    Transport,
};
use mina_p2p_messages::rpc_kernel::RpcTag;
use openmina_core::ChainId;
use p2p::PeerId;

use libp2p_rpc_behaviour::StreamId;

use libp2p::kad::{self, record::store::MemoryStore};

use crate::{cluster::PeerIdConfig, test_node::TestNode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Libp2pNodeId(pub(super) usize);

#[derive(Debug, Default, Clone)]
pub struct Libp2pNodeConfig {
    pub peer_id: PeerIdConfig,
    pub port_reuse: bool,
}

pub type Swarm = libp2p::Swarm<Libp2pBehaviour>;

pub struct Libp2pNode {
    swarm: Swarm,
}

impl Libp2pNode {
    pub(super) fn new(swarm: Swarm) -> Self {
        Libp2pNode { swarm }
    }

    pub fn swarm(&self) -> &Swarm {
        &self.swarm
    }

    pub fn swarm_mut(&mut self) -> &mut Swarm {
        &mut self.swarm
    }
}

impl TestNode for Libp2pNode {
    fn peer_id(&self) -> PeerId {
        self.swarm.local_peer_id().clone().into()
    }

    fn libp2p_port(&self) -> u16 {
        self.swarm.behaviour().port
    }
}

pub type Libp2pEvent = SwarmEvent<Libp2pBehaviourEvent, THandlerErr<Libp2pBehaviour>>;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, derive_more::From)]
pub enum Libp2pBehaviourEvent {
    // Identify(IdentifyEvent),
    Gossipsub(gossipsub::Event),
    Rpc((libp2p::identity::PeerId, libp2p_rpc_behaviour::Event)),
    Identify(identify::Event),
    Kademlia(kad::Event),
}

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "Libp2pBehaviourEvent")]
pub struct Libp2pBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub rpc: libp2p_rpc_behaviour::Behaviour,
    pub identify: identify::Behaviour,
    pub kademlia: kad::Behaviour<MemoryStore>,

    #[behaviour(ignore)]
    pub chain_id: ChainId,

    #[behaviour(ignore)]
    port: u16,

    // TODO(vlad9486): move maps inside `RpcBehaviour`
    // map msg_id into (tag, version)
    #[behaviour(ignore)]
    pub ongoing: BTreeMap<(PeerId, u64), (RpcTag, u32)>,
    // map from (peer, msg_id) into (stream_id, tag, version)
    //
    #[behaviour(ignore)]
    pub ongoing_incoming: BTreeMap<(PeerId, u64), (StreamId, String, u32)>,
}

pub(crate) fn create_swarm(
    secret_key: p2p::identity::SecretKey,
    port: u16,
    port_reuse: bool,
    chain_id: ChainId,
) -> Result<Swarm, Box<dyn Error>> {
    let identity_keys = libp2p::identity::Keypair::ed25519_from_bytes(secret_key.to_bytes())
        .expect("secret key bytes must be valid");

    let psk = libp2p::pnet::PreSharedKey::new(openmina_core::preshared_key(chain_id.clone()));
    let identify = libp2p::identify::Behaviour::new(libp2p::identify::Config::new(
        "ipfs/0.1.0".to_string(),
        identity_keys.public(),
    ));

    let kademlia = {
        let peer_id = identity_keys.public().to_peer_id();
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

        kademlia.set_mode(Some(libp2p::kad::Mode::Server));
        kademlia
    };

    // gossipsub
    let gossipsub = {
        let message_authenticity = gossipsub::MessageAuthenticity::Signed(identity_keys.clone());
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .max_transmit_size(1024 * 1024 * 32)
            .validate_messages()
            .build()
            .unwrap();
        let mut gossipsub: gossipsub::Behaviour =
            gossipsub::Behaviour::new(message_authenticity, gossipsub_config).unwrap();

        gossipsub
            .subscribe(&gossipsub::IdentTopic::new("coda/consensus-messages/0.0.1"))
            .expect("subscribe");

        gossipsub
    };

    // rpc
    let rpc = {
        use mina_p2p_messages::rpc::{
            AnswerSyncLedgerQueryV2, GetAncestryV2, GetBestTipV2,
            GetStagedLedgerAuxAndPendingCoinbasesAtHashV2, GetTransitionChainProofV1ForV2,
            GetTransitionChainV2,
        };

        libp2p_rpc_behaviour::BehaviourBuilder::default()
            .register_method::<GetBestTipV2>()
            .register_method::<GetAncestryV2>()
            .register_method::<GetStagedLedgerAuxAndPendingCoinbasesAtHashV2>()
            .register_method::<AnswerSyncLedgerQueryV2>()
            .register_method::<GetTransitionChainV2>()
            .register_method::<GetTransitionChainProofV1ForV2>()
            .build()
    };

    let behaviour = Libp2pBehaviour {
        gossipsub,
        identify,
        kademlia,
        rpc,
        chain_id,
        port,
        ongoing: Default::default(),
        ongoing_incoming: Default::default(),
    };

    let swarm = libp2p::SwarmBuilder::with_existing_identity(identity_keys)
        .with_tokio()
        .with_other_transport(|key| {
            let noise_config = libp2p::noise::Config::new(key).unwrap();
            let mut yamux_config = libp2p::yamux::Config::default();

            yamux_config.set_protocol_name("/coda/yamux/1.0.0");

            let mut base_transport = libp2p::tcp::tokio::Transport::new(
                libp2p::tcp::Config::default()
                    .nodelay(true)
                    .port_reuse(port_reuse),
            );

            base_transport
                .listen_on(
                    libp2p::core::transport::ListenerId::next(),
                    libp2p::multiaddr::multiaddr!(Ip4([127, 0, 0, 1]), Tcp(port)),
                )
                .expect("listen");

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
