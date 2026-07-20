# VRF Evaluator State Machine

Evaluates Verifiable Random Function outputs to determine slot leadership
eligibility through epoch-aware evaluation processes.

## Purpose

- **VRF Slot Leadership**: Calculates VRF outputs to determine block production
  eligibility based on stake delegation
- **Epoch Management**: Handles 7,140-slot epochs (≈3-minute slots) with
  Current/Next/Waiting epoch context transitions
- **Stake Calculations**: Implements threshold calculations using delegated
  stake percentages and epoch-specific staking ledgers
- **Won Slot Tracking**: Maintains `BTreeMap<u32, VrfWonSlotWithHash>` for
  efficient slot lookup and history retention

## Architecture

### Core State Structure

- **BlockProducerVrfEvaluatorState**: Contains evaluation status, won slots
  cache, epoch tracking, and context management
- **Won Slots Storage**: `BTreeMap<u32, VrfWonSlotWithHash>` providing O(log n)
  slot lookup with range querying
- **Epoch Context**: `EpochContext` enum handling Current/Next/Waiting states
- **Evaluation Tracking**: `latest_evaluated_slot` and `last_evaluated_epoch`
  for incremental processing

### Status State Flow

```
Idle → InitialisationPending → InitialisationComplete → ReadinessCheck →
ReadyToEvaluate → EpochDataPending → InitialSlotSelection →
DelegatorTableBuilding → SlotEvaluating → SlotEvalComplete
```

### Epoch Context Management

- **Current(EpochData)**: Evaluating current epoch with current staking ledger
- **Next(EpochData)**: Evaluating next epoch with next staking ledger (requires
  finalized epoch seed)
- **Waiting**: Waiting for epoch data availability or seed finalization

## Key Algorithms

### Epoch Boundary Detection (`evaluate_epoch_bounds`)

```rust
if global_slot % SLOTS_PER_EPOCH == 0 {
    SlotPositionInEpoch::Beginning
} else if (global_slot + 1) % SLOTS_PER_EPOCH == 0 {
    SlotPositionInEpoch::End
} else {
    SlotPositionInEpoch::Within
}
```

### Won Slot Lookup (`next_won_slot`)

- **Range Query**: `won_slots.range(cur_global_slot..)` for future slots
- **Chain Validation**: Filters slots valid for extending current best tip
- **Genesis Timestamp**: Converts VRF slots to blockchain timestamps

## Service Integration

### Ledger Service

- **Staking Data**: Reads delegation tables and account balances for VRF
  calculations
- **Epoch Data**: Fetches epoch seeds, stake distributions, and delegation
  mappings
- **Delegator Tables**: Builds stake lookup structures for VRF evaluation

### External VRF Service

- **Input Construction**: Creates `VrfEvaluatorInput` with epoch data and slot
  parameters
- **Cryptographic Evaluation**: Delegates VRF computation to external proof
  services
- **Result Processing**: Handles `VrfEvaluationOutput` to determine wins and
  update caches

## Implementation Details

### Multi-Phase Evaluation

1. **Readiness Check**: Validates epoch data and seed finalization
2. **Context Selection**: Chooses Current/Next/Waiting based on evaluation state
3. **Delegator Table Building**: Constructs stake lookup structures
4. **Incremental Evaluation**: Processes slots from `latest_evaluated_slot`
5. **Won Slot Validation**: Filters based on chain context and timing

### Resource Management

- **Incremental Processing**: Evaluates slots progressively rather than batch
- **Cache Maintenance**: Retains won slots across epochs with automatic cleanup
- **Context Switching**: Transitions between epoch evaluation contexts
- **Memory Bounds**: Prunes stale slots based on epoch transitions

## Technical Debt

### Critical Issues

- **Unimplemented Epoch Context**: `todo!()` for `EpochContext::Waiting` state
  in `SelectInitialSlot` action handler, indicating incomplete epoch transition
  handling when epoch data is not yet available
