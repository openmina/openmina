//! # Scenario Management
//!
//! This module provides functionality for managing test scenarios, including
//! loading, saving, and executing deterministic test sequences.
//!
//! ## How Scenarios Work
//!
//! ### Storage Format
//! Scenarios are stored as JSON files in the `res/scenarios/` directory relative
//! to the testing crate. Each scenario file contains:
//! - **ScenarioInfo**: Metadata (ID, description, parent relationships, node configs)
//! - **ScenarioSteps**: Ordered sequence of test actions to execute
//!
//! ### Load Process
//! 1. **File Location**: `load()` reads from `{CARGO_MANIFEST_DIR}/res/scenarios/{id}.json`
//! 2. **JSON Parsing**: Deserializes the file into a `Scenario` struct
//! 3. **Error Handling**: Returns `anyhow::Error` if file doesn't exist or is malformed
//!
//! ### Save Process
//! 1. **Atomic Write**: Uses temporary file + rename for atomic operations
//! 2. **Directory Creation**: Automatically creates `res/scenarios/` if needed
//! 3. **JSON Format**: Pretty-prints JSON for human readability
//! 4. **Temporary Files**: `.tmp.{scenario_id}.json` during write, renamed on success
//!
//! ### Scenario Inheritance
//! Scenarios can have parent-child relationships where child scenarios inherit
//! setup steps from their parents, enabling composition and reuse.
//!
//! For usage examples, see the [testing documentation](https://o1-labs.github.io/mina-rust/developers/testing/scenario-tests).

mod id;
pub use id::ScenarioId;

mod step;
pub use step::{ListenerNode, ScenarioStep};

mod event_details;
pub use event_details::event_details;

use anyhow::Context;
use mina_core::log::{debug, info, system_time};
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
        let mut files = tokio::fs::read_dir(Self::PATH).await.with_context(|| {
            format!(
                "Failed to read scenarios directory '{}'. Ensure the directory \
                exists or create it with: mkdir -p {}",
                Self::PATH,
                Self::PATH
            )
        })?;
        let mut list = vec![];

        while let Some(file) = files.next_entry().await? {
            let file_path = file.path();
            let encoded = tokio::fs::read(&file_path).await.with_context(|| {
                format!("Failed to read scenario file '{}'", file_path.display())
            })?;
            // TODO(binier): maybe somehow only parse info part of json?
            let full: Self = serde_json::from_slice(&encoded).with_context(|| {
                format!(
                    "Failed to parse scenario file '{}' as valid JSON",
                    file_path.display()
                )
            })?;
            list.push(full.info);
        }

        Ok(list)
    }

    /// Load a scenario from disk by ID.
    ///
    /// This method reads the scenario file from `res/scenarios/{id}.json`,
    /// deserializes it from JSON, and returns the complete scenario including
    /// both metadata and steps.
    ///
    /// # Arguments
    /// * `id` - The scenario identifier used to construct the file path
    ///
    /// # Returns
    /// * `Ok(Scenario)` - Successfully loaded scenario
    /// * `Err(anyhow::Error)` - File not found, invalid JSON, or I/O error
    pub async fn load(id: &ScenarioId) -> Result<Self, anyhow::Error> {
        let path = Self::file_path_by_id(id);
        debug!(system_time(); "Loading scenario '{}' from file '{}'", id, path);
        let encoded = tokio::fs::read(&path).await.with_context(|| {
            format!(
                "Failed to read scenario file '{}'. Ensure the scenario exists. \
                If using scenarios-run, the scenario must be generated first using \
                scenarios-generate, or check if the required feature flags (like \
                'p2p-webrtc') are enabled",
                path
            )
        })?;
        let scenario = serde_json::from_slice(&encoded)
            .with_context(|| format!("Failed to parse scenario file '{}' as valid JSON", path))?;
        info!(system_time(); "Successfully loaded scenario '{}'", id);
        Ok(scenario)
    }

    /// Reload this scenario from disk, discarding any in-memory changes.
    pub async fn reload(&mut self) -> Result<(), anyhow::Error> {
        *self = Self::load(&self.info.id).await?;
        Ok(())
    }

    /// Save the scenario to disk using atomic write operations.
    ///
    /// This method implements atomic writes by:
    /// 1. Creating the scenarios directory if it doesn't exist
    /// 2. Writing to a temporary file (`.tmp.{id}.json`)
    /// 3. Pretty-printing JSON for human readability
    /// 4. Atomically renaming the temp file to the final name
    ///
    /// This ensures the scenario file is never in a partially-written state,
    /// preventing corruption during concurrent access or system crashes.
    ///
    /// # File Location
    /// Saves to: `{CARGO_MANIFEST_DIR}/res/scenarios/{id}.json`
    ///
    /// # Errors
    /// Returns error if:
    /// - Cannot create the scenarios directory
    /// - Cannot serialize scenario to JSON
    /// - File I/O operations fail
    /// - Atomic rename fails
    pub async fn save(&self) -> Result<(), anyhow::Error> {
        let tmp_file = self.tmp_file_path();
        let final_file = self.file_path();

        debug!(system_time(); "Saving scenario '{}' to file '{}'", self.info.id, final_file);

        let encoded = serde_json::to_vec_pretty(self)
            .with_context(|| format!("Failed to serialize scenario '{}' to JSON", self.info.id))?;

        tokio::fs::create_dir_all(Self::PATH)
            .await
            .with_context(|| format!("Failed to create scenarios directory '{}'", Self::PATH))?;

        tokio::fs::write(&tmp_file, encoded)
            .await
            .with_context(|| format!("Failed to write temporary scenario file '{}'", tmp_file))?;

        tokio::fs::rename(&tmp_file, &final_file)
            .await
            .with_context(|| {
                format!(
                    "Failed to rename temporary file '{}' to final scenario file '{}'",
                    tmp_file, final_file
                )
            })?;

        info!(system_time(); "Successfully saved scenario '{}'", self.info.id);
        Ok(())
    }

    /// Synchronous version of `save()` for use in non-async contexts.
    ///
    /// Implements the same atomic write pattern as `save()` but uses
    /// blocking I/O operations instead of async.
    pub fn save_sync(&self) -> Result<(), anyhow::Error> {
        let tmp_file = self.tmp_file_path();
        let final_file = self.file_path();

        debug!(system_time(); "Saving scenario '{}' to file '{}'", self.info.id, final_file);

        let encoded = serde_json::to_vec_pretty(self)
            .with_context(|| format!("Failed to serialize scenario '{}' to JSON", self.info.id))?;

        std::fs::create_dir_all(Self::PATH)
            .with_context(|| format!("Failed to create scenarios directory '{}'", Self::PATH))?;

        std::fs::write(&tmp_file, encoded)
            .with_context(|| format!("Failed to write temporary scenario file '{}'", tmp_file))?;

        std::fs::rename(&tmp_file, &final_file).with_context(|| {
            format!(
                "Failed to rename temporary file '{}' to final scenario file '{}'",
                tmp_file, final_file
            )
        })?;

        info!(system_time(); "Successfully saved scenario '{}'", self.info.id);
        Ok(())
    }
}
