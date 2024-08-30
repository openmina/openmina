# Testing

## Table of contents

- [P2P tests](#p2p-tests)
    - [RPC](#rpc)
    - [Kademlia](#kademlia)
    - [Identify](#identify)
    - [Connection](#connection)
- [Scenarios](#scenarios)
    - [Connection Discovery](#connection-discovery)
    - [P2P Connections](#p2p-connections)
    - [Kademlia](#p2p-kademlia)
    - [Pubsub](#p2p-pubsub)
    - [P2P Incoming](#p2p-incoming)
    - [P2p Outgoing](#p2p-outgoing)
    - [Single Node](#single-node)
    - [Multi Node](#multi-node)
    - [Record/Reply](#recordreplay)

## P2p tests

### [RPC](../../p2p/tests/rpc.rs)

* `rust_to_rust`: test that rust node can receive and send response to and from another rust node 
* `rust_to_many_rust_query`: tests that rust node can respond to many rust peers
* `rust_to_many_rust`: test that rust node can send request to many rust peers
* rpc tests, these tests check if node can correctly communicate over rpc:
* `initial_peers`: check that initial peers are correctly sent and received
* `best_tip_with_proof`: check that best tip is correctly sent and received
* `ledger_query`:  check that ledger query is sent correctly and received
* `staged_ledger_aux_and_pending_coinbases_at_block`: fails with `attempt to subtract with overflow` in yamux
* `block`: fails with `attempt to subtract with overflow` in yamux

### [Kademlia](../../p2p/tests/kademlia.rs)

* `kademlia_routing_table`: tests that node receives peers using kademlia
* `kademlia_incoming_routing_table`: test that kademlia is updated with incoming peer
* `bootstrap_no_peers`: test that kademlia bootstrap finished event if no peers are passed
* `discovery_seed_single_peer`: test nodes discovery over kademlia
* `discovery_seed_multiple_peers`: test node discovery and identify integration
* `test_bad_node`: test that if node gives us invalid peers we handle it

### [Identify](../../p2p/tests/identify.rs)

* `rust_node_to_rust_node`: test if rust node can identify another rust node

### [Connection](../../p2p/tests/connection.rs)

* `rust_to_rust`: test if rust node can connect to rust node
* `rust_to_libp2p`: test if out node can connect to rust libp2p
* `libp2p_to_rust`: test if libp2p node can connect to rust node
* `mutual_rust_to_rust`: test if one rust node can connect to  second rust node, while second node is trying to connect to first one
* `mutual_rust_to_rust_many`: test that many rust nodes can connect to each other at the same time
* `mutual_rust_to_libp2p`: test if rust node can connect to libp2p node, while libp2p node is trying to connect to rust node
* `mutual_rust_to_libp2p_port_reuse`: test that rust node can resolve mutual connection between itself and libp2p node, currently failing due to [Issue #399](https://github.com/openmina/openmina/issues/399)

## Scenarios

### [Connection Discovery](../../node/testing/src/scenarios/multi_node/connection_discovery.rs)

We want to test whether the Rust node can connect and discover peers from Ocaml node, and vice versa

* `RustToOCaml`:
This test ensures that after the Rust node connects to an OCaml node with a known address, it adds its address to its Kademlia state. It also checks that the OCaml node has a peer with the correct peer_id and port corresponding to the Rust node.

* `OCamlToRust`:
This test ensures that after an OCaml node connects to the Rust node, its address becomes available in the Rust node’s Kademlia state. It also checks whether the OCaml node has a peer with the correct `peer_id` and a port corresponding to the Rust node.

* `RustToOCamlViaSeed`:
This test ensures that the Rust node can connect to an OCaml peer, the address of whom can only be discovered from an OCaml seed node, and that the Rust node adds its address to its Kademlia state. It also checks whether the OCaml node has a peer with the correct `peer_id` and port corresponding to the Rust node. Initially, the OCaml seed node has the other two nodes in its peer list, while the OCaml node and the Rust node only have the seed node. The two (OCaml and Rust) non-seed nodes connect to the OCaml seed node. Once connected, they gain information about each other from the seed node. They then make a connection between themselves. If the test is successful, then at the end of this process, each node has each other in its peer list.

* `OCamlToRustViaSeed`: This test ensures that an OCaml node can connect to the Rust node, the address of which can only be discovered from an OCaml seed node, and its address becomes available in the Rust node’s Kademlia state. It also checks whether the OCaml node has a peer with the correct `peer_id` and a port corresponding to the Rust node.

* `RustNodeAsSeed`: This test ensures that the Rust node can work as a seed node by running two OCaml nodes that only know about the Rust node’s address. After these nodes connect to the Rust node, the test makes sure that they also have each other’s addresses as their peers.

### [P2P Connections](../../node/testing/tests/p2p_basic_connections.rs)

* `SimultaneousConnections`:
Tests if two nodes are connecting to each other at the same time, they should be
connected, so each one has exactly one connection.

* `AllNodesConnectionsAreSymmetric`
Connections between all peers are symmetric, i.e. if the node1 has the node2 among its active peers, then the node2 should have the node1 as its active peers.

* `SeedConnectionsAreSymmetric`
Connections with other peers are symmetric for seed node, i.e. if a node is the seed's peer, then it has the node among its peers.

* `MaxNumberOfPeersIncoming`:
Test that Rust node's incoming connections are limited.

* `MaxNumberOfPeersIs1`
Two nodes with max peers = 1 can connect to each other.

### [P2P Kademlia](../../node/testing/tests/p2p_kad.rs)

Test related to kademlia layer.

* `KademliaBootstrap`:
Test that node discovers peers another rust node and is able to bootstrap

### [P2P Pubsub](../../node/testing/tests/p2p_pubsub.rs)

Tests related to pubsub layer.

* `P2pReceiveBlock`
Test that node receives block over meshsub from node

### [P2P Incoming](../../node/testing/tests/p2p_basic_incoming.rs)

Tests related to handling incoming connections.

* `AcceptIncomingConnection`: Node should accept incoming connections.
* `AcceptMultipleIncomingConnections`: Node should accept multiple incoming connections.

### [P2P Outgoing](../../node/testing/tests/p2p_basic_outgoing.rs)

Tests related to outgoing connections

* `MakeOutgoingConnection`: Node should be able to make an outgoing connection to a listening node.

* `MakeMultipleOutgoingConnections`: Node should be able to create multiple outgoing connections.

* `DontConnectToNodeWithSameId`: Node shouldn't establish connection with a node with the same peer_id.

* `DontConnectToInitialPeerWithSameId`: Node shouldn't connect to a node with the same peer id even if its address specified in initial peers.

* `DontConnectToSelfInitialPeer`: Node shouldn't connect to itself even if its address specified in initial peers.

* `ConnectToInitialPeers`: Node should be able to connect to all initial peers.

* `ConnectToUnavailableInitialPeers`: Node should repeat connecting to unavailable initial peer.

* `ConnectToInitialPeersBecomeReady`: Node should be able to connect to all initial peers after they become ready.

### [Single Node](../../node/testing/tests/single_node.rs):

We want to test whether the Rust node is compatible with the OCaml node. We achieve this by attempting to connect the Openmina node to the existing OCaml testnet.

For that purpose, we are utilizing a _solo node_, which is a single Open Mina node connected to a network of OCaml nodes. Currently, we are using the public testnet, but later on we want to use our own network of OCaml nodes on our cluster.

* `SoloNodeBasicConnectivityAcceptIncoming`: Local test to ensure that the Openmina node can accept a connection from an existing OCaml node.

* `SoloNodeBasicConnectivityInitialJoining`: Local test to ensure that the Openmina node can connect to an existing OCaml testnet.

* `SoloNodeSyncRootSnarkedLedger`: Set up single Rust node and sync up root snarked ledger.

* `SoloNodeBootstrap`: Set up single Rust node and bootstrap snarked ledger, bootstrap ledger and blocks.


### [Multi Node](../../node/testing/tests/multi_node.rs):

We also want to test a scenario in which the network consists only of Openmina nodes. If the Openmina node is using a functionality that is implemented only in the OCaml node, and it does not perform it correctly, then we will not be able to see it with solo node test. For that purpose, we utilize a Multi node test, which involves a network of our nodes, without any third party, so that the testing is completely local and under our control.

* `MultiNodeBasicConnectivityPeerDiscovery`: Tests that our node is able to discovery Ocaml nodes through Ocaml seed node.

* `MultiNodeBasicConnectivityInitialJoining`: Tests that node maintains number of peers between minimum and maximum allowed peers.

### [Record/Replay](../../node/testing/tests/record_replay.rs)

* `RecordReplayBootstrap`: Bootstrap a rust node while recorder of state and input actions is enabled and make sure we can successfully replay it.

* `RecordReplayBlockProduction`: Makes sure we can successfully record and replay multiple nodes in the cluster + block production.
