# Yamux State Machine

Implements Yamux stream multiplexing protocol for the P2P network layer.

## Purpose

- Multiplexes multiple streams over a single connection
- Manages stream lifecycle (creation, establishment, closure)
- Handles flow control with window-based backpressure
- Provides stream isolation and data routing
- Implements frame parsing and buffering

## Key Components

- **Frame Parser**: Parses incoming Yamux protocol frames
- **Stream Manager**: Tracks per-stream state and flow control
- **Buffer Management**: Handles incoming data buffering and optimization
- **Flow Controller**: Manages window sizes and backpressure

## Interactions

- Receives raw data from transport layer
- Parses Yamux frames from buffered data
- Creates and manages substreams
- Routes stream data to appropriate handlers
- Manages window sizes and flow control
- Handles stream closure and cleanup

## Technical Debt

This component has significant complexity issues. See
[p2p_network_yamux_refactoring.md](./p2p_network_yamux_refactoring.md) for
details on:

- **Reducer Complexity**: 387-line reducer with deep nesting (4-5 levels)
- **State Management**: Complex boolean flag combinations that represent the
  yamux protocol but need better documentation for clarity
- **Buffer Management**: Complex optimization logic mixing performance and
  correctness concerns
- **Flow Control**: Scattered window management using saturating arithmetic
- **Error Handling**: Nested error types making error handling complex

**Ongoing Work**: PR #1085 (`tweaks/yamux` branch) contains 9 commits with
refactoring to address these issues, including action splitting, state method
extraction, and comprehensive testing.
