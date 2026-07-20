# Streaming RPC Channel State Machine

Pull-based P2P specific streaming channel for progressive staged ledger
synchronization in web nodes.

## Purpose

- **Pull-based P2P only** - Designed specifically for pull-based P2P layer, not
  implemented for libp2p
- **Staged ledger synchronization** - Enables web nodes to progressively
  download staged ledger data
- **Progressive data transfer** - Handles large ledger state through incremental
  streaming with flow control
- **Web node support** - Specialized for browser-based nodes that need efficient
  ledger sync

## State Flow

```
Disabled/Enabled → Init → Pending → Ready → (WaitingForRequest ↔ Requested → Responded)
```

## Key Features

- **Progressive streaming** - Request/Response/Next message flow for incremental
  data transfer
- **Staged ledger specialization** - Dedicated support for ledger parts
  streaming during sync
- **Progress monitoring** - Tracks upload/download progress with receive/send
  progress states
- **Flow control** - Uses Next messages to control data flow rate and prevent
  overwhelming
- **Web node optimization** - Tailored for browser environments with limited
  resources

## Integration Points

- **Pull-based P2P data channels** - Chunked streaming over pull-based P2P
  connections only
- **Staged ledger sync** - Primary integration with ledger synchronization for
  web nodes
- **Progress tracking** - Provides sync progress feedback for user interfaces
- **P2pChannelsEffectfulAction** - Channel initialization and message routing

## Technical Implementation

- **Pull-based P2P specific** - Only operates over pull-based P2P connections,
  not libp2p
- **Chunked streaming** - Uses `Next` messages to request subsequent data chunks
- **Progress state tracking** - Maintains receive/send progress for long-running
  transfers
- **Staged ledger focus** - Specialized for efficient ledger synchronization in
  resource-constrained environments

## Technical Debt

- TODO: Use configuration system instead of hard-coded values in RPC handling
- TODO: Complete error handling implementations for some error paths
