# PubSub State Machine

Implements gossip protocol for message broadcasting across the P2P network.

## Purpose

- Manages topic subscriptions for blockchain data
- Routes messages to subscribers using mesh topology
- Implements flood-fill gossip with deduplication
- Handles message validation and caching

## Key Components

- **Message Cache**: Stores and manages message lifecycle and validation states
- **Mesh Manager**: Maintains peer topology for efficient gossip propagation
- **Subscription Manager**: Handles topic subscriptions and peer interests
- **Message Validator**: Validates incoming messages and handles routing

## Interactions

- Subscribes to blockchain topics (blocks, transactions, SNARKs)
- Broadcasts blocks and transactions to network
- Forwards messages from peers based on subscriptions
- Manages gossip mesh topology and peer relationships
- Handles message deduplication and validation

## Technical Debt

This component has significant complexity and performance issues. See
[p2p_network_pubsub_refactoring.md](./p2p_network_pubsub_refactoring.md) for
detailed analysis.

### Major Issues

- **Massive Reducer (963 lines)**: Single file handling multiple concerns that
  should be moved to state methods for better maintainability
- **Performance Problems**: O(n) message lookups (state.rs:395-406) and
  unbounded memory growth
- **Mixed Responsibilities**: State struct handles caching, peer management, and
  protocol logic simultaneously

### Moderate Issues

- **Incomplete Functionality**: Missing source tracking for messages
  (reducer.rs:300), platform compatibility concerns (state.rs:253)
- **Hard-coded Constants**: Non-configurable timeouts (5s, 300s) and magic
  numbers (3, 10, 50, 100) scattered throughout
- **Suboptimal Data Structures**: TODO to separate storage by message type
  (state.rs:214) would improve efficiency

### Refactoring Plan

1. **Move message handling logic to state methods** to reduce reducer complexity
2. **Implement proper indexing** to eliminate O(n) lookups
3. **Extract separate managers** for caching, peer management, and subscriptions
4. **Make constants configurable** through P2P configuration system
