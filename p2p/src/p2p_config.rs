use serde::{Deserialize, Serialize};

use crate::connection::outgoing::P2pConnectionOutgoingInitOpts;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConfig {
    pub initial_peers: Vec<P2pConnectionOutgoingInitOpts>,
}
