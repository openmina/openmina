use std::{cmp::Ordering, time::Duration};

use mina_p2p_messages::v2::MinaLedgerSyncLedgerQueryStableV1;
use node::{
    event_source::Event,
    ledger::LedgerAddress,
    p2p::{
        channels::{
            rpc::{P2pRpcRequest, RpcChannelMsg},
            ChannelMsg,
        },
        P2pChannelEvent, P2pEvent,
    },
    ActionKind, State,
};

use crate::{
    cluster::ClusterNodeId,
    node::RustNodeTestingConfig,
    scenario::{ListenerNode, ScenarioStep},
    scenarios::{ClusterRunner, RunCfg, RunDecision},
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
        let node_id = runner.add_rust_node(RustNodeTestingConfig::devnet_default());
        eprintln!("launch Openmina node with default configuration, id: {node_id}");

        const REPLAYER_1: &str =
            "/dns4/web-node-1/tcp/18302/p2p/12D3KooWD8jSyPFXNdAcMBHyHjRBcK1AW9t3xvnpfCFSRKMweVKi";
        const REPLAYER_2: &str =
            "/dns4/web-node-1/tcp/18303/p2p/12D3KooWBxbfeaxGHxdxP3U5jRKpNK5wQmbjKywGJEqTCNpVPxqk";

        // Initiate connection to 2 replayers.
        runner
            .exec_step(ScenarioStep::ConnectNodes {
                dialer: node_id,
                listener: ListenerNode::Custom(REPLAYER_1.parse().unwrap()),
            })
            .await
            .unwrap();
        eprintln!("node({node_id}) dialing to replayer: {REPLAYER_1}");
        runner
            .exec_step(ScenarioStep::ConnectNodes {
                dialer: node_id,
                listener: ListenerNode::Custom(REPLAYER_2.parse().unwrap()),
            })
            .await
            .unwrap();
        eprintln!("node({node_id}) dialing to replayer: {REPLAYER_2}");

        // Wait for both peers to be connected, hiding p2p ledger query
        // responses for now, as we want to control their order.
        runner
            .run(
                RunCfg::default()
                    .timeout(Duration::from_secs(10))
                    .event_handler(|_, state, event| {
                        if self.event_ledger_query_addr(state, event).is_some() {
                            // skip/hide ledger query events.
                            return RunDecision::Skip;
                        }
                        RunDecision::ContinueExec
                    })
                    .action_handler(|_, state, _, _| {
                        let connected_peer_count = state
                            .p2p
                            .ready_peers_iter()
                            .filter(|(_, p)| p.channels.rpc.is_ready())
                            .count();

                        // exit if both peers ready.
                        connected_peer_count >= 2
                    }),
            )
            .await
            .expect("waiting for 2 replayer peers to be connected timed out");

        eprintln!("2 replayers are now connected");

        // Exec ledger query responses until we are deep enough for there
        // to be more than 1 hash in the same height.
        eprintln!("exec ledger query responses until we are deep enough for there to be more than 1 hash in the same height");
        runner
            .run(
                RunCfg::default()
                    .timeout(Duration::from_secs(10))
                    .action_handler(move |_, state, _, action| {
                        matches!(action.action().kind(), ActionKind::CheckTimeouts)
                            && self.fetch_pending_count(state) >= 2
                    }),
            )
            .await
            .expect("time out");

        eprintln!("receive all hashes before first...");
        self.receive_all_hashes_before_first(&mut runner, node_id)
            .await;
        eprintln!("receive all hashes before last...");
        self.receive_all_hashes_before_last(&mut runner, node_id)
            .await;
        eprintln!("success");
    }

    fn fetch_pending_count(self, state: &State) -> usize {
        None.or_else(|| {
            let snarked_state = state.transition_frontier.sync.ledger()?.snarked()?;
            Some(snarked_state.fetch_pending().unwrap().len())
        })
        .unwrap_or(0)
    }

    async fn receive_single_hash(self, runner: &mut ClusterRunner<'_>, node_id: ClusterNodeId) {
        runner
            .run(
                RunCfg::default()
                    .timeout(Duration::from_secs(5))
                    .event_handler(|cur_node_id, state, event| {
                        if cur_node_id == node_id
                            && self.event_ledger_query_addr(state, event).is_some()
                        {
                            return RunDecision::StopExec;
                        }
                        RunDecision::Skip
                    }),
            )
            .await
            .expect("timeout");
    }

    async fn receive_all_hashes_before_first(
        self,
        runner: &mut ClusterRunner<'_>,
        node_id: ClusterNodeId,
    ) {
        self.receive_all_hashes_except_first(runner, node_id).await;
        self.receive_single_hash(runner, node_id).await;
    }

    async fn receive_all_hashes_except_first(
        self,
        runner: &mut ClusterRunner<'_>,
        _node_id: ClusterNodeId,
    ) {
        runner
            .run(
                RunCfg::default()
                    .timeout(Duration::from_secs(10))
                    .event_handler(|_, state, event| {
                        if self.is_event_first_ledger_query(state, event) {
                            return RunDecision::Skip;
                        }
                        RunDecision::ContinueExec
                    })
                    .action_handler(move |_, state, _, action| {
                        matches!(action.action().kind(), ActionKind::CheckTimeouts)
                            && self.fetch_pending_count(state) == 1
                    }),
            )
            .await
            .expect("timeout");
    }

    async fn receive_all_hashes_before_last(
        self,
        runner: &mut ClusterRunner<'_>,
        node_id: ClusterNodeId,
    ) {
        self.receive_all_hashes_except_last(runner, node_id).await;
        self.receive_single_hash(runner, node_id).await;
    }

    async fn receive_all_hashes_except_last(
        self,
        runner: &mut ClusterRunner<'_>,
        node_id: ClusterNodeId,
    ) {
        let mut biggest_addr = None;
        while self.fetch_pending_count(runner.node(node_id).unwrap().state()) > 1 {
            runner
                .run(
                    RunCfg::default()
                        .timeout(Duration::from_secs(10))
                        .event_handler(|_, state, event| {
                            let Some(addr) = self.event_ledger_query_addr(state, event) else {
                                return RunDecision::Skip;
                            };
                            match biggest_addr.as_mut() {
                                None => {
                                    biggest_addr = Some(addr);
                                    RunDecision::Skip
                                }
                                Some(biggest_addr) => match addr.cmp(biggest_addr) {
                                    Ordering::Less => RunDecision::ContinueExec,
                                    Ordering::Equal => RunDecision::Skip,
                                    Ordering::Greater => {
                                        *biggest_addr = addr;
                                        RunDecision::Stop
                                    }
                                },
                            }
                        })
                        .action_handler(move |_, state, _, action| {
                            matches!(action.action().kind(), ActionKind::CheckTimeouts)
                                && self.fetch_pending_count(state) == 1
                        }),
                )
                .await
                .expect("timeout");
        }
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
