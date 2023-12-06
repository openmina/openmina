# Basic Connectivity

## Connection Handling

### All connections should be tracked by the state machine

Connections that are initiated outside of the state machine (e.g. by Kademlia)
should be present in the state machine.

**Tests:** TODO

## Incoming Connections

### Node should accept incoming connections

We should accept an incoming connection from an arbitrary node.

**Tests:**
- [solo_node_accept_incoming](../node/testing/src/scenarios/solo_node/basic_connectivity_accept_incoming.rs)
- [multi_node_connection_discovery/OCamlToRust](../node/testing/src/scenarios/multi_node/connection_discovery.rs#L127) (should be replaced with one with non-OCaml peer)
- TODO: fast-running short test

### Node shouldn't accept duplicate incoming connections

The Rust node should reject a connection from a peer if there is one with the same
peer ID already.

**Tests:** TODO

### Node shouldn't try to connect to itself

The node can obtain its address from other peers. It shouldn't be used when connecting to new peers.

**Tests:** TODO

### Node shouldn't accept connection from itself

The node shouldn't accept a connection from a peer that uses the same peer id as
this one. This is either a program error (see above), network setup error, or a
malicious node that uses the same peer ID.

**Tests:** TODO

## Outgoing connections

### Node should connect to all available initial peers

TODO: what if the number of initial peers exceeds the max number of peers?

- [multi_node_initial_joining](../node/testing/src/scenarios/multi_node/basic_connectivity_initial_joining.rs) (partially?)

### Node should be able to connect to initial peers eventually, even if initially they are not available.

If, for some reason, the node can't connect to enough peers (e.g. it is the first
node in the network), it should keep retrying to those with failures (see also
below).

TODO: Use cases where this is important.

**Tests:** TODO

### Node should have a reasonable retry rate for reconnection

We should consider different reasons why the outgoing connection failed. The Rust
node shouldn't reconnect too soon to a node that dropped the connection.

**Tests:** TODO

## Peers Discovery

### Node advertises itself through Kademlia

- [solo_node_accept_incoming](../node/testing/src/scenarios/solo_node/basic_connectivity_accept_incoming.rs) (TODO: should be replaced by one with Rust-only peer)

### Node should be able to perform initial peer selection (Kademlia "bootstrap")

During this stage, the node queries its existing peers for more peers, thus getting more peer addresses.

See #148.

**Tests:** TODO

### Node should be able to select random peers for performing outgoing connections.

See #148.

To obtain a set of random peers, the Rust node performs a Kademlia query
that returns a list of peers that are "close" to some random peer.

This step starts after Kademlia initialization is complete.

- [multi_node_peer_discovery](../node/testing/src/scenarios/multi_node/basic_connectivity_peer_discovery.rs) (partially, should be replaced with one with a non-OCaml peer)
- [multi_node_connection_discovery/OCamlToRust](../node/testing/src/scenarios/multi_node/connection_discovery.rs#L127) (indirectly, should be replaced with one with a non-OCaml peer)
- TODO: fast-running Rust-only test

### Node should only advertise its "real" address

This is one of the requirements of LibP2P's Kademlia implementation. E.g. the
node behind NAT shouldn't advertise its address obtained using external
"what-is-my-IP"-like services.

**Tests:** TODO

# OCaml Node Interoperability

## Peers Discovery

### Advertising to OCaml nodes

- [solo_node_accept_incoming](../node/testing/src/scenarios/solo_node/basic_connectivity_accept_incoming.rs)
- [multi_node_connection_discovery/OCamlToRustViaSeed](../node/testing/src/scenarios/multi_node/connection_discovery.rs#L267)

### Peer discovery via Rust node

If a Rust node is used as a seed node, OCaml nodes connected to it should be
able to also discover and connect to each other.

- [multi_node_connection_discovery/RustAsSeed](../node/testing/src/scenarios/multi_node/connection_discovery.rs#L25)

### Peer discovery using OCaml seed node

- [multi_node_peer_discovery](../node/testing/src/scenarios/multi_node/basic_connectivity_peer_discovery.rs)
- [multi_node_connection_discovery/RustToOCamlViaSeed](../node/testing/src/scenarios/multi_node/connection_discovery.rs#L362)

## Incoming Connections

### OCaml node should be able to successfully connect to a Rust node

- [solo_node_accept_incoming](../node/testing/src/scenarios/solo_node/basic_connectivity_accept_incoming.rs)
- [multi_node_connection_discovery/OCamlToRust](../node/testing/src/scenarios/multi_node/connection_discovery.rs#L127)

## Outgoing Connections

### Rust node should be able to successfully connect to an OCaml node

- [multi_node_connection_discovery/RustToOCaml](../node/testing/src/scenarios/multi_node/connection_discovery.rs#201)

# General Safety

## Peer Maintaining

### Initial peer connection

The node should connect to as many peers as it is configured to (between min and max number).

- [multi_node_initial_joining](../node/testing/src/scenarios/multi_node/basic_connectivity_initial_joining.rs)

### Peer disconnection

The node should maintain a minimal number of peers in case it is disconnected
from its existing peers.

**Tests:** TODO

# Attacks Resistance


## DDoS 

**Tests:** TODO

## Eclipse Attack

**Tests:** TODO

## Sybil Attack

**Tests:** TODO
