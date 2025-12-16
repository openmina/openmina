use std::sync::Arc;

use crate::snark::{
    user_command_verify::SnarkUserCommandVerifyId, TransactionVerifier, VerifierSRS,
};
use ledger::scan_state::transaction_logic::{verifiable, WithStatus};

pub trait VerifyUserCommandsService: redux::Service {
    fn verify_init(
        &mut self,
        req_id: SnarkUserCommandVerifyId,
        commands: Vec<WithStatus<verifiable::UserCommand>>,
        verifier_index: TransactionVerifier,
        verifier_srs: Arc<VerifierSRS>,
    );
}
