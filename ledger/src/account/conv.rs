#![allow(clippy::type_complexity)]

use ark_ec::short_weierstrass_jacobian::GroupAffine;
use ark_ff::{fields::arithmetic::InvalidBigInt, Field, PrimeField};
use mina_hasher::Fp;
use mina_p2p_messages::{
    bigint::BigInt,
    binprot,
    pseq::PaddedSeq,
    v2::{
        self, MinaBaseAccountBinableArgStableV2, MinaBaseAccountIdDigestStableV1,
        MinaBaseAccountIdStableV2, MinaBaseAccountIndexStableV1, MinaBaseAccountTimingStableV2,
        MinaBasePermissionsAuthRequiredStableV2, MinaBasePermissionsStableV2,
        MinaBaseReceiptChainHashStableV1, MinaBaseVerificationKeyWireStableV1,
        MinaBaseVerificationKeyWireStableV1WrapIndex, NonZeroCurvePointUncompressedStableV1,
        PicklesBaseProofsVerifiedStableV1, TokenIdKeyHash,
    },
};

use crate::{
    proofs::{
        field::FieldWitness,
        transaction::{make_group, InnerCurve, PlonkVerificationKeyEvals},
    },
    scan_state::currency::{Amount, Balance, Nonce, Slot, SlotSpan, TxnVersion},
    AccountIndex, Permissions, ProofVerified, ReceiptChainHash, SetVerificationKey, Timing,
    TokenSymbol, VerificationKey, VotingFor, ZkAppAccount,
};

use super::{Account, AccountId, AuthRequired, TokenId, VerificationKeyWire};

