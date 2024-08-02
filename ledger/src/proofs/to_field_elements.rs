use std::borrow::Cow;

use crate::proofs::public_input::plonk_checks::ShiftingValue;
use ark_ff::{Field, One, Zero};
use kimchi::proof::{ProofEvaluations, ProverCommitments, ProverProof};
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use mina_p2p_messages::{string::ByteString, v2};
use mina_signer::CompressedPubKey;
use poly_commitment::evaluation_proof::OpeningProof;

use crate::{
    proofs::{
        public_input::prepared_statement::{DeferredValues, Plonk, ProofState},
        step::OptFlag,
        util::u64_to_field,
    },
    scan_state::{
        currency::{self, Sgn},
        fee_excess::FeeExcess,
        pending_coinbase,
        scan_state::transaction_snark::{Registers, SokDigest, Statement},
        transaction_logic::{
            protocol_state::{EpochData, EpochLedger},
            transaction_union_payload,
            zkapp_statement::ZkappStatement,
        },
    },
    staged_ledger::hash::StagedLedgerHash,
    Account, MyCow, ReceiptChainHash, TimingAsRecord, TokenId, TokenSymbol, VotingFor,
};

use super::{
    field::{Boolean, CircuitVar, FieldWitness, GroupAffine},
    numbers::currency::{CheckedCurrency, CheckedSigned},
    step::PerProofWitness,
    transaction::{
        field_to_bits, InnerCurve, PlonkVerificationKeyEvals, StepMainProofState, StepMainStatement,
    },
    unfinalized::{AllEvals, EvalsWithPublicInput},
};

pub trait ToFieldElementsDebug: ToFieldElements<Fp> + std::fmt::Debug {}

impl<T: ToFieldElements<Fp> + std::fmt::Debug> ToFieldElementsDebug for T {}

pub trait ToFieldElements<F: Field> {
    fn to_field_elements(&self, fields: &mut Vec<F>);

    fn to_field_elements_owned(&self) -> Vec<F> {
        let mut fields = Vec::with_capacity(1024);
        self.to_field_elements(&mut fields);
        fields
    }
}

impl ToFieldElements<Fp> for ZkappStatement {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        fields.extend(self.to_field_elements())
    }
}

// impl ToFieldElements for Statement<SokDigest> {
//     fn to_field_elements(&self) -> Vec<Fp> {
//         let mut inputs = crate::Inputs::new();
//         inputs.append(self);

//         inputs.to_fields()
//     }
// }

impl<F: Field> ToFieldElements<F> for () {
    fn to_field_elements(&self, _fields: &mut Vec<F>) {}
}

impl ToFieldElements<Fp> for SokDigest {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        const BITS: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];
        for byte in &self.0 {
            fields.extend(BITS.iter().map(|bit| Fp::from((byte & bit != 0) as u64)));
        }
    }
}

/// Unlike expectations, OCaml doesn't call `Sok_digest.to_field_elements` on
/// `Statement_intf.to_field_elements`, it is probably overwritten somewhere
/// but I was not able to find which method exactly is used:
/// I added lots of `printf` everywhere but they are never called/triggered.
/// I suspect it uses the `to_hlist`, or the `Typ`, or the data spec, but
/// again, I couldn't confirm.
///
/// This implementation relies only on the output I observed here, using
/// reproducible input test data:
/// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/pickles/composition_types/composition_types.ml#L714C11-L714C48
///
/// TODO: Fuzz this method, compare with OCaml
impl<T: ToFieldElements<Fp>> ToFieldElements<Fp> for Statement<T> {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self {
            source,
            target,
            connecting_ledger_left,
            connecting_ledger_right,
            supply_increase,
            fee_excess,
            sok_digest,
        } = self;

        let sign_to_field = |sgn| -> Fp {
            use crate::scan_state::currency::Sgn::*;
            match sgn {
                Pos => 1i64,
                Neg => -1,
            }
            .into()
        };

