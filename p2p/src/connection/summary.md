# Connection State Machine

Coordinates peer connection establishment for both WebRTC and libp2p transports
through incoming and outgoing connection management.

## Purpose

- **Connection lifecycle coordination** - Manages complete connection
  establishment process from initiation to finalization
- **Dual transport abstraction** - Provides unified interface for WebRTC and
  libp2p connection types
- **State delegation** - Routes connection actions to appropriate incoming or
  outgoing state machines
- **Connection validation** - Enforces connection acceptance rules and capacity
  limits

## State Flow

```
P2pConnectionState::Outgoing(OutgoingState)
P2pConnectionState::Incoming(IncomingState)
```

## Key Features

- **Transport abstraction** - Unified connection state enum supporting both
  WebRTC and libp2p connections
- **Directional state management** - Delegates to specialized incoming/outgoing
  state machines
- **Timeout handling** - Configurable timeouts for both connection types
- **RPC integration** - Associates connections with optional RPC request
  tracking
- **Success/error detection** - Common interface for checking connection status

## Integration Points

- **P2pConnectionOutgoingAction** - Delegates to outgoing connection state
  machine
- **P2pConnectionIncomingAction** - Delegates to incoming connection state
  machine
- **P2pPeerState** - Updates peer connection status and metadata
- **P2pTimeouts** - Applies configurable timeout policies

## Technical Implementation

- **Enum-based state delegation** - Uses `P2pConnectionState` enum to route to
  appropriate handler
- **Common interface methods** - Provides `is_success()`, `is_error()`,
  `rpc_id()`, and `time()` accessors
- **Timeout coordination** - Delegates timeout checking to specific connection
  types
- **State machine composition** - Composes incoming and outgoing state machines
  into unified interface
