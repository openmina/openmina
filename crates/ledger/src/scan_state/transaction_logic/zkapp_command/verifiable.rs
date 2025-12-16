use mina_curves::pasta::Fp;
use mina_p2p_messages::v2::MinaBaseZkappCommandVerifiableStableV1;
use std::collections::HashMap;

use super::{
    AccountId, AccountUpdate, AuthorizationKind, CallForest, Control, FeePayer, Memo, SetOrKeep,
    VerificationKeyWire,
};
use crate::sparse_ledger::LedgerIntf;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(try_from = "MinaBaseZkappCommandVerifiableStableV1")]
#[serde(into = "MinaBaseZkappCommandVerifiableStableV1")]
pub struct ZkAppCommand {
    pub fee_payer: FeePayer,
    pub account_updates: CallForest<(AccountUpdate, Option<VerificationKeyWire>)>,
    pub memo: Memo,
}

fn ok_if_vk_hash_expected(
    got: VerificationKeyWire,
    expected: Fp,
) -> Result<VerificationKeyWire, String> {
    if got.hash() == expected {
        return Ok(got.clone());
    }
    Err(format!(
        "Expected vk hash doesn't match hash in vk we received\
                 expected: {:?}\
                 got: {:?}",
        expected, got
    ))
}

pub fn find_vk_via_ledger<L>(
    ledger: L,
    expected_vk_hash: Fp,
    account_id: &AccountId,
) -> Result<VerificationKeyWire, String>
where
    L: LedgerIntf + Clone,
{
    let vk = ledger
        .location_of_account(account_id)
        .and_then(|location| ledger.get(&location))
        .and_then(|account| {
            account
                .zkapp
                .as_ref()
                .and_then(|zkapp| zkapp.verification_key.clone())
        });

    match vk {
        Some(vk) => ok_if_vk_hash_expected(vk, expected_vk_hash),
        None => Err(format!(
            "No verification key found for proved account update\
                             account_id: {:?}",
            account_id
        )),
    }
}

fn check_authorization(p: &AccountUpdate) -> Result<(), String> {
    use AuthorizationKind as AK;
    use Control as C;

    match (&p.authorization, &p.body.authorization_kind) {
        (C::NoneGiven, AK::NoneGiven)
        | (C::Proof(_), AK::Proof(_))
        | (C::Signature(_), AK::Signature) => Ok(()),
        _ => Err(format!(
            "Authorization kind does not match the authorization\
                         expected={:#?}\
                         got={:#?}",
            p.body.authorization_kind, p.authorization
        )),
    }
}

/// Ensures that there's a verification_key available for all account_updates
/// and creates a valid command associating the correct keys with each
/// account_id.
///
/// If an account_update replaces the verification_key (or deletes it),
/// subsequent account_updates use the replaced key instead of looking in the
/// ledger for the key (ie set by a previous transaction).
pub fn create(
    zkapp: &super::ZkAppCommand,
    is_failed: bool,
    find_vk: impl Fn(Fp, &AccountId) -> Result<VerificationKeyWire, String>,
) -> Result<ZkAppCommand, String> {
    let super::ZkAppCommand {
        fee_payer,
        account_updates,
        memo,
    } = zkapp;

    let mut tbl = HashMap::with_capacity(128);
    // Keep track of the verification keys that have been set so far
    // during this transaction.
    let mut vks_overridden: HashMap<AccountId, Option<VerificationKeyWire>> =
        HashMap::with_capacity(128);

    let account_updates = account_updates.try_map_to(|p| {
        let account_id = p.account_id();

        check_authorization(p)?;

        let result = match (&p.body.authorization_kind, is_failed) {
            (AuthorizationKind::Proof(vk_hash), false) => {
                let prioritized_vk = {
                    // only lookup _past_ vk setting, ie exclude the new one we
                    // potentially set in this account_update (use the non-'
                    // vks_overrided) .

                    match vks_overridden.get(&account_id) {
                        Some(Some(vk)) => ok_if_vk_hash_expected(vk.clone(), *vk_hash)?,
                        Some(None) => {
                            // we explicitly have erased the key
                            return Err(format!(
                                "No verification key found for proved account \
                                                update: the verification key was removed by a \
                                                previous account update\
                                                account_id={:?}",
                                account_id
                            ));
                        }
                        None => {
                            // we haven't set anything; lookup the vk in the fallback
                            find_vk(*vk_hash, &account_id)?
                        }
                    }
                };

                tbl.insert(account_id, prioritized_vk.hash());

                Ok((p.clone(), Some(prioritized_vk)))
            }

            _ => Ok((p.clone(), None)),
        };

        // NOTE: we only update the overriden map AFTER verifying the update to make sure
        // that the verification for the VK update itself is done against the previous VK.
        if let SetOrKeep::Set(vk_next) = &p.body.update.verification_key {
            vks_overridden.insert(p.account_id().clone(), Some(vk_next.clone()));
        }

        result
    })?;

    Ok(ZkAppCommand {
        fee_payer: fee_payer.clone(),
        account_updates,
        memo: memo.clone(),
    })
}
