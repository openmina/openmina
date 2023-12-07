
# A Testing Framework for Mina



Complex systems that handle important information such as blockchain networks must be thoroughly and continuously tested to ensure the highest degree of security, stability, and performance. 

To achieve that, we need to develop a comprehensive testing framework capable of deploying a variety of tests. We’ve also created a front end for the testing framework so that other developers can run their own tests to fine-tune the solutions and products they create for the Mina ecosystem. 

Such a framework plays a pivotal role in assessing a blockchain's resistance to various malicious attacks. By simulating these attack scenarios and vulnerabilities, the framework helps identify weaknesses in the blockchain's security measures, enabling developers to fortify the system's defenses. This proactive approach is essential to maintain trust and integrity within the blockchain ecosystem, as it minimizes the risk of breaches and instills confidence in users and stakeholders.

Secondly, a robust testing framework is equally crucial for evaluating the blockchain's scalability, speed, and stability. As blockchain networks grow in size and adoption, they must accommodate an increasing number of transactions and users while maintaining a high level of performance and stability. Scalability tests ensure that the system can handle greater workloads without degradation in speed or reliability, helping to avoid bottlenecks and congestion that can hinder transactions and overall network performance. 

Additionally, stability testing assesses the blockchain's ability to operate consistently under various conditions, even amid a protocol upgrade. We want to identify potential issues or crashes that could disrupt operations before they have a chance of occurring on the mainnet. 


### What we are testing

Here is a limited overview of test categories. Tests are mostly focused on the network and P2P layer,  the next steps will be consensus, ledger, and other parts.  

We need to work with the assumption that more than one-third of the nodes can be Byzantine for the system to function correctly.


## 1. Network Connectivity and Peer Management


### Network Connectivity

Nodes that get disconnected should eventually be able to reconnect and synchronize with the network.

_This test assesses the blockchain node's ability to maintain consistent network connectivity. It evaluates whether a node can gracefully handle temporary disconnections from the network and subsequently reestablish connections._

We want to ensure that new nodes can join the network and handle being overwhelmed with connections or data requests, including various resilience and stability conditions (e.g., handling reconnections, latency, intermittent connections, and dynamic IP handling).

This is crucial for ensuring that no node is permanently isolated and can always participate in the blockchain's consensus process.

We are testing two versions of the node:


#### Solo node

We want to be able to test whether the Rust node is compatible with the OCaml node. We achieve this by attempting to connect the Openmina node to the existing OCaml testnet.

For that purpose, we are utilizing a _solo node_, which is a single Open Mina node connected to a network of OCaml nodes. Currently, we are using the public testnet, but later on we want to use our own network of OCaml nodes on our cluster.

This test is performed by launching an Openmina node and connecting it to seed nodes of the public (or private) OCaml testnet.

_The source code for this test can be found in this repo:_

