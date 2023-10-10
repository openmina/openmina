mod id;
pub use id::ScenarioId;

mod step;
pub use step::{ListenerNode, ScenarioStep};

use serde::{Deserialize, Serialize};

use crate::node::NodeTestingConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Scenario {
    pub info: ScenarioInfo,
    pub steps: Vec<ScenarioStep>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScenarioInfo {
    pub id: ScenarioId,
    pub description: String,
    pub parent_id: Option<ScenarioId>,
    /// Nodes created in this scenario. Doesn't include ones defined in parent.
    pub nodes: Vec<NodeTestingConfig>,
}

impl Scenario {
    pub const PATH: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/res/scenarios");

    pub fn new(id: ScenarioId, parent_id: Option<ScenarioId>) -> Self {
        Self {
            info: ScenarioInfo {
                id,
                description: String::new(),
                parent_id,
                nodes: vec![],
            },
            steps: vec![],
        }
    }

    pub fn set_description(&mut self, description: String) {
        self.info.description = description;
    }

    pub fn add_node(&mut self, config: NodeTestingConfig) {
        self.info.nodes.push(config);
    }

    pub fn add_step(&mut self, step: ScenarioStep) -> Result<(), anyhow::Error> {
        self.steps.push(step);
        Ok(())
    }

    fn tmp_file_path(&self) -> String {
        format!("{}/.tmp.{}.json", Self::PATH, self.info.id)
    }

    pub fn file_path(&self) -> String {
        Self::file_path_by_id(&self.info.id)
    }

    fn file_path_by_id(id: &ScenarioId) -> String {
        format!("{}/{}.json", Self::PATH, id)
    }

    pub fn exists(id: &ScenarioId) -> bool {
        std::path::Path::new(&Self::file_path_by_id(id)).exists()
    }

    pub async fn list() -> Result<Vec<ScenarioInfo>, anyhow::Error> {
        let mut files = tokio::fs::read_dir(Self::PATH).await?;
        let mut list = vec![];

        while let Some(file) = files.next_entry().await? {
            let encoded = tokio::fs::read(file.path()).await?;
            // TODO(binier): maybe somehow only parse info part of json?
            let full: Self = serde_json::from_slice(&encoded)?;
            list.push(full.info);
        }

        Ok(list)
    }

    pub async fn load(id: &ScenarioId) -> Result<Self, anyhow::Error> {
        let encoded = tokio::fs::read(Self::file_path_by_id(id)).await?;
        Ok(serde_json::from_slice(&encoded)?)
    }

    pub async fn reload(&mut self) -> Result<(), anyhow::Error> {
        *self = Self::load(&self.info.id).await?;
        Ok(())
    }

    pub async fn save(&self) -> Result<(), anyhow::Error> {
        let tmp_file = self.tmp_file_path();
        let encoded = serde_json::to_vec_pretty(self)?;
        tokio::fs::write(&tmp_file, encoded).await?;
        Ok(tokio::fs::rename(tmp_file, self.file_path()).await?)
    }
}
