# RPC Channel State Machine

Transport-agnostic RPC communication channel that abstracts blockchain data
requests/responses over both libp2p and WebRTC.

## Purpose

- **Transport abstraction** - Provides unified RPC interface over libp2p (native
  RPC) and WebRTC (data channels)
- **Request/response coordination** - Manages bidirectional blockchain data
  exchanges with ID tracking
- **Transport-aware routing** - Routes requests based on transport capabilities
  (some RPCs WebRTC-only)
- **Timeout management** - Implements configurable timeouts per RPC type across
  transports

## State Flow

```
Disabled/Enabled → Init → Pending → Ready → (WaitingForRequest ↔ Requested → Responded)
```

## Key Features

- **Dual transport support** - Seamlessly operates over libp2p RPC streams and
  WebRTC data channels
- **RPC type awareness** - Different RPC types (BestTip, Ledger, Block, Snark,
  etc.) with transport-specific support
- **ID-based correlation** - Matches requests to responses using `P2pRpcId`
  across async operations
- **Concurrent request handling** - Manages multiple pending requests per peer
  with flow control
- **Timeout coordination** - Per-RPC-type timeouts adapted to transport
  characteristics

## Integration Points

- **libp2p RPC streams** - Native request/response over libp2p protocol streams
- **WebRTC data channels** - RPC message serialization over WebRTC connections
- **Blockchain services** - Routes to ledger, block store, SNARK pool,
  transaction pool
- **P2pChannelsEffectfulAction** - Transport-agnostic RPC initialization and
  message routing

## Technical Implementation

- **Transport detection** - Uses `supported_by_libp2p()` to route requests
  appropriately
- **Request queuing** - `VecDeque` for managing concurrent remote requests per
  peer
- **Response correlation** - ID-based matching of async responses to original
  requests
- **Channel abstraction** - Encapsulates libp2p vs WebRTC RPC differences behind
  unified interface

## Technical Debt

### Moderate Issues

- **Incomplete Request/Response Validation**: TODO comments indicate missing
  validation for matching requests to responses

### Minor Issues

- **Hard-coded Concurrency Limits**: Maximum of 5 concurrent remote requests per
  peer is not configurable
- **No Remote Request Cleanup**: Remote requests lack timeout mechanism and are
  only removed on ResponseSend. However, this is not a significant issue since:
  - Each peer has its own isolated channel
  - Maximum 5 requests at ~160 bytes each = ~800 bytes per peer
  - A peer that doesn't respond only blocks their own channel, not affecting
    other peers
  - **Suggested improvement**: Add a static assertion to verify request size
    remains small, and document in code why cleanup isn't critical
