# P2P State Machine

Core peer-to-peer networking layer providing dual transport abstraction for node
communication.

## Purpose

- Provides unified networking interface over libp2p and WebRTC transports
- Manages peer connections, discovery, and lifecycle across dual transports
- Implements transport abstraction for Mina-specific blockchain protocols
- Handles network topology maintenance and peer space management
- Coordinates encrypted communication and protocol negotiation

## Architecture Layers

- **Network Layer**: Low-level protocol implementations (Noise encryption, Yamux
  multiplexing, Kademlia DHT, etc.)
- **Channels Layer**: Transport abstraction for Mina-specific protocols
  (transactions, SNARKs, RPC, signaling)
- **Connection Layer**: Peer connection lifecycle management for
  incoming/outgoing connections
- **P2P Orchestration**: Top-level coordination, peer management, and system
  integration

## Dual Transport Support

- **libp2p Backend**: Server-to-server communication with gossip/pubsub
  broadcasting
- **WebRTC Browser**: Direct peer connections with request/response patterns
- **Transport Abstraction**: Unified API that adapts between push-based and
  pull-based paradigms
- **Protocol Adaptation**: Automatic routing based on peer capabilities and
  connection types

## Key Components

- **Connection Management**: Handles incoming/outgoing connection state machines
  and lifecycle
- **Channels**: Transport-agnostic communication for transactions, SNARKs, RPC,
  and blockchain data
- **Network Protocols**: Encryption (Noise), multiplexing (Yamux), discovery
  (Kademlia), messaging (Pubsub)
- **Disconnection**: Automated peer space management with stability protection
  and cleanup
- **Peer Management**: Individual peer state tracking across multiple transport
  connections

## Interactions

- Connects to bootstrap and peer nodes via libp2p and WebRTC
- Propagates blocks, transactions, and SNARK data across the network
- Handles peer discovery via Kademlia DHT with bootstrap coordination
- Manages connection limits with automated peer space management
- Routes protocol messages to appropriate business logic handlers
- Provides callbacks for decoupled integration with node components

## Integration

- **Business Logic**: Interfaces with transaction pool, SNARK pool, blockchain
  state, and block production
- **Transport Services**: Abstracts libp2p and WebRTC operations through service
  traits
- **Configuration**: Supports private networks, connection limits, and protocol
  versioning

## Technical Debt

- **Security**: Noise encryption component needs session key zeroization for
  defense-in-depth
- **Performance**: Pubsub component has O(n) lookups and monolithic 963-line
  reducer
- **Architecture**: Several components have large monolithic reducers that need
  refactoring
- **Partial Migration**: P2P components are partially migrated to new patterns -
  some components still have effects files with business logic beyond just
  service invocations
