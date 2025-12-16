use mina_curves::pasta::Fp;

use crate::{proofs::block::ProtocolState, ToInputs};
use poseidon::hash::{
    hash_with_kimchi,
    params::{MINA_PROTO_STATE, MINA_PROTO_STATE_BODY},
    Inputs,
};

pub trait MinaHash {
    fn hash(&self) -> Fp;
}

impl MinaHash for crate::proofs::block::ProtocolStateBody {
    fn hash(&self) -> Fp {
        self.hash_with_param(&MINA_PROTO_STATE_BODY)
    }
}

pub fn hashes_abstract(previous_state_hash: Fp, body_hash: Fp) -> Fp {
    let mut inputs = Inputs::new();

    inputs.append_field(previous_state_hash);
    inputs.append_field(body_hash);

    hash_with_kimchi(&MINA_PROTO_STATE, &inputs.to_fields())
}

impl ProtocolState {
    /// Returns (state_hash, state_body_hash)
    pub fn hashes(&self) -> (Fp, Fp) {
        let Self {
            previous_state_hash,
            body,
        } = self;

        let state_body_hash = MinaHash::hash(body);
        let state_hash = hashes_abstract(*previous_state_hash, state_body_hash);
        (state_hash, state_body_hash)
    }
}

impl MinaHash for ProtocolState {
    fn hash(&self) -> Fp {
        let Self {
            previous_state_hash,
            body,
        } = self;

        let body_hash = MinaHash::hash(body);
        hashes_abstract(*previous_state_hash, body_hash)
    }
}
