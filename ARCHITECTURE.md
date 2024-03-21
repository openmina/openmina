# Core Concepts

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
    fn is_enabled(&self, state: &State, time: Timestamp) -> bool {
        ...
    }
}
```

`is_enabled(state, time)` must return `false`, if action doesn't make sense given
the current state and, optionally, time.

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

# Mental Model Tips

When working with this architecture, conventional means of reading and
writing the code goes out the window. A lot of code that we write,
won't be a state machine, but the code that is, completely differs from
what developers are used to. In order to be productive with it, mental
model needs to be adjusted.

For example, when developer needs to understand the logic written in a
particular state machine, it might be tempting to use conventional methods,
like finding a starting point in the code and trying to follow the code.
One who'll attempt that, will quickly realise that the process is
extremely taxing and almost impossible. The amount of things you need
to keep in memory, number of jumps you need to perform, makes it very
difficult. This hinders one's productivity and in the end, most likely,
whatever change developer makes in the code will be buggy, due to
partial understanding obtained through tedious process.

Instead we need to shift our mental model and change the overall way we
process the code written with this architecture. We need to leverage
the way state machines are written, in order to minimize the cognitive load.
Instead of trying to follow the code, we should start with the state
of the state machine, look at it carefully and try to deduce the flow
using it. Our deductions just based on state will have flaws and holes,
but we can fix that by filtering through actions and finding ones based
on name, which most likely relates to those holes. By checking those
and their enabling conditions, we will have a more clearer idea about
the state machine. Sometimes it still might not be enough and we will
need to check (only relevant) pieces of the reducer and effects.

Main reason why conventional method doesn't work, is because nothing is
abstracted/hidden away. When you follow the code, you follow actual
execution that the cpu will perform. We have a single threaded state machine,
responsible for business logic + managing concurrent/parallel processes,
so following it's flow is like attempting to jump into async executor
code at every `.await` point.

## Importance of State

The `State` of the state machine, sits at the core of everything. It is
the first thing we carefully design, actions with enabling conditions,
effects and reducers come later.

`State` is supposed to be a declarative way to describe a flow, then
enabling conditions, reducers and effects enable that flow. Even if we
remove all the rest of the code, simply having a `State` definition should
be enough to get a general idea about the purpose and the flow of the
state machine. Each sub-state/sub-statemachine must follow this rule.

E.g. if we look at the state of the [snark_pool_candidate](node/src/snark_pool/candidate/snark_pool_candidate_state.rs)
state machine, which is responsible for processing received snark work.

(added comments represent thought process while reading the state).
```Rust
pub enum SnarkPoolCandidateState {
    // some info received regarding the candidate?
    InfoReceived {
        time: Timestamp,
        info: SnarkInfo,
    },
    // ok so we actually need to fetch the work, where do we need to fetch
    // the mentioned `Work` from?
    WorkFetchPending {
        time: Timestamp,
        info: SnarkInfo,
        // p2p rpc id, so we are fetching work using p2p. So above info
        // was probably just a metadata of the snark.
        rpc_id: P2pRpcId,
    },
    // we received the work from p2p.
    WorkReceived {
        time: Timestamp,
        work: Snark,
    },
    // after receiving snark work, we initiate verification and it's pending,
    // meaning verification happens concurrently or in parallel.
    WorkVerifyPending {
        time: Timestamp,
        work: Snark,
        verify_id: SnarkWorkVerifyId,
    },
    // verification failed.
    WorkVerifyError {
        time: Timestamp,
        work: Snark,
    },
    // verification succeeded. This is the last state of this state machine,
    // so continuation is handled elsewhere. In case of snark work, verified
    // snark would be moved from here to snark pool.
    WorkVerifySuccess {
        time: Timestamp,
        work: Snark,
    },
}

...
```

Above is just a demo to show how much we can deduce just by looking at
the state. Of course it isn't enough to have a full understanding of the
individual state machine, and it might leave some holes, but they can
easily be filled by looking at the actions most related to those holes.

Examples of the holes left by above is:
1. State machine starts with snark work info already received. How do we
   know we even need that snark? Where is the filtering happening?

   To find out we can look for the action name that would be "notifying"
   this state machine that info was received. If we check, it's
   `SnarkPoolCandidateInfoReceivedAction`. If we check it's enabling
   condition, we will see that unneeded snarks will get filtered out there.
2. Do snarks get verified one by one? Or is there batching involved?

   To find that out, we can look for actions regarding work verification
   in this state machine. We can see `SnarkPoolCandidateWorkVerifyPendingAction`,
   it's definition being:
    ```Rust
    pub struct SnarkPoolCandidateWorkVerifyPendingAction {
        pub peer_id: PeerId,
        pub job_ids: Vec<SnarkJobId>,
        pub verify_id: SnarkWorkVerifyId,
    }
    ```

    We can clearly see from above that we have an array of `job_ids`,
    while only having a single `verify_id`. Meaning we do have batching.

    Also we can see `peer_id` there, which might not be obvious why it's
    there. To understand that and how snarks are batched together, we
    need to check out a place where
    `SnarkPoolCandidateWorkVerifyPendingAction` gets dispatched(in the effects).

## Designing new state machine

#### Where it belongs

When creating a new state machine, first we need to figure out where
it belongs. Whether we should add new statemachine in the root dir of
the node, or if it needs to be a sub-statemachine.

Once we decide, we need to make a new module there.

#### Designing state

Once we have that down, we need to think about the flow and logic we
wish to introduce and use that in order to carefully craft the
definition of the state. This is where the most amount of thought needs
to go to. If we start with the state and make it represent the flow,
we will make our new state machine:

1. Easy to debug, since state represents the flow and if we want to debug
   it, we can easily follow the flow by observing state transitions.
2. Easy to read/process, since a lot of information will be conveyed
   just with state definition.
3. Minimized or non-existent impossible/duplicate states, since state
   represents the actual flow, we can use it to restrict the flow with
   enabling conditions as much as possible.

When designing a state, above expectations must be taken into account.
If `State` doesn't represent the flow and hides it, it will take other
developers much longer to process the code and impossible states could
become an issue.

#### Designing actions and enabling conditions

Actions should be a reflection of the `State`. After designing the state,
what actions we need to create should be clear, since we will simply be
adding actions which will cause those state transitions we described above.

Most of the action names should match state transition names. E.g.
If we have state transition: `SnarkPoolCandidateState::WorkVerifyPending { .. }`,
action which causes state to transition to that specific state, should be
named: `SnarkPoolCandidateWorkVerifyPendingAction`. That way it's easy
for developer reading it later on, to filter through actions more easily.
Actions not following this pattern should be as rare as possible as they
will need special attention while going through the code, in order to not
miss anything.

Action's enabling condition should be as limiting as possible, in order
to avoid impossible state transitions, which could break the node.

#### Designing reducers

Most of the time, reducers goal will simply be to facilitate state
transitions and nothing more. Pretty much grabbing data from one enum
variant and moving it to another, transforming any values that will need it.

In order to do those transitions however, we have to destructure current
enum variant and extract fields from there, so we need to make sure that
enabling condition guarantees that the action won't be triggered, unless
our current state variant is indeed what we expect in the reducer.

#### Designing effects

Simpler the effects are, better it is. Mostly they should just declare
what actions may be dispatched after the current action. If checks need
to be done before dispatching some action, those checks belong in the
enabling condition, not the effects. Simpler and smaller the effects are,
easier it is to traverse them.
