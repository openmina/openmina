#![allow(clippy::type_complexity)]

use std::{marker::PhantomData, ops::Deref};

use ark_ff::{BigInteger256, FromBytes, PrimeField, ToBytes};
use mina_hasher::Fp;
use mina_signer::CompressedPubKey;
use o1_utils::FieldHelpers;
///! Types generated with https://github.com/name-placeholder/bin-prot-rs
use serde::{Deserialize, Serialize, Serializer};

use crate::{FpExt, PlonkVerificationKeyEvals, VerificationKey};

use super::{Account, AccountId, AuthRequired, TokenId};

#[derive(Debug)]
pub struct BigInt(pub [u8; 32]);

impl Serialize for BigInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0[..])
    }
}

impl Deref for BigInt {
    type Target = [u8; 32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<BigInt> for Fp {
    fn from(val: BigInt) -> Self {
        assert_eq!(val.0.len(), 32);
        Fp::read(&val.0[..]).unwrap()
        // match Fp::read(&val.0[..]) {
        //     Ok(ok) => ok,
        //     Err(e) => {
        //         eprintln!("ERR={:?}", e);
        //         eprintln!("VALUE LEN={:?} V={:?}", val.0.len(), val);

        //         let bigint = BigInteger256::read(&val.0[..]).unwrap();
        //         println!("BIGINT={:?}", bigint);
        //         println!("BIGINT={:?}", bigint.to_string());
        //         // let fp = Fp::from(bigint);
        //         // let fp = Fp::from_bits;
        //         // println!("FP={:?}", fp);

        //         panic!()
        //     },
        // }
    }
}

impl<'de> Deserialize<'de> for BigInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Error, Visitor};

        struct BigIntVisitor;

        impl<'de> Visitor<'de> for BigIntVisitor {
            type Value = [u8; 32];

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a slice containing 32 bytes")
            }

            fn visit_bytes<E>(self, slice: &[u8]) -> Result<Self::Value, E>
            where
                E: Error,
            {
                slice
                    .get(..32)
                    .and_then(|v| v.try_into().ok())
                    .ok_or_else(|| E::invalid_length(slice.len(), &"32 bytes"))
            }
        }

        deserializer.deserialize_any(BigIntVisitor).map(BigInt)
    }
}

impl From<NonZeroCurvePointUncompressedStableV1> for CompressedPubKey {
    fn from(val: NonZeroCurvePointUncompressedStableV1) -> Self {
        CompressedPubKey {
            x: Fp::read(&val.x[..]).unwrap(),
            is_odd: val.is_odd,
        }
    }
}

impl From<CompressedPubKey> for NonZeroCurvePointUncompressedStableV1 {
    fn from(v: CompressedPubKey) -> NonZeroCurvePointUncompressedStableV1 {
        Self {
            x: v.x.into(),
            is_odd: v.is_odd,
        }
    }
}

impl From<Fp> for BigInt {
    fn from(fp: Fp) -> Self {
        let mut x = [0; 32];
        fp.write(&mut x[..]).unwrap();
        BigInt(x)
    }
}