impl binprot::BinProtRead for TokenId {
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        let token_id = TokenIdKeyHash::binprot_read(r)?;
        let token_id: MinaBaseAccountIdDigestStableV1 = token_id.into_inner();
        token_id
            .try_into()
            .map_err(|e| binprot::Error::CustomError(Box::new(e)))
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

impl From<TokenIdKeyHash> for TokenId {
    fn from(value: TokenIdKeyHash) -> Self {
        value.inner().try_into().unwrap()
    }
}

impl<F: FieldWitness> TryFrom<(BigInt, BigInt)> for InnerCurve<F>
where
    F: Field + From<BigInt>,
{
    type Error = InvalidBigInt;

    fn try_from((x, y): (BigInt, BigInt)) -> Result<Self, Self::Error> {
        Ok(Self::of_affine(make_group::<F>(
            x.to_field()?,
            y.to_field()?,
        )))
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

impl<F: FieldWitness> TryFrom<&(BigInt, BigInt)> for InnerCurve<F>
where
    F: Field,
{
    type Error = InvalidBigInt;

    fn try_from((x, y): &(BigInt, BigInt)) -> Result<Self, Self::Error> {
        Ok(Self::of_affine(make_group::<F>(
            x.to_field()?,
            y.to_field()?,
        )))
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
        account_id
            .try_into()
            .map_err(|e| binprot::Error::CustomError(Box::new(e)))
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
    value.each_ref().map(|value| U::from(value))
}

pub fn array_into_with<'a, T, U, F, const N: usize>(value: &'a [T; N], fun: F) -> [U; N]
where
    T: 'a,
    F: Fn(&T) -> U,
{
    value.each_ref().map(fun)
}

/// Note: Refactor when `core::array::try_map` is stable
/// https://github.com/rust-lang/rust/issues/79711
pub fn try_array_into_with<'a, T, E, U, F, const N: usize>(
    value: &'a [T; N],
    fun: F,
) -> Result<[U; N], E>
where
    T: 'a,
    F: Fn(&T) -> Result<U, E>,
    U: std::fmt::Debug,
{
    Ok(value
        .iter()
        .map(fun)
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .unwrap()) // Never fail: `value` contains `N` elements
}

impl TryFrom<&MinaBaseVerificationKeyWireStableV1> for VerificationKey {
    type Error = InvalidBigInt;

    fn try_from(vk: &MinaBaseVerificationKeyWireStableV1) -> Result<Self, Self::Error> {
        let MinaBaseVerificationKeyWireStableV1 {
            max_proofs_verified,
            actual_wrap_domain_size,
            wrap_index,
        } = vk;

        Ok(VerificationKey {
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
            wrap_index: Box::new(wrap_index.try_into()?),
            wrap_vk: None,
        })
    }
}

impl TryFrom<&MinaBaseVerificationKeyWireStableV1WrapIndex> for PlonkVerificationKeyEvals<Fp> {
    type Error = InvalidBigInt;

    fn try_from(value: &MinaBaseVerificationKeyWireStableV1WrapIndex) -> Result<Self, Self::Error> {
        let MinaBaseVerificationKeyWireStableV1WrapIndex {
            sigma_comm,
            coefficients_comm,
            generic_comm,
            psm_comm,
            complete_add_comm,
            mul_comm,
            emul_comm,
            endomul_scalar_comm,
        } = value;

        let sigma = try_array_into_with(sigma_comm, |s| s.try_into())?;
        let coefficients = try_array_into_with(coefficients_comm, |s| s.try_into())?;

        Ok(PlonkVerificationKeyEvals {
            sigma,
            coefficients,
            generic: generic_comm.try_into()?,
            psm: psm_comm.try_into()?,
            complete_add: complete_add_comm.try_into()?,
            mul: mul_comm.try_into()?,
            emul: emul_comm.try_into()?,
            endomul_scalar: endomul_scalar_comm.try_into()?,
        })
    }
}
impl TryFrom<MinaBaseVerificationKeyWireStableV1WrapIndex> for PlonkVerificationKeyEvals<Fp> {
    type Error = InvalidBigInt;

    fn try_from(value: MinaBaseVerificationKeyWireStableV1WrapIndex) -> Result<Self, Self::Error> {
        (&value).try_into()
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
            wrap_index: wrap_index.as_ref().into(),
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
            token_symbol: acc.token_symbol.as_bytes().into(),
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

                let verification_key = zkapp.verification_key.as_ref().map(|vk| vk.vk().into());

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
                    zkapp_uri: (&zkapp.zkapp_uri).into(),
                }
            }),
        }
    }
}
impl From<Account> for mina_p2p_messages::v2::MinaBaseAccountBinableArgStableV2 {
    fn from(account: Account) -> Self {
        (&account).into()
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
impl From<PlonkVerificationKeyEvals<Fp>>
    for mina_p2p_messages::v2::MinaBaseVerificationKeyWireStableV1WrapIndex
{
    fn from(value: PlonkVerificationKeyEvals<Fp>) -> Self {
        (&value).into()
    }
}

// // Following types were written manually

impl From<AccountId> for mina_p2p_messages::v2::MinaBaseAccountIdStableV2 {
    fn from(account_id: AccountId) -> Self {
        let public_key: NonZeroCurvePointUncompressedStableV1 = account_id.public_key.into();
        Self(public_key.into(), account_id.token_id.into())
    }
}

impl TryFrom<mina_p2p_messages::v2::MinaBaseAccountIdStableV2> for AccountId {
    type Error = InvalidBigInt;

    fn try_from(
        account_id: mina_p2p_messages::v2::MinaBaseAccountIdStableV2,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            public_key: account_id.0.into_inner().try_into()?,
            token_id: account_id.1.try_into()?,
        })
    }
}

impl TryFrom<&mina_p2p_messages::v2::MinaBaseAccountIdStableV2> for AccountId {
    type Error = InvalidBigInt;

    fn try_from(
        account_id: &mina_p2p_messages::v2::MinaBaseAccountIdStableV2,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            public_key: account_id.0.clone().into_inner().try_into()?,
            token_id: account_id.1.clone().try_into()?,
        })
    }
}

impl From<TokenId> for mina_p2p_messages::v2::MinaBaseAccountIdDigestStableV1 {
    fn from(token_id: TokenId) -> Self {
        Self(token_id.0.into())
    }
}

impl TryFrom<mina_p2p_messages::v2::MinaBaseAccountIdDigestStableV1> for TokenId {
    type Error = InvalidBigInt;

    fn try_from(
        token_id: mina_p2p_messages::v2::MinaBaseAccountIdDigestStableV1,
    ) -> Result<Self, Self::Error> {
        Ok(Self(token_id.0.try_into()?))
    }
}

impl TryFrom<&mina_p2p_messages::v2::MinaBaseAccountIdDigestStableV1> for TokenId {
    type Error = InvalidBigInt;

    fn try_from(
        token_id: &mina_p2p_messages::v2::MinaBaseAccountIdDigestStableV1,
    ) -> Result<Self, Self::Error> {
        Ok(Self(token_id.0.to_field()?))
    }
}