        let mut add_register = |registers: &Registers| {
            let Registers {
                first_pass_ledger,
                second_pass_ledger,
                pending_coinbase_stack,
                local_state,
            } = registers;

            first_pass_ledger.to_field_elements(fields);
            second_pass_ledger.to_field_elements(fields);
            pending_coinbase_stack.to_field_elements(fields);
            local_state.stack_frame.to_field_elements(fields);
            local_state.call_stack.to_field_elements(fields);
            local_state.transaction_commitment.to_field_elements(fields);
            local_state
                .full_transaction_commitment
                .to_field_elements(fields);
            local_state.excess.magnitude.to_field_elements(fields);
            sign_to_field(local_state.excess.sgn).to_field_elements(fields);
            local_state
                .supply_increase
                .magnitude
                .to_field_elements(fields);
            sign_to_field(local_state.supply_increase.sgn).to_field_elements(fields);
            local_state.ledger.to_field_elements(fields);
            local_state.success.to_field_elements(fields);
            local_state.account_update_index.to_field_elements(fields);
            local_state.will_succeed.to_field_elements(fields);
        };

        add_register(source);
        add_register(target);
        connecting_ledger_left.to_field_elements(fields);
        connecting_ledger_right.to_field_elements(fields);
        supply_increase.magnitude.to_field_elements(fields);
        sign_to_field(supply_increase.sgn).to_field_elements(fields);

        let FeeExcess {
            fee_token_l,
            fee_excess_l,
            fee_token_r,
            fee_excess_r,
        } = fee_excess;

        fee_token_l.to_field_elements(fields);
        fee_excess_l.magnitude.to_field_elements(fields);
        sign_to_field(fee_excess_l.sgn).to_field_elements(fields);

        fee_token_r.to_field_elements(fields);
        fee_excess_r.magnitude.to_field_elements(fields);
        sign_to_field(fee_excess_r.sgn).to_field_elements(fields);

        sok_digest.to_field_elements(fields)
    }
}

impl<F: FieldWitness> ToFieldElements<F>
    for v2::MinaStateBlockchainStateValueStableV2LedgerProofStatement
{
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self {
            source,
            target,
            connecting_ledger_left,
            connecting_ledger_right,
            supply_increase,
            fee_excess,
            sok_digest,
        } = self;

        let sign_to_field = |sgn: &v2::SgnStableV1| -> F {
            match sgn {
                v2::SgnStableV1::Pos => 1i64,
                v2::SgnStableV1::Neg => -1,
            }
            .into()
        };

        let mut add_register =
            |registers: &v2::MinaStateBlockchainStateValueStableV2LedgerProofStatementSource| {
                let v2::MinaStateBlockchainStateValueStableV2LedgerProofStatementSource {
                    first_pass_ledger,
                    second_pass_ledger,
                    pending_coinbase_stack,
                    local_state,
                } = registers;

                fields.push(first_pass_ledger.to_field());
                fields.push(second_pass_ledger.to_field());
                fields.push(pending_coinbase_stack.data.0.to_field());
                fields.push(pending_coinbase_stack.state.init.to_field());
                fields.push(pending_coinbase_stack.state.curr.to_field());
                fields.push(local_state.stack_frame.to_field());
                fields.push(local_state.call_stack.to_field());
                fields.push(local_state.transaction_commitment.to_field());
                fields.push(local_state.full_transaction_commitment.to_field());
                fields.push(local_state.excess.magnitude.as_u64().into());
                fields.push(sign_to_field(&local_state.excess.sgn));
                fields.push(local_state.supply_increase.magnitude.as_u64().into());
                fields.push(sign_to_field(&local_state.supply_increase.sgn));
                fields.push(local_state.ledger.to_field());
                fields.push((local_state.success as u64).into());
                fields.push(local_state.account_update_index.as_u32().into());
                fields.push((local_state.will_succeed as u64).into());
            };

        add_register(source);
        add_register(target);
        fields.push(connecting_ledger_left.to_field());
        fields.push(connecting_ledger_right.to_field());
        fields.push(supply_increase.magnitude.as_u64().into());
        fields.push(sign_to_field(&supply_increase.sgn));

        let v2::MinaBaseFeeExcessStableV1(
            v2::TokenFeeExcess {
                token: fee_token_l,
                amount: fee_excess_l,
            },
            v2::TokenFeeExcess {
                token: fee_token_r,
                amount: fee_excess_r,
            },
        ) = fee_excess;

        fields.push(fee_token_l.to_field());
        fields.push(fee_excess_l.magnitude.as_u64().into());
        fields.push(sign_to_field(&fee_excess_l.sgn));

        fields.push(fee_token_r.to_field());
        fields.push(fee_excess_r.magnitude.as_u64().into());
        fields.push(sign_to_field(&fee_excess_r.sgn));

        sok_digest.to_field_elements(fields)
    }
}

