# WebRTC Based P2P

## Design goals
In blockchain and especially in Mina, **security**, **decentralization**,
**scalability** and **eventual consistency** (in that order), is crucial.
Our design tries to achieve those, while building on top of
Mina Protocol's existing design (outside of p2p).

#### Security
By security, we are mostly talking about **DDOS Resilience**, as that is
the primary concern in p2p layer.

Main ways to achieve that:
1. Protocol design needs to enable an ability to identify malicious actors
   as soon as possible, so that they can be punished (disconnected, blacklisted)
   with minimal resource inverstment. This means, individual messages
   should be small and verifiable, so we don't have to allocate bunch 
   of resources(CPU + RAM + NET) before we are able to process them.
2. Even if the peer isn't breaking any protocol rules, single peer
   (or a group of them) shouldn't be able to consume big chunk of our
   resources, so there needs to be **fairness** enabled by the protocol itself.
3. Malicious peers shouldn't be able to flood us with incoming connections,
   blocking us from receiving connections from genuine peers.

#### Decentralization and Scalability
Mina Protocol, with it's consensus mechanism and recursive zk-snarks,
enables light-weight full clients. So anyone can run the full node
(as demonstrated with the WebNode). While this is great for decentralization,
it puts higher load on p2p network and raises it's requirements.

