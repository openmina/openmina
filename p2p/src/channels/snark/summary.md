# SNARK Channel State Machine

Transport-agnostic SNARK work distribution channel that abstracts over both
libp2p gossip and WebRTC pull-based protocols.

## Purpose

- **Transport abstraction** - Provides unified interface for SNARK work
  propagation over libp2p (gossip/pubsub) and WebRTC (request/response)
- **Protocol adaptation** - Handles push-based broadcasting for libp2p and
  pull-based requests for WebRTC
- **Flow control** - Implements request/response protocol for WebRTC peers with
  count-based limits
- **Network-wide distribution** - Ensures SNARK work reaches all network
  participants regardless of transport

## Key Components

- **Request Handler**: Manages GetNext/WillSend/Snark protocol flow
- **Distribution Manager**: Tracks work distribution state per peer
- **Broadcast Handler**: Manages work propagation via gossip
- **State Tracker**: Maintains request/response state for local and remote peers

## Interactions

- Broadcasts new SNARK work when available in pool
- Forwards work received from other peers
- Deduplicates work for libp2p transport (but not WebRTC)
- Integrates with local SNARK pool for work availability
- Handles both push (broadcast) and pull (request) distribution models

## Technical Debt

### Minor Issues

- **Inconsistent Deduplication**: WebRTC path lacks duplicate check while libp2p
  has it (different transport requirements may explain this difference)
- **State Methods**: Could benefit from additional helper methods to reduce
  pattern matching in reducer

### Note on Architecture

This channel provides a transport abstraction layer between inner logic
components and libp2p/WebRTC transports for SNARK work distribution. Validation
and security concerns are properly handled by the SNARK pool and other inner
logic components that use this abstraction.
