# Reducers

This document explains how reducers are defined and used in OpenMina's state machines.

## What are Reducers?

Reducers are pure functions that update the state based on actions. They are the core of the state machine, responsible for transitioning the state from one value to another in response to actions.

## Reducer Structure

In OpenMina, reducers are typically implemented as methods on the state type:

```rust
impl TransitionFrontierGenesisState {
    pub fn reducer<State, Action>(
        mut state_context: Substate<Action, State, Self>,
        action: ActionWithMeta<TransitionFrontierGenesisAction>,
    ) where
        State: SubstateAccess<Self>,
        Action: From<TransitionFrontierGenesisAction>
            + From<TransitionFrontierGenesisEffectfulAction>
            + From<redux::AnyAction>
            + EnablingCondition<State>,
    {
        let Ok(state) = state_context.get_substate_mut() else {
            // TODO: log or propagate
            return;
        };
        let (action, meta) = action.split();

        match action {
            TransitionFrontierGenesisAction::Produce => {
                // Handle Produce action
                // ...
            },
            TransitionFrontierGenesisAction::LedgerLoadSuccess { data } => {
                // Handle LedgerLoadSuccess action
                // ...
            },
            // ... other action handlers
        }
    }
}
```

## Substate Context

In the newer architecture, reducers accept a `Substate` context as their first argument, which provides:

- A mutable reference to the substate that the reducer will mutate
- An immutable reference to the global state
- A mutable reference to a `Dispatcher`

```rust
fn reducer(substate: &mut Substate<MyState>, action: &MyAction) {
    // Update state based on action
    match action {
        MyAction::SomeAction { param } => {
            substate.some_field = param.clone();
            
            // Dispatch a new action if needed
            let dispatcher = substate.dispatcher();
            dispatcher.dispatch(AnotherAction {});
        },
        // ...
    }
}
```

## Pure Functions

Reducers should be pure functions, meaning they:

1. Do not have side effects
2. Do not modify any state outside of the substate provided
3. Do not depend on any external state other than the provided state and action
4. Always produce the same output for the same input

This makes reducers predictable, testable, and easier to reason about.

## Action Handling

Reducers typically use pattern matching to handle different action types:

```rust
match action {
    TransitionFrontierGenesisAction::Produce => {
        match state {
            TransitionFrontierGenesisState::Idle => {
                *state = TransitionFrontierGenesisState::LedgerLoadPending {
                    time: meta.time(),
                };

                let dispatcher = state_context.dispatcher();
                dispatcher.dispatch(TransitionFrontierGenesisEffectfulAction::LedgerLoadInit {
                    config: Arc::new(GenesisConfig::default()),
                });
            }
            // ...
        }
    },
    // ... other action handlers
}
```

This pattern allows for clear and concise handling of different action types and state combinations.

## State Transitions

Reducers are responsible for transitioning the state from one value to another. This is typically done by assigning a new value to the state:

```rust
*state = TransitionFrontierGenesisState::LedgerLoadPending {
    time: meta.time(),
};
```

For more complex state transitions, reducers may need to perform additional operations before updating the state.

## Action Dispatching

Reducers can dispatch new actions using the dispatcher provided by the substate context:

```rust
let dispatcher = state_context.dispatcher();
dispatcher.dispatch(TransitionFrontierGenesisEffectfulAction::LedgerLoadInit {
    config: Arc::new(GenesisConfig::default()),
});
```

This allows reducers to trigger side effects or other state transitions.

## Reducer Composition

Reducers can be composed to handle different parts of the state:

```rust
impl TransitionFrontierState {
    pub fn reducer(
        mut state_context: crate::Substate<Self>,
        action: TransitionFrontierActionWithMetaRef<'_>,
    ) {
        let Ok(state) = state_context.get_substate_mut() else {
            // TODO: log or propagate
            return;
        };
        let (action, meta) = action.split();

        match action {
            TransitionFrontierAction::Genesis(a) => {
                super::genesis::TransitionFrontierGenesisState::reducer(
                    openmina_core::Substate::from_compatible_substate(state_context),
                    meta.with_action(a),
                )
            },
            TransitionFrontierAction::Candidate(a) => {
                super::candidate::TransitionFrontierCandidateState::reducer(
                    openmina_core::Substate::from_compatible_substate(state_context),
                    meta.with_action(a),
                )
            },
            // ... other action handlers
        }
    }
}
```

This allows for a modular and maintainable codebase, where each reducer is responsible for a specific part of the state.

## Error Handling

Reducers should handle errors gracefully, typically by transitioning to an error state:

```rust
match action {
    TransitionFrontierGenesisAction::LedgerLoadFailed { error } => {
        // Transition to idle state (error handling)
        *state = TransitionFrontierGenesisState::Idle;
        
        // Log the error
        log::error!("Failed to load ledger: {}", error);
    },
    // ... other action handlers
}
```

## Best Practices

When implementing reducers in OpenMina, follow these best practices:

1. **Keep Reducers Pure**: Reducers should be pure functions that only update the state based on the action.
2. **Use Pattern Matching**: Use pattern matching to handle different action types and state combinations.
3. **Handle All Cases**: Ensure that all possible action types and state combinations are handled.
4. **Keep Reducers Simple**: Keep reducers focused on state transitions, delegating side effects to effects.
5. **Use Composition**: Compose reducers to handle different parts of the state.
6. **Handle Errors Gracefully**: Handle errors by transitioning to an error state or logging the error.
7. **Test Reducers**: Write tests for reducers to ensure they behave as expected.

## Example: Block Verification Reducer

Here's a more detailed example of a reducer for block verification:

```rust
impl BlockVerifyState {
    pub fn reducer<State, Action>(
        mut state_context: Substate<Action, State, Self>,
        action: ActionWithMeta<BlockVerifyAction>,
    ) where
        State: SubstateAccess<Self>,
        Action: From<BlockVerifyAction>
            + From<BlockVerifyEffectfulAction>
            + From<redux::AnyAction>
            + EnablingCondition<State>,
    {
        let Ok(state) = state_context.get_substate_mut() else {
            // TODO: log or propagate
            return;
        };
        let (action, meta) = action.split();

        match action {
            BlockVerifyAction::Verify { block_hash, input } => {
                // Add the block to the verifying map
                state.verifying.insert(
                    block_hash.clone(),
                    BlockVerifyingState {
                        time: meta.time(),
                        input: input.clone(),
                    },
                );

                // Dispatch effectful action to perform verification
                let dispatcher = state_context.dispatcher();
                dispatcher.dispatch(BlockVerifyEffectfulAction::VerifyInit {
                    block_hash: block_hash.clone(),
                    input: input.clone(),
                });
            },
            BlockVerifyAction::VerifySuccess { block_hash } => {
                // Remove the block from the verifying map
                state.verifying.remove(&block_hash);

                // Add the block to the verified map
                state.verified.insert(
                    block_hash.clone(),
                    BlockVerifiedState {
                        time: meta.time(),
                    },
                );
            },
            BlockVerifyAction::VerifyFailed { block_hash, error } => {
                // Remove the block from the verifying map
                state.verifying.remove(&block_hash);

                // Add the block to the failed map
                state.failed.insert(
                    block_hash.clone(),
                    BlockVerifyFailedState {
                        time: meta.time(),
                        error: error.clone(),
                    },
                );

                // Log the error
                log::error!("Failed to verify block {}: {}", block_hash, error);
            },
        }
    }
}
```

This example shows how a reducer handles different action types, updates the state, and dispatches effectful actions when needed.
