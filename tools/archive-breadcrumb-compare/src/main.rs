use std::path::PathBuf;
use mina_p2p_messages::v2::ArchiveTransitionFronntierDiff;

use clap::Parser;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tokio::time::{interval, Duration, timeout};
use serde_json::Value;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// OCaml Node GraphQL endpoint
    #[arg(env = "OCAML_NODE_GRAPHQL")]
    ocaml_node_graphql: Option<String>,

    /// OCaml Node directory path
    #[arg(env = "OCAML_NODE_DIR", required = true)]
    ocaml_node_dir: PathBuf,

    /// Openmina Node GraphQL endpoint
    #[arg(env = "OPENMINA_NODE_GRAPHQL")]
    openmina_node_graphql: Option<String>,

    /// Openmina Node directory path
    #[arg(env = "OPENMINA_NODE_DIR", required = true)]
    openmina_node_dir: PathBuf,
}

#[derive(Serialize)]
struct GraphQLQuery {
    query: String,
}

#[derive(Deserialize, Debug)]
struct SyncStatusResponse {
    data: SyncStatusData,
}

#[derive(Deserialize, Debug)]
struct SyncStatusData {
    sync_status: String,
}

#[derive(Deserialize, Debug)]
struct BlockInfo {
    state_hash: String,
}

#[derive(Deserialize, Debug)]
struct BestChainResponse {
    data: BestChainData,
}

#[derive(Deserialize, Debug)]
struct BestChainData {
    best_chain: Vec<BlockInfo>,
}

async fn check_sync_status(endpoint: &str) -> Result<String> {
    let client = reqwest::Client::new();
    
    let query = GraphQLQuery {
        query: "query MyQuery { syncStatus }".to_string(),
    };

    let response = client
        .post(endpoint)
        .json(&query)
        .send()
        .await?
        .json::<SyncStatusResponse>()
        .await?;

    Ok(response.data.sync_status)
}

async fn get_best_chain(endpoint: &str) -> Result<Vec<String>> {
    let client = reqwest::Client::new();
    
    let query = GraphQLQuery {
        query: "query MyQuery { bestChain(maxLength: 290) { stateHash } }".to_string(),
    };

    let response = client
        .post(endpoint)
        .json(&query)
        .send()
        .await?
        .json::<BestChainResponse>()
        .await?;

    Ok(response.data.best_chain.into_iter().map(|block| block.state_hash).collect())
}

async fn wait_for_sync(endpoint: &str, node_name: &str) -> Result<()> {
    const TIMEOUT_DURATION: Duration = Duration::from_secs(300); // 5 minutes timeout
    const CHECK_INTERVAL: Duration = Duration::from_secs(5);

    let sync_check = async {
        let mut interval = interval(CHECK_INTERVAL);
        
        loop {
            interval.tick().await;
            
            let status = check_sync_status(endpoint).await?;
            println!("{} sync status: {}", node_name, status);
            
            if status == "SYNCED" {
                return Ok(());
            }
            
            println!("Waiting for {} to sync...", node_name);
        }
    };

    timeout(TIMEOUT_DURATION, sync_check)
        .await
        .map_err(|_| anyhow::anyhow!("Timeout waiting for {} to sync after {:?}", node_name, TIMEOUT_DURATION))?
}

async fn compare_chains(ocaml_endpoint: &str, openmina_endpoint: &str) -> Result<Vec<String>> {
    const MAX_RETRIES: u32 = 3;
    const RETRY_INTERVAL: Duration = Duration::from_secs(5);
    let mut interval = interval(RETRY_INTERVAL);

    for attempt in 1..=MAX_RETRIES {
        println!("\nAttempting chain comparison (attempt {}/{})", attempt, MAX_RETRIES);
        
        let ocaml_chain = get_best_chain(ocaml_endpoint).await?;
        let openmina_chain = get_best_chain(openmina_endpoint).await?;

        println!("Chain comparison:");
        println!("OCaml chain length: {}", ocaml_chain.len());
        println!("Openmina chain length: {}", openmina_chain.len());

        // Try to compare chains
        if let Err(e) = compare_chain_data(&ocaml_chain, &openmina_chain) {
            if attempt == MAX_RETRIES {
                return Err(e);
            }
            println!("Comparison failed: {}. Retrying in 5s...", e);
            interval.tick().await;
            continue;
        }

        println!("✅ Chains match perfectly!");
        return Ok(ocaml_chain);
    }

    unreachable!()
}

