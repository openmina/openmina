use std::{collections::BTreeSet, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{
    channels::ChannelId, connection::outgoing::P2pConnectionOutgoingInitOpts, identity::PublicKey,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConfig {
    /// TCP port where libp2p is listening incoming connections.
    pub libp2p_port: Option<u16>,
    /// The HTTP port where signaling server is listening SDP offers and SDP answers.
    pub listen_port: u16,
    /// The public key used for authentication all p2p communication.
    pub identity_pub_key: PublicKey,
    /// A list addresses of seed nodes.
    pub initial_peers: Vec<P2pConnectionOutgoingInitOpts>,

    /// The time interval that must elapse before the next peer discovery request.
    /// The node periodically polls peers for their connections to keep our list up to date.
    pub ask_initial_peers_interval: Duration,

    /// Channels supported by the protocol: `BestTipPropagation`,
    /// `SnarkPropagation`, `SnarkJobCommitmentPropagation`, `Rpc`.
    pub enabled_channels: BTreeSet<ChannelId>,

    /// Maximal allowed number of connections.
    pub max_peers: usize,
}
