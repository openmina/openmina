///#![feature(trivial_bounds)]
pub mod channels;
pub mod connection;
pub mod disconnection;
pub mod discovery;
pub mod identity;
pub mod peer;
use bootstrap::P2pNetworkKadBootstrapState;
use connection::incoming::P2pConnectionIncomingAction;
use identify::P2pIdentifyAction;
pub use identity::PeerId;
use network::identify::{
    stream_effectful::P2pNetworkIdentifyStreamEffectfulAction, P2pNetworkIdentifyState,
    P2pNetworkIdentifyStreamAction,
};
use openmina_core::SubstateAccess;

pub mod webrtc;

pub mod identify;

pub mod network;
pub use self::network::*;

mod p2p_config;
pub use p2p_config::*;

mod p2p_event;
pub use p2p_event::*;

mod p2p_actions;
pub use p2p_actions::*;

mod p2p_state;
pub use p2p_state::*;

mod p2p_reducer;

mod p2p_effects;
pub use self::p2p_effects::*;

mod p2p_service;
pub use p2p_service::*;
pub mod service {
    pub use super::p2p_service::*;
}

pub mod service_impl;

pub use libp2p_identity;
pub use multiaddr;

#[cfg(all(
    not(target_arch = "wasm32"),
    feature = "p2p-libp2p",
    feature = "fuzzing"
))]
pub mod fuzzer;

use redux::{EnablingCondition, SubStore};

pub trait P2pStore<GlobalState>: SubStore<GlobalState, P2pState, SubAction = P2pAction> {}
impl<S, T: SubStore<S, P2pState, SubAction = P2pAction>> P2pStore<S> for T {}

/// Returns true if duration value is configured, and, given the time is `now`,
/// that duration is passed since `then`.
fn is_time_passed(
    now: redux::Timestamp,
    then: redux::Timestamp,
    duration: Option<std::time::Duration>,
) -> bool {
    duration.map_or(false, |d| now.checked_sub(then) >= Some(d))
}

pub trait P2pStateTrait:
    SubstateAccess<P2pState>
    + SubstateAccess<P2pNetworkState>
    + SubstateAccess<P2pNetworkKadState>
    + SubstateAccess<P2pNetworkKadBootstrapState>
    + SubstateAccess<P2pNetworkIdentifyState>
    + SubstateAccess<P2pNetworkSchedulerState>
    + SubstateAccess<P2pLimits>
{
}

pub trait P2pActionTrait<State>:
    EnablingCondition<State>
    + From<P2pAction>
    + From<P2pNetworkKademliaStreamAction>
    + From<P2pNetworkKadRequestAction>
    + From<P2pNetworkKadBootstrapAction>
    + From<connection::outgoing::P2pConnectionOutgoingAction>
    + From<P2pNetworkYamuxAction>
    + From<peer::P2pPeerAction>
    + From<P2pNetworkKademliaAction>
    + From<P2pNetworkSchedulerAction>
    + From<P2pNetworkIdentifyStreamAction>
    + From<P2pIdentifyAction>
    + From<P2pNetworkIdentifyStreamEffectfulAction>
    + From<P2pNetworkSelectAction>
    + From<P2pNetworkPnetAction>
    + From<P2pNetworkNoiseAction>
    + From<P2pConnectionIncomingAction>
{
}
