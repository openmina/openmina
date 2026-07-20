# Kademlia Request State Machine

Manages individual FIND_NODE request lifecycle from connection establishment
through response processing.

## Purpose

- **Connection-aware request handling** - Manages connection state before
  issuing FIND_NODE queries
- **Multi-phase state tracking** - Tracks progression from connection to stream
  creation to response
- **Protobuf message serialization** - Handles encoding/decoding of Kademlia
  protocol messages
- **Peer discovery integration** - Processes responses to populate routing table
  and bootstrap

## State Flow

```
New → WaitingForConnection → WaitingForKadStream → Request → WaitingForReply → Reply
    ↘ Disconnected                                     ↘ Error
```

## Key Features

- **Connection lifecycle management** - Handles peer connection establishment
  before request dispatch
- **Stream multiplexing coordination** - Waits for yamux multiplexing before
  opening Kademlia streams
- **Callback-based integration** - Uses Redux callbacks for decoupled response
  handling
- **Bootstrap integration** - Automatically notifies bootstrap component of
  request completion
- **Automatic cleanup** - Prunes completed requests and closes streams

## Integration Points

- **P2pConnectionOutgoingAction::Init** - Initiates connections to target peers
  with success callbacks
- **P2pNetworkYamuxAction::OpenStream** - Opens Kademlia protocol streams over
  established connections
- **P2pNetworkKadBootstrapAction::RequestDone/RequestError** - Reports bootstrap
  request results
- **P2pNetworkKadEffectfulAction::Discovered** - Processes discovered peer
  addresses from responses
- **P2pNetworkKademliaStreamAction::Close** - Closes streams after successful
  response processing

## Technical Implementation

- **State-driven connection handling** - Different logic based on peer
  connection state (none, connecting, ready)
- **Stream ID management** - Coordinates with yamux for stream allocation and
  lifecycle
- **Protobuf serialization** - Uses quick_protobuf for message encoding with
  error handling
- **Peer filtering** - Supports local address filtering for discovered peers

## Technical Debt

- TODO: Add callbacks for stream operations
- TODO: Error handling for invalid request keys needs improvement
