#![allow(clippy::type_complexity)]

use ark_ff::Field;
use mina_p2p_messages::{
    bigint::BigInt,
    pseq::PaddedSeq,
    v2::{
        MinaBaseAccountBinableArgStableV2, MinaBaseAccountIdDigestStableV1,
        MinaBaseAccountIdStableV2, MinaBaseAccountTimingStableV1,
        MinaBasePermissionsAuthRequiredStableV2, MinaBasePermissionsStableV2,
        MinaBaseTokenPermissionsStableV1, MinaBaseVerificationKeyWireStableV1,
        NonZeroCurvePointUncompressedStableV1, PicklesBaseProofsVerifiedStableV1, TokenIdKeyHash,
    },
};

use crate::{
    scan_state::{
        currency::{Amount, Balance},
        transaction_logic::{zkapp_command::Nonce, Slot},
    },
    CurveAffine, Permissions, PlonkVerificationKeyEvals, ProofVerified, ReceiptChainHash, Timing,
    TokenPermissions, VerificationKey, VotingFor, ZkAppAccount,
};

use super::{Account, AccountId, AuthRequired, TokenId};

impl binprot::BinProtRead for TokenId {
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        let token_id = TokenIdKeyHash::binprot_read(r)?;
        let token_id: MinaBaseAccountIdDigestStableV1 = token_id.into_inner();
        Ok(token_id.into())
    }
}

impl binprot::BinProtWrite for TokenId {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let token_id: MinaBaseAccountIdDigestStableV1 = self.clone().into();
        let token_id: TokenIdKeyHash = token_id.into();
        token_id.binprot_write(w)?;
        Ok(())
    }
}

impl<F> From<(BigInt, BigInt)> for CurveAffine<F>
where
    F: Field + From<BigInt>,
{
    fn from((a, b): (BigInt, BigInt)) -> Self {
        Self(a.into(), b.into())
    }
}

impl<F> From<CurveAffine<F>> for (BigInt, BigInt)
where
    F: Field + Into<BigInt>,
{
    fn from(fps: CurveAffine<F>) -> Self {
        (fps.0.into(), fps.1.into())
    }
}

impl<F> From<&(BigInt, BigInt)> for CurveAffine<F>
where
    F: Field + From<BigInt>,
{
    fn from((a, b): &(BigInt, BigInt)) -> Self {
        Self(a.to_field(), b.to_field())
    }
}

impl<F> From<&CurveAffine<F>> for (BigInt, BigInt)
where
    F: Field + Into<BigInt>,
{
    fn from(fps: &CurveAffine<F>) -> Self {
        (fps.0.into(), fps.1.into())
    }
}

impl binprot::BinProtRead for AccountId {
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        let account_id = MinaBaseAccountIdStableV2::binprot_read(r)?;
        Ok(account_id.into())
    }
}

impl binprot::BinProtWrite for AccountId {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let account_id: MinaBaseAccountIdStableV2 = self.clone().into();
        account_id.binprot_write(w)?;
        Ok(())
    }
}

pub fn array_into<'a, T, U, const N: usize>(value: &'a [T; N]) -> [U; N]
where
    T: 'a,
    U: From<&'a T>,
{
    std::array::from_fn(|i| U::from(&value[i]))
}

pub fn array_into_with<'a, T, U, F, const N: usize>(value: &'a [T; N], fun: F) -> [U; N]
where
    T: 'a,
    F: Fn(&T) -> U,
{
    std::array::from_fn(|i| fun(&value[i]))
}

impl From<&MinaBaseVerificationKeyWireStableV1> for VerificationKey {
    fn from(vk: &MinaBaseVerificationKeyWireStableV1) -> Self {
        let sigma = array_into(&vk.wrap_index.sigma_comm);
        let coefficients = array_into(&vk.wrap_index.coefficients_comm);

        VerificationKey {
            max_proofs_verified: match vk.max_proofs_verified {
                PicklesBaseProofsVerifiedStableV1::N0 => ProofVerified::N0,
                PicklesBaseProofsVerifiedStableV1::N1 => ProofVerified::N1,
                PicklesBaseProofsVerifiedStableV1::N2 => ProofVerified::N2,
            },
            wrap_index: PlonkVerificationKeyEvals {
                sigma,
                coefficients,
                generic: (&vk.wrap_index.generic_comm).into(),
                psm: (&vk.wrap_index.psm_comm).into(),
                complete_add: (&vk.wrap_index.complete_add_comm).into(),
                mul: (&vk.wrap_index.mul_comm).into(),
                emul: (&vk.wrap_index.emul_comm).into(),
                endomul_scalar: (&vk.wrap_index.endomul_scalar_comm).into(),
            },
            wrap_vk: None,
        }
    }
}

