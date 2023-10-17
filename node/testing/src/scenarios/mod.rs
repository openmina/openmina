pub mod solo_node;

mod cluster_runner;
use cluster_runner::ClusterRunner;

use strum_macros::{EnumIter, EnumString, IntoStaticStr};

use crate::cluster::Cluster;
use crate::scenario::{Scenario, ScenarioId, ScenarioStep};

use self::solo_node::sync_root_snarked_ledger::SoloNodeSyncRootSnarkedLedger;

#[derive(EnumIter, EnumString, IntoStaticStr, Clone, Copy)]
#[strum(serialize_all = "kebab-case")]
pub enum Scenarios {
    SoloNodeSyncRootSnarkedLedger(SoloNodeSyncRootSnarkedLedger),
}

impl Scenarios {
    pub fn iter() -> ScenariosIter {
        <Self as strum::IntoEnumIterator>::iter()
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
        }
    }

    pub fn parent_id(self) -> Option<ScenarioId> {
        self.parent().map(Self::id)
    }

    pub fn description(self) -> &'static str {
        use documented::Documented;
        match self {
            Self::SoloNodeSyncRootSnarkedLedger(_) => SoloNodeSyncRootSnarkedLedger::DOCS,
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
                "initial_time": 1695702049579000000
            }
                                                                           "#,
            )
            .unwrap()],
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

    async fn build_cluster_and_run_parents(self) -> Cluster {
        let mut parents = std::iter::repeat(())
            .scan(self.parent(), |parent, _| {
                let cur_parent = parent.take();
                *parent = cur_parent.and_then(|p| p.parent());
                cur_parent
            })
            .collect::<Vec<_>>();

        let mut cluster = Cluster::new(Default::default());
        while let Some(scenario) = parents.pop() {
            scenario.run_only(&mut cluster).await;
        }

        cluster
    }

    pub async fn run_and_save_from_scratch(self) {
        let mut cluster = self.build_cluster_and_run_parents().await;
        self.run_and_save(&mut cluster).await;
    }

    pub async fn run_only_from_scratch(self) {
        let mut cluster = self.build_cluster_and_run_parents().await;
        self.run_only(&mut cluster).await;
    }
}