impl From<Account> for MinaBaseAccountBinableArgStableV2 {
    fn from(acc: Account) -> Self {
        Self {
            public_key: acc.public_key.into(),
            token_id: acc.token_id.0.into(),
            token_permissions: match acc.token_permissions {
                super::TokenPermissions::TokenOwned {
                    disable_new_accounts,
                } => MinaBaseTokenPermissionsStableV1::TokenOwned {
                    disable_new_accounts,
                },
                super::TokenPermissions::NotOwned { account_disabled } => {
                    MinaBaseTokenPermissionsStableV1::NotOwned { account_disabled }
                }
            },
            token_symbol: acc.token_symbol,
            balance: acc.balance as i64,
            nonce: acc.nonce as i32,
            receipt_chain_hash: acc.receipt_chain_hash.0.into(),
            delegate: acc.delegate.map(|d| d.into()),
            voting_for: acc.voting_for.0.into(),
            timing: match acc.timing {
                super::Timing::Untimed => MinaBaseAccountTimingStableV1::Untimed,
                super::Timing::Timed {
                    initial_minimum_balance,
                    cliff_time,
                    cliff_amount,
                    vesting_period,
                    vesting_increment,
                } => MinaBaseAccountTimingStableV1::Timed {
                    initial_minimum_balance: initial_minimum_balance as i64,
                    cliff_time: cliff_time as i32,
                    cliff_amount: cliff_amount as i64,
                    vesting_period: vesting_period as i32,
                    vesting_increment: vesting_increment as i64,
                },
            },
            permissions: MinaBasePermissionsStableV2 {
                edit_state: acc.permissions.edit_state.into(),
                send: acc.permissions.send.into(),
                receive: acc.permissions.receive.into(),
                set_delegate: acc.permissions.set_delegate.into(),
                set_permissions: acc.permissions.set_permissions.into(),
                set_verification_key: acc.permissions.set_verification_key.into(),
                set_zkapp_uri: acc.permissions.set_zkapp_uri.into(),
                edit_sequence_state: acc.permissions.edit_sequence_state.into(),
                set_token_symbol: acc.permissions.set_token_symbol.into(),
                increment_nonce: acc.permissions.increment_nonce.into(),
                set_voting_for: acc.permissions.set_voting_for.into(),
            },
            zkapp: acc.zkapp.map(|zkapp| {
                let s = zkapp.app_state;
                let app_state = (
                    s[0].into(),
                    (
                        s[1].into(),
                        (
                            s[2].into(),
                            (
                                s[3].into(),
                                (s[4].into(), (s[5].into(), (s[6].into(), (s[7].into(), ())))),
                            ),
                        ),
                    ),
                );

                let verification_key =
                    zkapp
                        .verification_key
                        .map(|vk| MinaBaseVerificationKeyWireStableV1 {
                            max_proofs_verified: match vk.max_proofs_verified {
                                super::ProofVerified::N0 => PicklesBaseProofsVerifiedStableV1::N0,
                                super::ProofVerified::N1 => PicklesBaseProofsVerifiedStableV1::N1,
                                super::ProofVerified::N2 => PicklesBaseProofsVerifiedStableV1::N2,
                            },
                            wrap_index: MinaBaseVerificationKeyWireStableV1WrapIndex::from(
                                vk.wrap_index,
                            ),
                        });

                let seq = zkapp.sequence_state;
                let sequence_state = (
                    seq[0].into(),
                    (
                        seq[1].into(),
                        (seq[2].into(), (seq[3].into(), (seq[4].into(), ()))),
                    ),
                );

                MinaBaseZkappAccountStableV2 {
                    app_state,
                    verification_key,
                    zkapp_version: zkapp.zkapp_version as i32,
                    sequence_state,
                    last_sequence_slot: zkapp.last_sequence_slot as i32,
                    proved_state: zkapp.proved_state,
                }
            }),
            zkapp_uri: acc.zkapp_uri,
        }
    }
}

impl From<AuthRequired> for MinaBasePermissionsAuthRequiredStableV2 {
    fn from(perms: AuthRequired) -> Self {
        match perms {
            AuthRequired::None => Self::None,
            AuthRequired::Either => Self::Either,
            AuthRequired::Proof => Self::Proof,
            AuthRequired::Signature => Self::Signature,
            AuthRequired::Impossible => Self::Impossible,
            AuthRequired::Both => panic!("doesn't exist in `develop` branch"),
        }
    }
}

/// Origin: Mina_base__Account.Binable_arg.Stable.V2.t
/// Location: [src/lib/mina_base/account.ml:313:6](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/mina_base/account.ml#L313)
/// Location: [src/lib/mina_base/account.ml:226:6](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/mina_base/account.ml#L226)
#[derive(Debug, Serialize, Deserialize)]
pub struct MinaBaseAccountBinableArgStableV2 {
    pub public_key: NonZeroCurvePointUncompressedStableV1,
    pub token_id: MinaBaseAccountIdDigestStableV1,
    pub token_permissions: MinaBaseTokenPermissionsStableV1,
    pub token_symbol: MinaBaseAccountTokenSymbolStableV1,
    pub balance: CurrencyBalanceStableV1,
    pub nonce: MinaBaseAccountBinableArgStableV2Arg5,
    pub receipt_chain_hash: MinaBaseReceiptChainHashStableV1,
    pub delegate: Option<NonZeroCurvePointUncompressedStableV1>,
    pub voting_for: DataHashLibStateHashStableV1,
    pub timing: MinaBaseAccountTimingStableV1,
    pub permissions: MinaBasePermissionsStableV2,
    pub zkapp: Option<MinaBaseZkappAccountStableV2>,
    pub zkapp_uri: String,
}

