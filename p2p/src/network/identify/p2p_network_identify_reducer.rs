use redux::ActionWithMeta;

use crate::P2pLimits;

use super::stream::P2pNetworkIdentifyStreamAction;

impl super::P2pNetworkIdentifyState {
    pub fn reducer(
        &mut self,
        action: ActionWithMeta<&super::P2pNetworkIdentifyAction>,
        limits: &P2pLimits,
    ) -> Result<(), String> {
        let (action, meta) = action.split();
        match action {
            super::P2pNetworkIdentifyAction::Stream(
                action @ P2pNetworkIdentifyStreamAction::New { .. },
            ) => self
                .create_identify_stream_state(action.peer_id(), action.stream_id())
                .map_err(|stream| {
                    format!("Identify stream already exists for action {action:?}: {stream:?}")
                })
                .and_then(|stream| stream.reducer(meta.with_action(action), limits)),
            super::P2pNetworkIdentifyAction::Stream(
                action @ P2pNetworkIdentifyStreamAction::Prune { .. },
            ) => self
                .remove_identify_stream_state(action.peer_id(), action.stream_id())
                .then_some(())
                .ok_or_else(|| format!("Identify stream not found for action {action:?}")),
            super::P2pNetworkIdentifyAction::Stream(action) => self
                .find_identify_stream_state_mut(action.peer_id(), action.stream_id())
                .ok_or_else(|| format!("Identify stream not found for action {action:?}"))
                .and_then(|stream| stream.reducer(meta.with_action(action), limits)),
        }
    }
}
