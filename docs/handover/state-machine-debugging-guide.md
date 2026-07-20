# State Machine Debugging Guide

This guide provides comprehensive tools and techniques for troubleshooting and
investigating issues in OpenMina's state machine architecture.

## Prerequisites

Before using this guide, understand:

- [Architecture Walkthrough](architecture-walkthrough.md) - Core concepts and
  patterns
- [State Machine Development Guide](state-machine-development-guide.md) -
  Implementation basics
- [State Machine Structure](state-machine-structure.md) - System organization

> **Related Guides**: [Testing Infrastructure](testing-infrastructure.md),
> [Services Technical Debt](services-technical-debt.md),
> [State Machine Technical Debt](state-machine-technical-debt.md)

## Action Tracing and Logging

### Understanding the ActionEvent Macro

The `ActionEvent` derive macro generates structured logging for actions in
OpenMina. It automatically creates log events when actions are dispatched,
integrating with the tracing framework for efficient debugging.

**Basic Usage (from actual codebase):**

```rust
#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum BlockProducerAction {
    VrfEvaluator(BlockProducerVrfEvaluatorAction),
    BestTipUpdate { best_tip: ArcBlockWithHash },
}
```

**Setting Default Log Levels:**

```rust
#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(level = info)]  // Default for all variants
pub enum BlockProducerAction {
    WonSlotSearch,  // Uses info level

    #[action_event(level = trace)]  // Override for specific variant
    BlockInject,
}
```

**Field Extraction (real examples from codebase):**

```rust
// Simple field inclusion
#[action_event(level = info, fields(slot, current_time))]
WonSlot {
    slot: u32,
    current_time: Timestamp,
}

// Complex field expressions (from VRF evaluator)
#[action_event(
    level = info,
    fields(
        slot = won_slot.global_slot.slot_number.as_u32(),
        slot_time = openmina_core::log::to_rfc_3339(won_slot.slot_time)
            .unwrap_or_else(|_| "<error>".to_owned()),
    )
)]
WonSlot {
    won_slot: BlockProducerWonSlot,
}

// Display formatting
#[action_event(level = info, fields(display(chain_id)))]
Initialize { chain_id: openmina_core::ChainId },
```

**Automatic Level Assignment:**

- Actions ending in `Error` or `Warn` automatically get `warn` level
- Default level is `debug` if not specified
- Enum-level `#[action_event(level = X)]` sets default for all variants

**Documentation Integration:**

```rust
/// Initializes p2p layer.
#[action_event(level = info)]
Initialize { chain_id: ChainId },
```

Doc comments become `summary = "Initializes p2p layer"` in log events.

### Using Log Levels for Debugging

**Environment Variable Control:**

```bash
# See everything (expensive, use sparingly)
OPENMINA_TRACING_LEVEL=trace cargo run --release -p cli node

# Development debugging
OPENMINA_TRACING_LEVEL=debug cargo run --release -p cli node

# Production logging
OPENMINA_TRACING_LEVEL=info cargo run --release -p cli node

# Only warnings and errors
OPENMINA_TRACING_LEVEL=warn cargo run --release -p cli node
```

**Level Guidelines (based on actual usage):**

- **trace** - Very frequent actions (use sparingly due to performance)
- **debug** - Regular operations during development
- **info** - Important business events suitable for production
- **warn** - Error conditions and anomalies

**Debugging Strategy:**

1. Start with `OPENMINA_TRACING_LEVEL=info` to see important events
2. Increase specific component actions to `debug` level in code
3. Use `OPENMINA_TRACING_LEVEL=debug` to see those specific actions
4. Only use `trace` level for short debugging sessions

## Recording and Replay

### Record Execution

OpenMina supports recording execution for deterministic debugging:

```bash
# Record input actions and initial state for replay
./target/release/openmina node --record state-with-input-actions

# No recording (default)
./target/release/openmina node --record none
```

The recorded data is stored in `~/.openmina/recorder/` and includes:

- Initial state snapshot with RNG seed and P2P secret key
- All input actions (timeouts and external events that drive state changes)

