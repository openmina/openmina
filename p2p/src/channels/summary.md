# P2P Channels State Machine

Provides transport abstraction layer for Mina-specific protocols over dual P2P
transports.

## Purpose

- Abstracts communication over both libp2p and WebRTC transports
- Implements Mina-specific protocols with transport-agnostic interfaces
- Handles protocol adaptation between push-based (libp2p) and pull-based
  (WebRTC) paradigms
- Manages message serialization, routing, and validation across transport layers

## Transport Abstraction

- **Unified Interface**: Single API for both libp2p gossip/pubsub and WebRTC
  request/response
- **Protocol Adaptation**: Automatically adapts between push-based broadcasting
  and pull-based requests
- **Transport Detection**: Routes messages based on peer connection type and
  capabilities
- **Message Size Management**: Handles different size limits per channel (1KB to
  256MB)

## Key Channels

- **Best Tip**: Propagates chain head information and blockchain state updates
- **RPC**: Handles peer-to-peer RPC calls with request correlation
- **SNARK**: Distributes SNARK work assignments and proof submissions
- **Transaction**: Propagates pending transactions across the network
- **SNARK Job Commitment**: Manages SNARK work commitments (legacy, being phased
  out)
- **Signaling Discovery/Exchange**: WebRTC connection establishment and peer
  discovery
- **Streaming RPC**: Long-lived data streams for large responses (ledger sync,
  etc.)

## Architecture

- **State Machine Pattern**: Consistent Disabled → Enabled → Init → Pending →
  Ready flow
- **Effectful Actions**: Transport-agnostic operations dispatched to appropriate
  services
- **Bidirectional Tracking**: Separate local/remote state management within
  channels
- **Service Abstraction**: Clean separation between channel logic and transport
  implementation

## Interactions

- Multiplexes over P2P connections with channel identification
- Routes messages to appropriate business logic handlers (transaction pool,
  SNARK pool, etc.)
- Coordinates with network layer for connection management and transport
  capabilities
- Manages protocol versioning and backward compatibility
- Handles message validation, rate limiting, and error recovery

## Integration Points

- **Business Logic**: Connects to transaction pool, SNARK pool, blockchain state
  management
- **Transport Layer**: Interfaces with libp2p pubsub and WebRTC data channels
- **Connection Management**: Coordinates with P2P connection lifecycle and peer
  discovery
