# Ledger Crate

## Overview

The `ledger` crate is a comprehensive Rust implementation of the Mina protocol's
ledger, transaction pool, staged ledger, scan state, proof verification, and
zkApp functionality, providing a direct port of the OCaml implementation. For
developers familiar with the OCaml codebase, this maintains the same
architecture and business logic while adapting to Rust idioms.

For technical debt and critical issues, see
[`ledger/summary.md`](../../ledger/summary.md).

## Architecture

### Core Components

**BaseLedger Trait** (`src/base.rs`)

- Direct mapping to OCaml's `Ledger_intf.S`
- Defines the fundamental ledger interface for account management, Merkle tree
  operations, and state queries
- All ledger implementations (Database, Mask) implement this trait

**Mask System** (`src/mask/`)

- Port of OCaml's `Ledger.Mask` with identical copy-on-write semantics
- Provides layered ledger views for efficient state management
- Uses `Arc<Mutex<MaskImpl>>` for cheap reference counting; `Mask::clone()` is
  fast
- Used extensively in transaction processing to create temporary ledger states

**Database** (`src/database/`)

- In-memory implementation (ondisk module exists but is not used)
- Corresponds to OCaml's `Ledger.Db` interface
- Handles account storage and Merkle tree management

### Transaction Processing

**Transaction Pool** (`src/transaction_pool.rs`)

- Complete port of `Transaction_pool.Indexed_pool` with identical behavior:
  - Fee-based transaction ordering
  - Sender queue management with nonce tracking
  - Revalidation on best tip changes
  - `VkRefcountTable` for verification key reference counting
- Handles transaction mempool operations, expiration, and replacement logic

**Staged Ledger** (`src/staged_ledger/`)

- Maps directly to OCaml's staged ledger implementation
- `Diff` corresponds to `Staged_ledger_diff` with same partitioning
- Manages transaction application and block validation
- Handles pre-diff info for coinbase and fee transfers

**Scan State** (`src/scan_state/`)

- Direct port of the parallel scan tree structure
- `transaction_logic` module maps to OCaml's `Transaction_logic`
- Manages SNARK work coordination and pending coinbase
- Maintains same proof requirements as OCaml implementation

### Proof System Integration

**Proof Generation and Verification** (`src/proofs/`)

- Transaction proof generation and verification (`transaction.rs`)
- Block proof generation and verification (`block.rs`)
- zkApp proof generation and handling (`zkapp.rs`)
- Merge proof generation for scan state
- Witness generation for circuits (`witness.rs`)
- Uses Kimchi proof system via proof-systems crate
- Maintains protocol compatibility with OCaml proofs

Note: The crate implements witness generation for circuits but not the
constraint generation, so circuits cannot be fully generated from this crate
alone.

For detailed technical information about the proof system implementation, see
[`ledger/src/proofs/summary.md`](../../ledger/src/proofs/summary.md).

**zkApp Support** (`src/zkapps/`)

- Full zkApp transaction processing
- Account update validation
- Permission and authorization checks
- SNARK verification for zkApp proofs

### Additional Components

**Account Management** (`src/account/`)

- Account structure with balances, permissions, and timing
- Token support with owner tracking
- Delegate and voting rights management

**Sparse Ledger** (`src/sparse_ledger/`)

- Efficient partial ledger representation
- Used for witness generation in SNARK proofs
- Maintains minimal account set needed for proof creation

## Key Differences from OCaml

1. **Memory-only implementation** - No persistent disk storage used
2. **Rust idioms**:
   - `Result<T, E>` instead of OCaml's `Or_error.t`
   - `HashMap`/`BTreeMap` instead of OCaml's `Map`/`Hashtbl`
   - Ownership model instead of garbage collection
3. **Serialization** - Uses serde for state machine persistence and network
   communication

**Note**: The FFI code present in the crate is stale and unused - it was from
earlier integration attempts before a implementing a full node was even planned.

## Compatibility

The crate maintains full protocol compatibility with the OCaml implementation:

- Same Merkle tree structure and hashing
- Identical transaction validation rules
- Compatible proof verification
- Same account model and permissions

The `port_ocaml` module provides compatible implementations with the OCaml
runtime, including:

- Hash functions that match OCaml's behavior
- Hash table implementation that behaves like Jane Street's `Base.Hashtbl` for
  compatibility

## Future Refactoring

The ledger crate is currently monolithic and should ideally be split into
separate crates. At a minimum, it could be split into:

- `mina-account` - Account structures and management
- `mina-ledger` - Base ledger implementation, staged ledger, masks, sparse
  ledger, Merkle tree infrastructure
- `mina-transaction-logic` - Transaction application and validation logic,
  currency types
- `mina-scan-state` - SNARK work coordination and parallel scan (depends on
  transaction-logic)
- `mina-transaction-pool` - Transaction mempool logic (depends on ledger for
  masks)
- `mina-proofs` - Proof generation and verification

**Note**: The staged ledger remains with the core ledger as it's tightly coupled
with the mask system and represents the fundamental "next state" computation.
Attempting to separate it would create circular dependencies and break the
natural layering of Database → Mask → StagedLedger.

This would improve compilation times, enable better testing isolation, and allow
other components to depend only on what they need.

## Usage in OpenMina

The ledger crate is primarily used by:

- Block production for transaction selection
- Block application for state transitions
- P2P for transaction pool management
- SNARK workers for proof generation
- RPC endpoints for balance and account queries

All ledger operations go through the state machine actions defined in the node
crate, ensuring deterministic execution.
