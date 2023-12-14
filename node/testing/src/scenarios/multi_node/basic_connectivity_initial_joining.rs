use std::{collections::HashMap, time::Duration};

use libp2p::Multiaddr;
use node::{
    event_source::Event,
    p2p::{
        connection::outgoing::{
            P2pConnectionOutgoingInitLibp2pOpts, P2pConnectionOutgoingInitOpts,
        },
        webrtc::SignalingMethod,
        P2pEvent,
    },
};

use crate::{
    node::RustNodeTestingConfig, scenario::ScenarioStep, scenarios::cluster_runner::ClusterRunner,
};

/// Global test that aims to be deterministic.
/// Launch `TOTAL_PEERS` number of nodes with `MAX_PEERS_PER_NODE` is est as the maximum number of peers.
/// Launch a seed node where `TOTAL_PEERS` is set as the maximum number of peers.
/// Run the simulation until the following condition is satisfied:
/// * Each node is connected to a number of peers determined by the `P2pState::min_peers` method.
/// Fail the test if any node exceeds the maximum number of connections.
/// Fail the test if the specified number of steps occur but the condition is not met.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct MultiNodeBasicConnectivityInitialJoining;

impl MultiNodeBasicConnectivityInitialJoining {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        const TOTAL_PEERS: usize = 20;
        const STEPS_PER_PEER: usize = 10;
        const EXTRA_STEPS: usize = 2000;
        const MAX_PEERS_PER_NODE: usize = 12;
        const STEP_DELAY: Duration = Duration::from_millis(200);

        let seed_node =
            runner.add_rust_node(RustNodeTestingConfig::berkeley_default().max_peers(TOTAL_PEERS));
        let full_config = &runner
            .node(seed_node)
            .expect("must exist")
            .state()
            .p2p
            .config;

        let peer_id = full_config.identity_pub_key.peer_id();
        let this = format!(
            "/ip4/127.0.0.1/tcp/{}/p2p/{}",
            full_config.libp2p_port.unwrap(),
            libp2p::PeerId::from(peer_id)
        );
        let this_maddr = this.parse::<Multiaddr>().unwrap();
        eprintln!("launch Openmina seed node, id: {seed_node}, addr: {this}");
        let init_opts = P2pConnectionOutgoingInitOpts::LibP2P(
            P2pConnectionOutgoingInitLibp2pOpts::try_from(&this_maddr).unwrap(),
        );
        let signaling = SignalingMethod::Http(([127, 0, 0, 1], full_config.listen_port).into());
        let init_opts_webrtc = P2pConnectionOutgoingInitOpts::WebRTC { peer_id, signaling };

        let mut nodes = vec![seed_node];

