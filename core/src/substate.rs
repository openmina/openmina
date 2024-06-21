use redux::Dispatcher;

pub type SubstateResult<T> = Result<T, String>;

/// A trait for obtaining immutable and mutable references to a substate of type `T`.
///
/// # Example
///
/// ```ignore
/// impl SubstateAccess<P2pState> for State {
///    fn substate(&self) -> SubstateResult<&P2pState> {
///        self.p2p
///            .ready()
///            .ok_or_else(|| "P2P state unavailable. P2P layer is not ready".to_owned())
///    }
///  
///    fn substate_mut(&mut self) -> SubstateResult<&mut P2pState> {
///        self.p2p
///            .ready_mut()
///            .ok_or_else(|| "P2P state unavailable. P2P layer is not ready".to_owned())
///    }
///  }
/// ```
pub trait SubstateAccess<T> {
    /// Attempts to obtain an immutable reference to the substate.
    ///
    /// In case of failure, an error `String` describing the reason is returned.
    fn substate(&self) -> SubstateResult<&T>;

    /// Attempts to obtain a mutable reference to the substate.
    ///
    /// In case of failure, an error `String` describing the reason is returned.
    fn substate_mut(&mut self) -> SubstateResult<&mut T>;
}

/// Substate context which provides mutable access to a substate of type `S`, and can
/// be consumed to obtain a [redux::Dispatcher] and a parent state of type `T`.
pub struct Substate<'a, A, T, S> {
    state: &'a mut T,
    dispatcher: &'a mut Dispatcher<A, T>,
    _marker: std::marker::PhantomData<S>,
}

impl<'a, A, T, S> Substate<'a, A, T, S>
where
    T: SubstateAccess<S>,
{
    /// Creates a new instance from a parent state and dispatcher.
    pub fn new(state: &'a mut T, dispatcher: &'a mut Dispatcher<A, T>) -> Self {
        Self {
            state,
            dispatcher,
            _marker: Default::default(),
        }
    }

    /// Creates a new instance from an already existing [Substate] for the same parent state.
    pub fn from_compatible_substate<OS>(other: Substate<'a, A, T, OS>) -> Substate<'a, A, T, S> {
        let Substate {
            state, dispatcher, ..
        } = other;

        Self::new(state, dispatcher)
    }

    /// Attempts to obtain a mutable reference to the substate.
    ///
    /// In case of failure, an error `String` describing the reason is returned.
    pub fn get_substate(&self) -> SubstateResult<&S> {
        self.state.substate()
    }

    /// Attempts to obtain a mutable reference to the substate.
    ///
    /// In case of failure, an error `String` describing the reason is returned.
    pub fn get_substate_mut(&mut self) -> SubstateResult<&mut S> {
        self.state.substate_mut()
    }

    /// Consumes itself to produce a reference to a dispatcher and parent state.
    pub fn into_dispatcher_and_state(self) -> (&'a mut Dispatcher<A, T>, &'a T) {
        (self.dispatcher, self.state)
    }

    /// Consumes itself to produce a reference to a dispatcher.
    pub fn into_dispatcher(self) -> &'a mut Dispatcher<A, T> {
        self.dispatcher
    }
}

/// Helper macro for the trivial substate access pattern.
///
/// # Example:
///
/// ```ignore
/// impl_substate_access!(State, TransitionFrontierSyncState, transition_frontier.sync);
/// ```
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
