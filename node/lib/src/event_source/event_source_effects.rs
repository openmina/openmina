use crate::action::CheckTimeoutsAction;
use crate::{Service, Store};

use super::{EventSourceAction, EventSourceActionWithMeta};

pub fn event_source_effects<S: Service>(store: &mut Store<S>, action: EventSourceActionWithMeta) {
    let (action, _) = action.split();
    match action {
        EventSourceAction::ProcessEvents(_) => {
            // process max 1024 events at a time.
            for _ in 0..1024 {
                match store.service.next_event() {
                    Some(event) => {}
                    None => break,
                }
            }
            store.dispatch(CheckTimeoutsAction {});
        }
        EventSourceAction::WaitTimeout(_) => {
            store.dispatch(CheckTimeoutsAction {});
        }
        EventSourceAction::WaitForEvents(_) => {}
    }
}