### Replay Debugging Sessions

Replay previously recorded sessions to reproduce issues deterministically:

```bash
# Replay a recorded session
./target/release/openmina replay-state-with-input-actions --dir ~/.openmina/recorder

# Replay with verbose output for debugging
./target/release/openmina replay-state-with-input-actions --verbosity debug --dir ~/.openmina/recorder

# Ignore build environment mismatches if needed
./target/release/openmina replay-state-with-input-actions --ignore-mismatch --dir ~/.openmina/recorder
```

**Programmatic replay usage:**

```rust
// Read recorded session
let reader = StateWithInputActionsReader::new("~/.openmina/recorder");
let initial_state = reader.read_initial_state()?;

// Replay actions step by step
for (path, actions) in reader.read_actions() {
    // Process recorded actions to reproduce bug
}
```

## Network Analysis Tools

### P2P Connection Analysis

Network debugging in OpenMina is primarily done through structured logging and
the testing framework:

**For P2P debugging, use:**

- Action tracing with `OPENMINA_TRACING_LEVEL=debug` to see P2P events
- Component-specific logging for connection and message flow analysis
- Testing framework tools for controlled network scenario testing

### Protocol Message Analysis

**Available through logging:**

- Connection lifecycle events via P2P action traces
- Kademlia DHT operations in debug logs
- Gossipsub message propagation events
- Stream multiplexing details in trace-level logs

**For advanced network analysis:**

- The testing framework in `node/testing/` includes network debugging
  capabilities
- The `bpf-recorder` tool provides packet-level analysis in test scenarios
- See [Testing Infrastructure](testing-infrastructure.md) for testing-specific
  debugging

## Testing Framework

OpenMina has comprehensive testing infrastructure for state machines. For
detailed information, see [Testing Infrastructure](testing-infrastructure.md).

**Available Testing Approaches:**

- **Unit Testing** - Test individual actions and reducers
- **Scenario-Based Testing** - Test component workflows
- **Multi-Node Simulation** - Test distributed behavior
- **Fuzzing** - Test with random inputs
- **Differential Testing** - Compare against OCaml implementation

**Basic Unit Test Pattern:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enabling_conditions() {
        let state = create_test_state();
        let action = YourComponentAction::Process { data: test_data() };
        assert!(action.is_enabled(&state, Timestamp::ZERO));
    }
}
```

## Common Error Patterns

### Enabling Condition Mismatches

**Problem:** `bug_condition!` triggers indicate enabling conditions don't match
reducer assumptions.

**What `bug_condition!` Does:**

- Defensive programming macro for unreachable code paths
- In development (`OPENMINA_PANIC_ON_BUG=true`): Panics to catch bugs early
- In production (default): Logs error and continues gracefully
- Should only trigger if enabling conditions have bugs

**Example:**

```rust
// Enabling condition allows action
impl EnablingCondition<State> for MyAction {
    fn is_enabled(&self, state: &State, _time: Timestamp) -> bool {
        state.my_component.is_ready()  // Returns true
    }
}

// But reducer expects different state
MyAction::Process { data } => {
    let Some(processor) = &state.processor else {
        bug_condition!("Process action enabled but no processor available");
        return;
    };
    // This bug_condition! indicates a mismatch
}
```

**Solution:** Align enabling condition with reducer expectations:

```rust
impl EnablingCondition<State> for MyAction {
    fn is_enabled(&self, state: &State, _time: Timestamp) -> bool {
        state.my_component.is_ready() && state.processor.is_some()
    }
}
```

### State Machine Initialization Issues

**Problem:** Actions dispatched before component is ready.

**Example:**

```rust
// Action reaches reducer before initialization
Action::P2p(p2p_action) => match &mut state.p2p {
    P2p::Pending(_) => {
        // This indicates premature action dispatch
        error!(meta.time(); "p2p not initialized", action = debug(p2p_action));
    }
    P2p::Ready(_) => {
        // Process action normally
    }
}
```

**Solution:** Use initialization state in enabling conditions:

```rust
impl EnablingCondition<State> for P2pAction {
    fn is_enabled(&self, state: &State, _time: Timestamp) -> bool {
        matches!(state.p2p, P2p::Ready(_))
    }
}
```

### Invalid State Transitions

**Problem:** Attempting impossible state transitions.

**Example:**

```rust
// Multiple initialization attempts
YourAction::Init { config } => {
    if state.is_initialized() {
        bug_condition!("Already initialized but Init action enabled");
        return;
    }
    state.initialize(config);
}
```

**Solution:** Use state enums to enforce valid transitions:

```rust
#[derive(Debug, Clone)]
pub enum YourComponentState {
    Uninitialized,
    Initializing { config: Config },
    Ready { data: ComponentData },
    Error { error: String },
}

