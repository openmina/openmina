#![allow(clippy::type_complexity)]

use crate::PlonkVerificationKeyEvals;

use super::{Account, AccountId, AuthRequired, TokenId};

impl From<Account> for mina_p2p_messages::v2::MinaBaseAccountBinableArgStableV2 {
    fn from(acc: Account) -> Self {
        use mina_p2p_messages::v2::*;

        Self {
            public_key: acc.public_key.into(),
            token_id: MinaBaseAccountIdMakeStrDigestStableV1(acc.token_id.0.into()),
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
            token_symbol: MinaBaseAccountTokenSymbolStableV1(acc.token_symbol.as_bytes().into()),
            balance: CurrencyMakeStrBalanceStableV1(CurrencyMakeStrAmountMakeStrStableV1(
                UnsignedExtendedUInt64StableV1((acc.balance as i64).into()),
            )),
            nonce: UnsignedExtendedUInt32StableV1((acc.nonce as i32).into()),
            receipt_chain_hash: MinaBaseReceiptChainHashStableV1(acc.receipt_chain_hash.0.into()),
            delegate: acc.delegate.map(|d| d.into()),
            voting_for: DataHashLibStateHashStableV1(acc.voting_for.0.into()),
            timing: match acc.timing {
                super::Timing::Untimed => MinaBaseAccountTimingStableV1::Untimed,
                super::Timing::Timed {
                    initial_minimum_balance,
                    cliff_time,
                    cliff_amount,
                    vesting_period,
                    vesting_increment,
                } => MinaBaseAccountTimingStableV1::Timed {
                    initial_minimum_balance: CurrencyMakeStrBalanceStableV1(
                        CurrencyMakeStrAmountMakeStrStableV1(UnsignedExtendedUInt64StableV1(
                            (initial_minimum_balance as i64).into(),
                        )),
                    ),
                    cliff_time: UnsignedExtendedUInt32StableV1((cliff_time as i32).into()),
                    cliff_amount: CurrencyMakeStrAmountMakeStrStableV1(
                        UnsignedExtendedUInt64StableV1((cliff_amount as i64).into()),
                    ),
                    vesting_period: UnsignedExtendedUInt32StableV1((vesting_period as i32).into()),
                    vesting_increment: CurrencyMakeStrAmountMakeStrStableV1(
                        UnsignedExtendedUInt64StableV1((vesting_increment as i64).into()),
                    ),
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
                let app_state = MinaBaseZkappStateValueStableV1(
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
                    zkapp_version: MinaNumbersNatMake32StableV1(UnsignedExtendedUInt32StableV1(
                        (zkapp.zkapp_version as i32).into(),
                    )),
                    sequence_state,
                    last_sequence_slot: UnsignedExtendedUInt32StableV1(
                        (zkapp.last_sequence_slot as i32).into(),
                    ),
                    proved_state: zkapp.proved_state,
                }
            }),
            zkapp_uri: acc.zkapp_uri.as_bytes().into(),
        }
    }
}

impl From<AuthRequired> for mina_p2p_messages::v2::MinaBasePermissionsAuthRequiredStableV2 {
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

impl From<PlonkVerificationKeyEvals>
    for mina_p2p_messages::v2::MinaBaseVerificationKeyWireStableV1WrapIndex
{
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

// // Following types were written manually

impl From<AccountId> for mina_p2p_messages::v2::MinaBaseAccountIdMakeStrStableV2 {
    fn from(account_id: AccountId) -> Self {
        Self(account_id.public_key.into(), account_id.token_id.into())
    }
}

impl From<mina_p2p_messages::v2::MinaBaseAccountIdMakeStrStableV2> for AccountId {
    fn from(account_id: mina_p2p_messages::v2::MinaBaseAccountIdMakeStrStableV2) -> Self {
        Self {
            public_key: account_id.0.into(),
            token_id: account_id.1.into(),
        }
    }
}

impl From<TokenId> for mina_p2p_messages::v2::MinaBaseAccountIdMakeStrDigestStableV1 {
    fn from(token_id: TokenId) -> Self {
        Self(token_id.0.into())
    }
}

impl From<mina_p2p_messages::v2::MinaBaseAccountIdMakeStrDigestStableV1> for TokenId {
    fn from(token_id: mina_p2p_messages::v2::MinaBaseAccountIdMakeStrDigestStableV1) -> Self {
        Self(token_id.0.into())
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
        let result: Account = Account::deserialize(bytes);
        println!("account={:#?}", result);
    }
}