impl From<&VerificationKey> for MinaBaseVerificationKeyWireStableV1 {
    fn from(vk: &VerificationKey) -> Self {
        Self {
            max_proofs_verified: match vk.max_proofs_verified {
                super::ProofVerified::N0 => PicklesBaseProofsVerifiedStableV1::N0,
                super::ProofVerified::N1 => PicklesBaseProofsVerifiedStableV1::N1,
                super::ProofVerified::N2 => PicklesBaseProofsVerifiedStableV1::N2,
            },
            wrap_index: (&vk.wrap_index).into(),
        }
    }
}

impl From<&MinaBaseAccountTimingStableV1> for Timing {
    fn from(timing: &MinaBaseAccountTimingStableV1) -> Self {
        match timing {
            MinaBaseAccountTimingStableV1::Untimed => Timing::Untimed,
            MinaBaseAccountTimingStableV1::Timed {
                initial_minimum_balance,
                cliff_time,
                cliff_amount,
                vesting_period,
                vesting_increment,
            } => Timing::Timed {
                initial_minimum_balance: Balance::from_u64(initial_minimum_balance.as_u64()),
                cliff_time: Slot::from_u32(cliff_time.as_u32()),
                cliff_amount: Amount::from_u64(cliff_amount.as_u64()),
                vesting_period: Slot::from_u32(vesting_period.as_u32()),
                vesting_increment: Amount::from_u64(vesting_increment.as_u64()),
            },
        }
    }
}

impl From<&Timing> for MinaBaseAccountTimingStableV1 {
    fn from(timing: &Timing) -> Self {
        use mina_p2p_messages::v2::*;

        match timing {
            super::Timing::Untimed => MinaBaseAccountTimingStableV1::Untimed,
            super::Timing::Timed {
                initial_minimum_balance,
                cliff_time,
                cliff_amount,
                vesting_period,
                vesting_increment,
            } => MinaBaseAccountTimingStableV1::Timed {
                initial_minimum_balance: CurrencyBalanceStableV1(CurrencyAmountStableV1(
                    UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
                        (initial_minimum_balance.as_u64() as i64).into(),
                    ),
                )),
                cliff_time: UnsignedExtendedUInt32StableV1((cliff_time.as_u32() as i32).into()),
                cliff_amount: CurrencyAmountStableV1(
                    UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
                        (cliff_amount.as_u64() as i64).into(),
                    ),
                ),
                vesting_period: UnsignedExtendedUInt32StableV1(
                    (vesting_period.as_u32() as i32).into(),
                ),
                vesting_increment: CurrencyAmountStableV1(
                    UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
                        (vesting_increment.as_u64() as i64).into(),
                    ),
                ),
            },
        }
    }
}

impl From<Account> for mina_p2p_messages::v2::MinaBaseAccountBinableArgStableV2 {
    fn from(acc: Account) -> Self {
        use mina_p2p_messages::v2::*;

        Self {
            public_key: {
                let public_key: NonZeroCurvePointUncompressedStableV1 = acc.public_key.into();
                public_key.into()
            },
            token_id: {
                let token_id: MinaBaseAccountIdDigestStableV1 = acc.token_id.into();
                token_id.into()
            },
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
            token_symbol: MinaBaseSokMessageDigestStableV1(acc.token_symbol.as_bytes().into()),
            balance: CurrencyBalanceStableV1(CurrencyAmountStableV1(
                UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
                    (acc.balance.as_u64() as i64).into(),
                ),
            )),
            nonce: UnsignedExtendedUInt32StableV1((acc.nonce.as_u32() as i32).into()),
            receipt_chain_hash: MinaBaseReceiptChainHashStableV1(acc.receipt_chain_hash.0.into()),
            delegate: acc.delegate.map(|delegate| {
                let delegate: NonZeroCurvePointUncompressedStableV1 = delegate.into();
                delegate.into()
            }),
            voting_for: DataHashLibStateHashStableV1(acc.voting_for.0.into()),
            timing: (&acc.timing).into(),
            permissions: (&acc.permissions).into(),
            zkapp: acc.zkapp.map(|zkapp| {
                let s = zkapp.app_state;
                let app_state = MinaBaseZkappStateValueStableV1(PaddedSeq(s.map(|v| v.into())));

                let verification_key = zkapp.verification_key.as_ref().map(|vk| vk.into());

                let seq = zkapp.sequence_state;
                let sequence_state = PaddedSeq(seq.map(|v| v.into()));

                MinaBaseZkappAccountStableV2 {
                    app_state,
                    verification_key,
                    zkapp_version: MinaNumbersNatMake32StableV1(UnsignedExtendedUInt32StableV1(
                        (zkapp.zkapp_version as i32).into(),
                    )),
                    sequence_state,
                    last_sequence_slot: UnsignedExtendedUInt32StableV1(
                        (zkapp.last_sequence_slot.as_u32() as i32).into(),
                    ),
                    proved_state: zkapp.proved_state,
                    zkapp_uri: zkapp.zkapp_uri.as_bytes().into(),
                }
            }),
        }
    }
}

