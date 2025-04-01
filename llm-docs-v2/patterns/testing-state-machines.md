# Testing State Machines

This document explains how to test state machines in OpenMina, focusing on unit testing, integration testing, and property-based testing.

## Unit Testing State Machines

Unit testing state machines involves testing the behavior of a single state machine in isolation. This typically involves:

1. Creating an initial state
2. Dispatching actions to the state machine
3. Asserting that the state transitions correctly
4. Asserting that the expected actions are dispatched

### Testing Reducers

Reducers are tested by creating an initial state, dispatching an action, and asserting that the state transitions correctly:

```rust
#[test]
fn test_transition_frontier_genesis_produce() {
    // Arrange
    let mut state = TransitionFrontierGenesisState::Idle;
    let action = TransitionFrontierGenesisAction::Produce;
    
    // Act
    let mut dispatcher = MockDispatcher::new();
    let mut state_context = Substate::new(&mut state, &mut dispatcher);
    TransitionFrontierGenesisState::reducer(state_context, ActionWithMeta::new(action));
    
    // Assert
    assert!(matches!(state, TransitionFrontierGenesisState::LedgerLoadPending { .. }));
    assert!(dispatcher.dispatched_action::<TransitionFrontierGenesisEffectfulAction>(
        |a| matches!(a, TransitionFrontierGenesisEffectfulAction::LedgerLoadInit { .. })
    ));
}
```

### Testing Effects

Effects are tested by creating a mock service, dispatching an effectful action, and asserting that the service is called correctly:

```rust
#[test]
fn test_transition_frontier_genesis_ledger_load_init() {
    // Arrange
    let config = Arc::new(GenesisConfig::default());
    let action = TransitionFrontierGenesisEffectfulAction::LedgerLoadInit {
        config: config.clone(),
    };
    
    // Act
    let mut mock_service = MockTransitionFrontierGenesisService::new();
    mock_service.expect_load_genesis()
        .with(eq(config))
        .times(1)
        .return_const(());
    
    let mut store = Store::new(mock_service);
    action.effects(&ActionMeta::default(), &mut store);
    
    // Assert
    // The mock service expectations are checked automatically when it's dropped
}
```

### Testing Enabling Conditions

Enabling conditions are tested by creating a state, creating an action, and asserting that the enabling condition returns the expected result:

```rust
#[test]
fn test_transition_frontier_genesis_produce_enabling_condition() {
    // Arrange
    let mut state = State::default();
    state.transition_frontier.genesis = TransitionFrontierGenesisState::Idle;
    let action = TransitionFrontierGenesisAction::Produce;
    
    // Act
    let result = action.is_enabled(&state, Timestamp::now());
    
    // Assert
    assert!(result);
}
```

## Integration Testing State Machines

Integration testing state machines involves testing the behavior of multiple state machines working together. This typically involves:

1. Creating an initial global state
2. Dispatching actions to the global state machine
3. Asserting that the global state transitions correctly
4. Asserting that the expected actions are dispatched

```rust
#[test]
fn test_transition_frontier_genesis_and_sync() {
    // Arrange
    let mut state = State::default();
    
    // Act
    let mut dispatcher = MockDispatcher::new();
    let mut state_context = Substate::new(&mut state, &mut dispatcher);
    
    // Dispatch genesis produce action
    State::reducer(state_context, ActionWithMeta::new(Action::TransitionFrontier(
        TransitionFrontierAction::Genesis(TransitionFrontierGenesisAction::Produce)
    )));
    
    // Simulate genesis success
    State::reducer(state_context, ActionWithMeta::new(Action::TransitionFrontier(
        TransitionFrontierAction::Genesis(TransitionFrontierGenesisAction::LedgerLoadSuccess {
            data: GenesisConfigLoaded::default(),
        })
    )));
    
    // Dispatch sync start action
    State::reducer(state_context, ActionWithMeta::new(Action::TransitionFrontier(
        TransitionFrontierAction::Sync(TransitionFrontierSyncAction::Start {
            target: StateHash::default(),
        })
    )));
    
    // Assert
    assert!(matches!(state.transition_frontier.genesis, TransitionFrontierGenesisState::LedgerLoadSuccess { .. }));
    assert!(matches!(state.transition_frontier.sync, TransitionFrontierSyncState::Init { .. }));
}
```

## Property-Based Testing

Property-based testing involves generating random inputs and asserting that certain properties hold for all inputs. This is particularly useful for testing state machines, as it can help find edge cases that might be missed with traditional unit tests.

```rust
#[test]
fn test_transition_frontier_genesis_properties() {
    // Define a property: After producing the genesis block, the state should be LedgerLoadPending
    let property = |state: TransitionFrontierGenesisState| {
        let mut state = state;
        let action = TransitionFrontierGenesisAction::Produce;
        
        let mut dispatcher = MockDispatcher::new();
        let mut state_context = Substate::new(&mut state, &mut dispatcher);
        TransitionFrontierGenesisState::reducer(state_context, ActionWithMeta::new(action));
        
        if matches!(state, TransitionFrontierGenesisState::Idle) {
            matches!(state, TransitionFrontierGenesisState::LedgerLoadPending { .. })
        } else {
            true // Other states are not affected by Produce action
        }
    };
    
    // Test the property with randomly generated states
    QuickCheck::new().quickcheck(property as fn(TransitionFrontierGenesisState) -> bool);
}
```

## Testing State Machine Composition

Testing state machine composition involves testing how state machines interact with each other. This typically involves:

1. Creating initial states for multiple state machines
2. Dispatching actions that affect multiple state machines
3. Asserting that the states transition correctly
4. Asserting that the expected actions are dispatched

