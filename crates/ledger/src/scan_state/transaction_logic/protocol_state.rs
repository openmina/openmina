use crate::{
    proofs::field::FieldWitness,
    scan_state::currency::{Amount, Length, Signed, Slot},
    sparse_ledger::LedgerIntf,
};
use mina_curves::pasta::Fp;
use mina_p2p_messages::{
    bigint::InvalidBigInt,
    v2::{self, MinaStateProtocolStateValueStableV2},
};

#[derive(Debug, Clone)]
pub struct EpochLedger<F: FieldWitness> {
    pub hash: F,
    pub total_currency: Amount,
}

#[derive(Debug, Clone)]
pub struct EpochData<F: FieldWitness> {
    pub ledger: EpochLedger<F>,
    pub seed: F,
    pub start_checkpoint: F,
    pub lock_checkpoint: F,
    pub epoch_length: Length,
}

#[derive(Debug, Clone)]
pub struct ProtocolStateView {
    pub snarked_ledger_hash: Fp,
    pub blockchain_length: Length,
    pub min_window_density: Length,
    pub total_currency: Amount,
    pub global_slot_since_genesis: Slot,
    pub staking_epoch_data: EpochData<Fp>,
    pub next_epoch_data: EpochData<Fp>,
}

/// <https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/mina_state/protocol_state.ml#L180>
pub fn protocol_state_view(
    state: &MinaStateProtocolStateValueStableV2,
) -> Result<ProtocolStateView, InvalidBigInt> {
    let MinaStateProtocolStateValueStableV2 {
        previous_state_hash: _,
        body,
    } = state;

    protocol_state_body_view(body)
}

pub fn protocol_state_body_view(
    body: &v2::MinaStateProtocolStateBodyValueStableV2,
) -> Result<ProtocolStateView, InvalidBigInt> {
    let cs = &body.consensus_state;
    let sed = &cs.staking_epoch_data;
    let ned = &cs.next_epoch_data;

    Ok(ProtocolStateView {
        // <https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/mina_state/blockchain_state.ml#L58>
        //
        snarked_ledger_hash: body
            .blockchain_state
            .ledger_proof_statement
            .target
            .first_pass_ledger
            .to_field()?,
        blockchain_length: Length(cs.blockchain_length.as_u32()),
        min_window_density: Length(cs.min_window_density.as_u32()),
        total_currency: Amount(cs.total_currency.as_u64()),
        global_slot_since_genesis: (&cs.global_slot_since_genesis).into(),
        staking_epoch_data: EpochData {
            ledger: EpochLedger {
                hash: sed.ledger.hash.to_field()?,
                total_currency: Amount(sed.ledger.total_currency.as_u64()),
            },
            seed: sed.seed.to_field()?,
            start_checkpoint: sed.start_checkpoint.to_field()?,
            lock_checkpoint: sed.lock_checkpoint.to_field()?,
            epoch_length: Length(sed.epoch_length.as_u32()),
        },
        next_epoch_data: EpochData {
            ledger: EpochLedger {
                hash: ned.ledger.hash.to_field()?,
                total_currency: Amount(ned.ledger.total_currency.as_u64()),
            },
            seed: ned.seed.to_field()?,
            start_checkpoint: ned.start_checkpoint.to_field()?,
            lock_checkpoint: ned.lock_checkpoint.to_field()?,
            epoch_length: Length(ned.epoch_length.as_u32()),
        },
    })
}

pub type GlobalState<L> = GlobalStateSkeleton<L, Signed<Amount>, Slot>;

#[derive(Debug, Clone)]
pub struct GlobalStateSkeleton<L, SignedAmount, Slot> {
    pub first_pass_ledger: L,
    pub second_pass_ledger: L,
    pub fee_excess: SignedAmount,
    pub supply_increase: SignedAmount,
    pub protocol_state: ProtocolStateView,
    /// Slot of block when the transaction is applied.
    /// NOTE: This is at least 1 slot after the protocol_state's view,
    /// which is for the *previous* slot.
    pub block_global_slot: Slot,
}

impl<L: LedgerIntf + Clone> GlobalState<L> {
    pub fn first_pass_ledger(&self) -> L {
        self.first_pass_ledger.create_masked()
    }

    #[must_use]
    pub fn set_first_pass_ledger(&self, should_update: bool, ledger: L) -> Self {
        let mut this = self.clone();
        if should_update {
            this.first_pass_ledger.apply_mask(ledger);
        }
        this
    }

    pub fn second_pass_ledger(&self) -> L {
        self.second_pass_ledger.create_masked()
    }

    #[must_use]
    pub fn set_second_pass_ledger(&self, should_update: bool, ledger: L) -> Self {
        let mut this = self.clone();
        if should_update {
            this.second_pass_ledger.apply_mask(ledger);
        }
        this
    }

    pub fn fee_excess(&self) -> Signed<Amount> {
        self.fee_excess
    }

    #[must_use]
    pub fn set_fee_excess(&self, fee_excess: Signed<Amount>) -> Self {
        let mut this = self.clone();
        this.fee_excess = fee_excess;
        this
    }

    pub fn supply_increase(&self) -> Signed<Amount> {
        self.supply_increase
    }

    #[must_use]
    pub fn set_supply_increase(&self, supply_increase: Signed<Amount>) -> Self {
        let mut this = self.clone();
        this.supply_increase = supply_increase;
        this
    }

    pub fn block_global_slot(&self) -> Slot {
        self.block_global_slot
    }
}
