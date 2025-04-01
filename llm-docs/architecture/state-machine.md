# State Machine Architecture

OpenMina uses a state machine architecture that provides a predictable and debuggable flow of data and operations. This document explains the core concepts of this architecture.

## Architecture Diagram

```mermaid
flowchart LR
    Action["Action"] --> Reducer["Reducer"]
    Reducer --> Effects["Effects"]
    Reducer --> State["State"]
    Effects --> Action

    %% Legend
    L1["Input"] --- L2["Process"] --- L3["Output"] --- L4["State"]

    classDef actionClass stroke:#ff9f1c,stroke-width:2px,fill:none;
    classDef reducerClass stroke:#2ec4b6,stroke-width:2px,fill:none;
    classDef stateClass stroke:#e71d36,stroke-width:2px,fill:none;
    classDef effectsClass stroke:#011627,stroke-width:2px,fill:none;
    classDef legend stroke-dasharray: 5 5, fill:none;

    class Action actionClass;
    class Reducer reducerClass;
    class State stateClass;
    class Effects effectsClass;
    class L1,L2,L3,L4 legend;
```

**Diagram Legend:**

-   **Action**: Operation to be performed (input to the state machine)
-   **Reducer**: Pure function that updates state based on action
-   **Effects**: Handles side effects and may dispatch new actions
-   **State**: Current state of the system

## Core Concepts

### Action

An action is an object representing some sort of operation to be performed. Actions can be nested based on their context.

```rust
pub enum Action {
    CheckTimeouts(CheckTimeoutsAction),
    P2p(P2pAction),
    // ...
}

pub struct CheckTimeoutsAction {}

pub enum P2pAction {
    Connect {
        // ...
    },
    // ...
}
```

Actions are defined in various files throughout the codebase, with the root action defined in [node/src/action.rs](../node/src/action.rs).

### Enabling Condition

Each action must implement the `EnablingCondition` trait, which determines whether the action can be processed based on the current state.

```rust
pub trait EnablingCondition<State> {
    /// Enabling condition for the Action.
    ///
    /// Checks if the given action is enabled for a given state.
    fn is_enabled(&self, state: &State, time: Timestamp) -> bool {
        // ...
    }
}
```

Enabling conditions help restrict the flow of actions and prevent impossible or duplicate states.

### Reducer

Reducers are functions that update the state based on an action. They are pure functions that take the current state and an action and return a new state.

In the newer architecture, reducers accept a `Substate` context as their first argument, which provides:

-   A mutable reference to the substate that the reducer will mutate
-   An immutable reference to the global state
-   A mutable reference to a `Dispatcher`

```rust
fn reducer(substate: &mut Substate<MyState>, action: &MyAction) {
    // Update state based on action
}
```

### State

The state is the core of the state machine. It represents the current state of the system and is carefully designed to represent the flow of the application.

Benefits of this state-centric approach:

1. Easy to debug, since state represents the flow
2. Easy to read/process, since state definition conveys a lot of information
3. Minimized or non-existent impossible/duplicate states

### Effects

Effects handle side effects that may occur as a result of an action. They can dispatch new actions but should be kept as simple as possible.

In the newer architecture, effects are often combined with reducers, with a clear separation between the state update phase and the action dispatching phase.

```rust
fn reducer_with_effects(substate: &mut Substate<MyState>, action: &MyAction) {
    // First update state
    substate.my_field = new_value;

    // Then dispatch new actions
    let dispatcher = substate.dispatcher();
    dispatcher.dispatch(NewAction {});
}
```

## Action Types

Actions are divided into two categories:

### Stateful Actions

These actions update the state and have a `reducer` function. They closely resemble the traditional state machine code, and most of the state machine logic should reside here.

### Effectful Actions

These actions involve calling external services and have an `effects` function. They should serve as thin layers for handling service interactions.

```rust
pub enum TransitionFrontierGenesisEffectfulAction {
    LedgerLoadInit {
        config: Arc<GenesisConfig>,
    },
    ProveInit {
        block_hash: StateHash,
        input: Box<ProverExtendBlockchainInputStableV2>,
    },
    // ...
}
```

## Practical Considerations

When working with this architecture, conventional means of reading and writing code may not apply. The code follows the actual execution that the CPU will perform, with nothing abstracted or hidden away.

For more practical examples of how this architecture works in practice, see [Action-Reducer-Effect Cycle](action-reducer-effect.md).
