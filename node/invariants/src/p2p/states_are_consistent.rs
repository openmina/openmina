use std::collections::BTreeSet;

use node::{p2p::PeerId, ActionKind, ActionWithMeta, Service, Store};

use crate::{Invariant, InvariantResult};

/// Makes sure that:
/// 1. For WebRTC peers, we have same number of peers in state and service.
/// 2. TODO: libp2p
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct P2pStatesAreConsistent;

impl Invariant for P2pStatesAreConsistent {
    type InternalState = ();
    fn triggers(&self) -> &[ActionKind] {
        &[ActionKind::P2pPeerReady, ActionKind::P2pDisconnectionFinish]
    }

    fn check<S: Service>(
        self,
        _: &mut Self::InternalState,
        store: &Store<S>,
        _action: &ActionWithMeta,
    ) -> InvariantResult {
        if let Some((missing_connections, extra_connections)) =
            self.webrtc_peer_inconsistencies(store)
        {
            return InvariantResult::Violation(format!("WebRTC inconsistency! missing_connections:\n{missing_connections:?}\nextra_connections:\n{extra_connections:?}"));
        }

        InvariantResult::Ok
    }
}

impl P2pStatesAreConsistent {
    fn webrtc_peer_inconsistencies<S: Service>(
        self,
        store: &Store<S>,
    ) -> Option<(BTreeSet<PeerId>, BTreeSet<PeerId>)> {
        if store.service.is_replay() {
            return None;
        }
        let Some(p2p_state) = store.state().p2p.ready() else {
            return None;
        };
        let mut connections = store.service.connections();
        let peers = p2p_state
            .peers
            .iter()
            .filter(|(_, s)| !s.is_libp2p() && s.status.is_connected_or_connecting())
            .map(|(peer_id, _)| *peer_id)
            .filter(|peer_id| connections.remove(peer_id))
            .collect::<BTreeSet<_>>();

        if !peers.is_empty() || !connections.is_empty() {
            return Some((peers, connections));
        }
        None
    }
}
