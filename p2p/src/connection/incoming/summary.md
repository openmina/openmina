# Incoming Connection State Machine

Manages incoming WebRTC and libp2p connection establishment from offer receipt
through connection finalization.

## Purpose

- **WebRTC connection handling** - Processes incoming WebRTC offers and
  generates encrypted SDP answers
- **Dual transport support** - Handles both WebRTC (browser-based) and libp2p
  (backend) incoming connections
- **Answer generation workflow** - Creates SDP answers, encrypts them, and sends
  via signaling channels
- **Connection finalization** - Completes connection setup and transitions to
  ready state

## State Flow

```
Init → AnswerSdpCreatePending → AnswerSdpCreateSuccess → AnswerReady → AnswerSendSuccess → FinalizePending → Success
     → FinalizePendingLibp2p → Libp2pReceived (libp2p path)
     → Error (failure cases)
```

## Key Features

- **SDP answer creation** - Generates WebRTC Session Description Protocol
  answers for incoming offers
- **Signaling method support** - Handles HTTP signaling and P2P signaling
  channel routing
- **Duplicate connection handling** - Manages close_duplicates for libp2p
  connections
- **Timeout management** - Configurable timeouts for incoming connection
  establishment
- **RPC request tracking** - Associates connections with optional RPC request
  IDs

## Integration Points

- **P2pConnectionIncomingEffectfulAction::Init** - Initiates WebRTC answer
  creation process
- **P2pChannelsSignalingExchangeAction** - Coordinates with signaling exchange
  for answer transmission
- **P2pNetworkSchedulerAction** - Integrates with libp2p connection scheduler
- **P2pDisconnectionAction** - Handles connection failures and cleanup

## Technical Implementation

- **Encrypted answer handling** - Uses `Box<webrtc::Answer>` for encrypted SDP
  answers
- **Signaling method abstraction** - Supports `IncomingSignalingMethod::Http`
  and `::P2p`
- **Peer state coordination** - Creates and updates `P2pPeerState` with
  connection information
- **Error state management** - Comprehensive error handling with typed error
  enums

## Technical Debt

### Major Issues

- **Missing Resource Management**: Basic capacity checks but no comprehensive
  resource cleanup or bounded collections

### Moderate Issues

- **Code Duplication**: Repetitive state validation logic across actions in
  enabling conditions - opportunity to extract common patterns to state methods
- **Scattered Feature Flags**: libp2p conditional compilation logic scattered
  throughout reducer makes maintenance difficult
- **Poor Error Context**: Error handling loses context by early string
  conversion in error handling paths
- TODO: Move `IncomingSignalingMethod` to `crate::webrtc` module for better
  organization
