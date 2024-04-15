use redux::ActionWithMeta;

use super::stream::P2pNetworkFloodsubStreamAction;

impl super::P2pNetworkFloodsubState {
    pub fn reducer(
        &mut self,
        action: ActionWithMeta<&super::P2pNetworkFloodsubAction>,
    ) -> Result<(), String> {
        let (action, meta) = action.split();
        match action {
            super::P2pNetworkFloodsubAction::Stream(
                action @ P2pNetworkFloodsubStreamAction::New { .. },
            ) => self
                .create_floodsub_stream_state(action.peer_id(), action.stream_id())
                .map_err(|stream| {
                    format!("Floodsub stream already exists for action {action:?}: {stream:?}")
                })
                .and_then(|stream| stream.reducer(meta.with_action(action))),
            super::P2pNetworkFloodsubAction::Stream(
                action @ P2pNetworkFloodsubStreamAction::Prune { .. },
            ) => self
                .remove_floodsub_stream_state(action.peer_id(), action.stream_id())
                .then_some(())
                .ok_or_else(|| format!("Floodsub stream not found for action {action:?}")),
            super::P2pNetworkFloodsubAction::Stream(action) => self
                .find_floodsub_stream_state_mut(action.peer_id(), action.stream_id())
                .ok_or_else(|| format!("Floodsub stream not found for action {action:?}"))
                .and_then(|stream| stream.reducer(meta.with_action(action))),
        }
    }
}
