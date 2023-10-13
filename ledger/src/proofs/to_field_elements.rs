use ark_ff::Field;
use mina_hasher::Fp;
use mina_p2p_messages::v2::{self, MinaStateProtocolStateValueStableV2};

use crate::{
    proofs::public_input::protocol_state::MinaHash,
    scan_state::{
        fee_excess::FeeExcess,
        scan_state::transaction_snark::{Registers, SokDigest, Statement},
        transaction_logic::zkapp_statement::ZkappStatement,
    },
};

use super::witness::FieldWitness;

pub trait ToFieldElements<F: Field> {
    fn to_field_elements(&self, fields: &mut Vec<F>);

    fn to_field_elements_owned(&self) -> Vec<F> {
        let mut fields = Vec::with_capacity(1024);
        self.to_field_elements(&mut fields);
        fields
    }
}

impl ToFieldElements<Fp> for MinaStateProtocolStateValueStableV2 {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        fields.push(MinaHash::hash(self))
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

            fields.push(*first_pass_ledger);
            fields.push(*second_pass_ledger);
            fields.push(pending_coinbase_stack.data.0);
            fields.push(pending_coinbase_stack.state.init);
            fields.push(pending_coinbase_stack.state.curr);
            fields.push(local_state.stack_frame);
            fields.push(local_state.call_stack);
            fields.push(local_state.transaction_commitment);
            fields.push(local_state.full_transaction_commitment);
            fields.push(local_state.excess.magnitude.as_u64().into());
            fields.push(sign_to_field(local_state.excess.sgn));
            fields.push(local_state.supply_increase.magnitude.as_u64().into());
            fields.push(sign_to_field(local_state.supply_increase.sgn));
            fields.push(local_state.ledger);
            fields.push((local_state.success as u64).into());
            fields.push(local_state.account_update_index.as_u32().into());
            fields.push((local_state.will_succeed as u64).into());
        };

        add_register(source);
        add_register(target);
        fields.push(*connecting_ledger_left);
        fields.push(*connecting_ledger_right);
        fields.push(supply_increase.magnitude.as_u64().into());
        fields.push(sign_to_field(supply_increase.sgn));

        let FeeExcess {
            fee_token_l,
            fee_excess_l,
            fee_token_r,
            fee_excess_r,
        } = fee_excess;

        fields.push(fee_token_l.0);
        fields.push(fee_excess_l.magnitude.as_u64().into());
        fields.push(sign_to_field(fee_excess_l.sgn));

        fields.push(fee_token_r.0);
        fields.push(fee_excess_r.magnitude.as_u64().into());
        fields.push(sign_to_field(fee_excess_r.sgn));

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
