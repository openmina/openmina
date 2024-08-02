use once_cell::sync::OnceCell;

use crate::constants::ConstraintConstants;

// From mina-signer, to avoid dependency
#[derive(Debug, Clone)]
pub enum NetworkId {
    /// Id for all testnets
    TESTNET = 0x00,

    /// Id for mainnet
    MAINNET = 0x01,
}

#[derive(Debug)]
pub struct NetworkConfig {
    pub name: &'static str,
    pub network_id: NetworkId,
    pub signature_prefix: &'static str,
    pub account_update_hash_param: &'static str,
    pub constraint_system_digests: &'static [[u8; 16]; 3],
    pub default_peers: Vec<&'static str>,
    pub circuits_config: &'static CircuitsConfig,
    pub constraint_constants: &'static ConstraintConstants,
}

#[derive(Debug)]
pub struct CircuitsConfig {
    pub directory_name: &'static str,

    pub step_transaction_gates: &'static str,
    pub wrap_transaction_gates: &'static str,
    pub step_merge_gates: &'static str,
    pub step_blockchain_gates: &'static str,
    pub wrap_blockchain_gates: &'static str,
    pub step_transaction_opt_signed_opt_signed_gates: &'static str,
    pub step_transaction_opt_signed_gates: &'static str,
    pub step_transaction_proved_gates: &'static str,
}

static CONFIG: OnceCell<NetworkConfig> = OnceCell::new();

impl NetworkConfig {
    pub fn global() -> &'static Self {
        CONFIG.get_or_init(|| {
            let config = Self::default_config();
            eprintln!(
                "WARNING: no network config initialized, using default config: {}",
                config.name
            );
            config
        })
    }

    pub fn init(network_name: &str) -> Result<(), String> {
        let config = match network_name {
            "devnet" => Self::devnet_config(),
            "mainnet" => Self::mainnet_config(),
            other => Err(format!("Unknown network {other}"))?,
        };

        CONFIG
            .set(config)
            .map_err(|_| "Double network configuration initialization".to_owned())?;

        Ok(())
    }

    fn default_config() -> Self {
        Self::devnet_config()
    }

    fn mainnet_config() -> Self {
        Self {
            name: mainnet::NAME,
            network_id: mainnet::NETWORK_ID,
            signature_prefix: mainnet::SIGNATURE_PREFIX,
            account_update_hash_param: mainnet::ACCOUNT_UPDATE_HASH_PARAM,
            constraint_system_digests: &mainnet::CONSTRAINT_SYSTEM_DIGESTS,
            default_peers: mainnet::default_peers(),
            circuits_config: &mainnet::CIRCUITS_CONFIG,
            constraint_constants: &mainnet::CONSTRAINT_CONSTANTS,
        }
    }

    fn devnet_config() -> Self {
        Self {
            name: devnet::NAME,
            network_id: devnet::NETWORK_ID,
            signature_prefix: devnet::SIGNATURE_PREFIX,
            account_update_hash_param: devnet::ACCOUNT_UPDATE_HASH_PARAM,
            constraint_system_digests: &devnet::CONSTRAINT_SYSTEM_DIGESTS,
            default_peers: devnet::default_peers(),
            circuits_config: &devnet::CIRCUITS_CONFIG,
            constraint_constants: &devnet::CONSTRAINT_CONSTANTS,
        }
    }
}

// Network constants

pub mod devnet {
    use super::{CircuitsConfig, NetworkId};
    use crate::constants::{ConstraintConstants, ForkConstants};
    use mina_hasher::Fp;

    pub const NETWORK_ID: NetworkId = NetworkId::TESTNET;
    pub const NAME: &str = "devnet";
    pub const SIGNATURE_PREFIX: &str = "CodaSignature";
    pub const ACCOUNT_UPDATE_HASH_PARAM: &str = "TestnetZkappBody";

    pub const CONSTRAINT_SYSTEM_DIGESTS: [[u8; 16]; 3] = [
        // transaction-merge
        [
            0xb8, 0x87, 0x9f, 0x67, 0x7f, 0x62, 0x2a, 0x1d, 0x86, 0x64, 0x80, 0x30, 0x70, 0x1f,
            0x43, 0xe1,
        ],
        // transaction-base
        [
            0x3b, 0xf6, 0xbb, 0x8a, 0x97, 0x66, 0x5f, 0xe7, 0xa9, 0xdf, 0x6f, 0xc1, 0x46, 0xe4,
            0xf9, 0x42,
        ],
        // blockchain-step
        [
            0xd0, 0x24, 0xa9, 0xac, 0x78, 0xd4, 0xc9, 0x3a, 0x88, 0x8b, 0x63, 0xfc, 0x85, 0xee,
            0xb6, 0x6a,
        ],
    ];

