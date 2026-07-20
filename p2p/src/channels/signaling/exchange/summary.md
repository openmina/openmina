# Signaling Exchange Channel State Machine

Relays encrypted WebRTC signaling messages between peers to establish direct
connections through intermediary signaling servers.

## Purpose

- **Encrypted offer/answer relay** - Routes encrypted SDP offers and answers
  between connecting peers
- **Signaling server coordination** - Uses intermediate peers as signaling
  relays for WebRTC connection setup
- **Connection request management** - Handles GetNext requests to receive
  pending connection offers
- **Bidirectional state tracking** - Manages both local (outgoing) and remote
  (incoming) signaling flows

## State Flow

```
Disabled/Enabled → Init → Pending → Ready → (RequestSend ↔ RequestReceived)
                                         ↘ (OfferSend → Offered → Answered)
```

## Key Features

- **GetNext protocol** - Requests pending connection offers from signaling
  relays
- **Encrypted message handling** - Processes `EncryptedOffer` and
  `EncryptedAnswer` messages
- **Offer relay service** - Acts as signaling server for other peers attempting
  connections
- **Answer coordination** - Handles optional encrypted answers (can be None for
  rejection)

## Integration Points

- **P2pChannelsEffectfulAction::InitChannel** - Initializes the signaling
  exchange channel
- **P2pChannelsEffectfulAction::MessageSend** - Sends GetNext, OfferToYou, and
  Answer messages
- **P2pConnectionIncomingAction::Init** - Initiates incoming WebRTC connections
  from received offers
- **P2pChannelsSignalingDiscoveryAction** - Coordinates with discovery channel
  for peer advertisement

## Technical Implementation

- **Dual state tracking** - Separate local/remote state machines within Ready
  state
- **Three-message protocol** - GetNext → OfferToYou → Answer message flow
- **Public key tracking** - Associates offers with offerer public keys for
  authentication
- **Optional answer handling** - Supports None answers for connection rejection
