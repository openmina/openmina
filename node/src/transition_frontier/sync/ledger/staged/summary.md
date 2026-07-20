# Staged Ledger Sync State Machine

Synchronizes the staged ledger containing pending transactions and scan state
through parts fetching and reconstruction.

## Purpose

- Downloads staged ledger auxiliary data and pending coinbases from peers
- Reconstructs staged ledger from snarked ledger base plus fetched components
- Validates scan state integrity and transaction ordering
- Handles both empty and non-empty staged ledger cases

## Two-Path Reconstruction Process

### Path 1: Empty Staged Ledger (`ReconstructEmpty`)

- Detects when staged ledger is empty (aux_hash and pending_coinbase_aux are
  zero)
- Directly uses snarked ledger as the staged ledger
- Bypasses parts fetching for efficiency

### Path 2: Non-Empty Staged Ledger (`ReconstructPending`)

- Fetches `StagedLedgerAuxAndPendingCoinbases` from peers via RPC
- Validates fetched parts against expected hashes
- Delegates heavy reconstruction to staged ledger service
- Collects needed protocol states during reconstruction process

## Multi-Peer Validation

- **Parts fetching** - requests auxiliary data from multiple peers
- **Validation** - verifies fetched parts match expected structure and hashes
- **Error recovery** - retries with different peers on validation failures
- **Consensus** - requires valid parts from at least one peer to proceed

## State Flow

```
PartsFetchPending → PartsFetchSuccess → ReconstructPending → ReconstructSuccess → Success
                                   ↘ ReconstructEmpty ↗
```

## Key Features

- **Selective reconstruction** - optimizes empty staged ledger case
- **Service delegation** - uses specialized service for complex reconstruction
  work
- **Hash validation** - ensures reconstructed ledger matches expected target
  hash
- **Protocol state collection** - gathers protocol states needed for transaction
  validation
- **Multi-peer resilience** - validates parts from multiple sources for
  reliability

## Interactions

- Fetches staged ledger parts via P2P RPC from multiple peers
- Validates auxiliary data and pending coinbase structures
- Delegates reconstruction to staged ledger service for heavy computation
- Coordinates with snarked ledger sync (requires snarked completion first)
- Provides reconstructed staged ledger for transition frontier use
