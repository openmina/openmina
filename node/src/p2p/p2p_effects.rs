use crate::{Service, Store};
use p2p::P2pEffectfulAction;
use redux::ActionWithMeta;

pub fn node_p2p_effects<S: Service>(
    store: &mut Store<S>,
    action: ActionWithMeta<P2pEffectfulAction>,
) {
    let (action, meta) = action.split();

    match action {
        P2pEffectfulAction::Initialize =>
        {
            #[cfg(feature = "p2p-libp2p")]
            if store.state().p2p.ready().is_some() {
                store.service().start_mio();
            }
        }
        action => action.effects(meta, store),
    }
}
