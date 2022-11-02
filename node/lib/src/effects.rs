use crate::event_source::event_source_effects;
use crate::p2p::p2p_effects;
use crate::{Action, ActionWithMeta, Service, Store};

pub fn effects<S: Service>(store: &mut Store<S>, action: ActionWithMeta) {
    let (action, meta) = action.split();
    match action {
        Action::CheckTimeouts(_) => {}
        Action::EventSource(action) => {
            event_source_effects(store, meta.with_action(action));
        }
        Action::P2p(action) => {
            p2p_effects(store, meta.with_action(action));
        }
    }
}
