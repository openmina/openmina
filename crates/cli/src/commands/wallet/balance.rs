use anyhow::Context;
use mina_node_account::AccountSecretKey;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, clap::Args)]
pub struct Balance {
    /// Public key to query the balance for
    #[arg(long, conflicts_with = "from")]
    pub address: Option<String>,

    /// Path to encrypted key file
    #[arg(long, conflicts_with = "address")]
    pub from: Option<PathBuf>,

    /// Password to decrypt the key
    #[arg(
        env = "MINA_PRIVKEY_PASS",
        default_value = "",
        help = "Password to decrypt the key (env: MINA_PRIVKEY_PASS)"
    )]
    pub password: String,

    /// GraphQL endpoint URL
    #[arg(
        long,
        default_value = "http://localhost:3000/graphql",
        help = "GraphQL endpoint URL"
    )]
    pub endpoint: String,

    /// Output format (text or json)
    #[arg(long, default_value = "text")]
    pub format: OutputFormat,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Serialize)]
struct GraphQLRequest {
    query: String,
    variables: serde_json::Value,
}

#[derive(Deserialize, Debug)]
struct GraphQLResponse {
    data: Option<DataResponse>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Deserialize, Debug)]
struct DataResponse {
    account: Option<AccountResponse>,
}

#[derive(Deserialize, Debug)]
struct AccountResponse {
    balance: BalanceResponse,
    nonce: String,
    #[serde(rename = "delegateAccount")]
    delegate_account: Option<DelegateAccount>,
}

#[derive(Deserialize, Debug)]
struct BalanceResponse {
    total: String,
    liquid: Option<String>,
    locked: Option<String>,
}

#[derive(Deserialize, Debug)]
struct DelegateAccount {
    #[serde(rename = "publicKey")]
    public_key: String,
}

#[derive(Deserialize, Debug)]
struct GraphQLError {
    message: String,
}

#[derive(Serialize, Debug)]
struct BalanceOutput {
    account: String,
    balance: BalanceOutputData,
    nonce: String,
    delegate: Option<String>,
}

#[derive(Serialize, Debug)]
struct BalanceOutputData {
    total: String,
    total_mina: String,
    liquid: Option<String>,
    liquid_mina: Option<String>,
    locked: Option<String>,
    locked_mina: Option<String>,
}

impl Balance {
    pub fn run(self) -> anyhow::Result<()> {
        // Get the public key either from address or from key file
        let public_key = if let Some(address) = self.address {
            address
        } else if let Some(from) = self.from {
            if self.password.is_empty() {
                anyhow::bail!(
                    "Password is required when using --from. Provide it via --password argument or MINA_PRIVKEY_PASS environment variable"
                );
            }
            let secret_key = AccountSecretKey::from_encrypted_file(&from, &self.password)
                .with_context(|| format!("Failed to decrypt key file: {}", from.display()))?;
            secret_key.public_key().to_string()
        } else {
            anyhow::bail!("Either --address or --from must be provided to specify the account");
        };

        // GraphQL query
        let query = r#"
            query GetBalance($publicKey: String!) {
                account(publicKey: $publicKey) {
                    balance {
                        total
                        liquid
                        locked
                    }
                    nonce
                    delegateAccount {
                        publicKey
                    }
                }
            }
        "#;

        let variables = serde_json::json!({
            "publicKey": public_key
        });

        let request = GraphQLRequest {
            query: query.to_string(),
            variables,
        };

        // Make the GraphQL request
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(&self.endpoint)
            .json(&request)
            .send()
            .with_context(|| format!("Failed to connect to GraphQL endpoint: {}", self.endpoint))?;

        if !response.status().is_success() {
            anyhow::bail!("GraphQL request failed with status: {}", response.status());
        }

        let graphql_response: GraphQLResponse = response
            .json()
            .context("Failed to parse GraphQL response")?;

        // Check for GraphQL errors
        if let Some(errors) = graphql_response.errors {
            let error_messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
            anyhow::bail!("GraphQL errors: {}", error_messages.join(", "));
        }

        // Extract account data
        let account = graphql_response
            .data
            .and_then(|d| d.account)
            .with_context(|| format!("Account not found: {}", public_key))?;

        // Create output structure
        let output = BalanceOutput {
            account: public_key.clone(),
            balance: BalanceOutputData {
                total: account.balance.total.clone(),
                total_mina: format_balance(&account.balance.total),
                liquid: account.balance.liquid.clone(),
                liquid_mina: account.balance.liquid.as_ref().map(|l| format_balance(l)),
                locked: account.balance.locked.clone(),
                locked_mina: account.balance.locked.as_ref().map(|l| format_balance(l)),
            },
            nonce: account.nonce.clone(),
            delegate: account
                .delegate_account
                .as_ref()
                .map(|d| d.public_key.clone()),
        };

        // Display the balance information based on format
        match self.format {
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(&output)
                    .context("Failed to serialize output to JSON")?;
                println!("{}", json);
            }
            OutputFormat::Text => {
                println!("Account: {}", output.account);
                println!();
                println!("Balance:");
                println!("  Total:  {} MINA", output.balance.total_mina);

                if let Some(liquid_mina) = &output.balance.liquid_mina {
                    println!("  Liquid: {} MINA", liquid_mina);
                }

                if let Some(locked_mina) = &output.balance.locked_mina {
                    println!("  Locked: {} MINA", locked_mina);
                }

                println!();
                println!("Nonce: {}", output.nonce);

                if let Some(delegate) = &output.delegate {
                    println!();
                    println!("Delegate: {}", delegate);
                }
            }
        }

        Ok(())
    }
}

fn format_balance(nanomina: &str) -> String {
    // Convert nanomina to MINA (1 MINA = 1,000,000,000 nanomina)
    let nano = nanomina.parse::<u64>().unwrap_or(0);
    let mina = nano as f64 / 1_000_000_000.0;
    format!("{:.9}", mina)
}
