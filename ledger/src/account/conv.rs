#![allow(clippy::type_complexity)]

use ark_ec::short_weierstrass_jacobian::GroupAffine;
use ark_ff::Field;
use mina_hasher::Fp;
use mina_p2p_messages::{
    bigint::BigInt,
    binprot,
    pseq::PaddedSeq,
    v2::{
        MinaBaseAccountBinableArgStableV2, MinaBaseAccountIdDigestStableV1,
        MinaBaseAccountIdStableV2, MinaBaseAccountTimingStableV2,
        MinaBasePermissionsAuthRequiredStableV2, MinaBasePermissionsStableV2,
        MinaBaseVerificationKeyWireStableV1, MinaBaseVerificationKeyWireStableV1WrapIndex,
        NonZeroCurvePointUncompressedStableV1, PicklesBaseProofsVerifiedStableV1, TokenIdKeyHash,
    },
};

use crate::{
    proofs::{
        field::FieldWitness,
        transaction::{make_group, InnerCurve, PlonkVerificationKeyEvals},
    },
    scan_state::currency::{Amount, Balance, Nonce, Slot, SlotSpan},
    Permissions, ProofVerified, ReceiptChainHash, Timing, TokenSymbol, VerificationKey, VotingFor,
    ZkAppAccount,
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

impl From<TokenId> for TokenIdKeyHash {
    fn from(value: TokenId) -> Self {
        MinaBaseAccountIdDigestStableV1(value.0.into()).into()
    }
}

impl<F: FieldWitness> From<(BigInt, BigInt)> for InnerCurve<F>
where
    F: Field + From<BigInt>,
{
    fn from((x, y): (BigInt, BigInt)) -> Self {
        Self::of_affine(make_group::<F>(x.to_field(), y.to_field()))
    }
}

impl<F: FieldWitness> From<InnerCurve<F>> for (BigInt, BigInt)
where
    F: Field + Into<BigInt>,
{
    fn from(fps: InnerCurve<F>) -> Self {
        let GroupAffine { x, y, .. } = fps.to_affine();
        (x.into(), y.into())
    }
}

impl<F: FieldWitness> From<&(BigInt, BigInt)> for InnerCurve<F>
where
    F: Field + From<BigInt>,
{
    fn from((x, y): &(BigInt, BigInt)) -> Self {
        Self::of_affine(make_group::<F>(x.to_field(), y.to_field()))
    }
}

impl<F: FieldWitness> From<&InnerCurve<F>> for (BigInt, BigInt)
where
    F: Field + Into<BigInt>,
{
    fn from(fps: &InnerCurve<F>) -> Self {
        let GroupAffine { x, y, .. } = fps.to_affine();
        (x.into(), y.into())
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
        let MinaBaseVerificationKeyWireStableV1 {
            max_proofs_verified,
            actual_wrap_domain_size,
            wrap_index:
                MinaBaseVerificationKeyWireStableV1WrapIndex {
                    sigma_comm,
                    coefficients_comm,
                    generic_comm,
                    psm_comm,
                    complete_add_comm,
                    mul_comm,
                    emul_comm,
                    endomul_scalar_comm,
                },
        } = vk;

        let sigma = array_into(sigma_comm);
        let coefficients = array_into(coefficients_comm);

        VerificationKey {
            max_proofs_verified: match max_proofs_verified {
                PicklesBaseProofsVerifiedStableV1::N0 => ProofVerified::N0,
                PicklesBaseProofsVerifiedStableV1::N1 => ProofVerified::N1,
                PicklesBaseProofsVerifiedStableV1::N2 => ProofVerified::N2,
            },
            actual_wrap_domain_size: match actual_wrap_domain_size {
                PicklesBaseProofsVerifiedStableV1::N0 => ProofVerified::N0,
                PicklesBaseProofsVerifiedStableV1::N1 => ProofVerified::N1,
                PicklesBaseProofsVerifiedStableV1::N2 => ProofVerified::N2,
            },
            wrap_index: PlonkVerificationKeyEvals {
                sigma,
                coefficients,
                generic: generic_comm.into(),
                psm: psm_comm.into(),
                complete_add: complete_add_comm.into(),
                mul: mul_comm.into(),
                emul: emul_comm.into(),
                endomul_scalar: endomul_scalar_comm.into(),
            },
            wrap_vk: None,
        }
    }
}

impl From<&VerificationKey> for MinaBaseVerificationKeyWireStableV1 {
    fn from(vk: &VerificationKey) -> Self {
        let VerificationKey {
            max_proofs_verified,
            actual_wrap_domain_size,
            wrap_index,
            wrap_vk: _, // Unused
        } = vk;

        Self {
            max_proofs_verified: match max_proofs_verified {
                super::ProofVerified::N0 => PicklesBaseProofsVerifiedStableV1::N0,
                super::ProofVerified::N1 => PicklesBaseProofsVerifiedStableV1::N1,
                super::ProofVerified::N2 => PicklesBaseProofsVerifiedStableV1::N2,
            },
            actual_wrap_domain_size: match actual_wrap_domain_size {
                super::ProofVerified::N0 => PicklesBaseProofsVerifiedStableV1::N0,
                super::ProofVerified::N1 => PicklesBaseProofsVerifiedStableV1::N1,
                super::ProofVerified::N2 => PicklesBaseProofsVerifiedStableV1::N2,
            },
            wrap_index: wrap_index.into(),
        }
    }
}

impl From<&MinaBaseAccountTimingStableV2> for Timing {
    fn from(timing: &MinaBaseAccountTimingStableV2) -> Self {
        match timing {
            MinaBaseAccountTimingStableV2::Untimed => Timing::Untimed,
            MinaBaseAccountTimingStableV2::Timed {
                initial_minimum_balance,
                cliff_time,
                cliff_amount,
                vesting_period,
                vesting_increment,
            } => Timing::Timed {
                initial_minimum_balance: Balance::from_u64(initial_minimum_balance.as_u64()),
                cliff_time: Slot::from_u32(cliff_time.as_u32()),
                cliff_amount: Amount::from_u64(cliff_amount.as_u64()),
                vesting_period: SlotSpan::from_u32(vesting_period.as_u32()),
                vesting_increment: Amount::from_u64(vesting_increment.as_u64()),
            },
        }
    }
}

impl From<&Timing> for MinaBaseAccountTimingStableV2 {
    fn from(timing: &Timing) -> Self {
        use mina_p2p_messages::v2::*;

        match timing {
            super::Timing::Untimed => MinaBaseAccountTimingStableV2::Untimed,
            super::Timing::Timed {
                initial_minimum_balance,
                cliff_time,
                cliff_amount,
                vesting_period,
                vesting_increment,
            } => MinaBaseAccountTimingStableV2::Timed {
                initial_minimum_balance: CurrencyBalanceStableV1(CurrencyAmountStableV1(
                    UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
                        initial_minimum_balance.as_u64().into(),
                    ),
                )),
                cliff_time: cliff_time.into(),
                cliff_amount: CurrencyAmountStableV1(
                    UnsignedExtendedUInt64Int64ForVersionTagsStableV1(cliff_amount.as_u64().into()),
                ),
                vesting_period: vesting_period.into(),
                vesting_increment: CurrencyAmountStableV1(
                    UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
                        vesting_increment.as_u64().into(),
                    ),
                ),
            },
        }
    }
}

