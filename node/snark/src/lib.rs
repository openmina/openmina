pub mod block_verify;

mod snark_actions;
pub use snark_actions::*;

mod snark_state;
pub use snark_state::*;

mod snark_reducer;
pub use snark_reducer::*;

use redux::SubStore;
pub trait SnarkStore<GlobalState>:
    SubStore<GlobalState, SnarkState, SubAction = SnarkAction>
{
}
impl<S, T: SubStore<S, SnarkState, SubAction = SnarkAction>> SnarkStore<S> for T {}