impl<F: FieldWitness, T: ToFieldElements<F>> ToFieldElements<F> for Vec<T> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        self.iter().for_each(|v| v.to_field_elements(fields));
    }
}

impl<F: FieldWitness, T: ToFieldElements<F>> ToFieldElements<F> for Box<[T]> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        self.iter().for_each(|v| v.to_field_elements(fields));
    }
}

impl<F: FieldWitness> ToFieldElements<F> for Fp {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        use crate::proofs::field::IntoGeneric;
        fields.push(self.into_gen());
    }
}

impl<F: FieldWitness, T: ToFieldElements<F> + Clone> ToFieldElements<F> for Cow<'_, T> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let this: &T = self.as_ref();
        this.to_field_elements(fields)
    }
}

// pack
pub fn field_of_bits<F: FieldWitness, const N: usize>(bs: &[bool; N]) -> F {
    bs.iter().rev().fold(F::zero(), |acc, b| {
        let acc = acc + acc;
        if *b {
            acc + F::one()
        } else {
            acc
        }
    })
}

impl<F: FieldWitness> ToFieldElements<F> for Fq {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        use crate::proofs::field::IntoGeneric;
        use std::any::TypeId;

        // TODO: Refactor when specialization is stable
        if TypeId::of::<F>() == TypeId::of::<Fq>() {
            fields.push(self.into_gen());
        } else {
            // `Fq` is larger than `Fp` so we have to split the field (low & high bits)
            // See:
            // https://github.com/MinaProtocol/mina/blob/e85cf6969e42060f69d305fb63df9b8d7215d3d7/src/lib/pickles/impls.ml#L94C1-L105C45

            let to_high_low = |fq: Fq| {
                let [low, high @ ..] = field_to_bits::<Fq, 255>(fq);
                [field_of_bits(&high), F::from(low)]
            };
            fields.extend(to_high_low(*self));
        }
    }
}

impl<F: FieldWitness, T: ToFieldElements<F>, const N: usize> ToFieldElements<F> for [T; N] {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        self.iter().for_each(|v| v.to_field_elements(fields));
    }
}

impl<F: FieldWitness> ToFieldElements<F> for StagedLedgerHash<F> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self {
            non_snark,
            pending_coinbase_hash,
        } = self;

        let non_snark_digest = non_snark.digest();

        const BITS: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];
        fields.extend(
            non_snark_digest
                .iter()
                .flat_map(|byte| BITS.iter().map(|bit| F::from((*byte & bit != 0) as u64))),
        );

        pending_coinbase_hash.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for ByteString {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let slice: &[u8] = self;
        slice.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for GroupAffine<F> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self {
            x, y, infinity: _, ..
        } = self;
        y.to_field_elements(fields);
        x.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for &'_ [u8] {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        const BITS: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];
        fields.extend(
            self.iter()
                .flat_map(|byte| BITS.iter().map(|bit| F::from((*byte & bit != 0) as u64))),
        );
    }
}

impl<F: FieldWitness> ToFieldElements<F> for &'_ [bool] {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        fields.reserve(self.len());
        fields.extend(self.iter().copied().map(F::from))
    }
}

impl<F: FieldWitness> ToFieldElements<F> for bool {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        F::from(*self).to_field_elements(fields)
    }
}

impl<F: FieldWitness> ToFieldElements<F> for u64 {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        F::from(*self).to_field_elements(fields)
    }
}

impl<F: FieldWitness> ToFieldElements<F> for u32 {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        F::from(*self).to_field_elements(fields)
    }
}

