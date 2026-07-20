# Best Tip Channel State Machine

Transport-agnostic best blockchain tip exchange channel that abstracts over both
libp2p gossip and WebRTC request protocols.

## Purpose

- **Transport abstraction** - Provides unified interface for best tip
  propagation over libp2p (gossip/pubsub) and WebRTC (request/response)
- **Protocol adaptation** - Handles push-based broadcasting for libp2p and
  pull-based requests for WebRTC
- **Consensus coordination** - Helps network converge on canonical best chain
  through tip sharing
- **Sync decision support** - Enables informed sync decisions based on peer tip
  quality

## State Flow

```
Disabled/Enabled → Init → Pending → Ready → (WaitingForRequest ↔ Requested → Responded)
```

## Key Features

- **Dual transport support** - Seamlessly operates over libp2p gossip and WebRTC
  connections
- **Simple request protocol** - GetNext/BestTip message flow for single block
  exchange
- **Tip tracking** - Maintains last sent/received tip information per peer
- **Bidirectional state management** - Separate local/remote state machines
  within Ready state

## Integration Points

- **libp2p pubsub** - Broadcasts best tips via gossip protocol for libp2p peers
- **WebRTC data channels** - Sends tip requests/responses over WebRTC
  connections
- **P2pChannelsEffectfulAction** - Transport-agnostic channel initialization and
  message sending
- **Transition frontier coordination** - Sources and evaluates tips from/to
  blockchain state

## Technical Implementation

- **Transport detection** - Adapts behavior based on peer connection type
  (libp2p vs WebRTC)
- **Block exchange** - Handles `ArcBlock` serialization and transmission
- **State synchronization** - Coordinates tip propagation across heterogeneous
  network
- **Channel abstraction** - Encapsulates transport-specific details behind
  unified interface

## Technical Debt

### Minor Issues

- **TODO Comments**: Some incomplete functionality noted in comments regarding
  tip comparison and consensus enforcement logic
- **State Methods**: Could benefit from additional helper methods to reduce
  pattern matching in reducer

### Note on Architecture

This channel provides an abstraction layer between inner logic components and
libp2p/WebRTC transports. The channel is functioning correctly within its
intended scope - validation and consensus logic properly belong in the inner
logic components that use this abstraction.
