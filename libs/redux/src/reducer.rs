use crate::{ActionWithMeta, Dispatcher};

/// Function signature for a reducer.
pub type Reducer<State, Action> =
    fn(&mut State, &ActionWithMeta<Action>, &mut Dispatcher<Action, State>);
