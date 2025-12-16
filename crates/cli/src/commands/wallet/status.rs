use anyhow::{Context, Result};
use serde_json::Value;

#[derive(Debug, clap::Args)]
pub struct Status {
    /// Transaction hash to check
    #[arg(long)]
    pub hash: String,

    /// Node RPC endpoint
    #[arg(long, default_value = "http://localhost:3000")]
    pub node: String,

    /// Check if transaction is in mempool (pooled transactions)
    #[arg(long)]
    pub check_mempool: bool,
}

impl Status {
    pub fn run(self) -> Result<()> {
        println!("Checking transaction status...");
        println!("Transaction hash: {}", self.hash);

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        let url = format!("{}/graphql", self.node);

        // First, try to find the transaction in the best chain
        let query = serde_json::json!({
            "query": format!(
                r#"query {{
                    transactionStatus(payment: "{}")
                }}"#,
                self.hash
            )
        });

        let response = client
            .post(&url)
            .json(&query)
            .send()
            .context("Failed to query transaction status")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to query node: HTTP {}", response.status());
        }

        let response_json: Value = response
            .json()
            .context("Failed to parse GraphQL response")?;

        // Check for GraphQL errors
        if let Some(_errors) = response_json.get("errors") {
            // Transaction might not be found in the chain yet
            // Automatically check mempool as fallback
            println!("\nTransaction not found in blockchain, checking mempool...");
            return self.check_pooled_transactions(&client, &url);
        }

        // Parse transaction status
        if let Some(status) = response_json["data"]["transactionStatus"].as_str() {
            println!("\nTransaction Status: {}", status);

            match status {
                "INCLUDED" => {
                    println!("✓ Transaction has been included in a block");
                }
                "PENDING" => {
                    println!("⏳ Transaction is pending inclusion");
                }
                "UNKNOWN" => {
                    println!("? Transaction status is unknown");
                    if !self.check_mempool {
                        println!(
                            "\nTry using --check-mempool to check if it's in the transaction pool"
                        );
                    }
                }
                _ => {
                    println!("Status: {}", status);
                }
            }
        } else {
            println!("Unable to determine transaction status");
        }

        Ok(())
    }

    fn check_pooled_transactions(
        &self,
        client: &reqwest::blocking::Client,
        url: &str,
    ) -> Result<()> {
        let query = serde_json::json!({
            "query": r#"query {
                pooledUserCommands {
                    hash
                    from
                    to
                    amount
                    fee
                    nonce
                }
            }"#
        });

        let response = client
            .post(url)
            .json(&query)
            .send()
            .context("Failed to query pooled transactions")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to query mempool: HTTP {}", response.status());
        }

        let response_json: Value = response
            .json()
            .context("Failed to parse GraphQL response")?;

        if let Some(pooled) = response_json["data"]["pooledUserCommands"].as_array() {
            // Look for our transaction in the pool
            for tx in pooled {
                if let Some(hash) = tx["hash"].as_str() {
                    if hash == self.hash {
                        println!("\n✓ Transaction found in mempool!");
                        println!("\nTransaction Details:");
                        println!("  Hash:   {}", hash);
                        if let Some(from) = tx["from"].as_str() {
                            println!("  From:   {}", from);
                        }
                        if let Some(to) = tx["to"].as_str() {
                            println!("  To:     {}", to);
                        }
                        if let Some(amount) = tx["amount"].as_str() {
                            println!("  Amount: {} nanomina", amount);
                        }
                        if let Some(fee) = tx["fee"].as_str() {
                            println!("  Fee:    {} nanomina", fee);
                        }
                        // Nonce can be either string or number depending on the node
                        if let Some(nonce) = tx["nonce"].as_u64() {
                            println!("  Nonce:  {}", nonce);
                        } else if let Some(nonce) = tx["nonce"].as_str() {
                            println!("  Nonce:  {}", nonce);
                        }

                        println!("\nStatus: PENDING (waiting to be included in a block)");
                        return Ok(());
                    }
                }
            }

            println!("\n✗ Transaction not found in mempool");
            println!("\nThe transaction may have:");
            println!("  - Already been included in a block");
            println!("  - Been rejected by the network");
            println!("  - Not yet propagated to this node");
        }

        Ok(())
    }
}
