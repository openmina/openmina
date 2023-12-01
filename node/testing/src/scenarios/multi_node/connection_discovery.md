# OCaml Peers Discovery Tests

This group of contains basic tests that check correct peer discovery
implementation (Kademlia) in the Rust node when it is connected to OCaml peers.

## OCaml to Rust

This test ensure that after an OCaml node connects to the Rust node, its address
becomes available in the Rust node's Kademlia state. Also it checks that OCaml
node has a peer with correct peer_id and port that correspond to the Rust node.

Steps:
- Config and create a Rust node
- Start OCaml node with the Rust node as the only peer
- Run the Rust node until it receives an event signalling that the OCaml node is connected
- Wait for an event `Identify` that is used to identify remote peer's address and port
- Check that the Rust node has an address of the OCaml node in its Kademlia part of the state

## Rust to OCaml

This test ensure that after the Rust node connects to an OCaml node with known
address, it adds its address to its Kademlia state. Also it checks that OCaml
node has a peer with correct peer_id and port that correspond to the Rust node.

Steps:
- Start an OCaml node and wait for its p2p to be ready
- Start a Rust node and initiate its connection to the OCaml node
- Run the Rust node until it receives an event signalling that connection is established
- Run the Rust node until it receives a Kademlia event signalling that the address of the OCaml node is added
- Check that the Rust node has an address of the OCaml node in its Kademlia part of the state


## OCaml to Rust Via Seed Node

This test ensure that an OCaml node can connect to to the Rust node which
address can only be discovered from a seed node, and its address becomes
available in the Rust node's Kademlia state. Also it checks that the OCaml node
has a peer with correct peer_id and port that correspond to the Rust node.

Steps:
- Start an OCaml node acting as a seed node and wait for its p2p to be ready
- Start a Rust node and initiate its connection to the seed node
- Run the Rust node until it receives an event signalling that connection is established
- Start an OCaml node acting with the seed node as its peer
- Run the Rust node until it receives an event signalling that connection with the OCaml node is established
- Check that the Rust node has an address of the OCaml node in its Kademlia part of the state

## Rust to OCaml via Seed Node

This test ensure that the Rust node can connect to an OCaml peer which address
can only be discovered from a seed node, and that the Rust node adds its address
to its Kademlia state. Also it checks that the OCaml node has a peer with correct
peer_id and port that correspond to the Rust node.

Steps:
- Start an OCaml node acting as a seed node
- Start an OCaml node acting with the seed node as its peer and wait for its p2p to be ready
- Start a Rust node and initiate its connection to the seed node
- Run the Rust node until it receives an event signalling that connection with the seed node is established
- Run the Rust node until it receives an event signalling that connection with the non-seed OCaml node is established
- Check that the Rust node has an address of the OCaml node in its Kademlia part of the state


## Rust as a Seed Node

This test ensures that the Rust node can work as a seed node by running two
OCaml node that only know about the Rust node address. After these nodes connect
to the Rust node, the test makes sure that they also have each other's addresses
as their peers.

Steps:
- Start a Rust node
- Start two OCaml nodes, specifying the Rust node address as their peer
- Wait for events indicating that connections with both OCaml nodes are established
- Check that both OCaml nodes have each other's address as their peers
- Check that the Rust node has addresses of both OCaml nodes in Kademlia state
