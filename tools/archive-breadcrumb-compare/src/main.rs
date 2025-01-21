use mina_p2p_messages::v2::ArchiveTransitionFronntierDiff;
use std::{collections::HashSet, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use tokio::time::{interval, timeout, Duration};

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

    /// Check for missing breadcrumbs
    #[arg(long)]
    check_missing: bool,
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
#[serde(rename_all = "camelCase")]
struct SyncStatusData {
    sync_status: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct BlockInfo {
    state_hash: String,
}

#[derive(Deserialize, Debug)]
struct BestChainResponse {
    data: BestChainData,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
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

    Ok(response
        .data
        .best_chain
        .into_iter()
        .map(|block| block.state_hash)
        .collect())
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

    timeout(TIMEOUT_DURATION, sync_check).await.map_err(|_| {
        anyhow::anyhow!(
            "Timeout waiting for {} to sync after {:?}",
            node_name,
            TIMEOUT_DURATION
        )
    })?
}

async fn compare_chains(ocaml_endpoint: &str, openmina_endpoint: &str) -> Result<Vec<String>> {
    const MAX_RETRIES: u32 = 3;
    const RETRY_INTERVAL: Duration = Duration::from_secs(5);
    let mut interval = interval(RETRY_INTERVAL);

    for attempt in 1..=MAX_RETRIES {
        println!(
            "\nAttempting chain comparison (attempt {}/{})",
            attempt, MAX_RETRIES
        );

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

    for (i, (ocaml_hash, openmina_hash)) in
        ocaml_chain.iter().zip(openmina_chain.iter()).enumerate()
    {
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
            ArchiveTransitionFronntierDiff::BreadcrumbAdded {
                block: (b1, (body_hash1, state_hash1)),
                accounts_accessed: a1,
                accounts_created: c1,
                tokens_used: t1,
                sender_receipt_chains_from_parent_ledger: s1,
            },
            ArchiveTransitionFronntierDiff::BreadcrumbAdded {
                block: (b2, (body_hash2, state_hash2)),
                accounts_accessed: a2,
                accounts_created: c2,
                tokens_used: t2,
                sender_receipt_chains_from_parent_ledger: s2,
            },
        ) => {
            let mut mismatches = Vec::new();

            if body_hash1 != body_hash2 {
                if body_hash1.is_some() {
                    mismatches.push(format!(
                        "Body hash mismatch:\nOCaml: {:?}\nOpenmina: {:?}",
                        body_hash1, body_hash2
                    ));
                }
            } else if state_hash1 != state_hash2 {
                mismatches.push(format!(
                    "State hash mismatch:\nOCaml: {}\nOpenmina: {}",
                    state_hash1, state_hash2
                ));
            } else if b1.header.protocol_state_proof != b2.header.protocol_state_proof {
                // Note this is not a real mismatch, we can have different protocol state proofs for the same block.
                // If both proofs are valid, we can ignore the mismatch.
                // Create a temporary copy of b1 with b2's proof for comparison
                let mut b1_with_b2_proof = b1.clone();
                b1_with_b2_proof.header.protocol_state_proof =
                    b2.header.protocol_state_proof.clone();

                if b1_with_b2_proof != b2 {
                    let ocaml_json =
                        serde_json::to_string_pretty(&serde_json::to_value(b1).unwrap()).unwrap();
                    let openmina_json =
                        serde_json::to_string_pretty(&serde_json::to_value(b2).unwrap()).unwrap();
                    mismatches.push(format!(
                        "Block data mismatch:\nOCaml:\n{}\nOpenmina:\n{}",
                        ocaml_json, openmina_json
                    ));
                }
            } else if b1 != b2 {
                let ocaml_json =
                    serde_json::to_string_pretty(&serde_json::to_value(b1).unwrap()).unwrap();
                let openmina_json =
                    serde_json::to_string_pretty(&serde_json::to_value(b2).unwrap()).unwrap();
                mismatches.push(format!(
                    "Block data mismatch:\nOCaml:\n{}\nOpenmina:\n{}",
                    ocaml_json, openmina_json
                ));
            }

            if a1 != a2 {
                let ids_ocaml = a1.iter().map(|(id, _)| id.as_u64()).collect::<HashSet<_>>();
                let ids_openmina = a2.iter().map(|(id, _)| id.as_u64()).collect::<HashSet<_>>();

                // Find missing IDs in openmina (present in ocaml but not in openmina)
                let missing_in_openmina: Vec<_> = ids_ocaml.difference(&ids_openmina).collect();
                // Find extra IDs in openmina (present in openmina but not in ocaml)
                let extra_in_openmina: Vec<_> = ids_openmina.difference(&ids_ocaml).collect();

                if !missing_in_openmina.is_empty() {
                    println!("Missing in Openmina: {:?}", missing_in_openmina);
                }
                if !extra_in_openmina.is_empty() {
                    println!("Extra in Openmina: {:?}", extra_in_openmina);
                }

                let ocaml_json =
                    serde_json::to_string_pretty(&serde_json::to_value(a1).unwrap()).unwrap();
                let openmina_json =
                    serde_json::to_string_pretty(&serde_json::to_value(a2).unwrap()).unwrap();
                mismatches.push(format!(
                    "Accounts accessed mismatch:\nOCaml:\n{}\nOpenmina:\n{}",
                    ocaml_json, openmina_json
                ));
            }
            if c1 != c2 {
                let ocaml_json =
                    serde_json::to_string_pretty(&serde_json::to_value(c1).unwrap()).unwrap();
                let openmina_json =
                    serde_json::to_string_pretty(&serde_json::to_value(c2).unwrap()).unwrap();
                mismatches.push(format!(
                    "Accounts created mismatch:\nOCaml:\n{}\nOpenmina:\n{}",
                    ocaml_json, openmina_json
                ));
            }
            if t1 != t2 {
                let ocaml_json =
                    serde_json::to_string_pretty(&serde_json::to_value(t1).unwrap()).unwrap();
                let openmina_json =
                    serde_json::to_string_pretty(&serde_json::to_value(t2).unwrap()).unwrap();
                mismatches.push(format!(
                    "Tokens used mismatch:\nOCaml:\n{}\nOpenmina:\n{}",
                    ocaml_json, openmina_json
                ));
            }
            if s1 != s2 {
                let ocaml_json =
                    serde_json::to_string_pretty(&serde_json::to_value(s1).unwrap()).unwrap();
                let openmina_json =
                    serde_json::to_string_pretty(&serde_json::to_value(s2).unwrap()).unwrap();
                mismatches.push(format!(
                    "Sender receipt chains mismatch:\nOCaml:\n{}\nOpenmina:\n{}",
                    ocaml_json, openmina_json
                ));
            }

            if mismatches.is_empty() {
                None
            } else {
                Some(mismatches.join("\n\n"))
            }
        }
        _ => {
            let ocaml_json =
                serde_json::to_string_pretty(&serde_json::to_value(ocaml).unwrap()).unwrap();
            let openmina_json =
                serde_json::to_string_pretty(&serde_json::to_value(openmina).unwrap()).unwrap();
            Some(format!(
                "Different diff types:\nOCaml:\n{}\nOpenmina:\n{}",
                ocaml_json, openmina_json
            ))
        }
    }
}

async fn check_missing_breadcrumbs(
    openmina_node_dir: PathBuf,
    openmina_endpoint: &str,
) -> Result<()> {
    let files = openmina_node_dir.read_dir()?;
    let best_chain = get_best_chain(openmina_endpoint).await?;
    let mut missing_breadcrumbs = Vec::new();

    let file_names = files
        .map(|file| {
            file.unwrap()
                .file_name()
                .to_str()
                .unwrap()
                .to_string()
                .strip_suffix(".bin")
                .unwrap()
                .to_owned()
        })
        .collect::<HashSet<String>>();

    for best_chain_hash in best_chain {
        if !file_names.contains(&best_chain_hash.to_string()) {
            missing_breadcrumbs.push(best_chain_hash.to_string());
        }
    }

    if !missing_breadcrumbs.is_empty() {
        println!(
            "❌ Found {} missing breadcrumbs:",
            missing_breadcrumbs.len()
        );
        for missing_breadcrumb in missing_breadcrumbs {
            println!("{}", missing_breadcrumb);
        }
    } else {
        println!("✅ All breadcrumbs present!");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut best_chain = Vec::new();

    println!("Checking for missing breadcrumbs...");

    if args.check_missing {
        check_missing_breadcrumbs(
            args.openmina_node_dir,
            args.openmina_node_graphql.as_deref().unwrap(),
        )
        .await?;
        return Ok(());
    }

    if let (Some(ocaml_graphql), Some(openmina_graphql)) =
        (args.ocaml_node_graphql, args.openmina_node_graphql)
    {
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

    let mismatches =
        compare_binary_diffs(args.ocaml_node_dir, args.openmina_node_dir, &best_chain).await?;

    if mismatches.is_empty() {
        println!("✅ All binary diffs match perfectly!");
    } else {
        println!("\n❌ Found {} mismatches:", mismatches.len());

        // let first_mismatch = mismatches.first().unwrap();
        // println!(
        //     "\nMismatch #{}: \nState Hash: {}\nReason: {}",
        //     1, first_mismatch.state_hash, first_mismatch.reason
        // );
        // println!("Another {} missmatches are pending", mismatches.len() - 1);
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