```rust
#[test]
fn test_transition_frontier_and_block_producer() {
    // Arrange
    let mut state = State::default();
    
    // Act
    let mut dispatcher = MockDispatcher::new();
    let mut state_context = Substate::new(&mut state, &mut dispatcher);
    
    // Dispatch block producer won slot action
    State::reducer(state_context, ActionWithMeta::new(Action::BlockProducer(
        BlockProducerAction::WonSlot {
            won_slot: BlockProducerWonSlot::default(),
        }
    )));
    
    // Simulate block production
    State::reducer(state_context, ActionWithMeta::new(Action::BlockProducer(
        BlockProducerAction::BlockProveSuccess {
            block: ArcBlockWithHash::default(),
        }
    )));
    
    // Assert
    assert!(matches!(state.block_producer.current, BlockProducerCurrentState::BlockProveSuccess { .. }));
    assert!(dispatcher.dispatched_action::<Action>(
        |a| matches!(a, Action::TransitionFrontier(TransitionFrontierAction::Candidate(
            TransitionFrontierCandidateAction::BlockReceived { .. }
        )))
    ));
}
```

## Testing Async Operations

Testing async operations involves testing how state machines handle asynchronous operations. This typically involves:

1. Creating an initial state
2. Dispatching an action that initiates an async operation
3. Asserting that the state transitions to a pending state
4. Simulating the completion of the async operation
5. Asserting that the state transitions to a success or error state

```rust
#[test]
fn test_transition_frontier_genesis_ledger_load() {
    // Arrange
    let mut state = TransitionFrontierGenesisState::Idle;
    
    // Act
    let mut dispatcher = MockDispatcher::new();
    let mut state_context = Substate::new(&mut state, &mut dispatcher);
    
    // Dispatch produce action
    TransitionFrontierGenesisState::reducer(state_context, ActionWithMeta::new(
        TransitionFrontierGenesisAction::Produce
    ));
    
    // Assert that state transitions to pending
    assert!(matches!(state, TransitionFrontierGenesisState::LedgerLoadPending { .. }));
    
    // Simulate ledger load success
    TransitionFrontierGenesisState::reducer(state_context, ActionWithMeta::new(
        TransitionFrontierGenesisAction::LedgerLoadSuccess {
            data: GenesisConfigLoaded::default(),
        }
    ));
    
    // Assert that state transitions to success
    assert!(matches!(state, TransitionFrontierGenesisState::LedgerLoadSuccess { .. }));
}
```

## Testing Error Handling

Testing error handling involves testing how state machines handle errors. This typically involves:

1. Creating an initial state
2. Dispatching an action that can fail
3. Simulating an error
4. Asserting that the state transitions to an error state
5. Asserting that the error is handled correctly

```rust
#[test]
fn test_transition_frontier_genesis_ledger_load_failed() {
    // Arrange
    let mut state = TransitionFrontierGenesisState::LedgerLoadPending {
        time: Timestamp::now(),
    };
    
    // Act
    let mut dispatcher = MockDispatcher::new();
    let mut state_context = Substate::new(&mut state, &mut dispatcher);
    
    // Simulate ledger load failure
    TransitionFrontierGenesisState::reducer(state_context, ActionWithMeta::new(
        TransitionFrontierGenesisAction::LedgerLoadFailed {
            error: "Failed to load ledger".to_string(),
        }
    ));
    
    // Assert that state transitions to idle (error handling)
    assert!(matches!(state, TransitionFrontierGenesisState::Idle));
}
```

## Testing Tools

OpenMina provides several tools for testing state machines:

### MockDispatcher

The `MockDispatcher` is used to test that the correct actions are dispatched:

```rust
let mut dispatcher = MockDispatcher::new();
let mut state_context = Substate::new(&mut state, &mut dispatcher);
TransitionFrontierGenesisState::reducer(state_context, ActionWithMeta::new(action));

assert!(dispatcher.dispatched_action::<TransitionFrontierGenesisEffectfulAction>(
    |a| matches!(a, TransitionFrontierGenesisEffectfulAction::LedgerLoadInit { .. })
));
```

### MockService

Mock services are used to test that the correct service methods are called:

```rust
let mut mock_service = MockTransitionFrontierGenesisService::new();
mock_service.expect_load_genesis()
    .with(eq(config))
    .times(1)
    .return_const(());

let mut store = Store::new(mock_service);
action.effects(&ActionMeta::default(), &mut store);
```

### TestStore

The `TestStore` is used to test the behavior of the entire state machine:

```rust
let mut store = TestStore::new();
store.dispatch(Action::TransitionFrontier(
    TransitionFrontierAction::Genesis(TransitionFrontierGenesisAction::Produce)
));

assert!(matches!(store.state().transition_frontier.genesis, TransitionFrontierGenesisState::LedgerLoadPending { .. }));
```

## Best Practices

When testing state machines in OpenMina, follow these best practices:

1. **Test State Transitions**: Test that the state transitions correctly in response to actions.
2. **Test Action Dispatching**: Test that the correct actions are dispatched in response to other actions.
3. **Test Enabling Conditions**: Test that enabling conditions correctly determine when actions can be processed.
4. **Test Error Handling**: Test that errors are handled correctly.
5. **Test Edge Cases**: Test edge cases, such as empty states, maximum values, and error conditions.
6. **Use Property-Based Testing**: Use property-based testing to find edge cases that might be missed with traditional unit tests.
7. **Test Composition**: Test how state machines interact with each other.
8. **Test Async Operations**: Test how state machines handle asynchronous operations.
9. **Use Mock Services**: Use mock services to test that the correct service methods are called.
10. **Keep Tests Simple**: Keep tests simple and focused on a single aspect of the state machine.
