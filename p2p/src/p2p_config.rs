use std::{collections::BTreeSet, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{
    channels::ChannelId, connection::outgoing::P2pConnectionOutgoingInitOpts, identity::PublicKey,
};

pub const BERKELEY_SEEDS: &[&str] = &[
    "/ip4/34.70.183.166/tcp/10001/p2p/12D3KooWAdgYL6hv18M3iDBdaK1dRygPivSfAfBNDzie6YqydVbs",
    "/ip4/34.135.63.47/tcp/10001/p2p/12D3KooWLjs54xHzVmMmGYb7W5RVibqbwD1co7M2ZMfPgPm7iAag",
    "/ip4/34.170.114.52/tcp/10001/p2p/12D3KooWEiGVAFC7curXWXiGZyMWnZK9h8BKr88U8D5PKV3dXciv",
];

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

    pub timeouts: P2pTimeouts,

    pub limits: P2pLimits,

    /// Use peers discovery.
    pub peer_discovery: bool,

    /// Unix time. Used as an initial nonce for pubsub.
    pub initial_time: Duration,
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
    pub kademlia_initial_bootstrap: Option<Duration>,
    pub select: Option<Duration>,
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
            kademlia_initial_bootstrap: Some(Duration::from_secs(5)),
            select: Some(Duration::from_secs(5)),
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, derive_more::Display)]
pub enum Limit<T> {
    #[display(fmt = "{}", _0)]
    Some(T),
    #[display(fmt = "unlimited")]
    Unlimited,
}

impl<T> Limit<T> {
    pub fn map<F, O>(self, f: F) -> Limit<O>
    where
        F: FnOnce(T) -> O,
    {
        match self {
            Limit::Some(t) => Limit::Some(f(t)),
            Limit::Unlimited => Limit::Unlimited,
        }
    }
}

macro_rules! impls {
    ($ty:ty) => {
        impl From<Option<$ty>> for Limit<$ty> {
            fn from(value: Option<$ty>) -> Self {
                match value {
                    Some(v) => Limit::Some(v),
                    None => Limit::Unlimited,
                }
            }
        }

        impl From<Limit<$ty>> for Option<$ty> {
            fn from(value: Limit<$ty>) -> Self {
                match value {
                    Limit::Some(v) => Some(v),
                    Limit::Unlimited => None,
                }
            }
        }

        impl std::cmp::PartialEq<$ty> for Limit<$ty> {
            fn eq(&self, other: &$ty) -> bool {
                match self {
                    Limit::Some(v) => v.eq(other),
                    Limit::Unlimited => false,
                }
            }
        }

        impl std::cmp::PartialEq<Limit<$ty>> for $ty {
            fn eq(&self, other: &Limit<$ty>) -> bool {
                match other {
                    Limit::Some(other) => self.eq(other),
                    Limit::Unlimited => false,
                }
            }
        }

        impl std::cmp::PartialEq<Limit<$ty>> for Limit<$ty> {
            fn eq(&self, other: &Limit<$ty>) -> bool {
                match (self, other) {
                    (Limit::Some(this), Limit::Some(other)) => this.eq(other),
                    (Limit::Unlimited, Limit::Unlimited) => true,
                    _ => false,
                }
            }
        }

        impl std::cmp::Eq for Limit<$ty> {}

        impl std::cmp::PartialOrd<$ty> for Limit<$ty> {
            fn partial_cmp(&self, other: &$ty) -> Option<std::cmp::Ordering> {
                match self {
                    Limit::Some(v) => v.partial_cmp(other),
                    Limit::Unlimited => Some(std::cmp::Ordering::Greater),
                }
            }
        }

        impl std::cmp::PartialOrd<Limit<$ty>> for $ty {
            fn partial_cmp(&self, other: &Limit<$ty>) -> Option<std::cmp::Ordering> {
                match other {
                    Limit::Some(other) => self.partial_cmp(other),
                    Limit::Unlimited => Some(std::cmp::Ordering::Less),
                }
            }
        }
    };
}

