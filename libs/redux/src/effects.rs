use crate::{ActionWithMeta, Store};

pub type Effects<State, Service, Action> =
    fn(&mut Store<State, Service, Action>, ActionWithMeta<Action>);
