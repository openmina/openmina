//! Copied from https://github.com/o1-labs/proof-systems/blob/932fa7e6429f8160586d13efcaf72fccc3fc53ac/signer/tests/transaction.rs
//! since it's defined in test and not accessable for now.

use ledger::scan_state::transaction_logic::{signed_command, transaction_union_payload::TransactionUnionPayload, UserCommand};
use mina_hasher::{Hashable, ROInput};
use mina_p2p_messages::{
    bigint::BigInt,
    number::Number,
    string::CharString,
    v2::{
        CurrencyAmountStableV1, CurrencyFeeStableV1, MinaBasePaymentPayloadStableV2,
        MinaBaseSignatureStableV1, MinaBaseSignedCommandMemoStableV1,
        MinaBaseSignedCommandPayloadBodyStableV2, MinaBaseSignedCommandPayloadCommonStableV2,
        MinaBaseSignedCommandPayloadStableV2, MinaBaseSignedCommandStableV2,
        MinaBaseUserCommandStableV2, NonZeroCurvePoint, NonZeroCurvePointUncompressedStableV1,
        UnsignedExtendedUInt32StableV1, UnsignedExtendedUInt64Int64ForVersionTagsStableV1, MinaNumbersGlobalSlotSinceGenesisMStableV1,
    },
};
use mina_signer::{CompressedPubKey, NetworkId, PubKey, Signature, Keypair, Signer};

fn sign_payload(keypair: &Keypair, payload: &signed_command::SignedCommandPayload) -> mina_signer::Signature {
    let tx = TransactionUnionPayload::of_user_command_payload(payload);
    let mut signer = mina_signer::create_legacy(NetworkId::TESTNET);
    signer.sign(keypair, &tx)
}

fn new_signed_command(
    keypair: &Keypair,
    fee: ledger::scan_state::currency::Fee,
    fee_payer_pk: CompressedPubKey,
    nonce: ledger::scan_state::currency::Nonce,
    valid_until: Option<ledger::scan_state::currency::Slot>,
    memo: ledger::scan_state::transaction_logic::Memo,
    body: signed_command::Body,
) -> signed_command::SignedCommand {
    let payload = signed_command::SignedCommandPayload::create(fee, fee_payer_pk, nonce, valid_until, memo, body);
    let signature = sign_payload(keypair, &payload);

    signed_command::SignedCommand {
        payload,
        signer: keypair.public.into_compressed(),
        signature,
    }
}

fn new_payment(
    source_pk: CompressedPubKey,
    receiver_pk: CompressedPubKey,
    amount: ledger::scan_state::currency::Amount,
) -> signed_command::Body {
    let payload = signed_command::PaymentPayload {
        receiver_pk,
        amount,
    };
    signed_command::Body::Payment(payload)
}

pub fn new_signed_payment(
    keypair: &Keypair,
    fee: ledger::scan_state::currency::Fee,
    nonce: ledger::scan_state::currency::Nonce,
    valid_until: Option<ledger::scan_state::currency::Slot>,
    memo: ledger::scan_state::transaction_logic::Memo,
    receiver_pk: CompressedPubKey,
    amount: ledger::scan_state::currency::Amount,
) -> UserCommand {
    let body = new_payment(keypair.public.into_compressed(), receiver_pk, amount);
    let signed = new_signed_command(keypair, fee, keypair.public.into_compressed(), nonce, valid_until, memo, body);
    UserCommand::SignedCommand(signed.into())
}
