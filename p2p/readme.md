


# Implementation of the LibP2P networking stack 

A peer-to-peer (P2P) network serves as the backbone of decentralized communication and data sharing among blockchain nodes. It enables the propagation of transaction and block information across the network, facilitating the consensus process crucial for maintaining the blockchain's integrity. Without a P2P network, nodes in the Mina blockchain would be isolated and unable to exchange vital information, leading to fragmentation and compromising the blockchain's trustless nature. 

To begin with, we need a P2P _networking stack_, a set of protocols and layers that define how P2P communication occurs between devices over our network. Think of it as a set of rules and conventions for data transmission, addressing, and error handling, enabling different devices and applications to exchange data effectively in a networked environment. We want our networking stack to have the following features:



* **Security**

    A secure networking stack in a blockchain node is essential to protect the network from malicious attacks and ensure the integrity and privacy of financial transactions. Security is our utmost priority across every module of our blockchain node.

* **Versatility**

    We want to be able to adapt to various network conditions and protocols and allow the node to run on a variety of devices (including smartphones), enhancing its ability to interoperate and scale within a diverse and evolving blockchain ecosystem.

* **Flexibility** 

    Blockchains are constantly evolving through constant protocol updates and user demands may also change due to external factors. We need our networking stack to be flexible so that we can continuously adapt it to this dynamic environment. 


For our networking stack, we are utilizing LibP2P, a modular networking stack that provides a unified framework for building decentralized P2P network applications.


## LibP2P 

LibP2P is a particularly useful tool for organizing our P2P network due to the following:


### Modularity

Being modular means that we can customize the stacks for various types of devices, i.e. a smartphone may use a different set of modules than a server. 


### Cohesion

Modules in the stack can communicate between each other despite differences in what each module should do according to its specification. 


### Layers

LibP2P provides vertical complexity in the form of layers. Each layer serves a specific purpose, which lets us neatly organize the various functions of the P2P network. It allows us to separate concerns, making the network architecture easier to manage and debug.

<img width="250" alt="P2Players" src="https://github.com/openmina/openmina/assets/60480123/add64ddd-3201-4413-8b64-b9e3b954693e">


_Above: A simplified overview of the Open Mina LibP2P networking stack. The abstraction is in an ascending order, i.e. the layers at the top have more abstraction than the layers at the bottom. _

LibP2P offers us a cohesive way of abstracting the complexity of P2P communication. It uses Kademlia for efficient peer discovery and DHT functionalities, transport protocols, peer discovery, encryption, authentication, and multiplexing.

Now we describe each layer of the P2P networking stack in descending order of abstraction.


## RPCs

A node needs to continuously receive and send information across the P2P network.

For certain types of information, such as new transitions (blocks), the best tips or ban notifications, Mina nodes utilize remote procedure calls (RPCs).

An RPC is a query for a particular type of information that is sent to a peer over the P2P network. After an RPC is made, the node expects a response from it. 

Mina nodes use the following RPCs.



* `get_staged_ledger_aux_and_pending_coinbases_at_hash`
* `answer_sync_ledger_query`
* `get_transition_chain`
* `get_transition_chain_proof`
* `Get_transition_knowledge` (note the initial capital)
* `get_ancestry`
* `ban_notify`
* `get_best_tip`
* `get_node_status` (v1 and v2)
* `Get_epoch_ledger`



## Kademlia for peer discovery

The P2P layer enables nodes in the Mina network to discover and connect with each other. 

We want the Open Mina node to be able to connect to peers, both other Open Mina nodes (that are written in Rust) as well as native Mina nodes (written in OCaml). 

To achieve that, we need to implement peer discovery via Kademlia as part of our LibP2P networking stack. Previously, we used the RPC `get_initial_peers` as a sort of workaround to connect our nodes between themselves. Now, to ensure compatibility with the native Mina node, we’ve implemented KAD for peer discovery for the Openmina node. 

