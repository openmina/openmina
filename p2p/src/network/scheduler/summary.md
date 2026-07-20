# Network Scheduler State Machine

Manages network connections and protocol negotiation (despite its name, it
doesn't actually schedule tasks).

## Purpose

- Manages P2P connection lifecycle and state transitions
- Coordinates protocol selection and negotiation between peers
- Handles connection establishment, maintenance, and cleanup
- Routes messages between different network protocols
- Manages connection limits and resource allocation

## Key Components

- **Connection Manager**: Tracks connection states and peer relationships
- **Protocol Coordinator**: Handles protocol selection and handshakes
- **Stream Manager**: Manages individual protocol streams within connections
- **Resource Manager**: Enforces connection limits and handles cleanup

## Interactions

- Establishes and tears down peer connections
- Coordinates protocol negotiations (Noise, Yamux, Kademlia, etc.)
- Routes messages between network protocols and higher-level channels
- Manages connection state transitions and error recovery
- Enforces network-level resource limits and quotas

## Technical Debt

This component has significant naming and architectural issues:

- **Identity Crisis**: Named "scheduler" but actually manages connections, not
  task scheduling
- **Missing Features**: Summary claims bandwidth allocation, rate limiting, and
  task scheduling but none are implemented
- **Large Reducer**: 650-line monolithic reducer handling multiple concerns
  (connection management, protocol selection, error handling)
- **Mixed Responsibilities**: Single component handles too many different
  network concerns
- **TODO Comments**: Multiple incomplete features (connection state handling,
  error logging, async DNS resolution)
- **Documentation Mismatch**: Summary describes functionality that doesn't exist

The component should be renamed to "Network Connection Manager" and either
implement the promised scheduling features or update documentation to reflect
actual functionality.
