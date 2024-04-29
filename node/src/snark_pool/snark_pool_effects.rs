use crate::{Service, SnarkPoolEffectfulAction, Store};

use super::SnarkPoolEffectfulActionWithMeta;

pub fn snark_pool_effects<S: Service>(
    store: &mut Store<S>,
    action: SnarkPoolEffectfulActionWithMeta,
) {
    let (action, _meta) = action.split();

    match action {
        SnarkPoolEffectfulAction::SnarkPoolJobsRandomChoose {
            choices,
            count,
            on_result,
        } => {
            let job_ids = store.service.random_choose(choices.iter(), count);
            store.dispatch_callback(on_result, job_ids);
        }
    }
}
