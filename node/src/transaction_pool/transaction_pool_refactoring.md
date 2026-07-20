# Transaction Pool Refactoring Notes

This document outlines architectural improvements needed to align the
transaction pool component with the standard state machine patterns used
throughout the OpenMina codebase.

## Current Implementation Issues

### 1. Pending Actions Pattern

The component uses an unconventional pattern where actions are stored in
`pending_actions` and retrieved later:

```rust
// Current pattern
let pending_id = substate.make_action_pending(action);
// ... later ...
let action = substate.pending_actions.remove(pending_id).unwrap()
```

This pattern appears in:

- `StartVerify` → `StartVerifyWithAccounts`
- `ApplyVerifiedDiff` → `ApplyVerifiedDiffWithAccounts`
- `ApplyTransitionFrontierDiff` → `ApplyTransitionFrontierDiffWithAccounts`
- `BestTipChanged` → `BestTipChangedWithAccounts`

**Issue**: This breaks the standard Redux pattern where state should represent
the current state, not store actions.

### 2. Blocking Service Call

The ledger service call in `transaction_pool_effects.rs` is synchronous:

```rust
let accounts = match store
    .service()
    .ledger_manager()
    .get_accounts(&ledger_hash, account_ids.iter().cloned().collect())
```

**Issue**: This blocks the state machine thread, violating the principle of
async service interactions.

### 3. Direct Global State Access

Uses `unsafe_get_state()` to access global state:

```rust
Self::global_slots(state.unsafe_get_state())
```

**Issue**: Components should receive necessary data through actions or maintain
it in their local state.

### 4. Complex Multi-Step Flows

The current implementation has implicit multi-step flows that are hard to follow
and test.

## Proposed Solution

### 1. Replace Pending Actions with Explicit State Machine

Model the verification flow as explicit states:

```rust
pub enum VerificationState {
    Idle,
    FetchingAccounts {
        commands: Vec<TransactionWithHash>,
        from_source: TransactionPoolMessageSource,
        request_id: LedgerRequestId,
    },
    Verifying {
        commands: Vec<TransactionWithHash>,
        accounts: BTreeMap<AccountId, Account>,
        from_source: TransactionPoolMessageSource,
        verify_id: SnarkUserCommandVerifyId,
    },
}

pub enum DiffApplicationState {
    Idle,
    FetchingAccounts {
        diff: DiffVerified,
        best_tip_hash: LedgerHash,
        from_source: TransactionPoolMessageSource,
        request_id: LedgerRequestId,
    },
    Applying {
        diff: DiffVerified,
        accounts: BTreeMap<AccountId, Account>,
        from_source: TransactionPoolMessageSource,
    },
}
```

### 2. Implement Async Ledger Service Pattern

Convert to event-based pattern:

```rust
// In effects:
TransactionPoolEffectfulAction::FetchAccounts {
    request_id,
    account_ids,
    ledger_hash
} => {
    store.service().ledger_fetch_accounts(
        request_id,
        ledger_hash,
        account_ids,
    );
}

// In event source (or future distributed event handling):
Event::Ledger(LedgerEvent::AccountsFetched { request_id, accounts }) => {
    store.dispatch(TransactionPoolAction::AccountsFetched {
        request_id,
        accounts
    });
}
```

### 3. Update Reducer to Handle State Transitions

Example for verification flow:

```rust
TransactionPoolAction::StartVerify { commands, from_source } => {
    let request_id = LedgerRequestId::new();

    // Set state
    substate.verification_state = VerificationState::FetchingAccounts {
        commands: commands.clone(),
        from_source: *from_source,
        request_id,
    };

    // Dispatch async request
    let account_ids = /* extract account ids */;
    dispatcher.push(TransactionPoolEffectfulAction::FetchAccounts {
        request_id,
        account_ids,
        ledger_hash: substate.best_tip_hash.clone().unwrap(),
    });
}

TransactionPoolAction::AccountsFetched { request_id, accounts } => {
    match &substate.verification_state {
        VerificationState::FetchingAccounts {
            commands,
            from_source,
            request_id: expected_id
        } if request_id == expected_id => {
            // Transition to verifying state
            // Dispatch SNARK verification
        }
        _ => {} // Ignore if not expecting this response
    }
}
```

### 4. Maintain Required State Locally

```rust
pub struct TransactionPoolState {
    // ... existing fields ...
    current_global_slot: Option<(u32, u32)>,
}

// Update via action when global slot changes
TransactionPoolAction::GlobalSlotChanged { slot, slot_since_genesis } => {
    substate.current_global_slot = Some((*slot, *slot_since_genesis));
}
```

## Benefits

1. **Predictable State**: State represents what's happening, not stored actions
2. **Non-blocking**: Async ledger calls don't block the state machine
3. **Testable**: Each state transition can be tested independently
4. **Standard Pattern**: Aligns with other OpenMina components
5. **Clear Flow**: State machine makes the flow explicit and debuggable

## Migration Strategy

1. Start by converting the ledger service to async pattern
2. Introduce state enums alongside existing pending_actions
3. Gradually migrate each flow to use state transitions
4. Remove pending_actions once all flows are migrated
5. Remove unsafe_get_state usage

## Related Files

- `transaction_pool_reducer.rs` - Main reducer implementation
- `transaction_pool_effects.rs` - Effectful actions
- `transaction_pool_state.rs` - State definition