fn compare_chain_data(ocaml_chain: &[String], openmina_chain: &[String]) -> Result<()> {
    if ocaml_chain.len() != openmina_chain.len() {
        anyhow::bail!(
            "Chain lengths don't match! OCaml: {}, Openmina: {}", 
            ocaml_chain.len(), 
            openmina_chain.len()
        );
    }

    for (i, (ocaml_hash, openmina_hash)) in ocaml_chain.iter().zip(openmina_chain.iter()).enumerate() {
        if ocaml_hash != openmina_hash {
            anyhow::bail!(
                "Chain mismatch at position {}: \nOCaml: {}\nOpenmina: {}", 
                i, 
                ocaml_hash, 
                openmina_hash
            );
        }
    }

    Ok(())
}

#[derive(Debug)]
struct DiffMismatch {
    state_hash: String,
    reason: String,
}

async fn compare_binary_diffs(
    ocaml_dir: PathBuf,
    openmina_dir: PathBuf,
    state_hashes: &[String],
) -> Result<Vec<DiffMismatch>> {
    let mut mismatches = Vec::new();

    if state_hashes.is_empty() {
        println!("No state hashes provided, comparing all diffs");
        let files = openmina_dir.read_dir()?;
        files.for_each(|file| {
            let file = file.unwrap();
            let file_name = file.file_name();
            let file_name_str = file_name.to_str().unwrap();
            let ocaml_path = ocaml_dir.join(file_name_str);
            let openmina_path = openmina_dir.join(file_name_str);

            // Load and deserialize both files
            let ocaml_diff = match load_and_deserialize(&ocaml_path) {
                Ok(diff) => diff,
                Err(e) => {
                    mismatches.push(DiffMismatch {
                        state_hash: file_name_str.to_string(),
                        reason: format!("Failed to load OCaml diff: {}", e),
                    });
                    return;
                }
            };
    
            let openmina_diff = match load_and_deserialize(&openmina_path) {
                Ok(diff) => diff,
                Err(e) => {
                    mismatches.push(DiffMismatch {
                            state_hash: file_name_str.to_string(),
                        reason: format!("Failed to load Openmina diff: {}", e),
                    });
                    return;
                }
            };
    
            // Compare the diffs
            if let Some(reason) = compare_diffs(&ocaml_diff, &openmina_diff) {
                mismatches.push(DiffMismatch {
                    state_hash: file_name_str.to_string(),
                    reason,
                });
            }
        });
        Ok(mismatches)
    } else {
        for state_hash in state_hashes {
            let ocaml_path = ocaml_dir.join(format!("{}.bin", state_hash));
            let openmina_path = openmina_dir.join(format!("{}.bin", state_hash));
    
            // Load and deserialize both files
            let ocaml_diff = match load_and_deserialize(&ocaml_path) {
                Ok(diff) => diff,
                Err(e) => {
                    mismatches.push(DiffMismatch {
                        state_hash: state_hash.clone(),
                        reason: format!("Failed to load OCaml diff: {}", e),
                    });
                    continue;
                }
            };
    
            let openmina_diff = match load_and_deserialize(&openmina_path) {
                Ok(diff) => diff,
                Err(e) => {
                    mismatches.push(DiffMismatch {
                        state_hash: state_hash.clone(),
                        reason: format!("Failed to load Openmina diff: {}", e),
                    });
                    continue;
                }
            };
    
            // Compare the diffs
            if let Some(reason) = compare_diffs(&ocaml_diff, &openmina_diff) {
                mismatches.push(DiffMismatch {
                    state_hash: state_hash.clone(),
                    reason,
                });
            }
        }
        Ok(mismatches)
    }
}

fn load_and_deserialize(path: &PathBuf) -> Result<ArchiveTransitionFronntierDiff> {
    let data = std::fs::read(path)?;
    let diff = binprot::BinProtRead::binprot_read(&mut data.as_slice())?;
    Ok(diff)
}

