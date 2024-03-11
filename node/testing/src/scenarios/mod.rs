//! Basic connectivity tests.
//! Initial Joining:
//! * Ensure new nodes can discover peers and establish initial connections.
//! * Test how nodes handle scenarios when they are overwhelmed with too many connections or data requests.
//! TODO(vlad9486):
//! Reconnection: Validate that nodes can reconnect after both intentional and unintentional disconnections.
//! Handling Latency: Nodes should remain connected and synchronize even under high latency conditions.
//! Intermittent Connections: Nodes should be resilient to sporadic network dropouts and still maintain synchronization.
//! Dynamic IP Handling: Nodes with frequently changing IP addresses should maintain stable connections.

pub mod multi_node;
pub mod simulation;
pub mod solo_node;

pub mod p2p;

mod cluster_runner;
pub use cluster_runner::*;
mod driver;
pub use driver::*;

use strum_macros::{EnumIter, EnumString, IntoStaticStr};

use crate::cluster::{Cluster, ClusterConfig};
use crate::scenario::{Scenario, ScenarioId, ScenarioStep};

use self::multi_node::basic_connectivity_initial_joining::MultiNodeBasicConnectivityInitialJoining;
use self::multi_node::basic_connectivity_peer_discovery::MultiNodeBasicConnectivityPeerDiscovery;
use self::multi_node::sync_4_block_producers::MultiNodeSync4BlockProducers;
use self::multi_node::vrf_correct_ledgers::MultiNodeVrfGetCorrectLedgers;
use self::multi_node::vrf_correct_slots::MultiNodeVrfGetCorrectSlots;
use self::multi_node::vrf_epoch_bounds_correct_ledgers::MultiNodeVrfEpochBoundsCorrectLedger;
use self::multi_node::vrf_epoch_bounds_evaluation::MultiNodeVrfEpochBoundsEvaluation;
use self::simulation::small::SimulationSmall;
use self::solo_node::sync_to_genesis::SoloNodeSyncToGenesis;
use self::solo_node::sync_to_genesis_custom::SoloNodeSyncToGenesisCustom;
use self::solo_node::{
    basic_connectivity_accept_incoming::SoloNodeBasicConnectivityAcceptIncoming,
    basic_connectivity_initial_joining::SoloNodeBasicConnectivityInitialJoining,
    bootstrap::SoloNodeBootstrap, sync_root_snarked_ledger::SoloNodeSyncRootSnarkedLedger,
};

#[derive(EnumIter, EnumString, IntoStaticStr, derive_more::From, Clone, Copy)]
#[strum(serialize_all = "kebab-case")]
pub enum Scenarios {
    SoloNodeSyncToGenesis(SoloNodeSyncToGenesis),
    SoloNodeBootstrap(SoloNodeBootstrap),
    SoloNodeSyncToGenesisCustom(SoloNodeSyncToGenesisCustom),
    SoloNodeSyncRootSnarkedLedger(SoloNodeSyncRootSnarkedLedger),
    SoloNodeBasicConnectivityInitialJoining(SoloNodeBasicConnectivityInitialJoining),
    SoloNodeBasicConnectivityAcceptIncoming(SoloNodeBasicConnectivityAcceptIncoming),
    MultiNodeSync4BlockProducers(MultiNodeSync4BlockProducers),
    MultiNodeVrfGetCorrectLedgers(MultiNodeVrfGetCorrectLedgers),
    MultiNodeVrfGetCorrectSlots(MultiNodeVrfGetCorrectSlots),
    MultiNodeVrfEpochBoundsEvaluation(MultiNodeVrfEpochBoundsEvaluation),
    MultiNodeVrfEpochBoundsCorrectLedger(MultiNodeVrfEpochBoundsCorrectLedger),
    MultiNodeBasicConnectivityInitialJoining(MultiNodeBasicConnectivityInitialJoining),
    MultiNodeBasicConnectivityPeerDiscovery(MultiNodeBasicConnectivityPeerDiscovery),
    SimulationSmall(SimulationSmall),
}

impl Scenarios {
    // Turn off global test
    pub fn iter() -> impl IntoIterator<Item = Scenarios> {
        <Self as strum::IntoEnumIterator>::iter().filter(|s| !s.skip())
    }

