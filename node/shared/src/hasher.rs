use std::cell::RefCell;

use mina_p2p_messages::v1::{MinaStateProtocolStateValueStableV1VersionedV1, StateHashStable};

pub type StateHash = StateHashStable;

thread_local! {
    pub static STATE_HASHER: RefCell<mina_hasher::PoseidonHasherLegacy<MinaStateProtocolStateValueStableV1VersionedV1>> = RefCell::new(mina_hasher::create_legacy(()));
}

pub fn protocol_state_hash(
    protocol_state: &MinaStateProtocolStateValueStableV1VersionedV1,
) -> StateHash {
    STATE_HASHER.with(|hasher| protocol_state.hash(&mut *hasher.borrow_mut()))
}
