pub use ::snark::*;

pub mod block_verify;
pub mod work_verify;

impl<S> redux::SubStore<crate::State, SnarkState> for crate::Store<S>
where
    S: redux::Service,
{
    type SubAction = SnarkAction;
    type Service = S;

    fn state(&self) -> &SnarkState {
        &self.state.get().snark
    }

    fn service(&mut self) -> &mut Self::Service {
        &mut self.service
    }

    fn state_and_service(&mut self) -> (&SnarkState, &mut Self::Service) {
        (&self.state.get().snark, &mut self.service)
    }

    fn dispatch<A>(&mut self, action: A) -> bool
    where
        A: Into<SnarkAction> + redux::EnablingCondition<SnarkState>,
    {
        crate::Store::sub_dispatch(self, action)
    }

    fn dispatch_callback<T>(&mut self, callback: redux::Callback, args: T) -> bool
    where
        T: 'static,
    {
        crate::Store::dispatch_callback(self, callback, args)
    }
}
