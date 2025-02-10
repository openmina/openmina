use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// TODO(adonagy): Do we need this? Is it just unnecessary boilerplate?

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArchiveConfig {
    pub address: String,
}

impl ArchiveConfig {
    pub fn new(work_dir: String) -> Self {
        Self { address: work_dir }
    }
}
