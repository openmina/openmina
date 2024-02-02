use ::node::{ActionWithMeta, Store};
use openmina_node_invariants::{InvariantResult, Invariants};
use openmina_node_native::NodeService;

pub mod ret {
    macro_rules! define {
        (
            $(
                $(#[$docs:meta])*
                ($num:expr, $konst:ident);
            )+
        ) => {
            $(
                $(#[$docs])*
                pub const $konst: u8 = $num;
            )+
        }
    }

    define! {
        /// Continue till next action after which `replay_dynamic_effects`
        /// will be called again.
        (0, CONTINUE);
        /// Pause execution and wait for `replay_dynamic_effects` lib to be modified.
        (1, PAUSE);
    }
}

#[no_mangle]
extern "C" fn replay_dynamic_effects(
    store: &mut Store<NodeService>,
    action: &ActionWithMeta,
) -> u8 {
    for (invariant, res) in Invariants::check_all(store, action) {
        match res {
            InvariantResult::Violation(violation) => {
                eprintln!(
                    "Invariant({}) violated! violation: {violation}",
                    invariant.to_str()
                );
                return ret::PAUSE;
            }
            InvariantResult::Updated => {}
            InvariantResult::Ok => {}
        }
    }

    let (action, meta) = (action.action(), action.meta().clone());
    let state = store.state.get();
    let _ = (state, meta, action);

    ret::CONTINUE
}
