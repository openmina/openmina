Derives `[mina_core::ActionEvent]` trait implementation for action.

### Action containers

For action containers, it simply delegates to inner actions.

```rust
# use mina_core::ActionEvent;
#
#[derive(ActionEvent)]
enum ActionContainer {
    SubAction1(Action1),
}
#[derive(ActionEvent)]
enum Action1 {
    Init,
    Done,
}

ActionContainer::SubAction1(Action1::Init).action_event(context);
```

```rust
impl ActionEvent for ActionContainer {
    fn action_event<T>(&self, context: &T)
    where T: ActionContext
    {
        match self {
            ActionContainer(action) => action.action_event(context),
        }
    }
}

impl ActionEvent for Action1 {
    fn action_event<T>(&self, context: &T)
    where T: ActionContext
    {
        match self {
            Action1::Init => mina_core::action_debug!(context),
            Action1::Done => mina_core::action_debug!(context),
        }
    }
}
```

### Tracing level

By default, tracing event of level `debug` is generated for an action. It can be
overriden by using `#[action_event(level = ...)]` attribute. Also, actions that
names ends with `Error` or `Warn` will be traced with `warn` level.

```rust
#[derive(mina_core::ActionEvent)]
#[action_event(level = trace)]
pub enum Action {
    ActionDefaultLevel,
    #[action_event(level = warn)]
    ActionOverrideLevel,
    ActionWithError,
    ActionWithWarn,
}
```

```rust
impl mina_core::ActionEvent for Action {
    fn action_event<T>(&self, context: &T)
    where
        T: mina_core::log::EventContext,
    {
        #[allow(unused_variables)]
        match self {
            Action::ActionDefaultLevel => mina_core::action_trace!(context),
            Action::ActionOverrideLevel => mina_core::action_warn!(context),
            Action::ActionWithError => mina_core::action_warn!(context),
            Action::ActionWithWarn => mina_core::action_warn!(context),
        }
    }
}
```

### Summary field

If an action has doc-comment, its first line will be used for `summary` field of
tracing events for the action.

```rust
#[derive(mina_core::ActionEvent)]
pub enum Action {
    Unit,
    /// documentation
    UnitWithDoc,
    /// Multiline documentation.
    /// Another line.
    ///
    /// And another.
    UnitWithMultilineDoc,
}
```

```rust
impl mina_core::ActionEvent for Action {
    fn action_event<T>(&self, context: &T)
    where
        T: mina_core::log::EventContext,
    {
        match self {
            Action::Unit => mina_core::action_debug!(context),
            Action::UnitWithDoc => mina_core::action_debug!(context, summary = "documentation"),
            Action::UnitWithMultilineDoc => mina_core::action_debug!(context, summary = "Multiline documentation"),
        }
    }
}
```

### Fields

Certain fields can be added to the tracing event, using
`#[action_event(fields(...))]` attribute.

```rust
#[derive(mina_core::ActionEvent)]
pub enum Action {
    NoFields { f1: bool },
    #[action_event(fields(f1))]
    Field { f1: bool },
    #[action_event(fields(f = f1))]
    FieldWithName { f1: bool },
    #[action_event(fields(debug(f1)))]
    DebugField { f1: bool },
    #[action_event(fields(display(f1)))]
    DisplayField { f1: bool },
}
```

```rust
impl mina_core::ActionEvent for Action {
    fn action_event<T>(&self, context: &T)
    where
        T: mina_core::log::EventContext,
    {
        match self {
            Action::NoFields { f1 } => mina_core::action_debug!(context),
            Action::Field { f1 } => mina_core::action_debug!(context, f1 = f1),
            Action::FieldWithName { f1 } => mina_core::action_debug!(context, f = f1),
            Action::DebugField { f1 } => mina_core::action_debug!(context, f1 = debug(f1)),
            Action::DisplayField { f1 } => mina_core::action_debug!(context, f1 = display(f1)),
        }
    }
}
```

### Logging using custom expression.

When an action needs some custom logic to log (e.g. different logging basing on
a field's enum variant), logging can be delegated to a function implementing
that logic.

```rust
#[derive(mina_core::ActionEvent)]
pub enum Action {
    #[action_event(expr(foo(context)))]
    Unit,
    #[action_event(expr(bar(context, f1)))]
    Named { f1: bool },
}
```

```rust
impl mina_core::ActionEvent for Action {
    fn action_event<T>(&self, context: &T)
    where
        T: mina_core::log::EventContext,
    {
        #[allow(unused_variables)]
        match self {
            Action::Unit => foo(context),
            Action::Named { f1 } => bar(context, f1),
        }
    }
}
```
