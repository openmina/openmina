# Transaction Channel State Machine

Transport-agnostic transaction propagation channel that abstracts over both
libp2p gossip and WebRTC pull-based protocols.

## Purpose

- **Transport abstraction** - Provides unified interface for transaction
  propagation over libp2p (gossip/pubsub) and WebRTC (request/response)
- **Protocol adaptation** - Handles push-based broadcasting for libp2p and
  pull-based requests for WebRTC
- **Flow control** - Implements request/response protocol for WebRTC peers with
  count-based limits
- **Network-wide propagation** - Ensures transactions reach all network
  participants regardless of transport

## State Flow

```
Disabled/Enabled → Init → Pending → Ready → (Transport-specific propagation patterns)
```

## Key Features

- **Dual transport support** - Seamlessly operates over libp2p gossip and WebRTC
  connections
- **WebRTC request protocol** - GetNext/WillSend/Transaction message flow for
  pull-based propagation
- **libp2p gossip integration** - Broadcast/subscription model for push-based
  propagation
- **Index tracking** - Maintains propagation state across different transport
  mechanisms
- **Unified channel interface** - Same API for transaction propagation
  regardless of underlying transport

## Integration Points

- **libp2p pubsub** - Broadcasts transactions via gossip protocol for libp2p
  peers
- **WebRTC data channels** - Sends transaction requests/responses over WebRTC
  connections
- **P2pChannelsEffectfulAction** - Transport-agnostic channel initialization and
  message sending
- **Transaction pool coordination** - Sources and deposits transactions from/to
  local transaction pool

## Technical Implementation

- **Transport detection** - Adapts behavior based on peer connection type
  (libp2p vs WebRTC)
- **Protocol multiplexing** - Handles both push (gossip) and pull
  (request/response) paradigms
- **State synchronization** - Coordinates transaction propagation across
  heterogeneous network
- **Channel abstraction** - Encapsulates transport-specific details behind
  unified interface

## Technical Debt

- TODO: Propagate transaction info received to transaction pool for proper
  integration
