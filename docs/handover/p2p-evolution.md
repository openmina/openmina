# P2P Layer Evolution Plan

This document outlines the evolution plan for Mina's P2P networking layer,
building on the successful pull-based design already implemented for OpenMina
webnodes. The idea of using QUIC as a transport was originally proposed by
George in his "Networking layer 2.0" document.

**Status**: The pull-based P2P protocol is implemented and operational. This
document proposes enhancements including QUIC transport, block propagation
optimizations, and integration with the OCaml node to create a unified
networking layer across all Mina implementations. Coordination with OCaml Mina
team required for ecosystem-wide adoption.

## Current State

### The Problem: Divergent P2P Architectures

The Mina ecosystem currently has divergent P2P implementations:

1. **Mina (OCaml) nodes**
   - Use libp2p exclusively via external Golang helper process (`libp2p_helper`)
   - Push-based GossipSub protocol
   - Known weaknesses in network performance and scalability

2. **OpenMina (Rust) nodes**
   - Support both libp2p (for OCaml compatibility) AND pull-based WebRTC
   - Must internally normalize between push and pull models, adding complexity
   - Webnodes use WebRTC exclusively and require Rust nodes as bridges to libp2p
     network
   - Maintenance burden of supporting two different protocol designs

This creates significant complexity:

- OpenMina maintains two protocol implementations
- Webnodes cannot directly communicate with OCaml nodes
- Different security and performance characteristics
- Inconsistent behavior and debugging challenges

## Vision: Unified Pull-Based P2P Layer

The goal is to evolve OpenMina's pull-based P2P design to improve webnode
networking immediately and potentially become the universal networking layer for
all Mina nodes (both Rust and OCaml), with multiple transport options. Full
ecosystem adoption would require coordination and agreement with the OCaml Mina
team.

### Core Design Principles

The pull-based model (detailed in [p2p/readme.md](../../p2p/readme.md))
provides:

- **Security**: Recipients grant permission to send, preventing flooding
- **Fairness**: Protocol-enforced resource allocation among peers
- **Simplicity**: No message queues or dropping strategies needed
- **Consistency**: Senders know what recipients have processed

### Target Architecture

The plan is for Mina OCaml nodes to replace `libp2p_helper` with OpenMina's Rust
P2P implementation.

| Node Type        | Current P2P         | Target P2P          | Transport                                             |
| ---------------- | ------------------- | ------------------- | ----------------------------------------------------- |
| OpenMina Webnode | Pull-based          | Same protocol       | WebRTC (browser-to-browser), QUIC (browser-to-server) |
| OpenMina Server  | libp2p + Pull-based | Pull-based only     | QUIC + WebRTC signaling                               |
| Mina OCaml Node  | libp2p via Golang   | Pull-based via Rust | QUIC + WebRTC signaling                               |

_Note: Server nodes primarily use QUIC for direct communication, with WebRTC
signaling infrastructure maintained to help webnodes discover peers._

## Evolution Phases

### Phase 1: Add QUIC Transport to OpenMina

**Goal**: Extend OpenMina's existing pull-based protocol to support QUIC as an
additional transport.

**Transport Quality Note**: Server-side WebRTC implementations are generally of
lower quality compared to QUIC implementations. This led to consideration of
implementing a minimalistic custom WebRTC library with only features needed for
the webnode protocol. However, if QUIC is adopted for webnode-to-server
communication, this custom implementation becomes unnecessary. Server nodes
would primarily use QUIC for direct communication, maintaining WebRTC signaling
infrastructure only to help webnodes discover and connect to peers.

**Scope**:

- Research and select Rust QUIC library (preference for minimal dependencies)
- Extend existing P2P channels abstraction to support QUIC transport
- Implement QUIC transport for server-to-server communication
- Maintain existing WebRTC support for browser compatibility

**Benefits**:

- Direct server connections without WebRTC signaling overhead
- Better performance (0-RTT, improved congestion control)
- Foundation for replacing libp2p

**Research needed**:

- Channel-to-stream mapping strategy
- Integration of QUIC flow control with pull-based model
- Library evaluation (quinn, s2n-quic, etc.)

### Phase 2: Create Rust P2P Library for OCaml

**Goal**: Package OpenMina's P2P implementation as a library for OCaml
integration.

**Prerequisites**: Before integration, the Rust libp2p implementation must be:

- Thoroughly reviewed and cleaned up
- Stress tested for production readiness
- Extended with testing features from `libp2p_helper` if still required (e.g.,
  gating and other ITN testing features)
- Validated for feature parity with current `libp2p_helper` functionality

**Scope**:

- Create Rust P2P helper process (similar to `libp2p_helper`) or OCaml-Rust FFI
  integration