impl From<&Account> for mina_p2p_messages::v2::MinaBaseAccountBinableArgStableV2 {
    fn from(acc: &Account) -> Self {
        use mina_p2p_messages::v2::*;

        Self {
            public_key: (&acc.public_key).into(),
            token_id: (&acc.token_id).into(),
            token_symbol: MinaBaseZkappAccountZkappUriStableV1(acc.token_symbol.as_bytes().into()),
            balance: CurrencyBalanceStableV1(CurrencyAmountStableV1(
                UnsignedExtendedUInt64Int64ForVersionTagsStableV1(acc.balance.as_u64().into()),
            )),
            nonce: UnsignedExtendedUInt32StableV1(acc.nonce.as_u32().into()),
            receipt_chain_hash: MinaBaseReceiptChainHashStableV1(acc.receipt_chain_hash.0.into()),
            delegate: acc.delegate.as_ref().map(|delegate| {
                let delegate: NonZeroCurvePointUncompressedStableV1 = delegate.into();
                delegate.into()
            }),
            voting_for: DataHashLibStateHashStableV1(acc.voting_for.0.into()).into(),
            timing: (&acc.timing).into(),
            permissions: (&acc.permissions).into(),
            zkapp: acc.zkapp.as_ref().map(|zkapp| {
                let s = zkapp.app_state;
                let app_state = MinaBaseZkappStateValueStableV1(PaddedSeq(s.map(|v| v.into())));

                let verification_key = zkapp.verification_key.as_ref().map(|vk| vk.into());

                let seq = zkapp.action_state;
                let action_state = PaddedSeq(seq.map(|v| v.into()));

                MinaBaseZkappAccountStableV2 {
                    app_state,
                    verification_key,
                    zkapp_version: MinaNumbersNatMake32StableV1(UnsignedExtendedUInt32StableV1(
                        zkapp.zkapp_version.into(),
                    )),
                    action_state,
                    last_action_slot: (&zkapp.last_action_slot).into(),
                    proved_state: zkapp.proved_state,
                    zkapp_uri: MinaBaseZkappAccountZkappUriStableV1(
                        zkapp.zkapp_uri.as_bytes().into(),
                    ),
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

impl From<&PlonkVerificationKeyEvals<Fp>>
    for mina_p2p_messages::v2::MinaBaseVerificationKeyWireStableV1WrapIndex
{
    fn from(vk: &PlonkVerificationKeyEvals<Fp>) -> Self {
        let sigma = PaddedSeq(array_into(&vk.sigma));
        let coef = PaddedSeq(array_into(&vk.coefficients));

        Self {
            sigma_comm: sigma,
            coefficients_comm: coef,
            generic_comm: (&vk.generic).into(),
            psm_comm: (&vk.psm).into(),
            complete_add_comm: (&vk.complete_add).into(),
            mul_comm: (&vk.mul).into(),
            emul_comm: (&vk.emul).into(),
            endomul_scalar_comm: (&vk.endomul_scalar).into(),
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
        Ok((&account).into())
    }
}

impl binprot::BinProtWrite for Account {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let account: MinaBaseAccountBinableArgStableV2 = self.into();
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
        } = perms;

        Permissions {
            edit_state: edit_state.into(),
            send: send.into(),
            receive: receive.into(),
            set_delegate: set_delegate.into(),
            set_permissions: set_permissions.into(),
            set_verification_key: set_verification_key.into(),
            set_zkapp_uri: set_zkapp_uri.into(),
            edit_action_state: edit_action_state.into(),
            set_token_symbol: set_token_symbol.into(),
            increment_nonce: increment_nonce.into(),
            set_voting_for: set_voting_for.into(),
            access: access.into(),
            set_timing: set_timing.into(),
        }
    }
}

impl From<&Permissions<AuthRequired>> for MinaBasePermissionsStableV2 {
    fn from(perms: &Permissions<AuthRequired>) -> Self {
        let Permissions {
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
        } = perms;

        MinaBasePermissionsStableV2 {
            edit_state: edit_state.into(),
            send: send.into(),
            receive: receive.into(),
            set_delegate: set_delegate.into(),
            set_permissions: set_permissions.into(),
            set_verification_key: set_verification_key.into(),
            set_zkapp_uri: set_zkapp_uri.into(),
            edit_action_state: edit_action_state.into(),
            set_token_symbol: set_token_symbol.into(),
            increment_nonce: increment_nonce.into(),
            set_voting_for: set_voting_for.into(),
            access: access.into(),
            set_timing: set_timing.into(),
        }
    }
}

impl From<&MinaBaseAccountBinableArgStableV2> for Account {
    fn from(acc: &MinaBaseAccountBinableArgStableV2) -> Self {
        Self {
            public_key: acc.public_key.inner().into(),
            token_id: acc.token_id.inner().into(),
            token_symbol: {
                let s: String = (&acc.token_symbol.0).try_into().unwrap();
                TokenSymbol::from(s)
            },
            balance: Balance::from_u64(acc.balance.0 .0 .0 .0 as u64),
            nonce: Nonce::from_u32(acc.nonce.0 .0 as u32),
            receipt_chain_hash: ReceiptChainHash((&acc.receipt_chain_hash.0).into()),
            delegate: acc.delegate.as_ref().map(|d| d.inner().into()),
            voting_for: VotingFor(acc.voting_for.0.to_field()),
            timing: (&acc.timing).into(),
            permissions: (&acc.permissions).into(),
            zkapp: acc.zkapp.as_ref().map(|zkapp| {
                let app_state = std::array::from_fn(|i| zkapp.app_state[i].to_field());

                ZkAppAccount {
                    app_state,
                    verification_key: zkapp.verification_key.as_ref().map(Into::into),
                    zkapp_version: zkapp.zkapp_version.as_u32(),
                    action_state: std::array::from_fn(|i| zkapp.action_state[i].to_field()),
                    last_action_slot: Slot::from_u32(zkapp.last_action_slot.as_u32()),
                    proved_state: zkapp.proved_state,
                    zkapp_uri: (&zkapp.zkapp_uri.0).try_into().unwrap(),
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
        let account = Account::rand();
        let hash = account.hash();

        let bytes = account.serialize();

        elog!("len={:?}", bytes.len());
        let result: Account = Account::deserialize(&bytes);
        elog!("account={:#?}", result);

        assert_eq!(hash, result.hash());
    }
}
