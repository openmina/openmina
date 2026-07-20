# Transaction Pool Candidate State Machine

Coordinates P2P transaction discovery and fetching before forwarding to main
transaction pool for validation.

## Purpose

- Manages per-peer transaction discovery and state tracking
- Coordinates fetching full transactions from transaction info received from
  peers
- Collects and batches transactions for verification by main pool
- Prioritizes pubsub messages over direct peer requests

## Transaction Flow

1. **Info Received** - peer sends transaction info (hash + fee)
2. **Fetch Pending** - requests full transaction via RPC
3. **Received** - full transaction received from peer
4. **Verify Pending** - forwards batch to main pool via `StartVerify` action
5. **Verify Success/Error** - cleans up candidate state based on result

## Key Features

- **Per-Peer State Tracking** - maintains transaction states for each peer
  independently
- **Priority Ordering** - orders transaction fetching by fee and arrival order
- **Batch Processing** - collects transactions and forwards batches rather than
  individual transactions
- **Pubsub Priority** - processes pubsub messages before peer-specific requests
- **State Coordination** - integrates with main transaction pool without
  duplicating validation logic

## Interactions

- Receives transaction info from P2P peers
- Fetches full transactions via P2P RPC requests
- Forwards transaction batches to main pool for validation
- Manages peer connection lifecycle (prunes disconnected peers)
- Coordinates with ledger service availability for verification timing

## Note

This component does NOT perform transaction validation itself - it only
coordinates P2P discovery and fetching. All validation (signatures, nonces,
balances, spam filtering) happens in the main transaction pool layer.
