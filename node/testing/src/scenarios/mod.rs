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
pub mod solo_node;

mod cluster_runner;
use cluster_runner::ClusterRunner;

use strum_macros::{EnumIter, EnumString, IntoStaticStr};

use crate::cluster::{Cluster, ClusterConfig};
use crate::scenario::{Scenario, ScenarioId, ScenarioStep};

use self::multi_node::basic_connectivity_initial_joining::MultiNodeBasicConnectivityInitialJoining;
use self::solo_node::{
    basic_connectivity_initial_joining::SoloNodeBasicConnectivityInitialJoining,
    sync_root_snarked_ledger::SoloNodeSyncRootSnarkedLedger,
};

#[derive(EnumIter, EnumString, IntoStaticStr, Clone, Copy)]
#[strum(serialize_all = "kebab-case")]
pub enum Scenarios {
    SoloNodeSyncRootSnarkedLedger(SoloNodeSyncRootSnarkedLedger),
    SoloNodeBasicConnectivityInitialJoining(SoloNodeBasicConnectivityInitialJoining),
    MultiNodeBasicConnectivityInitialJoining(MultiNodeBasicConnectivityInitialJoining),
}

impl Scenarios {
    // Turn off global test
    pub fn iter() -> impl IntoIterator<Item = Scenarios> {
        <Self as strum::IntoEnumIterator>::iter().filter(|s| !s.skip())
    }

    fn skip(&self) -> bool {
        match self {
            Self::SoloNodeSyncRootSnarkedLedger(_) => true,
            Self::SoloNodeBasicConnectivityInitialJoining(_) => false,
            Self::MultiNodeBasicConnectivityInitialJoining(_) => true,
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
            Self::SoloNodeSyncRootSnarkedLedger(_) => None,
            Self::SoloNodeBasicConnectivityInitialJoining(_) => None,
            Self::MultiNodeBasicConnectivityInitialJoining(_) => None,
        }
    }

    pub fn parent_id(self) -> Option<ScenarioId> {
        self.parent().map(Self::id)
    }

    pub fn description(self) -> &'static str {
        use documented::Documented;
        match self {
            Self::SoloNodeSyncRootSnarkedLedger(_) => SoloNodeSyncRootSnarkedLedger::DOCS,
            Self::SoloNodeBasicConnectivityInitialJoining(_) => {
                SoloNodeBasicConnectivityInitialJoining::DOCS
            }
            Self::MultiNodeBasicConnectivityInitialJoining(_) => {
                MultiNodeBasicConnectivityInitialJoining::DOCS
            }
        }
    }

    pub fn blank_scenario(self) -> Scenario {
        let mut scenario = Scenario::new(self.id(), self.parent_id());
        scenario.set_description(self.description().to_owned());
        scenario.info.nodes = match self {
            Self::SoloNodeSyncRootSnarkedLedger(_) => vec![serde_json::from_str(
                r#"
            {
                "kind": "Rust",
                "chain_id": "3c41383994b87449625df91769dff7b507825c064287d30fada9286f3f1cb15e",
                "initial_time": 1695702049579000000,
                "max_peers": 100,
                "ask_initial_peers_interval": { "secs": 10, "nanos": 0 }
            }
                                                                           "#,
            )
            .unwrap()],
            // TODO(vlad9486):
            Self::SoloNodeBasicConnectivityInitialJoining(_) => vec![],
            Self::MultiNodeBasicConnectivityInitialJoining(_) => vec![],
        };

        scenario
    }

    async fn run<F>(self, cluster: &mut Cluster, add_step: F)
    where
        F: FnMut(&ScenarioStep),
    {
        let runner = ClusterRunner::new(cluster, add_step);
        match self {
            Self::SoloNodeSyncRootSnarkedLedger(v) => v.run(runner).await,
            Self::SoloNodeBasicConnectivityInitialJoining(v) => v.run(runner).await,
            Self::MultiNodeBasicConnectivityInitialJoining(v) => v.run(runner).await,
        }
    }

    pub async fn run_and_save(self, cluster: &mut Cluster) {
        let mut scenario = self.blank_scenario();
        self.run(cluster, |step| scenario.add_step(step.clone()).unwrap())
            .await;
        scenario
            .save()
            .await
            .expect("failed to save scenario after run");
    }

    pub async fn run_only(self, cluster: &mut Cluster) {
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
