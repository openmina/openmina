## Action

Object representing some sort of action.

Actions can be nested based on their context.

```Rust
pub enum Action {
    CheckTimeouts(CheckTimeoutsAction),
    P2p(P2pAction),
    ...
}

pub struct CheckTimeoutsAction {}

pub enum P2pAction {
    Connect(P2pConnectAction),
    ...
}

pub struct P2pConnectAction {
    ...
}
```
link to the definition of the node("root") action:
[openmina_node::Action](node/src/action.rs).

## Enabling Condition

Each [Action](#action) must implement the trait
```Rust
pub trait EnablingCondition<State> {
    /// Enabling condition for the Action.
    ///
    /// Checks if the given action is enabled for a given state.
    fn is_enabled(&self, state: &State) -> bool {
        ...
    }
}
```

`is_enabled(state)` must return `false`, if action doesn't make sense given
the current state.

For example message action from peer that isn't connected or we don't know
about in the state, must not be enabled.

Or timeout action. If according to state, duration for timeout hasn't
passed, then we shouldn't enable that timeout action.

Thanks to enabling condition, if properly written, impossible/unexpected
state transition can be made impossible.

#### Avoiding code duplication

We can also utilize this property to avoid code duplication, by simply
trying to dispatch an action instead of checking something in the effects
before dispatch, knowing that enabling condition will filter out actions
that shouldn't happen.

So for example for checking [timeouts](#timeouts), in the `CheckTimeoutsAction`
effects, we can simply dispatch timeout action. If timeout duration
hasn't passed yet, then action will be dropped, if it has passed, then
action will be dispatched as expected.

#### Did action get dispatched

If it's important to know in the effects by dispatcher, if action was
enabled and got dispatched, it can be checked by checking the return value
of the `store.dispatch(..)` call.
It will return `true` if action did get dispatched, otherwise `false`.

Sometimes we want to dispatch one action or the other. In such case we
can write:

```Rust
if !store.dispatch(A) {
    store.dispatch(B);
}
```

## Reducer

Responsible for state management. Only function that's able to change
the [state](node/src/state.rs) is the reducer.

Takes current `State` and an `Action` and computes new `State`.

Pseudocode: `reducer(state, action) -> state`

We don't really need to take state immutably, in JavaScript/frontend
it makes sense, but in our case, it doesn't.

So reducer now looks like this:
```Rust
type Reducer<State, Action> = fn(&mut State, &Action);
```

Main reducer function that gets called on every action:
[openmina_node::reducer](node/src/reducer.rs).

## Effects(side-effects)

Effects are a way to:
1. interact with the [Service](#service).
1. manage [control-flow](#control-flow).

`Effects` run after every action and triggers side-effects (calls to the
service or dispatches some other action).

It has access to global `State`, as well as services.
It can't mutate `Action` or the `State` directly. It can however dispatch
another action, which can in turn mutate the state.

```Rust
type Effects<State, Service, Action> = fn(&mut Store<State, Service, Action>, &Action);
```

Main effects function that gets called on every action:
[openmina_node::effects](node/src/effects.rs).

## Service

Services are mostly just IO or computationally heavy tasks that we want
to run in another thread.

- `Service` should have a minimal state! As a rule of thumb,
  anything that can be serialized, should go inside our global `State`.
- Logic in service should be minimal. No decision-making should be done
  there. It should mostly only act as a common interface for interacting
  with the outside platform (OS, Browser, etc...).

## State

Our global state. [openmina_node::State](node/src/state.rs)

## Store

Provided by the framework. Responsible for executing reducers and effects.

Has a main method `dispatch`, which calls the `reducer` with the given action
and calls `effects` after it.

## State Machine Inputs

State machine's execution is fully predictable/deterministic. If it
receives same inputs in the same order, it's behaviour will be the same.

State machine has 3 kinds of inputs:

1. [Event](#event)
1. Time, which is an extension of every action that gets dispatched.

   see: [ActionMeta](deps/redux-rs/src/action/action_meta.rs)
1. Synchronously returned values by services.

   Idially this one should be avoided as much as possible, because it
   introduces non-determinism in the `effects` function.

Thanks to this property, state machine's inputs can be [recorded](node/src/recorder/recorder.rs),
to then be [replayed](cli/src/commands/replay/replay_state_with_input_actions.rs),
which is very useful for debugging.

## Event

An [Event](node/src/event_source/event.rs), represents all types of data/input
comming from outside the state machine (mostly from services).

It is wrapped by [EventSourceNewEventAction](node/src/event_source/event_source_actions.rs),
and dispatched when new event arrives.

Only events can be dispatched outside the state machine. For case when
there isn't any events, waiting for events will timeout and
[EventSourceWaitTimeoutAction](node/src/event_source/event_source_actions.rs)
will be dispatched. This way we won't get stuck forever waiting for events.

## Control Flow

[event_source](node/src/event_source/event_source_actions.rs) actions
are the only "root" actions that get dispatched. The rest get dispatched
directly or indirectly by event's effects. We could have one large
reducer/effects function for each of the event and it would work as it
does now, but it would be very hard to write and debug.

By having "non-root"(or effect) actions, we make it easier to represent
complex logic and all the state transitions.

There are 2 types of effects:
1. **Local Effects**

   Effects which make sense in local scope/subslice of the state machine.
   Ideally/Mostly such functions are written as `effects` function on the action:
   ```Rust
    impl P2pConnectionOutgoingSuccessAction {
        pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
        {
            let peer_id = self.peer_id;
            store.dispatch(P2pPeerReadyAction { peer_id });
        }
    }
   ```
1. **Global Effects**

   Effects that don't fit in the local scope.

   For example, we could receive rpc request on p2p layer to send our
   current best tip. Since [p2p](p2p/) is in a separate crate, it's not even
   possible to answer that rpc there, as in that crate, we only have
   partial view (p2p part) of the state. But we do have that access
   in [openmina-node](node/) crate, so we write effects to respond to that
   rpc [in there](https://github.com/openmina/openmina/blob/f6bde2138157dcdacd4baa0cd07c22506dc2a7c0/node/src/p2p/p2p_effects.rs#L517).

Examples of the flow:
- [Sync staged ledger](node/src/transition_frontier/sync/ledger/staged)

## Timeouts

To ensure that the node is low in complexity, easy to test, to reason
about and debug, all timeouts must be triggered inside state machine.

If timeouts happen in the service, then it's beyond our full control
during testing, which will limit testing possibilities and will make
tests flaky.

We have a special action: [CheckTimeoutsAction](node/src/action.rs).
It triggers bunch of actions which will check timeouts and if timeout is
detected, timeout action will be dispatched which will trigger other effects.
