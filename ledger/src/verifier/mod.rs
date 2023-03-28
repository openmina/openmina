use crate::scan_state::{
    scan_state::transaction_snark::LedgerProofWithSokMessage,
    transaction_logic::{valid, verifiable, WithStatus},
};

use self::common::CheckResult;

#[derive(Debug, Clone)]
pub struct Verifier;

impl Verifier {
    pub fn verify(&self, _proofs: &[LedgerProofWithSokMessage]) -> Result<Result<(), ()>, String> {
        // Implement verification later
        //
        // https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/pickles/pickles.ml#L1122
        // https://viable-systems.slack.com/archives/D01SVA87PQC/p1671715846448749
        Ok(Ok(()))
    }

    pub fn verify_commands(
        &self,
        cmds: Vec<WithStatus<verifiable::UserCommand>>,
    ) -> Result<Vec<valid::UserCommand>, VerifierError> {
        // TODO

        let xs: Vec<_> = cmds
            .into_iter()
            .map(common::check)
            .map(|cmd| {
                match cmd {
                    common::CheckResult::Valid(cmd) => Ok(cmd),
                    e => Err(e)
                // common::CheckResult::ValidAssuming(_) => todo!(),
                // common::CheckResult::InvalidKeys(_) => todo!(),
                // common::CheckResult::InvalidSignature(_) => todo!(),
                // common::CheckResult::InvalidProof => todo!(),
                // common::CheckResult::MissingVerificationKey(_) => todo!(),
            }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(xs)
    }
}

#[derive(Debug, derive_more::From)]
pub enum VerifierError {
    CheckError(CheckResult),
}

pub mod common {
    use mina_signer::CompressedPubKey;

    use crate::scan_state::transaction_logic::{valid, verifiable, zkapp_command, WithStatus};

    #[derive(Debug)]
    pub enum CheckResult {
        Valid(valid::UserCommand),
        ValidAssuming((valid::UserCommand, Vec<()>)),
        InvalidKeys(Vec<CompressedPubKey>),
        InvalidSignature(Vec<CompressedPubKey>),
        InvalidProof,
        MissingVerificationKey(Vec<CompressedPubKey>),
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/verifier/common.ml#L29
    pub fn check(cmd: WithStatus<verifiable::UserCommand>) -> CheckResult {
        use verifiable::UserCommand::{SignedCommand, ZkAppCommand};

        // TODO: Implement

        match cmd.data {
            SignedCommand(cmd) => {
                if !cmd.check_valid_keys() {
                    let public_keys = cmd.public_keys().into_iter().cloned().collect();
                    return CheckResult::InvalidKeys(public_keys);
                }
                match verifiable::check_only_for_signature(cmd) {
                    Ok(cmd) => CheckResult::Valid(cmd),
                    Err(cmd) => CheckResult::InvalidSignature(
                        cmd.public_keys().into_iter().cloned().collect(),
                    ),
                }
            }
            ZkAppCommand(cmd) => {
                // TODO: Implement rest

                match zkapp_command::valid::of_verifiable(*cmd) {
                    Some(cmd) => {
                        CheckResult::Valid(valid::UserCommand::ZkAppCommand(Box::new(cmd)))
                    }
                    None => CheckResult::InvalidProof, // TODO
                }
            }
        }
    }
}
