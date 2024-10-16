use std::{
    collections::{BTreeSet, HashSet},
    time::Duration,
};

use multiaddr::{multiaddr, Multiaddr};
use p2p::{
    identity::SecretKey,
    network::identify::{
        stream_effectful::P2pNetworkIdentifyStreamEffectfulAction, P2pNetworkIdentify,
        P2pNetworkIdentifyEffectfulAction, P2pNetworkIdentifyStreamAction,
    },
    token::{self, DiscoveryAlgorithm},
    Data, P2pEffectfulAction, P2pNetworkEffectfulAction, P2pNetworkYamuxAction, PeerId,
};
use p2p_testing::{
    cluster::{Cluster, ClusterBuilder, ClusterEvent, Listener},
    event::{allow_disconnections, event_mapper_effect, RustNodeEvent},
    futures::TryStreamExt,
    predicates::{async_fn, listener_is_ready, peer_is_connected},
    redux::{Action, State},
    rust_node::{RustNodeConfig, RustNodeId},
    service::ClusterService,
    stream::ClusterStreamExt,
    test_node::TestNode,
};
use redux::{ActionWithMeta, Store};

#[tokio::test]
async fn rust_node_to_rust_node() -> anyhow::Result<()> {
    let mut cluster = ClusterBuilder::new()
        .ports_with_len(10)
        .idle_duration(Duration::from_millis(100))
        .start()
        .await?;

    let node1 = cluster.add_rust_node(RustNodeConfig::default())?;

    let node2 = cluster.add_rust_node(RustNodeConfig::default())?;

    let peer_id1 = cluster.rust_node(node1).state().my_id();
    let peer_id2 = cluster.rust_node(node2).state().my_id();

    // wait for node1 to be ready to accept incoming conections
    let listener_is_ready = cluster
        .try_stream()
        .take_during(Duration::from_secs(2))
        .try_any(listener_is_ready(node1))
        .await?;
    assert!(listener_is_ready, "node1 should be ready");

    // connect node2 to node1
    cluster.connect(node2, node1)?;

    // wait for node2 to have peer_id1 (node1) as its peer in `ready` state`
    let connected = cluster
        .try_stream()
        .take_during(Duration::from_secs(2))
        .try_any(peer_is_connected(node2, peer_id1))
        .await?;
    assert!(
        connected,
        "node should be able to connect to {peer_id1}: {connected:?}\nnode state: {:#?}",
        cluster.rust_node(node2).state().peers.get(&peer_id1)
    );

    {
        let mut not_identified = BTreeSet::from_iter([(node1, peer_id2), (node2, peer_id1)]);

        // run the cluster until both nodes have identify data about each other
        let addrs =
            wait_for_identify(&mut cluster, &mut not_identified, Duration::from_secs(10)).await?;

        // for each address provided by identify, create a node and ensure it
        // can connect to that address
        for (peer_id, addr) in addrs {
            let node = cluster
                .add_rust_node(RustNodeConfig::default())
                .expect("no error");
            cluster
                .connect(
                    node,
                    addr.clone()
                        .with_p2p(peer_id.try_into().expect("Error converting PeerId"))
                        .expect("no error"),
                )
                .expect("no error");
            let connected = cluster
                .try_stream()
                .take_during(Duration::from_secs(5))
                .try_any(peer_is_connected(node, peer_id))
                .await
                .expect("unexpected error");
            assert!(
                connected,
                "node {} should be able to connect to {peer_id} via {addr}: {connected:?}\nnode state: {:#?}", cluster.peer_id(node),
                cluster.rust_node(node).state().peers.get(&peer_id)
            );
        }
    }

    Ok(())
}

#[tokio::test]
/// Test that even if bad node spams many different listen_addrs we don't end up with duplicates
async fn test_bad_node() -> anyhow::Result<()> {
    let mut cluster = ClusterBuilder::new()
        .ports_with_len(100)
        .idle_duration(Duration::from_millis(100))
        .is_error(allow_disconnections)
        .start()
        .await?;

    let bad_node = cluster.add_rust_node(
        RustNodeConfig::default()
            .with_discovery(true)
            .with_override(bad_node_effects),
    )?;
    let bad_node_peer_id = cluster.rust_node(bad_node).peer_id();
    let bad_node_port = cluster.rust_node(bad_node).libp2p_port();

    let node = cluster.add_rust_node(
        RustNodeConfig::default()
            .with_discovery(true)
            .with_initial_peers([Listener::Rust(bad_node)]),
    )?;

    let mut not_identified = BTreeSet::from_iter([(node, bad_node_peer_id)]);
    wait_for_identify(&mut cluster, &mut not_identified, Duration::from_secs(10)).await?;

    let routing_table = &cluster
        .rust_node(node)
        .state()
        .network
        .scheduler
        .discovery_state()
        .expect("State must be initialized")
        .routing_table;

    let bad_peer_entry = routing_table
        .look_up(
            &bad_node_peer_id
                .try_into()
                .expect("PeerId conversion failed"),
        )
        .expect("Node not found");

    let bad_peer_addresses = bad_peer_entry
        .addresses()
        .iter()
        .map(Clone::clone)
        .collect::<HashSet<_>>();

    let expected_addrs = [
        multiaddr!(Ip4([127, 0, 0, 1]), Tcp(bad_node_port)),
        multiaddr!(Unix("domain.com")),
        multiaddr!(Dns("domain.com"), Tcp(10530u16)),
        multiaddr!(Https),
        multiaddr!(Ip4([127, 0, 0, 1]), Tcp(10500u16)),
        multiaddr!(Ip6([0; 16]), Tcp(10500u16)),
        multiaddr!(Ip6([1; 16]), Tcp(10500u16)),
    ]
    .into_iter()
    .collect::<HashSet<_>>();

    assert_eq!(bad_peer_addresses, expected_addrs);

    Ok(())
}

