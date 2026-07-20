# Identify Stream State Machine

Manages individual libp2p identify protocol streams for peer capability
discovery and address exchange.

## Purpose

- **Bidirectional stream management** - Handles both incoming (send identify)
  and outgoing (receive identify) streams
- **Protocol message exchange** - Serializes and deserializes protobuf identify
  messages with length prefixing
- **Chunked data handling** - Reassembles partial messages across multiple data
  frames
- **Peer information propagation** - Updates peer metadata based on received
  identify information

## State Flow

```
Default → RecvIdentify → IdentifyReceived (outgoing streams)
Default → SendIdentify → closed (incoming streams)
Default → IncomingPartialData → IdentifyReceived (chunked messages)
```

## Key Features

- **Message Chunking** - Handles partial data reception via
  `IncomingPartialData` state with incremental reassembly
- **Size Validation** - Enforces identify message size limits to prevent
  resource exhaustion attacks
- **Stream Direction Logic** - Incoming streams send identify info, outgoing
  streams receive it
- **Automatic Cleanup** - Dispatches stream closure and pruning actions after
  successful exchange

## Integration Points

- **P2pIdentifyAction::UpdatePeerInformation** - Propagates peer capabilities
  and addresses to identify component
- **P2pNetworkYamuxAction::OutgoingData** - Sends serialized identify messages
  via yamux multiplexer
- **P2pNetworkSchedulerAction::Error** - Reports stream errors to connection
  scheduler
- **P2pNetworkIdentifyStreamEffectfulAction::GetListenAddresses** - Retrieves
  local addresses for outbound identify messages

## Technical Implementation

- **Length-delimited protobuf encoding** - Uses varint32 length prefix for
  message framing
- **Memory-safe chunking** - Buffers partial data in `Vec<u8>` until complete
  message received
- **Error propagation** - Converts protobuf decode errors to
  `P2pNetworkStreamProtobufError`

## Technical Debt

- TODO: Enabling conditions not implemented (`is_enabled` always returns `true`)
- TODO: Error state handling incomplete
- TODO: External address configuration hardcoded
- TODO: Observed address reporting not implemented