impls!(usize);
impls!(std::time::Duration);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pLimits {
    max_peers: Limit<usize>,

    identify_message: Limit<usize>,
    kademlia_request: Limit<usize>,
    kademlia_response: Limit<usize>,

    rpc_service_message: Limit<usize>,
    rpc_query: Limit<usize>,
    rpc_get_best_tip: Limit<usize>,
    rpc_answer_sync_ledger_query: Limit<usize>,
    rpc_get_staged_ledger: Limit<usize>,
    rpc_get_transition_chain: Limit<usize>,
    rpc_get_some_initial_peers: Limit<usize>,
}

macro_rules! limit {
    (#[$meta:meta] $limit:ident) => {
        #[$meta]
        pub fn $limit(&self) -> Limit<usize> {
            self.$limit
        }
    };

    (#[$meta:meta] $limit:ident, #[$setter_meta:meta] $setter:ident) => {
        limit!(#[$meta] $limit);

        #[$setter_meta]
        pub fn $setter<T: Into<Limit<usize>>>(mut self, $limit: T) -> Self {
            self.$limit = $limit.into();
            self
        }
    };

    (#[$meta:meta] $limit:ident(&$self:ident): $expr:expr) => {
        #[$meta]
        pub fn $limit(&$self) -> Limit<usize> {
            $expr
        }
    };
}

impl P2pLimits {
    limit!(
        /// Maximum number of peers.
        max_peers,
        /// Sets maximum number of peers.
        with_max_peers
    );

    limit!(
        /// Minimum number of peers.
        min_peers(&self): self.max_peers.map(|v| (v / 2).max(3).min(v))
    );

    limit!(
        /// Maximum number of connections.
        max_connections(&self): self.max_peers.map(|v| v + 10)
    );

    limit!(
        /// Maximum length of Identify message.
        identify_message
    );
    limit!(
        /// Maximum length of Kademlia request message.
        kademlia_request
    );
    limit!(
        /// Maximum length of Kademlia response message.
        kademlia_response
    );

    limit!(
        #[doc = "RPC service message"]
        rpc_service_message
    );
    limit!(
        #[doc = "RPC query"]
        rpc_query
    );
    limit!(
        #[doc = "RPC get_best_tip"]
        rpc_get_best_tip
    );
    limit!(
        #[doc = "RPC answer_sync_ledger_query"]
        rpc_answer_sync_ledger_query
    );
    limit!(
        #[doc = "RPC get_staged_ledger"]
        rpc_get_staged_ledger
    );
    limit!(
        #[doc = "RPC get_transition_chain"]
        rpc_get_transition_chain
    );
    limit!(
        #[doc = "RPC some_initial_peers"]
        rpc_get_some_initial_peers
    );
}

impl Default for P2pLimits {
    fn default() -> Self {
        let max_peers = Limit::Some(100);

        let identify_message = Limit::Some(0x1000);
        let kademlia_request = Limit::Some(50);
        let kademlia_response = identify_message.map(|v| v * 20); // should be enough to fit 20 addresses supplied by identify

        let rpc_service_message = Limit::Some(7); // 7 for handshake, 1 for heartbeat
        let rpc_query = Limit::Some(256); // max is 96
        let rpc_get_best_tip = Limit::Some(3_500_000); // 3182930 as observed, may vary
        let rpc_answer_sync_ledger_query = Limit::Some(200_000); // 124823 as observed
        let rpc_get_staged_ledger = Limit::Some(40_000_000); // 36371615 as observed, may vary
        let rpc_get_transition_chain = Limit::Some(3_500_000); // 2979112 as observed
        let rpc_get_some_initial_peers = Limit::Some(32_000); // TODO: calculate
        Self {
            max_peers,

            identify_message,
            kademlia_request,
            kademlia_response,

            rpc_service_message,
            rpc_query,
            rpc_get_best_tip,
            rpc_answer_sync_ledger_query,
            rpc_get_staged_ledger,
            rpc_get_transition_chain,
            rpc_get_some_initial_peers,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::Limit;

    #[test]
    fn test_limits() {
        let limit = Limit::Some(10);
        assert!(0 < limit);
        assert!(10 <= limit);
        assert!(11 > limit);

        let unlimited = Limit::Unlimited;
        assert!(0 < unlimited);
        assert!(usize::MAX < unlimited);
    }
}
