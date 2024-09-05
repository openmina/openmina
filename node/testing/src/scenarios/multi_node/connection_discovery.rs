use std::time::Duration;

use crate::{
    cluster::{ClusterNodeId, ClusterOcamlNodeId},
    node::{OcamlNodeTestingConfig, RustNodeTestingConfig},
    scenario::{ListenerNode, ScenarioStep},
    scenarios::{get_peers_iter, ClusterRunner, RunCfg, PEERS_QUERY},
};
use anyhow::Context;
use node::{
    p2p::{identify::P2pIdentifyAction, peer::P2pPeerAction, PeerId},
    ActionKind, P2pAction,
};
use tokio::time::sleep;

/// Ensure that Rust node can pass information about peers when used as a seed node.
/// 1. Create rust seed node and wait for it to be ready
/// 2. Create 2 Ocaml nodes with rust seed node as initial peer
/// 3. Check that Ocaml nodes know each other via rust seed node
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct RustNodeAsSeed;

impl RustNodeAsSeed {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        std::env::set_var("OPENMINA_DISCOVERY_FILTER_ADDR", "false");
        let rust_node_id = runner.add_rust_node(RustNodeTestingConfig::devnet_default());
        let rust_node_dial_addr = runner.node(rust_node_id).unwrap().dial_addr();
        let rust_peer_id = *rust_node_dial_addr.peer_id();
        wait_for_node_ready(&mut runner, rust_node_id).await;

        let ocaml_node_config = OcamlNodeTestingConfig {
            initial_peers: vec![rust_node_dial_addr],
            ..Default::default()
        };

        let ocaml_node0 = runner.add_ocaml_node(ocaml_node_config.clone());
        let ocaml_peer_id0 = runner.ocaml_node(ocaml_node0).unwrap().peer_id();

        // this is needed, to make sure that `ocaml_node0` connects before `ocaml_node1`
        sleep(Duration::from_secs(30)).await;

        let ocaml_node1 = runner.add_ocaml_node(ocaml_node_config.clone());
        let ocaml_peer_id1 = runner.ocaml_node(ocaml_node1).unwrap().peer_id();

        wait_for_ready_connection(
            &mut runner,
            rust_node_id,
            ocaml_peer_id0,
            true,
            Some(Duration::from_secs(500)),
        )
        .await;
        wait_for_ready_connection(
            &mut runner,
            rust_node_id,
            ocaml_peer_id1,
            true,
            Some(Duration::from_secs(300)),
        )
        .await;

        let _ = runner
            .run(RunCfg::default().timeout(Duration::from_secs(120)))
            .await;

        let ocaml_node0_check =
            check_ocaml_peers(&mut runner, ocaml_node0, [rust_peer_id, ocaml_peer_id1])
                .await
                .unwrap_or_default();

        let ocaml_node1_check =
            check_ocaml_peers(&mut runner, ocaml_node1, [rust_peer_id, ocaml_peer_id0])
                .await
                .unwrap_or_default();

        assert!(ocaml_node0_check, "OCaml node 0 doesn't have valid peers");
        assert!(ocaml_node1_check, "OCaml node 1 doesn't have valid peers");

        let has_peer_in_routing_table =
            check_kademlia_entries(&mut runner, rust_node_id, [ocaml_peer_id0, ocaml_peer_id1])
                .unwrap_or_default();

        assert!(
            has_peer_in_routing_table,
            "Peers not found in rust node's routing table"
        );
    }
}

/// Test Rust node peer discovery when OCaml node connects to it
/// 1. Create rust node and wait for it to be ready
/// 2. Create OCaml node with rust node as initial peer
/// 3. Check that OCaml node connects to rust node
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct OCamlToRust;

impl OCamlToRust {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        std::env::set_var("OPENMINA_DISCOVERY_FILTER_ADDR", "false");
        let rust_node_id = runner.add_rust_node(RustNodeTestingConfig::devnet_default());
        let rust_node_dial_addr = runner.node(rust_node_id).unwrap().dial_addr();
        let rust_peer_id = *rust_node_dial_addr.peer_id();
        wait_for_node_ready(&mut runner, rust_node_id).await;

        let ocaml_node_config = OcamlNodeTestingConfig {
            initial_peers: vec![rust_node_dial_addr],
            ..Default::default()
        };

        let ocaml_node = runner.add_ocaml_node(ocaml_node_config.clone());
        let ocaml_peer_id = runner.ocaml_node(ocaml_node).unwrap().peer_id();

        wait_for_ready_connection(
            &mut runner,
            rust_node_id,
            ocaml_peer_id,
            true,
            Some(Duration::from_secs(300)),
        )
        .await;