/// Origin: Non_zero_curve_point.Uncompressed.Stable.V1.t
/// Location: [src/lib/non_zero_curve_point/non_zero_curve_point.ml:44:6](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/non_zero_curve_point/non_zero_curve_point.ml#L44)
/// Location: [src/lib/non_zero_curve_point/compressed_poly.ml:13:6](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/non_zero_curve_point/compressed_poly.ml#L13)
#[derive(Debug, Serialize, Deserialize)]
pub struct NonZeroCurvePointUncompressedStableV1 {
    pub x: BigInt,
    pub is_odd: bool,
}

/// Origin: Mina_base__Account_id.Digest.Stable.V1.t
/// Location: [src/lib/mina_base/account_id.ml:53:6](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/mina_base/account_id.ml#L53)
pub type MinaBaseAccountIdDigestStableV1 = BigInt;

/// Origin: Mina_base__Token_permissions.Stable.V1.t
/// Location: [src/lib/mina_base/token_permissions.ml:9:4](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/mina_base/token_permissions.ml#L9)
#[derive(Debug, Serialize, Deserialize)]
pub enum MinaBaseTokenPermissionsStableV1 {
    TokenOwned { disable_new_accounts: bool },
    NotOwned { account_disabled: bool },
}

/// Origin: Mina_base__Account.Token_symbol.Stable.V1.t
/// Location: [src/string.ml:14:6](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/string.ml#L14)
pub type MinaBaseAccountTokenSymbolStableV1 = String;

/// Origin: Unsigned_extended.UInt64.Stable.V1.t
/// Location: [src/int64.ml:6:6](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/int64.ml#L6)
pub type UnsignedExtendedUInt64StableV1 = i64;

/// Origin: Currency.Amount.Make_str.Stable.V1.t
/// Location: [src/lib/currency/currency.ml:992:8](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/currency/currency.ml#L992)
pub type CurrencyAmountMakeStrStableV1 = UnsignedExtendedUInt64StableV1;

/// Origin: Currency.Balance.Stable.V1.t
/// Location: [src/lib/currency/currency.ml:1031:6](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/currency/currency.ml#L1031)
pub type CurrencyBalanceStableV1 = CurrencyAmountMakeStrStableV1;

/// Origin: Unsigned_extended.UInt32.Stable.V1.t
/// Location: [src/int32.ml:6:6](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/int32.ml#L6)
pub type UnsignedExtendedUInt32StableV1 = i32;

/// Location: [src/lib/mina_numbers/nat.ml:258:6](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/mina_numbers/nat.ml#L258)
pub type MinaBaseAccountBinableArgStableV2Arg5 = UnsignedExtendedUInt32StableV1;

/// Origin: Mina_base__Receipt.Chain_hash.Stable.V1.t
/// Location: [src/lib/mina_base/receipt.ml:31:6](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/mina_base/receipt.ml#L31)
pub type MinaBaseReceiptChainHashStableV1 = BigInt;

/// Origin: Data_hash_lib__State_hash.Stable.V1.t
/// Location: [src/lib/data_hash_lib/state_hash.ml:42:4](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/data_hash_lib/state_hash.ml#L42)
pub type DataHashLibStateHashStableV1 = BigInt;

/// Location: [src/lib/mina_numbers/nat.ml:258:6](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/mina_numbers/nat.ml#L258)
pub type MinaBaseAccountTimingStableV1Arg0 = UnsignedExtendedUInt32StableV1;

/// Origin: Mina_base__Account_timing.Stable.V1.t
/// Location: [src/lib/mina_base/account_timing.ml:30:4](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/mina_base/account_timing.ml#L30)
/// Location: [src/lib/mina_base/account_timing.ml:13:6](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/mina_base/account_timing.ml#L13)
#[derive(Debug, Serialize, Deserialize)]
pub enum MinaBaseAccountTimingStableV1 {
    Untimed,
    Timed {
        initial_minimum_balance: CurrencyBalanceStableV1,
        cliff_time: MinaBaseAccountTimingStableV1Arg0,
        cliff_amount: CurrencyAmountMakeStrStableV1,
        vesting_period: MinaBaseAccountTimingStableV1Arg0,
        vesting_increment: CurrencyAmountMakeStrStableV1,
    },
}

/// Origin: Mina_base__Permissions.Auth_required.Stable.V2.t
/// Location: [src/lib/mina_base/permissions.ml:53:6](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/mina_base/permissions.ml#L53)
#[derive(Debug, Serialize, Deserialize)]
pub enum MinaBasePermissionsAuthRequiredStableV2 {
    None,
    Either,
    Proof,
    Signature,
    Impossible,
}

