use ledger::FpExt;
use mina_p2p_messages::bigint::BigInt;
use mina_signer::Signature;
use redux::Timestamp;
use serde::Serialize;

use super::RpcNodeStatus;
use crate::p2p::PeerId;
use openmina_node_account::{AccountPublicKey, AccountSecretKey};

/// Matches the representation used by o1js where each field is a string
/// containing a decimal representation of the field.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct SignatureJson {
    pub field: String,
    pub scalar: String,
}

impl From<Signature> for SignatureJson {
    fn from(sig: Signature) -> Self {
        Self {
            field: sig.rx.to_decimal(),
            scalar: sig.s.to_decimal(),
        }
    }
}

impl TryInto<Signature> for SignatureJson {
    type Error = String;

    fn try_into(self) -> Result<Signature, Self::Error> {
        let rx = BigInt::from_decimal(&self.field)
            .map_err(|_| "Failed to parse decimals as BigInt")?
            .try_into()
            .map_err(|_| "Failed to convert rx BigInt to field element")?;
        let s = BigInt::from_decimal(&self.scalar)
            .map_err(|_| "Failed to parse decimals as BigInt")?
            .try_into()
            .map_err(|_| "Failed to convert rx BigInt to field element")?;

        Ok(Signature::new(rx, s))
    }
}

/// A signed heartbeat message from a node
#[derive(Serialize, Debug, Clone)]
pub struct SignedNodeHeartbeat {
    pub version: u8,
    /// base64 encoded json of the payload
    pub payload: String,
    pub submitter: AccountPublicKey,
    pub signature: SignatureJson,
}

impl SignedNodeHeartbeat {
    /// Verifies that the signature is valid for this heartbeat
    pub fn verify_signature(&self) -> bool {
        use blake2::digest::{Update, VariableOutput};
        use mina_signer::{CompressedPubKey, PubKey, Signer};

        let signature = match self.signature.clone().try_into() {
            Ok(sig) => sig,
            Err(_) => return false,
        };

        let pk: CompressedPubKey = match self.submitter.clone().try_into() {
            Ok(pk) => pk,
            Err(_) => return false,
        };

        let pk = match PubKey::from_address(&pk.into_address()) {
            Ok(pk) => pk,
            Err(_) => return false,
        };

        // Calculate digest from payload
        let mut hasher = blake2::Blake2bVar::new(32).expect("Invalid Blake2bVar output size");
        let mut blake2_hash = [0u8; 32];
        hasher.update(self.payload.as_bytes());
        hasher.finalize_variable(&mut blake2_hash).unwrap();

        let digest = NodeHeartbeatPayloadDigest(blake2_hash);
        let mut signer = mina_signer::create_legacy::<NodeHeartbeatPayloadDigest>(
            mina_signer::NetworkId::TESTNET,
        );

        signer.verify(&signature, &pk, &digest)
    }
}

/// Node heartbeat
#[derive(Serialize, Debug, Clone)]
pub struct NodeHeartbeat {
    pub status: RpcNodeStatus,
    pub node_timestamp: Timestamp,
    pub peer_id: PeerId,
    // binprot+base64 encoded block
    pub last_produced_block: Option<String>,
}

/// Blake2b hash of the encoded heartbeat payload
#[derive(Clone, Debug)]
pub struct NodeHeartbeatPayloadDigest([u8; 32]);

impl mina_hasher::Hashable for NodeHeartbeatPayloadDigest {
    type D = mina_signer::NetworkId;

    fn to_roinput(&self) -> mina_hasher::ROInput {
        let mut hex = [0u8; 64];
        hex::encode_to_slice(self.0, &mut hex).unwrap();

        // Bits must be reversed to match the JS implementation
        for b in hex.iter_mut() {
            *b = b.reverse_bits();
        }

        mina_hasher::ROInput::new().append_bytes(&hex)
    }

    fn domain_string(network_id: Self::D) -> Option<String> {
        match network_id {
            Self::D::MAINNET => openmina_core::network::mainnet::SIGNATURE_PREFIX,
            Self::D::TESTNET => openmina_core::network::devnet::SIGNATURE_PREFIX,
        }
        .to_string()
        .into()
    }
}

impl NodeHeartbeat {
    const CURRENT_VERSION: u8 = 1;

    /// Creates base64 encoded payload and its Blake2b digest
    fn payload_and_digest(&self) -> (String, NodeHeartbeatPayloadDigest) {
        use base64::{engine::general_purpose::URL_SAFE, Engine as _};
        use blake2::{
            digest::{Update, VariableOutput},
            Blake2bVar,
        };

        let payload = serde_json::to_string(self).unwrap();
        let encoded_payload = URL_SAFE.encode(&payload);

        let mut hasher = Blake2bVar::new(32).expect("Invalid Blake2bVar output size");
        let mut blake2_hash = [0u8; 32];

        hasher.update(encoded_payload.as_bytes());
        hasher.finalize_variable(&mut blake2_hash).unwrap();

        (encoded_payload, NodeHeartbeatPayloadDigest(blake2_hash))
    }

