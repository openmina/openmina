use std::{fmt, io};

use binprot::BinProtWrite;
use serde::{Deserialize, Serialize};
use sha2::{
    digest::{generic_array::GenericArray, typenum::U32},
    Digest, Sha256,
};

use super::generated;

impl generated::MinaBaseStagedLedgerHashNonSnarkStableV1 {
    pub fn sha256(&self) -> GenericArray<u8, U32> {
        let mut ledger_hash_bytes: [u8; 32] = [0; 32];

        ledger_hash_bytes.copy_from_slice(self.ledger_hash.as_ref());
        ledger_hash_bytes.reverse();

        let mut hasher = Sha256::new();
        hasher.update(ledger_hash_bytes);
        hasher.update(self.aux_hash.as_ref());
        hasher.update(self.pending_coinbase_aux.as_ref());

        hasher.finalize()
    }
}

impl generated::ConsensusVrfOutputTruncatedStableV1 {
    pub fn blake2b(&self) -> Vec<u8> {
        use blake2::{
            digest::{Update, VariableOutput},
            Blake2bVar,
        };
        let mut hasher = Blake2bVar::new(32).expect("Invalid Blake2bVar output size");
        hasher.update(&self.0);
        hasher.finalize_boxed().to_vec()
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct TransactionHash(Vec<u8>);

impl std::str::FromStr for TransactionHash {
    type Err = bs58::decode::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = bs58::decode(s).with_check(Some(0x12)).into_vec()?[1..].to_vec();
        Ok(Self(bytes))
    }
}

impl fmt::Display for TransactionHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        bs58::encode(&self.0)
            .with_check_version(0x12)
            .into_string()
            .fmt(f)
    }
}

impl fmt::Debug for TransactionHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
        // write!(f, "TransactionHash({})", self)
    }
}

impl generated::MinaBaseUserCommandStableV2 {
    pub fn hash(&self) -> io::Result<TransactionHash> {
        match self {
            Self::SignedCommand(v) => v.hash(),
            Self::ZkappCommand(_) => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "zkapp tx hashing is not yet supported",
            )),
        }
    }
}

impl generated::MinaBaseSignedCommandStableV2 {
    pub fn hash(&self) -> io::Result<TransactionHash> {
        use blake2::{
            digest::{Update, VariableOutput},
            Blake2bVar,
        };
        let mut hasher = Blake2bVar::new(32).expect("Invalid Blake2bVar output size");

        let mut encoded = vec![];
        self.binprot_write(&mut encoded)?;
        hasher.update(&encoded);
        let mut hash = vec![0; 34];
        hash[..2].copy_from_slice(&[1, 32]);
        hash[2..].copy_from_slice(&hasher.finalize_boxed());

        Ok(TransactionHash(hash))
    }
}

#[cfg(test)]
mod tests {
    use super::super::manual;
    use super::*;

    fn pub_key(address: &str) -> manual::NonZeroCurvePoint {
        let key = mina_signer::PubKey::from_address(address)
            .unwrap()
            .into_compressed();
        let v = generated::NonZeroCurvePointUncompressedStableV1 {
            x: crate::bigint::BigInt::from(key.x),
            is_odd: key.is_odd,
        };
        v.into()
    }

    fn tx_hash(
        from: &str,
        to: &str,
        amount: u64,
        fee: u64,
        nonce: u32,
        valid_until: i32,
    ) -> String {
        use crate::bigint::BigInt;
        use crate::number::Number;
        use crate::string::CharString;

        let from = pub_key(from);
        let to = pub_key(to);

        let v = Number(fee as i64);
        let v = generated::UnsignedExtendedUInt64Int64ForVersionTagsStableV1(v);
        let fee = generated::CurrencyFeeStableV1(v);

        let nonce = generated::UnsignedExtendedUInt32StableV1(Number(nonce as i32));

        let valid_until = generated::UnsignedExtendedUInt32StableV1(Number(valid_until));

        let memo = bs58::decode("E4Yd67s51QN9DZVDy8JKPEoNGykMsYQ5KRiKpZHiLZTjA8dB9SnFT")
            .with_check(Some(0x14))
            .into_vec()
            .unwrap()[1..]
            .to_vec();
        let v = CharString::from(&memo[..]);
        let memo = generated::MinaBaseSignedCommandMemoStableV1(v);

        let common = generated::MinaBaseSignedCommandPayloadCommonStableV2 {
            fee,
            fee_payer_pk: from.clone(),
            nonce,
            valid_until,
            memo,
        };

        let v = Number(amount as i64);
        let v = generated::UnsignedExtendedUInt64Int64ForVersionTagsStableV1(v);
        let amount = generated::CurrencyAmountStableV1(v);

        let v = generated::MinaBasePaymentPayloadStableV2 {
            source_pk: from.clone(),
            receiver_pk: to.clone(),
            amount,
        };
        let body = generated::MinaBaseSignedCommandPayloadBodyStableV2::Payment(v);

        let payload = generated::MinaBaseSignedCommandPayloadStableV2 { common, body };

        let signature = generated::MinaBaseSignatureStableV1(BigInt::one(), BigInt::one());

        let v = generated::MinaBaseSignedCommandStableV2 {
            payload,
            signer: from.clone(),
            signature,
        };
        let v = generated::MinaBaseUserCommandStableV2::SignedCommand(v);
        dbg!(v.hash().unwrap()).to_string()
    }

    #[test]
    fn test_payment_hash_1() {
        let expected_hash = "CkpYVhMRYP3zfgpHzKJfJoHNkL9dNTgpA9zKQ8y2X533Pm9yZdN8q";
        let expected_tx_hash: TransactionHash = expected_hash.parse().unwrap();
        dbg!(expected_tx_hash);

        assert_eq!(
            tx_hash(
                "B62qp3B9VW1ir5qL1MWRwr6ecjC2NZbGr8vysGeme9vXGcFXTMNXb2t",
                "B62qieNixrVNNK3G6nNviFa77yTCbR4tfCcm7w7H8LqTjFoqvdEfF4W",
                1500000,
                75495949,
                25646,
                -1
            ),
            expected_hash
        )
    }
}
