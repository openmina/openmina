/// Global test that aims to be deterministic.
/// Launch `TOTAL_PEERS` number of nodes with `MAX_PEERS_PER_NODE` is est as the maximum number of peers.
/// Launch a seed node where `TOTAL_PEERS` is set as the maximum number of peers.
/// Run the simulation until the following condition is satisfied:
/// * Each node is connected to a number of peers determined by the `P2pState::min_peers` method.
/// Fail the test if any node exceeds the maximum number of connections.
/// Fail the test if the specified number of steps occur but the condition is not met.
pub mod global;

/// Local test to ensure that the Openmina node can connect to an existing OCaml testnet.
/// Launch an Openmina node and connect it to seed nodes of the public (or private) OCaml testnet.
/// Run the simulation until:
/// * Number of known peers is greater than or equal to the maximum number of peers.
/// *Nnumber of connected peers is greater than or equal to some threshold.
/// Fail the test if the specified number of steps occur but the condition is not met.
pub mod local;

pub fn run() {
    local::run();
    global::run();
}
