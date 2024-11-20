use super::ExternalSnarkWorkerEffectfulAction;
use crate::ExternalSnarkWorkerAction;
use redux::ActionWithMeta;

pub fn external_snark_worker_effectful_effects<S: crate::Service>(
    store: &mut crate::Store<S>,
    action: ActionWithMeta<ExternalSnarkWorkerEffectfulAction>,
) {
    let (action, _) = action.split();
    match action {
        ExternalSnarkWorkerEffectfulAction::Start { public_key, fee } => {
            if let Err(err) = store.service.start(public_key, fee) {
                store.dispatch(ExternalSnarkWorkerAction::Error {
                    error: err,
                    permanent: true,
                });
            }
        }
        ExternalSnarkWorkerEffectfulAction::Kill => {
            if let Err(err) = store.service().kill() {
                store.dispatch(ExternalSnarkWorkerAction::Error {
                    error: err,
                    permanent: true,
                });
            }
        }
        ExternalSnarkWorkerEffectfulAction::SubmitWork { spec } => {
            if let Err(err) = store.service().submit(*spec) {
                store.dispatch(ExternalSnarkWorkerAction::WorkError { error: err.into() });
            }
        }
        ExternalSnarkWorkerEffectfulAction::CancelWork => {
            if let Err(error) = store.service().cancel() {
                store.dispatch(ExternalSnarkWorkerAction::Error {
                    error,
                    permanent: true,
                });
            }
        }
    }
}
