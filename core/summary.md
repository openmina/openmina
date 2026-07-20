# Core Crate Summary

The core crate contains foundational infrastructure and shared utilities that
all other crates depend on. We use this crate for shared types, utilities, and
abstractions that need to be accessible across the entire project.

## What This Crate Contains

- **Substate System** (`src/substate.rs`): Type-safe access patterns for Redux
  state slicing and mutation control
- **WASM Threading** (`src/thread.rs`): Main thread task delegation system
  needed for browser environments
- **Network Configuration** (`src/network.rs`): Static network constants and
  config for mainnet/devnet
- **Core Domain Types**: Basic blockchain types (blocks, SNARKs, requests,
  consensus) that everything uses
  - `src/block/` - Block structures and validation
  - `src/snark/` - SNARK work and job management
  - `src/transaction/` - Transaction helper types and metadata
  - `src/consensus.rs` - Consensus types and fork decision logic
- **Request Management** (`src/requests/`): Type-safe request ID generation and
  lifecycle tracking
- **Channel Wrappers** (`src/channels.rs`): Abstractions over flume channels for
  message passing
- **Distributed Pool** (`src/distributed_pool.rs`): BTreeMap-based data
  structure for network-synchronized state

## Technical Debt and Issues

### Medium Priority

**Hardcoded Network Constants**

- Fork constants are hardcoded in the `NetworkConfig` struct
- Impact: Can't deploy flexibly, need code changes for network updates

### Low Priority

**Unsafe Redux Access**

- `unsafe_get_state()` method in `Substate` struct breaks Redux safety
- Only used by the transaction pool state machine which requires refactoring

**Inconsistent Error Handling**

- Various instances of `panic!`, `unwrap()`, or `expect()` calls throughout core

## Known Limitations

1. **Network Config**: Can't dynamically configure network parameters without
   code changes
2. **WASM Constraints**: Browser limitations require specialized threading
   patterns
