# Kademlia Bootstrap State Machine

Manages the iterative FIND_NODE process to discover peers closest to the local
node's key for initial DHT integration.

## Purpose

- **Iterative peer discovery** - Executes FIND_NODE requests to discover peers
  closest to local node key
- **Concurrent request management** - Maintains up to 3 concurrent requests with
  rate limiting
- **Statistics collection** - Tracks success/failure rates and timing for
  bootstrap requests
- **Routing table population** - Processes discovered peers to populate Kademlia
  routing table

## State Flow

```
CreateRequests → AppendRequest (×3) → FinalizeRequests → RequestDone/RequestError → CreateRequests
```

## Key Features

- **Batched request processing** - Groups up to 3 requests per batch with
  completion synchronization
- **Closest peer selection** - Uses routing table to select unprocessed peers
  closest to local Kademlia key
- **Request deduplication** - Tracks processed peers in `BTreeSet` to avoid
  redundant requests
- **Success threshold** - Continues until 20 successful requests or peer
  exhaustion
- **Fallback address handling** - Stores backup addresses for connection retry
  logic

## Integration Points

- **P2pNetworkKadEffectfulAction::MakeRequest** - Initiates connection attempts
  to discovered peers
- **P2pNetworkKadRequestAction::New** - Creates FIND_NODE requests for connected
  peers
- **P2pNetworkKademliaAction::BootstrapFinished** - Signals completion when no
  more requests available
- **Routing table access** - Queries closest peers and updates with discovered
  nodes

## Technical Implementation

- **Kademlia key mapping** - Converts PeerId to SHA256-based Kademlia key for
  distance calculations
- **Multi-phase processing** - CreateRequests → AppendRequest → FinalizeRequests
  cycle
- **Concurrent limiting** - Enabling conditions enforce maximum 3 concurrent
  requests
- **Statistics tracking** - Records ongoing, successful, and failed request
  metrics with timestamps

## Technical Debt

- TODO: Replace BTreeMap-based request tracking with lightweight alternative for
  3 concurrent requests
- TODO: Generalize to DNS addresses instead of just SocketAddr
- TODO: Use Multiaddr instead of SocketAddr for address handling
