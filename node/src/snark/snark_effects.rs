use crate::{Service, Store};

use super::{SnarkAction, SnarkActionWithMeta};

pub fn snark_effects<S: Service>(store: &mut Store<S>, action: SnarkActionWithMeta) {
    let (action, meta) = action.split();

    match action {
        SnarkAction::BlockVerify(_) => {}
        SnarkAction::BlockVerifyEffect(a) => {
            a.effects(&meta, store);
        }
        SnarkAction::WorkVerify(_) => {}
        SnarkAction::WorkVerifyEffect(a) => {
            a.effects(&meta, store);
        }
    }
}
