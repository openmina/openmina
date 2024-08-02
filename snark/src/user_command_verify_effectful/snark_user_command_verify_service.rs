use std::sync::{Arc, Mutex};

use mina_p2p_messages::{list::List, v2};

use crate::{VerifierIndex, VerifierSRS};

use super::SnarkUserCommandVerifyId;

pub trait SnarkUserCommandVerifyService: redux::Service {
    fn verify_init(
        &mut self,
        req_id: SnarkUserCommandVerifyId,
        verifier_index: Arc<VerifierIndex>,
        verifier_srs: Arc<Mutex<VerifierSRS>>,
        commands: List<v2::MinaBaseUserCommandStableV2>,
    );
}