        wait_for_identify(
            &mut runner,
            rust_node_id,
            ocaml_peer_id,
            "github.com/codaprotocol/coda/tree/master/src/app/libp2p_helper",
        )
        .await;

        let ocaml_check = check_ocaml_peers(&mut runner, ocaml_node, [rust_peer_id])
            .await
            .expect("Error querying graphql");

        assert!(ocaml_check, "OCaml node doesn't have rust as peer");
    }
}

/// Tests Rust node peer discovery when it connects to OCaml node
/// 1. Create Rust node and wait for it to be ready
/// 2. Create OCaml node and wait for it to be ready
/// 3. Connect rust node to Ocaml node
/// 4. Check that it is connected
/// 5. Check for kademlia and identify
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct RustToOCaml;

impl RustToOCaml {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        std::env::set_var("OPENMINA_DISCOVERY_FILTER_ADDR", "false");
        let rust_node_id = runner.add_rust_node(RustNodeTestingConfig::devnet_default());
        let rust_peer_id = runner.node(rust_node_id).expect("Node not found").peer_id();
        wait_for_node_ready(&mut runner, rust_node_id).await;

        let ocaml_seed_config = OcamlNodeTestingConfig::default();

        let seed_node = runner.add_ocaml_node(ocaml_seed_config);
        let seed_peer_id = runner.ocaml_node(seed_node).unwrap().peer_id();

        runner.wait_for_ocaml(seed_node).await;

        runner
            .exec_step(ScenarioStep::ConnectNodes {
                dialer: rust_node_id,
                listener: ListenerNode::Ocaml(seed_node),
            })
            .await
            .expect("Error connecting nodes");

        wait_for_ready_connection(&mut runner, rust_node_id, seed_peer_id, false, None).await;
        wait_for_identify(
            &mut runner,
            rust_node_id,
            seed_peer_id,
            "github.com/codaprotocol/coda/tree/master/src/app/libp2p_helper",
        )
        .await;

        let ocaml_has_rust_peer = check_ocaml_peers(&mut runner, seed_node, [rust_peer_id])
            .await
            .unwrap_or_default();
        assert!(ocaml_has_rust_peer, "Ocaml doesn't have rust node");
    }
}

/// Tests Rust node peer discovery when OCaml node is connected to it via an OCaml seed node.
/// 1. Create Rust node and wait for it to be ready
/// 2. Create OCaml seed node and wait for it to be ready
/// 3. Connect rust node to OCaml seed node
/// 4. Create OCaml node and connect to OCaml seed node
/// 5. Check that OCaml node connects to rust node via seed
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct OCamlToRustViaSeed;

impl OCamlToRustViaSeed {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        std::env::set_var("OPENMINA_DISCOVERY_FILTER_ADDR", "false");
        let rust_node_id = runner.add_rust_node(RustNodeTestingConfig::devnet_default());
        wait_for_node_ready(&mut runner, rust_node_id).await;

        let ocaml_seed_config = OcamlNodeTestingConfig::default();
        let seed_node = runner.add_ocaml_node(ocaml_seed_config.clone());
        let (seed_peer_id, seed_addr) = runner
            .ocaml_node(seed_node)
            .map(|node| (node.peer_id(), node.dial_addr()))
            .unwrap();

        runner.wait_for_ocaml(seed_node).await;

        runner
            .exec_step(ScenarioStep::ConnectNodes {
                dialer: rust_node_id,
                listener: ListenerNode::Ocaml(seed_node),
            })
            .await
            .unwrap();

        wait_for_ready_connection(&mut runner, rust_node_id, seed_peer_id, false, None).await;

        let ocaml_node = runner.add_ocaml_node(OcamlNodeTestingConfig {
            initial_peers: vec![seed_addr],
            ..ocaml_seed_config
        });
        let ocaml_peer_id = runner.ocaml_node(ocaml_node).unwrap().peer_id();

        runner.wait_for_ocaml(ocaml_node).await;
        wait_for_ready_connection(&mut runner, rust_node_id, ocaml_peer_id, true, None).await;
    }
}

/// Tests Rust node peer discovery when it connects to OCaml node via an OCaml seed node.
/// 1. Create rust node and wait for it to be ready
/// 2. Create OCaml seed node and wait for it to be ready
/// 3. Create OCaml node with OCaml seed as initial peer
/// 4. Wait for OCaml node to be ready
/// 5. Connect rust node to OCaml seed node
/// 6. Check that rust node connects to OCaml node via OCaml seed node
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct RustToOCamlViaSeed;