impl<F: FieldWitness> ToFieldElements<F> for ProofEvaluations<[F; 2]> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self {
            w,
            z,
            s,
            coefficients,
            generic_selector,
            poseidon_selector,
            complete_add_selector,
            mul_selector,
            emul_selector,
            endomul_scalar_selector,
            range_check0_selector,
            range_check1_selector,
            foreign_field_add_selector,
            foreign_field_mul_selector,
            xor_selector,
            rot_selector,
            lookup_aggregation,
            lookup_table,
            lookup_sorted,
            runtime_lookup_table,
            runtime_lookup_table_selector,
            xor_lookup_selector,
            lookup_gate_lookup_selector,
            range_check_lookup_selector,
            foreign_field_mul_lookup_selector,
        } = self;

        let mut push = |[a, b]: &[F; 2]| {
            a.to_field_elements(fields);
            b.to_field_elements(fields);
        };

        w.iter().for_each(&mut push);
        coefficients.iter().for_each(&mut push);
        push(z);
        s.iter().for_each(&mut push);
        push(generic_selector);
        push(poseidon_selector);
        push(complete_add_selector);
        push(mul_selector);
        push(emul_selector);
        push(endomul_scalar_selector);
        range_check0_selector.as_ref().map(&mut push);
        range_check1_selector.as_ref().map(&mut push);
        foreign_field_add_selector.as_ref().map(&mut push);
        foreign_field_mul_selector.as_ref().map(&mut push);
        xor_selector.as_ref().map(&mut push);
        rot_selector.as_ref().map(&mut push);
        lookup_aggregation.as_ref().map(&mut push);
        lookup_table.as_ref().map(&mut push);
        lookup_sorted.iter().for_each(|v| {
            v.as_ref().map(&mut push);
        });
        runtime_lookup_table.as_ref().map(&mut push);
        runtime_lookup_table_selector.as_ref().map(&mut push);
        xor_lookup_selector.as_ref().map(&mut push);
        lookup_gate_lookup_selector.as_ref().map(&mut push);
        range_check_lookup_selector.as_ref().map(&mut push);
        foreign_field_mul_lookup_selector.as_ref().map(&mut push);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for AllEvals<F> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self {
            ft_eval1,
            evals:
                EvalsWithPublicInput {
                    evals,
                    public_input,
                },
        } = self;

        public_input.to_field_elements(fields);
        evals.to_field_elements(fields);
        ft_eval1.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for &[AllEvals<F>] {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        self.iter().for_each(|e| e.to_field_elements(fields))
    }
}

