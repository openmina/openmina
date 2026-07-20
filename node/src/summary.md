# Main Node State Machine

The top-level state machine that orchestrates all node operations.

## Purpose

- Coordinates all subsystems (P2P, consensus, storage, RPC)
- Manages node lifecycle and configuration
- Routes actions between state machine components
- Handles global node events and transitions

## Key Interactions

- Dispatches actions to all sub-state machines
- Aggregates state from all components
- Manages service initialization and shutdown
- Coordinates cross-component effects