        for step in 0..(TOTAL_PEERS * STEPS_PER_PEER + EXTRA_STEPS) {
            tokio::time::sleep(STEP_DELAY).await;

            if step % STEPS_PER_PEER == 0 && nodes.len() < TOTAL_PEERS {
                let node = runner.add_rust_node(
                    RustNodeTestingConfig::berkeley_default()
                        .max_peers(MAX_PEERS_PER_NODE)
                        .initial_peers(vec![init_opts.clone(), init_opts_webrtc.clone()])
                        .ask_initial_peers_interval(Duration::from_secs(10)),
                );
                eprintln!("launch Openmina node, id: {node}, connects to {seed_node}");

                nodes.push(node);
            }

            let steps = runner
                .pending_events()
                .map(|(node_id, _, events)| {
                    events.map(move |(_, event)| {
                        match event {
                            Event::P2p(P2pEvent::Discovery(event)) => {
                                eprintln!("event: {event}");
                            }
                            _ => {}
                        }
                        ScenarioStep::Event {
                            node_id,
                            event: event.to_string(),
                        }
                    })
                })
                .flatten()
                .collect::<Vec<_>>();
            for step in steps {
                runner.exec_step(step).await.unwrap();
            }

            if nodes.len() < TOTAL_PEERS {
                continue;
            }

            let mut conditions_met = true;
            for &node_id in &nodes {
                runner
                    .exec_step(ScenarioStep::AdvanceNodeTime {
                        node_id,
                        by_nanos: STEP_DELAY.as_nanos() as _,
                    })
                    .await
                    .unwrap();

                runner
                    .exec_step(ScenarioStep::CheckTimeouts { node_id })
                    .await
                    .unwrap();

                let node = runner.node(node_id).expect("node must exist");
                let p2p = &node.state().p2p;
                let ready_peers = p2p.ready_peers_iter().count();

                // each node connected to some peers
                conditions_met &= ready_peers >= node.state().p2p.min_peers();

                // maximum is not exceeded
                let max_peers = if node_id == seed_node {
                    TOTAL_PEERS
                } else {
                    MAX_PEERS_PER_NODE
                };
                assert!(ready_peers <= max_peers);
            }

            if conditions_met {
                let mut total_connections_known = 0;
                let mut total_connections_ready = 0;
                for &node_id in &nodes {
                    let node = runner.node(node_id).expect("node must exist");
                    let p2p = &node.state().p2p;
                    let ready_peers = p2p.ready_peers_iter().count();
                    let known_peers = p2p.kademlia.known_peers.len();
                    let state_machine_peers = if cfg!(feature = "p2p-webrtc") {
                        ready_peers
                    } else {
                        ready_peers.max(known_peers)
                    };
                    total_connections_ready += ready_peers;
                    total_connections_known += state_machine_peers;
                    eprintln!(
                        "node {} has {ready_peers} peers",
                        p2p.config.identity_pub_key.peer_id(),
                    );
                }

                // TODO: calculate per peer
                if let Some(debugger) = runner.cluster().debugger() {
                    tokio::time::sleep(Duration::from_secs(10)).await;

                    let connections = debugger
                        .connections_raw(0)
                        .map(|(id, c)| (id, (c.info.addr, c.info.fd, c.info.pid, c.incoming)))
                        .collect::<HashMap<_, _>>();

                    // dbg
                    for (id, cn) in &connections {
                        eprintln!("{id}: {}", serde_json::to_string(cn).unwrap());
                    }
                    // dbg
                    for (id, msg) in debugger.messages(0, "stream_kind=/noise") {
                        eprintln!("{id}: {}", serde_json::to_string(&msg).unwrap());
                    }

                    // TODO: fix debugger returns timeout
                    let connections = debugger
                        .connections()
                        .filter_map(|id| Some((id, connections.get(&id)?.clone())))
                        .collect::<HashMap<_, _>>();
                    let incoming = connections.iter().filter(|(_, (_, _, _, i))| *i).count();
                    let outgoing = connections.len() - incoming;
                    eprintln!(
                        "debugger seen {incoming} incoming connections and {outgoing} outgoing connections",
                    );

                    for (id, (addr, fd, pid, incoming)) in connections {
                        eprintln!(
                            "id: {id}, pid: {pid}, fd: {fd}, addr: {addr}, incoming: {incoming}"
                        );
                    }

                    eprintln!("total_connections_known: {total_connections_known}");
                    assert_eq!(
                        incoming + outgoing,
                        total_connections_ready,
                        "debugger must see the same number of connections as the state machine"
                    );
                } else {
                    eprintln!("no debugger, run test with --use-debugger for additional check");
                }

                eprintln!("success");

                return;
            }
        }

        for node_id in &nodes {
            let node = runner.node(*node_id).expect("node must exist");
            let p2p: &node::p2p::P2pState = &node.state().p2p;
            let ready_peers = p2p.ready_peers_iter().count();
            // each node connected to some peers
            println!(
                "must hold {ready_peers} >= {}",
                node.state().p2p.min_peers()
            );
        }

        for node_id in nodes {
            let node = runner.node(node_id).expect("node must exist");
            println!("{node_id:?} - p2p state: {:#?}", &node.state().p2p);
        }

        assert!(false);
    }
}
