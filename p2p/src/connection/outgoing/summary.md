# Outgoing Connection State Machine

Manages outgoing WebRTC and libp2p connection establishment from offer creation
through connection finalization.

## Purpose

- **WebRTC connection initiation** - Creates and sends encrypted SDP offers to
  target peers
- **Dual transport support** - Handles both WebRTC (browser-based) and libp2p
  (backend) outgoing connections
- **Offer creation workflow** - Generates SDP offers, encrypts them, and sends
  via signaling channels
- **Connection completion** - Processes answers and finalizes connection
  establishment

## State Flow

```
Init → OfferSdpCreatePending → OfferSdpCreateSuccess → OfferReady → OfferSendSuccess → AnswerRecvPending → AnswerRecvSuccess → FinalizePending → Success
     → Error (failure cases)
```

## Key Features

- **SDP offer creation** - Generates WebRTC Session Description Protocol offers
  for outgoing connections
- **Signaling method coordination** - Routes offers through HTTP signaling or
  P2P signaling channels
- **Answer processing** - Receives and processes encrypted SDP answers from
  target peers
- **Callback support** - Executes success callbacks with peer ID and RPC ID upon
  connection establishment
- **Timeout management** - Configurable timeouts for outgoing connection
  attempts

## Integration Points

- **P2pConnectionOutgoingEffectfulAction::Init** - Initiates WebRTC offer
  creation process
- **P2pNetworkSchedulerAction::OutgoingConnect** - Integrates with libp2p
  connection scheduler
- **Signaling channels** - Coordinates with discovery and exchange channels for
  offer/answer routing
- **P2pPeerAction** - Updates peer state upon successful connection
  establishment

## Technical Implementation

- **Encrypted offer handling** - Uses `Box<webrtc::Offer>` for encrypted SDP
  offers
- **Callback mechanism** - Redux callbacks for decoupled success notification
- **Connection options** - Supports `P2pConnectionOutgoingInitOpts::WebRTC` and
  `::LibP2P`
- **Error state management** - Comprehensive error handling with rejection
  reasons

## Technical Debt

### Critical Issues

- **Missing DNS Resolution**: libp2p connections lack DNS resolution
  implementation in `Init` and `Reconnect` actions, causing connection failures
  for domain-based addresses

### Major Issues

- **Complex State Machine**: Large monolithic reducer with multiple
  responsibilities makes maintenance difficult
- **Insufficient Input Validation**: String parsing for peer addresses lacks
  robust validation in address conversion

### Moderate Issues

- **Resource Management**: Extensive use of `Box<>` allocations without clear
  cleanup patterns may cause memory leaks
- **Incomplete Timeout Logic**: Timeout handling only applies to non-error
  states in timeout checking
- **Basic Error Mapping**: Simplistic error conversion loses debugging detail in
  error handling
- TODO: Replace hard-coded signaling server host with actual address (WebRTC
  offers currently use 127.0.0.1)
- TODO: Remove host field from offers and use ICE candidates instead for
  signaling server identification
- TODO: Rename `Init` and `Reconnect` actions to `New` and `Connect` for clearer
  semantics
- TODO: Move outgoing connection types to `crate::webrtc` module for better
  organization
