# Testing

## Status:

- [ ] Bootstrap
    - [x] Node can bootstrap from ocaml node, [Test](#bootstrap)
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
    - [ ] Identify
    - [ ] Gossipsub
        - [ ] Reachability (all nodes get the message)
        - [ ] Non-redundancy (minimal number of duplicating/unneeded messages)
        - [ ] Interoperability with Ocaml nodes
- [ ] Public network tests. This should be the only set of tests that involve publicly  available networks, and should be executed if we're sure we don't ruin them.
- [ ] Attack resistance testing
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

## [CI Tests](.github/workflows/ci.yaml)

### [Bootstrap](.github/workflows/ci.yaml?plain=1#L323)

Bootstrap can be found in [`bootstrap.rs`](cli/tests/bootstrap.rs), it checks if node is health and ready.

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

Currently only tests in [`single_node.rs`](node/testing/tests/single_node.rs) and [`multi_node.rs`](node/testing/tests/multi_node.rs) are ran in `scenario-tests` job. For `p2p-scenario-tests` tests in [`p2p_basic_connections.rs`](node/testing/tests/p2p_basic_connections.rs), [`p2p_basic_incoming.rs`](node/testing/tests/p2p_basic_incoming.rs), [`p2p_basic_outgoing.rs`](node/testing/tests/p2p_basic_outgoing.rs) and [`p2p_pubsub.rs`](node/testing/tests/p2p_pubsub.rs) and ran. Tests found in [`record_replay.rs`](node/testing/tests/record_replay.rs) are also ran under [`record-replay-tests`](.github/workflows/ci.yaml?plain=1#L290) job.

## P2p tests

Tests for p2p layer.

### [RPC](p2p/tests/rpc.rs)

* `rust_to_rust`: tests that rust node can receive and send response to and from another rust node 
* `rust_to_many_rust_query`: tests that rust node can respond to many rust peers
* `rust_to_many_rust`: test that rust node can send request to many rust peers
* rpc tests, these tests check if node can correctly communicate over rpc:
    * `initial_peers`
    * `best_tip_with_proof`
    * `ledger_query`
    * `staged_ledger_aux_and_pending_coinbases_at_block`: fails with `attempt to subtract with overflow` in yamux
    * `block`: fails with `attempt to subtract with overflow` in yamux

### [Kademlia](p2p/tests/kademlia.rs)

* `kademlia_routing_table`: tests that node receives peers using kademlia
* `kademlia_incoming_routing_table`: test that kademlia is updated with incoming peer
* `bootstrap_no_peers`: test that kademlia bootstrap finished event if no peers are passed
* `discovery_seed_single_peer`: test nodes discovery over kademlia
* `discovery_seed_multiple_peers`: test node discovery and identify integration

### [Identify](p2p/tests/identify.rs)

* `rust_node_to_rust_node`: test if rust node can identify another rust node

### [Connection](p2p/tests/connection.rs)

* `rust_to_rust`: test if rust node can connect to rust node
* `rust_to_libp2p`: test if out node can connect to rust libp2p
* `libp2p_to_rust`: test if libp2p node can connect to rust node
* `mutual_rust_to_rust`: test if one rust node can connect to  second rust node, while second node is trying to connect to first one
* `mutual_rust_to_rust_many`: test that many rust nodes can connect to each other at the same time
* `mutual_rust_to_libp2p`: test if rust node can connect to libp2p node, while libp2p node is trying to connect to rust node
* `mutual_rust_to_libp2p_port_reuse`: test that rust node can resolve mutual connection between itself and libp2p node, currently failing due to [Issue #399](https://github.com/openmina/openmina/issues/399)

## Scenarios

Scenario test are found in [`node/testing/src/scenarios`](./node/testing/src/scenarios) and they are added as test in [`node/testing/tests`](./node/testing/tests) using `scenario_test` macro. Checked tests are ran in ci. In order to run some scenario tests locally mina executable or docker is needed, to spawn ocaml node.

- [ ] [`node_libp2p_only`](./node/testing/tests/node_libp2p_only.rs)
- [x] [`p2p_basic_connections`](./node/testing/tests/p2p_basic_connections.rs):
    - [`SimultaneousConnections`](./node/testing/src/scenarios/p2p/basic_connection_handling.rs?plain=1#L23)
    - [`AllNodesConnectionsAreSymmetric`](./node/testing/src/scenarios/p2p/basic_connection_handling.rs?plain=1#L88)
    - [`SeedConnectionsAreSymmetric`](./node/testing/src/scenarios/p2p/basic_connection_handling.rs?plain=1#L151)
    - [`MaxNumberOfPeersIncoming`](./node/testing/src/scenarios/p2p/basic_connection_handling.rs?plain=1#L196)
    - [`MaxNumberOfPeersIs1`](./node/testing/src/scenarios/p2p/basic_connection_handling.rs?plain=1#L275)
- [ ] [`p2p_kad`](./node/testing/tests/p2p_kad.rs):
    - [`IncomingFindNode`](./node/testing/src/scenarios/p2p/kademlia.rs?plain=1#L27)
    - [`KademliaBootstrap`](./node/testing/src/scenarios/p2p/kademlia.rs?plain=1#L119)
- [x] [`p2p_basic_incoming`](./node/testing/tests/p2p_basic_incoming.rs)
    - [`AcceptIncomingConnection`](./node/testing/src/scenarios/p2p/basic_incoming_connections.rs?plain=1#L13)
    - [`AcceptMultipleIncomingConnections`](./node/testing/src/scenarios/p2p/basic_incoming_connections.rs?plain=1#L66)
- [x] [`p2p_pubsub`](./node/testing/tests/p2p_pubsub.rs):
    - [`P2pReceiveBlock`](./node/testing/src/scenarios/p2p/pubsub.rs)
- [ ] [`record_replay`](./node/testing/tests/record_replay.rs):
    - [`RecordReplayBootstrap`](./node/testing/src/scenarios/record_replay/bootstrap.rs)
    - [`RecordReplayBlockProduction`](./node/testing/src/scenarios/record_replay/block_production.rs)
- [x] [`p2p_basic_outgoing`](./node/testing/tests/p2p_basic_outgoing.rs):
    - [`MakeOutgoingConnection`](./node/testing/src/scenarios/p2p/basic_outgoing_connections.rs?plain=1#L34)
    - [`MakeMultipleOutgoingConnections`](./node/testing/src/scenarios/p2p/basic_outgoing_connections.rs?plain=1#L76)
    - [`DontConnectToNodeWithSameId`](./node/testing/src/scenarios/p2p/basic_outgoing_connections.rs?plain=1#L129)
    - [`DontConnectToInitialPeerWithSameId`](./node/testing/src/scenarios/p2p/basic_outgoing_connections.rs?plain=1#L208)
    - [`DontConnectToSelfInitialPeer`](./node/testing/src/scenarios/p2p/basic_outgoing_connections.rs?plain=1#L170)
    - [`ConnectToInitialPeers`](./node/testing/src/scenarios/p2p/basic_outgoing_connections.rs?plain=1#L244)
    - [`ConnectToUnavailableInitialPeers`](./node/testing/src/scenarios/p2p/basic_outgoing_connections.rs?plain=1#L363)
    - [`ConnectToInitialPeersBecomeReady`](./node/testing/src/scenarios/p2p/basic_outgoing_connections.rs?plain=1#L301)
- [ ] [`node_libp2p_with_rust_to_rust_webrtc`](./node/testing/tests/node_libp2p_with_rust_to_rust_webrtc.rs)
- [x] [`multi_node`](./node/testing/tests/multi_node.rs):
    - [`MultiNodeBasicConnectivityPeerDiscovery`](./node/testing/src/scenarios/multi_node/basic_connectivity_peer_discovery.rs)
    - [`MultiNodeBasicConnectivityInitialJoining`](./node/testing/src/scenarios/multi_node/basic_connectivity_initial_joining.rs)
- [x] [`single_node`](./node/testing/tests/single_node.rs):
    - [`SoloNodeBasicConnectivityAcceptIncoming`](./node/testing/src/scenarios/solo_node/basic_connectivity_accept_incoming.rs)
    - [`SoloNodeBasicConnectivityInitialJoining`](./node/testing/src/scenarios/solo_node/basic_connectivity_initial_joining.rs), ignored
    - [`SoloNodeSyncRootSnarkedLedger`](./node/testing/src/scenarios/solo_node/sync_root_snarked_ledger.rs), ignored
    - [`SoloNodeBootstrap`](./node/testing/src/scenarios/solo_node/bootstrap.rs), commented out until [#506](https://github.com/openmina/openmina/issues/506) is fixed
- [ ] [`connection_discovery`](./node/testing/tests/connection_discovery.rs):
    - [`RustToOCaml`](./node/testing/src/scenarios/multi_node/connection_discovery.rs?plain=1#L169)
    - [`OCamlToRust`](./node/testing/src/scenarios/multi_node/connection_discovery.rs?plain=1#L117)
    - [`RustToOCamlViaSeed`](./node/testing/src/scenarios/multi_node/connection_discovery.rs?plain=1#L351)
    - [`OCamlToRustViaSeed`](./node/testing/src/scenarios/multi_node/connection_discovery.rs?plain=1#L246)
    - [`RustNodeAsSeed`](./node/testing/src/scenarios/multi_node/connection_discovery.rs?plain=1#L20)