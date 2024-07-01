use mina_p2p_messages::binprot::{
    self,
    macros::{BinProtRead, BinProtWrite},
};
use mina_p2p_messages::v2::NonZeroCurvePoint;
use serde::{Deserialize, Serialize};

use super::{Transaction, TransactionHash};

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub struct TransactionInfo {
    pub hash: TransactionHash,
    pub fee_payer: NonZeroCurvePoint,
    pub fee: u64,
    pub nonce: u32,
}

impl From<&Transaction> for TransactionInfo {
    fn from(tx: &Transaction) -> Self {
        match tx {
            Transaction::SignedCommand(tx) => Self {
                hash: tx.hash().unwrap(),
                fee_payer: tx.payload.common.fee_payer_pk.clone(),
                fee: tx.payload.common.fee.as_u64(),
                nonce: tx.payload.common.nonce.as_u32(),
            },
            Transaction::ZkappCommand(tx) => Self {
                hash: tx.hash().unwrap(),
                fee_payer: tx.fee_payer.body.public_key.clone(),
                fee: tx.fee_payer.body.fee.as_u64(),
                nonce: tx.fee_payer.body.nonce.as_u32(),
            },
        }
    }
}
