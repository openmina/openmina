# Node Invariants

Defines node invariants that must always hold true.

For performance reasons, invariants won't be checked when running the node,
but they will be checked when using node replayer or when running testing
scenarios/simulations.

## Creating a new invariant

1. Add a new struct with an unique name, ideally name of which makes
   it clear what it's about.
2. Derive macros: ` #[derive(documented::Documented, Default, Clone, Copy)]`.
3. Add doc comment to the struct further describing what invariant checks for.
4. Implement an `Invariant` trait for it.
5. Add an invariant in the [invariants definition list](src/lib.rs#L64).


## Invariant internal state

```rust
trait Invariant {
    type InternalState;
    ...
}
```


Internal state of the invariant can be used to preserve state across
this invariant checks.

With this an `Invariant` can be used to represent either safety or liveness
conditions (concepts from TLA+).

- If state doesn't need to be preserved across invariant checks, then we
are working with stateless safety condition, which can be represented
by an `Invariant` where `Invariant::InternalState = ()`.
- If state does need to be preserved across invariant checks, then we
are working with liveness condition, which can be represented
by an `Invariant` where `Invariant::InternalState = TypeRepresentingNecessaryData`.

Can be any type satisfying bounds: `InternalState: 'static + Send + Default`.

Storing and loading of the internal state is fully taken care of by
the framework.

## Invariant triggers

```rust
trait Invariant {
    ...
    fn triggers(&self) -> &[ActionKind];
    ...
}
```

Invariant `triggers` define a list actions, which should cause
`Invariant::check` to be called.

If empty, an invariant will never be checked!
