# Signaling Discovery Channel State Machine

Facilitates WebRTC peer discovery by exchanging information about available
WebRTC-capable peers in the network.

## Purpose

- **Peer capability discovery** - Requests and shares information about
  WebRTC-capable peers
- **Connection bootstrapping** - Enables discovery of peers that can accept
  WebRTC connections
- **Rate-limited discovery** - Manages discovery request frequency to prevent
  spam
- **Bidirectional state tracking** - Tracks both local (outgoing) and remote
  (incoming) discovery flows

## State Flow

```
Disabled/Enabled → Init → Pending → Ready → (RequestSend ↔ DiscoveryRequestReceived)
                                         ↘ (DiscoveredSend → DiscoveredAccepted/Rejected)
```

## Key Features

- **GetNext message protocol** - Sends `GetNext` requests to discover
  WebRTC-capable peers
- **Discovered message responses** - Responds with `Discovered` messages
  containing peer public keys
- **Rate limiting** - Enforces 60-second minimum interval between discovery
  requests
- **Accept/reject handling** - Manages responses to discovery offers from other
  peers

## Integration Points

- **P2pChannelsEffectfulAction::InitChannel** - Initializes the discovery
  channel
- **P2pChannelsEffectfulAction::MessageSend** - Sends GetNext and Discovered
  messages
- **webrtc_discovery_respond_with_availble_peers** - Generates responses with
  available WebRTC peers
- **P2pChannelsSignalingExchangeAction** - Coordinates with exchange channel for
  connection setup

## Technical Implementation

- **Dual state tracking** - Separate local/remote state machines within Ready
  state
- **Message-based protocol** - Uses `SignalingDiscoveryChannelMsg::GetNext` and
  `::Discovered`
- **Time-based rate limiting** - Checks elapsed time since last request in
  enabling conditions
- **Public key exchange** - Shares target peer public keys for WebRTC connection
  establishment

## Technical Debt

- TODO: Make 60-second discovery interval configurable in `RequestSend` enabling
  conditions
- TODO: Add interval constraints between incoming discovery requests to prevent
  spam
- TODO: Implement custom error handling instead of generic errors in discovery
  response handling
- TODO: Address potential enabling condition issues in discovery message
  processing
