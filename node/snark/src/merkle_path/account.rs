use std::{borrow::Cow, str::FromStr};

use ark_ff::{One, Zero};
use mina_hasher::Fp;
use mina_p2p_messages::{
    bigint::BigInt,
    pseq::PaddedSeq,
    v2::{
        MinaBaseAccountBinableArgStableV2, MinaBaseAccountTimingStableV1,
        MinaBasePermissionsAuthRequiredStableV2, MinaBaseTokenPermissionsStableV1,
        MinaBaseVerificationKeyWireStableV1, MinaBaseVerificationKeyWireStableV1WrapIndex,
        MinaBaseZkappAccountStableV2, MinaBaseZkappStateValueStableV1,
        MinaNumbersNatMake32StableV1, PicklesBaseProofsVerifiedStableV1,
        UnsignedExtendedUInt32StableV1,
    },
};

use crate::{
    hash::{hash_noinputs, hash_with_kimchi, Inputs},
    public_input::protocol_state::MinaHash,
};

fn array_of<T: Clone, const N: usize>(value: T) -> [T; N] {
    std::array::from_fn(|_| value.clone())
}

#[derive(Copy, Clone, Debug)]
pub struct AuthRequiredEncoded {
    constant: bool,
    signature_necessary: bool,
    signature_sufficient: bool,
}

pub fn encode_auth_required(auth: &MinaBasePermissionsAuthRequiredStableV2) -> AuthRequiredEncoded {
    use MinaBasePermissionsAuthRequiredStableV2::*;

    let (constant, signature_necessary, signature_sufficient) = match auth {
        None => (true, false, true),
        Either => (false, false, true),
        Proof => (false, false, false),
        Signature => (false, true, true),
        Impossible => (true, true, false),
        // Both => (false, true, false), // `Both` variant doesn't exist in `p2p` crate
    };

    AuthRequiredEncoded {
        constant,
        signature_necessary,
        signature_sufficient,
    }
}

impl AuthRequiredEncoded {
    #[allow(dead_code)]
    pub fn decode(self) -> MinaBasePermissionsAuthRequiredStableV2 {
        use MinaBasePermissionsAuthRequiredStableV2::*;

        match (
            self.constant,
            self.signature_necessary,
            self.signature_sufficient,
        ) {
            (true, _, false) => Impossible,
            (true, _, true) => None,
            (false, false, false) => Proof,
            (false, true, true) => Signature,
            (false, false, true) => Either,
            (false, true, false) => panic!(), // Should be variant `Both here`
        }
    }

    pub fn to_bits(self) -> [bool; 3] {
        [
            self.constant,
            self.signature_necessary,
            self.signature_sufficient,
        ]
    }
}

