# Peer discovery

## Objectives

The Openmina node must maintain connections with peers. The list of peers must meet the requirements:

* The number of peers must not exceed an upper limit and should not fall below a lower limit.
* Peers must be as good as possible. Node must evaluate each peer by uptime, correctness of information provided, and ping. Node must find a balance point to dynamically maximize all of these values.
* Node must choose peers in a way that allows global consistency of the network, provides security and avoids the network centralization.

This specification describes initial peer discovery, peer selecting and peer evaluation algorithms.

### Initial peer discovery

There are so-called seed peers. These peers are normal nodes except for a few features:

* The seed peer's address must be static. 
* The seed peer must not do block production or snark work.
* The seed peer should have high uptime and support many connections.

The node should have a list of seed peer addresses at startup. To do a peer discovery, the node must connect to seed peers and call the `get_some_initial_peers` RPC. It will return a list of addresses of peers to start with.

The call `get_some_initial_peers` doesn't have parameters, the response is the list of structures that represents a peer:

```Rust
type Response = Vec<InitialPeerAddress>;

struct InitialPeerAddress {
    ip: String, // must be string representation of Ipv4 or Ipv6
    port: u16, // a TCP port number
    peer_id: String, // a string representation of multihash
}
```

The same RPC call can be made to any peer, not just the seed peer.

The node must distinguish between a temporary connection, which is made only for calling the `get_some_initial_peers` RPC, and a normal connection, which is used for all other tasks.

### Peer selecting

After the node receives initial peers from seed nodes, the total number of known peers may already exceed the maximum number of peers. However, using these peers is not optimal because it leads to centralization. Imagine there are three seed nodes, each connected to 100 peers, they can provide a total of 100-300 unique peers. And thousands of fresh users connect to those same peers. These 100-300 peers will be congested and the network will be unreliable and centralized.

To avoid this situation, the node must select peers.

#### Create a reference graph

Node should keep a database of known nodes and their references. Let call the node $A$ references the node $B$ (denoted $A \to B$) if and only if the node $A$ return address of node $B$ in the response for `get_some_initial_peers` call.

Having this relation we can create a peer graph. 

```Rust
struct PeerGraphNode {
    address: Multiaddr,
    last_updated: u64,
    availability_score: u8,
    correctness_score: u8,
    ping_ms: u16,
}

struct PeerGraphEdge;
```

The node must keep this graph current and complete. To do this, the node must make a temporary connection to the peer that was updated a long time ago and re-run the `get_some_initial_peers` RPC.

#### Security

Having the peer graph allows us to assign to each peer the one more evaluation: the number of unique paths between a given seed peer and the peer under evaluation. The peer that has more unique paths to reach it is better because it is less likely to be part of the malicious network. Malicious actor can create arbitrary many peers, but it is difficult to create arbitrary many links and it is even more difficult to create arbitrary many paths between honest and malicious nodes.

#### Decentralization

To maintain decentralization the node must select peers uniformly across the graph, so more distant nodes has the same chance to be selected.

### Peer evaluation

<!-- TODO: -->

## Implementation

High level algorithm overview:

* Maintain the peer graph using temporal connections and the `get_some_initial_peers` RPC.
* Define a "base probability to select". Compute the probability equal to the maximum allowed number of connections divided by the number of nodes in the peer graph.
* Compute an integrated score of the peer, a single normalized number that includes "security", "uptime", "correctness", and "ping" scores.
* Vary the base probability to account for the score.
* Randomly select peers with probability. 
* Truncate the smallest (by the score) peers if the number of peers selected exceeds the maximum.

<!-- TODO: How to achieve the objectives -->
