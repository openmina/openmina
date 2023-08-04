use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkPoolConfig {
    pub auto_commit: bool,
    pub commitment_timeout: Duration,
}
