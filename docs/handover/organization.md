# OpenMina Project Organization

This document provides a navigation guide to the OpenMina codebase structure,
focusing on entry points, supporting libraries, and build organization.

> **Prerequisites**: Read
> [Architecture Walkthrough](architecture-walkthrough.md) and
> [State Machine Structure](state-machine-structure.md) first. **Next Steps**:
> After understanding the codebase layout, dive into specific components via
> [Services](services.md) or start developing with
> [State Machine Development Guide](state-machine-development-guide.md).

## Component Details

For detailed information about state machine components and their interactions,
see [State Machine Structure](state-machine-structure.md). This document focuses
on codebase navigation and supporting infrastructure.

## Entry Points

### CLI (`cli/`)

The main entry point for running OpenMina nodes. Contains:

- **`src/main.rs`** - Application entry point with memory allocator setup and
  signal handling
- **`src/commands/`** - CLI command implementations:
  - `node/` - Node startup and configuration
  - `replay/` - State replay functionality for debugging
  - `snark/` - SNARK-related utilities and precalculation
  - `build_info/` - Build information and version details
  - `misc.rs` - Miscellaneous utilities

The CLI supports different networks (devnet/mainnet) and provides the
server-side node functionality.

## Core Components

### Node (`node/`)

The main orchestrating component containing both state machine logic and service
implementations. For state machine component details, see
[State Machine Structure](state-machine-structure.md).

**Special Implementations:**

- **`node/web/`** - WebAssembly-compatible service layer for browser deployment
  - Exports a default `Service` for web (WASM) environments
  - Provides Rayon-based parallelism configuration for web workers
  - Enables OpenMina to run as a light client in browsers

### P2P Networking (`p2p/`)

OpenMina includes two distinct P2P implementations. For state machine details,
see [State Machine Structure](state-machine-structure.md).

#### libp2p Implementation

Traditional P2P networking for server-to-server communication and OCaml node
compatibility with custom WebRTC transport, Noise security, Yamux multiplexing,
and standard libp2p protocols.

#### WebRTC-Based P2P Implementation

Pull-based (long-polling) P2P protocol designed for webnode deployment and
browser environments.

**Design Features:**

- Pull-based message flow where recipients request messages instead of receiving
  unsolicited pushes (long-polling approach)
- 8 specialized channels (BestTipPropagation, TransactionPropagation, etc.)
- Efficient pool propagation with eventual consistency
- DDOS resilience through fairness mechanisms
- WebRTC transport for browser-to-browser communication

**Implementation:**

- Core WebRTC code: `p2p/src/webrtc/` and `p2p/src/service_impl/webrtc/`
- Channel implementations: `p2p/src/channels/` (8 specialized state machines)
- Multi-backend support: Rust WebRTC, C++ WebRTC, and Web/WASM

**Future Enhancements:**

- QUIC transport integration for protocol consolidation
- Block propagation optimization with header/body splitting
- Advanced bandwidth reduction using local pool references
- See [P2P Evolution Plan](p2p-evolution.md) for detailed plans

**Documentation:** See [p2p/readme.md](../../p2p/readme.md) for design overview

### SNARK Verification (`snark/`)

State machine components for managing proof verification workflows. For
component details, see [State Machine Structure](state-machine-structure.md).

**Note:** The actual proof system implementations and cryptographic proof
generation/verification are located in the `ledger` crate's `proofs/` module.
This crate only contains the state machine logic for orchestrating proof
verification workflows.

### Ledger (`ledger/`)

Blockchain state management library. For detailed information, see
[Ledger Crate](ledger-crate.md).

## Supporting Libraries

### Core Types (`core/`)

Foundational shared types and utilities used across the entire codebase:

- **Block types** - Applied blocks, genesis configuration, prevalidation
- **Transaction types** - Transaction info and hash wrappers
- **SNARK types** - Job commitments, IDs, and comparison utilities
- **Network types** - P2P configuration and network utilities
- **Request types** - Request and RPC ID management
- **Constants** - Constraint constants and protocol parameters
- **Substate system** - Fine-grained state access control for the state machine
- **Logging and threading utilities**

This crate provides the common foundation that all other components depend on.

### Cryptographic Primitives

#### VRF (`vrf/`)

