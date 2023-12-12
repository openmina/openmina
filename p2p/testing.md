# Basic Connectivity

## Connection Handling

### Two nodes connecting to each other should succeed

If two nodes are connecting to each other at the same time, they should be
connected, so each one has exactly one connection.

- - [`p2p_basic_connections(simultaneous_connections)`](../node/testing/src/scenarios/p2p/basic_connection_handling.rs#L25)


### All connections should be tracked by the state machine

Connections that are initiated outside of the state machine (e.g. by Kademlia)
should be present in the state machine.

**Tests:**
- [`p2p_basic_connections(all_nodes_connections_are_symmetric)`](../node/testing/src/scenarios/p2p/basic_connection_handling.rs#L98)
- [`p2p_basic_connections(seed_connections_are_symmetric)`](../node/testing/src/scenarios/p2p/basic_connection_handling.rs#L165)

### Number of active peers should not exceed configured maximum number

**Tests:**
- [`p2p_basic_connections(max_number_of_peers)`](../node/testing/src/scenarios/p2p/basic_connection_handling.rs#L242)


## Incoming Connections

### Node should accept incoming connections

We should accept an incoming connection from an arbitrary node.

**Tests:**
- [p2p_basic_incoming(accept_connection)](../node/testing/src/scenarios/p2p/basic_incoming_connections.rs#L16)
- [p2p_basic_incoming(accept_multiple_connections)](../node/testing/src/scenarios/p2p/basic_incoming_connections.rs#L62)
- [solo_node_accept_incoming](../node/testing/src/scenarios/solo_node/basic_connectivity_accept_incoming.rs)
- [multi_node_connection_discovery/OCamlToRust](../node/testing/src/scenarios/multi_node/connection_discovery.rs#L127) (should be replaced with one with non-OCaml peer)
- TODO: fast-running short test

### Node shouldn't accept duplicate incoming connections

The Rust node should reject a connection from a peer if there is one with the same
peer ID already.

**Tests:** TODO

### Node shouldn't accept connection with its own peer_id

The node shouldn't accept a connection from a peer that uses the same peer id as
this one. This is either a program error (see above), network setup error, or a
malicious node that uses the same peer ID.

**Tests:**
- [`p2p_basic_incoming(does_not_accept_self_connection)`](../node/testing/src/scenarios/p2p/basic_incoming_connections.rs#L120)


## Outgoing connections

### Node should connect to other nodes

- [`p2p_basic_outgoing(make_connection)`](../node/testing/src/scenarios/p2p/basic_outgoing_connections.rs#L19)
- [`p2p_basic_outgoing(make_multiple_connections)`](../node/testing/src/scenarios/p2p/basic_outgoing_connections.rs#L74)

### Node shouldn't try to make outgoing connection using its own peer_id

The node can obtain its address from other peers. It shouldn't use it when connecting to new peers.

**Tests:**
- [`p2p_basic_outgoing(dont_connect_to_node_same_id)`](node/testing/src/scenarios/p2p/basic_outgoing_connections.rs#L134)
- [`p2p_basic_outgoing(dont_connect_to_initial_peer_same_id)`](node/testing/src/scenarios/p2p/basic_outgoing_connections.rs#L187)
- [`p2p_basic_outgoing(dont_connect_to_self_initial_peer)`](node/testing/src/scenarios/p2p/basic_outgoing_connections.rs#L226)

### Node should connect to all available initial peers

TODO: what if the number of initial peers exceeds the max number of peers?

- [`p2p_basic_outgoing(connect_to_all_initial_peers)`](../node/testing/src/scenarios/p2p/basic_outgoing_connections.rs#L293)
- [multi_node_initial_joining](../node/testing/src/scenarios/multi_node/basic_connectivity_initial_joining.rs) (partially?)

### Node should be able to connect to initial peers eventually, even if initially they are not available.

If, for some reason, the node can't connect to enough peers (e.g. it is the first
node in the network), it should keep retrying to those with failures (see also
below).

TODO: Use cases where this is important.

**Tests:**
- [`p2p_basic_outgoing(connect_to_all_initial_peers_become_ready)`](../node/testing/src/scenarios/p2p/basic_outgoing_connections.rs#L362)

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
