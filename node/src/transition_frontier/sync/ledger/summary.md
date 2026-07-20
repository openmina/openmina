# Ledger Sync State Machine

Coordinates sequential synchronization of snarked and staged ledgers during
transition frontier sync.

## Purpose

- Orchestrates multi-phase ledger synchronization process
- Manages sequential flow from snarked to staged ledger sync
- Handles target updates and sync interruptions gracefully
- Collects protocol states needed for transaction validation

## Sequential Synchronization Flow

### Phase 1: Snarked Ledger Sync

- Delegates to snarked ledger sync component for Merkle tree reconstruction
- Synchronizes the fully verified base ledger state
- Required foundation for staged ledger reconstruction

### Phase 2: Staged Ledger Sync (if needed)

- Only triggered if target includes staged ledger data
- Builds upon completed snarked ledger to reconstruct pending transactions
- Delegates to staged ledger sync component for parts fetching and
  reconstruction

## State Flow

```
Init → Snarked(NumAccountsPending → ... → Success) → Staged(PartsFetchPending → ... → Success) → Success
     ↘ (direct to Success if no staged ledger required)
```

## Target Management

- **Flexible targeting** - supports different ledger sync scenarios (staking,
  next epoch, root)
- **Target updates** - handles best tip changes during sync with intelligent
  restart logic
- **Compatibility checking** - validates that ledger hashes are compatible for
  incremental sync

## Key Features

- **Sequential coordination** - ensures snarked completes before staged begins
- **Smart restarts** - only restarts from beginning if target hashes change
  incompatibly
- **Protocol state aggregation** - collects needed protocol states from staged
  sync
- **Target validation** - ensures sync targets match expected ledger structure

## Sub-Components

- **Snarked Sync** - BFS Merkle tree reconstruction with multi-peer validation
- **Staged Sync** - Parts fetching and reconstruction with empty ledger
  optimization

## Interactions

- Coordinates between snarked and staged sync sub-components
- Reports aggregate sync progress to parent transition frontier sync
- Handles target updates from parent when best tip changes during sync
- Provides completed ledger state and protocol states to transition frontier