- Design integration approach (helper process vs direct FFI)
- Create migration path from `libp2p_helper` to Rust P2P
- Implement configuration for transport selection

Mina OCaml nodes would replace `libp2p_helper` with a Rust program that reuses
OpenMina's P2P implementation. The integration could be either as a helper
process (like current `libp2p_helper`) or through OCaml-Rust FFI. Initially,
this Rust P2P helper would support both libp2p (for backward compatibility) and
the pull-based protocol, allowing for a gradual transition.

**Architecture**:

```
Mina OCaml Node
    ↓
Rust P2P Helper (replaces `libp2p_helper`)
    ↓
Pull-based Protocol + libp2p (initially)
    ↓
QUIC / WebRTC Signaling / libp2p transport
```

**Benefits**:

- Eliminate Golang dependency in Mina
- Single P2P implementation across ecosystem
- Direct integration without external processes

### Phase 3: Dual Protocol Support Period

**Goal**: Support both libp2p and pull-based protocols while proving the new
system in production.

**Scope**:

- Dual protocol support maintained (libp2p + pull-based)
- QUIC transport initially used only for webnode-to-server communication
- Extensive testing of server-to-server pull-based communication on private
  networks or devnet
- Production validation before wider adoption

**Testing Strategy**:

- Private network deployment with full server-to-server pull-based communication
- Devnet testing under realistic load conditions
- Performance comparison between libp2p and pull-based protocols
- Stability and reliability validation over extended periods

**Success Criteria**:

- Proven performance and stability of pull-based server-to-server communication
- Successful integration with OCaml nodes via Rust P2P library
- Demonstrable benefits over current libp2p implementation

### Phase 4: libp2p Deprecation

**Goal**: Complete transition to unified pull-based P2P layer.

**Important Note**: Full replacement of libp2p across the Mina ecosystem
requires coordination with the OCaml Mina team. This evolution plan represents
OpenMina's vision for improving P2P networking, starting with immediate benefits
for webnode-to-server communication and potentially becoming the new P2P
standard for Mina if adopted ecosystem-wide.

**Scope**:

- Coordinate network-wide migration timeline with OCaml Mina team
- Remove libp2p support from OpenMina (after ecosystem coordination)
- Remove libp2p protocol support from both implementations
- Simplify both codebases to single protocol

**End state**:

- All nodes use pull-based protocol
- Multiple transport options (WebRTC, QUIC)
- Unified implementation via Rust library

## Technical Enhancements

### Block Propagation Optimization

Independent of transport changes, an important optimization is planned
([Issue #998](https://github.com/openmina/openmina/issues/998)):

**Problem**: Mina blocks are large, causing slow propagation as nodes must
verify before forwarding.

**Solution**:

1. Propagate only consensus-critical headers first
2. Fetch full block body after consensus validation
3. Download only missing transactions/snarks not in local pools

**Advanced Optimization**: Since nodes maintain local pools of transactions and
snarks, many items in a new block may already be present locally. The protocol
could be enhanced to:

- Reference pool items by hash/ID rather than including full data
- Download only missing transactions and snarks not in local pools
- Leverage the existing efficient pool propagation mechanisms

**Impact**: Significant bandwidth reduction and faster block propagation.

## Implementation Considerations

### QUIC Library Selection

- Minimal dependencies (preferably avoiding async runtimes like tokio)
- Active maintenance and security updates

### Rust P2P Library Preparation

- Review and cleanup of existing libp2p implementation
- Stress testing for production readiness
- Extend with testing features from `libp2p_helper` if still required (e.g.,
  gating, ITN features)
- Validate feature parity with current `libp2p_helper`

### OCaml Integration

- Main challenge: architectural shift from push-based gossip to pull-based
  protocol
- Choose between helper process (like `libp2p_helper`) vs direct OCaml-Rust FFI
- Memory management across language boundaries (if FFI approach chosen)
- Error handling and recovery
- Shared implementation benefits: same Rust P2P code used by both Rust and OCaml
  nodes

### Testing Strategy

- Private network testing of server-to-server communication
- Devnet testing under realistic load
- Performance comparison between libp2p and pull-based protocols

### Network Transition

- Potential hardfork or softfork required for operator transition
- Gradual approach: nodes support both protocols, default switches to pull-based
- Eventually deprecate and remove libp2p support
- Network governance coordination needed for transition timeline

## References

- [OpenMina WebRTC P2P Implementation](../../p2p/readme.md)
- George's Networking layer 2.0 Proposal
- [Block Propagation Optimization (Issue #998)](https://github.com/openmina/openmina/issues/998)
- [Current libp2p Architecture](../../p2p/libp2p.md)
