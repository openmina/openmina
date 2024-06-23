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
    Connect {
        ...
    },
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

For example, message action from peer that isn't connected or we don't know
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

## Substate access, Queued Reducer-Dispatch, and Callbacks

The state machine is being refactored to make the code easier to follow and work with. While the core concepts remain mostly unchanged, the organization is evolving.

**Note**: For reference on the direction of these changes, see [this document](https://github.com/openmina/state_machine_exp/blob/main/node/README.md).

**What is new**:
- `SubstateAccess` trait: Specifies how specific substate slices are obtained from a parent state.
- `Substate` context: Provides fine-grained control over state and dispatcher access, ensuring clear separation of concerns.
- `Dispatcher`: Manages the queuing and execution of actions, allowing reducers to queue additional actions for dispatch after the current state update phase.
- `Callback` handlers: Facilitate flexible control flow by enabling actions to specify follow-up actions at dispatch time, reducing coupling between components and making control flow more local.
- *Stateful* vs *Effectful* actions:
    - Stateful actions update the state and dispatch other actions. These are processed by `reducer` functions.
    - Effectful actions are very thing layers over services that expose them as actions and can dispatch callback actions. These are processed by `effects` functions.

### New-Style Reducers

New-style reducers accept a `Substate` context as their first argument instead of the state they act on.

This substate context provides the reducer function with access to:
- A mutable reference to the substate that the reducer will mutate.
- An immutable reference to the global state.
- A mutable reference to a `Dispatcher`.

The reducer function cannot access both the substate and the dispatcher/global state references simultaneously. This enforces a separation between the state update phase and the further action dispatching phase.

This setup allows us to combine, the reducer function and the effect handler function into one, removing a level of flow indirection while keeping the phases separate.

```rust
impl WatchedAccountsState {
    pub fn reducer(
        mut state_context: crate::Substate<Self>,
        action: WatchedAccountsActionWithMetaRef<'_>,
    ) {
        let Ok(state) = state_context.get_substate_mut() else { return; };
        let (action, meta) = action.split();

        match action {
            WatchedAccountsAction::Add { pub_key } => {
                state.insert(
                    pub_key.clone(),
                    WatchedAccountState {
                        initial_state: WatchedAccountLedgerInitialState::Idle { time: meta.time() },
                        blocks: Default::default(),
                    },
                );

                // === End of state update phase / Start of dispatch phase ===

                let pub_key = pub_key.clone();
                // To access the global state and dispatcher:
                // let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                let dispatcher = state_context.into_dispatcher();
                // This action will be automatically dispatched by the effect handler
                dispatcher.push(WatchedAccountsAction::LedgerInitialStateGetInit { pub_key });
            }
            // Other actions...
        }
    }
}
```

This reducer is called from the root reducer like this:

```rust
pub fn reducer(
    state: &mut State,
    action: &ActionWithMeta,
    // **New argument**
    dispatcher: &mut redux::Dispatcher<Action, State>,
) {
    let meta = action.meta().clone();
    match action.action() {
        // ...
        Action::WatchedAccounts(a) => {
            WatchedAccountsState::reducer(
                // Substate context is created from the global state and dispatcher
                Substate::new(state, dispatcher),
                meta.with_action(a),
            );
        }
        // Other actions...
    }
    // ...
}
```

### Effectful Actions

Actions and their handling code are divided into two categories: *stateful* actions and *effectful* actions.

- **Stateful Actions**: These actions update the state and have a `reducer` function. They closely resemble the traditional state machine code, and most of the state machine logic should reside here.
- **Effectful Actions**: These actions involve calling external services and have an `effects` function. They should serve as thin layers for handling service interactions.

Example effectful action:

```rust
pub enum TransitionFrontierGenesisEffectfulAction {
    LedgerLoadInit {
        config: Arc<GenesisConfig>,
    },
    ProveInit {
        block_hash: StateHash,
        input: Box<ProverExtendBlockchainInputStableV2>,
    },
}
```

And the effect handler:

```rust
impl TransitionFrontierGenesisEffectfulAction {
    pub fn effects<S: redux::Service>(&self, _: &ActionMeta, store: &mut Store<S>)
    where
        S: TransitionFrontierGenesisService,
    {
        match self {
            TransitionFrontierGenesisEffectfulAction::LedgerLoadInit { config } => {
                store.service.load_genesis(config.clone());
            }
            TransitionFrontierGenesisEffectfulAction::ProveInit { block_hash, input } => {
                store.service.prove(block_hash.clone(), input.clone());
            }
        }
    }
}
```

### Callbacks

Callbacks are a new construct that permit the uncoupling of state machine components by enabling the dynamic composition of actions and their sequencing.

With callbacks, a caller (handling actions of type `A`) can dispatch an action of type `B` that will produce a result. The action `B` includes callback values that specify how to return the result to `A`. When the result of processing `B` is ready (either further down the action chain or asynchronously from a service call), the callback is invoked with the result.

This is particularly useful when implementing effectful actions to interact with services, but also for composing multiple components without introducing inter-dependencies (with callbacks we can avoid the *global effects* pattern that was described before in this document).

Callback blocks are declared with the `redux::callback!` macro and are described by a lambda block with a single output, which must produce an `Action` value as a result.

Example:

```rust
impl ConsensusState {
    pub fn reducer(
        mut state_context: crate::Substate<Self>,
        action: ConsensusActionWithMetaRef<'_>,
    ) {
        // ...
        match action {
            ConsensusAction::BlockReceived {
                hash,
                block,
                chain_proof,
            } => {
                // ... state updates ...

                // Dispatch
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                let req_id = global_state.snark.block_verify.next_req_id();
                // Start the verification process
                dispatcher.push(SnarkBlockVerifyAction::Init {
                    req_id,
                    block: (hash.clone(), block.clone()).into(),
                    // Will be called after the block has been successfully verified
                    on_success: redux::callback!(|hash: BlockHash| -> crate::Action {
                        ConsensusAction::BlockSnarkVerifySuccess { hash }
                    }),
                    // Will be called if there is an error
                    on_error: redux::callback!(|(hash: BlockHash, error: SnarkBlockVerifyError)| -> crate::Action {
                        ConsensusAction::BlockSnarkVerifyError { hash, error }
                    }),
                });
                // Set the block verification state as pending
                dispatcher.push(ConsensusAction::BlockSnarkVerifyPending {
                    req_id,
                    hash: hash.clone(),
                });
            }
    // ...
```

### Porting old code

For a given `LocalState` that is a substate of `State`:

#### Implement `SubstateAccess<LocalState>`

Implement in `node/src/state.rs` the `SubstateAccess<LocalState>` trait for `State` if it is not defined already. For trivial cases use the `impl_substate_access!` macro.

#### Update the `reducer` function

Update the `reducer` function so that:

1. It is implemented as a method on `LocalState`.
2. It accepts as it's first argument `mut state_context: crate::Substate<Self>` instead of `&mut self`.
3. It obtains `state` by calling `state_context.get_state_mut()`.
3. All references to `self` are updated to instead reference `state`.

Example:

```diff
impl ConsensusState {
     pub fn reducer(
-        &mut self,
+        mut state_context: crate::Substate<Self>,
         action: ConsensusActionWithMetaRef<'_>,
     ) {
+        let Ok(state) = state_context.get_substate_mut() else {
+            // TODO: log or propagate
+            return;
+        };
         let (action, meta) = action.split();
         match action {
             ConsensusAction::BlockReceived {
                 hash,
                 block,
                 chain_proof,
             } => {
-                self.blocks.insert(
+                state.blocks.insert(
                     hash.clone(),
                     ConsensusBlockState {
                         block: block.clone(),
@@ -22,16 +38,41 @@ impl ConsensusState {
                         chain_proof: chain_proof.clone(),
                     },
                 );

         // ...
```

#### Move dispatches from `effects` to `reducer`

For each action that doesn't call a service in it's effect handler, delete it's body from the effect handler and move it to the end of the body of the reducer's match branch that handles that action:

Example:

```diff
 // consensus_effects.rs

 pub fn consensus_effects<S: crate::Service>(store: &mut Store<S>, action: ConsensusActionWithMeta) {
     let (action, _) = action.split();
 
     match action {
         ConsensusAction::BlockReceived { hash, block, .. } => {
-            let req_id = store.state().snark.block_verify.next_req_id();
-            store.dispatch(SnarkBlockVerifyAction::Init {
-                req_id,
-                block: (hash.clone(), block).into(),
-            });
-            store.dispatch(ConsensusAction::BlockSnarkVerifyPending { req_id, hash });
         }
         // ...

 // consensus_reducer.rs

 impl ConsensusState {
     pub fn reducer(
         mut state_context: crate::Substate<Self>,
         action: ConsensusActionWithMetaRef<'_>,
     ) {
         let Ok(state) = state_context.get_substate_mut() else {
             // TODO: log or propagate
             return;
         };
         let (action, meta) = action.split();
         match action {
             ConsensusAction::BlockReceived {
                 hash,
                 block,
                 chain_proof,
             } => {
                 state.blocks.insert(
                     hash.clone(),
                     ConsensusBlockState {
                         block: block.clone(),
@@ -22,16 +38,41 @@ impl ConsensusState {
                         chain_proof: chain_proof.clone(),
                     },
                 );
+
+                // Dispatch
+                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
+                let req_id = global_state.snark.block_verify.next_req_id();
+                dispatcher.push(SnarkBlockVerifyAction::Init {
+                    req_id,
+                    block: (hash.clone(), block.clone()).into(),
+                });
+                dispatcher.push(ConsensusAction::BlockSnarkVerifyPending {
+                    req_id,
+                    hash: hash.clone(),
+                });
             }
             // ...
```

#### Update the reducer invocation in the parent reducer

Replace the call in the parent reducer so that it creates a new `Substate` instance.

Example:

```diff
 pub fn reducer(
     state: &mut State,
     action: &ActionWithMeta,
     dispatcher: &mut redux::Dispatcher<Action, State>,
 ) {
     let meta = action.meta().clone();
     match action.action() {
         // ...
         Action::Consensus(a) => {
-            state.consensus.reducer(meta.with_action(a));
+            crate::consensus::ConsensusState::reducer(
+                Substate::new(state, dispatcher),
+                meta.with_action(a),
+            );
         }
    // ..
```

#### Define effectful actions for service interactions

Actions that interact with services must be updated so that the interaction is performed by dispatching a new effectful action. No reducer function should be implemented for these new effectful actions.

Example:

See `node/src/transition_frontier/genesis{_effectful}`, `snark/src/block_verify{_effectful}` and `snark/src/work_verify{_effectful}`.

#### Add callbacks

There are 3 main situations in which callbacks are an improvement:

- Passing them to effectful actions that will call a service
- Cross-component calls, to make the flow clearer and avoid inter-dependencies (eg. interactions between the transition frontier, and the p2p layer).
- Abstraction of lower level layers (e.g. higher level p2p abstractions over lower level tcp and mio implementations).

Example: when a block is received, the consensus state machine will dispatch an action to verify the block. This action will trigger an asynchronous snark verification process that will complete (or fail) some time in the future, and we are interested in its result.

The `SnarkBlockVerifyAction::Init` action gets updated with the addition of two callbacks, one that will be called after a successful verification, and another when an error occurs:

```diff
 pub enum SnarkBlockVerifyAction {
     Init {
         req_id: SnarkBlockVerifyId,
         block: VerifiableBlockWithHash,
+        on_success: redux::Callback<BlockHash>,
+        on_error: redux::Callback<(BlockHash, SnarkBlockVerifyError)>,
     },
     // ...
 }
```

The consensus reducer, after receiving a block, initializes the asynchronous block snark verification process specifying the callbacks, and sets the state to "pending". The dispatching of `SnarkBlockVerifyAction::Init` gets updated with the required callbacks:

```diff
     match action {
         ConsensusAction::BlockReceived { hash, block, .. } => {
            // ... state updates omitted
            dispatcher.push(SnarkBlockVerifyAction::Init {
                req_id,
                block: (hash.clone(), block.clone()).into(),
+               on_success: redux::callback!(|hash: BlockHash| -> crate::Action {
+                   ConsensusAction::BlockSnarkVerifySuccess { hash }
+               }),
+               on_error: redux::callback!(|(hash: BlockHash, error: SnarkBlockVerifyError)| -> crate::Action {
+                   ConsensusAction::BlockSnarkVerifyError { hash, error }
+               }),
            });
            dispatcher.push(ConsensusAction::BlockSnarkVerifyPending {
                req_id,
                hash: hash.clone(),
            });
         }
         // ...
```

Then when handling `SnarkBlockVerifyAction::Init` a job is added to the state, with the callbacks stored there. Then the effectful action that will interact with the service is dispatched (**NOTE:** not shown here, see `snark/src/block_verify_effectful/`).


```rust
// when matching `SnarkBlockVerifyAction::Init`
state.jobs.add(SnarkBlockVerifyStatus::Init {
    time: meta.time(),
    block: block.clone(),
    on_success: on_success.clone(),
    on_errir: on_errir.clone(),
});

// Dispatch
let verifier_index = state.verifier_index.clone();
let verifier_srs = state.verifier_srs.clone();
let dispatcher = state_context.into_dispatcher();
dispatcher.push(SnarkBlockVerifyEffectfulAction::Init {
    req_id: *req_id,
    block: block.clone(),
    verifier_index,
    verifier_srs,
});
dispatcher.push(SnarkBlockVerifyAction::Pending { req_id: *req_id });
```

Finally, on the handling of the `SnarkBlockVerifyAction::Success` action, the internal state is updated, and the callback fetched and dispatched with the block hash as input.

```rust
let callback_and_arg = state.jobs.get_mut(*req_id).and_then(|req| {
    if let SnarkBlockVerifyStatus::Pending {
        block, on_success, ..
    } = req
    {
        let callback = on_success.clone();
        let block_hash = block.hash_ref().clone();
        *req = SnarkBlockVerifyStatus::Success {
            time: meta.time(),
            block: block.clone(),
        };
        Some((callback, block_hash))
    } else {
        None
    }
});

// Dispatch
let dispatcher = state_context.into_dispatcher();

if let Some((callback, block_hash)) = callback_and_arg {
    dispatcher.push_callback(callback, block_hash);
}

dispatcher.push(SnarkBlockVerifyAction::Finish { req_id: *req_id });
```
