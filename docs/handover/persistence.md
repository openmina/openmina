# Persistence Design (Not Yet Implemented)

This document outlines the proposed design for persisting the Mina ledger and
other critical state to disk, reducing memory usage and enabling faster node
restarts.

**Status**: Not yet implemented - this is a design proposal only.

**Critical for Mainnet**: This is one of the most important changes required to
make the webnode mainnet-ready.

## Overview

Currently, OpenMina keeps the entire ledger in memory, which creates scalability
issues for mainnet deployment where the ledger can be large. A persistent
storage solution is needed to:

- Reduce memory usage for both server-side nodes and webnodes
- Enable faster node restarts by avoiding full ledger reconstruction
- Deduplicate SNARK verification work across blocks and pools
- Support partial ledger storage for light clients

## Design Reference

A draft design for the persistence database is outlined in
[Issue #522](https://github.com/openmina/openmina/issues/522), which proposes an
approach for efficiently storing, updating, and retrieving accounts and hashes.

**Note**: There is a very old implementation for on-disk storage in
`ledger/src/ondisk` that was never used - a lightweight key-value store
implemented to avoid the RocksDB dependency. This is unrelated to the new
persistence design which intends to solve persistence for everything, not just
the ledger. But the old implementation may be worth revisiting anyway.

**Database Design Resources**: For those implementing persistence,
[Database Internals](https://www.databass.dev/) and
[Designing Data-Intensive Applications](https://dataintensive.net/) are
excellent books on database design and implementation. However, for Mina's
storage needs, nothing terribly advanced is required.

## Key Design Principles (from Issue #522)

1. **Simplicity First**: The design prioritizes simplicity over optimal
   performance
2. **Fixed-Size Storage**: Most data (except zkApp accounts) uses fixed-size
   slots for predictable access patterns
3. **Sequential Account Creation**: Mina creates accounts sequentially, filling
   leaves from left to right in the Merkle tree, enabling an append-only design
4. **Selective Persistence**: Only epoch ledgers and the root ledger need
   persistence; masks can remain in-memory
5. **Infrequent Updates**: Root ledger updates occur only when the transition
   frontier root moves (at most once per slot during high traffic)
6. **Hashes in Memory**: All Merkle tree hashes remain in RAM for quick access
7. **Recoverable**: Data corruption is not catastrophic as ledgers can be
   reconstructed from the network, but corruption must be easily detectable
   (e.g., through checksums or hash verification)

## Problems to be Solved

### 1. Memory Usage Reduction

- **Current**: Entire ledger in memory
- **Proposed Solution**: Only active masks and hashes in memory
- **Expected Impact**: Would dramatically reduce memory footprint for
  mainnet-scale ledgers

### 2. Faster Node Restarts

- **Current**: Must reconstruct ledger from genesis or snapshot
- **Proposed Solution**: Load persisted ledger directly from disk
- **Expected Impact**: Could reduce restart times from minutes to seconds
- **Critical for Webnodes**: Network sync is particularly expensive for webnodes
  due to limited bandwidth and connection quality typical in browser
  environments, making fast restarts essential for usability

### 3. Webnode Scalability

- **Current**: Limited by browser memory constraints - cannot handle
  mainnet-scale ledgers
- **Proposed Solution**: Store ledger through browser storage APIs
  (IndexedDB/OPFS)
- **Expected Impact**: Would enable true browser-based full nodes (hard
  requirement for mainnet support)

### 4. SNARK Verification Deduplication

- **Current**: OpenMina re-verifies all SNARKs every time they appear, even if
  previously seen
  - When a SNARK arrives in the snark pool, it's verified
  - When the same SNARK appears in a block, it's verified again
  - When the same SNARK appears in another block, it's verified yet again
- **Proposed Solution**: Store verified SNARKs in the persistence database
  - When a SNARK arrives, check if it exists in the database
  - If found in database, skip verification (already verified)
  - If not found, verify and store the result
  - Simple database lookup replaces expensive re-verification
- **Expected Impact**: Would significantly reduce redundant verification work,
  especially during high network activity
- **Reference Implementation**: We implemented this optimization in the OCaml
  node ([PR #12522](https://github.com/MinaProtocol/mina/pull/12522)) and
  demonstrated dramatic performance improvements: block application time for
  blocks with many completed works was reduced from ~8-14 seconds to ~0.015
  seconds by avoiding re-verification of SNARKs already present in the SNARK
  pool. This was not implemented in OpenMina yet as it was planned to be done as
  part of the persistence implementation.

### 5. Reduced Network Traffic and Improved Pool Consistency

- **Current**: Nodes frequently need to sync ledgers and pools from peers,
  creating network overhead
- **Proposed Solution**: Persist ledgers, SNARK pools, and transaction pools to
  disk
  - Nodes maintain state across restarts without full resync
  - Combined with webnode's pull-based P2P layer, enables better pool
    convergence
  - Less frequent ledger synchronization reduces network bandwidth usage
  - Especially beneficial for webnodes that may be restarted more often than
    server nodes
- **Expected Impact**: Would reduce overall network traffic and help nodes
  maintain consistent views of transaction and SNARK pools

## Open Questions

1. Exact zkApp slot size (depends on 8 vs 32 field implementation and
   verification key maximum size)
2. Optimal prefetching strategies for block producers?
3. Integration with existing mask hierarchy?
