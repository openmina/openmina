# State Machine Development Guide

This guide provides practical knowledge for developers working with OpenMina's
state machine architecture. It focuses on common development patterns and
workflows for implementing features within the Redux-style state management
system.

## Prerequisites

Before using this guide, read:

- [Architecture Walkthrough](architecture-walkthrough.md) - Core concepts and
  patterns
- [State Machine Structure](state-machine-structure.md) - Action/reducer
  organization
- [State Machine Patterns](state-machine-patterns.md) - Common patterns and when
  to use them
- [Project Organization](organization.md) - Codebase navigation
- Main [README](../../README.md) - Building and running the project

> **Related Guides**: [Adding RPC Endpoints](adding-rpc-endpoints.md),
> [State Machine Debugging Guide](state-machine-debugging-guide.md),
> [Testing Infrastructure](testing-infrastructure.md)

## Making Your First Changes

### Choose the Right Pattern

Before implementing a new state machine, determine which pattern fits your use
case:

**For Async Operations** (most common):

- Use **Pure Lifecycle Pattern** (Init → Pending → Success/Error)
- Examples: Network requests, proof generation, data loading
- See [State Machine Patterns](state-machine-patterns.md#pure-lifecycle-pattern)
  for examples

**For Multi-Phase Operations**:

- Use **Sequential Lifecycle Pattern** or **Connection Lifecycle Pattern**
- Examples: Sync operations, P2P handshakes, protocol negotiations
- See
  [State Machine Patterns](state-machine-patterns.md#sequential-lifecycle-pattern)
  for examples

**For Complex Workflows**:

- Use **Hybrid Patterns** or **Iterative Process Pattern**
- Examples: Block production, VRF evaluation, long-running computations
- See
  [State Machine Patterns](state-machine-patterns.md#hybrid-lifecycle--domain-specific-patterns)
  for examples

### Finding the Right Component

When implementing a feature or fixing a bug, locate the relevant state machine:

**By Feature Domain:**

- **P2P networking** → `p2p/src/`
- **Block production** → `node/src/block_producer/`
- **Transaction processing** → `node/src/transaction_pool/`
- **Ledger operations** → `node/src/ledger/`
- **SNARK verification** → `snark/src/`
- **Consensus logic** → `node/src/transition_frontier/`

**By Action Type:**

1. **Search for existing actions** - Use `rg "SomeAction"` to find similar
   functionality
2. **Follow state flow** - Look at state definitions to understand data flow
3. **Check `summary.md`** - Most components have purpose and technical debt
   notes

**Example: Adding transaction validation**

```bash
# Find transaction-related actions
rg "TransactionPool.*Action" --type rust

# Look at transaction pool state
cat node/src/transaction_pool/transaction_pool_state.rs

# Check component documentation
cat node/src/transaction_pool/summary.md
```

### Code Change Workflow

**1. Understand the Existing Pattern**

- Find similar functionality in the same component
- Note the action → reducer → effect flow
- Check enabling conditions and state transitions

**2. Follow Component Conventions**

- Use existing naming patterns for actions and state
- Match the style of enabling conditions
- Follow the same error handling patterns

**3. Use Existing Code as Templates**

```rust
// Template for new stateful actions (most common pattern)
YourComponentAction::NewAction { data } => {
    let Ok(state) = state_context.get_substate_mut() else {
        // TODO: log or propagate
        return;
    };
    state.some_field = data.clone();

    // Dispatch follow-up actions
    let dispatcher = state_context.into_dispatcher();
    dispatcher.push(YourComponentAction::NextAction { ... });
}

// Template for new effectful actions
YourEffectfulAction::NewRequest { params } => {
    store.service.call_external_method(params);
}
```

## Adding New State Machines

### Directory Structure

Follow the standard layout for new components:

```
your_component/
├── your_component_state.rs           # State definition
├── your_component_actions.rs         # Stateful action types
├── your_component_reducer.rs         # State transitions + dispatching
├── your_component_effectful/          # Effectful actions directory
│   ├── your_component_effectful_actions.rs  # Effectful action types
│   ├── your_component_effectful_effects.rs  # Effects implementations
│   └── your_component_service.rs     # Service interface
└── summary.md                        # Component purpose and notes
```

**Alternative flat structure (also used):**

```
your_component/
├── your_component_state.rs           # State definition
├── your_component_actions.rs         # Stateful action types
├── your_component_reducer.rs         # State transitions + dispatching
├── your_component_effects.rs         # Effectful actions and effects
├── your_component_service.rs         # Service interface
└── summary.md                        # Component purpose and notes
```

**Architecture Migration Status:** The codebase is in transition from "old
style" (separate reducer/effects) to "new style" (unified reducers). The
transition frontier (`node/src/transition_frontier/`) still uses the old
pattern, while most other components use the new pattern described in this
guide. For detailed migration instructions, see
[ARCHITECTURE.md](../../ARCHITECTURE.md).

### Action Patterns

**1. Define State Structure**

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct YourComponentState {
    pub status: YourComponentStatus,
    pub data: BTreeMap<Id, YourData>,
    pub pending_requests: VecDeque<PendingRequest>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum YourComponentStatus {
    Idle,
    Processing { request_id: Id },
    Error { error: String },
}
```

**2. Categorize Actions**

```rust
// Stateful actions - handled by reducers
#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum YourComponentAction {
    #[action_event(level = info)]
    Init { config: Config },

    #[action_event(level = debug)]
    ProcessData { data: Data },

    #[action_event(level = warn, fields(debug(error)))]
    Error { error: String },
}

// Effectful actions - handled by effects
#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum YourComponentEffectfulAction {
    #[action_event(level = debug)]
    ExternalRequest { params: Params },

    #[action_event(level = trace)]
    ServiceCall { input: Input },
}
```

**3. Write Enabling Conditions**

```rust
impl EnablingCondition<crate::State> for YourComponentAction {
    fn is_enabled(&self, state: &crate::State, time: Timestamp) -> bool {
        match self {
            YourComponentAction::Init { .. } => {
                // Only allow initialization when not already initialized
                matches!(state.your_component.status, YourComponentStatus::Idle)
            }
            YourComponentAction::ProcessData { data } => {
                // Only process when ready and data is valid
                matches!(state.your_component.status, YourComponentStatus::Idle)
                    && data.is_valid()
            }
            YourComponentAction::Error { .. } => {
                // Errors always allowed for defensive programming
                true
            }
        }
    }
}
```

### Integration Points

**1. Add to Main State**

```rust
// In node/src/state.rs
pub struct State {
    // ... existing fields
    pub your_component: YourComponentState,
}
```

**2. Add to Main Action Enum**

```rust
// In node/src/action.rs
pub enum Action {
    // ... existing variants
    YourComponent(YourComponentAction),
    YourComponentEffectful(YourComponentEffectfulAction),
}
```

**Note on Action Type Generation:** The `node/src/action_kind.rs` file is
autogenerated by the build script. Your actions will be automatically picked up
when you follow the naming convention (`*_actions.rs` or `action.rs`).

**3. Add to Main Reducer**

```rust
// In node/src/reducer.rs
Action::YourComponent(action) => {
    YourComponentState::reducer(state.substate(), action.with_meta(&meta))
}
```

## Component Communication

### Callback Pattern

The codebase uses callbacks for decoupled component communication. The
`redux::callback!` macro enables components to specify how async operations
should respond without tight coupling.

**When to use callbacks:**

- Async operations that need custom response handling
- Cross-component communication without dependencies
- Operations where different callers need different completion behavior

For detailed callback patterns and examples, see
[Architecture Walkthrough](architecture-walkthrough.md#callbacks-pattern).

### Global State Access Pattern

When reducers need to access multiple parts of the global state or coordinate
between different subsystems, use `into_dispatcher_and_state()` instead of
`into_dispatcher()`.

**When to use `into_dispatcher_and_state()`:**

- Generating request IDs from other subsystems
- Reading configuration from other state machines
- Coordinating between multiple components
- Accessing peer information or network state

**Common use cases:**

```rust
// Getting request IDs from other subsystems
let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
let req_id = global_state.snark.user_command_verify.next_req_id();
dispatcher.push(SnarkUserCommandVerifyAction::Init { req_id, ... });

// Accessing peer information for P2P operations
let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
for peer_id in global_state.p2p.ready_peers() {
    dispatcher.push(TransactionPoolAction::P2pSend { peer_id });
}

// Coordinating between multiple state machines
let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
let best_tip = global_state.transition_frontier.best_tip()?;
let cur_slot = global_state.cur_global_slot()?;
// Use both pieces of information for decision making
```

**Pattern structure:**

```rust
let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
// Read from global state
let info = global_state.some_subsystem.get_info();
// Dispatch actions using that information
dispatcher.push(SomeAction::WithInfo { info });
```

**Important notes:**

- Only use when you need to read from global state
- Prefer `into_dispatcher()` for simple action dispatching
- The global state reference is read-only
- Common in P2P, block producer, and transaction pool reducers

## Basic Debugging

For comprehensive debugging tools and troubleshooting, see
[State Machine Debugging Guide](state-machine-debugging-guide.md).

**Quick debugging tips:**

```bash
# Basic logging control
OPENMINA_TRACING_LEVEL=debug cargo run --release -p cli node
```

```rust
// Add ActionEvent to your actions for automatic logging
#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum YourAction {
    #[action_event(level = debug)]
    ProcessData { data: Data },
}
```

## Quick Reference Checklists

### ✅ Adding a New Action to Existing Component

**Before you start:**

- [ ] Find similar actions in the same component
- [ ] Check the component's `summary.md` for known issues
- [ ] Understand the existing state flow

**Implementation steps:**

- [ ] Add action variant to `*_actions.rs`
- [ ] Add `#[action_event(level = debug)]` for logging
- [ ] Implement enabling condition in `EnablingCondition` trait
- [ ] Add handler in reducer with proper state access pattern
  - Use `into_dispatcher()` for simple action dispatching
  - Use `into_dispatcher_and_state()` when needing global state access
- [ ] Test enabling condition logic matches reducer expectations
- [ ] Add documentation comment explaining the action's purpose

### ✅ Adding a New Service Call

**Before you start:**

- [ ] Check if similar service calls exist
- [ ] Identify if this should be effectful action or direct service call
- [ ] Understand the async result handling pattern

**Implementation steps:**

- [ ] Add effectful action variant to `*_effectful_actions.rs`
- [ ] Add thin effect handler that only calls service
- [ ] Ensure service sends result via events
- [ ] Add event handling in event source (if needed)
- [ ] Test the complete async flow

### ✅ Adding a New State Machine Component

**Planning:**

- [ ] Identify the component's single responsibility
- [ ] Check if this fits better as part of existing component
- [ ] Plan the state structure (use enums for state flow)
- [ ] Identify what services this component will need

**File structure:**

- [ ] Create `component_state.rs` with state definition
- [ ] Create `component_actions.rs` with action types
- [ ] Create `component_reducer.rs` with unified reducer
- [ ] Create `component_effectful/` directory structure
- [ ] Create `summary.md` documenting purpose and any issues

**Integration:**

- [ ] Add to main `State` struct in `node/src/state.rs`
- [ ] Add to main `Action` enum in `node/src/action.rs`
- [ ] Add to main reducer in `node/src/reducer.rs`
- [ ] Add substate access with `impl_substate_access!` macro
- [ ] Add service integration if needed

### ✅ Debugging Common Issues

**Action not being processed:**

- [ ] Check if action appears in logs with `OPENMINA_TRACING_LEVEL=debug`
- [ ] Verify enabling condition allows the action
- [ ] Check if action is added to main Action enum and reducer
- [ ] Verify component is initialized before action dispatch

**Service call not returning results:**

- [ ] Check service implementation sends events
- [ ] Verify event source processes the event type
- [ ] Check if event gets converted to correct action
- [ ] Look for service-specific logs or errors

**State machine appears stuck:**

- [ ] Check logs for panic messages or `bug_condition!` triggers
- [ ] Look for blocking operations (sync service calls)
- [ ] Verify events are being processed by event source
- [ ] Check for infinite loops in reducer logic

## Best Practices

1. **Use structured logging** with appropriate `ActionEvent` levels
2. **Write enabling conditions** that match reducer logic exactly
3. **Keep effects thin** - only service calls, no business logic
4. **Use `bug_condition!`** for invariant checking in development
5. **Test with scenarios** that cover your component's edge cases
6. **Document technical debt** in `summary.md` files
7. **Follow existing patterns** in the same component
8. **Use recording/replay** for reproducible debugging
9. **Error handling** - The codebase commonly uses `unwrap()` and `expect()` for
   substate access, as enabling conditions should prevent invalid states
