use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct SnarkConfig {
    pub block_verifier_index: Arc<crate::VerifierIndex>,
    pub block_verifier_srs: Arc<Mutex<crate::VerifierSRS>>,
    pub work_verifier_index: Arc<crate::VerifierIndex>,
    pub work_verifier_srs: Arc<Mutex<crate::VerifierSRS>>,
}

impl std::fmt::Debug for SnarkConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SnarkConfig")
            .field("block_verifier_index", &"<content too big>")
            .field("block_verifier_srs", &"<content too big>")
            .field("work_verifier_index", &"<content too big>")
            .field("work_verifier_srs", &"<content too big>")
            .finish()
    }
}
