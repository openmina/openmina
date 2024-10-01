mod run;
pub use run::*;

use std::{path::PathBuf, time::Duration};

use ledger::BaseLedger;
use node::account::{AccountPublicKey, AccountSecretKey};
use node::{event_source::Event, ledger::LedgerService, ActionKind, State};
use rand::{rngs::StdRng, SeedableRng};
use time::OffsetDateTime;

use crate::node::OcamlStep;
use crate::{
    cluster::{Cluster, ClusterNodeId, ClusterOcamlNodeId},
    network_debugger::Debugger,
    node::{
        DaemonJson, DaemonJsonGenConfig, Node, NodeTestingConfig, NonDeterministicEvent, OcamlNode,
        OcamlNodeTestingConfig, RustNodeTestingConfig,
    },
    scenario::ScenarioStep,
    service::{DynEffects, PendingEventId},
};

pub struct ClusterRunner<'a> {
    cluster: &'a mut Cluster,
    add_step: Box<dyn 'a + FnMut(&ScenarioStep)>,
    rng: StdRng,
    latest_advance_time: Option<redux::Timestamp>,
}

impl<'a> ClusterRunner<'a> {
    pub fn new<F>(cluster: &'a mut Cluster, add_step: F) -> Self
    where
        F: 'a + FnMut(&ScenarioStep),
    {
        Self {
            cluster,
            add_step: Box::new(add_step),
            rng: StdRng::seed_from_u64(0),
            latest_advance_time: None,
        }
    }

    pub fn node(&self, node_id: ClusterNodeId) -> Option<&Node> {
        self.cluster.node(node_id)
    }

    fn node_mut(&mut self, node_id: ClusterNodeId) -> Option<&mut Node> {
        self.cluster.node_mut(node_id)
    }

    pub fn ocaml_node(&self, node_id: ClusterOcamlNodeId) -> Option<&OcamlNode> {
        self.cluster.ocaml_node(node_id)
    }

    pub fn nodes_iter(&self) -> impl Iterator<Item = (ClusterNodeId, &Node)> {
        self.cluster.nodes_iter()
    }

    pub fn ocaml_nodes_iter(&self) -> impl Iterator<Item = (ClusterOcamlNodeId, &OcamlNode)> {
        self.cluster.ocaml_nodes_iter()
    }

    pub fn daemon_json_gen(
        &mut self,
        genesis_timestamp: &str,
        config: DaemonJsonGenConfig,
    ) -> DaemonJson {
        DaemonJson::gen(
            |sec_key| self.cluster.add_account_sec_key(sec_key),
            genesis_timestamp,
            config,
        )
    }

    pub fn daemon_json_gen_with_counts(
        &mut self,
        genesis_timestamp: &str,
        whales_n: usize,
        fish_n: usize,
    ) -> DaemonJson {
        DaemonJson::gen_with_counts(
            |sec_key| self.cluster.add_account_sec_key(sec_key),
            genesis_timestamp,
            whales_n,
            fish_n,
        )
    }

    pub fn daemon_json_load(&mut self, path: PathBuf, genesis_timestamp: &str) -> DaemonJson {
        DaemonJson::load(
            |sec_key| self.cluster.add_account_sec_key(sec_key),
            path,
            Some(genesis_timestamp),
        )
    }

    pub fn get_initial_time(&self) -> Option<redux::Timestamp> {
        self.cluster.get_initial_time()
    }

    pub fn set_initial_time(&mut self, initial_time: redux::Timestamp) {
        self.cluster.set_initial_time(initial_time)
    }

    pub fn get_account_sec_key(&self, pub_key: &AccountPublicKey) -> Option<&AccountSecretKey> {
        self.cluster.get_account_sec_key(pub_key)
    }

    pub fn add_rust_node(&mut self, testing_config: RustNodeTestingConfig) -> ClusterNodeId {
        let step = ScenarioStep::AddNode {
            config: Box::new(testing_config.into()),
        };
        (self.add_step)(&step);
        let ScenarioStep::AddNode { config } = step else {
            unreachable!()
        };
        let NodeTestingConfig::Rust(config) = *config else {
            unreachable!()
        };

        self.cluster.add_rust_node(config)
    }

    pub fn add_ocaml_node(&mut self, testing_config: OcamlNodeTestingConfig) -> ClusterOcamlNodeId {
        let step = ScenarioStep::AddNode {
            config: Box::new(testing_config.into()),
        };
        (self.add_step)(&step);
        let ScenarioStep::AddNode { config } = step else {
            unreachable!()
        };
        let NodeTestingConfig::Ocaml(config) = *config else {
            unreachable!()
        };

        self.cluster.add_ocaml_node(config)
    }

    pub async fn exec_step(&mut self, step: ScenarioStep) -> anyhow::Result<bool> {
        match &step {
            ScenarioStep::Event { node_id, event } => {
                let node_id = *node_id;
                let event_id = self.cluster.wait_for_pending_event(node_id, event).await?;
                let node = self.cluster.node(node_id).unwrap();
                let event_ref = node.get_pending_event(event_id).unwrap();
                if let Some(event) = NonDeterministicEvent::new(event_ref) {
                    (self.add_step)(&ScenarioStep::NonDeterministicEvent { node_id, event });
                } else {
                    (self.add_step)(&step);
                }
                Ok(self
                    .node_mut(node_id)
                    .unwrap()
                    .take_event_and_dispatch(event_id))
            }
            _ => {
                (self.add_step)(&step);
                self.cluster.exec_step(step).await
            }
        }
    }

    async fn exec_step_with_dyn_effects(
        &mut self,
        dyn_effects: DynEffects,
        node_id: ClusterNodeId,
        step: ScenarioStep,
    ) -> DynEffects {
        self.node_mut(node_id).unwrap().set_dyn_effects(dyn_effects);
        self.exec_step(step).await.unwrap();
        self.node_mut(node_id)
            .unwrap()
            .remove_dyn_effects()
            .unwrap()
    }

    pub async fn run_until_nodes_synced(
        &mut self,
        mut timeout: Duration,
        nodes: &[ClusterNodeId],
    ) -> anyhow::Result<()> {
        while !timeout.is_zero()
            && !nodes.iter().all(|node| {
                self.node(*node)
                    .unwrap()
                    .state()
                    .transition_frontier
                    .sync
                    .is_synced()
            })
        {
            let t = redux::Instant::now();
            self.run(
                RunCfg::default()
                    .timeout(timeout)
                    .action_handler(|_, _, _, action| {
                        matches!(action.action().kind(), ActionKind::TransitionFrontierSynced)
                    }),
            )
            .await?;
            timeout = timeout.checked_sub(t.elapsed()).unwrap_or_default();
        }
        if timeout.is_zero() {
            anyhow::bail!("timeout has elapsed while waiting for nodes to be synced");
        }
        Ok(())
    }

    pub fn pending_events(
        &mut self,
        poll: bool,
    ) -> impl Iterator<
        Item = (
            ClusterNodeId,
            &State,
            impl Iterator<Item = (PendingEventId, &Event)>,
        ),
    > {
        self.cluster.pending_events(poll)
    }

    pub fn node_pending_events(
        &mut self,
        node_id: ClusterNodeId,
        poll: bool,
    ) -> anyhow::Result<(&State, impl Iterator<Item = (PendingEventId, &Event)>)> {
        self.cluster.node_pending_events(node_id, poll)
    }

    pub async fn wait_for_pending_events(&mut self) {
        self.cluster.wait_for_pending_events().await
    }

    pub async fn wait_for_pending_events_with_timeout(&mut self, timeout: Duration) -> bool {
        self.cluster
            .wait_for_pending_events_with_timeout(timeout)
            .await
    }

    pub fn debugger(&self) -> Option<&Debugger> {
        self.cluster.debugger()
    }

    /// Block producer accounts, ordered by total stake, largest first.
    ///
    /// Warning: caller must ensure we are using custom daemon json if
    /// this method is called, so that we have secret keys for
    /// all block producers.
    pub fn block_producer_sec_keys(&self, node_id: ClusterNodeId) -> Vec<(AccountSecretKey, u64)> {
        let Some(block_producers) = None.or_else(|| {
            let node = self.node(node_id)?;
            let best_tip = node.state().transition_frontier.best_tip()?;
            let staking_ledger_hash = best_tip.staking_epoch_ledger_hash();
            LedgerService::ledger_manager(node.service()).producers_with_delegates(
                staking_ledger_hash,
                move |pub_key| {
                    pub_key != &AccountSecretKey::genesis_producer().public_key_compressed()
                },
            )
        }) else {
            return Default::default();
        };

        let mut block_producers = block_producers
            .into_iter()
            .map(|(pub_key, delegates)| {
                let sec_key = self
                    .get_account_sec_key(&pub_key)
                    .expect("sec key for block producer not found");
                let stake: u64 = delegates.into_iter().map(|(_, _, balance)| balance).sum();
                (sec_key.clone(), stake)
            })
            .collect::<Vec<_>>();

        // order by stake
        block_producers.sort_by(|(_, s1), (_, s2)| s2.cmp(s1));
        block_producers
    }

    pub fn accounts_with_sec_keys<'b>(
        &'b self,
        node_id: ClusterNodeId,
    ) -> Box<dyn 'b + Iterator<Item = (AccountSecretKey, Box<ledger::Account>)>> {
        let Some(mask) = self.node(node_id).and_then(|node| {
            let best_tip = node.state().transition_frontier.best_tip()?;
            let ledger_hash = best_tip.merkle_root_hash();
            let (mask, _) = LedgerService::ledger_manager(node.service()).get_mask(ledger_hash)?;
            Some(mask)
        }) else {
            return Box::new(std::iter::empty());
        };

