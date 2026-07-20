# Kademlia State Machine

Implements Kademlia DHT for peer discovery and routing.

## Purpose

- Maintains distributed hash table
- Discovers network peers
- Routes queries through network
- Manages k-buckets and routing table

## Key Components

- **Bootstrap**: Initial network join
- **Request**: Query handling
- **Stream**: Kademlia protocol streams

## Interactions

- Finds peers by ID
- Stores peer addresses
- Handles DHT queries
- Maintains network topology

## Technical Debt

### Major Issues

- **Large Internals File (912 lines)**: `p2p_network_kad_internals.rs` mixes
  routing table, distance calculations, K-buckets, and iterators - should be
  split into separate modules for maintainability
- **Large Stream Reducer (633 lines)**: Complex state transitions handling both
  incoming/outgoing streams could benefit from moving logic to state methods
- **Missing Error Reporting**: Silently ignores multiaddr parsing errors
  (kad_effectful_effects.rs:87) making debugging difficult

### Moderate Issues

- **Incomplete Functionality**: Missing callbacks for stream operations
  (request_reducer.rs:94,159) and incomplete error handling with string-based
  errors (stream_state.rs:45)
- **Suboptimal Data Structures**: Bootstrap uses heavy `BTreeMap` for request
  tracking (bootstrap_state.rs:26) and inconsistent address handling between
  `SocketAddr` and `Multiaddr`
- **Hard-coded Values**: Magic numbers for bootstrap thresholds (20) and batch
  sizes (3) should be configurable

### Refactoring Plan

1. **Extract modules** from internals file: routing_table.rs, distance.rs,
   bucket.rs
2. **Move complex logic to state methods** to simplify reducers
3. **Implement structured error types** instead of string errors
4. **Add error reporting** for failed operations
5. **Standardize on Multiaddr** for consistent address handling