/// Origin: Mina_base__Permissions.Stable.V2.t
/// Location: [src/lib/mina_base/permissions.ml:350:4](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/mina_base/permissions.ml#L350)
/// Location: [src/lib/mina_base/permissions.ml:319:6](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/mina_base/permissions.ml#L319)
#[derive(Debug, Serialize, Deserialize)]
pub struct MinaBasePermissionsStableV2 {
    pub edit_state: MinaBasePermissionsAuthRequiredStableV2,
    pub send: MinaBasePermissionsAuthRequiredStableV2,
    pub receive: MinaBasePermissionsAuthRequiredStableV2,
    pub set_delegate: MinaBasePermissionsAuthRequiredStableV2,
    pub set_permissions: MinaBasePermissionsAuthRequiredStableV2,
    pub set_verification_key: MinaBasePermissionsAuthRequiredStableV2,
    pub set_zkapp_uri: MinaBasePermissionsAuthRequiredStableV2,
    pub edit_sequence_state: MinaBasePermissionsAuthRequiredStableV2,
    pub set_token_symbol: MinaBasePermissionsAuthRequiredStableV2,
    pub increment_nonce: MinaBasePermissionsAuthRequiredStableV2,
    pub set_voting_for: MinaBasePermissionsAuthRequiredStableV2,
}

/// Origin: Mina_base__Zkapp_state.Value.Stable.V1.t
/// Location: [src/lib/mina_base/zkapp_state.ml:50:6](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/mina_base/zkapp_state.ml#L50)
/// Location: [src/lib/mina_base/zkapp_state.ml:17:6](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/mina_base/zkapp_state.ml#L17)
pub type MinaBaseZkappStateValueStableV1 = (
    BigInt,
    (
        BigInt,
        (BigInt, (BigInt, (BigInt, (BigInt, (BigInt, (BigInt, ())))))),
    ),
);

/// Origin: Pickles_base__Proofs_verified.Stable.V1.t
/// Location: [src/lib/pickles_base/proofs_verified.ml:7:4](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/pickles_base/proofs_verified.ml#L7)
#[derive(Debug, Serialize, Deserialize)]
pub enum PicklesBaseProofsVerifiedStableV1 {
    N0,
    N1,
    N2,
}