impl<F: FieldWitness> ToFieldElements<F> for EpochData<F> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self {
            ledger:
                EpochLedger {
                    hash,
                    total_currency,
                },
            seed,
            start_checkpoint,
            lock_checkpoint,
            epoch_length,
        } = self;

        hash.to_field_elements(fields);
        total_currency.to_field_elements(fields);
        seed.to_field_elements(fields);
        start_checkpoint.to_field_elements(fields);
        lock_checkpoint.to_field_elements(fields);
        epoch_length.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for v2::NonZeroCurvePoint {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let v2::NonZeroCurvePointUncompressedStableV1 { x, is_odd } = self.inner();

        x.to_field::<F>().to_field_elements(fields);
        is_odd.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F>
    for v2::ConsensusProofOfStakeDataConsensusStateValueStableV2
{
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let v2::ConsensusProofOfStakeDataConsensusStateValueStableV2 {
            blockchain_length,
            epoch_count,
            min_window_density,
            sub_window_densities,
            last_vrf_output,
            total_currency,
            curr_global_slot_since_hard_fork:
                v2::ConsensusGlobalSlotStableV1 {
                    slot_number,
                    slots_per_epoch,
                },
            global_slot_since_genesis,
            staking_epoch_data,
            next_epoch_data,
            has_ancestor_in_same_checkpoint_window,
            block_stake_winner,
            block_creator,
            coinbase_receiver,
            supercharge_coinbase,
        } = self;

        let staking_epoch_data: EpochData<F> = staking_epoch_data.into();
        let next_epoch_data: EpochData<F> = next_epoch_data.into();

        blockchain_length.as_u32().to_field_elements(fields);
        epoch_count.as_u32().to_field_elements(fields);
        min_window_density.as_u32().to_field_elements(fields);
        fields.extend(sub_window_densities.iter().map(|w| F::from(w.as_u32())));

        {
            let vrf: &[u8] = last_vrf_output.as_ref();
            (&vrf[..31]).to_field_elements(fields);
            // Ignore the last 3 bits
            let last_byte = vrf[31];
            for bit in [1, 2, 4, 8, 16] {
                F::from(last_byte & bit != 0).to_field_elements(fields);
            }
        }

        total_currency.as_u64().to_field_elements(fields);
        slot_number.as_u32().to_field_elements(fields);
        slots_per_epoch.as_u32().to_field_elements(fields);
        global_slot_since_genesis.as_u32().to_field_elements(fields);
        staking_epoch_data.to_field_elements(fields);
        next_epoch_data.to_field_elements(fields);
        has_ancestor_in_same_checkpoint_window.to_field_elements(fields);
        block_stake_winner.to_field_elements(fields);
        block_creator.to_field_elements(fields);
        coinbase_receiver.to_field_elements(fields);
        supercharge_coinbase.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for v2::MinaBaseProtocolConstantsCheckedValueStableV1 {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self {
            k,
            slots_per_epoch,
            slots_per_sub_window,
            grace_period_slots,
            delta,
            genesis_state_timestamp,
        } = self;

        k.as_u32().to_field_elements(fields);
        slots_per_epoch.as_u32().to_field_elements(fields);
        slots_per_sub_window.as_u32().to_field_elements(fields);
        grace_period_slots.as_u32().to_field_elements(fields);
        delta.as_u32().to_field_elements(fields);
        genesis_state_timestamp.as_u64().to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for v2::MinaStateBlockchainStateValueStableV2 {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self {
            staged_ledger_hash,
            genesis_ledger_hash,
            ledger_proof_statement,
            timestamp,
            body_reference,
        } = self;

        let staged_ledger_hash: StagedLedgerHash<F> = staged_ledger_hash.into();

        staged_ledger_hash.to_field_elements(fields);
        genesis_ledger_hash
            .inner()
            .to_field::<F>()
            .to_field_elements(fields);
        ledger_proof_statement.to_field_elements(fields);
        timestamp.as_u64().to_field_elements(fields);
        body_reference.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for v2::MinaStateProtocolStateBodyValueStableV2 {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let v2::MinaStateProtocolStateBodyValueStableV2 {
            genesis_state_hash,
            blockchain_state,
            consensus_state,
            constants,
        } = self;

        genesis_state_hash
            .inner()
            .to_field::<F>()
            .to_field_elements(fields);
        blockchain_state.to_field_elements(fields);
        consensus_state.to_field_elements(fields);
        constants.to_field_elements(fields);
    }
}

impl ToFieldElements<Fp> for TokenId {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self(token_id) = self;
        token_id.to_field_elements(fields);
    }
}

impl ToFieldElements<Fp> for CompressedPubKey {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self { x, is_odd } = self;
        x.to_field_elements(fields);
        is_odd.to_field_elements(fields);
    }
}

impl ToFieldElements<Fp> for mina_signer::Signature {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self { rx, s } = self;

        rx.to_field_elements(fields);
        let s_bits = field_to_bits::<_, 255>(*s);
        s_bits.to_field_elements(fields);
    }
}

impl ToFieldElements<Fp> for mina_signer::PubKey {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let GroupAffine::<Fp> { x, y, .. } = self.point();
        x.to_field_elements(fields);
        y.to_field_elements(fields);
    }
}

impl ToFieldElements<Fp> for transaction_union_payload::TransactionUnion {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        use transaction_union_payload::{Body, Common, TransactionUnionPayload};

        let Self {
            payload:
                TransactionUnionPayload {
                    common:
                        Common {
                            fee,
                            fee_token,
                            fee_payer_pk,
                            nonce,
                            valid_until,
                            memo,
                        },
                    body:
                        Body {
                            tag,
                            source_pk,
                            receiver_pk,
                            token_id,
                            amount,
                        },
                },
            signer,
            signature,
        } = self;

