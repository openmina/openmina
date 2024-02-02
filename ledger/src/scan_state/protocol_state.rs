use mina_hasher::Fp;
use mina_p2p_messages::v2::{
    MinaStateProtocolStateBodyValueStableV2, MinaStateProtocolStateValueStableV2,
};

use crate::{
    hash::{hash_with_kimchi, Inputs},
    ToInputs,
};

pub trait MinaHash {
    fn hash(&self) -> Fp;
}

impl MinaHash for MinaStateProtocolStateBodyValueStableV2 {
    fn hash(&self) -> Fp {
        self.hash_with_param("MinaProtoStateBody")
    }
}

/// Returns (state_hash, state_body_hash)
pub fn hashes(state: &MinaStateProtocolStateValueStableV2) -> (Fp, Fp) {
    let state_body_hash = MinaHash::hash(&state.body);

    let state_hash = { hashes_abstract(state.previous_state_hash.to_field(), state_body_hash) };

    (state_hash, state_body_hash)
}

pub fn hashes_abstract(previous_state_hash: Fp, body_hash: Fp) -> Fp {
    let mut inputs = Inputs::new();

    inputs.append_field(previous_state_hash);
    inputs.append_field(body_hash);

    hash_with_kimchi("MinaProtoState", &inputs.to_fields())
}

impl MinaHash for MinaStateProtocolStateValueStableV2 {
    fn hash(&self) -> Fp {
        let previous_state_hash = self.previous_state_hash.to_field();
        let body_hash = MinaHash::hash(&self.body);

        hashes_abstract(previous_state_hash, body_hash)
    }
}
