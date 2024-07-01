use serde::{Deserialize, Serialize};

pub use super::ocaml::*;
pub use super::rust::*;

#[derive(Serialize, Deserialize, derive_more::From, Debug, Clone)]
#[serde(tag = "kind")]
#[allow(clippy::large_enum_variant)]
pub enum NodeTestingConfig {
    Rust(RustNodeTestingConfig),
    Ocaml(OcamlNodeTestingConfig),
}
