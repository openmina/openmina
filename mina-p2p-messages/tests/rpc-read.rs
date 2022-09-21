use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use binprot::BinProtRead;
use mina_p2p_messages::{
    rpc::{
        AnswerSyncLedgerQueryV1, AnswerSyncLedgerQueryV2, GetAncestryV1, GetAncestryV2,
        GetBestTipV1, GetBestTipV2, GetEpochLedgerV1,
        GetStagedLedgerAuxAndPendingCoinbasesAtHashV1,
        GetStagedLedgerAuxAndPendingCoinbasesAtHashV2, GetTransitionChainProofV1,
        GetTransitionChainProofV1ForV2, GetTransitionChainV1, GetTransitionChainV2,
        GetTransitionKnowledgeV1, VersionedRpcMenuV1,
    },
    rpc_kernel::{Message, MessageHeader, RpcMethod, Tag},
    utils::get_sized_slice,
    versioned::Ver,
};
use utils::for_all_with_path;

use crate::utils::files_path;

#[macro_use]
mod utils;

macro_rules! rpc_read_test {
    (ignore($reason:literal), $name:ident, $path:expr, $ty:ty) => {
        #[test]
        #[ignore = $reason]
        fn $name() {
            utils::for_all(concat!($path, "/query"), |encoded| {
                utils::assert_binprot_read::<Message<<$ty as RpcMethod>::Query>>(&encoded)
            })
            .unwrap();
            utils::for_all(concat!($path, "/response"), |encoded| {
                utils::assert_binprot_read::<Message<<$ty as RpcMethod>::Response>>(&encoded)
            })
            .unwrap();
        }
    };
    ($name:ident, $path:expr, $ty:ty) => {
        #[test]
        fn $name() {
            utils::for_all(concat!($path, "/query"), |encoded| {
                utils::assert_binprot_read::<Message<<$ty as RpcMethod>::Query>>(&encoded)
            })
            .unwrap();
            utils::for_all(concat!($path, "/response"), |encoded| {
                utils::assert_binprot_read::<Message<<$ty as RpcMethod>::Response>>(&encoded)
            })
            .unwrap();
        }
    };
}

rpc_read_test!(menu, "rpc-debugger/menu", VersionedRpcMenuV1);

rpc_read_test!(
    get_epoch_ledger_v1,
    "rpc/get-epoch-ledger",
    GetEpochLedgerV1
);

rpc_read_test!(get_best_tip_v1, "rpc-debugger/get-best-tip", GetBestTipV1);
rpc_read_test!(get_best_tip_v2, "v2/rpc/get-best-tip", GetBestTipV2);

rpc_read_test!(
    get_staged_ledger_aux_v1,
    "rpc-debugger/get-staged-ledger-aux",
    GetStagedLedgerAuxAndPendingCoinbasesAtHashV1
);

rpc_read_test!(
    get_staged_ledger_aux_v2,
    "v2/rpc/get-staged-ledger-aux",
    GetStagedLedgerAuxAndPendingCoinbasesAtHashV2
);

rpc_read_test!(
    answer_sync_ledger_v1,
    "rpc-debugger/answer-sync-ledger",
    AnswerSyncLedgerQueryV1
);

rpc_read_test!(
    answer_sync_ledger_v2,
    "v2/rpc/answer-sync-ledger",
    AnswerSyncLedgerQueryV2
);

rpc_read_test!(
    get_transition_chain_v1,
    "rpc-debugger/get-transition-chain",
    GetTransitionChainV1
);

rpc_read_test!(
    get_transition_chain_v2,
    "v2/rpc/get-transition-chain",
    GetTransitionChainV2
);

rpc_read_test!(
    get_transition_chain_proof_v1,
    "rpc-debugger/get-transition-chain-proof",
    GetTransitionChainProofV1
);

rpc_read_test!(
    get_transition_chain_proof_v2,
    "v2/rpc/get-transition-chain-proof",
    GetTransitionChainProofV1ForV2
);

rpc_read_test!(
    ignore("No test data"),
    get_transition_knowledge,
    "rpc-debugger/get-transition-knowledge",
    GetTransitionKnowledgeV1
);

rpc_read_test!(get_ancestry_v1, "rpc-debugger/get-ancestry", GetAncestryV1);

rpc_read_test!(get_ancestry_v2, "v2/rpc/get-ancestry", GetAncestryV2);

///////
///////
#[test]
#[ignore = "not test, but utility"]
fn make_rpc_v2() {
    #[derive(Debug, Default)]
    struct T {
        tag: Option<String>,
        query: Option<Vec<u8>>,
        response: Option<Vec<u8>>,
    }
    let mut mapping: BTreeMap<i64, T> = BTreeMap::new();
    utils::for_all("rpc-v2", |encoded| {
        utils::stream_read_with::<MessageHeader, _>(&encoded, |header, slice| match header {
            Ok(MessageHeader::Heartbeat) => {}
            Ok(MessageHeader::Query(q)) => {
                let t = mapping.entry(q.id).or_default();
                t.tag = Some(q.tag.to_string_lossy());
                t.query = Some(get_sized_slice(slice).unwrap().to_vec());
            }
            Ok(MessageHeader::Response(r)) => {
                let t = mapping.entry(r.id).or_default();
                t.response = Some(get_sized_slice(slice).unwrap().to_vec());
            }
            Err(e) => eprintln!("error: {e}"),
        })
    })
    .unwrap();
    let mut count = BTreeMap::new();
    for (
        _,
        T {
            tag,
            query,
            response,
        },
    ) in mapping
    {
        let c: &mut usize = count.entry(tag.clone()).or_default();
        if *c < 30 {
            if let (Some(tag), Some(query), Some(response)) = (tag, query, response) {
                let path = files_path(format!("v2/rpc/{tag}")).unwrap();
                let query_path = path.join("query");
                let response_path = path.join("response");
                fs::create_dir_all(&query_path).unwrap();
                fs::create_dir_all(&response_path).unwrap();
                let file = format!("{c:#02}.bin");
                File::create(query_path.join(&file))
                    .and_then(|mut f| f.write_all(&query))
                    .unwrap();
                File::create(response_path.join(&file))
                    .and_then(|mut f| f.write_all(&response))
                    .unwrap();
                *c += 1;
            }
        }
    }
}

#[test]
#[ignore]
fn debugger_to_wire() {
    for d in [
        "rpc-debugger/menu",
        "rpc-debugger/get-best-tip",
        "rpc-debugger/get-staged-ledger-aux",
        "rpc-debugger/answer-sync-ledger",
        "rpc-debugger/get-transition-chain",
        "rpc-debugger/get-transition-chain-proof",
        "rpc-debugger/get-transition-knowledge",
        "rpc-debugger/get-ancestry",
    ] {
        for_all_with_path(&PathBuf::from(d).join("response"), |encoded, path| {
            let mut p = &encoded[1..];
            let tag = Tag::binprot_read(&mut p).unwrap().to_string_lossy();
            let ver = Ver::binprot_read(&mut p).unwrap();
            println!("{tag}:{ver}");
            File::create(path)
                .and_then(|mut f| {
                    f.write(&encoded[..1])?;
                    f.write_all(p)?;
                    Ok(f)
                })
                .unwrap();
        })
        .unwrap()
    }
}