    pub const CONSTRAINT_CONSTANTS: ConstraintConstants = ConstraintConstants {
        sub_windows_per_window: 11,
        ledger_depth: 35,
        work_delay: 2,
        block_window_duration_ms: 180000,
        transaction_capacity_log_2: 7,
        pending_coinbase_depth: 5,
        coinbase_amount: 720000000000,
        supercharged_coinbase_factor: 1,
        account_creation_fee: 1000000000,
        // TODO(tizoc): This should come from the config file, but
        // it affects the circuits. Since we cannot produce the circuits
        // ourselves right now, we cannot react to changes in this value,
        // so it will be hardcoded for now.
        fork: Some(ForkConstants {
            state_hash: ark_ff::field_new!(
                Fp,
                "7908066420535064797069631664846455037440232590837253108938061943122344055350"
            ),
            blockchain_length: 296371,
            global_slot_since_genesis: 445860,
        }),
    };

    pub const CIRCUITS_CONFIG: CircuitsConfig = CircuitsConfig {
        directory_name: "3.0.0devnet",

        step_transaction_gates: "step-step-proving-key-transaction-snark-transaction-0-c33ec5211c07928c87e850a63c6a2079",
        wrap_transaction_gates:
            "wrap-wrap-proving-key-transaction-snark-b9a01295c8cc9bda6d12142a581cd305",
        step_merge_gates:
            "step-step-proving-key-transaction-snark-merge-1-ba1d52dfdc2dd4d2e61f6c66ff2a5b2f",
        step_blockchain_gates:
            "step-step-proving-key-blockchain-snark-step-0-55f640777b6486a6fd3fdbc3fcffcc60",
        wrap_blockchain_gates:
            "wrap-wrap-proving-key-blockchain-snark-bbecaf158ca543ec8ac9e7144400e669",
        step_transaction_opt_signed_opt_signed_gates: "step-step-proving-key-transaction-snark-opt_signed-opt_signed-2-48925e6a97197028e1a7c1ecec09021d",
        step_transaction_opt_signed_gates:
            "step-step-proving-key-transaction-snark-opt_signed-3-9eefed16953d2bfa78a257adece02d47",
        step_transaction_proved_gates:
            "step-step-proving-key-transaction-snark-proved-4-0cafcbc6dffccddbc82f8c2519c16341",
    };

    pub fn default_peers() -> Vec<&'static str> {
        vec![
            // "/dns4/seed-1.devnet.gcp.o1test.net/tcp/10003/p2p/12D3KooWAdgYL6hv18M3iDBdaK1dRygPivSfAfBNDzie6YqydVbs",
            // "/dns4/seed-2.devnet.gcp.o1test.net/tcp/10003/p2p/12D3KooWLjs54xHzVmMmGYb7W5RVibqbwD1co7M2ZMfPgPm7iAag",
            // "/dns4/seed-3.devnet.gcp.o1test.net/tcp/10003/p2p/12D3KooWEiGVAFC7curXWXiGZyMWnZK9h8BKr88U8D5PKV3dXciv",
            "/ip4/34.45.167.81/tcp/10003/p2p/12D3KooWAdgYL6hv18M3iDBdaK1dRygPivSfAfBNDzie6YqydVbs",
            "/ip4/34.28.194.121/tcp/10003/p2p/12D3KooWLjs54xHzVmMmGYb7W5RVibqbwD1co7M2ZMfPgPm7iAag",
            "/ip4/34.44.189.148/tcp/10003/p2p/12D3KooWEiGVAFC7curXWXiGZyMWnZK9h8BKr88U8D5PKV3dXciv",
        ]
    }
}

pub mod mainnet {
    use super::{CircuitsConfig, NetworkId};
    use crate::constants::{ConstraintConstants, ForkConstants};
    use mina_hasher::Fp;

    pub const NETWORK_ID: NetworkId = NetworkId::MAINNET;
    pub const NAME: &str = "mainnet";
    pub const SIGNATURE_PREFIX: &str = "MinaSignatureMainnet";
    pub const ACCOUNT_UPDATE_HASH_PARAM: &str = "MainnetZkappBody";

    pub const CONSTRAINT_SYSTEM_DIGESTS: [[u8; 16]; 3] = [
        // transaction-merge
        [
            0xb8, 0x87, 0x9f, 0x67, 0x7f, 0x62, 0x2a, 0x1d, 0x86, 0x64, 0x80, 0x30, 0x70, 0x1f,
            0x43, 0xe1,
        ],
        // transaction-base
        [
            0xd3, 0x19, 0x48, 0xe6, 0x61, 0xcc, 0x66, 0x26, 0x75, 0xb0, 0xc0, 0x79, 0x45, 0x8f,
            0x71, 0x4a,
        ],
        // blockchain-step
        [
            0x14, 0xab, 0x55, 0x62, 0xed, 0x29, 0x2d, 0xe7, 0xa3, 0xde, 0xb9, 0xe1, 0x2f, 0x00,
            0xae, 0xc0,
        ],
    ];

