use node::{ActionKind, ActionWithMeta, Store};
use strum::VariantArray;

use crate::{Invariant, InvariantResult};

/// Makes sure we don't have cycles in action chain.
///
/// Cycles in action chain can cause whole lot of issues:
/// 1. Stack overflow (as long as actions live on stack).
/// 2. Infinite loop.
/// 3. Harder to reason about and debug state machine.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct NoRecursion;

impl Invariant for NoRecursion {
    type InternalState = Vec<ActionKind>;
    fn triggers(&self) -> &[ActionKind] {
        ActionKind::VARIANTS
    }

    fn check<S: redux::Service>(
        self,
        action_stack: &mut Self::InternalState,
        _: &Store<S>,
        action: &ActionWithMeta,
    ) -> InvariantResult {
        let action_kind = action.action().kind();
        let action_depth = action.depth() as usize;

        if action_stack.len() < action_depth {
            assert_eq!(action_stack.len() + 1, action_depth);
        } else {
            action_stack.truncate(action_depth.saturating_sub(1));
        }

        // let is_recursing = action_stack.iter().any(|kind| *kind == action_kind);
        let is_recursing = contains_cycle(action_stack);
        action_stack.push(action_kind);

        if is_recursing {
            InvariantResult::Violation(format!("recursion detected: {action_stack:?}"))
        } else {
            InvariantResult::Updated
        }
    }
}

fn contains_cycle(stack: &[ActionKind]) -> bool {
    let len = stack.len();
    if len < 2 {
        return false; // No cycle possible if the stack is too short
    }

    for cycle_length in 1..=(len / 2) {
        let slice1 = &stack[len - cycle_length..];
        let slice2 = &stack[len - 2 * cycle_length..len - cycle_length];
        if slice1 == slice2 {
            return true;
        }
    }
    false
}