fn zkapp_account_default() -> MinaBaseZkappAccountStableV2 {
    MinaBaseZkappAccountStableV2 {
        app_state: {
            let zero: BigInt = Fp::zero().into();
            let app_state: [BigInt; 8] = array_of(zero);
            MinaBaseZkappStateValueStableV1(PaddedSeq(app_state))
        },
        verification_key: None,
        zkapp_version: MinaNumbersNatMake32StableV1(UnsignedExtendedUInt32StableV1(0.into())),
        sequence_state: {
            let empty: BigInt = hash_noinputs("MinaZkappSequenceStateEmptyElt").into();
            let sequence_state: [BigInt; 5] = array_of(empty);
            PaddedSeq(sequence_state)
        },
        last_sequence_slot: UnsignedExtendedUInt32StableV1(0.into()),
        proved_state: false,
        zkapp_uri: Vec::new().into(),
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
                let sigma_comm: [(BigInt, BigInt); 7] = array_of(g.clone());
                PaddedSeq(sigma_comm)
            },
            coefficients_comm: {
                let coefficients_comm: [(BigInt, BigInt); 15] = array_of(g.clone());
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

impl MinaHash for MinaBaseVerificationKeyWireStableV1 {
    fn hash(&self) -> Fp {
        let mut inputs = Inputs::new();

        // https://github.com/MinaProtocol/mina/blob/35b1702fbc295713f9bb46bb17e2d007bc2bab84/src/lib/pickles_base/proofs_verified.ml#L108-L118
        let bits = match self.max_proofs_verified {
            PicklesBaseProofsVerifiedStableV1::N0 => [true, false, false],
            PicklesBaseProofsVerifiedStableV1::N1 => [false, true, false],
            PicklesBaseProofsVerifiedStableV1::N2 => [false, false, true],
        };

        for bit in bits {
            inputs.append_bool(bit);
        }

        let index = &self.wrap_index;

        for field in &index.sigma_comm[..] {
            inputs.append_field(field.0.to_field());
            inputs.append_field(field.1.to_field());
        }

        for field in &index.coefficients_comm[..] {
            inputs.append_field(field.0.to_field());
            inputs.append_field(field.1.to_field());
        }

        inputs.append_field(index.generic_comm.0.to_field());
        inputs.append_field(index.generic_comm.1.to_field());

        inputs.append_field(index.psm_comm.0.to_field());
        inputs.append_field(index.psm_comm.1.to_field());

        inputs.append_field(index.complete_add_comm.0.to_field());
        inputs.append_field(index.complete_add_comm.1.to_field());

        inputs.append_field(index.mul_comm.0.to_field());
        inputs.append_field(index.mul_comm.1.to_field());

        inputs.append_field(index.emul_comm.0.to_field());
        inputs.append_field(index.emul_comm.1.to_field());

        inputs.append_field(index.endomul_scalar_comm.0.to_field());
        inputs.append_field(index.endomul_scalar_comm.1.to_field());

        hash_with_kimchi("MinaSideLoadedVk", &inputs.to_fields())
    }
}

impl MinaHash for MinaBaseAccountBinableArgStableV2 {
    fn hash(&self) -> Fp {
        let mut inputs = Inputs::new();

        // Self::zkapp
        let field_zkapp = {
            let zkapp = match self.zkapp.as_ref() {
                Some(zkapp) => Cow::Borrowed(zkapp),
                None => Cow::Owned(zkapp_account_default()),
            };
            let zkapp = zkapp.as_ref();

            let mut inputs = Inputs::new();

            // Self::zkapp_uri
            // Note: This doesn't cover when zkapp_uri is None, which
            // is never the case for accounts
            let field_zkapp_uri = {
                let mut inputs = Inputs::new();

                for c in zkapp.zkapp_uri.as_ref() {
                    for j in 0..8 {
                        inputs.append_bool((c & (1 << j)) != 0);
                    }
                }
                inputs.append_bool(true);

                crate::hash::hash_with_kimchi("MinaZkappUri", &inputs.to_fields())
            };

            inputs.append_field(field_zkapp_uri);

            inputs.append_bool(zkapp.proved_state);
            inputs.append_u32(zkapp.last_sequence_slot.as_u32());
            for fp in &zkapp.sequence_state[..] {
                inputs.append_field(fp.to_field());
            }
            inputs.append_u32(zkapp.zkapp_version.as_u32());
            let vk_hash = match zkapp.verification_key.as_ref() {
                Some(vk) => vk.hash(),
                None => dummy_vk().hash(),
            };
            inputs.append_field(vk_hash);
            for fp in &zkapp.app_state[..] {
                inputs.append_field(fp.to_field());
            }

            crate::hash::hash_with_kimchi("MinaZkappAccount", &inputs.to_fields())
        };

        inputs.append_field(field_zkapp);

        // Self::permissions
        for auth in [
            &self.permissions.edit_state,
            &self.permissions.send,
            &self.permissions.receive,
            &self.permissions.set_delegate,
            &self.permissions.set_permissions,
            &self.permissions.set_verification_key,
            &self.permissions.set_zkapp_uri,
            &self.permissions.edit_sequence_state,
            &self.permissions.set_token_symbol,
            &self.permissions.increment_nonce,
            &self.permissions.set_voting_for,
        ] {
            for bit in encode_auth_required(auth).to_bits() {
                inputs.append_bool(bit);
            }
        }

        // Self::timing
        match &self.timing {
            MinaBaseAccountTimingStableV1::Untimed => {
                inputs.append_bool(false);
                inputs.append_u64(0); // initial_minimum_balance
                inputs.append_u32(0); // cliff_time
                inputs.append_u64(0); // cliff_amount
                inputs.append_u32(1); // vesting_period
                inputs.append_u64(0); // vesting_increment
            }
            MinaBaseAccountTimingStableV1::Timed {
                initial_minimum_balance,
                cliff_time,
                cliff_amount,
                vesting_period,
                vesting_increment,
            } => {
                inputs.append_bool(true);
                inputs.append_u64(initial_minimum_balance.as_u64());
                inputs.append_u32(cliff_time.as_u32());
                inputs.append_u64(cliff_amount.as_u64());
                inputs.append_u32(vesting_period.as_u32());
                inputs.append_u64(vesting_increment.as_u64());
            }
        }

        // Self::voting_for
        inputs.append_field(self.voting_for.to_field());

        // Self::delegate
        match self.delegate.as_ref() {
            Some(delegate) => {
                inputs.append_field(delegate.x.to_field());
                inputs.append_bool(delegate.is_odd);
            }
            None => {
                // Public_key.Compressed.empty
                inputs.append_field(Fp::zero());
                inputs.append_bool(false);
            }
        }

        // Self::receipt_chain_hash
        inputs.append_field(self.receipt_chain_hash.to_field());

        // Self::nonce
        inputs.append_u32(self.nonce.as_u32());

        // Self::balance
        inputs.append_u64(self.balance.as_u64());

        // Self::token_symbol

        // https://github.com/MinaProtocol/mina/blob/2fac5d806a06af215dbab02f7b154b4f032538b7/src/lib/mina_base/account.ml#L97
        assert!(self.token_symbol.len() <= 6);

        let mut s = <[u8; 6]>::default();
        if !self.token_symbol.is_empty() {
            let len = self.token_symbol.len();
            s[..len].copy_from_slice(self.token_symbol.as_ref());
        }
        inputs.append_u48(s);

        // Self::token_permissions
        // match self.zkapp {
        //     MinaBasePermissionsStableV2::TokenOwned {
        //         disable_new_accounts,
        //     } => {
        //         let bits = if disable_new_accounts { 0b10 } else { 0b00 };
        //         inputs.append_u2(0b01 | bits);
        //     }
        //     MinaBasePermissionsStableV2::NotOwned { account_disabled } => {
        //         let bits = if account_disabled { 0b10 } else { 0b00 };
        //         inputs.append_u2(bits);
        //     }
        // }

        // Self::token_id
        inputs.append_field(self.token_id.to_field());

        // Self::public_key
        inputs.append_field(self.public_key.x.to_field());
        inputs.append_bool(self.public_key.is_odd);

        crate::hash::hash_with_kimchi("MinaAccount", &inputs.to_fields())
    }
}