    pub const CONSTRAINT_CONSTANTS: ConstraintConstants = ConstraintConstants {
        sub_windows_per_window: 11,
        ledger_depth: 35,
        work_delay: 2,
        block_window_duration_ms: 180000,
        transaction_capacity_log_2: 7,
        pending_coinbase_depth: 5,
        coinbase_amount: 720000000000,
        supercharged_coinbase_factor: 1,
        account_creation_fee: 1000000000,
        // TODO(tizoc): This should come from the config file, but
        // it affects the circuits. Since we cannot produce the circuits
        // ourselves right now, we cannot react to changes in this value,
        // so it will be hardcoded for now.
        fork: Some(ForkConstants {
            state_hash: ark_ff::field_new!(
                Fp,
                "24465973112608446515163575794792913472627621028836869800891179577915755065526"
            ),
            blockchain_length: 359604,
            global_slot_since_genesis: 564480,
        }),
    };

    pub const CIRCUITS_CONFIG: CircuitsConfig = CircuitsConfig {
        directory_name: "3.0.0mainnet",

        step_transaction_gates: "step-step-proving-key-transaction-snark-transaction-0-b421ac835a0e73935f3d3569ff87f484",
        wrap_transaction_gates:
            "wrap-wrap-proving-key-transaction-snark-93928b62a1803f78b59f698ee4d36e63",
        step_merge_gates:
            "step-step-proving-key-transaction-snark-merge-1-ba1d52dfdc2dd4d2e61f6c66ff2a5b2f",
        step_blockchain_gates:
            "step-step-proving-key-blockchain-snark-step-0-281a97b76f28a0b850065190cbb892af",
        wrap_blockchain_gates:
            "wrap-wrap-proving-key-blockchain-snark-26c8a899619ad2682c077b0fecef87f8",
        step_transaction_opt_signed_opt_signed_gates: "step-step-proving-key-transaction-snark-opt_signed-opt_signed-2-a84fb2a46cf4f9b58857ea5922f23266",
        step_transaction_opt_signed_gates:
            "step-step-proving-key-transaction-snark-opt_signed-3-a7e0f70d44ac6f0dd0afd3478e2b38ac",
        step_transaction_proved_gates:
            "step-step-proving-key-transaction-snark-proved-4-7bb3855dfcf14da4b3ffa7091adc0143",
    };

