---
title: P2P wire messages
description:
  Complete reference of all messages sent over the wire in the P2P layer
sidebar_position: 10
---

# P2P wire messages

This page provides a complete reference of all messages sent over the wire in
Mina's P2P layer. The P2P layer uses a dual-transport architecture supporting
both libp2p and WebRTC, with messages organized into logical channels.

## Architecture overview

Messages are organized in three layers:

1. **Channel layer** - High-level message types organized by function (8
   channels)
2. **Network layer** - Low-level protocol implementation (RPC, Kademlia, Pubsub,
   Identify)
3. **Wire format** - Binary serialization using BinProt

## Channel messages

The main message container is
[`ChannelMsg`](https://github.com/o1-labs/mina-rust/blob/develop/p2p/src/channels/mod.rs#L129-L199),
which routes all high-level wire messages to their respective channels:

```rust
pub enum ChannelMsg {
    SignalingDiscovery(SignalingDiscoveryChannelMsg),
    SignalingExchange(SignalingExchangeChannelMsg),
    BestTipPropagation(BestTipPropagationChannelMsg),
    TransactionPropagation(TransactionPropagationChannelMsg),
    SnarkPropagation(SnarkPropagationChannelMsg),
    SnarkJobCommitmentPropagation(SnarkJobCommitmentPropagationChannelMsg),
    Rpc(RpcChannelMsg),
    StreamingRpc(StreamingRpcChannelMsg),
}
```

### Channel identifiers

Each channel has a numeric identifier defined in
[`ChannelId`](https://github.com/o1-labs/mina-rust/blob/develop/p2p/src/channels/mod.rs#L35-L107):

| Channel ID | Name                          | Transport | Max size |
| ---------- | ----------------------------- | --------- | -------- |
| 1          | SignalingDiscovery            | WebRTC    | 16 KB    |
| 2          | SignalingExchange             | WebRTC    | 16 KB    |
| 3          | BestTipPropagation            | Both      | 32 MB    |
| 4          | TransactionPropagation        | Both      | 1 KB     |
| 5          | SnarkPropagation              | Both      | 1 KB     |
| 6          | SnarkJobCommitmentPropagation | WebRTC    | 2 KB     |
| 7          | Rpc                           | Both      | 256 MB   |
| 8          | StreamingRpc                  | WebRTC    | 16 MB    |

## Channel message types

### Signaling discovery

**Purpose:** WebRTC peer discovery and signaling through a relay peer.

**Definition:**
[`SignalingDiscoveryChannelMsg`](https://github.com/o1-labs/mina-rust/blob/develop/p2p/src/channels/signaling/discovery/mod.rs#L17-L33)

```rust
pub enum SignalingDiscoveryChannelMsg {
    GetNext,
    Discover,
    Discovered { target_public_key: PublicKey },
    DiscoveredReject,
    DiscoveredAccept(EncryptedOffer),
    Answer(Option<EncryptedAnswer>),
}
```

**Messages:**

- **`GetNext`** - Request next queued signaling message
- **`Discover`** - Request to be discovered by other peers
- **`Discovered`** - Notification that a peer wants to connect
- **`DiscoveredReject`** - Reject the connection offer
- **`DiscoveredAccept`** - Accept connection with encrypted WebRTC offer
- **`Answer`** - WebRTC answer to complete connection setup

### Signaling exchange

**Purpose:** Direct WebRTC signaling message exchange between peers.

**Definition:**
[`SignalingExchangeChannelMsg`](https://github.com/o1-labs/mina-rust/blob/develop/p2p/src/channels/signaling/exchange/mod.rs#L17-L28)

```rust
pub enum SignalingExchangeChannelMsg {
    GetNext,
    OfferToYou {
        offerer_pub_key: PublicKey,
        offer: EncryptedOffer,
    },
    Answer(Option<EncryptedAnswer>),
}
```

**Messages:**

- **`GetNext`** - Request next queued signaling message
- **`OfferToYou`** - WebRTC offer from another peer
- **`Answer`** - WebRTC answer to complete connection

### Best tip propagation

**Purpose:** Propagation of the best known block (tip) in the blockchain.

**Definition:**
[`BestTipPropagationChannelMsg`](https://github.com/o1-labs/mina-rust/blob/develop/p2p/src/channels/best_tip/mod.rs#L13-L18)

```rust
pub enum BestTipPropagationChannelMsg {
    GetNext,
    BestTip(ArcBlock),
}
```

**Messages:**

- **`GetNext`** - Request next best tip update
- **`BestTip`** - New best tip block

### Transaction propagation

**Purpose:** Propagation of pending transactions with flow control.

**Definition:**
[`TransactionPropagationChannelMsg`](https://github.com/o1-labs/mina-rust/blob/develop/p2p/src/channels/transaction/mod.rs#L13-L29)

```rust
pub enum TransactionPropagationChannelMsg {
    GetNext { limit: u8 },
    WillSend { count: u8 },
    Transaction(TransactionInfo),
}
```

**Messages:**

- **`GetNext`** - Request up to `limit` transactions
- **`WillSend`** - Promise to send `count` transactions
- **`Transaction`** - Individual transaction data

**Flow:** Request → Promise → Deliver pattern for backpressure control.

### Snark propagation

**Purpose:** Propagation of SNARK proofs with flow control.

**Definition:**
[`SnarkPropagationChannelMsg`](https://github.com/o1-labs/mina-rust/blob/develop/p2p/src/channels/snark/mod.rs#L13-L29)

```rust
pub enum SnarkPropagationChannelMsg {
    GetNext { limit: u8 },
    WillSend { count: u8 },
    Snark(SnarkInfo),
}
```

**Messages:**

- **`GetNext`** - Request up to `limit` snarks
- **`WillSend`** - Promise to send `count` snarks
- **`Snark`** - Individual SNARK proof data

**Flow:** Request → Promise → Deliver pattern for backpressure control.

### Snark job commitment propagation

**Purpose:** SNARK worker commitment to produce proofs.

**Definition:**
[`SnarkJobCommitmentPropagationChannelMsg`](https://github.com/o1-labs/mina-rust/blob/develop/p2p/src/channels/snark_job_commitment/mod.rs#L13-L29)

```rust
pub enum SnarkJobCommitmentPropagationChannelMsg {
    GetNext { limit: u8 },
    WillSend { count: u8 },
    Commitment(SnarkJobCommitment),
}
```

**Messages:**

- **`GetNext`** - Request up to `limit` commitments
- **`WillSend`** - Promise to send `count` commitments
- **`Commitment`** - SNARK job commitment

**Flow:** Request → Promise → Deliver pattern for backpressure control.

### RPC

**Purpose:** Request-response communication for data retrieval.

**Definition:**
[`RpcChannelMsg`](https://github.com/o1-labs/mina-rust/blob/develop/p2p/src/channels/rpc/mod.rs#L33-L211)

```rust
pub enum RpcChannelMsg {
    Request(P2pRpcId, P2pRpcRequest),
    Response(P2pRpcId, Option<P2pRpcResponse>),
}
```

**RPC request types:**

```rust
pub enum P2pRpcRequest {
    BestTipWithProof,
    LedgerQuery(LedgerHash, MinaLedgerSyncLedgerQueryStableV1),
    StagedLedgerAuxAndPendingCoinbasesAtBlock(StateHash),
    Block(StateHash),
    Snark(SnarkJobId),
    Transaction(TransactionHash),
    InitialPeers,
}
```

**RPC response types:**

```rust
pub enum P2pRpcResponse {
    BestTipWithProof(BestTipWithProof),
    LedgerQuery(MinaLedgerSyncLedgerAnswerStableV2),
    StagedLedgerAuxAndPendingCoinbasesAtBlock(Arc<StagedLedgerAuxAndPendingCoinbases>),
    Block(ArcBlock),
    Snark(Snark),
    Transaction(Transaction),
    InitialPeers(List<P2pConnectionOutgoingInitOpts>),
}
```

**Request types:**

- **`BestTipWithProof`** - Request best tip with consensus proof
- **`LedgerQuery`** - Query ledger state for sync
- **`StagedLedgerAuxAndPendingCoinbasesAtBlock`** - Request staged ledger data
- **`Block`** - Request specific block by state hash
- **`Snark`** - Request specific SNARK by job ID
- **`Transaction`** - Request specific transaction by hash
- **`InitialPeers`** - Request list of peers for bootstrapping

### Streaming RPC

**Purpose:** Streaming request-response for large data transfers.

**Definition:**
[`StreamingRpcChannelMsg`](https://github.com/o1-labs/mina-rust/blob/develop/p2p/src/channels/streaming_rpc/mod.rs#L17-L33)

```rust
pub enum StreamingRpcChannelMsg {
    Next(P2pStreamingRpcId),
    Request(P2pStreamingRpcId, P2pStreamingRpcRequest),
    Response(P2pStreamingRpcId, Option<P2pStreamingRpcResponse>),
}
```

**Streaming RPC types:**
[`P2pStreamingRpcRequest`](https://github.com/o1-labs/mina-rust/blob/develop/p2p/src/channels/streaming_rpc/rpcs/mod.rs#L16-L34)

```rust
pub enum P2pStreamingRpcRequest {
    StagedLedgerParts(StateHash),
}

pub enum P2pStreamingRpcResponse {
    StagedLedgerParts(StagedLedgerPartsResponse),
}
```

**Staged ledger parts responses:**

```rust
pub enum StagedLedgerPartsResponse {
    Base(StagedLedgerPartsBase),
    ScanStateBase(ScanStateBase),
    PreviousIncompleteZkappUpdates(PreviousIncompleteZkappUpdates),
    ScanStateTree(ScanStateTree),
}
```

**Messages:**

- **`Next`** - Request next chunk in stream
- **`Request`** - Initiate streaming RPC
- **`Response`** - Stream response chunk

**Request types:**

- **`StagedLedgerParts`** - Stream staged ledger parts in chunks

## Network layer messages

### Network RPC protocol

**Purpose:** Low-level RPC protocol messages for libp2p RPC behavior.

**Definition:**
[`RpcMessage`](https://github.com/o1-labs/mina-rust/blob/develop/p2p/src/network/rpc/p2p_network_rpc_state.rs#L64-L70)

```rust
pub enum RpcMessage {
    Handshake,
    Heartbeat,
    Query { header: QueryHeader, bytes: Data },
    Response { header: ResponseHeader, bytes: Data },
}
```

**Messages:**

- **`Handshake`** - Establish RPC connection
- **`Heartbeat`** - Keep-alive message
- **`Query`** - RPC query with header and data
- **`Response`** - RPC response with header and data

### Kademlia (DHT)

**Purpose:** Distributed hash table for peer discovery and routing.

**Definition:**
[`Message`](https://github.com/o1-labs/mina-rust/blob/develop/p2p/src/network/kad/p2p_network_kad_message.rs)

**Message types:**

```rust
pub enum MessageType {
    PUT_VALUE = 0,
    GET_VALUE = 1,
    ADD_PROVIDER = 2,
    GET_PROVIDERS = 3,
    FIND_NODE = 4,
    PING = 5,
}
```

**Connection types:**

```rust
pub enum ConnectionType {
    NOT_CONNECTED = 0,
    CONNECTED = 1,
    CAN_CONNECT = 2,
    CANNOT_CONNECT = 3,
}
```

**Message structure:**

```rust
pub struct Message<'a> {
    pub type_pb: MessageType,
    pub clusterLevelRaw: i32,
    pub key: Cow<'a, [u8]>,
    pub record: Option<Record<'a>>,
    pub closerPeers: Vec<Peer<'a>>,
    pub providerPeers: Vec<Peer<'a>>,
}

pub struct Peer<'a> {
    pub id: Cow<'a, [u8]>,
    pub addrs: Vec<Cow<'a, [u8]>>,
    pub connection: ConnectionType,
}

pub struct Record<'a> {
    pub key: Cow<'a, [u8]>,
    pub value: Cow<'a, [u8]>,
    pub timeReceived: Cow<'a, str>,
}
```

### Gossipsub (pubsub)

**Purpose:** Publish-subscribe messaging for efficient broadcast.

**Definition:**
[`BroadcastMessageId`](https://github.com/o1-labs/mina-rust/blob/develop/p2p/src/network/pubsub/mod.rs#L1-L68)

**Topic:** `coda/consensus-messages/0.0.1`

**Message identifiers:**

```rust
pub enum BroadcastMessageId {
    BlockHash {
        hash: mina_p2p_messages::v2::StateHash,
    },
    Snark {
        job_id: SnarkJobId,
    },
    MessageId {
        message_id: P2pNetworkPubsubMessageCacheId,
    },
}
```

**Wire message type:** `GossipNetMessageV2` (from `mina_p2p_messages` crate)

### Identify protocol

**Purpose:** Peer identification and capability exchange.

**Definition:**
[`P2pNetworkIdentify`](https://github.com/o1-labs/mina-rust/blob/develop/p2p/src/network/identify/p2p_network_identify_protocol.rs#L1-L82)

```rust
pub struct P2pNetworkIdentify {
    pub protocol_version: Option<String>,
    pub agent_version: Option<String>,
    pub public_key: Option<PublicKey>,
    pub listen_addrs: Vec<Multiaddr>,
    pub observed_addr: Option<Multiaddr>,
    pub protocols: Vec<token::StreamKind>,
}
```

**Fields:**

- **`protocol_version`** - P2P protocol version
- **`agent_version`** - Node software version
- **`public_key`** - Peer's public key
- **`listen_addrs`** - Addresses peer is listening on
- **`observed_addr`** - Peer's observed external address
- **`protocols`** - Supported protocol streams

## LibP2P RPC methods

The following RPC methods are registered with the libp2p behavior (defined in
[`libp2p_node.rs`](https://github.com/o1-labs/mina-rust/blob/develop/p2p/testing/src/libp2p_node.rs#L143-L157)):

- **`GetBestTipV2`** - Retrieve best known block
- **`GetAncestryV2`** - Retrieve chain ancestry
- **`GetStagedLedgerAuxAndPendingCoinbasesAtHashV2`** - Retrieve staged ledger
  data
- **`AnswerSyncLedgerQueryV2`** - Answer ledger sync query
- **`GetTransitionChainV2`** - Retrieve chain of state transitions
- **`GetTransitionChainProofV1ForV2`** - Retrieve proof for transition chain

## Wire format

All messages are serialized using
[BinProt](https://github.com/janestreet/bin_prot), a binary protocol developed
by Jane Street. This provides:

- **Compact encoding** - Efficient binary representation
- **OCaml compatibility** - Interoperability with OCaml Mina node
- **Versioning** - Stable message formats across versions

## Transport support

### LibP2P

Supported channels:

- BestTipPropagation
- TransactionPropagation
- SnarkPropagation
- Rpc

Uses TCP connections with multiplexing via yamux or mplex.

### WebRTC

Supported channels:

- SignalingDiscovery
- SignalingExchange
- BestTipPropagation
- TransactionPropagation
- SnarkPropagation
- SnarkJobCommitmentPropagation
- Rpc
- StreamingRpc

Uses browser-compatible WebRTC data channels with DTLS encryption.

## Message size limits

Each channel has a maximum message size to prevent resource exhaustion:

| Channel                       | Max size | Rationale                   |
| ----------------------------- | -------- | --------------------------- |
| SignalingDiscovery            | 16 KB    | Small signaling messages    |
| SignalingExchange             | 16 KB    | Small signaling messages    |
| BestTipPropagation            | 32 MB    | Large blocks with proofs    |
| TransactionPropagation        | 1 KB     | Individual transactions     |
| SnarkPropagation              | 1 KB     | Individual SNARK references |
| SnarkJobCommitmentPropagation | 2 KB     | Job commitments             |
| Rpc                           | 256 MB   | Large data transfers        |
| StreamingRpc                  | 16 MB    | Chunked large transfers     |

## Related documentation

- [P2P Architecture](./architecture.md)
- [WebRTC Support](./webrtc.md)
- [Network Protocol](./network-protocol.md)
