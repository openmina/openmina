use std::time::Duration;

use node::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;

use crate::{cluster::Cluster, node::RustNodeTestingConfig, scenario::ScenarioStep};

use super::{Node, NodeKey};

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    const STEPS: usize = 4_000;
    const STEP_DELAY: Duration = Duration::from_millis(200);

    let mut cluster = Cluster::new(Default::default());

    // run only ocaml nodes for now
    let _ = &mut cluster;

    let seed_a_key = NodeKey::generate();
    let mut seed_a = Node::spawn_with_key(seed_a_key, 8302, 3085, 8301, true, Some(&[]));

    tokio::time::sleep(Duration::from_secs(60)).await;
    let nodes = (1..4)
        .map(|i| {
            Node::spawn(
                8302 + i * 10,
                3085 + i * 10,
                8301 + i * 10,
                Some(&[&seed_a.local_addr()]),
            )
        })
        .collect::<Vec<_>>();

    tokio::time::sleep(Duration::from_secs(180)).await;

    let config = RustNodeTestingConfig::berkeley_default()
        .ask_initial_peers_interval(Duration::from_secs(3600))
        .max_peers(100)
        .libp2p_port(10000)
        .initial_peers(
            nodes
                .iter()
                .chain(std::iter::once(&seed_a))
                .map(|n| P2pConnectionOutgoingInitOpts::try_from(&n.local_addr()).unwrap())
                .collect(),
        );
    let node_id = cluster.add_rust_node(config);

    let mut additional_ocaml_node = None::<Node>;

    let mut timeout = STEPS;
    loop {
        if timeout == 0 {
            break;
        }
        timeout -= 1;

        tokio::time::sleep(STEP_DELAY).await;

        let steps = cluster
            .pending_events()
            .map(|(node_id, _, events)| {
                events.map(move |(_, event)| ScenarioStep::Event {
                    node_id,
                    event: event.to_string(),
                })
            })
            .flatten()
            .collect::<Vec<_>>();

        for step in steps {
            cluster.exec_step(step).await.unwrap();
        }

        cluster
            .exec_step(ScenarioStep::AdvanceNodeTime {
                node_id,
                by_nanos: STEP_DELAY.as_nanos() as _,
            })
            .await
            .unwrap();

        cluster
            .exec_step(ScenarioStep::CheckTimeouts { node_id })
            .await
            .unwrap();

        // finish discovering
        if cluster
            .node(node_id)
            .unwrap()
            .state()
            .p2p
            .kademlia
            .saturated
            .is_some()
        {
            additional_ocaml_node.get_or_insert_with(|| {
                Node::spawn(9000, 4000, 9001, Some(&[&seed_a.local_addr()]))
            });
        }

        if let Some(additional_ocaml_node) = &additional_ocaml_node {
            let peer_id = additional_ocaml_node.peer_id;

            if cluster
                .node(node_id)
                .unwrap()
                .state()
                .p2p
                .peers
                .iter()
                .filter(|(id, n)| n.is_libp2p && **id == peer_id.into())
                .filter_map(|(_, n)| n.status.as_ready())
                .find(|n| n.is_incoming)
                .is_some()
            {
                break;
            }
        }
    }

    for mut node in nodes {
        node.kill();
    }
    seed_a.kill();
    additional_ocaml_node.as_mut().map(Node::kill);

    if timeout == 0 {
        panic!("timeout");
    }

    Ok(())
}
