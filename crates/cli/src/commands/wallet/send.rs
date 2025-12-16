use std::{path::PathBuf, str::FromStr};

use anyhow::{Context, Result};
use ledger::scan_state::{
    currency::{Amount, Fee, Nonce, Slot},
    transaction_logic::{
        signed_command::{Body, Common, PaymentPayload, SignedCommand, SignedCommandPayload},
        transaction_union_payload::TransactionUnionPayload,
        Memo,
    },
};
use mina_node_account::{AccountPublicKey, AccountSecretKey};
use mina_p2p_messages::v2::MinaBaseSignedCommandStableV2;
use mina_signer::{CompressedPubKey, Keypair, Signer};

use super::super::Network;

fn network_to_network_id(network: &Network) -> mina_signer::NetworkId {
    match network {
        Network::Mainnet => mina_signer::NetworkId::MAINNET,
        Network::Devnet => mina_signer::NetworkId::TESTNET,
    }
}

#[derive(Debug, clap::Args)]
pub struct Send {
    /// Path to encrypted sender key file
    #[arg(long, env)]
    pub from: PathBuf,

    /// Password to decrypt the sender key
    #[arg(
        env = "MINA_PRIVKEY_PASS",
        default_value = "",
        help = "Password to decrypt the sender key (env: MINA_PRIVKEY_PASS)"
    )]
    pub password: String,

    /// Receiver's public key
    #[arg(long)]
    pub to: AccountPublicKey,

    /// Amount in nanomina (1 MINA = 1,000,000,000 nanomina)
    #[arg(long)]
    pub amount: u64,

    /// Transaction fee in nanomina
    #[arg(long)]
    pub fee: u64,

    /// Optional memo (max 32 bytes)
    #[arg(long, default_value = "")]
    pub memo: String,

    /// Transaction nonce (if not provided, will be fetched from node)
    #[arg(long)]
    pub nonce: Option<u32>,

    /// Slot number until which transaction is valid
    /// If not provided, defaults to maximum slot (transaction never expires)
    #[arg(long)]
    pub valid_until: Option<u32>,

    /// Optional fee payer public key (if different from sender)
    /// If not provided, the sender will pay the fee
    #[arg(long)]
    pub fee_payer: Option<AccountPublicKey>,

    /// Node RPC endpoint
    #[arg(long, default_value = "http://localhost:3000")]
    pub node: String,
}

impl Send {
    pub fn run(self, network: Network) -> Result<()> {
        // Check node is synced and on the correct network
        println!("Checking node status...");
        self.check_node_status(&network)?;

        // Load the sender's secret key
        let sender_key = AccountSecretKey::from_encrypted_file(&self.from, &self.password)
            .with_context(|| {
                format!("Failed to decrypt sender key file: {}", self.from.display())
            })?;

        let sender_pk = sender_key.public_key_compressed();
        println!("Sender: {}", AccountPublicKey::from(sender_pk.clone()));

        // Determine the fee payer (use fee_payer if provided, otherwise use sender)
        let fee_payer_pk: CompressedPubKey = if let Some(ref fee_payer) = self.fee_payer {
            println!("Fee payer: {}", fee_payer);
            fee_payer
                .clone()
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid fee payer public key"))?
        } else {
            sender_pk.clone()
        };

        // Convert receiver public key to CompressedPubKey
        let receiver_pk: CompressedPubKey = self
            .to
            .clone()
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid receiver public key"))?;

        // Fetch nonce from node if not provided
        // Note: GraphQL API expects nonce to be account_nonce, but we need to sign
        // with account_nonce for the first transaction from a new account
        let nonce = if let Some(nonce) = self.nonce {
            nonce
        } else {
            println!("Fetching nonce from node...");
            self.fetch_nonce(&fee_payer_pk)?
        };

        println!("Using nonce: {}", nonce);

        // Create the payment payload
        let payload = SignedCommandPayload {
            common: Common {
                fee: Fee::from_u64(self.fee),
                fee_payer_pk,
                nonce: Nonce::from_u32(nonce),
                valid_until: self
                    .valid_until
                    .map(Slot::from_u32)
                    .unwrap_or_else(Slot::max),
                memo: Memo::from_str(&self.memo).unwrap_or_else(|_| Memo::empty()),
            },
            body: Body::Payment(PaymentPayload {
                receiver_pk,
                amount: Amount::from_u64(self.amount),
            }),
        };

        // Sign the transaction
        println!("Signing transaction...");
        let network_id = network_to_network_id(&network);
        let signed_command = self.sign_transaction(payload, &sender_key, network_id)?;

        // Submit to node
        println!("Submitting transaction to node...");
        let tx_hash = self.submit_transaction(signed_command)?;

        println!("\nTransaction submitted successfully!");
        println!("Transaction hash: {}", tx_hash);
        println!("Status: Pending");
        println!("\nYou can check the transaction status with:");
        println!("  mina wallet status --hash {}", tx_hash);

        Ok(())
    }

    fn check_node_status(&self, network: &Network) -> Result<()> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;
        let url = format!("{}/graphql", self.node);

