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

    pub enabled_channels: BTreeSet<ChannelId>,

    /// Maximal allowed number of connections.
    pub max_peers: usize,

    pub timeouts: P2pTimeouts,

    /// Chain id
    pub chain_id: String,

    /// Use peers discovery.
    pub peer_discovery: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pTimeouts {
    pub incoming_connection_timeout: Option<Duration>,
    pub outgoing_connection_timeout: Option<Duration>,
    pub reconnect_timeout: Option<Duration>,
    pub incoming_error_reconnect_timeout: Option<Duration>,
    pub outgoing_error_reconnect_timeout: Option<Duration>,
    pub best_tip_with_proof: Option<Duration>,
    pub ledger_query: Option<Duration>,
    pub staged_ledger_aux_and_pending_coinbases_at_block: Option<Duration>,
    pub block: Option<Duration>,
    pub snark: Option<Duration>,
    pub initial_peers: Option<Duration>,
    pub kademlia_bootstrap: Option<Duration>,
}

impl Default for P2pTimeouts {
    fn default() -> Self {
        Self {
            incoming_connection_timeout: Some(Duration::from_secs(30)),
            outgoing_connection_timeout: Some(Duration::from_secs(10)),
            reconnect_timeout: Some(Duration::from_secs(1)),
            incoming_error_reconnect_timeout: Some(Duration::from_secs(30)),
            outgoing_error_reconnect_timeout: Some(Duration::from_secs(30)),
            best_tip_with_proof: Some(Duration::from_secs(10)),
            ledger_query: Some(Duration::from_secs(2)),
            staged_ledger_aux_and_pending_coinbases_at_block: Some(Duration::from_secs(120)),
            block: Some(Duration::from_secs(5)),
            snark: Some(Duration::from_secs(5)),
            initial_peers: Some(Duration::from_secs(5)),
            kademlia_bootstrap: Some(Duration::from_secs(60)),
        }
    }
}

impl P2pTimeouts {
    pub fn without_rpc() -> Self {
        Self {
            best_tip_with_proof: None,
            ledger_query: None,
            staged_ledger_aux_and_pending_coinbases_at_block: None,
            block: None,
            snark: None,
            ..Default::default()
        }
    }

}
