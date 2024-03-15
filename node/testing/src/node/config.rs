use serde::{Deserialize, Serialize};

pub use super::ocaml::{
    DaemonJson, DaemonJsonGenConfig, OcamlNodeConfig, OcamlNodeExecutable, OcamlNodeTestingConfig,
    OcamlVrfOutput,
};
pub use super::rust::{RustNodeBlockProducerTestingConfig, RustNodeTestingConfig, TestPeerId};

#[derive(Serialize, Deserialize, derive_more::From, Debug, Clone)]
#[serde(tag = "kind")]
pub enum NodeTestingConfig {
    Rust(RustNodeTestingConfig),
    Ocaml(OcamlNodeTestingConfig),
}
