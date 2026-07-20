# Ledger Write State Machine

Manages sequential write operations and state updates to the ledger.

## Purpose

- Applies blocks to staged ledgers with transaction validation
- Creates staged ledger diffs for block production
- Reconstructs staged ledgers from auxiliary data during sync
- Commits ledger state and manages mask lifecycle

## Key Operations

- **Block Application**: Applies transactions and updates account states
- **Diff Creation**: Generates staged ledger diffs for block production
- **Reconstruction**: Rebuilds staged ledgers from scan state and pending
  coinbases
- **Commit**: Finalizes ledger state and prunes old masks

## Interactions

- **Transition Frontier**: Coordinates block application and sync operations
- **Block Producer**: Provides staged ledger diffs for new blocks
- **Service Layer**: Routes operations through LedgerManager for actual ledger
  manipulation
- **Archive Integration**: Provides additional data for archive nodes

## Technical Debt

- Heavy coupling with transition frontier sync makes testing difficult
- Mask leak detection is unreliable during testing scenarios
