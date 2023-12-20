use serde::{Deserialize, Serialize};

pub use super::rust::{RustNodeTestingConfig, TestPeerId};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum NodeTestingConfig {
    Rust(RustNodeTestingConfig),
}
