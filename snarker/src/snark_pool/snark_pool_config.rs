use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkPoolConfig {
    pub commitment_timeout: Duration,
}
