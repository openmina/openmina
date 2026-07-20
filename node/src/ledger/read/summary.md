# Ledger Read State Machine

Manages concurrent read operations on ledgers with cost-based throttling to
prevent service overload.

## Purpose

- Processes ledger read requests with cost limiting (MAX_TOTAL_COST = 256)
- Manages concurrent read access through PendingRequests system
- Provides account lookups and merkle proofs
- Handles request deduplication to avoid redundant work

## Key Operations

- Account queries and delegator tables
- Merkle tree traversal (num accounts, child hashes, account contents)
- Staged ledger auxiliary data and pending coinbases
- Scan state summaries for RPC

## Interactions

- **RPC Integration**: Serves account queries, ledger status, and scan state
  summaries
- **P2P Integration**: Responds to ledger sync queries and staged ledger part
  requests
- **VRF Integration**: Constructs delegator tables for block production
- **Service Integration**: Routes requests through LedgerManager with cost
  tracking

## Technical Debt

- Request cost calculation not well documented
- Complex integration patterns with multiple callback types
