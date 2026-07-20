# Kademlia Stream State Machine

Manages bidirectional Kademlia protocol streams for FIND_NODE request/response
message exchange.

## Purpose

- **Bidirectional stream management** - Handles both incoming (server) and
  outgoing (client) Kademlia streams
- **Length-delimited message handling** - Processes varint32-prefixed protobuf
  messages with chunking support
- **FIND_NODE protocol implementation** - Handles key lookup requests and
  closest peer responses
- **Stream lifecycle coordination** - Manages stream states from creation
  through closure

## State Flow

**Incoming Streams (Server):**

```
Default → WaitingForRequest → PartialRequestReceived → RequestIsReady → WaitingForReply → ResponseBytesAreReady → Closing → Closed
```

**Outgoing Streams (Client):**

```
Default → WaitingForRequest → RequestBytesAreReady → WaitingForReply → PartialReplyReceived → ResponseIsReady → Closing → Closed
```

## Key Features

- **Message chunking** - Handles partial message reception via dedicated partial
  states with incremental reassembly
- **Size validation** - Enforces Kademlia message size limits to prevent
  resource exhaustion
- **Protobuf serialization** - Uses quick_protobuf for message encoding/decoding
  with error handling
- **Directional state separation** - Distinct state machines for incoming vs
  outgoing stream handling

## Integration Points

- **P2pNetworkKademliaStreamAction::WaitOutgoing** - Triggers response
  generation for incoming FIND_NODE requests
- **P2pNetworkKadRequestAction::ReplyReceived** - Processes FIND_NODE responses
  from outgoing streams
- **P2pNetworkYamuxAction::OutgoingData** - Sends serialized messages and FIN
  flags via yamux
- **P2pNetworkSchedulerAction::Error** - Reports stream errors to connection
  scheduler

## Technical Implementation

- **Varint32 length prefixing** - Uses protobuf varint32 encoding for message
  length headers
- **Incremental parsing** - Buffers partial data until complete messages are
  received
- **Error state handling** - Converts protobuf and parsing errors to stream
  error states
- **Stream direction awareness** - Different message flow patterns for client vs
  server streams

## Technical Debt

- TODO: Use enum for errors instead of string-based error handling
