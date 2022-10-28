use std::io::Read;

use mina_p2p_messages::gossip::GossipNetMessageV2;

use binprot::BinProtRead;

use mina_p2p_messages::{rpc::*, rpc_kernel::*, string::CharString, versioned::Ver};

#[allow(dead_code)]
pub fn tx_pool_diff() {
    const BYTES: &[u8] = include_bytes!("files/v2/gossip/transaction_pool_diff.bin");
    let mut p = BYTES;
    GossipNetMessageV2::binprot_read(&mut p).unwrap();
}

#[allow(dead_code)]
pub fn snark_pool_diff() {
    const BYTES: &[u8] = include_bytes!("files/v2/gossip/snark_pool_diff.bin");
    let mut p = BYTES;
    GossipNetMessageV2::binprot_read(&mut p).unwrap();
}

#[allow(dead_code)]
pub fn new_state() {
    const BYTES: &[u8] = include_bytes!("files/v2/gossip/new_state.bin");
    let mut p = BYTES;
    GossipNetMessageV2::binprot_read(&mut p).unwrap();
}

#[allow(dead_code)]
pub fn incoming_rpc() {
    static STREAM: &[u8] = include_bytes!("files/v2/rpc/catchup_in.bin");
    let mut p = STREAM;
    while !p.is_empty() {
        let (tag, version) = read_rpc_tag_version(&mut p).unwrap();
        read_rpc_response_untyped(&tag, &version, &mut p).unwrap();
    }
}

// #[test]
// #[ignore = "Memory allocation benches should be run individually"]
// fn incoming_rpc_one_by_one() {
//     static STREAM: &[u8] = include_bytes!("files/v2/rpc/catchup_in.bin");
//     let mut p = STREAM;
//     let mut max: Option<(f64, usize, MemoryStats, Tag, Ver)> = None;
//     while !p.is_empty() {
//         let (tag, version) = read_rpc_tag_version(&mut p).unwrap();
//         let pp = p;
//         let (_, stats) =
//             trace_allocs(|| read_rpc_response_untyped(&tag, &version, &mut p).unwrap());
//         let encoded = pp.len() - p.len();
//         let ratio = (stats.peak as f64) / (encoded as f64);
//         if ratio > 1.5 {
//             println!("{}:{}", tag.to_string_lossy(), version);
//             println!("Ratio: {}", ratio);
//             println!("Encoded size (B): {}", encoded);
//             println!("{stats}");
//         }
//         if matches!(&max, Some((r, ..)) if ratio > *r) || max.is_none() {
//             max = Some((ratio, encoded, stats, tag, version));
//         }
//     }
//     if let Some((ratio, encoded, stats, tag, version)) = max {
//         println!("{}:{}", tag.to_string_lossy(), version);
//         println!("Ratio: {}", ratio);
//         println!("Encoded size (B): {}", encoded);
//         println!("{stats}");
//     }
// }

pub fn staged_ledger() {
    static ENCODED: &[u8] = include_bytes!("files/v2/rpc/get-staged-ledger-aux/response/00.bin");
    let mut p = ENCODED;
    Message::<<GetStagedLedgerAuxAndPendingCoinbasesAtHashV2 as RpcMethod>::Response>::binprot_read(
        &mut p,
    )
    .unwrap();
}

fn read_rpc_tag_version<R: Read>(read: &mut R) -> Result<(CharString, Ver), binprot::Error> {
    Ok((CharString::binprot_read(read)?, Ver::binprot_read(read)?))
}

fn read_rpc_response_untyped<R: Read>(
    tag: &mina_p2p_messages::string::CharString,
    version: &i32,
    read: &mut R,
) -> anyhow::Result<()> {
    match (tag, version) {
        (tag, 1) if tag == "__Versioned_rpc.Menu" => {
            binprot_read_rpc_response::<VersionedRpcMenuV1, _>(read)?;
        }
        (tag, 1) if tag == "get_some_initial_peers" => {
            binprot_read_rpc_response::<GetSomeInitialPeersV1ForV2, _>(read)?;
        }
        (tag, 2) if tag == "get_staged_ledger_aux_and_pending_coinbases_at_hash" => {
            binprot_read_rpc_response::<GetStagedLedgerAuxAndPendingCoinbasesAtHashV2, _>(read)?;
        }
        (tag, 2) if tag == "answer_sync_ledger_query" => {
            binprot_read_rpc_response::<AnswerSyncLedgerQueryV2, _>(read)?;
        }
        (tag, 2) if tag == "get_transition_chain" => {
            binprot_read_rpc_response::<GetTransitionChainV2, _>(read)?;
        }
        (tag, 1) if tag == "get_transition_chain_proof" => {
            binprot_read_rpc_response::<GetTransitionChainProofV1ForV2, _>(read)?;
        }
        (tag, 1) if tag == "Get_transition_knowledge" => {
            binprot_read_rpc_response::<GetTransitionKnowledgeV1ForV2, _>(read)?;
        }
        (tag, 2) if tag == "get_ancestry" => {
            binprot_read_rpc_response::<GetAncestryV2, _>(read)?;
        }
        (tag, 1) if tag == "ban_notify" => {
            binprot_read_rpc_response::<BanNotifyV1, _>(read)?;
        }
        (tag, 2) if tag == "get_best_tip" => {
            binprot_read_rpc_response::<GetBestTipV2, _>(read)?;
        }
        (tag, 1) if tag == "get_node_status" => {
            binprot_read_rpc_response::<GetNodeStatusV1, _>(read)?;
        }
        (tag, 2) if tag == "get_node_status" => {
            binprot_read_rpc_response::<GetNodeStatusV2, _>(read)?;
        }
        (tag, 2) if tag == "get_epoch_ledger" => {
            binprot_read_rpc_response::<GetEpochLedgerV2, _>(read)?;
        }
        _ => anyhow::bail!("unexpected rpc ({tag}, {version})"),
    }
    Ok(())
}

fn binprot_read_rpc_response<T, R>(read: &mut R) -> anyhow::Result<()>
where
    T: RpcMethod,
    R: Read,
{
    let msg = Message::<T::Response>::binprot_read(read)?;
    if let Message::Response(_r) = &msg {
        Ok(())
    } else {
        anyhow::bail!("Response expected")
    }
}
