use serde::{Deserialize, Serialize};

use crate::{connection::outgoing::P2pConnectionOutgoingInitOpts, identity::PublicKey};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConfig {
    pub identity_pub_key: PublicKey,
    pub initial_peers: Vec<P2pConnectionOutgoingInitOpts>,

    pub max_peers: usize,
}
