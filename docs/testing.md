# Testing

## Table of contents

- [Introduction](#introduction)
- [Status](#status)
- [K8s Cluster Usage for Testing](#k8s-cluster-usage-for-testing)
    - [Daily Runs Namespace](#daily-runs-namespace)
- [CI test](#ci-tests)
    - [Bootstrap](#bootstrap)
    - [Ledger](#ledger)
    - [P2P tests](#p2p-tests)
    - [Scenario tests](#scenario-tests)
- [Daily bootstrap](#daily-bootstrap):
    - [Bootstrapping to the devnet](#bootstrapping-on-the-devnet)
- [P2P tests](#p2p-tests-1)
    - [RPC](#rpc)
    - [Kademlia](#kademlia)
    - [Identify](#identify)
    - [Connection](#connection)
- [Scenarios](#scenarios)
    - [P2P tests only](#p2p-tests-only)
    - [LibP2P rust to rust(webrtc)](#libp2p-rust-to-rustwebrtc)
    - [Connection Discovery](#connection-discovery)
    - [P2P Connections](#p2p-connections)
    - [Kademlia](#p2p-kademlia)
    - [Pubsub](#p2p-pubsub)
    - [P2P Incoming](#p2p-incoming)
    - [P2p Outgoing](#p2p-outgoing)
    - [Single Node](#single-node)
    - [Multi Node](#multi-node)
    - [Record/Reply](#recordreplay)

## Introduction

Complex systems that handle important information such as blockchain networks must be thoroughly and continuously tested to ensure the highest degree of security, stability, and performance.

To achieve that, we need to develop a comprehensive testing framework capable of deploying a variety of tests.

Such a framework plays a pivotal role in assessing a blockchain's resistance to various malicious attacks. By simulating these attack scenarios and vulnerabilities, the framework helps identify weaknesses in the blockchain's security measures, enabling developers to fortify the system's defenses. This proactive approach is essential to maintain trust and integrity within the blockchain ecosystem, as it minimizes the risk of breaches and instills confidence in users and stakeholders.

Secondly, a robust testing framework is equally crucial for evaluating the blockchain's scalability, speed, and stability. As blockchain networks grow in size and adoption, they must accommodate an increasing number of transactions and users while maintaining a high level of performance and stability. Scalability tests ensure that the system can handle greater workloads without degradation in speed or reliability, helping to avoid bottlenecks and congestion that can hinder transactions and overall network performance.

Additionally, stability testing assesses the blockchain's ability to operate consistently under various conditions, even amid a protocol upgrade. We want to identify potential issues or crashes that could disrupt operations before they have a chance of occurring on the mainnet.

## Status:

- [ ] Bootstrap
    - [x] Node can bootstrap to devnet, [Test](#daily-bootstrap)
    - [ ] Node can bootstrap to mainnet
    - [ ] Node can bootstrap from rust node
    - [ ] Ocaml node can bootstrap from rust node
    - [x] Bootstrap using record/replay
    - [ ] Bootstrap from replayer tool
- [ ] p2p layer
    - [ ] p2p messages:
        - [ ] Binprot types (de)serialization testing/fuzzing
        - [ ] Mina RPC types testing (ideally along with OCaml codecs)
        - [ ] hashing testing (ideally along with OCaml hash implementations)
    - [ ] Connection
        - [ ] Proper initial peers handling, like reconnecting if offline
        - [ ] Peers number maintaining, including edge cases, when we have max peers but still allow peers to connect for e.g. discovery, that is dropping connection strategy
        - [ ] Other connection constraints, like no duplicate connections to the same peer, peer_id
        - [ ] Connection quality metrics
        - [ ] Unable to connect to node with same peer ID, [Issue #402](https://github.com/openmina/openmina/issues/402), [Test](node/testing/src/scenarios/p2p/basic_outgoing_connections.rs?plain=1#L129)
    - [ ] Yamux
    - [ ] Kademlia
        - [ ] Peers discovery, according to Kademlia parameters (a new node gets 20 new peers)
        - [ ] Kademlia routing table is up-to-date with the network (each peer status, like connected/disconnected/can_connect/cant_connect, reflects actual peer state)
        - [ ] Big network test
        - [ ] Adaptive peer management
    - [ ] Identify
    - [ ] Gossipsub
        - [ ] Reachability (all nodes get the message)
        - [ ] Non-redundancy (minimal number of duplicating/unneeded messages)
        - [ ] Interoperability with Ocaml nodes
- [ ] Public network tests. This should be the only set of tests that involve publicly  available networks, and should be executed if we're sure we don't ruin them.
- [ ] Attack resistance testing
    - [ ] Eclipse attacks
    - [ ] Sybil attacks
    - [ ] DDOS attacks
    - [ ] Censorship attacks
- [ ] VRF Evaluator
    - [ ] Correctness test - Selecting the correct ledgers
        - [ ] (Edge case) In genesis epoch
        - [ ] In other (higher) epochs
    - [x] Correctness test - Computation output comparison with mina cli
    - [x] Correctness test - Start a new VRF evaluation on epoch switch for the next available epoch
    - [ ] Correctness test - Retaining the slot data only for future blocks
- [ ] Block Producing, [Issue #394](https://github.com/openmina/openmina/issues/394), [Issue #417](https://github.com/openmina/openmina/issues/417)
    - [x] Make sure rust produced genesis block matches one produced by the ocaml node.
    - [x] Blocks after the genesis block is accepted by the ocaml node.
    - [ ] Empty blocks
        - [x] At genesis
        - [x] After genesis
        - [ ] At epoch boundaries
        - [x] With delegator as slot winner
        - [ ] With producer account as slot winner
        - [ ] With locked token account as slot winner
    - [ ] Blocks with transactions
        - [ ] After genesis
        - [ ] At epoch boundaries
        - [ ] With payment transactions
        - [ ] With zkapp transactions
- [ ] Scalability and upgradeability
    - [ ] Network Scalability, the number of participants or the rate of transactions increases, the network should still maintain its liveness properties.
    - [ ] Upgradeability, the network should be able to upgrade or change protocols without halting or fragmenting, ensuring continuous operation

## K8s Cluster Usage for Testing

### Daily Runs Namespace

The namespace `test-openmina-daily` is used. It has a service account
`github-tester` with `edit` role that allows it to have full control over the
namespace's resources. This account is used by GitHub actions that run daily
tests.

## [CI Tests](.github/workflows/ci.yaml)

### [Bootstrap](.github/workflows/ci.yaml?plain=1#L323)

Bootstrap test can be found in [`bootstrap.rs`](../cli/tests/bootstrap.rs), it checks if node is health and ready.

**_NOTE:_** Bootstrap is also ran daily in CI, see [here](#daily-bootstrap)

### [Ledger](.github/workflows/ci.yaml?plain=1#L14):

In order to run ledger tests circuits are needed, repo for them is [here](https://github.com/openmina/circuit-blobs.git)

```sh
git clone --depth 1 https://github.com/openmina/circuit-blobs.git
ln -s -b $PWD/circuit-blobs/* ledger/
cd ledger
cargo build --release --tests
cargo test --release
```

### [P2p Tests](.github/workflows/ci.yaml?plain=1#L43)

```sh
cargo test -p p2p --tests
```

### Scenario tests

Scenario tests are split into [`scenario-tests`](.github/workflows/ci.yaml?plain=1#L250) and [`p2p-scenario-tests`](.github/workflows/ci.yaml?plain=1#L200).

Currently only tests in [`single_node.rs`](../node/testing/tests/single_node.rs) and [`multi_node.rs`](../node/testing/tests/multi_node.rs) are ran in `scenario-tests` job. For `p2p-scenario-tests` tests in [`p2p_basic_connections.rs`](../node/testing/tests/p2p_basic_connections.rs), [`p2p_basic_incoming.rs`](../node/testing/tests/p2p_basic_incoming.rs), [`p2p_basic_outgoing.rs`](../node/testing/tests/p2p_basic_outgoing.rs) and [`p2p_pubsub.rs`](../node/testing/tests/p2p_pubsub.rs) and ran. Tests found in [`record_replay.rs`](../node/testing/tests/record_replay.rs) are also ran under [`record-replay-tests`](../.github/workflows/ci.yaml?plain=1#L290) job.

## Daily Bootstrap

### Bootstrapping on the Devnet

This test is focused on ensuring that the latest Openmina build is able to bootstrap against Devnet. It is executed on a daily basis.

The node's HTTP port is accessible at http://1.k8.openmina.com:31001.

These are the main steps and checks:

First, it performs some checks on the instance deployed previously:
- Node is in sync state
- Node's best tip is the one that of Devnet

Then it deploys the new instance of Openmina and waits until it is bootstrapped
(with a timeout of 10 minutes). After that. it performs the following checks:

- The node's best tip is the same as in Devnet
- There were no restarts for the openmina container

See the [Openmina Daily](../.github/workflows/daily.yaml) workflow file for
further details.

## P2p tests

Tests for p2p layer.

### [RPC](../p2p/tests/rpc.rs)

* `rust_to_rust`: tests that rust node can receive and send response to and from another rust node 
* `rust_to_many_rust_query`: tests that rust node can respond to many rust peers
* `rust_to_many_rust`: test that rust node can send request to many rust peers
* rpc tests, these tests check if node can correctly communicate over rpc:
    * `initial_peers`
    * `best_tip_with_proof`
    * `ledger_query`
    * `staged_ledger_aux_and_pending_coinbases_at_block`: fails with `attempt to subtract with overflow` in yamux
    * `block`: fails with `attempt to subtract with overflow` in yamux

### [Kademlia](../p2p/tests/kademlia.rs)

* `kademlia_routing_table`: tests that node receives peers using kademlia
* `kademlia_incoming_routing_table`: test that kademlia is updated with incoming peer
* `bootstrap_no_peers`: test that kademlia bootstrap finished event if no peers are passed
* `discovery_seed_single_peer`: test nodes discovery over kademlia
* `discovery_seed_multiple_peers`: test node discovery and identify integration

### [Identify](../p2p/tests/identify.rs)

* `rust_node_to_rust_node`: test if rust node can identify another rust node

### [Connection](../p2p/tests/connection.rs)

* `rust_to_rust`: test if rust node can connect to rust node
* `rust_to_libp2p`: test if out node can connect to rust libp2p
* `libp2p_to_rust`: test if libp2p node can connect to rust node
* `mutual_rust_to_rust`: test if one rust node can connect to  second rust node, while second node is trying to connect to first one
* `mutual_rust_to_rust_many`: test that many rust nodes can connect to each other at the same time
* `mutual_rust_to_libp2p`: test if rust node can connect to libp2p node, while libp2p node is trying to connect to rust node
* `mutual_rust_to_libp2p_port_reuse`: test that rust node can resolve mutual connection between itself and libp2p node, currently failing due to [Issue #399](https://github.com/openmina/openmina/issues/399)

## Scenarios

Scenario test are found in [`node/testing/src/scenarios`](./node/testing/src/scenarios) and they are added as test in [`node/testing/tests`](../node/testing/tests) using `scenario_test` macro. Checked tests are ran in ci. In order to run some scenario tests locally, mina executable or docker is needed, to spawn ocaml node.

### [P2P tests only](../node/testing/tests/node_libp2p_only.rs)

### [LibP2P rust to rust(webrtc)](../node/testing/tests/node_libp2p_with_rust_to_rust_webrtc.rs)

### [Connection Discovery](../node/testing/tests/connection_discovery.rs)

We want to test whether the Rust node can connect and discovery peers from Ocaml node, and vice versa

#### [`RustToOCaml`](../node/testing/src/scenarios/multi_node/connection_discovery.rs?plain=1#L184)

This test ensures that after the Rust node connects to an OCaml node with a known address, it adds its address to its Kademlia state. It also checks that the OCaml node has a peer with the correct peer_id and port corresponding to the Rust node.

#### [`OCamlToRust`](../node/testing/src/scenarios/multi_node/connection_discovery.rs?plain=1#L127)

This test ensures that after an OCaml node connects to the Rust node, its address becomes available in the Rust node’s Kademlia state. It also checks whether the OCaml node has a peer with the correct `peer_id` and a port corresponding to the Rust node.

#### [`RustToOCamlViaSeed`](../node/testing/src/scenarios/multi_node/connection_discovery.rs?plain=1#L378)

This test ensures that the Rust node can connect to an OCaml peer, the address of whom can only be discovered from an OCaml seed node, and that the Rust node adds its address to its Kademlia state. It also checks whether the OCaml node has a peer with the correct `peer_id` and port corresponding to the Rust node.

Initially, the OCaml seed node has the other two nodes in its peer list, while the OCaml node and the Rust node only have the seed node.

![Initial state](https://github.com/openmina/openmina/assets/60480123/bb2c8428-7e89-4748-949a-4b8aa5954205)

The two (OCaml and Rust) non-seed nodes connect to the OCaml seed node

![Non seed nodes connect to seed node](https://github.com/openmina/openmina/assets/60480123/480ffeb0-e7c7-4f16-bed3-76281a19e2bf)

Once connected, they gain information about each other from the seed node. They then make a connection between themselves. If the test is successful, then at the end of this process, each node has each other in its peer list.

![Final state](https://github.com/openmina/openmina/assets/60480123/3ee75cd4-68cf-453c-aa7d-40c09b11d83b)


#### [`OCamlToRustViaSeed`](../node/testing/src/scenarios/multi_node/connection_discovery.rs?plain=1#L267)

This test ensures that an OCaml node can connect to the Rust node, the address of which can only be discovered from an OCaml seed node, and its address becomes available in the Rust node’s Kademlia state. It also checks whether the OCaml node has a peer with the correct `peer_id` and a port corresponding to the Rust node.

#### [`RustNodeAsSeed`](../node/testing/src/scenarios/multi_node/connection_discovery.rs?plain=1#L24)

This test ensures that the Rust node can work as a seed node by running two OCaml nodes that only know about the Rust node’s address. After these nodes connect to the Rust node, the test makes sure that they also have each other’s addresses as their peers.

### [P2P Connections`](../node/testing/tests/p2p_basic_connections.rs)

#### [`SimultaneousConnections`](../node/testing/src/scenarios/p2p/basic_connection_handling.rs?plain=1#L23)

Tests if two nodes are connecting to each other at the same time, they should be
connected, so each one has exactly one connection.

#### [`AllNodesConnectionsAreSymmetric`](../node/testing/src/scenarios/p2p/basic_connection_handling.rs?plain=1#L88)

Connections between all peers are symmetric, i.e. if the node1 has the node2 among its active peers, then the node2 should have the node1 as its active peers.

#### [`SeedConnectionsAreSymmetric`](../node/testing/src/scenarios/p2p/basic_connection_handling.rs?plain=1#L151)

Connections with other peers are symmetric for seed node, i.e. if a node is the seed's peer, then it has the node among its peers.

#### [`MaxNumberOfPeersIncoming`](../node/testing/src/scenarios/p2p/basic_connection_handling.rs?plain=1#L196)

A Rust node's incoming connections should be limited.

#### [`MaxNumberOfPeersIs1`](../node/testing/src/scenarios/p2p/basic_connection_handling.rs?plain=1#L275)

Two nodes with max peers = 1 can connect to each other.

### [P2P Kademlia](../node/testing/tests/p2p_kad.rs)

Test related to kademlia layer, and it's handling of scenarios. 

TODO: These tests need to be updated/replaced

#### [`IncomingFindNode`](../node/testing/src/scenarios/p2p/kademlia.rs?plain=1#L27)
#### [`KademliaBootstrap`](../node/testing/src/scenarios/p2p/kademlia.rs?plain=1#L119)

### [P2P Pubsub](../node/testing/tests/p2p_pubsub.rs)

Tests related to pubsub layer.

#### [`P2pReceiveBlock`](../node/testing/src/scenarios/p2p/pubsub.rs)

Test that node receives block over meshsub from node

### [P2P Incoming](../node/testing/tests/p2p_basic_incoming.rs)

Tests related to handling incoming connections.

#### [`AcceptIncomingConnection`](../node/testing/src/scenarios/p2p/basic_incoming_connections.rs?plain=1#L13)

Node should accept incoming connections.

#### [`AcceptMultipleIncomingConnections`](../node/testing/src/scenarios/p2p/basic_incoming_connections.rs?plain=1#L66)

Node should accept multiple incoming connections.

### [P2P Outgoing](../node/testing/tests/p2p_basic_outgoing.rs)

Tests related to outgoing connections

#### [`MakeOutgoingConnection`](../node/testing/src/scenarios/p2p/basic_outgoing_connections.rs?plain=1#L34)

Node should be able to make an outgoing connection to a listening node.

#### [`MakeMultipleOutgoingConnections`](../node/testing/src/scenarios/p2p/basic_outgoing_connections.rs?plain=1#L76)

Node should be able to create multiple outgoing connections.

#### [`DontConnectToNodeWithSameId`](../node/testing/src/scenarios/p2p/basic_outgoing_connections.rs?plain=1#L129)

Node shouldn't establish connection with a node with the same peer_id.

#### [`DontConnectToInitialPeerWithSameId`](../node/testing/src/scenarios/p2p/basic_outgoing_connections.rs?plain=1#L208)

Node shouldn't connect to a node with the same peer id even if its address specified in initial peers.

#### [`DontConnectToSelfInitialPeer`](../node/testing/src/scenarios/p2p/basic_outgoing_connections.rs?plain=1#L170)

Node shouldn't connect to itself even if its address specified in initial peers.

#### [`ConnectToInitialPeers`](../node/testing/src/scenarios/p2p/basic_outgoing_connections.rs?plain=1#L244)

Node should be able to connect to all initial peers.

#### [`ConnectToUnavailableInitialPeers`](../node/testing/src/scenarios/p2p/basic_outgoing_connections.rs?plain=1#L363)

Node should repeat connecting to unavailable initial peer.

#### [`ConnectToInitialPeersBecomeReady`](../node/testing/src/scenarios/p2p/basic_outgoing_connections.rs?plain=1#L301)

Node should be able to connect to all initial peers after they become ready.

### [Single Node](../node/testing/tests/single_node.rs):

We want to test whether the Rust node is compatible with the OCaml node. We achieve this by attempting to connect the Openmina node to the existing OCaml testnet.

For that purpose, we are utilizing a _solo node_, which is a single Open Mina node connected to a network of OCaml nodes. Currently, we are using the public testnet, but later on we want to use our own network of OCaml nodes on our cluster.

#### [`SoloNodeBasicConnectivityAcceptIncoming`](../node/testing/src/scenarios/solo_node/basic_connectivity_accept_incoming.rs)

Local test to ensure that the Openmina node can accept a connection from an existing OCaml node.

#### [`SoloNodeBasicConnectivityInitialJoining`](../node/testing/src/scenarios/solo_node/basic_connectivity_initial_joining.rs)

Local test to ensure that the Openmina node can connect to an existing OCaml testnet.

#### [`SoloNodeSyncRootSnarkedLedger`](../node/testing/src/scenarios/solo_node/sync_root_snarked_ledger.rs)

Set up single Rust node and sync up root snarked ledger.

#### [`SoloNodeBootstrap`](../node/testing/src/scenarios/solo_node/bootstrap.rs)

Commented out until [#506](https://github.com/openmina/openmina/issues/506) is fixed


### [Multi Node](../node/testing/tests/multi_node.rs):

We also want to test a scenario in which the network consists only of Openmina nodes. If the Openmina node is using a functionality that is implemented only in the OCaml node, and it does not perform it correctly, then we will not be able to see it with solo node test. For that purpose, we utilize a Multi node test, which involves a network of our nodes, without any third party, so that the testing is completely local and under our control.

#### [`MultiNodeBasicConnectivityPeerDiscovery`](../node/testing/src/scenarios/multi_node/basic_connectivity_peer_discovery.rs)

Tests that our node is able to discovery Ocaml nodes through Ocaml seed node.

#### [`MultiNodeBasicConnectivityInitialJoining`](../node/testing/src/scenarios/multi_node/basic_connectivity_initial_joining.rs)

Tests that node maintains number of peers between minimum and maximum allowed peers.

### [Record/Replay](../node/testing/tests/record_replay.rs)

#### [`RecordReplayBootstrap`](../node/testing/src/scenarios/record_replay/bootstrap.rs)

Bootstrap a rust node while recorder of state and input actions is enabled and make sure we can successfully replay it.

#### [`RecordReplayBlockProduction`](../node/testing/src/scenarios/record_replay/block_production.rs)

Makes sure we can successfully record and replay multiple nodes in the cluster + block production.