impl From<&TokenId> for mina_p2p_messages::v2::MinaBaseTokenIdStableV2 {
    fn from(token_id: &TokenId) -> Self {
        Self(MinaBaseAccountIdDigestStableV1(token_id.0.into()))
    }
}

impl From<TokenId> for mina_p2p_messages::v2::MinaBaseTokenIdStableV2 {
    fn from(token_id: TokenId) -> Self {
        Self(MinaBaseAccountIdDigestStableV1(token_id.0.into()))
    }
}

impl TryFrom<&mina_p2p_messages::v2::MinaBaseTokenIdStableV2> for TokenId {
    type Error = InvalidBigInt;

    fn try_from(
        token_id: &mina_p2p_messages::v2::MinaBaseTokenIdStableV2,
    ) -> Result<Self, Self::Error> {
        Ok(Self(token_id.to_field()?))
    }
}

impl TryFrom<mina_p2p_messages::v2::MinaBaseTokenIdStableV2> for TokenId {
    type Error = InvalidBigInt;

    fn try_from(
        token_id: mina_p2p_messages::v2::MinaBaseTokenIdStableV2,
    ) -> Result<Self, Self::Error> {
        Ok(Self(token_id.to_field()?))
    }
}

impl binprot::BinProtRead for Account {
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        let account = MinaBaseAccountBinableArgStableV2::binprot_read(r)?;
        (&account)
            .try_into()
            .map_err(|e| binprot::Error::CustomError(Box::new(e)))
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
            set_verification_key: (auth, txn_version),
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
            set_verification_key: SetVerificationKey {
                auth: auth.into(),
                txn_version: TxnVersion::from_u32(txn_version.as_u32()),
            },
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
            set_verification_key: SetVerificationKey { auth, txn_version },
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
            set_verification_key: (auth.into(), txn_version.as_u32().into()),
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

impl TryFrom<&MinaBaseAccountBinableArgStableV2> for Account {
    type Error = InvalidBigInt;

    fn try_from(acc: &MinaBaseAccountBinableArgStableV2) -> Result<Self, Self::Error> {
        Ok(Self {
            public_key: acc.public_key.inner().try_into()?,
            token_id: acc.token_id.inner().try_into()?,
            token_symbol: {
                let s = acc.token_symbol.0.clone();
                TokenSymbol::from(s)
            },
            balance: Balance::from_u64(acc.balance.0 .0 .0 .0),
            nonce: Nonce::from_u32(acc.nonce.0 .0),
            receipt_chain_hash: ReceiptChainHash(acc.receipt_chain_hash.to_field()?),
            delegate: match acc.delegate.as_ref() {
                Some(delegate) => Some(delegate.try_into()?),
                None => None,
            },
            voting_for: VotingFor(acc.voting_for.to_field()?),
            timing: (&acc.timing).into(),
            permissions: (&acc.permissions).into(),
            zkapp: match acc.zkapp.as_ref() {
                Some(zkapp) => {
                    let v2::MinaBaseZkappAccountStableV2 {
                        app_state,
                        verification_key,
                        zkapp_version,
                        action_state,
                        last_action_slot,
                        proved_state,
                        zkapp_uri,
                    } = zkapp;

                    Some(Box::new(ZkAppAccount {
                        app_state: try_array_into_with(app_state, BigInt::to_field)?,
                        verification_key: match verification_key.as_ref() {
                            Some(vk) => Some(VerificationKeyWire::new(vk.try_into()?)),
                            None => None,
                        },
                        zkapp_version: zkapp_version.as_u32(),
                        action_state: try_array_into_with(action_state, BigInt::to_field)?,
                        last_action_slot: Slot::from_u32(last_action_slot.as_u32()),
                        proved_state: *proved_state,
                        zkapp_uri: zkapp_uri.into(),
                    }))
                }
                None => None,
            },
        })
    }
}
impl TryFrom<MinaBaseAccountBinableArgStableV2> for Account {
    type Error = InvalidBigInt;

    fn try_from(account: MinaBaseAccountBinableArgStableV2) -> Result<Self, Self::Error> {
        (&account).try_into()
    }
}

impl From<AccountIndex> for MinaBaseAccountIndexStableV1 {
    fn from(value: AccountIndex) -> Self {
        Self(value.as_u64().into())
    }
}

impl From<ReceiptChainHash> for mina_p2p_messages::v2::ReceiptChainHash {
    fn from(value: ReceiptChainHash) -> Self {
        MinaBaseReceiptChainHashStableV1(value.0.into_repr().into()).into()
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