        // GraphQL query to check sync status and network ID
        let query = serde_json::json!({
            "query": r#"query {
                syncStatus
                networkID
            }"#
        });

        let response = client
            .post(&url)
            .json(&query)
            .send()
            .context("Failed to query node status")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to connect to node: HTTP {}", response.status());
        }

        let response_json: serde_json::Value = response
            .json()
            .context("Failed to parse GraphQL response")?;

        // Check for GraphQL errors
        if let Some(errors) = response_json.get("errors") {
            let error_msg = serde_json::to_string_pretty(errors)
                .unwrap_or_else(|_| "Unknown GraphQL error".to_string());
            anyhow::bail!("GraphQL error: {}", error_msg);
        }

        // Check sync status
        let sync_status = response_json["data"]["syncStatus"]
            .as_str()
            .context("Sync status not found in GraphQL response")?;

        if sync_status != "SYNCED" {
            anyhow::bail!(
                "Node is not synced (status: {}). Please wait for the node to sync before sending transactions.",
                sync_status
            );
        }

        println!("Node is synced: {}", sync_status);

        // Check network ID
        let network_id = response_json["data"]["networkID"]
            .as_str()
            .context("Network ID not found in GraphQL response")?;

        // Expected network ID based on selected network
        let expected_network = match network {
            Network::Mainnet => "mina:mainnet",
            Network::Devnet => "mina:devnet",
        };

        if !network_id.contains(expected_network) {
            anyhow::bail!(
                "Network mismatch: node is on '{}' but you selected {:?}. Use --network to specify the correct network.",
                network_id,
                network
            );
        }

        println!("Network verified: {}", network_id);

        Ok(())
    }

    fn fetch_nonce(&self, sender_pk: &CompressedPubKey) -> Result<u32> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;
        let url = format!("{}/graphql", self.node);

        // GraphQL query to fetch account information
        let query = serde_json::json!({
            "query": format!(
                r#"query {{
                    account(publicKey: "{}") {{
                        nonce
                    }}
                }}"#,
                AccountPublicKey::from(sender_pk.clone()).to_string()
            )
        });

        let response = client
            .post(&url)
            .json(&query)
            .send()
            .context("Failed to query account from node")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch account: HTTP {}", response.status());
        }

        let response_json: serde_json::Value = response
            .json()
            .context("Failed to parse GraphQL response")?;

        // Extract nonce from GraphQL response
        let nonce_str = response_json["data"]["account"]["nonce"]
            .as_str()
            .context("Nonce not found in GraphQL response")?;

        let nonce = nonce_str
            .parse::<u32>()
            .context("Failed to parse nonce as u32")?;

        Ok(nonce)
    }

    fn sign_transaction(
        &self,
        payload: SignedCommandPayload,
        sender_key: &AccountSecretKey,
        network_id: mina_signer::NetworkId,
    ) -> Result<SignedCommand> {
        // Create the transaction union payload for signing
        let payload_to_sign = TransactionUnionPayload::of_user_command_payload(&payload);

        // Create signer and sign the transaction
        let mut signer = mina_signer::create_legacy(network_id);
        let kp: Keypair = sender_key.clone().into();
        // Use packed=true for OCaml/TypeScript compatibility (required by Mina protocol)
        let signature = signer.sign(&kp, &payload_to_sign, true);

        Ok(SignedCommand {
            payload,
            signer: sender_key.public_key_compressed(),
            signature,
        })
    }

    fn submit_transaction(&self, signed_command: SignedCommand) -> Result<String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .context("Failed to create HTTP client")?;
        let url = format!("{}/graphql", self.node);

        // Convert to v2 types for easier field extraction
        let signed_cmd_v2: MinaBaseSignedCommandStableV2 = (&signed_command).into();

        // Convert signature to GraphQL format (field and scalar as decimal strings)
        let sig_field =
            mina_p2p_messages::bigint::BigInt::from(signed_command.signature.rx).to_decimal();
        let sig_scalar =
            mina_p2p_messages::bigint::BigInt::from(signed_command.signature.s).to_decimal();

        // Extract payment details from signed command
        let (receiver_pk, amount) = match &signed_cmd_v2.payload.body {
            mina_p2p_messages::v2::MinaBaseSignedCommandPayloadBodyStableV2::Payment(payment) => {
                (payment.receiver_pk.to_string(), payment.amount.to_string())
            }
            _ => anyhow::bail!("Expected payment body in signed command"),
        };

        let fee_payer_pk = signed_cmd_v2.payload.common.fee_payer_pk.to_string();

        // Build memo field - omit if empty
        let memo_field = if self.memo.is_empty() {
            String::new()
        } else {
            format!(r#"memo: "{}""#, self.memo)
        };

        // Build GraphQL mutation
        let mutation = format!(
            r#"mutation {{
                sendPayment(
                    input: {{
                        from: "{}"
                        to: "{}"
                        amount: "{}"
                        fee: "{}"
                        {}
                        nonce: "{}"
                        validUntil: "{}"
                    }}
                    signature: {{
                        field: "{}"
                        scalar: "{}"
                    }}
                ) {{
                    payment {{
                        hash
                        id
                    }}
                }}
            }}"#,
            fee_payer_pk,
            receiver_pk,
            amount,
            ***signed_cmd_v2.payload.common.fee,
            memo_field,
            **signed_cmd_v2.payload.common.nonce,
            signed_cmd_v2.payload.common.valid_until.as_u32(),
            sig_field,
            sig_scalar,
        );

        let query = serde_json::json!({
            "query": mutation
        });

        let response = client
            .post(&url)
            .json(&query)
            .send()
            .context("Failed to submit transaction to node")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!(
                "Failed to submit transaction: HTTP {} - {}",
                status,
                error_text
            );
        }

        let response_json: serde_json::Value = response
            .json()
            .context("Failed to parse GraphQL response")?;

        // Check for GraphQL errors
        if let Some(errors) = response_json.get("errors") {
            let error_msg = serde_json::to_string_pretty(errors)
                .unwrap_or_else(|_| "Unknown GraphQL error".to_string());
            anyhow::bail!("GraphQL error: {}", error_msg);
        }

        // Extract transaction hash from response
        let hash = response_json["data"]["sendPayment"]["payment"]["hash"]
            .as_str()
            .context("Transaction hash not found in GraphQL response")?
            .to_string();

        Ok(hash)
    }
}
