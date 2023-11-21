# P2P Services
Native (Rust) nodes support P2P over the libp2p and WebRTC protocols (`webrtc_with_libp2p`), whereas the webnode (wasm) supports WebRTC only. 

## webrtc_with_libp2p 
Implements WebRTC and libp2p services for the native node.

When `P2pServiceWebrtcWithLibp2p::init` is called it will initialize the `webrtc::P2pServiceCtx` and `Libp2pService` services, returning a `P2pServiceCtx` instance.  

```
pub struct P2pServiceCtx {
    pub webrtc: super::webrtc::P2pServiceCtx,
    pub libp2p: Libp2pService,
}
```

## Libp2pService
Uses the *libp2p* crate to implement P2P communication over a transport stack composed of: **TCP -> PNet -> Noise -> Yamux**.

Over this transport the following RPCs are implemented:
- `AnswerSyncLedgerQueryV2`
- `GetAncestryV2`
- `GetBestTipV2`
- `GetStagedLedgerAuxAndPendingCoinbasesAtHashV2`
- `GetTransitionChainProofV1ForV2`
- `GetTransitionChainV2`

The service's main-loop task (using *Tokio*) handles [events](#event-handling-handle_event) coming from *libp2p*, and [commands](#command-handling-handle_cmd) coming from the *state machine*.

### Event handling (handle_event)
Events come from *libp2p* in `SwarmEvent` form and are translated into messages (of `P2pEvent` type) that are sent to the *state machine* over a *MPSC* channel.

