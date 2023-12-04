use std::time::Duration;

use mina_p2p_messages::v2::MinaLedgerSyncLedgerQueryStableV1;
use node::{
    event_source::Event,
    ledger::LedgerAddress,
    p2p::{
        channels::{
            rpc::{P2pRpcKind, P2pRpcRequest, RpcChannelMsg},
            ChannelMsg,
        },
        P2pChannelEvent, P2pEvent,
    },
    State,
};

use crate::{
    cluster::ClusterNodeId,
    node::RustNodeTestingConfig,
    scenario::{ListenerNode, ScenarioStep},
    scenarios::cluster_runner::ClusterRunner,
};

/// Set up single Rust node and sync up root snarked ledger.
///
/// 1. Node will connect to 2 peers (replayers).
/// 2. At some chosen height, node will receive all hashes before receiving the first one.
/// 3. At next height, node will receive all hashes before receiving the last hash on that height.
/// 4. At next height we will do same above 2 steps, except those first and last hash requests will timeout instead of being received at the end.
/// 5. Continue till we are done syncing root snarked ledger.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct SoloNodeSyncRootSnarkedLedger;

impl SoloNodeSyncRootSnarkedLedger {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        let node_id = runner.add_rust_node(RustNodeTestingConfig::berkeley_default());
        eprintln!("launch Openmina node with default configuration, id: {node_id}");

        const REPLAYER_1: &'static str =
            "/ip4/65.109.110.75/tcp/18302/p2p/12D3KooWD8jSyPFXNdAcMBHyHjRBcK1AW9t3xvnpfCFSRKMweVKi";
        const REPLAYER_2: &'static str =
            "/ip4/65.109.110.75/tcp/18303/p2p/12D3KooWBxbfeaxGHxdxP3U5jRKpNK5wQmbjKywGJEqTCNpVPxqk";

        // Initiate connection to 2 replayers.
        runner
            .exec_step(ScenarioStep::ConnectNodes {
                dialer: node_id,
                listener: ListenerNode::Custom(REPLAYER_1.parse().unwrap()),
            })
            .await
            .unwrap();
        eprintln!("node: {node_id} dialing to replayer: {REPLAYER_1}");
        runner
            .exec_step(ScenarioStep::ConnectNodes {
                dialer: node_id,
                listener: ListenerNode::Custom(REPLAYER_2.parse().unwrap()),
            })
            .await
            .unwrap();
        eprintln!("node: {node_id} dialing to replayer: {REPLAYER_2}");

        loop {
            if !runner
                .wait_for_pending_events_with_timeout(Duration::from_secs(10))
                .await
            {
                panic!("waiting for connection event timed out");
            }
            let (state, events) = runner.node_pending_events(node_id).unwrap();

            let connected_peer_count = state
                .p2p
                .ready_peers_iter()
                .filter(|(_, p)| p.channels.rpc.is_ready())
                .count();

            // Break loop if both replayers got connected and we started
            // sending ledger queries.
            if connected_peer_count >= 2 {
                let has_sent_ledger_query = state
                    .p2p
                    .ready_peers_iter()
                    .filter_map(|(_, p)| p.channels.rpc.pending_local_rpc_kind())
                    .any(|rpc_kind| matches!(rpc_kind, P2pRpcKind::LedgerQuery));

                if has_sent_ledger_query {
                    break;
                }
            }

            let events = events
                .filter_map(|(_, event)| {
                    // Don't dispatch ledger query responses yet. We want
                    // to later manually control their order.
                    Some(())
                        .filter(|_| self.event_ledger_query_addr(state, event).is_none())
                        .map(|_| event.to_string())
                })
                .collect::<Vec<_>>();

            for event in events {
                runner
                    .exec_step(ScenarioStep::Event { node_id, event })
                    .await
                    .unwrap();
            }
        }
        eprintln!("2 replayers are now connected");

        // Exec ledger query responses until we are deep enough for there
        // to be more than 1 hash in the same height.
        eprintln!("exec ledger query responses until we are deep enough for there to be more than 1 hash in the same height");
        loop {
            if !runner
                .wait_for_pending_events_with_timeout(Duration::from_secs(5))
                .await
            {
                panic!("waiting for events event timed out");
            }
            let (state, events) = runner.node_pending_events(node_id).unwrap();

            let snarked_state = state
                .transition_frontier
                .sync
                .ledger()
                .unwrap()
                .snarked()
                .unwrap();
            if snarked_state.fetch_pending().unwrap().len() >= 2 {
                break;
            }

            for event in events.map(|(_, e)| e.to_string()).collect::<Vec<_>>() {
                runner
                    .exec_step(ScenarioStep::Event { node_id, event })
                    .await
                    .unwrap();
            }
        }

