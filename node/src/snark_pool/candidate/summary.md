# SNARK Pool Candidate State Machine

Manages incoming SNARK work from peers through a multi-stage validation pipeline
before promoting verified work to the main pool.

## Purpose

- Coordinates P2P discovery and fetching of SNARK work from network peers
- Manages per-peer candidate work state tracking and progression
- Batches SNARK work for efficient verification processing
- Promotes verified work to main pool while removing inferior candidates
- Maintains quality control through fee-based prioritization

## Multi-Stage Validation Pipeline

```
InfoReceived → WorkFetchPending → WorkReceived → WorkVerifyPending → WorkVerifySuccess/Error
```

1. **InfoReceived** - peer announces available SNARK work with job ID and fee
   information
2. **WorkFetchPending** - requests full SNARK work from peer via P2P RPC
3. **WorkReceived** - complete SNARK work received and ready for verification
4. **WorkVerifyPending** - SNARK work submitted to verification service in
   batches
5. **WorkVerifySuccess/Error** - verification completed with success or failure
   result

## Per-Peer State Tracking

- **Dual indexing** - maintains work by peer (`by_peer`) and by job ID
  (`by_job_id`)
- **Consistency checking** - validates index consistency with `check()` method
- **Peer lifecycle** - handles peer connections and disconnections gracefully
- **Work comparison** - only accepts better work (higher fees) for same job

## Priority-Based Work Fetching

- **Order-based prioritization** - fetches work based on job order (priority)
- **Fee-based secondary ordering** - higher fees take precedence within same
  order
- **Deduplication** - only fetches one work per job order to avoid redundancy
- **Available peer filtering** - only requests from peers with RPC capacity

## Batch Verification Processing

- **Job ID ordering** - processes verification in priority order
- **Per-peer batching** - groups work from same peer for efficient verification
- **State coordination** - tracks verification requests and results across
  batches
- **Error handling** - manages verification failures without affecting other
  work

## Quality Control Features

- **Superior work filtering** - `remove_inferior_snarks()` removes lower-fee
  work for same job
- **Fee validation** - ensures only competitive work progresses through pipeline
- **Work comparison** - implements `SnarkCmp` for consistent quality assessment
- **Retention policies** - supports custom filtering for stale or invalid
  candidates

## Integration Points

- **P2P RPC system** - fetches complete SNARK work from peer announcements
- **SNARK verification service** - batches work for proof validation
- **Main pool coordination** - promotes verified work and removes inferior
  candidates
- **Peer management** - integrates with P2P lifecycle for connection handling

## State Management

- **Deterministic progression** - clear state transitions with enabling
  conditions
- **Concurrent peer handling** - manages work from multiple peers independently
- **Memory efficiency** - cleans up completed/failed candidates automatically
- **Verification coordination** - tracks verification IDs to correlate results
  with requests

## Key Features

- **Two-way indexing** - efficient lookups by peer or job ID
- **Fee-based competition** - ensures highest quality work reaches main pool
- **Batch processing** - optimizes verification throughput through batching
- **Peer resilience** - handles peer disconnections without losing valid work
- **Priority ordering** - respects job priorities for systematic work processing
