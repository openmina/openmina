use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pDisconnectedState {
    pub reason: String,
}