Verifiable Random Function implementation for block producer selection:

- Implements the cryptographic VRF used in Proof of Stake consensus
- Generates verifiable random numbers for fair block producer selection
- Provides threshold evaluation and message handling
- Compatible with the OCaml node's VRF implementation

#### Poseidon Hash (`poseidon/`)

Poseidon hash function implementation optimized for zero-knowledge proofs:

- Sponge construction constants and parameters
- Field arithmetic over Mina's base field (Fp) and scalar field (Fq)
- ZK-friendly hash function used throughout the protocol
- Compatible with Kimchi proof system requirements

### Message Serialization (`mina-p2p-messages/`)

Comprehensive message format definitions for network communication, generated
from OCaml binprot shapes:

**Code Generation:**

- Types auto-generated from OCaml `bin_prot` shapes stored in `shapes/`
  directory
- Generated code in `src/v2/generated.rs` with OCaml source references for every
  type
- Manual implementations in `src/v2/manual.rs` for complex types requiring
  custom logic
- Configuration files (`default-v2.toml`) control the generation process

**Protocol Support:**

- **v2 protocol** - Current Mina protocol version with full type coverage
- **RPC messages** - Method definitions and request/response types
- **Gossip messages** - Network propagation message formats
- **Binary compatibility** - Full `bin_prot` serialization compatibility with
  OCaml

**Current Limitations:**

- **Monomorphized types** - All generic types have been specialized, leading to
  code duplication
- **Manual maintenance** - Complex types require hand-written implementations
  that must be kept in sync
- **Code bloat** - Many similar wrapper types for different contained types

**Future Improvements Needed:**

- **Transition to manual maintenance** - Move away from code generation to
  hand-written types
- **Polymorphize types to match OCaml** - Where OCaml uses polymorphic types
  (generics), Rust should use matching generic definitions rather than
  monomorphized variants
- **Maintain structural compatibility** - Ensure type definitions match the
  original OCaml structure and polymorphism
- **Preserve protocol compatibility** - Ensure binary serialization
  compatibility is maintained during manual refactoring

### Development Tools

#### Macros (`macros/`)

Procedural macros for code generation:

- Action and event system macros
- Serialization helpers for OCaml compatibility

#### Testing Infrastructure (`node/testing/`)

Comprehensive testing framework for multi-node scenarios. For details, see
[`testing-infrastructure.md`](testing-infrastructure.md).

## Additional Components

### Frontend (`frontend/`)

Angular-based web interface providing:

- Node monitoring dashboard
- Network visualization
- Block and transaction exploration
- Real-time metrics and debugging tools

### Tools (`tools/`)

Various utilities for development and analysis:

- **`ledger-tool/`** - Ledger inspection and manipulation
- **`hash-tool/`** - Hash verification utilities
- **`bootstrap-sandbox/`** - Network bootstrapping testing
- **`fuzzing/`** - Fuzz testing infrastructure
- **And many more specialized tools**

### Producer Dashboard (`producer-dashboard/`)

Block producer monitoring and metrics collection system.

## Build and Configuration

### Root Configuration

- **`Cargo.toml`** - Workspace configuration defining all crates
- **Docker configurations** - Various deployment scenarios
- **Helm charts** - Kubernetes deployment configurations (probably stale)

### Development Support

- **`tests/`** - Integration test files and test data
- **`genesis_ledgers/`** - Genesis ledger data for devnet
- **Scripts and tooling** - Build, deployment, and analysis scripts

## Dependencies and Build Order

The project follows a clear dependency hierarchy:

1. **Foundation**: `core`, `macros`, `poseidon`, `vrf`
2. **Protocols**: `mina-p2p-messages`, `ledger`
3. **Networking**: `p2p`
4. **Services**: `snark`, `node`
5. **Applications**: `cli`, frontend tools

This organization enables:

- **Incremental compilation** - Changes to high-level components don't rebuild
  foundations
- **Clear boundaries** - Each component has well-defined responsibilities
- **Testability** - Components can be tested in isolation
- **Modularity** - Browser deployment through `node/web`, various tool builds

The architecture supports multiple deployment targets: native nodes, browser
light clients, testing frameworks, and various specialized tools, all sharing
the same core protocol implementation.
