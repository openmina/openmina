use ledger::scan_state::transaction_logic::{verifiable, WithStatus};

use super::SnarkUserCommandVerifyId;

pub trait SnarkUserCommandVerifyService: redux::Service {
    fn verify_init(
        &mut self,
        req_id: SnarkUserCommandVerifyId,
        commands: Vec<WithStatus<verifiable::UserCommand>>,
    );
}
