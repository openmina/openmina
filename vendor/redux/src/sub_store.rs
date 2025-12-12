/// Useful when state machine is split into multiple crates. Using this
/// trait we can pass `Store<GlobalState, Service, GlobalAction>`
/// almost as if it were `Store<SubState, Service, SubAction>`.
pub trait SubStore<GlobalState, SubState> {
    type SubAction;
    type Service;

    fn state(&self) -> &SubState;
    fn service(&mut self) -> &mut Self::Service;
    fn state_and_service(&mut self) -> (&SubState, &mut Self::Service);
    fn dispatch<A>(&mut self, action: A) -> bool
    where
        A: Into<Self::SubAction> + crate::EnablingCondition<SubState>;

    fn dispatch_callback<T>(&mut self, callback: crate::Callback<T>, args: T) -> bool
    where
        T: 'static,
        Self::SubAction: From<crate::AnyAction> + crate::EnablingCondition<SubState>;
}
