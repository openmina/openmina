# Current status of the Rust node

* [High Level Functionality Overview](#overview)
* [VRF Evaluator](#vrf-evaluator)
* [Block Producer](#block-producer)
* [Ledger](#ledger)
* [Proofs](#proofs)
* [P2P Implementation (State Machine Version)](#state-machine-p2p)
* [P2P Related Tests](#p2p-tests)
* [Frontend](#frontend)
* [Documentation](#documentation)
* [Experimental State Machine Architecture](#experimental-state-machine-architecture)

## High Level Functionality Overview <a name="overview"></a>

- [x] Consensus logic
- [x] VRF evaluator
- Block production logic 
  - [x] Without transactions and without proof
  - [ ] Full block with proof
  - [ ] Blocks with transactions. - Missing because we don't yet have the transaction pool logic.
- Networking layer
    - [x] P2P layer in general along with serialization/deserialization of all messages
    - RPCs support
        - [x] `Get_some_initial_peers`(this is not used by the OCaml node)
        - [x] `Get_staged_ledger_aux_and_pending_coinbases_at_hash`
        - [x] `Answer_sync_ledger_query`
        - [x] `Get_transition_chain`
        - `Get_transition_knowledge` (I don't think this one is used at all, `Get_transition_chain_proof` is used instead)
        - [x] `Get_transition_chain_proof`
        - [x] `Get_ancestry`
        - `Ban_notify`
        - [x] `Get_best_tip`
        - `Get_node_status`
    - Peer discovery/advertising
        - [x] Peer discovery through kademlia
        - [x] Advertising the node through kademlia so that OCaml nodes can see us
- [ ] Trust system (to punish/ban peers): **not implemented (and no equivalent)**
- Pools
    - [ ] Transaction pool: **not implemented**
        - No pool is maintained, transactions received over the gossip network are not processed or re-broadcasted
    - SNARK pool
        - [x] SNARK Verification
        - [x] Pool is implemented
        - [x] SNARK work production is implemented (through OCaml). Node can complete and broadcast SNARK work.
- [x] Compatible ledger implementation
- [x] Transition frontier
- Bootstrap/Catchup process
    - [x] Ledger synchronization
       - [x] Snarked ledgers (staking and next epoch ledgers + transition frontier root)
         - [x] Handling of peer disconnections, timeouts or cases when the peer doesn't have the data
         - [x] Detecting ledger hash mismatches for the downloaded chunk
         - [x] Handling ledger hash mismatches gracefully, without crashing the node
         - [x] Optimized snarked ledgers synchronization (reusing previous ledgers when constructing the next during (re)synchronization)
       - [x] Staged ledgers (transition frontier root)
         - [x] Handling of peer disconnections, timeouts or cases when the peer doesn't have the data
         - [x] Detection and handling of validation errors
       - [x] Handling of the rpc requests from other nodes to sync them up
    - [x] Moving root of the transition frontier
    - [x] Maintaining ledgers for transition frontier root, staking and next epoch ledgers
      -  [x] When scan state tree gets committed, snarked ledger of the block is updated. When that happens for the root block in the transition frontier, reconstruct the new root snarked ledger
      -  [x] At the end of an epoch make the "next epoch" ledger the new "staking" ledger, discard the old "staking" ledger and make the snarked ledger of the best tip the new "next epoch" ledger
    - [x] Best chain synchronization
      - [x] Download missing blocks from peers
        - [x] Handling of peer disconnections, timeouts or cases when the peer doesn't have the data
        - [x] Downloaded block header integrity validation by checking it's hash and handling the mismatch
        - [ ] Downloaded block body integrity validation by checking it's hash and handling the mismatch
      - [x] Missing blocks application
        - [ ] Graceful handling of block application error without crashing the node
    - [x] Handling of reorgs (short/long range forks) or best chain extension after or even mid-synchronization, by adjusting synchronization target and reusing what we can from the previous synchronization attempt
- Block application
    - [x] Transaction application logic
    - [x] Block application logic
    - Proof verification:
        - [x] Block proof verification
        - [x] Transaction proof verification (same as above)
        - [x] Zkapp proof verification (same as above)
- [ ] Client API (currently the node has a very partial support, not planned at the moment)
- [ ] Archive node (not planned at the moment)

## VRF Evaluator <a name="vrf-evaluator"></a>

- [x] VRF evaluator functionality:
    - [x] Calculation of the VRF output
    - [x] Threshold calculation determining if the slot has been won
    - [ ] (Optional) Providing verification of the producers VRF output (Does not impact the node functionality, just provides a way for the delegates to verify their impact on winning/losing a slot)
- [x] Implement VRF evaluator state machine
  - [x] Computation service
  - [x] Collecting the delegator table for the producer
  - [x] Integrate with the block producer
  - [x] Handling epoch changes - starting new evaluation as soon as new epoch data is available
  - [ ] Retention logic - cleanup slot data that is in the past based on current global slot (Slight node impact - the won slot map grows indefinitely)
- [ ] Testing 
  - [ ] Correctness test - Selecting the correct ledgers
    - [x] (Edge case) In genesis epoch
    - [ ] In other (higher) epochs
  - [x] Correctness test - Computation output comparison with mina cli
  - [x] Correctness test - Start a new VRF evaluation on epoch switch for the next available epoch
  - [ ] Correctness test - Retaining the slot data only for future blocks
- [ ] Documentation 

## Block Producer <a name="block-producer"></a>

- [ ] Block producer
  - [x] Integrate with VRF evaluator
  - [x] Include coinbase transactions
  - [x] Include fee transfers
  - [ ] Include simple transactions (transaction pool missing)
  - [ ] Include zkapp transactions (transaction pool missing)
  - [x] Ledger diff creation
  - [x] Integrate with transition frontier
  - [x] New epoch seed calculation
  - [x] Staking epoch ledger selection 
  - [ ] Proof generation 
- [ ] Testing
- [ ] Documentation

## Ledger <a name="ledger"></a>

- [x] Ledger/Mask implementation 
- [x] Staged Ledger implementation
   - [x] Scan state
   - [x] Pending coinbase collection
   - [x] Transaction application
      - [x] Regular transaction (payment, delegation, coinbase, fee transfer)
      - [x] Zkapps
- [x] Persistent database
   - https://github.com/MinaProtocol/mina/pull/13340
   - Drop-in replacement for RocksDB

## Proofs <a name="proofs"></a>

- [x] Proof verification
   - [x] Block proof
   - [x] Transaction/Merge proof
   - [x] Zkapp proof
- [x] Proof/Witness generation
   - [x] Block proof
   - [x] Transaction/Merge proof
   - [x] Zkapp proof
- [ ] Circuit generation

## P2P Implementation (State Machine Version) <a name="state-machine-p2p"></a>

### Handshake

- [x] Create a service for low level TCP networking (mio, epoll).
- [ ] DNS support.
- [x] Pnet protocol.
- [x] Multistream select protocol.
- [ ] Handle simultaneous connect case.
- [x] Noise protocol for outgoing connections.
- [x] Noise protocol for incoming connections.
- [x] Yamux multiplexer.

### Peer management

- [ ] Create connection scheduler to limit work for each peer
- [ ] Handle reconnection and exponential backoff.

### Peer discovery

- [ ] Implement Kademlia algorithm.
  - [x] Implement Kademlia FIND_NODE (client/server).
  - [x] Implement Kademlia Bootstrap process.
  - [ ] Update Kademlia routing table according to Identify protocol messages.
  

### RPC 

- [x] Perform outgoing RPC requests.
- [ ] Handle incoming RPC requests.

### Gossipsub

- [ ] Implement gossipsub compatible with libp2p.
- [ ] Research how to use "expander graph" theory to make gossipsub robust and efficient.

### Testing

- [ ] Fix bootstrap sandbox record/replay for the latest berkeley network.
- [x] Fix network debugger for the latest berkeley network.
- [x] Test that the Openmina node can bootstrap from the replayer tool.
- [ ] Test that the OCaml node can bootstrap from the Openmina node.
- [ ] Test that the Openmina node can bootstrap from another instance of openmina node.

## P2P Related Tests <a name="p2p-tests"></a>

- [ ] P2p functionality tests
  - [ ] p2p messages
      - [ ] Binprot types (de)serialization testing/fuzzing
      - [ ] Mina RPC types testing (ideally along with OCaml codecs)
      - [ ] hashing testing (ideally along with OCaml hash implementations)
  - [ ] Connection
      - [ ] Proper initial peers handling, like reconnecting if offline
      - [ ] Peers number maintaining, including edge cases, when we have max peers but still allow peers to connect for e.g. discovery, that is dropping connection strategy
      - [ ] Other connection constraints, like no duplicate connections to the same peer, peer_id, no self connections etc
      - [ ] Connection quality metrics
  - [ ] Kademlia
      - [ ] Peers discovery, according to Kademlia parameters (a new node gets 20 new peers)
      - [ ] Kademlia routing table is up-to-date with the network (each peer status, like connected/disconnected/can_connect/cant_connect, reflects actual peer state)
  - [ ] Gossipsub
      - [ ] Reacheability (all nodes get the message)
      - [ ] Non-redundancy (minimal number of duplicating/unneeded messages)
- [ ] Interoperability with OCaml node
    - [ ] Bootstrap Rust node from OCaml and vice versa
    - [ ] Discovery using Rust node
    - [ ] Gossipsub relaying
- [ ] Public network tests. This should be the only set of tests that involve publicly  available networks, and should be executed if we're sure we don't ruin them.
- [ ] Attack resistance testing

## Frontend <a name="frontend"></a>

### Pages

- [x] Nodes - Overview 
- [x] Nodes - Live
- [x] Nodes - Bootstrap
- [x] State - Actions
- [x] Snarks - Work Pool
- [x] Snarks - Scan State
- [x] Resources - Memory
- [x] Network - Messages
- [x] Network - Blocks
- [x] Network - Connections
- [ ] Network - Topology
- [ ] Network - Node DHT
- [x] Peers - Dashboard
- [x] Testing Framework - Scenarios

### Testing
- [x] Tests for Nodes Overview
- [x] Tests for Nodes Live
- [ ] Tests for Nodes Bootstrap
- [ ] Tests for State - Actions
- [ ] Tests for Snarks - Work Pool
- [ ] Tests for Snarks - Scan State
- [x] Tests for Resources - Memory
- [x] Tests for Network - Messages
- [x] Tests for Network - Blocks
- [x] Tests for Network - Connections
- [ ] Tests for Network - Topology
- [ ] Tests for Network - Node DHT
- [x] Tests for Peers - Dashboard
- [ ] Tests for Testing Framework - Scenarios

### Other
- [x] CI Integration and Docker build & upload
- [x] State management
- [x] Update to Angular v16
- [x] Ensure performant application (lazy load & standalone components)
- [x] Ensure reusable components/css/BL

## Documentation <a name="documentation"></a>

### By module

- [x] [Openmina Node](https://github.com/openmina/openmina#the-open-mina-node)
- [x] [The Mina Web Node](https://github.com/openmina/webnode/blob/main/README.md)
- P2P
  - [ ] [P2P Networking Stack](https://github.com/openmina/openmina/blob/develop/p2p/readme.md) in progress
  - [x] [P2P services](https://github.com/openmina/openmina/blob/documentation/docs/p2p_service.md)
  - [ ] [RPCs support](https://github.com/JanSlobodnik/pre-publishing/blob/main/RPCs.md) - in progress
  -	[x] [GossipSub](https://github.com/openmina/mina-wiki/blob/3ea9041e52fb2e606918f6c60bd3a32b8652f016/p2p/mina-gossip.md)
- [x] [Scan state](https://github.com/openmina/openmina/blob/main/docs/scan-state.md)
- [x] [SNARKs](https://github.com/openmina/openmina/blob/main/docs/snark-work.md)
- Developer tools
  - [x] [Debugger](https://github.com/openmina/mina-network-debugger/blob/main/README.md)
  - [x] [Front End](https://github.com/openmina/mina-frontend/blob/main/README.md)
    - [x] [Dashboard](https://github.com/openmina/mina-frontend/blob/main/docs/MetricsTracing.md#Dashboard)
    - [x] [Debugger](https://github.com/openmina/mina-network-debugger)


### By use-case

- [x] [Why we are developing Open Mina](https://github.com/openmina/openmina/blob/main/docs/why-openmina.md)
- [ ] Consensus logic - not documented yet
- Block production logic 
  - [ ] [Internal transition](https://github.com/JanSlobodnik/pre-publishing/blob/main/block-production.md) - in progress
  - [ ] External transition - not documented yet
  - [ ] [VRF function](https://github.com/openmina/openmina/blob/feat/block_producer/vrf_evaluator/vrf/README.md) - in progress
- Peer discovery/advertising
  - [ ] [Peer discovery through Kademlia](https://github.com/openmina/openmina/blob/develop/p2p/readme.md#kademlia-for-peer-discovery) - in progress
- [x] [SNARK work](https://github.com/openmina/openmina/blob/main/docs/snark-work.md) - SNARK production is implemented (through OCaml). Node can complete and broadcast SNARK work.
  - [ ] [Witness folding](https://github.com/JanSlobodnik/pre-publishing/blob/main/witness-folding.md) - in progress
- [ ] [Bootstrapping process](https://github.com/JanSlobodnik/pre-publishing/blob/main/bootstrap-catchup.md) - in progress
- [ ] Block application - not documented yet
- Testing
  - [ ] [Testing framework](https://github.com/openmina/openmina/blob/main/docs/testing/testing.md) - partially complete (some tests are documented)
- How to run
  - [x] [Launch Openmina node](https://github.com/openmina/openmina#how-to-launch-without-docker-compose)
  - [x] [Launch Node with UI](https://github.com/openmina/openmina#how-to-launch-with-docker-compose)
  - [x] [Launch Debugger](https://github.com/openmina/mina-network-debugger?tab=readme-ov-file#Preparing-for-build)
  - [x] [Launch Web Node](https://github.com/openmina/webnode/blob/main/README.md#try-out-the-mina-web-node)

## Experimental State Machine Architecture

### Core state machine

- [x] Automaton implementation that separates *action* kinds in *pure* and *effectful*.
- [x] Callback (dispatch-back) support for action composition: enable us to specify in the action itself the actions that will dispatched next.
- [x] Fully serializable state machine state and actions (including descriptors to callbacks!).
- State machine state management
  - [x] Partitioning of the state machine state between models sub-states (for *pure* models).
  - [x] Forbid direct access to state machine state in *effectful* models.
  - [x] Support for running multiple instances concurrently in the same state machine for testing scenarios: for example if the state machine represents a node, we can "run" multiple of them inside the same state machine.

### Models

Each model handles a subset of actions and they are registered like a plugin system.

#### Effectful

Thin layer of abstraction between the "external world" (IO) and the state machine.

- [x] MIO model: provides the abstraction layer for the polling and TCP APIs of the MIO crate.
- [x] Time model: provides the abstraction layer for `SystemTime::now()`

#### Pure

Handle state transitions and can dispatch actions to other models.

- [x] Time model: this is the *pure* counterpart which dispatches an action to *effectful* time model to get the system time and updates the internal time in the state machine state.
- [x] TCP model: built on top of the MIO layer to provide all necessary features for handling TCP connections (it also uses the time model to provide timeout support for all actions).
- [x] TCP-client model: built on top of the TCP model, provides a high-level interface for building client applications.
- [x] TCP-server model: built on top of the TCP model, provides a high-level interface for building server applications.
- [x] PRNG model: unsafe, fast, pure RNG for testing purposes.
- PNET models: implements the private network transport used in libp2p.
   - [x] Server
   - [x] Client
 - Testing models:
   - [x] Echo client: connects to an echo server and sends random data, then checks that it receives the same data.
   - [x] Echo server.
   - [x] Echo client (PNET).
   - [x] Echo server (PNET).
   - [x] Simple PNET client: connects to berkeleynet and does a simple multistream negotiation.
 
### Tests

- Echo network
  - [x] State machine with a network composed of 1 client and 1 server instance.
  - [x] State machine with a network composed of 5 clients and 1  erver instance.
  - [x] State machine with a network composed of 50 clients and 1  erver instance.
- [x] Echo network PNET (same tests as echo network but over the PNET transport).
- [x] Berkeley PNET test: runs the simple PNET client model. 
