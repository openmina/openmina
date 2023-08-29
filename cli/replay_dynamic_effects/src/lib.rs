use cli::commands::snarker::SnarkerService;

use ::snarker::{ActionWithMeta, Store};

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
    #[allow(unused_variables)] store: &mut Store<SnarkerService>,
    #[allow(unused_variables)] action: &ActionWithMeta,
) -> u8 {
    ret::CONTINUE
}
