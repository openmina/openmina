use std::sync::Arc;

use binprot_derive::{BinProtRead, BinProtWrite};
use mina_p2p_messages::v2::{
    CurrencyFeeStableV1, MinaBaseFeeWithProverStableV1,
    MinaStateBlockchainStateValueStableV2LedgerProofStatement, MinaStateSnarkedLedgerStateStableV2,
    MinaStateSnarkedLedgerStateWithSokStableV2,
    NetworkPoolSnarkPoolDiffVersionedStableV2AddSolvedWork1, NonZeroCurvePoint,
    TransactionSnarkWorkStatementStableV2, TransactionSnarkWorkTStableV2Proofs,
};
use serde::{Deserialize, Serialize};

use crate::snark_job_id::SnarkJobId;

#[derive(BinProtRead, BinProtWrite, Serialize, Deserialize, Debug, Clone)]
pub struct Snark {
    pub prover: NonZeroCurvePoint,
    pub fee: CurrencyFeeStableV1,
    pub proofs: Arc<TransactionSnarkWorkTStableV2Proofs>,
}

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub struct SnarkInfo {
    pub job_id: SnarkJobId,
    pub fee: CurrencyFeeStableV1,
    pub prover: NonZeroCurvePoint,
}

impl Snark {
    pub fn job_id(&self) -> SnarkJobId {
        (&*self.proofs).into()
    }

    pub fn info(&self) -> SnarkInfo {
        SnarkInfo {
            job_id: self.job_id(),
            fee: self.fee.clone(),
            prover: self.prover.clone(),
        }
    }

    pub fn statement(&self) -> TransactionSnarkWorkStatementStableV2 {
        // TODO(binier): move conversion to mina-p2p-messages-rs
        fn conv_stmt(
            stmt: &MinaStateSnarkedLedgerStateWithSokStableV2,
        ) -> MinaStateSnarkedLedgerStateStableV2 {
            let v = MinaStateBlockchainStateValueStableV2LedgerProofStatement {
                source: stmt.source.clone(),
                target: stmt.target.clone(),
                connecting_ledger_left: stmt.connecting_ledger_left.clone(),
                connecting_ledger_right: stmt.connecting_ledger_right.clone(),
                supply_increase: stmt.supply_increase.clone(),
                fee_excess: stmt.fee_excess.clone(),
                sok_digest: (),
            };
            MinaStateSnarkedLedgerStateStableV2(v)
        }
        match &*self.proofs {
            TransactionSnarkWorkTStableV2Proofs::One(p) => {
                TransactionSnarkWorkStatementStableV2::One(conv_stmt(&p.0.statement))
            }
            TransactionSnarkWorkTStableV2Proofs::Two((p1, p2)) => {
                let stmt1 = conv_stmt(&p1.0.statement);
                let stmt2 = conv_stmt(&p2.0.statement);
                TransactionSnarkWorkStatementStableV2::Two((stmt1, stmt2))
            }
        }
    }
}

impl From<NetworkPoolSnarkPoolDiffVersionedStableV2AddSolvedWork1> for Snark {
    fn from(value: NetworkPoolSnarkPoolDiffVersionedStableV2AddSolvedWork1) -> Self {
        Self {
            prover: value.fee.prover,
            fee: value.fee.fee,
            proofs: value.proof.into(),
        }
    }
}

impl From<&Snark> for NetworkPoolSnarkPoolDiffVersionedStableV2AddSolvedWork1 {
    fn from(value: &Snark) -> Self {
        Self {
            proof: (*value.proofs).clone(),
            fee: MinaBaseFeeWithProverStableV1 {
                fee: value.fee.clone(),
                prover: value.prover.clone(),
            },
        }
    }
}
