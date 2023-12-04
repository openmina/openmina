
# **OCaml Peers Discovery Tests**

As we develop the Openmina node, a new Rust-based node implementation for the Mina network, we must ensure that the Rust node can utilize the Kademlia protocol (KAD) to discover and connect to OCaml nodes, and vice versa.

For that purpose, we have developed a series of basic tests that check the correct peer discovery via KAD when the Rust node is connected to OCaml peers.


## **OCaml to Rust**

This test ensures that after an OCaml node connects to the Rust node, its address \
becomes available in the Rust node’s Kademlia state. It also checks whether the OCaml \
node has a peer with the correct peer_id and a port corresponding to the Rust node.

Steps:



1. Configure and launch a Rust node
2. Start an OCaml node with the Rust node as the only peer
3. Run the Rust node until it receives an event signaling that the OCaml node is connected
4. Wait for an event Identify that is used to identify the remote peer’s address and port
5. Check that the Rust node has an address of the OCaml node in its Kademlia part of the state


## **Rust to OCaml**

This test ensures that after the Rust node connects to an OCaml node with a known \
address, it adds its address to its Kademlia state. It also checks that the OCaml \
node has a peer with the correct peer_id and port corresponding to the Rust node.

Steps:



1. Start an OCaml node and wait for its p2p to be ready
2. Start a Rust node and initiate its connection to the OCaml node
3. Run the Rust node until it receives an event signaling that connection is established
4. Run the Rust node until it receives a Kademlia event signaling that the address of the OCaml node has been added
5. Check that the Rust node has an address of the OCaml node in its Kademlia part of the state


## **OCaml to Rust via seed Node**

This test ensures that an OCaml node can connect to the Rust node, the address of which can only be discovered from an OCaml seed node, and its address becomes available in the Rust node’s Kademlia state. It also checks whether the OCaml node has a peer with the correct peer_id and a port corresponding to the Rust node.

Steps:



1. Start an OCaml node acting as a seed node and wait for its P2P to be ready
2. Start a Rust node and initiate its connection to the seed node
3. Run the Rust node until it receives an event signaling that connection is established
4. Start an OCaml node acting with the seed node as its peer
5. Run the Rust node until it receives an event signaling that the connection with the OCaml node has been established
6. Check that the Rust node has an address of the OCaml node in its Kademlia part of the state


## **Rust to OCaml via seed Node**

This test ensures that the Rust node can connect to an OCaml peer, the address of whom can only be discovered from an OCaml seed node, and that the Rust node adds its address to its Kademlia state. It also checks whether the OCaml node has a peer with the correct peer_id and port corresponding to the Rust node.

Steps:



1. Start an OCaml node acting as a seed node
2. Start an OCaml node acting with the seed node as its peer and wait for its p2p to be ready
3. Start a Rust node and initiate its connection to the seed node
4. Run the Rust node until it receives an event signaling that connection with the seed node is established
5. Run the Rust node until it receives an event signaling that connection with the non-seed OCaml node is established
6. Check that the Rust node has an address of the OCaml node in its Kademlia part of the state


## **Rust as a Seed Node**

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