        eprintln!("receive all hashes before first...");
        self.receive_all_hashes_before_first(&mut runner, node_id)
            .await;
        eprintln!("receive all hashes before last...");
        self.receive_all_hashes_before_last(&mut runner, node_id)
            .await;
        eprintln!("success");
    }

    async fn receive_all_hashes_before_first(
        self,
        runner: &mut ClusterRunner<'_>,
        node_id: ClusterNodeId,
    ) {
        self.receive_all_hashes_except_first(runner, node_id).await;
        let (_state, events) = runner.node_pending_events(node_id).unwrap();
        for event in events.map(|(_, e)| e.to_string()).collect::<Vec<_>>() {
            runner
                .exec_step(ScenarioStep::Event { node_id, event })
                .await
                .unwrap();
        }
    }

    async fn receive_all_hashes_except_first(
        self,
        runner: &mut ClusterRunner<'_>,
        node_id: ClusterNodeId,
    ) {
        loop {
            if !runner
                .wait_for_pending_events_with_timeout(Duration::from_secs(5))
                .await
            {
                panic!("waiting for events event timed out");
            }
            let (state, events) = runner.node_pending_events(node_id).unwrap();

            let snarked_state = state
                .transition_frontier
                .sync
                .ledger()
                .unwrap()
                .snarked()
                .unwrap();
            if snarked_state.fetch_pending().unwrap().len() == 1 {
                break;
            }

            let events = events.filter(|(_, e)| !self.is_event_first_ledger_query(state, e));

            for event in events.map(|(_, e)| e.to_string()).collect::<Vec<_>>() {
                runner
                    .exec_step(ScenarioStep::Event { node_id, event })
                    .await
                    .unwrap();
            }
        }

        runner
            .exec_step(ScenarioStep::CheckTimeouts { node_id })
            .await
            .unwrap();
    }

    async fn receive_all_hashes_before_last(
        self,
        runner: &mut ClusterRunner<'_>,
        node_id: ClusterNodeId,
    ) {
        self.receive_all_hashes_except_last(runner, node_id).await;
        let (_state, events) = runner.node_pending_events(node_id).unwrap();
        for event in events.map(|(_, e)| e.to_string()).collect::<Vec<_>>() {
            runner
                .exec_step(ScenarioStep::Event { node_id, event })
                .await
                .unwrap();
        }
    }

    async fn receive_all_hashes_except_last(
        self,
        runner: &mut ClusterRunner<'_>,
        node_id: ClusterNodeId,
    ) {
        loop {
            if !runner
                .wait_for_pending_events_with_timeout(Duration::from_secs(5))
                .await
            {
                panic!("waiting for events event timed out");
            }
            let (state, events) = runner.node_pending_events(node_id).unwrap();

            let snarked_state = state
                .transition_frontier
                .sync
                .ledger()
                .unwrap()
                .snarked()
                .unwrap();
            if snarked_state.fetch_pending().unwrap().len() == 1 {
                break;
            }

            let mut events = events
                .filter_map(|(_, e)| Some((e, self.event_ledger_query_addr(state, e)?)))
                .collect::<Vec<_>>();

            events.sort_by(|(_, addr1), (_, addr2)| addr1.cmp(addr2));

            let events = events.into_iter().rev().skip(1);

            for event in events.map(|(e, _)| e.to_string()).collect::<Vec<_>>() {
                runner
                    .exec_step(ScenarioStep::Event { node_id, event })
                    .await
                    .unwrap();
            }
        }

        runner
            .exec_step(ScenarioStep::CheckTimeouts { node_id })
            .await
            .unwrap();
    }

    fn event_ledger_query_addr(self, state: &State, event: &Event) -> Option<LedgerAddress> {
        let Event::P2p(P2pEvent::Channel(P2pChannelEvent::Received(
            peer_id,
            Ok(ChannelMsg::Rpc(RpcChannelMsg::Response(_, _))),
        ))) = event
        else {
            return None;
        };
        let rpc = state
            .p2p
            .get_ready_peer(peer_id)
            .unwrap()
            .channels
            .rpc
            .pending_local_rpc()
            .unwrap();
        let P2pRpcRequest::LedgerQuery(_, MinaLedgerSyncLedgerQueryStableV1::WhatChildHashes(addr)) =
            rpc
        else {
            return None;
        };
        Some(addr.into())
    }

    fn is_event_first_ledger_query(self, state: &State, event: &Event) -> bool {
        self.event_ledger_query_addr(state, event)
            .map_or(false, |addr| addr == LedgerAddress::first(addr.length()))
    }
}
