use std::sync::{Arc, Mutex};

use ledger::scan_state::transaction_logic::{verifiable, WithStatus};

use crate::{VerifierIndex, VerifierSRS};

use super::SnarkUserCommandVerifyId;

pub trait SnarkUserCommandVerifyService: redux::Service {
    fn verify_init(
        &mut self,
        req_id: SnarkUserCommandVerifyId,
        commands: Vec<WithStatus<verifiable::UserCommand>>,
        verifier_index: Arc<VerifierIndex>,
        verifier_srs: Arc<Mutex<VerifierSRS>>,
    );
}
