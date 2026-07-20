# Transition Frontier Candidate State Machine

Manages incoming block candidates through multi-stage validation and
consensus-based ordering to determine the best verified block for transition
frontier updates.

## Purpose

- Receives and validates incoming candidate blocks from P2P network
- Orders candidates using consensus rules (worst to best) in priority queue
- Manages multi-stage validation pipeline for block verification
- Tracks chain proofs for fork validation and consensus decisions
- Maintains invalid block blacklist to prevent reprocessing

## Multi-Stage Validation Pipeline

```
BlockReceived → Prevalidated → SnarkVerifyPending → SnarkVerifySuccess
```

1. **Received** - initial candidate block received from network
2. **Prevalidated** - basic block structure and consensus validation passed
3. **SnarkVerifyPending** - block SNARK proof verification in progress
4. **SnarkVerifySuccess** - SNARK proof verified successfully

## Consensus-Based Ordering

- **Priority queue** - maintains `BTreeSet` ordered by consensus rules (worst to
  best)
- **Best candidate selection** - identifies highest priority verified candidate
- **Fork decision support** - provides ordered candidates for consensus
  evaluation
- **Pruning** - removes candidates worse than best verified candidate

## Chain Proof Management

- **Chain proof collection** - gathers proof chains for fork validation
- **Automatic chain proof derivation** - constructs proofs from existing
  transition frontier
- **Fork validation support** - provides necessary proofs for consensus fork
  decisions

## Invalid Block Tracking

- **Blacklist maintenance** - tracks blocks that failed validation permanently
- **Memory optimization** - moves failed blocks to lightweight invalid tracking
- **Reprocessing prevention** - avoids re-validating known invalid blocks
- **Slot-based pruning** - removes old invalid blocks based on finality

## Key Features

- **Consensus ordering** - uses `consensus_take` function for accurate priority
  ordering
- **Memory efficient** - prunes worse candidates and optimizes invalid block
  storage
- **Fork decision ready** - provides best verified candidate for transition
  frontier sync
- **Multi-peer resilience** - handles candidates from multiple peers
  independently
- **Chain proof optimization** - derives chain proofs automatically when
  possible

## Integration Points

- **SNARK verification** - coordinates with SNARK block verify service
- **Transition frontier sync** - triggers sync when better candidate is found
- **Block prevalidation** - integrates with block prevalidation logic
- **Consensus evaluation** - provides candidates for fork decision algorithms

## State Management

- **Deterministic ordering** - maintains consistent candidate priority across
  restarts
- **Status tracking** - tracks validation progress for each candidate
  independently
- **Chain proof caching** - stores and reuses chain proofs across validation
  stages
- **Best candidate identification** - efficiently identifies best verified block
  for sync initiation