| Incoming SwarmEvent | Resulting P2pEvent |
|-------------------|-----------------|
| `SwarmEvent::NewListenAddr` | none |
| `SwarmEvent::ListenerError` | none |
| `SwarmEvent::ListenerClosed` | none |
| `SwarmEvent::ConnectionEstablished` | `P2pEvent::Connection(P2pConnectionEvent::Finalized(..))` |
| `SwarmEvent::ConnectionClosed` | `P2pEvent::Connection(P2pConnectionEvent::Closed(..))` |
| `SwarmEvent::OutgoingConnectionError` | `P2pEvent::Connection(P2pConnectionEvent::Finalized(..))` |
| `SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(GossipsubEvent::Message{.. message}))` | See [GossipsubEvent](#GossipsubEvent) |
| `SwarmEvent::Behaviour(BehaviourEvent::Rpc((peer_id, RpcBehaviourEvent::ConnectionClosed)))` | none (TODO?) |
| `SwarmEvent::Behaviour(BehaviourEvent::Rpc((peer_id, RpcBehaviourEvent::ConnectionEstablished)))` | none (TODO?) |
| `SwarmEvent::Behaviour(BehaviourEvent::Rpc((peer_id, RpcBehaviourEvent::Stream{Received::Menu(_), stream_id})))` | none |
| `SwarmEvent::Behaviour(BehaviourEvent::Rpc((peer_id, RpcBehaviourEvent::Stream{Received::HandshakeDone, stream_id})))` | `P2pEvent::Channel(P2pChannelEvent::Received(peer_id, Ok(ChannelMsg::Rpc(P2pChannelEvent::Opened(..)))))` |
| `SwarmEvent::Behaviour(BehaviourEvent::Rpc((peer_id, RpcBehaviourEvent::Stream{Received::Query{ header: QueryHeader { tag ..} ..}, stream_id})))` | See [Request](#Request) |
| `SwarmEvent::Behaviour(BehaviourEvent::Rpc((peer_id, RpcBehaviourEvent::Stream{Received::Response{header: ResponseHeader {id ..} ..}, stream_id})))` | See [Response](#Response) |

| Incoming libp2p events (other) | Resulting P2pEvent |
|-------------------|-----------------|               
| `BehaviourEvent::Identify(..)` | `P2pEvent::Libp2pIdentify(..)` |


#### GossipsubEvent
| Incoming binprot message | Outgoing P2pEvent |
|-------------------|-----------------|
| `GossipNetMessage::NewState(block)` | `P2pEvent::Channel(P2pChannelEvent::Received(.., ChannelMsg::BestTipPropagation(..))` |
| `GossipNetMessage::SnarkPoolDiff{..}` | `P2pEvent::Channel(P2pChannelEvent::Libp2pSnarkReceived(..))` |

#### Request
The outgoing message has the form `P2pEvent::Channel(P2pChannelEvent::Received(peer_id, Ok(ChannelMsg::Rpc(RpcChannelMsg::Request(.., request)))))`, where `request` depends on the `tag` value of the `QueryHeader` of the received query.

| Input tag | request value |
|-------------------|-----------------|
| `GetBestTipV2` | `P2pRpcRequest::BestTipWithProof` |
| `GetAncestryV2` | `P2pRpcRequest::BestTipWithProof` |
| `AnswerSyncLedgerQueryV2` | `P2pRpcRequest::LedgerQuery(..)` |
| `GetStagedLedgerAuxAndPendingCoinbasesAtHashV2` | `P2pRpcRequest::StagedLedgerAuxAndPendingCoinbasesAtBlock(..)` | 
| `GetTransitionChainV2` | multiple `P2pRpcRequest::Block(..)` messages for each hash |
| `GetTransitionChainProofV1ForV2` | sends empty response over libp2p without state machine interaction |
| `GetSomeInitialPeersV1ForV2` | `P2pRpcRequest::InitialPeers` |

#### Response
The libp2p `NetworkBehaviour` implementation used by the *Libp2pService* contains extra state (`ongoing` field) to map a `(peer, msg_id)` pair to a `(tag, version)` pair.

```
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "Event")]
pub struct Behaviour<E: 'static + From<P2pEvent>> {
    pub gossipsub: gossipsub::Behaviour,
    pub rpc: RpcBehaviour,
    pub identify: identify::Behaviour,
    #[behaviour(ignore)]
    pub event_source_sender: mpsc::UnboundedSender<E>,
    // TODO(vlad9486): move maps inside `RpcBehaviour`
    // map msg_id into (tag, version)
    #[behaviour(ignore)]
    pub ongoing: BTreeMap<(PeerId, u32), (String, i32)>,
    // map from (peer, msg_id) into (stream_id, tag, version)
    //
    #[behaviour(ignore)]
    pub ongoing_incoming: BTreeMap<(PeerId, u32), (StreamId, String, i32)>,
}
```

The outgoing message has the form `P2pEvent::Channel(P2pChannelEvent::Received(peer_id, Ok(ChannelMsg::Rpc(RpcChannelMsg::Response(.., response)))))`, where `response` depends on a `tag` value obtained from from the `outgoing` map. The `peer_id` and `msg_id` (`id` from `ResponseHeader`) from the incoming message are used to find the `tag` value in this map.

| tag | response value |
|-------------------|-----------------|
| `GetBestTipV2` | `P2pRpcResponse::BestTipWithProof` |
| `AnswerSyncLedgerQueryV2` | `P2pRpcResponse::LedgerQuery` |
| `GetStagedLedgerAuxAndPendingCoinbasesAtHashV2` | `P2pRpcResponse::StagedLedgerAuxAndPendingCoinbasesAtBlock` |
| `GetTransitionChainV2` | multiple `P2pRpcResponse::Block(..)` messages for each hash |
| `GetSomeInitialPeersV1ForV2` | `P2pRpcResponse::InitialPeers(..)` |


### Command handling (handle_cmd)
The *state machine* can send over MPSC the following commands to the *Libp2pService*:
```
pub enum Cmd {
    Dial(DialOpts),
    Disconnect(PeerId),
    SendMessage(PeerId, ChannelMsg),
    SnarkBroadcast(Snark, u32),
}
```

These are handled by the `Libp2pservice::handle_cmd` (async) function which will usually perform some swarm action(s).

| Incoming Cmd | Libp2pService action(s) |
|--------------|-------------------------|
| `Cmd::Dial(maddr)` |  `swam.dial(addr)` |
| `Cmd::Disconnect(peer_id)` | `swarm.disconnect_peer_id(peer_id)` |
| `Cmd::SendMessage(peer_id, ChannelMsg::SnarkPropagation(..))` | none (unsupported) |
| `Cmd::SendMessage(peer_id, ChannelMsg::SnarkJobCommitmentPropagation(..))` | none (unsupported) |
| `Cmd::SendMessage(peer_id, ChannelMsg::BestTipPropagation(BestTipPropagationChannelMsg::GetNext))` | none (TODO?) |
| `Cmd::SendMessage(peer_id, ChannelMsg::BestTipPropagation(BestTipPropagationChannelMsg::BestTip(block)))` | `Self::gossipsub_send(swarm, &GossipNetMessage::NewState(block))` |
| `Cmd::SendMessage(peer_id, ChannelMsg::Rpc(RpcChannelMsg::Request(id, request)))` | See [RPC requests](#rpc-requests) |
| `Cmd::SendMessage(peer_id, ChannelMsg::Rpc(RpcChannelMsg::Response(id, response)))` | See [RPC responses](#rpc-response) |
| `Cmd::SendMessage(peer_id, Cmd::SnarkBroadcast(snark, nonce))` | `Self::gossipsub_send(swarm, &GossipNetMessage::SnarkPoolDiff { NetworkPoolSnarkPoolDiffVersionedStableV2::AddSolvedWork(...), nonce })` |



#### RPC requests
RPC requests are handled based on the their `P2pRpcRequest` variant, which contains the request information from the point of view of the *state machine*. These are converted by the *Libp2pService* into the wire form used by the MINA protocol (binprot encoded) and the resulting binary blob is embedded into the `bytes` field of `Command::Send`.

```
pub enum Command {
    Send { stream_id: StreamId, bytes: Vec<u8> },
    Open { outgoing_stream_id: u32 },
}
```

The `Command` type abstracts the transport-specific details away, so only the `peer_id` and `stream_id` (yamux provides multiplexing) are needed to deliver the binprot encoded messages over a libp2p connection.

The MINA P2P messages that can be binprot encoded are of the `Message<T>` type. The RPC requests in particular are of type `Query<T>`.
```
pub enum Message<T> {
    Heartbeat,
    Query(Query<T>),
    Response(Response<T>),
}
```

Summing up: we receive from the *state machine* a `P2pRpcRequest` variant, based on the variant the *Libp2pService* builds a `Query<T>` message that gets binprot serialized and delivered the correct peer/stream over the libp2p transport.

| Request variant | `Query<T>` |
|-----------------|---|
| `P2pRpcRequest::BestTipWithProof` | `Query<GetBestTipV2>` |
| `P2pRpcRequest::LedgerQuery(..)` | `Query<AnswerSyncLedgetQueryV2>` |
| `P2pRpcRequest::StagedLedgerAuxAndPendingCoinbasesAtBlock(..)` | `Query<GetStagedLedgerAuxAndPendingCoinbasesAtHashV2>` |
| `P2pRpcRequest::Block(..)` | `Query<GetTransitionChainV2>` |
| `P2pRpcRequest::Snark(..)` | none |
| `P2pRpcRequest::InitialPeers` | `Query<GetSomeInitialPeersV1ForV2>` |


#### RPC response
Based on the `Cmd::SendMessage`s `peer_id` and stream `id` information, the *Libp2pService* can find in the `ongoing_incoming` map of the [Behavior](#response) which is the RPC request that we are responding to. 

The way the RPC response is constructed and delivered over libp2p is almost identical to RPC queries, but this time we build the `Response` variant of the `Message<T>`.

| `Response<T>` (from previous request) | response data (input / state machine) | response data (output / libp2p) |
|---------------------------------------|---------------------------------------|---------------------------------|
| `Response<GetBestTipV2>` | none | none |
| `Response<GetBestTipV2>` | `P2pRpcResponse::BestTipWithProof(msg)` | `ProofCarryingDataStableV1{..}` |
| `Response<GetAncestryV2>` | none | none |
| `Response<GetAncestryV2>` | `P2pRpcResponse::BestTipWithProof(msg)` | `ProofCarryingDataWithHashV1{..}` |
| `Response<AnswerSyncLedgerQueryV2>` | none | none |
| `Response<AnswerSyncLedgerQueryV2>` | `P2pRpcResponse::LedgerQuery(msg)` | `MinaLedgerSyncLedgerAnswerStableV2` |
| `Response<GetStagedLedgerAuxAndPendingCoinbasesAtHashV2>` | none | none |
| `Response<GetStagedLedgerAuxAndPendingCoinbasesAtHashV2>` | `P2pRpcResponse::StagedLedgerAuxAndPendingCoinbasesAtBlock(msg)` | `(msg.scan_state, msg.staged_ledger_hash, msg.pending_coinbase, msg.needed_blocks)` |
| `Response<GetTransitionChainV2>` | none | none |
| `Response<GetTransitionChainV2>` | `P2pRpcResponse::Block(msg)` | `vec![MinaBlockBlockStableV2{..}]` |
| `Response<GetSomeInitialPeersV1ForV2>` | none | none |
| `Response<GetSomeInitialPeersV1ForV2>` | `P2pRpcResponse::InitialPeers(peers)` | `vec![NetworkPeerPeerStableV1{..}]` |
|  | `P2pRpcResponse::Snark(..)` | none |
