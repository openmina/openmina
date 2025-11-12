use crate::{
    hosts,
    node::RustNodeTestingConfig,
    scenario::{ListenerNode, ScenarioStep},
    scenarios::ClusterRunner,
};
use mina_core::constants::constraint_constants;
use node::transition_frontier::sync::TransitionFrontierSyncState;
use redux::Instant;
use std::time::Duration;

/// Set up single Rust node and bootstrap snarked ledger, bootstrap ledger and blocks.
///
/// 1. Node will connect to replayer.
/// 2. Observe that stacking ledger is synchronized.
/// 3. Observe that next epoch ledger is synchronized.
/// 4. Continue until transition frontier is synchronized.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct SoloNodeBootstrap;

// TODO(tizoc): this is ugly, do a cleaner conversion or figure out a better way.
// This test will fail if we don't start with this as the initial time because
// the time validation for the first block will reject it.
fn first_block_slot_timestamp_nanos(config: &RustNodeTestingConfig) -> u64 {
    let first_block_global_slot = 279218; // Update if replay changes
    let protocol_constants = config.genesis.protocol_constants().unwrap();
    let genesis_timestamp_ms = protocol_constants.genesis_state_timestamp.0.as_u64();
    let milliseconds_per_slot = constraint_constants().block_window_duration_ms;
    let first_block_global_slot_delta_ms = first_block_global_slot * milliseconds_per_slot;

    // Convert to nanos
    genesis_timestamp_ms
        .checked_add(first_block_global_slot_delta_ms)
        .unwrap()
        .checked_mul(1_000_000)
        .unwrap()
}

impl SoloNodeBootstrap {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        use self::TransitionFrontierSyncState::*;

        const TIMEOUT: Duration = Duration::from_secs(60 * 40);

        let replayer = hosts::replayer();

        let mut config = RustNodeTestingConfig::devnet_default();

        config.initial_time = redux::Timestamp::new(first_block_slot_timestamp_nanos(&config));

        let node_id =
            runner.add_rust_node(config.initial_peers(vec![ListenerNode::Custom(replayer)]));
        eprintln!("launch Mina Rust node with default configuration, id: {node_id}");

        let mut timeout = TIMEOUT;
        let mut last_time = Instant::now();
        loop {
            if runner
                .wait_for_pending_events_with_timeout(Duration::from_secs(1))
                .await
            {
                let steps = runner
                    .pending_events(true)
                    .flat_map(|(node_id, _, events)| {
                        events.map(move |(_, event)| ScenarioStep::Event {
                            node_id,
                            event: event.to_string(),
                        })
                    })
                    .collect::<Vec<_>>();

                for step in steps {
                    runner.exec_step(step).await.unwrap();
                }
            }

            runner
                .exec_step(ScenarioStep::CheckTimeouts { node_id })
                .await
                .unwrap();

            let new = Instant::now();
            let elapsed = new - last_time;
            last_time = new;

            match timeout.checked_sub(elapsed) {
                Some(new_timeout) => timeout = new_timeout,
                None => panic!("timeout"),
            }

            runner
                .exec_step(ScenarioStep::AdvanceNodeTime {
                    node_id,
                    by_nanos: elapsed.as_nanos() as _,
                })
                .await
                .unwrap();

            let this = runner.node(node_id).unwrap();
            let best_chain = &this.state().transition_frontier.best_chain;
            let sync_state = &this.state().transition_frontier.sync;
            if let Synced { time } = &sync_state {
                if best_chain.len() > 1 {
                    eprintln!(
                        "node: {node_id}, is synced at {time:?}, total elapsed {:?}",
                        TIMEOUT - timeout
                    );

                    break;
                }
            }
        }
    }
}