fn compare_diffs(
    ocaml: &ArchiveTransitionFronntierDiff,
    openmina: &ArchiveTransitionFronntierDiff,
) -> Option<String> {
    match (ocaml, openmina) {
        (
            ArchiveTransitionFronntierDiff::BreadcrumbAdded { block: b1, accounts_accessed: a1, accounts_created: c1, tokens_used: t1, sender_receipt_chains_from_parent_ledger: s1 },
            ArchiveTransitionFronntierDiff::BreadcrumbAdded { block: b2, accounts_accessed: a2, accounts_created: c2, tokens_used: t2, sender_receipt_chains_from_parent_ledger: s2 }
        ) => {
            if b1 != b2 {
                let ocaml_json = serde_json::to_string_pretty(&serde_json::to_value(b1).unwrap()).unwrap();
                let openmina_json = serde_json::to_string_pretty(&serde_json::to_value(b2).unwrap()).unwrap();
                return Some(format!("Block data mismatch:\nOCaml:\n{}\nOpenmina:\n{}", ocaml_json, openmina_json));
            }
            if a1 != a2 {
                let ocaml_json = serde_json::to_string_pretty(&serde_json::to_value(a1).unwrap()).unwrap();
                let openmina_json = serde_json::to_string_pretty(&serde_json::to_value(a2).unwrap()).unwrap();
                return Some(format!("Accounts accessed mismatch:\nOCaml:\n{}\nOpenmina:\n{}", ocaml_json, openmina_json));
            }
            if c1 != c2 {
                let ocaml_json = serde_json::to_string_pretty(&serde_json::to_value(c1).unwrap()).unwrap();
                let openmina_json = serde_json::to_string_pretty(&serde_json::to_value(c2).unwrap()).unwrap();
                return Some(format!("Accounts created mismatch:\nOCaml:\n{}\nOpenmina:\n{}", ocaml_json, openmina_json));
            }
            if t1 != t2 {
                let ocaml_json = serde_json::to_string_pretty(&serde_json::to_value(t1).unwrap()).unwrap();
                let openmina_json = serde_json::to_string_pretty(&serde_json::to_value(t2).unwrap()).unwrap();
                return Some(format!("Tokens used mismatch:\nOCaml:\n{}\nOpenmina:\n{}", ocaml_json, openmina_json));
            }
            if s1 != s2 {
                let ocaml_json = serde_json::to_string_pretty(&serde_json::to_value(s1).unwrap()).unwrap();
                let openmina_json = serde_json::to_string_pretty(&serde_json::to_value(s2).unwrap()).unwrap();
                return Some(format!("Sender receipt chains mismatch:\nOCaml:\n{}\nOpenmina:\n{}", ocaml_json, openmina_json));
            }
            None
        }
        _ => {
            let ocaml_json = serde_json::to_string_pretty(&serde_json::to_value(ocaml).unwrap()).unwrap();
            let openmina_json = serde_json::to_string_pretty(&serde_json::to_value(openmina).unwrap()).unwrap();
            Some(format!("Different diff types:\nOCaml:\n{}\nOpenmina:\n{}", ocaml_json, openmina_json))
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut best_chain = Vec::new();

    if let (Some(ocaml_graphql), Some(openmina_graphql)) = (args.ocaml_node_graphql, args.openmina_node_graphql) {
        // Wait for both nodes to be synced
        println!("Waiting for nodes to sync...");
        wait_for_sync(&ocaml_graphql, "OCaml Node").await?;
        wait_for_sync(&openmina_graphql, "Openmina Node").await?;
        println!("Both nodes are synced! ✅\n");
         // Compare chains with retry logic
        let bc = compare_chains(&ocaml_graphql, &openmina_graphql).await?;
        println!("Comparing binary diffs for {} blocks...", bc.len());
        best_chain.extend_from_slice(&bc);
    } else {
        println!("No graphql endpoints provided, skipping chain comparison");
    }

    let mismatches = compare_binary_diffs(
        args.ocaml_node_dir,
        args.openmina_node_dir,
        &best_chain,
    ).await?;

    if mismatches.is_empty() {
        println!("✅ All binary diffs match perfectly!");
    } else {
        println!("\n❌ Found {} mismatches:", mismatches.len());
        for (i, mismatch) in mismatches.iter().enumerate() {
            println!(
                "\nMismatch #{}: \nState Hash: {}\nReason: {}", 
                i + 1,
                mismatch.state_hash,
                mismatch.reason
            );
        }
        anyhow::bail!("Binary diff comparison failed");
    }

    Ok(())
}
