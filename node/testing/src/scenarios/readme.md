
# Network Connectivity and Peer Management


### Network Connectivity

Nodes that get disconnected should eventually be able to reconnect and synchronize with the network.

_This test assesses the blockchain node's ability to maintain consistent network connectivity. It evaluates whether a node can gracefully handle temporary disconnections from the network and subsequently reestablish connections. _

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


##### Kademlia peer discovery

We want the Open Mina node to be able to connect to peers, both other Open Mina nodes (that are written in Rust) as well as native Mina nodes (written in OCaml). 

Native Mina nodes use Kademlia (KAD), a distributed hash table (DHT) for peer-to-peer computer networks. Hash tables are data structures that map _keys_ to _values_. Think of a hash table as a dictionary, where a word (i.e. dog) is mapped to a definition (furry, four-legged animal that barks). 

In Mina nodes, KAD specifies the structure of the network and the exchange of information through node lookups, which makes it efficient for locating nodes in the network. 

Since we initially focused on other parts of the node, we used the RPC get_initial_peers as a sort-of workaround to connect our nodes between themselves. Now, to ensure compatibility with the native Mina node, we’ve implemented KAD for peer discovery for the Open Mina node. 


##### How does Mina utilize Kademlia?

Kademlia has two main parts - the routing table and the peer store. 



1. The routing table is used to store information about network paths, enabling efficient data packet routing. It maintains peer information (peer id and network addresses)
2. The peer store is a database for storing and retrieving network peer information (peer IDs and their network addresses), forming part of the provider store. Providers are nodes that possess specific data and are willing to share or provide this data to other nodes in the network.

Peers are added to the routing table if they can communicate, support the correct protocol, and send/respond to valid queries. They are removed if unresponsive. Peers are added to the peer store when handling AddProvider messages.

A provider in Kademlia announces possession of specific data (identified by a unique key) and shares it with others. In MINA's case, all providers use the same key, which is the SHA256 hash of a specific string pattern. In MINA, every node acts as a “provider,” making the advertisement as providers redundant. Non-network nodes are filtered at the PNet layer.

If there are no peers, KAD will automatically search for new ones. KAD will also search for new peers whenever the node is restarted. If a connection is already made, it will search for more peers every hour. 


##### Message types



* AddProvider - informs the peer that you can provide the information described by the specified key.
* GetProviders - a query for nodes that have already performed AddProvider.
* FindNode - is used for different purposes. It can find a place in the network where your node should be. Or it may find a node that you need to send an AddProvider (or GetProviders) message to.


##### Potential issues identified
* An earlier issue in the Open Mina (Rust node) with incorrect provider key advertising is now fixed.
* The protocol's use in OCaml nodes might be a potential security risk; an adversary could exploit this to DoS the network. One possible solution is to treat all Kademlia peers as providers.
* The peer can choose its peer_id (not arbitrarily, the peer chooses a secret key, and then the peer_id is derived from the secret key). The peer can repeat this process until its peer_id is close to a key that identifies some desired information. Thus, the peer will be responsible for storing providers of this information.

* The malicious peer can deliberately choose the peer_id and deny access to the information, just always say there are no providers.
This problem has been inherited from the OCaml implementation of the node. We have mitigated it by making the Openmina node not rely on GetProviders, instead, we only do AddProviders to advertise ourselves, but treat any peer of the Kademlia network as a valid Mina peer, no matter if it is a provider or not, so a malicious peer can prevent OCaml nodes from discovering us, but it will not prevent us from discovering OCaml nodes.

* Kademlia exposes both internal and external addresses, which may be unnecessary in the MINA network and could be a security risk. 

    If we test on any local network that belongs to these ranges because they will get filtered unless we manually disable (in code) these checks.
* There's also an issue with the handling of private IP ranges.
* We need more testing on support for IPv6. libp2p_helper code can't handle IPv6 for the IP range filtering.