impl RustToOCamlViaSeed {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        std::env::set_var("OPENMINA_DISCOVERY_FILTER_ADDR", "false");
        let rust_node_id = runner.add_rust_node(RustNodeTestingConfig::devnet_default());
        wait_for_node_ready(&mut runner, rust_node_id).await;

        let ocaml_seed_config = OcamlNodeTestingConfig::default();

        let seed_node = runner.add_ocaml_node(ocaml_seed_config.clone());
        runner.wait_for_ocaml(seed_node).await;

        let (seed_peer_id, seed_addr) = runner
            .ocaml_node(seed_node)
            .map(|node| (node.peer_id(), node.dial_addr()))
            .unwrap();

        let ocaml_node = runner.add_ocaml_node(OcamlNodeTestingConfig {
            initial_peers: vec![seed_addr],
            ..ocaml_seed_config
        });

        let ocaml_peer_id = runner.ocaml_node(ocaml_node).unwrap().peer_id();
        runner.wait_for_ocaml(ocaml_node).await;

        runner
            .exec_step(ScenarioStep::ConnectNodes {
                dialer: rust_node_id,
                listener: ListenerNode::Ocaml(seed_node),
            })
            .await
            .unwrap();

        wait_for_ready_connection(&mut runner, rust_node_id, seed_peer_id, false, None).await;
        wait_for_ready_connection(&mut runner, rust_node_id, ocaml_peer_id, false, None).await;
    }
}

pub async fn wait_for_node_ready(runner: &mut ClusterRunner<'_>, node_id: ClusterNodeId) {
    runner
        .run(RunCfg::default().action_handler(move |id, _, _, action| {
            node_id == id && matches!(action.action().kind(), ActionKind::P2pInitializeInitialize)
        }))
        .await
        .expect("Node not ready")
}

pub async fn wait_for_identify(
    runner: &mut ClusterRunner<'_>,
    node_id: ClusterNodeId,
    connecting_peer_id: PeerId,
    agent_version: &str,
) {
    let agent_version = agent_version.to_owned();
    runner
        .run(
            RunCfg::default()
                .action_handler(move |id, _, _, action| {
                    id == node_id
                        && matches!(
                            action.action(),
                            node::Action::P2p(P2pAction::Identify(P2pIdentifyAction::UpdatePeerInformation {
                                peer_id,
                                info
                            })) if peer_id == &connecting_peer_id && info.agent_version == Some(agent_version.to_string())
                        )
                }),
        )
        .await
        .expect("Identify not exchanged");
}

async fn wait_for_ready_connection(
    runner: &mut ClusterRunner<'_>,
    node_id: ClusterNodeId,
    connecting_peer_id: PeerId,
    incoming_: bool,
    duration: Option<Duration>,
) {
    runner
        .run(
            RunCfg::default()
                .timeout(duration.unwrap_or(Duration::from_secs(60)))
                .action_handler(move |id, _, _, action| {
                    id == node_id
                        && matches!(
                            action.action(),
                            &node::Action::P2p(P2pAction::Peer(P2pPeerAction::Ready {
                                peer_id,
                                incoming
                            })) if peer_id == connecting_peer_id && incoming == incoming_
                        )
                }),
        )
        .await
        .expect("Nodes not connected");
}

async fn check_ocaml_peers<A>(
    runner: &mut ClusterRunner<'_>,
    node_id: ClusterOcamlNodeId,
    peer_ids: A,
) -> anyhow::Result<bool>
where
    A: IntoIterator<Item = PeerId>,
{
    let data = runner
        .ocaml_node(node_id)
        .expect("OCaml node not found")
        .grapql_query(PEERS_QUERY)
        .await?;

    let peers = get_peers_iter(&data)
        .with_context(|| "Failed to convert graphql response")?
        .flatten()
        .map(|peer| peer.2.to_owned())
        .collect::<Vec<_>>();

    Ok(peer_ids
        .into_iter()
        .all(|peer_id| peers.contains(&peer_id.to_libp2p_string())))
}

pub fn check_kademlia_entries<A>(
    runner: &mut ClusterRunner<'_>,
    node_id: ClusterNodeId,
    peer_ids: A,
) -> anyhow::Result<bool>
where
    A: IntoIterator<Item = PeerId>,
{
    let table = &runner
        .node(node_id)
        .with_context(|| "Node not found")?
        .state()
        .p2p
        .ready()
        .with_context(|| "P2p state not ready")?
        .network
        .scheduler
        .discovery_state()
        .with_context(|| "Discovery state not ready")?
        .routing_table;

    Ok(peer_ids.into_iter().all(|peer_id| {
        table
            .look_up(&peer_id.try_into().unwrap())
            .map(|entry| entry.peer_id == peer_id)
            .unwrap_or_default()
    }))
}