/// Location: [src/lib/pickles_types/plonk_verification_key_evals.ml:9:4](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/pickles_types/plonk_verification_key_evals.ml#L9)
#[derive(Debug, Serialize, Deserialize)]
pub struct MinaBaseVerificationKeyWireStableV1WrapIndex {
    pub sigma_comm: (
        (BigInt, BigInt),
        (
            (BigInt, BigInt),
            (
                (BigInt, BigInt),
                (
                    (BigInt, BigInt),
                    ((BigInt, BigInt), ((BigInt, BigInt), ((BigInt, BigInt), ()))),
                ),
            ),
        ),
    ),
    pub coefficients_comm: (
        (BigInt, BigInt),
        (
            (BigInt, BigInt),
            (
                (BigInt, BigInt),
                (
                    (BigInt, BigInt),
                    (
                        (BigInt, BigInt),
                        (
                            (BigInt, BigInt),
                            (
                                (BigInt, BigInt),
                                (
                                    (BigInt, BigInt),
                                    (
                                        (BigInt, BigInt),
                                        (
                                            (BigInt, BigInt),
                                            (
                                                (BigInt, BigInt),
                                                (
                                                    (BigInt, BigInt),
                                                    (
                                                        (BigInt, BigInt),
                                                        ((BigInt, BigInt), ((BigInt, BigInt), ())),
                                                    ),
                                                ),
                                            ),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    ),
    pub generic_comm: (BigInt, BigInt),
    pub psm_comm: (BigInt, BigInt),
    pub complete_add_comm: (BigInt, BigInt),
    pub mul_comm: (BigInt, BigInt),
    pub emul_comm: (BigInt, BigInt),
    pub endomul_scalar_comm: (BigInt, BigInt),
}

/// Origin: Mina_base__Verification_key_wire.Stable.V1.t
/// Location: [src/lib/pickles/side_loaded_verification_key.ml:169:6](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/pickles/side_loaded_verification_key.ml#L169)
/// Location: [src/lib/pickles_base/side_loaded_verification_key.ml:150:6](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/pickles_base/side_loaded_verification_key.ml#L150)
#[derive(Debug, Serialize, Deserialize)]
pub struct MinaBaseVerificationKeyWireStableV1 {
    pub max_proofs_verified: PicklesBaseProofsVerifiedStableV1,
    pub wrap_index: MinaBaseVerificationKeyWireStableV1WrapIndex,
}

impl From<PlonkVerificationKeyEvals> for MinaBaseVerificationKeyWireStableV1WrapIndex {
    fn from(vk: PlonkVerificationKeyEvals) -> Self {
        let sigma = vk.sigma;
        let sigma = (
            sigma[0].into(),
            (
                sigma[1].into(),
                (
                    sigma[2].into(),
                    (
                        sigma[3].into(),
                        (sigma[4].into(), (sigma[5].into(), (sigma[6].into(), ()))),
                    ),
                ),
            ),
        );

        let coef = vk.coefficients;
        #[rustfmt::skip]
        let coef = {
            (
                coef[0].into(),
                (
                    coef[1].into(), (coef[2].into(), (coef[3].into(), (coef[4].into(), (coef[5].into(), (coef[6].into(), (coef[7].into(), (coef[8].into(),
                                                                                                                                           (coef[9].into(), (coef[10].into(), (coef[11].into(), (coef[12].into(), (coef[13].into(), (coef[14].into(), ())))))))))))),
                    ),
                ),
            )
        };

        Self {
            sigma_comm: sigma,
            coefficients_comm: coef,
            generic_comm: vk.generic.into(),
            psm_comm: vk.psm.into(),
            complete_add_comm: vk.complete_add.into(),
            mul_comm: vk.mul.into(),
            emul_comm: vk.emul.into(),
            endomul_scalar_comm: vk.endomul_scalar.into(),
        }
    }
}

/// Origin: Mina_numbers__Nat.Make32.Stable.V1.t
/// Location: [src/lib/mina_numbers/nat.ml:258:6](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/mina_numbers/nat.ml#L258)
pub type MinaNumbersNatMake32StableV1 = UnsignedExtendedUInt32StableV1;

/// Origin: Mina_base__Zkapp_account.Stable.V2.t
/// Location: [src/lib/mina_base/zkapp_account.ml:149:4](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/mina_base/zkapp_account.ml#L149)
/// Location: [src/lib/mina_base/zkapp_account.ml:115:6](https://github.com/MinaProtocol/mina/blob/b14f0da9ebae87acd8764388ab4681ca10f07c89/src/lib/mina_base/zkapp_account.ml#L115)
#[derive(Debug, Serialize, Deserialize)]
pub struct MinaBaseZkappAccountStableV2 {
    pub app_state: MinaBaseZkappStateValueStableV1,
    pub verification_key: Option<MinaBaseVerificationKeyWireStableV1>,
    pub zkapp_version: MinaNumbersNatMake32StableV1,
    pub sequence_state: (BigInt, (BigInt, (BigInt, (BigInt, (BigInt, ()))))),
    pub last_sequence_slot: MinaBaseAccountTimingStableV1Arg0,
    pub proved_state: bool,
}

// Following types were written manually

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountIdV2 {
    pub public_key: NonZeroCurvePointUncompressedStableV1,
    pub token_id: MinaBaseAccountIdDigestStableV1,
}

impl From<AccountId> for AccountIdV2 {
    fn from(account_id: AccountId) -> Self {
        Self {
            public_key: account_id.public_key.into(),
            token_id: account_id.token_id.0.into(),
        }
    }
}

impl From<AccountIdV2> for AccountId {
    fn from(account_id: AccountIdV2) -> Self {
        Self {
            public_key: account_id.public_key.into(),
            token_id: TokenId(account_id.token_id.into()),
        }
    }
}

impl From<TokenId> for MinaBaseAccountIdDigestStableV1 {
    fn from(token_id: TokenId) -> Self {
        token_id.0.into()
    }
}

impl From<MinaBaseAccountIdDigestStableV1> for TokenId {
    fn from(token_id: MinaBaseAccountIdDigestStableV1) -> Self {
        Self(token_id.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    #[test]
    fn test_deserialize_account() {
        let bytes: &[u8] = &[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 1, 0, 3, 115, 101, 98, 0, 0, 155, 228, 183, 197, 30, 217, 194,
            228, 82, 71, 39, 128, 95, 211, 111, 82, 32, 251, 252, 112, 167, 73, 246, 38, 35, 176,
            237, 41, 8, 67, 51, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 0, 3, 3, 3, 3, 3, 3, 3, 3, 0, 0,
        ];

        println!("len={:?}", bytes.len());
        let result: MinaBaseAccountBinableArgStableV2 = serde_binprot::from_slice(bytes).unwrap();
        println!("account={:#?}", result);
    }
}