        fee.to_field_elements(fields);
        fee_token.to_field_elements(fields);
        fee_payer_pk.to_field_elements(fields);
        nonce.to_field_elements(fields);
        valid_until.to_field_elements(fields);
        memo.as_slice().to_field_elements(fields);
        tag.to_untagged_bits().to_field_elements(fields);
        source_pk.to_field_elements(fields);
        receiver_pk.to_field_elements(fields);
        token_id.to_field_elements(fields);
        amount.to_field_elements(fields);
        signer.to_field_elements(fields);
        signature.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for v2::MinaNumbersGlobalSlotSinceGenesisMStableV1 {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        self.as_u32().to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for v2::MinaBasePendingCoinbaseStackVersionedStableV1 {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self {
            data,
            state: v2::MinaBasePendingCoinbaseStateStackStableV1 { init, curr },
        } = self;

        data.to_field::<F>().to_field_elements(fields);
        init.to_field::<F>().to_field_elements(fields);
        curr.to_field::<F>().to_field_elements(fields);
    }
}

impl ToFieldElements<Fp> for pending_coinbase::StateStack {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self { init, curr } = self;
        init.to_field_elements(fields);
        curr.to_field_elements(fields);
    }
}

impl ToFieldElements<Fp> for pending_coinbase::Stack {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self {
            data: pending_coinbase::CoinbaseStack(data),
            state,
        } = self;

        data.to_field_elements(fields);
        state.to_field_elements(fields);
    }
}

impl ToFieldElements<Fp> for TokenSymbol {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let field: Fp = self.to_field();
        field.to_field_elements(fields);
    }
}

// TODO: De-deduplicate with ToInputs
impl<F: FieldWitness> ToFieldElements<F> for crate::Timing {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let TimingAsRecord {
            is_timed,
            initial_minimum_balance,
            cliff_time,
            cliff_amount,
            vesting_period,
            vesting_increment,
        } = self.to_record();

        F::from(is_timed).to_field_elements(fields);
        F::from(initial_minimum_balance.as_u64()).to_field_elements(fields);
        F::from(cliff_time.as_u32()).to_field_elements(fields);
        F::from(cliff_amount.as_u64()).to_field_elements(fields);
        F::from(vesting_period.as_u32()).to_field_elements(fields);
        F::from(vesting_increment.as_u64()).to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for crate::Permissions<crate::AuthRequired> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        use crate::AuthOrVersion;

        self.iter_as_bits(|bit| match bit {
            AuthOrVersion::Auth(bit) => bit.to_field_elements(fields),
            AuthOrVersion::Version(version) => version.to_field_elements(fields),
        });
    }
}

impl<F: FieldWitness> ToFieldElements<F> for crate::AuthRequired {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        // In OCaml `Controller.if_`
        // push values in reverse order (because of OCaml evaluation order)
        // https://github.com/MinaProtocol/mina/blob/4283d70c8c5c1bd9eebb0d3e449c36fb0bf0c9af/src/lib/mina_base/permissions.ml#L174
        let crate::AuthRequiredEncoded {
            constant,
            signature_necessary,
            signature_sufficient,
        } = self.encode();

        [signature_sufficient, signature_necessary, constant].to_field_elements(fields);
    }
}

impl ToFieldElements<Fp> for Box<Account> {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Account {
            public_key,
            token_id: TokenId(token_id),
            token_symbol,
            balance,
            nonce,
            receipt_chain_hash: ReceiptChainHash(receipt_chain_hash),
            delegate,
            voting_for: VotingFor(voting_for),
            timing,
            permissions,
            zkapp,
        } = &**self;

        // Important: Any changes here probably needs the same changes in `AccountUnhashed`
        public_key.to_field_elements(fields);
        token_id.to_field_elements(fields);
        token_symbol.to_field_elements(fields);
        balance.to_field_elements(fields);
        nonce.to_field_elements(fields);
        receipt_chain_hash.to_field_elements(fields);
        let delegate = MyCow::borrow_or_else(delegate, CompressedPubKey::empty);
        delegate.to_field_elements(fields);
        voting_for.to_field_elements(fields);
        timing.to_field_elements(fields);
        permissions.to_field_elements(fields);
        MyCow::borrow_or_default(zkapp)
            .hash()
            .to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for crate::MerklePath {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        self.hash().to_field_elements(fields);
    }
}

