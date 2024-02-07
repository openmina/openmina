use std::{array, str::FromStr};

use ark_ff::One;
use mina_hasher::Fp;

use crate::{
    bigint::BigInt,
    hash_input::{Inputs, ToInput},
    pseq::PaddedSeq,
    v2::{
        MinaBaseAccountBinableArgStableV2, MinaBaseAccountTimingStableV2,
        MinaBasePermissionsAuthRequiredStableV2, MinaBasePermissionsStableV2,
        MinaBaseVerificationKeyWireStableV1, MinaBaseVerificationKeyWireStableV1WrapIndex,
        MinaBaseZkappAccountStableV2,
        MinaNumbersGlobalSlotSinceGenesisMStableV1, PicklesBaseProofsVerifiedStableV1,
    },
};

use super::hash_noinputs;

impl ToInput for MinaBaseAccountBinableArgStableV2 {
    fn to_input(&self, inputs: &mut Inputs) {
        let MinaBaseAccountBinableArgStableV2 {
            public_key,
            token_id,
            token_symbol,
            balance,
            nonce,
            receipt_chain_hash,
            delegate,
            voting_for,
            timing,
            permissions,
            zkapp,
        } = self;
        to_input_fields!(
            inputs,
            zkapp,
            permissions,
            timing,
            voting_for,
            delegate,
            receipt_chain_hash,
            nonce,
            balance,
            token_symbol,
            token_id,
            public_key,
        );
    }
}

impl ToInput for MinaBaseZkappAccountStableV2 {
    fn to_input(&self, inputs: &mut Inputs) {
        let MinaBaseZkappAccountStableV2 {
            app_state,
            verification_key,
            zkapp_version,
            action_state,
            last_action_slot,
            proved_state,
            zkapp_uri,
        } = self;
        to_input_fields!(
            inputs,
            zkapp_uri,
            *proved_state,
            last_action_slot,
            action_state,
            zkapp_version,
            verification_key,
            app_state
        );
    }
}

impl ToInput for MinaBasePermissionsStableV2 {
    fn to_input(&self, inputs: &mut Inputs) {
        let MinaBasePermissionsStableV2 {
            edit_state,
            access,
            send,
            receive,
            set_delegate,
            set_permissions,
            set_verification_key,
            set_zkapp_uri,
            edit_action_state,
            set_token_symbol,
            increment_nonce,
            set_voting_for,
            set_timing,
        } = self;
        to_input_fields!(
            inputs,
                edit_state,
                access,
                send,
                receive,
                set_delegate,
                set_permissions,
                set_verification_key.0,
                set_verification_key.1,
                set_zkapp_uri,
                edit_action_state,
                set_token_symbol,
                increment_nonce,
                set_voting_for,
                set_timing
        );
    }
}

impl ToInput for MinaBaseAccountTimingStableV2 {
    fn to_input(&self, inputs: &mut Inputs) {
        match self {
            MinaBaseAccountTimingStableV2::Untimed => {
                inputs.append_bool(false);
                inputs.append_u64(0); // initial_minimum_balance
                inputs.append_u32(0); // cliff_time
                inputs.append_u64(0); // cliff_amount
                inputs.append_u32(1); // vesting_period
                inputs.append_u64(0); // vesting_increment
            }
            MinaBaseAccountTimingStableV2::Timed {
                initial_minimum_balance,
                cliff_time,
                cliff_amount,
                vesting_period,
                vesting_increment,
            } => {
                to_input_fields!(
                    inputs,
                    true,
                    initial_minimum_balance,
                    cliff_time,
                    cliff_amount,
                    vesting_period,
                    vesting_increment
                );
            }
        }
    }
}

impl Default for MinaNumbersGlobalSlotSinceGenesisMStableV1 {
    fn default() -> Self {
        Self::SinceGenesis(Default::default())
    }
}

impl Default for MinaBaseZkappAccountStableV2 {
    fn default() -> Self {
        Self {
            app_state: Default::default(),
            verification_key: Default::default(),
            zkapp_version: Default::default(),
            action_state: {
                let empty: BigInt = hash_noinputs("MinaZkappSequenceStateEmptyElt").into();
                let sequence_state: [_; 5] = array::from_fn(|_| empty.clone());
                PaddedSeq(sequence_state)
            },
            last_action_slot: Default::default(),
            proved_state: Default::default(),
            zkapp_uri: Default::default(),
        }
    }
}

impl ToInput for MinaBasePermissionsAuthRequiredStableV2 {
    fn to_input(&self, inputs: &mut Inputs) {
        use MinaBasePermissionsAuthRequiredStableV2::*;
        let (constant, signature_necessary, signature_sufficient) = match self {
            None => (true, false, true),
            Either => (false, false, true),
            Proof => (false, false, false),
            Signature => (false, true, true),
            Impossible => (true, true, false),
        };
        to_input_fields!(inputs, constant, signature_necessary, signature_sufficient);
    }
}

impl Default for MinaBaseVerificationKeyWireStableV1 {
    fn default() -> Self {
        dummy_vk()
    }
}

fn dummy_vk() -> MinaBaseVerificationKeyWireStableV1 {
    let g: (BigInt, BigInt) = (
        Fp::one().into(),
        Fp::from_str(
            "12418654782883325593414442427049395787963493412651469444558597405572177144507",
        )
        .unwrap()
        .into(),
    );
    MinaBaseVerificationKeyWireStableV1 {
        max_proofs_verified: PicklesBaseProofsVerifiedStableV1::N2,
        wrap_index: MinaBaseVerificationKeyWireStableV1WrapIndex {
            sigma_comm: {
                let sigma_comm: [(BigInt, BigInt); 7] = array::from_fn(|_| g.clone());
                PaddedSeq(sigma_comm)
            },
            coefficients_comm: {
                let coefficients_comm: [(BigInt, BigInt); 15] = array::from_fn(|_| g.clone());
                PaddedSeq(coefficients_comm)
            },
            generic_comm: g.clone(),
            psm_comm: g.clone(),
            complete_add_comm: g.clone(),
            mul_comm: g.clone(),
            emul_comm: g.clone(),
            endomul_scalar_comm: g,
        },
        actual_wrap_domain_size: PicklesBaseProofsVerifiedStableV1::N2,
    }
}
