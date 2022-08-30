use binprot::BinProtRead;
use mina_p2p_messages::{
    gossip::GossipNetMessageV2,
    rpc::VersionedRpcMenuV1,
    rpc_kernel::{Message, RpcMethod},
};

mod utils;

#[test]
fn jsonify_rpc_menu() {
    let data = utils::read("v1/rpc/menu/response/40160.bin").unwrap();
    let mut p = data.as_slice();
    let response =
        Message::<<VersionedRpcMenuV1 as RpcMethod>::Response>::binprot_read(&mut p).unwrap();
    let response_json = serde_json::to_value(&response).unwrap();
    let expected_json = serde_json::json!(
        {
            "Response": {
                "data": {
                    "Ok": [
                        [
                            "get_some_initial_peers",
                            1
                        ],
                        [
                            "get_staged_ledger_aux_and_pending_coinbases_at_hash",
                            1
                        ],
                        [
                            "answer_sync_ledger_query",
                            1
                        ],
                        [
                            "get_best_tip",
                            1
                        ],
                        [
                            "get_ancestry",
                            1
                        ],
                        [
                            "Get_transition_knowledge",
                            1
                        ],
                        [
                            "get_transition_chain",
                            1
                        ],
                        [
                            "get_transition_chain_proof",
                            1
                        ],
                        [
                            "ban_notify",
                            1
                        ],
                        [
                            "get_epoch_ledger",
                            1
                        ]
                    ]
                },
                "id": 330,
            }
        }
    );
    assert_eq!(response_json, expected_json);
}

#[test]
fn jsonify_gossip_v2_roundtrip() {
    utils::for_all("v2/gossip", |_, mut encoded| {
        let from_bin_prot = GossipNetMessageV2::binprot_read(&mut encoded).unwrap();
        let json = serde_json::to_value(&from_bin_prot).unwrap();
        let from_json = serde_json::from_value(json).unwrap();
        assert_eq!(from_bin_prot, from_json);
    })
    .unwrap();
}
