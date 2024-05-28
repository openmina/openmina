use redux::Dispatcher;

pub type SubstateResult<T> = Result<T, String>;

pub trait SubstateAccess<T> {
    fn substate(&self) -> SubstateResult<&T>;
    fn substate_mut(&mut self) -> SubstateResult<&mut T>;
}

pub struct Substate<'a, A, T, S> {
    state: &'a mut T,
    dispatcher: &'a mut Dispatcher<A, T>,
    _marker: std::marker::PhantomData<S>,
}

impl<'a, A, T, S> Substate<'a, A, T, S>
where
    T: SubstateAccess<S>,
{
    pub fn new(state: &'a mut T, dispatcher: &'a mut Dispatcher<A, T>) -> Self {
        Self {
            state,
            dispatcher,
            _marker: Default::default(),
        }
    }

    pub fn from_compatible_substate<OS>(other: Substate<'a, A, T, OS>) -> Substate<'a, A, T, S> {
        let Substate {
            state, dispatcher, ..
        } = other;

        Self::new(state, dispatcher)
    }

    pub fn get_substate(&self) -> SubstateResult<&S> {
        self.state.substate()
    }

    pub fn get_substate_mut(&mut self) -> SubstateResult<&mut S> {
        self.state.substate_mut()
    }

    pub fn into_dispatcher_and_state(self) -> (&'a mut Dispatcher<A, T>, &'a T) {
        (self.dispatcher, self.state)
    }

    pub fn into_dispatcher(self) -> &'a mut Dispatcher<A, T> {
        self.dispatcher
    }
}

impl<'a, A, T, S> From<(&'a mut T, &'a mut Dispatcher<A, T>)> for Substate<'a, A, T, S> {
    fn from(state_and_dispatcher: (&'a mut T, &'a mut Dispatcher<A, T>)) -> Self {
        let (state, dispatcher) = state_and_dispatcher;
        Self {
            state,
            dispatcher,
            _marker: Default::default(),
        }
    }
}

#[macro_export]
macro_rules! impl_substate_access {
    ($state:ty, $substate_type:ty, $($substate_path:tt)*) => {
        impl $crate::SubstateAccess<$substate_type> for $state {
            fn substate(&self) -> $crate::SubstateResult<&$substate_type> {
                Ok(&self.$($substate_path)*)
            }

            fn substate_mut(&mut self) -> $crate::SubstateResult<&mut $substate_type> {
                Ok(&mut self.$($substate_path)*)
            }
        }
    };
}
