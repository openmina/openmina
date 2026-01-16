//! SNARK verification integration for the node.
//!
//! This module integrates the SNARK verification state machine with the node's
//! Redux store, enabling verification of:
//!
//! - **Block proofs**: Consensus-layer proofs validating block production
//! - **Transaction proofs**: Ledger proofs for transaction validity (SNARK work)
//! - **User commands**: Signatures and zkApp proofs for user transactions
//!
//! ## Architecture
//!
//! The module re-exports [`mina_snark`] and provides the
//! [`redux::SubStore`] implementation that connects the SNARK state machine
//! to the node's global state.
//!
//! Verification runs in dedicated threads to avoid blocking the main Redux
//! loop:
//!
//! - Block verification: Single dedicated thread (`block_proof_verifier`)
//! - Work/command verification: Rayon thread pool with FIFO scheduling
//!
//! For the underlying verification implementation, see [`mina_snark`] and
//! [`ledger::proofs::verification`].

pub use ::mina_snark::*;

pub mod block_verify;
pub mod user_command_verify;
pub mod work_verify;

mod snark_effects;
pub use snark_effects::*;

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

    fn dispatch_callback<T>(&mut self, callback: redux::Callback<T>, args: T) -> bool
    where
        T: 'static,
        SnarkAction: From<redux::AnyAction> + redux::EnablingCondition<SnarkState>,
    {
        crate::Store::dispatch_callback(self, callback, args)
    }
}
