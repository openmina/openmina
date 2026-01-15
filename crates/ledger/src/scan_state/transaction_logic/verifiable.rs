use std::ops::Neg;

use ark_ff::{BigInteger, PrimeField};
use mina_signer::CompressedPubKey;

use super::{
    signed_command, transaction_union_payload::TransactionUnionPayload, valid, zkapp_command,
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum UserCommand {
    SignedCommand(Box<signed_command::SignedCommand>),
    ZkAppCommand(Box<zkapp_command::verifiable::ZkAppCommand>),
}

pub fn compressed_to_pubkey(pubkey: &CompressedPubKey) -> mina_signer::PubKey {
    // Taken from https://github.com/o1-labs/proof-systems/blob/e3fc04ce87f8695288de167115dea80050ab33f4/signer/src/pubkey.rs#L95-L106
    let mut pt =
        mina_signer::CurvePoint::get_point_from_x_unchecked(pubkey.x, pubkey.is_odd).unwrap();

    if pt.y.into_bigint().is_even() == pubkey.is_odd {
        pt.y = pt.y.neg();
    }

    assert!(pt.is_on_curve());

    // Safe now because we checked point pt is on curve
    mina_signer::PubKey::from_point_unsafe(pt)
}

/// <https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/signed_command.ml#L436>
pub fn check_only_for_signature(
    cmd: Box<signed_command::SignedCommand>,
) -> Result<valid::UserCommand, Box<signed_command::SignedCommand>> {
    // <https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/signed_command.ml#L396>

    let signed_command::SignedCommand {
        payload,
        signer: pubkey,
        signature,
    } = &*cmd;

    let payload = TransactionUnionPayload::of_user_command_payload(payload);
    let pubkey = compressed_to_pubkey(pubkey);

    if crate::verifier::common::legacy_verify_signature(signature, &pubkey, &payload) {
        Ok(valid::UserCommand::SignedCommand(cmd))
    } else {
        Err(cmd)
    }
}
