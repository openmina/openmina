# RPC State Machine

Provides JSON-RPC over HTTP interface exposing node functionality for external
clients.

## Purpose

- **External API Gateway**: Exposes blockchain state, transactions, and network
  information via JSON-RPC over HTTP
- **Request Lifecycle Management**: Tracks requests through 4-phase lifecycle
  with unique IDs and timestamps
- **State Query Interface**: Provides filtered access to node state and
  blockchain data
- **Service Coordination**: Routes complex operations to appropriate backend
  services

## Architecture

### Core State Management

- **RpcState**: `BTreeMap<RpcId, RpcRequestState>` tracking active requests
- **Request Lifecycle**: Init → Pending → Success/Error states with timestamps
- **Request Correlation**: Unique `RpcId` for matching requests to responses
- **Extra Data Storage**: Optional request-specific data for complex operations

### Request Processing Patterns

- **Direct State Access**: Simple queries read directly from Redux state
- **Service Delegation**: Complex operations delegated to ledger, SNARK, or P2P
  services
- **Async Coordination**: Callback system handles asynchronous service responses
- **Request Cleanup**: Automatic state cleanup after response delivery

### API Categories

- **Node Information**: Status, heartbeat, health checks
- **Blockchain Data**: Blocks, chains, genesis information, consensus parameters
- **Transaction Operations**: Injection, status queries, pool monitoring
- **Account/Ledger Queries**: Account information, balances, delegators
- **Statistics**: Action stats, sync progress, performance metrics
- **Network Operations**: Peer management, connection handling
- **SNARK Operations**: Proof jobs, worker coordination

## Service Integration

### Request Routing

- **Ledger Service**: Account queries, scan state operations
- **Transaction Pool**: Transaction injection and pool queries
- **SNARK Pool**: Proof job management and worker coordination
- **P2P Network**: Peer information and connection management

### HTTP Server Integration

- **RESTful Endpoints**: Standard HTTP API for common operations
- **WebRTC Signaling**: P2P connection establishment support
- **JSON Serialization**: Comprehensive data type serialization
- **Error Handling**: Structured error responses with proper HTTP status codes

## Technical Debt

### Data Type Improvements

- **Type System**: Several fields use `String` instead of proper typed
  representations for hashes and memos
- **Command Handling**: Incomplete enum handling for all user command types
- **Hash Operations**: Missing error handling for hash conversion failures

### Code Cleanup

- **Legacy Code**: Commented transaction injection code marked for removal
- **TODO Items**: Various type refinements and error handling improvements
  needed
