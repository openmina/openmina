//! Copied from https://github.com/o1-labs/proof-systems/blob/932fa7e6429f8160586d13efcaf72fccc3fc53ac/signer/tests/transaction.rs
//! since it's defined in test and not accessable for now.

use mina_hasher::{Hashable, ROInput};
use mina_p2p_messages::{
    bigint::BigInt,
    gossip::GossipNetMessageV2,
    number::Number,
    string::CharString,
    v2::{
        CurrencyAmountStableV1, CurrencyFeeStableV1, MinaBasePaymentPayloadStableV2,
        MinaBaseSignatureStableV1, MinaBaseSignedCommandMemoStableV1,
        MinaBaseSignedCommandPayloadBodyStableV2, MinaBaseSignedCommandPayloadCommonStableV2,
        MinaBaseSignedCommandPayloadStableV2, MinaBaseSignedCommandStableV2,
        MinaBaseUserCommandStableV2, NetworkPoolTransactionPoolDiffVersionedStableV2,
        NonZeroCurvePoint, NonZeroCurvePointUncompressedStableV1, UnsignedExtendedUInt32StableV1,
        UnsignedExtendedUInt64Int64ForVersionTagsStableV1,
    },
};
use mina_signer::{CompressedPubKey, NetworkId, PubKey, Signature};

const MEMO_BYTES: usize = 34;
const TAG_BITS: usize = 3;
const PAYMENT_TX_TAG: [bool; TAG_BITS] = [false, false, false];
const DELEGATION_TX_TAG: [bool; TAG_BITS] = [false, false, true];

#[derive(Clone)]
pub struct Transaction {
    // Common
    pub fee: u64,
    pub fee_token: u64,
    pub fee_payer_pk: CompressedPubKey,
    pub nonce: u32,
    pub valid_until: u32,
    pub memo: [u8; MEMO_BYTES],
    // Body
    pub tag: [bool; TAG_BITS],
    pub source_pk: CompressedPubKey,
    pub receiver_pk: CompressedPubKey,
    pub token_id: u64,
    pub amount: u64,
    pub token_locked: bool,
}

impl Hashable for Transaction {
    type D = NetworkId;

    fn to_roinput(&self) -> ROInput {
        let mut roi = ROInput::new()
            .append_field(self.fee_payer_pk.x)
            .append_field(self.source_pk.x)
            .append_field(self.receiver_pk.x)
            .append_u64(self.fee)
            .append_u64(self.fee_token)
            .append_bool(self.fee_payer_pk.is_odd)
            .append_u32(self.nonce)
            .append_u32(self.valid_until)
            .append_bytes(&self.memo);

        for tag_bit in self.tag {
            roi = roi.append_bool(tag_bit);
        }

        roi.append_bool(self.source_pk.is_odd)
            .append_bool(self.receiver_pk.is_odd)
            .append_u64(self.token_id)
            .append_u64(self.amount)
            .append_bool(self.token_locked)
    }

    fn domain_string(network_id: NetworkId) -> Option<String> {
        // Domain strings must have length <= 20
        match network_id {
            NetworkId::MAINNET => "MinaSignatureMainnet",
            NetworkId::TESTNET => "CodaSignature",
        }
        .to_string()
        .into()
    }
}

impl Transaction {
    pub fn new_payment(from: PubKey, to: PubKey, amount: u64, fee: u64, nonce: u32) -> Self {
        Transaction {
            fee: fee,
            fee_token: 1,
            fee_payer_pk: from.into_compressed(),
            nonce: nonce,
            // TODO(zura): was u32::MAX?
            valid_until: i32::MAX as u32,
            memo: std::array::from_fn(|i| (i == 0) as u8),
            tag: PAYMENT_TX_TAG,
            source_pk: from.into_compressed(),
            receiver_pk: to.into_compressed(),
            token_id: 1,
            amount: amount,
            token_locked: false,
        }
    }

    // pub fn new_delegation(from: PubKey, to: PubKey, fee: u64, nonce: u32) -> Self {
    //     Transaction {
    //         fee: fee,
    //         fee_token: 1,
    //         fee_payer_pk: from.into_compressed(),
    //         nonce: nonce,
    //         valid_until: u32::MAX,
    //         memo: std::array::from_fn(|i| (i == 0) as u8),
    //         tag: DELEGATION_TX_TAG,
    //         source_pk: from.into_compressed(),
    //         receiver_pk: to.into_compressed(),
    //         token_id: 1,
    //         amount: 0,
    //         token_locked: false,
    //     }
    // }

    pub fn set_valid_until(mut self, global_slot: u32) -> Self {
        self.valid_until = global_slot;

        self
    }

    pub fn set_memo(mut self, memo: [u8; MEMO_BYTES - 2]) -> Self {
        self.memo[0] = 0x01;
        self.memo[1] = (MEMO_BYTES - 2) as u8;
        self.memo[2..].copy_from_slice(&memo[..]);

        self
    }

    pub fn set_memo_str(mut self, memo: &str) -> Self {
        self.memo[0] = 0x01;
        self.memo[1] = std::cmp::min(memo.len(), MEMO_BYTES - 2) as u8;
        let memo = format!("{:\0<32}", memo); // Pad user-supplied memo with zeros
        self.memo[2..]
            .copy_from_slice(&memo.as_bytes()[..std::cmp::min(memo.len(), MEMO_BYTES - 2)]);
        // Anything beyond MEMO_BYTES is truncated

        self
    }

    fn pub_key_to_p2p_type(key: CompressedPubKey) -> NonZeroCurvePoint {
        let v = NonZeroCurvePointUncompressedStableV1 {
            x: BigInt::from(key.x),
            is_odd: key.is_odd,
        };
        v.into()
    }

    pub fn to_gossipsub_v2_msg(self, sig: Signature) -> GossipNetMessageV2 {
        let from = Self::pub_key_to_p2p_type(self.source_pk.clone());
        let to = Self::pub_key_to_p2p_type(self.receiver_pk.clone());

        let v = Number(self.fee as i64);
        let v = UnsignedExtendedUInt64Int64ForVersionTagsStableV1(v);
        let fee = CurrencyFeeStableV1(v);

        let nonce = UnsignedExtendedUInt32StableV1(Number(self.nonce as i32));

        let valid_until = UnsignedExtendedUInt32StableV1(Number(self.valid_until as i32));

        let v = CharString::from(&self.memo[..]);
        let memo = MinaBaseSignedCommandMemoStableV1(v);

        let common = MinaBaseSignedCommandPayloadCommonStableV2 {
            fee,
            fee_payer_pk: from.clone(),
            nonce,
            valid_until,
            memo,
        };

        let v = Number(self.amount as i64);
        let v = UnsignedExtendedUInt64Int64ForVersionTagsStableV1(v);
        let amount = CurrencyAmountStableV1(v);

        let v = MinaBasePaymentPayloadStableV2 {
            source_pk: from.clone(),
            receiver_pk: from.clone(),
            amount,
        };
        let body = MinaBaseSignedCommandPayloadBodyStableV2::Payment(v);

        let payload = MinaBaseSignedCommandPayloadStableV2 { common, body };

        let signature = MinaBaseSignatureStableV1(sig.rx.into(), sig.s.into());

        let v = MinaBaseSignedCommandStableV2 {
            payload,
            signer: from.clone(),
            signature,
        };
        let v = MinaBaseUserCommandStableV2::SignedCommand(v);
        let v = NetworkPoolTransactionPoolDiffVersionedStableV2(vec![v]);
        GossipNetMessageV2::TransactionPoolDiff(v)
    }
}
