# Adding RPC Endpoints Guide

This guide explains how to navigate the codebase when adding new RPC endpoints
to OpenMina, focusing on the files and architectural patterns specific to RPC
functionality.

## Prerequisites

Before adding RPC endpoints, understand:

- [State Machine Development Guide](state-machine-development-guide.md) - Core
  Redux patterns
- [Architecture Walkthrough](architecture-walkthrough.md) - Overall system
  design
- [Services](services.md) - Service layer integration

## RPC Architecture Overview

OpenMina's RPC system follows this flow:

```
HTTP Request → RPC Action → RPC Reducer → RPC Effect → Service → Response
```

The RPC layer uses the same Redux patterns as other components but adds HTTP
server integration and service response handling. This guide shows you where to
make changes for each step.

## Files to Modify

1. **Request/Response Types** (`node/src/rpc/mod.rs`) - Add to `RpcRequest` enum
   and define response type alias
2. **RPC Actions** (`node/src/rpc/rpc_actions.rs`) - Add variant with `rpc_id`
   field and enabling condition
3. **RPC Reducer** (`node/src/rpc/rpc_reducer.rs`) - Implement business logic
   and dispatch effectful action with response data
4. **Effectful Action** (`node/src/rpc_effectful/rpc_effectful_action.rs`) - Add
   variant that carries the response data
5. **Effects** (`node/src/rpc_effectful/rpc_effectful_effects.rs`) - Thin
   wrapper that calls service respond method
6. **Service Interface** (`node/src/rpc_effectful/rpc_service.rs`) - Add
   `respond_*` method to `RpcService` trait
7. **Service Implementation** (`node/native/src/http_server.rs`) - Implement
   service method (usually just calls `self.respond`)
8. **HTTP Routes** (`node/native/src/http_server.rs`) - Add endpoint to
   `rpc_router` function

## Reference Examples

Study these existing endpoints in the codebase to understand the patterns:

**Simple Endpoints:**

- `StatusGet` - Basic node status information
- `HeartbeatGet` - Simple health check
- `SyncStatsGet` - Component statistics

**Parameterized Endpoints:**

- `ActionStatsGet` - Takes query parameters
- `LedgerAccountsGet` - Filtered data retrieval

**Streaming Endpoints:**

- Look at WebRTC signaling endpoints for `multishot_request` patterns

Each endpoint follows the same 8-step pattern above. The business logic in
effects varies, but the structural pattern is consistent.

## Key RPC-Specific Patterns

### Request/Response Flow

- HTTP requests come in via `oneshot_request` or `multishot_request`
- RPC actions include an `rpc_id` field for tracking
- Effects call service `respond_*` methods to send responses back
- The framework handles request state tracking automatically

### Service Layer Integration

- The `RpcService` trait abstracts over different transport mechanisms (HTTP,
  WASM)
- HTTP implementation is in `node/native/src/http_server.rs`
- WASM bindings are in `node/common/src/service/rpc/sender.rs`

### Streaming vs Single Responses

- Use `oneshot_request` for endpoints that return one response
- Use `multishot_request` for endpoints that return multiple responses over time
- Most endpoints use `oneshot_request`

## WASM Frontend Integration

WASM bindings are in `node/common/src/service/rpc/` organized across multiple
helper structs (`State`, `Stats`, `Ledger`, etc.) plus direct methods on
`RpcSender`. Check `frontend/src/app/core/services/web-node.service.ts` to see
how they're accessed from the frontend (e.g., `webnode.state().peers()`,
`webnode.stats().sync()`).

## Testing

Test RPC endpoints with curl against the HTTP server:

```bash
curl http://localhost:3000/your-endpoint
```

The RPC layer follows standard OpenMina Redux patterns with the addition of HTTP
routing and service response handling. Study existing endpoints to understand
the complete flow.