impl From<&AuthRequired> for mina_p2p_messages::v2::MinaBasePermissionsAuthRequiredStableV2 {
    fn from(perms: &AuthRequired) -> Self {
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

impl From<&PlonkVerificationKeyEvals>
    for mina_p2p_messages::v2::MinaBaseVerificationKeyWireStableV1WrapIndex
{
    fn from(vk: &PlonkVerificationKeyEvals) -> Self {
        let sigma = PaddedSeq(array_into(&vk.sigma));
        let coef = PaddedSeq(array_into(&vk.coefficients));

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

impl From<AccountId> for mina_p2p_messages::v2::MinaBaseAccountIdStableV2 {
    fn from(account_id: AccountId) -> Self {
        let public_key: NonZeroCurvePointUncompressedStableV1 = account_id.public_key.into();
        Self(public_key.into(), account_id.token_id.into())
    }
}

impl From<mina_p2p_messages::v2::MinaBaseAccountIdStableV2> for AccountId {
    fn from(account_id: mina_p2p_messages::v2::MinaBaseAccountIdStableV2) -> Self {
        Self {
            public_key: account_id.0.into_inner().into(),
            token_id: account_id.1.into(),
        }
    }
}

impl From<&mina_p2p_messages::v2::MinaBaseAccountIdStableV2> for AccountId {
    fn from(account_id: &mina_p2p_messages::v2::MinaBaseAccountIdStableV2) -> Self {
        Self {
            public_key: account_id.0.clone().into_inner().into(),
            token_id: account_id.1.clone().into(),
        }
    }
}

impl From<TokenId> for mina_p2p_messages::v2::MinaBaseAccountIdDigestStableV1 {
    fn from(token_id: TokenId) -> Self {
        Self(token_id.0.into())
    }
}

impl From<mina_p2p_messages::v2::MinaBaseAccountIdDigestStableV1> for TokenId {
    fn from(token_id: mina_p2p_messages::v2::MinaBaseAccountIdDigestStableV1) -> Self {
        Self(token_id.0.into())
    }
}

impl From<&mina_p2p_messages::v2::MinaBaseAccountIdDigestStableV1> for TokenId {
    fn from(token_id: &mina_p2p_messages::v2::MinaBaseAccountIdDigestStableV1) -> Self {
        Self(token_id.0.to_field())
    }
}

impl From<&TokenId> for mina_p2p_messages::v2::MinaBaseTokenIdStableV2 {
    fn from(token_id: &TokenId) -> Self {
        Self(MinaBaseAccountIdDigestStableV1(token_id.0.into()))
    }
}

impl From<&mina_p2p_messages::v2::MinaBaseTokenIdStableV2> for TokenId {
    fn from(token_id: &mina_p2p_messages::v2::MinaBaseTokenIdStableV2) -> Self {
        Self((&token_id.0 .0).into())
    }
}

impl binprot::BinProtRead for Account {
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        let account = MinaBaseAccountBinableArgStableV2::binprot_read(r)?;
        Ok(account.into())
    }
}

impl binprot::BinProtWrite for Account {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let account: MinaBaseAccountBinableArgStableV2 = self.clone().into();
        account.binprot_write(w)?;
        Ok(())
    }
}

impl From<&MinaBasePermissionsAuthRequiredStableV2> for AuthRequired {
    fn from(auth: &MinaBasePermissionsAuthRequiredStableV2) -> Self {
        match auth {
            MinaBasePermissionsAuthRequiredStableV2::None => Self::None,
            MinaBasePermissionsAuthRequiredStableV2::Either => Self::Either,
            MinaBasePermissionsAuthRequiredStableV2::Proof => Self::Proof,
            MinaBasePermissionsAuthRequiredStableV2::Signature => Self::Signature,
            MinaBasePermissionsAuthRequiredStableV2::Impossible => Self::Impossible,
        }
    }
}

impl From<&MinaBasePermissionsStableV2> for Permissions<AuthRequired> {
    fn from(perms: &MinaBasePermissionsStableV2) -> Self {
        Permissions {
            edit_state: (&perms.edit_state).into(),
            send: (&perms.send).into(),
            receive: (&perms.receive).into(),
            set_delegate: (&perms.set_delegate).into(),
            set_permissions: (&perms.set_permissions).into(),
            set_verification_key: (&perms.set_verification_key).into(),
            set_zkapp_uri: (&perms.set_zkapp_uri).into(),
            edit_sequence_state: (&perms.edit_sequence_state).into(),
            set_token_symbol: (&perms.set_token_symbol).into(),
            increment_nonce: (&perms.increment_nonce).into(),
            set_voting_for: (&perms.set_voting_for).into(),
        }
    }
}

impl From<&Permissions<AuthRequired>> for MinaBasePermissionsStableV2 {
    fn from(perms: &Permissions<AuthRequired>) -> Self {
        MinaBasePermissionsStableV2 {
            edit_state: (&perms.edit_state).into(),
            send: (&perms.send).into(),
            receive: (&perms.receive).into(),
            set_delegate: (&perms.set_delegate).into(),
            set_permissions: (&perms.set_permissions).into(),
            set_verification_key: (&perms.set_verification_key).into(),
            set_zkapp_uri: (&perms.set_zkapp_uri).into(),
            edit_sequence_state: (&perms.edit_sequence_state).into(),
            set_token_symbol: (&perms.set_token_symbol).into(),
            increment_nonce: (&perms.increment_nonce).into(),
            set_voting_for: (&perms.set_voting_for).into(),
        }
    }
}

impl From<MinaBaseAccountBinableArgStableV2> for Account {
    fn from(acc: MinaBaseAccountBinableArgStableV2) -> Self {
        Self {
            public_key: acc.public_key.into_inner().into(),
            token_id: acc.token_id.into_inner().into(),
            token_permissions: match acc.token_permissions {
                MinaBaseTokenPermissionsStableV1::TokenOwned {
                    disable_new_accounts,
                } => TokenPermissions::TokenOwned {
                    disable_new_accounts,
                },
                MinaBaseTokenPermissionsStableV1::NotOwned { account_disabled } => {
                    TokenPermissions::NotOwned { account_disabled }
                }
            },
            token_symbol: acc.token_symbol.0.try_into().unwrap(),
            balance: Balance::from_u64(acc.balance.0 .0 .0 .0 as u64),
            nonce: Nonce::from_u32(acc.nonce.0 .0 as u32),
            receipt_chain_hash: ReceiptChainHash(acc.receipt_chain_hash.0.into()),
            delegate: acc.delegate.map(|d| d.into_inner().into()),
            voting_for: VotingFor(acc.voting_for.0.into()),
            timing: (&acc.timing).into(),
            permissions: (&acc.permissions).into(),
            zkapp: acc.zkapp.map(|zkapp| {
                let app_state = zkapp.app_state.0 .0.map(|v| v.into());

                ZkAppAccount {
                    app_state,
                    verification_key: zkapp.verification_key.map(|vk| (&vk).into()),
                    zkapp_version: zkapp.zkapp_version.0 .0 .0 as u32,
                    sequence_state: zkapp.sequence_state.0.map(|v| v.into()),
                    last_sequence_slot: Slot::from_u32(zkapp.last_sequence_slot.as_u32()),
                    proved_state: zkapp.proved_state,
                    zkapp_uri: zkapp.zkapp_uri.try_into().unwrap(),
                }
            }),
        }
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

        elog!("len={:?}", bytes.len());
        let result: Account = Account::deserialize(bytes);
        elog!("account={:#?}", result);
    }
}
