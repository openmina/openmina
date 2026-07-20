# Mainnet Readiness

This document outlines the key features and improvements required for OpenMina
to be ready for mainnet deployment.

## Critical Requirements

### 1. Persistence Implementation

**Status**: Draft design
([Issue #522](https://github.com/openmina/openmina/issues/522), see
[persistence.md](persistence.md))

The ledger is currently kept entirely in memory, which is not sustainable for
mainnet's scale. Persistence is required for:

- Reducing memory usage to handle mainnet-sized ledgers and amount of snarks.
- Enabling fast node restarts without full resync
- Supporting webnodes with browser storage constraints
- Providing a clean foundation for implementing SNARK verification deduplication

**Note**: There is a very old implementation for on-disk storage in
`ledger/src/ondisk` that was never used - a lightweight key-value store
implemented to avoid the RocksDB dependency. This is unrelated to the new
persistence design which intends to solve persistence for everything, not just
the ledger. But the old implementation may be worth revisiting anyway.

**Performance Impact**: The importance of SNARK verification deduplication for
mainnet performance has been demonstrated in the OCaml node, where we achieved
dramatic improvements (8-14 seconds → 0.015 seconds for block application). See
the "SNARK Verification Deduplication" section in
[persistence.md](persistence.md) for details.

### 2. Wide Merkle Queries

**Status**: Not implemented
([Issue #1086](https://github.com/openmina/openmina/issues/1086))

Wide merkle queries are needed for:

- Protocol compatibility
- Faster synchronization

### 3. Delta Chain Proof Verification

**Status**: Not implemented
([Issue #1017](https://github.com/openmina/openmina/issues/1017))

When verifying blocks, OpenMina should verify the delta chain proofs.

### 4. Automatic Hardfork Handling

**Status**: Not implemented

OpenMina needs a mechanism to automatically handle protocol hardforks to
maintain compatibility with the Mina network. There is an implementation in
progress for the OCaml node, and the Rust node should implement something
compatible. However, we lack detailed knowledge of the OCaml implementation to
provide specific guidance.

### 5. Security Audit

**Status**: Not performed

A comprehensive security audit by qualified third-party security experts is
essential before mainnet deployment. This should cover cryptographic
implementations, consensus logic, networking protocols, and potential attack
vectors specific to the Rust implementation.

### 6. Mainnet Genesis Ledger Distribution

**Status**: Solution exists for devnet, mainnet needs implementation

The node requires efficient genesis ledger loading for practical operation. A
binary genesis ledger must be produced for mainnet and included in the node
distribution (or made downloadable from a location where the node can fetch it).
Currently, mainnet genesis ledgers would be too large and expensive to process
in JSON format.

**Current Implementation**:

- Genesis ledgers are available in JSON format at
  [openmina-genesis-ledgers](https://github.com/openmina/openmina-genesis-ledgers)
  repository
- Devnet uses a prebuilt binary format (`genesis_ledgers/devnet.bin`) that loads
  very quickly
- The `tools/ledger-tool` utility can generate these binary formats

**Requirements for Mainnet**:

- Generate equivalent binary format for mainnet genesis ledger to ensure fast
  node startup
- Establish workflow for creating and distributing mainnet binary ledgers
- Implement process for updating binary ledgers after hardforks
- Handle potential devnet relaunch scenarios requiring new binary ledgers

**Note**: These binary ledger files would become deprecated once persistence is
implemented, as the persisted database itself could be provided for initial node
setup instead.

### 7. Error Sink Service Integration

**Status**: Partially implemented (PR #1097)

Currently, the node intentionally panics in recoverable error situations (such
as block proof failures) to make errors highly visible during development. For
mainnet deployment, this needs to transition to using the error sink service to
report errors and continue operation instead of forcing node shutdown. This is
critical for operational stability in production environments.

## Protocol Compliance and Feature Parity

### 1. Block Processing Deviation

Currently, only the best block is processed and broadcasted. See
[this analysis](https://gist.github.com/tizoc/4a364dc2f8f29396a4097428a07f58d8)
for details on this deviation from the full protocol.

### 2. SNARK Work Partitioner

Feature parity requirement for full node capabilities.

### 3. GraphQL API for SNARK Workers

Feature parity requirement for supporting external SNARK workers. SNARK workers
need a proper API interface to:

- Submit completed work
- Query work requirements
- Coordinate with block producers

## Future Mina Compatibility

### 1. Dependency Updates

- Update proof-systems, ark, and other cryptographic dependencies to latest
  versions
- Ensure compatibility with future Mina protocol changes

### 2. App State Field Increase

Support for increasing app state from 8 to 32 fields, enabling more complex
smart contracts and zkApps.

## Webnode-Specific Requirements

For webnodes to handle mainnet:

- Persistence implementation (see [persistence.md](persistence.md)) - critical
  for webnodes due to memory constraints, intermittent connectivity, and
  frequent restarts
- Memory usage constraints are manageable without block proving
- Block production for webnodes is not a priority since block producers are
  unlikely to use browser-based setups for production operations, so the
  separate WASM memory block
  ([Issue #1128](https://github.com/openmina/openmina/issues/1128)) may not be
  necessary

### zkApp Integration

A popular feature request from users is direct zkApp and webnode integration.
This would allow zkApps to:

- Get better visibility into the network state
- Query the blockchain directly without relying on third-party nodes
- Avoid query limits imposed by external services
- Provide a more decentralized and reliable infrastructure for zkApp developers

Additionally, the webnode could be packaged as a Node.js library, enabling zkApp
developers to build testing frameworks that take advantage of OpenMina's
simulator capabilities for more comprehensive and realistic testing
environments. In such testing setups, block production would use dummy proofs
rather than full proof generation.

### Known Issues from Community Testing

During testing with ~100 webnode operators from the community, a few critical
issues were identified. See the
[official retrospective](https://minaprotocol.com/blog/retro-mina-web-node-testing)
for complete details.

#### 1. Seed Node Performance

- **Issue**: Performance problems on the first day due to UDP socket management
  issues in the webrtc-rs library.
- **Resolution**: Switched from webrtc-rs to the C++ "datachannel"
  implementation, which performs worse but is more stable.
- **Future Improvement**: Using QUIC transport for webnode-to-server
  communication would also help with seed node performance (see
  [P2P Evolution Plan](p2p-evolution.md)).
- **Status**: Resolved for testing, but seed node scalability should be
  monitored for mainnet deployment.

#### 2. Memory Limitations

- **Issue**: Webnodes are limited to 4GB of memory due to WASM constraints. When
  nodes sometimes reached this limit during block proving, they became stuck or
  experienced major thread crashes.
- **Root Cause**:
  - Block proving operations consuming excessive memory within the same memory
    space
  - WASM memory allocator limitations leading to fragmentation
- **Solutions**:
  - Moving the prover to its own WASM heap would alleviate the issue
    ([Issue #1128](https://github.com/openmina/openmina/issues/1128))
  - Memory limitations are less critical for nodes that don't produce and prove
    blocks
  - Implementing persistence (as per [persistence.md](persistence.md)) would
    considerably improve the situation

#### 3. Network Connectivity and Bootstrap Issues

- **Issue**: Many nodes had difficulty completing initial bootstrap, especially
  when the webnode or peers providing staged ledgers experienced network
  connectivity problems (instability, low bandwidth, high latency).
- **Impact**: Hard to finish initial sync for many community operators.
- **Note**: The RPC used to fetch the staged ledger is particularly problematic
  because it is very heavy and needs to download a lot of data in a single
  request. It would be a good idea to redesign this RPC, ideally enabling it to
  fetch parts from multiple peers in the same way that the snarked ledger sync
  process does.
- **Planned Solutions**:
  - Prefer server-side nodes for fetching initial ledgers instead of relying on
    other webnodes
  - Add support for wide merkle queries to reduce roundtrips and improve sync
    efficiency
  - Better peer selection algorithms for initial bootstrap

## Rollout Plan

### Testing Requirements

Before mainnet deployment, OpenMina requires extensive testing to ensure
protocol compatibility and reliable operation. Note that OpenMina has already
demonstrated stability in devnet environments: block producer nodes have run
continuously for over two months without issues (only restarting for upgrades),
and webnodes without block production have maintained perfect uptime for two
continuous weeks without memory issues.

However, there are rare situations where produced blocks cannot be proven (or
more precisely, where invalid proofs are produced). This is one of the primary
areas requiring further investigation and resolution.

#### Scenario Testing Expansion

- **Significantly increase scenario tests** to cover edge cases and protocol
  interactions
- **Multi-node scenarios** testing various network configurations and failure
  modes
- **Long-running stability tests** to validate node behavior over extended
  periods
- **Compatibility testing** with OCaml nodes to ensure seamless network
  operation
- **Stress testing** under high transaction volume and network load

#### Protocol Compatibility Validation

- **Comprehensive testing against OCaml implementation** for all protocol
  interactions
- **Cross-implementation consensus testing** to verify identical blockchain
  state
- **P2P protocol compatibility** testing for message handling and propagation
- **RPC API compatibility** testing to ensure client applications work
  seamlessly

### Prover/Verifier Implementation Strategy Options

OpenMina has a full implementation of both prover and verifier in Rust. The
prover includes many optimizations that make proving significantly faster than
the original OCaml implementation, including optimizations for server-side
proving. However, these implementations have not been audited, and auditing plus
ongoing maintenance requires significant time and effort.

#### Option 1: Complete Rust Implementation

Continue using OpenMina's Rust prover and verifier implementations.

**Pros**:

- Full control and better integration with the Rust codebase
- Significant performance improvements over OCaml implementation
- Single codebase without external dependencies

**Cons**:

- Requires comprehensive security audit before mainnet deployment
- Ongoing maintenance and compatibility responsibilities
- More implementation work for any missing features

#### Option 2: OCaml Subprocess Integration

Reuse the proven OCaml prover and verifier implementations through subprocess
services with wrapper services in the state machine to handle communication
(similar to what OpenMina used before it had its own prover).

**Pros**:

- Leverages battle-tested OCaml prover/verifier code
- Reduces implementation work and compatibility risks
- Faster path to mainnet readiness
- Proven correctness and performance

**Cons**:

- Additional complexity in service management
- Cross-process communication overhead
- Dependency on OCaml runtime

**Implementation Approach for Option 2**:

- Create service interfaces for prover/verifier subprocess communication
- Implement subprocess lifecycle management (start, restart, health checks)
- Design efficient data serialization for cross-process communication
- Add comprehensive error handling and fallback mechanisms

#### Webnode-Specific Considerations

**Essential Verifier Requirements**: All webnodes need verifiers for:

- Block proof verification
- zkApp proof verification
- Transaction proof verification

**Block Production Capability**: Webnodes can produce and prove blocks, but this
may not be a rollout priority.

**Implementation Strategy for Webnodes**:

- **Rust verifier implementation**: Much smaller and simpler than prover, making
  it easier to review, verify correctness, and maintain
- **Prover considerations**: If webnode block production isn't prioritized
  initially, prover implementation can be deferred
- **Alternative approaches**: If Rust verifier proves challenging, alternative
  verification methods need investigation
- **Hybrid approach**: Rust verifiers for webnodes, OCaml subprocess for full
  node provers when needed

### Deployment Phases

1. **Extended Testnet Testing** - Months of testing with comprehensive scenario
   coverage
2. **Limited Mainnet Beta** - Controlled deployment with selected nodes,
   potentially with block production disabled
3. **Initial Mainnet Rollout** - Non-block-producing nodes for network health
   safety
   - Verification-only deployment for both server-side and webnodes
   - Allows earlier rollout while building confidence
   - Reduces risk to network stability
4. **Block Production Enablement** - Enable block proving in later releases when
   confidence is established
5. **Full Production** - Complete mainnet readiness with all features enabled

**Rollout Strategy Benefits**:

- **Earlier deployment**: Verification-only nodes can be deployed sooner
- **Network safety**: Reduces risk of consensus issues during initial rollout
- **Gradual feature introduction**: Block production can be added once the core
  node functionality is proven stable
- **Confidence building**: Allows time to validate node behavior before enabling
  block production

## Summary

The most critical items for mainnet readiness are:

- **Persistence** - Without this, nodes may not be able to handle mainnet's
  ledger and snark pool size
- **Wide Merkle Queries** - Needed for compatibility and faster sync
- **Delta Chain Verification** - Required for protocol compliance
- **Hardfork Handling** - Essential for network compatibility
- **Security Audit** - Third-party security review before mainnet deployment
- **Mainnet Genesis Ledger** - Binary format required for mainnet distribution
  and hardfork updates
- **Error Sink Service** - Replace intentional panics with graceful error
  reporting
- **Comprehensive Testing** - Extensive scenario testing for protocol
  compatibility
- **Prover/Verifier Strategy** - Decision on Rust implementation vs OCaml
  subprocess integration
