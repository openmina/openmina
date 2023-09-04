use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkPoolConfig {
    pub auto_commit: bool,
}
