# Transition Frontier State Machine

Manages the blockchain's transition frontier through multi-component
coordination for genesis initialization, candidate evaluation, synchronization,
and chain maintenance.

## Purpose

- Maintains the active blockchain state from root to best tip
- Orchestrates complex synchronization with network peers through multi-phase
  process
- Evaluates incoming block candidates using consensus-based ordering
- Handles chain reorganizations and fork decisions
- Manages genesis block creation and chain initialization
- Provides chain diffs for transaction pool updates

## Architecture Overview

The transition frontier coordinates several components in a hierarchical state
machine:

### Core State Components

- **Best Chain**: Vector of applied blocks from transition frontier root to
  current best tip
- **Protocol States**: Cached protocol states needed for scan state operations
- **Chain Diff**: Transaction pool updates when chain changes
- **Blacklist**: Invalid blocks that failed application after SNARK verification

### Sub-Component State Machines

#### 1. Genesis Component

Handles genesis block creation and proving:

- Loads genesis configuration from external sources
- Produces genesis blocks with protocol state structure
- Generates real proofs for block producers, dummy proofs for others
- Provides foundation for chain initialization

#### 2. Candidates Component

Manages incoming block candidates with consensus-based ordering:

- Multi-stage validation pipeline (Received → Prevalidated → SnarkVerifyPending
  → SnarkVerifySuccess)
- Consensus-ordered priority queue using `consensus_take` function
- Chain proof management for fork validation
- Invalid block blacklisting and memory optimization

#### 3. Sync Component

Orchestrates blockchain synchronization:

- **Phase 1 (Bootstrap)**: Sequential ledger sync (Staking → NextEpoch → Root)
- **Phase 2 (Catchup)**: Block fetching and sequential application
- **Phase 3 (Commit)**: Chain commitment and finalization
- Multi-peer resilience with retry logic and error recovery

#### 4. Ledger Sync Sub-Component (within Sync)

Coordinates snarked and staged ledger synchronization:

- **Snarked Sync**: BFS Merkle tree reconstruction with optimized account
  fetching
- **Staged Sync**: Parts fetching and reconstruction with empty ledger
  optimization
- Sequential coordination ensuring snarked completion before staged begins

## Synchronization Process

### Bootstrap Phase (Ledger Synchronization)

```
StakingLedgerSync → NextEpochLedgerSync → RootLedgerSync
```

Each ledger sync uses multi-phase algorithms:

- **Snarked ledgers**: NumAccounts query → BFS Merkle tree sync → Success
- **Staged ledgers**: Parts fetching → Multi-peer validation → Reconstruction →
  Success

### Catchup Phase (Block Synchronization)

```
BlocksPending → BlocksSuccess → CommitPending → CommitSuccess → Synced
```

- Parallel multi-peer block fetching with retry logic
- Sequential block application maintaining order
- Root snarked ledger update tracking for proper reconstruction

## Key Features

- **Multi-phase sync strategy** - bootstrap → catchup → commit process
- **Consensus-based candidate ordering** - maintains worst-to-best candidate
  priority queue
- **Chain proof management** - supports fork validation and consensus decisions
- **Protocol state caching** - optimizes scan state operations with needed
  protocol states
- **Transaction pool integration** - provides chain diffs for efficient pool
  updates
- **Memory optimization** - prunes candidates and invalid blocks based on
  consensus rules

## Integration Points

- **P2P Network**: Receives blocks and handles multi-peer synchronization
- **SNARK Verification**: Coordinates block and transaction proof verification
- **Transaction Pool**: Provides chain diffs for pool updates and revalidation
- **Block Production**: Supplies genesis proofs and current chain state
- **Ledger Service**: Delegates heavy computation for ledger operations

## Technical Debt

This component uses the old-style state machine pattern and requires significant
refactoring:

- **Old Architecture**: Uses separate reducer and effects files instead of
  unified reducers
- **Direct State Access**: Effects directly access state via `state.get()` and
  `store.state()`
- **Service Interactions**: Service calls not properly abstracted through thin
  effectful actions
- **Error Handling**: Multiple TODO comments indicate missing error propagation
- **Refactoring TODOs**: Several inline comments indicate code that needs to be
  moved to proper locations

The migration to new-style patterns would involve:

1. Merging effects logic into unified reducers
2. Using proper substate contexts for state access
3. Converting service interactions to thin effectful actions
4. Implementing proper error handling throughout
5. Moving misplaced logic to appropriate modules (especially sync-related code)