    pub fn default_peers() -> Vec<&'static str> {
        vec![
            // /dns4/mina-seed.etonec.com/tcp/8302/p2p/12D3KooWKQ1YVtqZFzxDmSw8RASCPZpDCQBywnFz76RbrvZCXk5T
            // /dns4/mina-mainnet-seed.obscura.network/tcp/5002/p2p/12D3KooWFRpU3giZDFjJjwoHSY8kdpv8ktvferGkyQRUHozsXw4X
            // /dns4/mina-mainnet-seed.staketab.com/tcp/10003/p2p/12D3KooWSDTiXcdBVpN12ZqXJ49qCFp8zB1NnovuhZu6A28GLF1J
            // /dns4/mina-seed-1.zkvalidator.com/tcp/8302/p2p/12D3KooWSfEfnVCqzpMbmyUmRY3ESEVmJaRcd1EkLbnvvERQxwtu
            // /dns4/mina-seed.bitcat365.com/tcp/10001/p2p/12D3KooWQzozNTDKL7MqUh6Nh11GMA4pQhRCAsNTRWxCAzAi4VbE
            // /dns4/production-mainnet-libp2p.minaprotocol.network/tcp/10000/p2p/12D3KooWPywsM191KGGNVGiNqN35nyyJg4W2BhhYukF6hP9YBR8q
            // /dns4/production-mainnet-libp2p.minaprotocol.network/tcp/10010/p2p/12D3KooWGB6mJ9Ub9qRBDgHhedNXH4FawWjGQGGN2tQKaKa3gK2h
            // /dns4/production-mainnet-libp2p.minaprotocol.network/tcp/10020/p2p/12D3KooWMvsPx6A1XNa4V8bTbNb6Fh7WHWf92Ezgfxt6UWxiNq5n
            // /dns4/production-mainnet-libp2p.minaprotocol.network/tcp/10030/p2p/12D3KooW9wL9iaj7qbCTBFspi4gCwdZFCdNRnwkRrdRfe4GBJ978
            // /dns4/production-mainnet-libp2p.minaprotocol.network/tcp/10040/p2p/12D3KooWL8SFDx6PSzpSLgBtRSK1brjKFqs8EvW2yX9zexQEefAo
            // /dns4/seed-1.mainnet.gcp.o1test.net/tcp/10003/p2p/12D3KooWCa1d7G3SkRxy846qTvdAFX69NnoYZ32orWVLqJcDVGHW
            // /dns4/seed-2.mainnet.gcp.o1test.net/tcp/10003/p2p/12D3KooWK4NfthViCTyLgVQa1WvqDC1NccVxGruCXCZUt3GqvFvn
            // /dns4/seed-4.mainnet.gcp.o1test.net/tcp/10003/p2p/12D3KooWEdBiTUQqxp3jeuWaZkwiSNcFxC6d6Tdq7u2Lf2ZD2Q6X
            // /dns4/seed-5.mainnet.gcp.o1test.net/tcp/10003/p2p/12D3KooWL1DJTigSwuKQRfQE3p7puFUqfbHjXbZJ9YBWtMNpr3GU
            // /dns4/seed.minaexplorer.com/tcp/8302/p2p/12D3KooWR7coZtrMHvsgsfiWq2GESYypac3i29LFGp6EpbtjxBiJ
            // /dns4/seed.minataur.net/tcp/8302/p2p/12D3KooWNyExDzG8T1BYXHpXQS66kaw3zi6qi5Pg9KD3GEyHW5FF
            // /dns4/seed.piconbello.com/tcp/10001/p2p/12D3KooWRFac2AztcTeen2DYNwnTrmVBvwNDsRiFpDVdTkwdFAHP
            "/ip4/138.201.11.249/tcp/8302/p2p/12D3KooWKQ1YVtqZFzxDmSw8RASCPZpDCQBywnFz76RbrvZCXk5T",
            "/ip4/51.178.128.35/tcp/5002/p2p/12D3KooWFRpU3giZDFjJjwoHSY8kdpv8ktvferGkyQRUHozsXw4X",
            "/ip4/138.201.53.35/tcp/10003/p2p/12D3KooWSDTiXcdBVpN12ZqXJ49qCFp8zB1NnovuhZu6A28GLF1J",
            "/ip4/37.27.121.141/tcp/8302/p2p/12D3KooWSfEfnVCqzpMbmyUmRY3ESEVmJaRcd1EkLbnvvERQxwtu",
            "/ip4/94.130.21.18/tcp/10001/p2p/12D3KooWQzozNTDKL7MqUh6Nh11GMA4pQhRCAsNTRWxCAzAi4VbE",
            "/ip4/44.236.52.227/tcp/10000/p2p/12D3KooWPywsM191KGGNVGiNqN35nyyJg4W2BhhYukF6hP9YBR8q",
            "/ip4/44.236.52.227/tcp/10010/p2p/12D3KooWGB6mJ9Ub9qRBDgHhedNXH4FawWjGQGGN2tQKaKa3gK2h",
            "/ip4/44.236.52.227/tcp/10020/p2p/12D3KooWMvsPx6A1XNa4V8bTbNb6Fh7WHWf92Ezgfxt6UWxiNq5n",
            "/ip4/44.236.52.227/tcp/10030/p2p/12D3KooW9wL9iaj7qbCTBFspi4gCwdZFCdNRnwkRrdRfe4GBJ978",
            "/ip4/44.236.52.227/tcp/10040/p2p/12D3KooWL8SFDx6PSzpSLgBtRSK1brjKFqs8EvW2yX9zexQEefAo",
            "/ip4/34.86.219.199/tcp/10003/p2p/12D3KooWCa1d7G3SkRxy846qTvdAFX69NnoYZ32orWVLqJcDVGHW",
            "/ip4/34.145.137.93/tcp/10003/p2p/12D3KooWK4NfthViCTyLgVQa1WvqDC1NccVxGruCXCZUt3GqvFvn",
            "/ip4/34.95.19.83/tcp/10003/p2p/12D3KooWEdBiTUQqxp3jeuWaZkwiSNcFxC6d6Tdq7u2Lf2ZD2Q6X",
            "/ip4/35.203.59.118/tcp/10003/p2p/12D3KooWL1DJTigSwuKQRfQE3p7puFUqfbHjXbZJ9YBWtMNpr3GU",
            "/ip4/65.21.20.43/tcp/8302/p2p/12D3KooWR7coZtrMHvsgsfiWq2GESYypac3i29LFGp6EpbtjxBiJ",
            "/ip4/37.27.118.159/tcp/8302/p2p/12D3KooWNyExDzG8T1BYXHpXQS66kaw3zi6qi5Pg9KD3GEyHW5FF",
            "/ip4/144.76.18.153/tcp/10001/p2p/12D3KooWRFac2AztcTeen2DYNwnTrmVBvwNDsRiFpDVdTkwdFAHP",
        ]
    }
}