        let depth = mask.depth() as usize;
        let num_accounts = mask.num_accounts() as u64;
        Box::new(
            (0..num_accounts)
                .map(ledger::AccountIndex)
                .filter_map(move |index| mask.get(ledger::Address::from_index(index, depth)))
                .filter_map(|account| {
                    let pub_key = account.public_key.clone().into();
                    let sec_key = self.get_account_sec_key(&pub_key)?;
                    Some((sec_key.clone(), account))
                }),
        )
    }

    /// Produces blocks in 5 second run intervals advancing time to the next won slot each time until predicate is true
    /// Assumes there is a block producer running in the cluster
    pub async fn produce_blocks_until<F>(
        &mut self,
        producer_node: ClusterNodeId,
        log_tag: &str,
        timeout: Duration,
        step_duration: Duration,
        keep_synced: bool,
        predicate: F,
    ) -> u32
    where
        F: Fn(&State, u32, u32) -> bool,
    {
        let now = tokio::time::Instant::now();

        let mut last_slot: u32 = 0;
        let mut produced_blocks: u32 = 0;

        let nodes: Vec<_> = self.nodes_iter().map(|(id, _)| id).collect();
        while now.elapsed() <= timeout {
            // andvance the time to slot 1
            // TODO: this should be the next won slot, not slot 1
            if last_slot == 0 {
                let by_nanos = Duration::from_secs(3 * 60).as_nanos() as u64;
                self.exec_step(ScenarioStep::AdvanceTime { by_nanos })
                    .await
                    .unwrap();
            }

            // run
            let _ = self.run(RunCfg::default().timeout(step_duration)).await;
            if keep_synced {
                // make sure every node is synced, longer timeout in case one node disconnects and it needs to resync
                self.run_until_nodes_synced(Duration::from_secs(5 * 60), &nodes)
                    .await
                    .unwrap();
            }

            let (state, _) = self.node_pending_events(producer_node, false).unwrap();

            let current_state_machine_time = state.time();
            let current_state_machine_time_u64: u64 = current_state_machine_time.into();
            let current_state_machine_time_formated =
                OffsetDateTime::from_unix_timestamp_nanos(current_state_machine_time_u64 as i128)
                    .unwrap();

            let best_tip = if let Some(best_tip) = state.transition_frontier.best_tip() {
                best_tip
            } else {
                eprintln!("[{log_tag}] No best tip");
                continue;
            };

            let current_global_slot = state.cur_global_slot().unwrap();

            let next_won_slot = state
                .block_producer
                .vrf_evaluator()
                .and_then(|vrf_state| vrf_state.next_won_slot(current_global_slot, best_tip));

            let best_tip_slot = &best_tip
                .consensus_state()
                .curr_global_slot_since_hard_fork
                .slot_number
                .as_u32();

            let current_time = OffsetDateTime::now_utc();
            eprintln!("[{log_tag}][{current_time}][{current_state_machine_time_formated}] Slot(best tip / current slot): {best_tip_slot} / {current_global_slot}");

            if best_tip_slot <= &0 {
                let by_nanos = Duration::from_secs(3 * 60).as_nanos() as u64;
                self.exec_step(ScenarioStep::AdvanceTime { by_nanos })
                    .await
                    .unwrap();
                continue;
            } else if best_tip_slot > &last_slot {
                last_slot = *best_tip_slot;
                produced_blocks += 1;
            } else {
                continue;
            }

            let (state, _) = self.node_pending_events(producer_node, false).unwrap();

            if predicate(state, last_slot, produced_blocks) {
                eprintln!("[{log_tag}] Condition met");
                return produced_blocks;
            }

            if let Some(won_slot) = next_won_slot {
                if let Some(diff) = won_slot.slot_time.checked_sub(current_state_machine_time) {
                    eprintln!("[{log_tag}] advancing time by {diff:?}");
                    let by_nanos = diff.as_nanos() as u64;
                    self.exec_step(ScenarioStep::AdvanceTime { by_nanos })
                        .await
                        .unwrap();
                } else {
                    continue;
                }
            } else {
                continue;
            }
        }

        panic!("Global timeout reached");
    }

    /// Skip to 3 blocks before the epoch end by advancing time
    /// Assumes there is a block producer running in the cluster
    pub async fn advance_to_epoch_bounds(
        &mut self,
        producer_node: ClusterNodeId,
        timeout: Duration,
        step_duration: Duration,
    ) -> u32 {
        const SLOTS_PER_EPOCH: u32 = 7_140;

        let (state, _) = self.node_pending_events(producer_node, false).unwrap();
        let current_epoch = state.current_epoch().unwrap();
        let latest_slot = state.cur_global_slot().unwrap();
        let current_epoch_end = current_epoch * SLOTS_PER_EPOCH + SLOTS_PER_EPOCH - 1;
        let to_epoch_bound = ((current_epoch_end - latest_slot) - 3) as u64;

        let diff = Duration::from_secs(3 * 60 * to_epoch_bound);

        eprintln!("[EPOCH BOUNDS] advancing time by {diff:?}");
        let by_nanos = diff.as_nanos() as u64;
        self.exec_step(ScenarioStep::AdvanceTime { by_nanos })
            .await
            .unwrap();

        self.produce_blocks_until(
            producer_node,
            "EPOCH BOUNDS",
            timeout,
            step_duration,
            true,
            |state, last_slot, produced_blocks| {
                eprintln!("\nSnarks: {}", state.snark_pool.last_index());
                eprintln!("Produced blocks: {produced_blocks}");
                last_slot >= current_epoch_end
            },
        )
        .await
    }

    pub async fn wait_for_ocaml(&mut self, node_id: ClusterOcamlNodeId) {
        self.exec_step(ScenarioStep::Ocaml {
            node_id,
            step: OcamlStep::WaitReady {
                timeout: Duration::from_secs(6 * 60),
            },
        })
        .await
        .expect("Error waiting for ocaml node");
    }
}
