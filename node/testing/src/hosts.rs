use crate::scenario::ListenerNode;
use node::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;
/// This should be the only place where environment variables are converted to addresses.
///
use std::{env, str::FromStr};

pub fn replayer() -> P2pConnectionOutgoingInitOpts {
    let multiaddr = env::var("REPLAYER_MULTIADDR")
        .expect("must set variable `REPLAYER_MULTIADDR`")
        .parse::<libp2p::Multiaddr>()
        .expect("`REPLAYER_MULTIADDR` must be a valid multiaddress");
    (&multiaddr).try_into().expect("must be valid init opts")
}

pub fn devnet() -> Vec<ListenerNode> {
    let seeds =
        std::env::var("MINA_SCENARIO_SEEDS").unwrap_or_else(|_| node::p2p::DEVNET_SEEDS.join(" "));
    seeds
        .split_whitespace()
        .map(P2pConnectionOutgoingInitOpts::from_str)
        .filter_map(Result::ok)
        .filter_map(|p| p.with_host_resolved())
        .map(Into::into)
        .collect()
}
