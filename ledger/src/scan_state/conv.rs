use mina_p2p_messages::v2::{
    CurrencyAmountStableV1, CurrencyFeeStableV1, LedgerProofProdStableV2,
    MinaBaseFeeExcessStableV1, MinaBaseFeeExcessStableV1Fee, MinaBaseLedgerHash0StableV1,
    MinaBaseSokMessageDigestStableV1, MinaBaseSokMessageStableV1,
    MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount, SgnStableV1,
    TransactionSnarkScanStateLedgerProofWithSokMessageStableV2,
    TransactionSnarkScanStateTransactionWithWitnessStableV2, TransactionSnarkStableV2,
    TransactionSnarkStatementStableV2, TransactionSnarkStatementWithSokStableV2,
    TransactionSnarkStatementWithSokStableV2Source,
    UnsignedExtendedUInt64Int64ForVersionTagsStableV1,
};

use super::{
    currency::{Amount, Fee, Sgn, Signed},
    fee_excess::FeeExcess,
    scan_state::transaction_snark::{
        LedgerProof, LedgerProofWithSokMessage, Registers, SokMessage, Statement, TransactionSnark,
        TransactionWithWitness,
    },
};

impl From<&CurrencyAmountStableV1> for Amount {
    fn from(value: &CurrencyAmountStableV1) -> Self {
        Self(value.as_u64())
    }
}

impl From<&Amount> for CurrencyAmountStableV1 {
    fn from(value: &Amount) -> Self {
        Self(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
            (value.0 as i64).into(),
        ))
    }
}

impl From<&MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount>
    for Signed<Amount>
{
    fn from(
        value: &MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount,
    ) -> Self {
        Self {
            magnitude: (&value.magnitude).into(),
            sgn: (&value.sgn.0).into(),
        }
    }
}

impl From<&Signed<Amount>>
    for MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount
{
    fn from(value: &Signed<Amount>) -> Self {
        Self {
            magnitude: (&value.magnitude).into(),
            sgn: ((&value.sgn).into(),),
        }
    }
}

impl From<&CurrencyFeeStableV1> for Fee {
    fn from(value: &CurrencyFeeStableV1) -> Self {
        Self(value.as_u64())
    }
}

impl From<&SgnStableV1> for Sgn {
    fn from(value: &SgnStableV1) -> Self {
        match value {
            SgnStableV1::Pos => Self::Pos,
            SgnStableV1::Neg => Self::Neg,
        }
    }
}

impl From<&MinaBaseFeeExcessStableV1Fee> for Signed<Fee> {
    fn from(value: &MinaBaseFeeExcessStableV1Fee) -> Self {
        Self {
            magnitude: (&value.magnitude).into(),
            sgn: (&value.sgn.0).into(),
        }
    }
}

impl From<&Sgn> for SgnStableV1 {
    fn from(value: &Sgn) -> Self {
        match value {
            Sgn::Pos => Self::Pos,
            Sgn::Neg => Self::Neg,
        }
    }
}

impl From<&Fee> for CurrencyFeeStableV1 {
    fn from(value: &Fee) -> Self {
        Self(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
            (value.0 as i64).into(),
        ))
    }
}

impl From<&Signed<Fee>> for MinaBaseFeeExcessStableV1Fee {
    fn from(value: &Signed<Fee>) -> Self {
        Self {
            magnitude: (&value.magnitude).into(),
            sgn: ((&value.sgn).into(),),
        }
    }
}

impl From<&MinaBaseFeeExcessStableV1> for FeeExcess {
    fn from(value: &MinaBaseFeeExcessStableV1) -> Self {
        Self {
            fee_token_l: (&value.fee_token_l.0).into(),
            fee_excess_l: (&value.fee_excess_l).into(),
            fee_token_r: (&value.fee_token_r.0).into(),
            fee_excess_r: (&value.fee_excess_r).into(),
        }
    }
}

impl From<&FeeExcess> for MinaBaseFeeExcessStableV1 {
    fn from(value: &FeeExcess) -> Self {
        Self {
            fee_token_l: (&value.fee_token_l).into(),
            fee_excess_l: (&value.fee_excess_l).into(),
            fee_token_r: (&value.fee_token_r).into(),
            fee_excess_r: (&value.fee_excess_r).into(),
        }
    }
}

impl From<&TransactionSnarkStatementWithSokStableV2Source> for Registers {
    fn from(value: &TransactionSnarkStatementWithSokStableV2Source) -> Self {
        Self {
            ledger: value.ledger.to_field(),
            pending_coinbase_stack: value.pending_coinbase_stack.clone(),
            local_state: value.local_state.clone(),
        }
    }
}

impl From<&TransactionSnarkStatementStableV2> for Statement {
    fn from(value: &TransactionSnarkStatementStableV2) -> Self {
        Self {
            source: (&value.source).into(),
            target: (&value.target).into(),
            supply_increase: (&value.supply_increase).into(),
            fee_excess: (&value.fee_excess).into(),
            sok_digest: None,
        }
    }
}