    /// Signs the heartbeat using the provided secret key
    pub fn sign(&self, secret_key: &AccountSecretKey) -> SignedNodeHeartbeat {
        let (payload, digest) = self.payload_and_digest();
        let submitter = secret_key.public_key();

        let signature = {
            use mina_signer::{Keypair, Signer};
            let mut signer = mina_signer::create_legacy::<NodeHeartbeatPayloadDigest>(
                mina_signer::NetworkId::TESTNET,
            );
            let kp = Keypair::from(secret_key.clone());

            let signature = signer.sign(&kp, &digest);
            signature.into()
        };

        SignedNodeHeartbeat {
            version: Self::CURRENT_VERSION,
            payload,
            submitter,
            signature,
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {

    use crate::rpc::{
        RpcNodeStatusResources, RpcNodeStatusSnarkPool, RpcNodeStatusTransactionPool,
        RpcNodeStatusTransitionFrontier, RpcNodeStatusTransitionFrontierSync,
    };

    use super::*;
    use redux::Timestamp;

    #[test]
    fn test_heartbeat_signing() {
        let heartbeat = create_test_heartbeat();
        let secret_key = AccountSecretKey::deterministic(0);
        let signed = heartbeat.sign(&secret_key);

        println!("Private key: {}", secret_key);
        println!("Public key: {}", secret_key.public_key());
        println!("Payload: {}", signed.payload);
        println!("Signature: {:?}", signed.signature);

        assert_eq!(&signed.payload, "eyJzdGF0dXMiOnsiY2hhaW5faWQiOm51bGwsInRyYW5zaXRpb25fZnJvbnRpZXIiOnsiYmVzdF90aXAiOm51bGwsInN5bmMiOnsidGltZSI6bnVsbCwic3RhdHVzIjoiU3luY2VkIiwicGhhc2UiOiJSdW5uaW5nIiwidGFyZ2V0IjpudWxsfX0sInBlZXJzIjpbXSwic25hcmtfcG9vbCI6eyJ0b3RhbF9qb2JzIjowLCJzbmFya3MiOjB9LCJ0cmFuc2FjdGlvbl9wb29sIjp7InRyYW5zYWN0aW9ucyI6MCwidHJhbnNhY3Rpb25zX2Zvcl9wcm9wYWdhdGlvbiI6MCwidHJhbnNhY3Rpb25fY2FuZGlkYXRlcyI6MH0sImN1cnJlbnRfYmxvY2tfcHJvZHVjdGlvbl9hdHRlbXB0IjpudWxsfSwibm9kZV90aW1lc3RhbXAiOjAsInBlZXJfaWQiOiIyYkVnQnJQVHpMOHdvdjJENEt6MzRXVkxDeFI0dUNhcnNCbUhZWFdLUUE1d3ZCUXpkOUgiLCJsYXN0X3Byb2R1Y2VkX2Jsb2NrIjpudWxsfQ==");
        assert_eq!(
            &signed.signature.field,
            "25500978175045040705256298774101531557080530394536110798266178142513301557846"
        );
        assert_eq!(
            &signed.signature.scalar,
            "27991123709623419396663280967637181749724990269901703962618583375785482061803"
        );
        assert!(signed.verify_signature());
    }

    #[test]
    fn test_heartbeat_signature_deterministic() {
        let heartbeat = create_test_heartbeat();
        let secret_key = AccountSecretKey::deterministic(0);

        let signed1 = heartbeat.sign(&secret_key);
        let signed2 = heartbeat.sign(&secret_key);

        assert_eq!(signed1.payload, signed2.payload);
        assert_eq!(signed1.signature, signed2.signature);
    }

    #[test]
    fn test_heartbeat_different_keys_different_sigs() {
        let heartbeat = create_test_heartbeat();
        let sk1 = AccountSecretKey::deterministic(0);
        let sk2 = AccountSecretKey::deterministic(1);

        let signed1 = heartbeat.sign(&sk1);
        let signed2 = heartbeat.sign(&sk2);

        assert_eq!(signed1.payload, signed2.payload);
        assert_ne!(signed1.signature, signed2.signature);
        assert_ne!(signed1.submitter, signed2.submitter);
    }

    fn create_test_heartbeat() -> NodeHeartbeat {
        NodeHeartbeat {
            status: RpcNodeStatus {
                chain_id: None,
                transition_frontier: RpcNodeStatusTransitionFrontier {
                    best_tip: None,
                    sync: RpcNodeStatusTransitionFrontierSync {
                        time: None,
                        status: "Synced".to_string(),
                        phase: "Running".to_string(),
                        target: None,
                    },
                },
                peers: vec![],
                snark_pool: RpcNodeStatusSnarkPool::default(),
                transaction_pool: RpcNodeStatusTransactionPool::default(),
                current_block_production_attempt: None,
                resources_status: RpcNodeStatusResources {
                    p2p_malloc_size: 0,
                    transition_frontier: serde_json::Value::Null,
                    snark_pool: serde_json::Value::Null,
                },
            },
            node_timestamp: Timestamp::ZERO,
            peer_id: "2bEgBrPTzL8wov2D4Kz34WVLCxR4uCarsBmHYXWKQA5wvBQzd9H"
                .parse()
                .unwrap(),
            last_produced_block: None,
        }
    }
}