Kademlia, or KAD, is a distributed hash table (DHT) for peer-to-peer computer networks. Hash tables are a type of data structure that maps _keys_ to _values_. In very broad and simplistic terms, think of a hash table as a dictionary, where a word (i.e. dog) is mapped to a definition (furry, four-legged animal that barks). In more practical terms, each key is passed through a hash function, which computes an index based on the key's content. 

KAD specifically works as a distributed hash table by storing key-value pairs across the network, where keys are mapped to nodes using the so-called XOR metric, ensuring that data can be efficiently located and retrieved by querying nodes that are closest to the key's hash.


### Measuring distance via XOR

XOR is a unique feature of how KAD measures the distance between peers - it is defined as the XOR metric between two node IDs or between a node ID and a key, providing a way to measure closeness in the network's address space for efficient routing and data lookup. 

The term "XOR" stands for "exclusive or," which is a logical operation that outputs true only when the inputs differ (one is true, the other is false). See the diagram below for a visual explanation:


![image6](https://github.com/openmina/openmina/assets/60480123/4e57f9b9-9e68-4400-b0ad-ff17c14766a1)


_Above: A Kademlia binary tree organized into four distinct buckets (marked in orange) of varying sizes. _

The XOR metric used by Kademlia for measuring distance ensures uniformity and symmetry in distance calculations, allowing for predictable and decentralized routing without the need for hierarchical or centralized structures, which allows for better scalability and fault tolerance in our P2P network.

LibP2P leverages Kademlia for peer discovery and DHT functionalities, ensuring efficient routing and data location in the network. In Mina nodes, KAD specifies the structure of the network and the exchange of information through node lookups, which makes it efficient for locating nodes in the network. 


## Multiplexing via Yamux

In a P2P network, connections are a key resource. Establishing multiple connections between peers can be costly and impractical, particularly in a network consisting of devices with limited resources. To make the most of a single connection, we employ _multiplexing_, which means having multiple data streams transmitted over a single network connection concurrently.


![image3](https://github.com/openmina/openmina/assets/60480123/5f6a48c7-bbae-4ca2-9189-badae2369f3d)


This provides efficient, concurrent handling of multiple data streams over a single connection, aligning well with the needs of modern, scalable, and efficient network protocols and applications.

For multiplexing, we utilize _Yamux_, a multiplexer that has several specific advantages for use in the Open Mina node:


### Efficiency

Blockchain nodes must handle a significant amount of data with minimal latency, and Yamux has particularly efficient data transmission as well as a simple header-based framing mechanism that helps optimize network resources. 

Yamux is designed to have a very small control message overhead. The protocol uses a binary framing layer to manage the multiplexed streams, and its headers are concise, which minimizes the amount of additional data sent over the network and maximizes the bandwidth available for actual application data.


### Transport-agnosticism

We want to be able to launch Mina nodes from a variety of devices including smartphones. Yamux is transport-agnostic, meaning it can be used over various underlying transport mechanisms. This flexibility can be valuable in adapting to different network environments and ensuring that the node can communicate effectively, regardless of what device it is launched from.


## Noise encryption

We want to ensure that data exchanged between nodes remains confidential, authenticated, and resistant to tampering. For that purpose, we utilize Noise, a cryptographic protocol featuring ephemeral keys and forward secrecy, used to secure the connection.

Similarly to our other modules, Noise is flexible, modular, efficient and lightweight, which makes it particularly well-suited for the Open Mina node. In addition to that, it provides the following capabilities:


### Async

Noise supports asynchronous communication, allowing nodes to communicate without both being online simultaneously can efficiently handle the kind of non-blocking I/O operations that are typical in P2P networks, where nodes may not be continuously connected, even in the asynchronous and unpredictable environments that are characteristic of blockchain P2P networks.


### Forward secrecy

Noise utilizes _ephemeral keys_, which are random keys generated for each new connection that must be destroyed after use. The use of ephemeral keys forward secrecy. This means that decrypting a segment of the data does not provide any additional ability to decrypt the rest of the data. Simply put, forward secrecy means that if an adversary gains knowledge of the secret key, they will be able to participate in the network on the behalf of the peer, but they will not be able to decrypt past nor future messages.


### How Noise works

The Noise protocol implemented by libp2p uses the [XX](http://www.noiseprotocol.org/noise.html#interactive-handshake-patterns-fundamental) handshake pattern, which happens in the following stages:


![image4](https://github.com/openmina/openmina/assets/60480123/a1b2b2bf-980e-459c-8375-9e8b6162b6d1)




1. Alice sends Bob her ephemeral public key (32 bytes).


![image2](https://github.com/openmina/openmina/assets/60480123/721103dd-0bb9-4f0b-8998-97b0cc19f6fc)



2. Bob responds to Alice with a message that contains:
* Bob’s ephemeral public key (32 bytes).
* Bob's static public key (32 bytes).
* The tag (MAC) of the static public key (16 bytes).

As well as a payload of extra data that includes the peer’s `identity_key`, an `identity_sig`, Noise's static public key and the tag (MAC) of the payload (16 bytes).


![image5](https://github.com/openmina/openmina/assets/60480123/b7ed062d-2204-4b94-87af-abc6eecd7013)


1. Alice responds to Bob with her own message that contains:
* Alice's static public key (32 bytes).
* The tag (MAC) of Alice’s static public key (16 bytes).
* The payload, in the same fashion as Bob does in the second step, but with Alice's information instead.
* The tag (MAC) of the payload (16 bytes).

After the messages are exchanged (two sent by Alice, the _initiator,_ and one sent by Bob, the _responder_), both parties can derive a pair of symmetric keys that can be used to cipher and decipher messages.


## Pnet layer

We want to be able to determine whether the peer to whom we want to connect to is running the same network as our node. For instance, a node running on the Mina mainnet will connect to other mainnet nodes and avoid connecting to peers running on Mina’s testnet.

For that purpose, Mina utilizes pnet, an encryption transport layer that constitutes the lowest layer of libp2p. Please note that while the network (IP) and transport (TCP) layers are lower than pnet, they are not unique to libp2p.

In Mina, the pnet _secret key_ refers to the chain on which the node is running, 

for instance `mina/mainnet` or `mina/testnet`. This prevents nodes from attempting connections with the incorrect chain.

Although pnet utilizes a type of secret key known as a pre-shared key (PSK), every peer in the network knows this key. This is why, despite being encrypted, the pnet channel itself isn’t secure - that is achieved via the aforementioned Noise protocol.


## Transport

At the lowest level of abstraction, we want our P2P network to have a reliable, ordered, and error-checked method of transporting data between peers. crucial for maintaining the integrity and consistency of the blockchain. 

Libp2p connections are established by _dialing_ the peer address across a transport layer. Currently, Mina uses TCP, but it can also utilize UDP, which can be useful if we implement a node based on WebRTC.

Peer addresses are written in a convention known as _Multiaddress_, which is a universal method of specifying various kinds of addresses.

For example, let’s look at one of the addresses from the [Mina Protocol peer list](https://storage.googleapis.com/mina-seed-lists/mainnet_seeds.txt).


```
/dns4/seed-1.mainnet.o1test.net/tcp/10000/p2p/12D3KooWCa1d7G3SkRxy846qTvdAFX69NnoYZ32orWVLqJcDVGHW

```



* `/dns4/seed-1.mainnet.o1test.net/ `States that the domain name is resolvable only to IPv4 addresses
* `tcp/10000 `tells us we want to send TCP packets to port 10000.
* `p2p/12D3KooWCa1d7G3SkRxy846qTvdAFX69NnoYZ32orWVLqJcDVGHW` informs us of the hash of the peer’s public key, which allows us to encrypt communication with said peer.

An address written under the _Multiaddress_ convention is ‘future-proof’ in the sense that it is backwards-compatible. For example, since multiple transports are supported, we can change `tcp `to `udp`, and the address will still be readable and valid.

