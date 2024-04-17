use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{token::BroadcastAlgorithm, PeerId};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkPubsubState {
    pub clients: BTreeMap<PeerId, P2pNetworkPubsubClientState>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkPubsubClientState {
    pub protocol: BroadcastAlgorithm,
    pub topics: BTreeSet<String>,
}