fn bad_node_effects(
    store: &mut Store<State, ClusterService, Action>,
    action: ActionWithMeta<Action>,
) {
    {
        let (action, meta) = action.split();
        match action {
            Action::P2p(a) => {
                event_mapper_effect(store, a);
            }
            Action::P2pEffectful(P2pEffectfulAction::Network(
                P2pNetworkEffectfulAction::Identify(P2pNetworkIdentifyEffectfulAction::Stream(
                    P2pNetworkIdentifyStreamEffectfulAction::SendIdentify {
                        addr,
                        peer_id,
                        stream_id,
                    },
                )),
            )) => {
                let listen_addrs = vec![
                    multiaddr!(Ip4([127, 0, 0, 1]), Tcp(10500u16)),
                    multiaddr!(Ip4([127, 0, 0, 1]), Tcp(10500u16)),
                    multiaddr!(Ip6([0; 16]), Tcp(10500u16)),
                    multiaddr!(Ip6([0; 16]), Tcp(10500u16)),
                    multiaddr!(Ip6([1; 16]), Tcp(10500u16)),
                    multiaddr!(Ip6([1; 16]), Tcp(10500u16)),
                    multiaddr!(Dns("domain.com"), Tcp(10530u16)),
                    multiaddr!(Dns("domain.com"), Tcp(10530u16)),
                    multiaddr!(Dns("domain.com"), Tcp(10530u16)),
                    multiaddr!(Dns("domain.com"), Tcp(10530u16)),
                    multiaddr!(Unix("domain.com")),
                    multiaddr!(Https),
                ];

                let public_key = Some(SecretKey::rand().public_key());

                let protocols = vec![
                    token::StreamKind::Identify(token::IdentifyAlgorithm::Identify1_0_0),
                    token::StreamKind::Broadcast(p2p::token::BroadcastAlgorithm::Meshsub1_1_0),
                    p2p::token::StreamKind::Rpc(token::RpcAlgorithm::Rpc0_0_1),
                    p2p::token::StreamKind::Discovery(DiscoveryAlgorithm::Kademlia1_0_0),
                ];

                let identify_msg = P2pNetworkIdentify {
                    protocol_version: Some("ipfs/0.1.0".to_string()),
                    agent_version: Some("openmina".to_owned()),
                    public_key,
                    listen_addrs,
                    observed_addr: None,
                    protocols,
                };

                let mut out = Vec::new();
                let identify_msg_proto =
                    identify_msg.to_proto_message().expect("serialized message");

                prost::Message::encode_length_delimited(&identify_msg_proto, &mut out)
                    .expect("Error converting message");

                store.dispatch(Action::P2p(
                    P2pNetworkYamuxAction::OutgoingData {
                        addr,
                        stream_id,
                        data: Data(out.into_boxed_slice()),
                        flags: Default::default(),
                    }
                    .into(),
                ));

                store.dispatch(Action::P2p(
                    P2pNetworkIdentifyStreamAction::Close {
                        addr,
                        peer_id,
                        stream_id,
                    }
                    .into(),
                ));
            }
            Action::P2pEffectful(action) => action.effects(meta, store),
            _ => {}
        };
    }
}

async fn wait_for_identify(
    cluster: &mut Cluster,
    nodes_peers: &mut BTreeSet<(RustNodeId, PeerId)>,
    time: Duration,
) -> anyhow::Result<Vec<(PeerId, Multiaddr)>> {
    let mut addrs = Vec::new();
    let pred = |event: ClusterEvent| {
        if let ClusterEvent::Rust {
            id,
            event: RustNodeEvent::Identify { peer_id, info },
        } = event
        {
            if nodes_peers.remove(&(id, peer_id)) {
                addrs.extend(info.listen_addrs.iter().map(|addr| (peer_id, addr.clone())));
                return nodes_peers.is_empty();
            }
        }
        false
    };
    let identified = cluster
        .try_stream()
        .take_during(time)
        .try_any(async_fn(pred))
        .await?;
    assert!(identified, "all peers should be identified");
    Ok(addrs)
}
