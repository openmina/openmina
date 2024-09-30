use crate::v2::{
    MinaBaseTransactionStatusStableV2, MinaBaseUserCommandStableV2,
    MinaTransactionLogicTransactionAppliedCommandAppliedStableV2,
    MinaTransactionLogicTransactionAppliedVaryingStableV2, MinaTransactionTransactionStableV2,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Binprot error: {0}")]
    BinProt(#[from] binprot::Error),
    #[error("Base64 error: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Extract transaction data and status required by proof generation
impl From<MinaTransactionLogicTransactionAppliedVaryingStableV2>
    for (
        MinaTransactionTransactionStableV2,
        MinaBaseTransactionStatusStableV2,
    )
{
    fn from(value: MinaTransactionLogicTransactionAppliedVaryingStableV2) -> Self {
        match value {
            MinaTransactionLogicTransactionAppliedVaryingStableV2::Command(v) => match v {
                MinaTransactionLogicTransactionAppliedCommandAppliedStableV2::SignedCommand(v) => (
                    MinaTransactionTransactionStableV2::Command(Box::new(
                        MinaBaseUserCommandStableV2::SignedCommand(v.common.user_command.data),
                    )),
                    v.common.user_command.status,
                ),
                MinaTransactionLogicTransactionAppliedCommandAppliedStableV2::ZkappCommand(v) => (
                    MinaTransactionTransactionStableV2::Command(Box::new(
                        MinaBaseUserCommandStableV2::ZkappCommand(v.command.data),
                    )),
                    v.command.status,
                ),
            },
            MinaTransactionLogicTransactionAppliedVaryingStableV2::FeeTransfer(v) => (
                MinaTransactionTransactionStableV2::FeeTransfer(v.fee_transfer.data),
                v.fee_transfer.status,
            ),
            MinaTransactionLogicTransactionAppliedVaryingStableV2::Coinbase(v) => (
                MinaTransactionTransactionStableV2::Coinbase(v.coinbase.data),
                v.coinbase.status,
            ),
        }
    }
}