impl From<&TransactionSnarkStatementWithSokStableV2> for Statement {
    fn from(value: &TransactionSnarkStatementWithSokStableV2) -> Self {
        Self {
            source: (&value.source).into(),
            target: (&value.target).into(),
            supply_increase: (&value.supply_increase).into(),
            fee_excess: (&value.fee_excess).into(),
            sok_digest: Some(value.sok_digest.to_vec()),
        }
    }
}

impl From<&Statement> for TransactionSnarkStatementWithSokStableV2 {
    fn from(value: &Statement) -> Self {
        assert!(value.sok_digest.is_some());
        Self {
            source: (&value.source).into(),
            target: (&value.target).into(),
            supply_increase: (&value.supply_increase).into(),
            fee_excess: (&value.fee_excess).into(),
            sok_digest: MinaBaseSokMessageDigestStableV1(
                value.sok_digest.as_ref().unwrap().as_slice().into(),
            ),
        }
    }
}

impl From<&TransactionSnarkScanStateTransactionWithWitnessStableV2> for TransactionWithWitness {
    fn from(value: &TransactionSnarkScanStateTransactionWithWitnessStableV2) -> Self {
        Self {
            transaction_with_info: value.transaction_with_info.clone(),
            state_hash: value.state_hash.clone(),
            statement: (&value.statement).into(),
            init_stack: value.init_stack.clone(),
            ledger_witness: value.ledger_witness.clone(),
        }
    }
}

impl From<&Registers> for TransactionSnarkStatementWithSokStableV2Source {
    fn from(value: &Registers) -> Self {
        Self {
            ledger: MinaBaseLedgerHash0StableV1(value.ledger.into()).into(),
            pending_coinbase_stack: value.pending_coinbase_stack.clone(),
            local_state: value.local_state.clone(),
        }
    }
}

impl From<&Statement> for TransactionSnarkStatementStableV2 {
    fn from(value: &Statement) -> Self {
        Self {
            source: (&value.source).into(),
            target: (&value.target).into(),
            supply_increase: (&value.supply_increase).into(),
            fee_excess: (&value.fee_excess).into(),
            sok_digest: (),
        }
    }
}

impl From<&TransactionWithWitness> for TransactionSnarkScanStateTransactionWithWitnessStableV2 {
    fn from(value: &TransactionWithWitness) -> Self {
        Self {
            transaction_with_info: value.transaction_with_info.clone(),
            state_hash: value.state_hash.clone(),
            statement: (&value.statement).into(),
            init_stack: value.init_stack.clone(),
            ledger_witness: value.ledger_witness.clone(),
        }
    }
}

impl binprot::BinProtWrite for TransactionWithWitness {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let p2p: TransactionSnarkScanStateTransactionWithWitnessStableV2 = self.into();
        p2p.binprot_write(w)
    }
}

impl From<&TransactionSnarkStableV2> for TransactionSnark {
    fn from(value: &TransactionSnarkStableV2) -> Self {
        Self {
            statement: (&value.statement).into(),
            proof: value.proof.clone(),
        }
    }
}

impl From<&TransactionSnark> for TransactionSnarkStableV2 {
    fn from(value: &TransactionSnark) -> Self {
        Self {
            statement: (&value.statement).into(),
            proof: value.proof.clone(),
        }
    }
}

impl From<&LedgerProofProdStableV2> for LedgerProof {
    fn from(value: &LedgerProofProdStableV2) -> Self {
        Self((&value.0).into())
    }
}

impl From<&LedgerProof> for LedgerProofProdStableV2 {
    fn from(value: &LedgerProof) -> Self {
        Self((&value.0).into())
    }
}

impl binprot::BinProtWrite for LedgerProof {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let p2p: LedgerProofProdStableV2 = self.into();
        p2p.binprot_write(w)
    }
}

impl From<&MinaBaseSokMessageStableV1> for SokMessage {
    fn from(value: &MinaBaseSokMessageStableV1) -> Self {
        Self {
            fee: (&value.fee).into(),
            prover: (&value.prover).into(),
        }
    }
}

impl From<&SokMessage> for MinaBaseSokMessageStableV1 {
    fn from(value: &SokMessage) -> Self {
        Self {
            fee: (&value.fee).into(),
            prover: (&value.prover).into(),
        }
    }
}

impl From<&LedgerProofWithSokMessage>
    for TransactionSnarkScanStateLedgerProofWithSokMessageStableV2
{
    fn from(value: &LedgerProofWithSokMessage) -> Self {
        Self((&value.proof).into(), (&value.sok_message).into())
    }
}

impl From<&TransactionSnarkScanStateLedgerProofWithSokMessageStableV2>
    for LedgerProofWithSokMessage
{
    fn from(value: &TransactionSnarkScanStateLedgerProofWithSokMessageStableV2) -> Self {
        Self {
            proof: (&value.0).into(),
            sok_message: (&value.1).into(),
        }
    }
}

impl binprot::BinProtWrite for LedgerProofWithSokMessage {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let p2p: TransactionSnarkScanStateLedgerProofWithSokMessageStableV2 = self.into();
        p2p.binprot_write(w)
    }
}
