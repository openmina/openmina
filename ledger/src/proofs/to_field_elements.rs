use mina_hasher::Fp;
use mina_p2p_messages::v2::MinaStateProtocolStateValueStableV2;

use crate::{
    proofs::public_input::protocol_state::MinaHash,
    scan_state::{
        scan_state::transaction_snark::{SokDigest, Statement},
        transaction_logic::zkapp_statement::ZkappStatement,
    },
};

pub trait ToFieldElements {
    fn to_field_elements(&self) -> Vec<Fp>;
}

impl ToFieldElements for MinaStateProtocolStateValueStableV2 {
    fn to_field_elements(&self) -> Vec<Fp> {
        vec![MinaHash::hash(self)]
    }
}

impl ToFieldElements for Statement<SokDigest> {
    fn to_field_elements(&self) -> Vec<Fp> {
        let mut inputs = crate::Inputs::new();
        inputs.append(self);

        inputs.to_fields()
    }
}

impl ToFieldElements for ZkappStatement {
    fn to_field_elements(&self) -> Vec<Fp> {
        self.to_field_elements()
    }
}
