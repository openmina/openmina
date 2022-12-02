use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkConfig {
    pub block_verifier_index: Arc<crate::VerifierIndex>,
    pub block_verifier_srs: Arc<crate::VerifierSRS>,
}
