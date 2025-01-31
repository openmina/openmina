use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArchiveConfig {
    pub address: String,
}

impl ArchiveConfig {
    pub fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
        }
    }
}
