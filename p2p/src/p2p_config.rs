use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    channels::ChannelId, connection::outgoing::P2pConnectionOutgoingInitOpts, identity::PublicKey,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConfig {
    pub identity_pub_key: PublicKey,
    pub initial_peers: Vec<P2pConnectionOutgoingInitOpts>,

    pub enabled_channels: BTreeSet<ChannelId>,

    pub max_peers: usize,
}
