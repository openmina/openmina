use mina_curves::pasta::Fp;

use super::{
    verifiable::{self, create},
    AccountId, TransactionStatus, VerificationKeyWire,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ZkAppCommand {
    pub zkapp_command: super::ZkAppCommand,
}

impl ZkAppCommand {
    pub fn forget(self) -> super::ZkAppCommand {
        self.zkapp_command
    }
    pub fn forget_ref(&self) -> &super::ZkAppCommand {
        &self.zkapp_command
    }
}

/// <https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/zkapp_command.ml#L1499>
pub fn of_verifiable(cmd: verifiable::ZkAppCommand) -> ZkAppCommand {
    ZkAppCommand {
        zkapp_command: super::ZkAppCommand::of_verifiable(cmd),
    }
}

/// <https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/zkapp_command.ml#L1507>
pub fn to_valid(
    zkapp_command: super::ZkAppCommand,
    status: &TransactionStatus,
    find_vk: impl Fn(Fp, &AccountId) -> Result<VerificationKeyWire, String>,
) -> Result<ZkAppCommand, String> {
    create(&zkapp_command, status.is_failed(), find_vk).map(of_verifiable)
}
