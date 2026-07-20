# Snarked Ledger Sync State Machine

Synchronizes the fully verified (snarked) ledger using a two-phase BFS Merkle
tree reconstruction algorithm.

## Purpose

- Downloads snarked ledger from peers using optimized multi-phase approach
- Verifies Merkle tree integrity with hash consistency checks
- Reconstructs verified ledger state from fetched components
- Provides progress tracking and error recovery with peer retry logic

## Two-Phase Synchronization Process

### Phase 1: NumAccounts Query

- Queries peers for total account count and content hash
- Validates responses from multiple peers for consistency
- Establishes the scope and root hash for Merkle tree sync

### Phase 2: BFS Merkle Tree Sync

- Uses breadth-first search to traverse the Merkle tree
- Fetches child hashes for internal nodes (depth <
  `LEDGER_DEPTH - ACCOUNT_SUBTREE_HEIGHT`)
- Optimized account fetching at subtree level (`ACCOUNT_SUBTREE_HEIGHT = 6`)
- Fetches up to 64 accounts per request when reaching account subtrees

## Key Features

- **Multi-peer retry logic** - tracks per-peer RPC states with error recovery
- **Progress estimation** - provides detailed sync progress based on tree
  structure
- **Address-based querying** - systematically fetches tree components by ledger
  address
- **Peer availability checking** - validates peers have required ledger data
  before querying
- **Hash validation** - verifies all received hashes match expected Merkle tree
  structure

## State Flow

```
NumAccountsPending → NumAccountsSuccess → MerkleTreeSyncPending → MerkleTreeSyncSuccess → Success
```

## Interactions

- Requests account counts and Merkle tree data via P2P RPC
- Validates all received hashes against expected tree structure
- Coordinates with ledger service for tree reconstruction
- Provides progress updates for UI/monitoring
- Integrates with peer management for retry logic
