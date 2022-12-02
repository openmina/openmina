use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub snark: crate::snark::SnarkConfig,
}