impl<F: FieldWitness, A: ToFieldElements<F>, B: ToFieldElements<F>> ToFieldElements<F> for (A, B) {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let (a, b) = self;
        a.to_field_elements(fields);
        b.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for ReceiptChainHash {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self(receipt_chain_hash) = self;
        receipt_chain_hash.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for Sgn {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let field: F = self.to_field();
        field.to_field_elements(fields)
    }
}

impl<F: FieldWitness, T: currency::Magnitude + ToFieldElements<F>> ToFieldElements<F>
    for currency::Signed<T>
{
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self { magnitude, sgn } = self;

        magnitude.to_field_elements(fields);
        sgn.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for PlonkVerificationKeyEvals<F> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self {
            sigma,
            coefficients,
            generic,
            psm,
            complete_add,
            mul,
            emul,
            endomul_scalar,
        } = self;

        sigma.iter().for_each(|s| s.to_field_elements(fields));
        coefficients
            .iter()
            .for_each(|c| c.to_field_elements(fields));
        generic.to_field_elements(fields);
        psm.to_field_elements(fields);
        complete_add.to_field_elements(fields);
        mul.to_field_elements(fields);
        emul.to_field_elements(fields);
        endomul_scalar.to_field_elements(fields);
    }
}

impl<F: FieldWitness, const N: usize> ToFieldElements<F> for crate::address::raw::Address<N> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let zero = F::zero();
        let one = F::one();

        fields.extend(
            self.iter()
                .map(|b| match b {
                    crate::Direction::Left => zero,
                    crate::Direction::Right => one,
                })
                .rev(),
        );
    }
}

// Implementation for references
impl<F: FieldWitness, T: ToFieldElements<F>> ToFieldElements<F> for &T {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        (*self).to_field_elements(fields);
    }
}

impl<F: FieldWitness, T: CheckedCurrency<F>> ToFieldElements<F> for CheckedSigned<F, T> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        self.sgn.to_field_elements(fields);
        self.magnitude.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for InnerCurve<F> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let GroupAffine::<F> { x, y, .. } = self.to_affine();
        x.to_field_elements(fields);
        y.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for Boolean {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        self.to_field::<F>().to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for CircuitVar<Boolean> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        self.as_boolean().to_field_elements(fields);
    }
}

impl ToFieldElements<Fp> for StepMainStatement {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self {
            proof_state:
                StepMainProofState {
                    unfinalized_proofs,
                    messages_for_next_step_proof,
                },
            messages_for_next_wrap_proof,
        } = self;

        unfinalized_proofs.to_field_elements(fields);
        messages_for_next_step_proof.to_field_elements(fields);
        messages_for_next_wrap_proof.to_field_elements(fields);
    }
}

impl ToFieldElements<Fp> for PerProofWitness {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self {
            app_state,
            wrap_proof,
            proof_state,
            prev_proof_evals,
            prev_challenges,
            prev_challenge_polynomial_commitments,
            hack_feature_flags,
        } = self;

        assert!(app_state.is_none());

        let push_affine = |g: GroupAffine<Fp>, fields: &mut Vec<Fp>| {
            let GroupAffine::<Fp> { x, y, .. } = g;
            x.to_field_elements(fields);
            y.to_field_elements(fields);
        };

        let push_affines = |slice: &[GroupAffine<Fp>], fields: &mut Vec<Fp>| {
            slice.iter().copied().for_each(|g| push_affine(g, fields))
        };

        let ProverProof {
            commitments:
                ProverCommitments {
                    w_comm,
                    z_comm,
                    t_comm,
                    lookup: _,
                },
            proof:
                OpeningProof {
                    lr,
                    delta,
                    z1,
                    z2,
                    sg,
                },
            evals: _,
            ft_eval1: _,
            prev_challenges: _,
        } = wrap_proof;

        for w in w_comm {
            push_affines(&w.unshifted, fields);
        }

        push_affines(&z_comm.unshifted, fields);
        push_affines(&t_comm.unshifted, fields);

        for (a, b) in lr {
            push_affine(*a, fields);
            push_affine(*b, fields);
        }

        let shift = |f: Fq| <Fq as FieldWitness>::Shifting::of_field(f);

        shift(*z1).to_field_elements(fields);
        shift(*z2).to_field_elements(fields);

        push_affines(&[*delta, *sg], fields);

