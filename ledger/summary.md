# Ledger Crate Summary

The ledger crate is the most complex component in the codebase. For architecture
overview and design details, see
[docs/handover/ledger-crate.md](../docs/handover/ledger-crate.md).

## Quick Reference

**Core Ledger**

- `src/base.rs` - BaseLedger trait (fundamental interface)
- `src/database/` - In-memory account storage
- `src/mask/` - Layered ledger views with Arc-based sharing
- `src/tree.rs` - Merkle tree operations

**Transaction Processing**

- `src/transaction_pool.rs` - Mempool with fee-based ordering
- `src/staged_ledger/` - Block validation and transaction application
- `src/scan_state/` - SNARK work coordination and parallel scan

**Proof System**

- `src/proofs/` - Transaction, block, and zkApp proof generation/verification
- `src/sparse_ledger/` - Minimal ledger representation for proofs
- `src/zkapps/` - zkApp transaction processing

**Account Management**

- `src/account/` - Account structures, balances, permissions

## Status

The ledger components have proven reliable on devnet despite technical debt
patterns. The implementation maintains the same battle-tested logic that runs
the Mina network.

## Issues for Improvement

**Error Handling**

- Extensive use of `.unwrap()` and `.expect()` calls in code paths, particularly
  in `scan_state/transaction_logic.rs`, `staged_ledger/staged_ledger.rs`, and
  `transaction_pool.rs`
- These calls are generally in code paths with well-understood preconditions but
  could benefit from explicit error propagation
- Inconsistent error handling patterns across modules
- Verification key lookup bug fix from upstream Mina Protocol needs to be ported
  (https://github.com/MinaProtocol/mina/pull/16699)

**Monolithic Structure**

- Large files like `scan_state/transaction_logic.rs` and
  `staged_ledger/staged_ledger.rs` mirror OCaml's structure and are difficult to
  navigate
- Files contain embedded tests that are hard to discover
- When modifying these files, prefer small targeted changes over major
  restructuring

**Performance**

- Excessive cloning of large structures in hot paths:
  - `SparseLedger::of_ledger_subset_exn()` calls `oledger.copy()` creating
    unnecessary deep clones for sparse ledger construction
  - Transaction pool operations clone transaction objects with acknowledged TODO
    comments about performance
- Performance monitoring infrastructure exists but is disabled
- No memory pooling or reuse strategies (could help with memory fragmentation in
  WASM)

**Memory Management**

- Memory-only implementation, no persistence for production
- There's an unused `ondisk` implementation but we were planning a more
  comprehensive global solution (see persistence.md)
- Thread-local caching holds memory indefinitely

**Code Organization**

- Multiple TODO/FIXME items throughout the codebase requiring attention
- Incomplete implementations in `sparse_ledger/mod.rs` with unimplemented trait
  methods

## Refactoring Plan

**Phase 1: Safety**

- Replace `.unwrap()` with proper error propagation in production code
- Reduce cloning in hot paths
- Standardize error types

**Phase 2: Decomposition** Break into focused crates: `mina-account`,
`mina-ledger`, `mina-transaction-logic`, `mina-scan-state`,
`mina-transaction-pool`, `mina-proofs`

Changes must maintain strict OCaml compatibility while improving performance for
production.
