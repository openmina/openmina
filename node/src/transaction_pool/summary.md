# Transaction Pool State Machine

Manages the mempool of pending transactions using a two-layer architecture with
significant technical debt.

## Purpose

- Collects user transactions from P2P network and RPC
- Validates transaction signatures and balances via SNARK verification
- Maintains transaction ordering and priorities by fee
- Provides ordered transactions for block production
- Handles transaction propagation across the network

## Architecture

### Two-Layer Design

1. **Candidates Layer** (`TransactionPoolCandidatesState`) - manages incoming
   transactions from P2P peers
   - Tracks per-peer transaction states (info received → fetch pending →
     received → verify pending)
   - Prioritizes pubsub messages over direct peer requests
   - Coordinates transaction fetching from peers
2. **Main Pool Layer** (`TransactionPoolState`) - contains the actual
   transaction pool
   - Uses `ledger::transaction_pool::TransactionPool` for core logic
   - Maintains `DistributedPool` for P2P propagation
   - Handles multi-step verification and application flows

### Multi-Step Transaction Flows

Complex flows requiring account fetching from ledger service:

- **Verification**: `StartVerify` → fetch accounts → `StartVerifyWithAccounts` →
  SNARK verification → `VerifySuccess` → `ApplyVerifiedDiff`
- **Application**: `ApplyVerifiedDiff` → fetch accounts →
  `ApplyVerifiedDiffWithAccounts` → apply to pool
- **Best Tip Changes**: `BestTipChanged` → fetch accounts → revalidate existing
  transactions
- **Transition Frontier**: Handle blockchain state changes affecting transaction
  validity

## Implementation Note

The core transaction pool logic (validation, ordering, diff application) is
implemented in the `ledger` crate. This state machine wraps that functionality
and integrates it with the node's event-driven architecture, but uses
non-standard patterns that complicate the integration.

## Interactions

- Receives transactions from P2P network (pubsub/direct peer) and RPC
- Fetches account states from ledger service for all validation operations
- Requests SNARK verification for transaction signatures
- Provides fee-ordered transactions to block producer via
  `CollectTransactionsByFee`
- Handles best tip changes and transition frontier diffs from blockchain state
- Broadcasts valid transactions to network peers
- Manages transaction rebroadcasting for locally generated transactions

## Technical Debt

### Major Issues Requiring Refactoring

See [transaction_pool_refactoring.md](./transaction_pool_refactoring.md) for
details:

1. **Pending Actions Anti-Pattern** - Stores actions in state instead of using
   proper state transitions, violating Redux principles
2. **Blocking Service Calls** - Synchronous ledger service calls block the state
   machine thread
3. **Global State Access** - Uses `unsafe_get_state()` to access global slot
   information
4. **Complex Multi-Step Flows** - Implicit state transitions that are hard to
   follow and test

These patterns make the component difficult to test, debug, and maintain
compared to other OpenMina components that follow standard state machine
patterns.