// Enabling condition prevents invalid transitions
impl EnablingCondition<State> for YourAction {
    fn is_enabled(&self, state: &State, _time: Timestamp) -> bool {
        match self {
            YourAction::Init { .. } => {
                matches!(state.your_component, YourComponentState::Uninitialized)
            }
            YourAction::Process { .. } => {
                matches!(state.your_component, YourComponentState::Ready { .. })
            }
        }
    }
}
```

### Service Communication Errors

**Problem:** External services not responding or returning unexpected data.

**Example:**

```rust
// Handle service failures gracefully
YourAction::ServiceError { error } => {
    warn!(meta.time(); "service call failed", error = display(error));

    // Update state to reflect failure
    state.status = YourComponentStatus::Error {
        error: error.to_string()
    };

    // Dispatch retry or fallback action
    let dispatcher = state_context.into_dispatcher();
    dispatcher.push(YourAction::RetryOrFallback);
}
```

**Solution:** Implement robust error handling:

```rust
// In effectful actions
YourEffectfulAction::ExternalRequest { params } => {
    // Service call with timeout and retry logic
    store.service.call_with_retry(params, max_retries, timeout);
}

// In service implementation
impl YourService for ServiceImpl {
    fn call_with_retry(&self, params: Params, max_retries: u32, timeout: Duration) {
        // Implement retry logic with exponential backoff
        // Send success or failure events back to state machine
    }
}
```

### Mixing Stateful and Effectful Logic

**Problem:** Putting business logic in effects instead of reducers.

**Wrong:**

```rust
// Business logic in effects - DON'T DO THIS
impl YourEffectfulAction {
    pub fn effects(&self, store: &mut Store) {
        if complex_business_condition {
            // Complex logic here violates architecture
            if some_other_condition {
                store.dispatch(SomeAction);
            }
        }
        store.service.call_external();
    }
}
```

**Right:**

```rust
// Business logic in reducers
YourAction::ProcessRequest { request_id, data } => {
    let Ok(state) = state_context.get_substate_mut() else {
        // TODO: log or propagate
        return;
    };

    // State updates (enabling condition already verified this is valid)
    state.status = Status::Processing { request_id };
    state.pending_requests.push_back(PendingRequest {
        id: request_id,
        data: data.clone(),
        timestamp: meta.time(),
    });

    // Prepare and dispatch effectful action
    let dispatcher = state_context.into_dispatcher();
    dispatcher.push(YourEffectfulAction::ExternalCall {
        request_id,
        params: data.into_params(),
    });
}

// Thin effects wrapper
impl YourEffectfulAction {
    pub fn effects(&self, store: &mut Store) {
        match self {
            YourEffectfulAction::ExternalCall { params } => {
                // Only service calls, no business logic
                store.service.call_external(params);
            }
        }
    }
}
```

## Debugging Best Practices

1. **Start with `info` level logs** to understand the overall flow
2. **Use `debug` level selectively** for components under investigation
3. **Record execution** for reproducible debugging sessions
4. **Write enabling conditions** that match reducer logic exactly
5. **Use `bug_condition!`** for invariant checking in development
6. **Test with scenarios** that cover your component's edge cases
7. **Use structured logging and testing tools** for P2P communication issues
8. **Check technical debt** in `summary.md` files for known issues