    fn skip(&self) -> bool {
        match self {
            Self::SoloNodeSyncToGenesis(_) => true,
            Self::SoloNodeBootstrap(_) => false,
            Self::SoloNodeSyncToGenesisCustom(_) => true,
            Self::SoloNodeSyncRootSnarkedLedger(_) => false,
            Self::SoloNodeBasicConnectivityInitialJoining(_) => false,
            Self::SoloNodeBasicConnectivityAcceptIncoming(_) => cfg!(feature = "p2p-webrtc"),
            Self::MultiNodeSync4BlockProducers(_) => false,
            Self::MultiNodeVrfGetCorrectLedgers(_) => false,
            Self::MultiNodeVrfGetCorrectSlots(_) => false,
            Self::MultiNodeVrfEpochBoundsEvaluation(_) => false,
            Self::MultiNodeVrfEpochBoundsCorrectLedger(_) => false,
            Self::MultiNodeBasicConnectivityInitialJoining(_) => false,
            Self::MultiNodeBasicConnectivityPeerDiscovery(_) => cfg!(feature = "p2p-webrtc"),
            Self::SimulationSmall(_) => false,
        }
    }

    pub fn id(self) -> ScenarioId {
        self.into()
    }

    pub fn to_str(self) -> &'static str {
        self.into()
    }

    pub fn parent(self) -> Option<Self> {
        match self {
            Self::SoloNodeSyncToGenesis(_) => None,
            Self::SoloNodeBootstrap(_) => None,
            Self::SoloNodeSyncToGenesisCustom(_) => None,
            Self::SoloNodeSyncRootSnarkedLedger(_) => None,
            Self::SoloNodeBasicConnectivityInitialJoining(_) => None,
            Self::SoloNodeBasicConnectivityAcceptIncoming(_) => None,
            Self::MultiNodeSync4BlockProducers(_) => Some(SoloNodeSyncToGenesis.into()),
            Self::MultiNodeVrfGetCorrectLedgers(_) => Some(SoloNodeSyncToGenesisCustom.into()),
            Self::MultiNodeVrfGetCorrectSlots(_) => Some(SoloNodeSyncToGenesisCustom.into()),
            Self::MultiNodeVrfEpochBoundsEvaluation(_) => Some(SoloNodeSyncToGenesisCustom.into()),
            Self::MultiNodeVrfEpochBoundsCorrectLedger(_) => {
                Some(SoloNodeSyncToGenesisCustom.into())
            }
            Self::MultiNodeBasicConnectivityInitialJoining(_) => None,
            Self::MultiNodeBasicConnectivityPeerDiscovery(_) => None,
            Self::SimulationSmall(_) => None,
        }
    }

    pub fn parent_id(self) -> Option<ScenarioId> {
        self.parent().map(Self::id)
    }

    pub fn description(self) -> &'static str {
        use documented::Documented;
        match self {
            Self::SoloNodeSyncToGenesis(_) => SoloNodeSyncToGenesis::DOCS,
            Self::SoloNodeBootstrap(_) => SoloNodeBootstrap::DOCS,
            Self::SoloNodeSyncToGenesisCustom(_) => SoloNodeSyncToGenesis::DOCS,
            Self::SoloNodeSyncRootSnarkedLedger(_) => SoloNodeSyncRootSnarkedLedger::DOCS,
            Self::SoloNodeBasicConnectivityInitialJoining(_) => {
                SoloNodeBasicConnectivityInitialJoining::DOCS
            }
            Self::SoloNodeBasicConnectivityAcceptIncoming(_) => {
                SoloNodeBasicConnectivityAcceptIncoming::DOCS
            }
            Self::MultiNodeSync4BlockProducers(_) => MultiNodeSync4BlockProducers::DOCS,
            Self::MultiNodeVrfGetCorrectLedgers(_) => MultiNodeVrfGetCorrectLedgers::DOCS,
            Self::MultiNodeVrfGetCorrectSlots(_) => MultiNodeVrfGetCorrectSlots::DOCS,
            Self::MultiNodeVrfEpochBoundsEvaluation(_) => MultiNodeVrfEpochBoundsEvaluation::DOCS,
            Self::MultiNodeVrfEpochBoundsCorrectLedger(_) => {
                MultiNodeVrfEpochBoundsCorrectLedger::DOCS
            }
            Self::MultiNodeBasicConnectivityInitialJoining(_) => {
                MultiNodeBasicConnectivityInitialJoining::DOCS
            }
            Self::MultiNodeBasicConnectivityPeerDiscovery(_) => {
                MultiNodeBasicConnectivityPeerDiscovery::DOCS
            }
            Self::SimulationSmall(_) => SimulationSmall::DOCS,
        }
    }

    pub fn blank_scenario(self) -> Scenario {
        let mut scenario = Scenario::new(self.id(), self.parent_id());
        scenario.set_description(self.description().to_owned());
        scenario.info.nodes = Vec::new();

        scenario
    }

    async fn run<F>(self, cluster: &mut Cluster, add_step: F)
    where
        F: FnMut(&ScenarioStep),
    {
        let runner = ClusterRunner::new(cluster, add_step);
        match self {
            Self::SoloNodeSyncToGenesis(v) => v.run(runner).await,
            Self::SoloNodeBootstrap(v) => v.run(runner).await,
            Self::SoloNodeSyncToGenesisCustom(v) => v.run(runner).await,
            Self::SoloNodeSyncRootSnarkedLedger(v) => v.run(runner).await,
            Self::SoloNodeBasicConnectivityInitialJoining(v) => v.run(runner).await,
            Self::SoloNodeBasicConnectivityAcceptIncoming(v) => v.run(runner).await,
            Self::MultiNodeSync4BlockProducers(v) => v.run(runner).await,
            Self::MultiNodeVrfGetCorrectLedgers(v) => v.run(runner).await,
            Self::MultiNodeVrfGetCorrectSlots(v) => v.run(runner).await,
            Self::MultiNodeVrfEpochBoundsEvaluation(v) => v.run(runner).await,
            Self::MultiNodeVrfEpochBoundsCorrectLedger(v) => v.run(runner).await,
            Self::MultiNodeBasicConnectivityInitialJoining(v) => v.run(runner).await,
            Self::MultiNodeBasicConnectivityPeerDiscovery(v) => v.run(runner).await,
            Self::SimulationSmall(v) => v.run(runner).await,
        }
    }

    pub async fn run_and_save(self, cluster: &mut Cluster) {
        struct ScenarioSaveOnExit(Scenario);

        impl Drop for ScenarioSaveOnExit {
            fn drop(&mut self) {
                let info = self.0.info.clone();
                let steps = std::mem::take(&mut self.0.steps);
                let scenario = Scenario { info, steps };

                eprintln!("saving scenario({}) before exit...", scenario.info.id);
                if let Err(err) = scenario.save_sync() {
                    eprintln!(
                        "failed to save scenario({})! error: {}",
                        scenario.info.id, err
                    );
                }
            }
        }

        eprintln!("run_and_save: {}", self.to_str());
        let mut scenario = ScenarioSaveOnExit(self.blank_scenario());
        self.run(cluster, |step| scenario.0.add_step(step.clone()).unwrap())
            .await;
        // drop to save it.
        let _ = scenario;
    }

    pub async fn run_only(self, cluster: &mut Cluster) {
        eprintln!("run_only: {}", self.to_str());
        self.run(cluster, |_| {}).await
    }

    async fn build_cluster_and_run_parents(self, config: ClusterConfig) -> Cluster {
        let mut parents = std::iter::repeat(())
            .scan(self.parent(), |parent, _| {
                let cur_parent = parent.take();
                *parent = cur_parent.and_then(|p| p.parent());
                cur_parent
            })
            .collect::<Vec<_>>();

        let mut cluster = Cluster::new(config);
        while let Some(scenario) = parents.pop() {
            scenario.run_only(&mut cluster).await;
        }

        cluster
    }

    pub async fn run_and_save_from_scratch(self, config: ClusterConfig) {
        let mut cluster = self.build_cluster_and_run_parents(config).await;
        self.run_and_save(&mut cluster).await;
    }

    pub async fn run_only_from_scratch(self, config: ClusterConfig) {
        let mut cluster = self.build_cluster_and_run_parents(config).await;
        self.run_only(&mut cluster).await;
    }
}
