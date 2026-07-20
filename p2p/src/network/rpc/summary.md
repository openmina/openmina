# Network RPC State Machine

Low-level RPC protocol implementation for P2P communication using binprot
serialization.

## Purpose

- Implements RPC wire protocol with length-prefixed framing
- Manages request/response flow and correlation via query IDs
- Handles protocol framing and message parsing from byte streams
- Tracks RPC sessions with heartbeat and timeout mechanisms
- Provides foundation for higher-level channel RPC functionality

## Key Components

- **Message Parser**: Handles binprot deserialization and length framing
- **Request Tracker**: Manages pending queries and response correlation
- **Heartbeat Manager**: Implements keepalive mechanism for RPC sessions
- **Protocol Handler**: Routes messages between network layer and RPC channels

## Interactions

- Receives raw byte streams and parses RPC protocol messages
- Encodes and decodes RPC messages using binprot serialization
- Routes queries and responses to appropriate RPC channel handlers
- Manages session timeouts and heartbeat intervals
- Handles protocol errors and connection state recovery

## Technical Debt

This component is well-architected but has some minor maintainability issues:

- **TODO Comments**: Known limitations around heartbeat queueing behavior and
  multiple message assumptions
- **Buffer Management**: Complex parsing logic with manual offset tracking that
  could be simplified
- **Protocol Coupling**: Hard-coded protocol versions and type conversions
  throughout the code
- **Large Dispatch Functions**: `dispatch_rpc_query` and `dispatch_rpc_response`
  are lengthy and could be broken down
- **Memory Calculation**: Unimplemented malloc size calculation for monitoring

These are minor issues that don't affect functionality but could improve code
maintainability over time.