        let ProofState {
            deferred_values:
                DeferredValues {
                    plonk:
                        Plonk {
                            alpha,
                            beta,
                            gamma,
                            zeta,
                            zeta_to_srs_length,
                            zeta_to_domain_size,
                            perm,
                            lookup: _,
                            feature_flags: _,
                        },
                    combined_inner_product,
                    b,
                    xi,
                    bulletproof_challenges,
                    branch_data,
                },
            sponge_digest_before_evaluations,
            messages_for_next_wrap_proof: _,
        } = proof_state;

        u64_to_field::<Fp, 2>(alpha).to_field_elements(fields);
        u64_to_field::<Fp, 2>(beta).to_field_elements(fields);
        u64_to_field::<Fp, 2>(gamma).to_field_elements(fields);
        u64_to_field::<Fp, 2>(zeta).to_field_elements(fields);

        zeta_to_srs_length.to_field_elements(fields);
        zeta_to_domain_size.to_field_elements(fields);
        perm.to_field_elements(fields);
        match hack_feature_flags {
            OptFlag::Maybe => {
                // This block is used only when proving zkapps using proof authorization.
                // https://github.com/MinaProtocol/mina/blob/126d4d2e3495d03adc8f9597113d58a7e8fbcfd0/src/lib/pickles/composition_types/composition_types.ml#L150-L155
                // https://github.com/MinaProtocol/mina/blob/126d4d2e3495d03adc8f9597113d58a7e8fbcfd0/src/lib/pickles/per_proof_witness.ml#L149
                // https://github.com/MinaProtocol/mina/blob/a51f09d09e6ae83362ea74eaca072c8e40d08b52/src/lib/pickles_types/plonk_types.ml#L104-L119
                // https://github.com/MinaProtocol/mina/blob/a51f09d09e6ae83362ea74eaca072c8e40d08b52/src/lib/pickles_types/plonk_types.ml#L253-L303

                // the first 8 elements are the `Plonk_types.Features.typ`
                // The last 2 elements are the `Plonk_types.Opt.typ`
                // So far I've only seen proofs without feature flags.
                // TODO: Are feature flags ever used in the server node ? Or they are only used in browser/client ?
                let zeros: [u64; 10] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
                zeros.to_field_elements(fields);
            }
            OptFlag::Yes => unimplemented!(), // Is that used ?
            OptFlag::No => {}
        }

        combined_inner_product.to_field_elements(fields);
        b.to_field_elements(fields);
        u64_to_field::<Fp, 2>(xi).to_field_elements(fields);
        bulletproof_challenges.to_field_elements(fields);

        // Index
        {
            let v2::CompositionTypesBranchDataStableV1 {
                proofs_verified,
                domain_log2,
            } = branch_data;
            // https://github.com/MinaProtocol/mina/blob/32a91613c388a71f875581ad72276e762242f802/src/lib/pickles_base/proofs_verified.ml#L58
            let proofs_verified = match proofs_verified {
                v2::PicklesBaseProofsVerifiedStableV1::N0 => [Fp::zero(), Fp::zero()],
                v2::PicklesBaseProofsVerifiedStableV1::N1 => [Fp::zero(), Fp::one()],
                v2::PicklesBaseProofsVerifiedStableV1::N2 => [Fp::one(), Fp::one()],
            };
            let domain_log2: u64 = domain_log2.0.as_u8() as u64;

            proofs_verified.to_field_elements(fields);
            Fp::from(domain_log2).to_field_elements(fields);
        }

        u64_to_field::<Fp, 4>(sponge_digest_before_evaluations).to_field_elements(fields);

        let AllEvals {
            ft_eval1,
            evals:
                EvalsWithPublicInput {
                    evals,
                    public_input,
                },
        } = prev_proof_evals;

        public_input.to_field_elements(fields);
        evals.to_field_elements(fields);
        match hack_feature_flags {
            OptFlag::Maybe => {
                // See above.
                // https://github.com/MinaProtocol/mina/blob/a51f09d09e6ae83362ea74eaca072c8e40d08b52/src/lib/pickles_types/plonk_types.ml#L1028-L1046
                let zeros: [u64; 57] = [0; 57];
                zeros.to_field_elements(fields);
            }
            OptFlag::Yes => unimplemented!(), // Is that used ?
            OptFlag::No => {}
        }
        ft_eval1.to_field_elements(fields);
        prev_challenges.to_field_elements(fields);
        push_affines(prev_challenge_polynomial_commitments, fields);
    }
}