[https://github.com/openmina/openmina/blob/develop/node/testing/src/scenarios/solo_node/basic_connectivity_initial_joining.rs](https://github.com/openmina/openmina/blob/develop/node/testing/src/scenarios/solo_node/basic_connectivity_initial_joining.rs) 

 

We are testing these scenarios:



1. Whether the Openmina node can accept an incoming connection from OCaml node. This test will prove our Openmina node is listening to incoming connections and can accept them.
2. Whether the OCaml node can discover and connect to an Openmina node that is advertising itself. This is done by advertising the Openmina node so that the OCaml node can discover it and connect to the node. 

    This test is the same as the previous one, except we do not inform the OCaml node to connect to it explicitly, it should find it automatically and connect using peer discovery (performed through Kademlia). This test will ensure the Openmina node uses Kademlia in a way that is compatible with the OCaml node.

    However, with this test, we are currently experiencing problems that may be caused by OCaml nodes not being currently able to  "see" the Openmina nodes, because our implementation of the p2p layer is incomplete. 

    We have implemented the missing protocol (Kademlia) into the p2p layer to make OCaml nodes see our node. Despite being successfully implemented, the main test is not working. One possible reason is that our implementation of Kademlia is slightly incompatible with the OCaml implementation of Kademlia. 

    We are also missing certain p2p protocols like `/mina/peer-exchange`, `/mina/bitswap-exchange`, `/mina/node-status`, `/ipfs/id/1.0.0`

    While these p2p protocol may not be relevant, it is possible OCaml nodes do not recognize the Openmina node because we are missing some of them.


We run these tests until:



* The number of known peers is greater than or equal to the maximum number of peers.
* The number of connected peers is greater than or equal to some threshold.
* The test is failed if the specified number of steps occur but the conditions are not met.

**Kademlia peer discovery**

We want the Open Mina node to be able to connect to peers, both other Open Mina nodes (that are written in Rust) as well as native Mina nodes (written in OCaml). 

Native Mina nodes use Kademlia (KAD), a distributed hash table (DHT) for peer-to-peer computer networks. Hash tables are data structures that map _keys_ to _values_. Think of a hash table as a dictionary, where a word (i.e. dog) is mapped to a definition (furry, four-legged animal that barks). 

In Mina nodes, KAD specifies the structure of the network and the exchange of information through node lookups, which makes it efficient for locating nodes in the network. 

Since we initially focused on other parts of the node, we used the RPC get_initial_peers as a sort-of workaround to connect our nodes between themselves. Now, to ensure compatibility with the native Mina node, we’ve implemented KAD for peer discovery for the Open Mina node. 

**How does Mina utilize Kademlia?**

Kademlia has two main parts - the routing table and the peer store. 



1. The routing table is used to store information about network paths, enabling efficient data packet routing. It maintains peer information (peer id and network addresses)
2. The peer store is a database for storing and retrieving network peer information (peer IDs and their network addresses), forming part of the provider store. Providers are nodes that possess specific data and are willing to share or provide this data to other nodes in the network.

Peers are added to the routing table if they can communicate, support the correct protocol, and send/respond to valid queries. They are removed if unresponsive. Peers are added to the peer store when handling AddProvider messages.

A provider in Kademlia announces possession of specific data (identified by a unique key) and shares it with others. In MINA's case, all providers use the same key, which is the SHA256 hash of a specific string pattern. In MINA, every node acts as a “provider,” making the advertisement as providers redundant. Non-network nodes are filtered at the PNet layer.

If there are no peers, KAD will automatically search for new ones. KAD will also search for new peers whenever the node is restarted. If a connection is already made, it will search for more peers every hour. 

**Message types**



* AddProvider - informs the peer that you can provide the information described by the specified key.
* GetProviders - a query for nodes that have already performed AddProvider.
* FindNode - is used for different purposes. It can find a place in the network where your node should be. Or it may find a node that you need to send an AddProvider (or GetProviders) message to.

**Potential issues identified**

* An earlier issue in the Open Mina (Rust node) with incorrect provider key advertising is now fixed.
* The protocol's use in OCaml nodes might be a potential security risk; an adversary could exploit this to DoS the network. One possible solution is to treat all Kademlia peers as providers.
* The peer can choose its peer_id (not arbitrarily, the peer chooses a secret key, and then the peer_id is derived from the secret key). The peer can repeat this process until its peer_id is close to a key that identifies some desired information. Thus, the peer will be responsible for storing providers of this information.

    The malicious peer can deliberately choose the peer_id and deny access to the information, just always say there are no providers.

    This problem has been inherited from the OCaml implementation of the node. We have mitigated it by making the Openmina node not rely on GetProviders, instead, we only do AddProviders to advertise ourselves, but treat any peer of the Kademlia network as a valid Mina peer, no matter if it is a provider or not, so a malicious peer can prevent OCaml nodes from discovering us, but it will not prevent us from discovering OCaml nodes.

* Kademlia exposes both internal and external addresses, which may be unnecessary in the MINA network and could be a security risk. 
* if we test on any local network that belongs to these ranges because they will get filtered unless we manually disable (in code) these checks.
* There's also an issue with the handling of private IP ranges.
* We need more testing on support for IPv6. libp2p_helper code can't handle IPv6 for the IP range filtering.


#### Multi node

We also want to test a scenario in which the network consists only of Openmina nodes. If the Openmina node is using a functionality that is implemented only in the OCaml node, and it does not perform it correctly, then we will not be able to see it with solo node test. 

For that purpose, we utilize a Multi node test, which involves a network of our nodes, without any third party, so that the testing is completely local and under our control.

_The source code for this test can be found in this repo:_

[https://github.com/openmina/openmina/blob/develop/node/testing/src/scenarios/multi_node/basic_connectivity_initial_joining.rs#L9](https://github.com/openmina/openmina/blob/develop/node/testing/src/scenarios/multi_node/basic_connectivity_initial_joining.rs#L9) 


#### How it's tested

**Node cluster**: We use a `ClusterRunner` utility to manage the setup and execution of test scenarios on a cluster of nodes.

**Scenarios Enumeration**: `Scenarios` is an enum with derived traits to support iterating over the scenarios, converting them to strings, etc. It lists different test scenarios such as `SoloNodeSyncRootSnarkedLedger`, `SoloNodeBasicConnectivityInitialJoining`, and `MultiNodeBasicConnectivityInitialJoining`. 

Each scenario has a related module (e.g., `multi_node::basic_connectivity_initial_joining::MultiNodeBasicConnectivityInitialJoining`) which contains the logic for the test.

**Scenario Implementation**: The `Scenarios` enum has methods for executing tests such as `run`, `run_and_save`, and `run_only`. These methods use the `ClusterRunner` to run the scenarios and potentially save the results.

**Dynamic Scenario Building**: There's logic (`blank_scenario`) to dynamically build a scenario's configuration, potentially from a JSON representation, which then gets executed in a test run.

**Async/Await**: The methods within the `Scenarios` are asynchronous (`async`), indicating that the tests are run in an asynchronous context, which is common when dealing with network operations to allow for non-blocking I/O operations.

**Parent Scenarios**: The `parent` and `parent_id` methods suggest that some scenarios may depend on others. The code constructs a hierarchy of test scenarios, ensuring parent scenarios are run before their children.

**Cluster Configuration and Execution**: `build_cluster_and_run_parents` is an asynchronous method for setting up a cluster according to a specified configuration and running all parent scenarios to prepare the environment for a specific test.


#### Node Discovery Test

The main goal of this test is to ensure that the Rust node can discover peers in the network, and is discoverable by other peers as well.

In this test, three nodes are started:



1. OCaml seed node with known address and peer ID
2. OCaml node with the seed node set as the initial peer
3. Rust node with the seed node set as the initial peer

Initially, the OCaml seed node has the other two nodes in its peer list, while the OCaml node and the Rust node only have the seed node.


![peer1](https://github.com/openmina/openmina/assets/60480123/bb2c8428-7e89-4748-949a-4b8aa5954205)



The two (OCaml and Rust) non-seed nodes connect to the OCaml seed node


![peer2](https://github.com/openmina/openmina/assets/60480123/480ffeb0-e7c7-4f16-bed3-76281a19e2bf)


Once connected, they gain information about each other from the seed node.

They then make a connection between themselves. If the test is successful, then at the end of this process, each each node has each other in its peer list.

![peer3](https://github.com/openmina/openmina/assets/60480123/3ee75cd4-68cf-453c-aa7d-40c09b11d83b)


**Implementation Details**

The Helm chart that is used to deploy the network also contains the script that performs the checks.


#### OCaml Peer Discovery Test

As we develop the Openmina node, a new Rust-based node implementation for the Mina network, we must ensure that the Rust node can utilize the Kademlia protocol (KAD) to discover and connect to OCaml nodes, and vice versa.

For that purpose, we have developed a series of basic tests that check the correct peer discovery via KAD when the Rust node is connected to OCaml peers.


##### OCaml to Rust

This test ensures that after an OCaml node connects to the Rust node, its address \
becomes available in the Rust node’s Kademlia state. It also checks whether the OCaml \
node has a peer with the correct peer_id and a port corresponding to the Rust node.

Steps:



1. Configure and launch a Rust node
2. Start an OCaml node with the Rust node as the only peer
3. Run the Rust node until it receives an event signaling that the OCaml node is connected
4. Wait for an event Identify that is used to identify the remote peer’s address and port
5. Check that the Rust node has an address of the OCaml node in its Kademlia part of the state


##### Rust to OCaml

This test ensures that after the Rust node connects to an OCaml node with a known \
address, it adds its address to its Kademlia state. It also checks that the OCaml \
node has a peer with the correct peer_id and port corresponding to the Rust node.

Steps:



1. Start an OCaml node and wait for its p2p to be ready
2. Start a Rust node and initiate its connection to the OCaml node
3. Run the Rust node until it receives an event signaling that connection is established
4. Run the Rust node until it receives a Kademlia event signaling that the address of the OCaml node has been added
5. Check that the Rust node has an address of the OCaml node in its Kademlia part of the state


##### OCaml to Rust via seed Node

This test ensures that an OCaml node can connect to the Rust node, the address of which can only be discovered from an OCaml seed node, and its address becomes available in the Rust node’s Kademlia state. It also checks whether the OCaml node has a peer with the correct peer_id and a port corresponding to the Rust node.

Steps:



1. Start an OCaml node acting as a seed node and wait for its P2P to be ready
2. Start a Rust node and initiate its connection to the seed node
3. Run the Rust node until it receives an event signaling that connection is established
4. Start an OCaml node acting with the seed node as its peer
5. Run the Rust node until it receives an event signaling that the connection with the OCaml node has been established
6. Check that the Rust node has an address of the OCaml node in its Kademlia part of the state


##### Rust to OCaml via seed Node

This test ensures that the Rust node can connect to an OCaml peer, the address of whom can only be discovered from an OCaml seed node, and that the Rust node adds its address to its Kademlia state. It also checks whether the OCaml node has a peer with the correct peer_id and port corresponding to the Rust node.

Steps:



1. Start an OCaml node acting as a seed node
2. Start an OCaml node acting with the seed node as its peer and wait for its p2p to be ready
3. Start a Rust node and initiate its connection to the seed node
4. Run the Rust node until it receives an event signaling that connection with the seed node is established
5. Run the Rust node until it receives an event signaling that connection with the non-seed OCaml node is established
6. Check that the Rust node has an address of the OCaml node in its Kademlia part of the state


##### Rust as a Seed Node

This test ensures that the Rust node can work as a seed node by running two \
OCaml nodes that only know about the Rust node’s address. After these nodes connect \
to the Rust node, the test makes sure that they also have each other’s addresses \
as their peers.

Steps:



1. Start a Rust node
2. Start two OCaml nodes, specifying the Rust node address as their peer
3. Wait for events indicating that connections with both OCaml nodes are established
4. Check that both OCaml nodes have each other’s address as their peers
5. Check that the Rust node has addresses of both OCaml nodes in the Kademlia state


### Adaptive Peer Management

Nodes should be able to discover and connect to new peers if their current peers become unresponsive or malicious.

_This test evaluates the blockchain node's capacity to adapt to changing network conditions. It examines whether a node can autonomously identify unresponsive or malicious peers and replace them with trustworthy counterparts. Adaptive peer management enhances the network's resilience against potential attacks or unreliable participants._


## 2. Network Resistance


### Resistance to DDoS Attacks

The network should remain operational even under targeted Denial-of-Service attacks on specific nodes or infrastructure.

_This test focuses on the node's ability to withstand Distributed Denial-of-Service (DDoS) attacks, which can overwhelm a node's resources and render it inaccessible. It assesses whether the node can continue to function and serve the network even when subjected to deliberate and sustained attacks, as well as how much of these attacks it can withstand while remaining operational._


### Resistance to Eclipse Attacks

Honest nodes should not be isolated by malicious nodes in a way that they only receive information from these malicious entities.

_This test examines the blockchain node's resistance to eclipse attacks, where malicious nodes surround and isolate honest nodes, limiting their access to accurate information. It ensures that honest nodes can always access a diverse set of peers and aren't dominated by malicious actors._


### Resistance to Sybil Attacks

The network should function even if an adversary creates a large number of pseudonymous identities. Honest nodes should still be able to connect with other honest nodes.

_This test assesses the network's ability to mitigate Sybil attacks, wherein an adversary creates numerous fake identities to control a substantial portion of the network. It verifies that the network can maintain its integrity and continue to operate effectively despite the presence of these pseudonymous attackers._


### Resistance to Censorship

The network should resist attempts by any subset of nodes to consistently censor or block certain transactions or blocks.

_This test assesses the node's ability to resist censorship attempts by a subset of nodes. It verifies that the network's design prevents any small group from censoring specific transactions or blocks, upholding the blockchain's openness and decentralization._


## 3. Node Bootstrapping and Data Availability


### Node Bootstrapping 

New nodes joining the network should eventually discover and connect to honest peers and synchronize the latest blockchain state.

_This test evaluates the node's capability to bootstrap onto the network successfully. It ensures that newly joined nodes can find trustworthy peers, initiate synchronization, and catch up with the current state of the blockchain, enhancing network scalability._

This test is focused on ensuring that the latest Openmina build is able to bootstrap against Berkeleynet. It is executed on daily basis.

The node's HTTP port is accessible as [http://1.k8.openmina.com:31001](http://1.k8.openmina.com:31001/).

These are the main steps and checks.

First, it performs some checks on the instance deployed previously:



* Node is in sync state
* Node's best tip is the one that of berkeleynet

Then it deploys the new instance of Openmina and waits until it is bootstrapped (with 10 minutes timeout). After that it performs the following checks:



* Node's best tip is the one that of berkeleynet
* There were no restarts for openmina container

See [Openmina Daily](https://github.com/openmina/openmina/blob/develop/.github/workflows/daily.yaml) workflow file for further details.


### Data Availability

Any piece of data (like a block or transaction) that is part of the blockchain should be available to any node that requests it.

_This test confirms that the blockchain node can consistently provide requested data to other nodes in the network. It guarantees that data availability is maintained, promoting transparency and trust in the blockchain's history._


## 4. Ledger Consistency and Propagation


### Consistent View of the Ledger

All honest nodes in the network should eventually have a consistent view of the ledger, agreeing on the order and content of blocks.

_This test ensures that all honest nodes converge on a consistent ledger view. It validates that nodes reach consensus on the order and content of blocks, preventing forks and ensuring a single, agreed-upon version of the blockchain._


### Block Propagation

Every new block that is mined or created should eventually be received by every honest node in the network.

_This test checks the blockchain node's efficiency in propagating newly created blocks throughout the network. It verifies that no node is excluded from receiving critical block updates, maintaining the blockchain's integrity._


### Transaction/Snark Propagation

Every transaction/snark broadcasted by a user should eventually be received and processed by the miners or validators in the network.

_This test examines the node's ability to promptly disseminate user-generated transactions and Snarks to the network. It ensures that these transactions are reliably processed by miners or validators, facilitating efficient transaction processing._


## 5. Blockchain Progress and Fairness


### Chain Progress

New blocks should be added to the blockchain at regular intervals, ensuring that the system continues to process transactions.

_This test assesses whether the blockchain node can consistently add new blocks to the chain at regular intervals. It guarantees that the blockchain remains operational and can accommodate a continuous influx of transactions._


### Fairness in Transaction Processing

Transactions should not be perpetually ignored or deprioritized by the network. Honest transactions should eventually get processed.

_This test evaluates the node's fairness in processing transactions. We want to ensure that no valid transactions are unjustly ignored or delayed, maintaining a fair and efficient transaction processing system._


## 6. Scalability and upgradibility


### Network Scalability

As the number of participants or the rate of transactions increases, the network should still maintain its liveness properties.

_This test examines how well the blockchain network can handle increased traffic and participation without compromising its liveness properties, ensuring that it remains robust and responsive as it scales._


### Upgradability

The network should be able to upgrade or change protocols without halting or fragmenting, ensuring continuous operation

_This test ensures that the blockchain network can seamlessly undergo protocol upgrades or changes without causing disruptions or fragmenting the network. It supports the network's adaptability and longevity._

These expanded descriptions provide a comprehensive understanding of the key tests for assessing the functionality and security of a blockchain node. Each test contributes to the overall robustness and reliability of the blockchain network.


## 7. How to run tests

cargo test --release --features scenario-generators


## 8. The Front End


### Daily run

Multiple Scenarios executed on a Cluster of Nodes


### Scenario

Scenarios are designed to satisfy checks for Network Connectivity and Peer Management, Network Resistance

Scenario = List of Steps + Breakpoints mapped on Nodes

First of all, we need to load a scenario - a specific situation or condition created to test our system's functionality. Think of it as a detailed script that outlines how the system should behave under certain circumstances. 

We develop only one screen for only one scenario, we load that scenario, then we need to add steps.


#### Run Steps

We see the finished steps and the steps to do.

We start running the scenario, then even traces appear we can add even traces directly from here or we can see whether it's a new trace or it's already matching something.

There is a simple input for adding new steps as we have enough steps, we can start a run scenario.


#### Event Traces


![eventtraces](https://github.com/openmina/openmina/assets/60480123/10bc2cba-21d6-4cae-bb28-30b59f3423d2)


### Bootstrapping

#### Bootstrapping with recorded data

- [ ] Genesis block
- [ ] First epoch, < 290 blocks
- [ ] First epoch, > 290 blocks
- [ ] Third epoch and further

#### Bootstrapping with real peers

- [ ] Network split
- [ ] Long-running network (cluster)
- [x] Berkeley testnet: [![Openmina Daily](https://github.com/openmina/openmina/actions/workflows/daily.yaml/badge.svg)](https://github.com/openmina/openmina/actions/workflows/daily.yaml)

#### Various bootstrap scenarios

_TODO_

### General Network Behaviour

This might be a set of short and very long-running task, during that we make sure that our node
- can run in a (real) network for a long time, without being disconected too much
- can bootstrap other nodes (both OCaml and Rust)
- can relay p2p data
- can handle forks
- ...

### Snark Work

#### Network

- [ ] Check that snark pool is proadcasted properly
- [ ] Check that commitments are broadcasted properly

#### Correctness

- [ ] Check proof generated for several transactions

#### Coordination

Here we can test snark work is parallelized well, i.e.

$t_{total} < \frac {\sum {t_i} + C} {n}$

- [ ] Test that snark work is parallelized well

#### Throughput

- [ ] Run a tx load against network and make sure tx pool doesn't grow


