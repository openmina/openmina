use std::{collections::VecDeque, marker::PhantomData};

use crate::{AnyAction, Callback, EnablingCondition};

pub struct Dispatcher<Action, State> {
    queue: VecDeque<Action>,
    _marker: PhantomData<State>,
}

impl<Action, State> Default for Dispatcher<Action, State>
where
    Action: crate::EnablingCondition<State>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Action, State> Dispatcher<Action, State>
where
    Action: crate::EnablingCondition<State>,
{
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            _marker: Default::default(),
        }
    }

    pub fn push<T>(&mut self, action: T)
    where
        T: Into<Action>,
    {
        self.queue.push_back(action.into());
    }

    pub fn push_if_enabled<T>(&mut self, action: T, state: &State, time: crate::Timestamp) -> bool
    where
        T: Into<Action> + EnablingCondition<State>,
    {
        if action.is_enabled(state, time) {
            self.queue.push_back(action.into());
            true
        } else {
            false
        }
    }

    pub fn push_callback<T>(&mut self, callback: Callback<T>, args: T)
    where
        T: 'static,
        Action: From<AnyAction>,
    {
        let action: Action = callback.call(args);
        self.queue.push_back(action);
    }

    pub(crate) fn pop(&mut self) -> Option<Action> {
        self.queue.pop_front()
    }
}