So we needed to come up with the design that can support hundreeds of
active connections in order to increase fault tolerance and  not sacrifice
**scalability**, since if we have more nodes in the network but few connections
between them, [diameter](https://mathworld.wolfram.com/GraphDiameter.html)
of the network increases, so each message has to go through more hops
(each adding latency) in order to reach all nodes in the network.

#### Eventual Consistency
Nodes in the network should eventually come to the same state
(same best tip, transaction/snark pool), without the use of crude rebroadcasts.

## Transport Layer
**(TODO: explain why WebRTC is the best option for security and decentralization)**

## Poll-Based P2P
It's practically impossbile to achieve [above written design goals](#design-goals)
with a push-based approach, where messages are just sent to the peers,
without them ever requesting them, like it's done with libp2p GossipSub.
Since when you have a push based approach, most likely you can't process
all messages faster than they can be received, so you have to maintain a
queue for messages, which might even "expire" (be no longer relevant)
once we reach them. Also you can't grow queue infinitely, so you have to
drop some messages, which will break [eventual consistency](#eventual-consistency).
Also it's very bad for security as you are potentially letting peers
allocate significant amount of data.

So instead, we decided to go with poll based approach, or more accurately,
with something resembling the [long polling](https://www.pubnub.com/guides/long-polling/).

In a nutshell, instead of a peer flooding us with messages, we have to
request from them (send a sort of permit) for them to send us a message.
This way, the recipient controls the flow, so it can:
1. Enforce **fairness**, that we mentioned in the scalability design goals.
2. Prevent peer from overwhelming the system, because previous message
   needs to be processed, until the next is requested.

This removes whole lot of complexity from the implementation as well.
We no longer have to worry about the message queue, what to do if we
can process messages slower than we are receiving them, if we drop them,
how do we recover them if they were relevant, etc...

Also this unlocks whole lot of possibilities for **eventual consistency**.
Because it's not just about recipient. Now the sender has the guarantee that
the sent messages have been processed by the recipient, if they were followed
by a request for the next message. This way the sender can reason about what
the peer already has and what they lack, so it can adjust the messages
that it sends based on that.

## Implementation

### Connection
In order for two peers to connect to each other via WebRTC, they need
to exchange **Offer** and **Answer** messages between each other. Process of
exchanging those messages is called **Signaling**. There are many ways
to exchange them, our implementation supports two ways:
- **HTTP API** - Dialer sends an http request containing the **offer** and receives
  an **answer** if peer is ok with connecting, or otherwise an error.
- **Relay** - Dialer will discover the listener peer via relay peer. The relay
  peer needs to be connected to both dialer and listener, so that it can
  facilitate exchange of those messages, if both parties agree to connect.
  After those messages are exchanged, relay peer is no longer needed and
  they establish a direct connection.

For security, it's best to use just the **relay** option, since it doesn't
require a node to open port accessible publicly and so it can't be flooded
with incoming connections.
Seed nodes will have to support **HTTP Api** though, so that initial connection
can be formed by new clients.

### Channels
We are using different [WebRTC DataChannel](https://developer.mozilla.org/en-US/docs/Web/API/RTCDataChannel)
per protocol.

So far, we have 8 protocols:
1. **SignalingDiscovery** - used for discovering new peers via existing ones.
2. **SignalingExchange** - used for exchanging signaling messages via **relay** peer.
3. **BestTipPropagation** - used for best tip propagation. Instead of the whole
   block, only consensus state + block hash (things we need for consensus)
   is propagated and the rest can be fetched via **Rpc** channel.
4. **TransactionPropagation** - used for transaction propagation. Only info
   is sent which is necessary to determine if we want that transaction
   based on the current transaction pool state. Full transaction can be
   fetched with hash from **Rpc** channel.
5. **SnarkPropagation** - used for snark work propagation. Only info
   is sent which is necessary to determine if we want that snark
   based on the current snark pool state. Full snark can be
   fetched with job id from **Rpc** channel.
6. **SnarkJobCommitmentPropagation** - implemented but not used at the moment.
   It's for the decentralized snark work coordination to minimize wasted resources. 
7. **Rpc** - used for requesting specific data from peer.
8. **StreamingRpc** - used to fetch from peer big data in small verifiable chunks,
   like fetching data necessary to reconstruct staged ledger at the root.

For each channel, we have to receive a request before we can send a response.
E.g. in case of **BestTipPropagation** channel, block won't be propagated until
peer sends us the request.

### Efficient Pool Propagation with Eventual Consistency
To achieve scalable, eventually consistent and efficient (transaction/snark)
pool propagation, we need to utilize benefits of the poll-based approach.

Since we can track which messages were processed by the peer, in order
to achieve eventual consistency, we just need to:
1. Make sure we only send pool messages if peer's best tip is same
   or higher than our own, so that we make sure peer doesn't reject our
   messages because it is out of sync.
2. After the connection is established, make sure all transactions/snarks
   (just info) in the pool is sent to the connected peer.
3. Keep track of what we have already sent to the peer.
4. **(TODO eventual consistency with limited transaction pool size)**

For 2nd and 3rd points, to efficiently keep track of what messages we
have sent to which peer a special [data structure](../core/src/distributed_pool.rs) is used.
Basically it's an append only log, where each entry is indexed by a number
and if we want to update an entry, we have to remove it and append it at
the end. As new transactions/snarks are added to the pool, they are appended
to this log.

With each peer we just keep an index (initially 0) of the next message 
to propagate and we keep sending the next (+ jumping to the next index)
until we reach the end of the pool. This way we have to keep minimal data
with each peer and it efficiently avoids sending the same data twice.

## Appendix: Future Ideas

### Leveraging Local Pools for Smaller Blocks

Nodes already maintain local pools of transactions and snarks. Many of these stored items later appear in blocks. By using data the node already has, we reduce the amount of information needed for each new block.

#### Motivation
As nodes interact with the network, they receive, verify, and store transactions and snarks in local pools. When a new block arrives, it often includes some of these items. Because the node already has them, the sender need not retransmit the data. This approach offers:

1. **Reduced Bandwidth Usage:**
   Eliminating redundant transmissions of known snarks and transactions reduces block size and prevents wasted data exchange.

2. **Decreased Parsing and Validation Overhead:**
   With fewer embedded items, nodes spend less time parsing and validating large blocks and their contents, and can more quickly integrate them into local state.

3. **Memory Footprint Optimization:**
   By avoiding duplicate data, nodes can maintain more stable memory usage.

#### Practical Considerations
- **Snarks:**
  Snarks, being large, benefit most from this approach. Skipping their retransmission saves significant bandwidth.

- **Ensuring Synchronization:**
  This approach assumes nodes maintain consistent local pools. The poll-based model and eventual consistency ensure nodes receive needed items before they appear in a block, making it likely that a node has them on hand.

- **Adjusting the Block Format:**
  This idea may require altering the protocol so the block references, rather than embeds, items nodes probably have. The node would fetch only missing pieces if a reference does not match its local data.

#### Outcome
By using local data, the network can propagate smaller blocks, improving scalability, reducing resource usage, and speeding propagation.

# OCaml node compatibility
In order to be compatible with the current OCaml node p2p implementation,
we have [libp2p implementation](./libp2p.md) as well. So communication between OCaml
and Rust nodes is done via LibP2P, while Rust nodes will use WebRTC
to converse with each other.
