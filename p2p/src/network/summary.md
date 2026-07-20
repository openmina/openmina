# P2P Network State Machine

Low-level networking protocols and transport management.

## Purpose

- Implements libp2p protocol stack for server-to-server communication
- Manages transport layer (TCP, WebRTC) with dual transport support
- Handles protocol negotiation and stream multiplexing
- Provides encrypted networking primitives for higher-level channels

## Key Components

- **Scheduler**: Connection orchestrator and protocol coordinator (misnamed -
  manages connections, not scheduling)
- **Select**: Protocol negotiation using multistream-select (hardcoded protocol
  registry limits extensibility)
- **Kad**: Kademlia DHT for peer discovery (complex bootstrap and routing table
  issues)
- **Pubsub**: Gossip protocol for broadcasts (963-line monolithic reducer with
  performance issues)
- **Identify**: Peer identification and capability exchange
- **Yamux**: Stream multiplexing over connections
- **Noise**: Encryption protocol (security hardening opportunities - session
  keys not zeroized, debug output leaks, deprecated crypto functions)
- **Pnet**: Private network support with pre-shared key authentication
- **RPC**: Low-level request-response protocol with binprot serialization

## Technical Debt

- **Performance**: Pubsub has O(n) message lookups and unbounded memory growth
- **Architecture**: Large monolithic reducers across multiple components
- **Extensibility**: Hardcoded protocol registry in Select component

## Interactions

- Establishes encrypted connections with peer authentication
- Discovers peers via Kademlia DHT with bootstrap coordination
- Multiplexes multiple protocol streams over single connections
- Handles protocol negotiation with version compatibility
- Coordinates connection lifecycle and cleanup
- Provides transport abstraction for higher-level channels

## Additional Issues

- Several components need refactoring (Pubsub, Kad internals, Scheduler naming)
- Hard-coded values throughout that should be configurable
- Ongoing refactoring work in Yamux component (PR #1085)
