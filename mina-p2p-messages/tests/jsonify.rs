use binprot::BinProtRead;
use gloo_utils::format::JsValueSerdeExt;
use mina_p2p_messages::{
    gossip::{GossipNetMessageV1, GossipNetMessageV2},
    rpc::VersionedRpcMenuV1,
    rpc_kernel::{Message, RpcMethod},
};
use wasm_bindgen::JsValue;

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

#[test]
fn jsonify_javascript_v1_roundtrip() {
    utils::for_all("v1/gossip", |_, mut encoded| {
        let msg = GossipNetMessageV1::binprot_read(&mut encoded).unwrap();
        let js_value = JsValue::from_serde(&msg).unwrap();
        let msg_from_js_value: GossipNetMessageV1 = js_value.into_serde().unwrap();

        println!("--------||||||||||||");
        println!("msg: {:?}", msg);
        println!("--------||||||||||||");
        println!("msg_from_js_value: {:?}", msg_from_js_value);
        println!("--------||||||||||||");

        assert_eq!(msg, msg_from_js_value);
    })
    .unwrap();
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use binprot::BinProtRead;
    use mina_p2p_messages::{gossip::GossipNetMessageV2, number::Int64};

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test::wasm_bindgen_test]
    fn jsonify_gossip_v2_roundtrip_wasm() {
        let new_state = include_bytes!("files/v2/gossip/new_state.bin");
        let snark_pool_diff = include_bytes!("files/v2/gossip/snark_pool_diff.bin");
        let tx_pool_diff = include_bytes!("files/v2/gossip/transaction_pool_diff.bin");

        for mut encoded in [&new_state[..], &snark_pool_diff[..], &tx_pool_diff[..]] {
            let from_bin_prot = GossipNetMessageV2::binprot_read(&mut encoded).unwrap();
            let json = serde_json::to_value(&from_bin_prot).unwrap();
            let from_json = serde_json::from_value(json).unwrap();
            assert_eq!(from_bin_prot, from_json);
        }
    }

    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen(inline_js = r#"
export function js_roundtrip(s) {
  return JSON.stringify(JSON.parse(s))
}
"#)]
    extern "C" {
        fn js_roundtrip(s: &str) -> String;
    }

    #[wasm_bindgen_test::wasm_bindgen_test]
    fn integer_roundtrip_wasm() {
        for i in [0_i64, 1, 256, 5688895253889439275] {
            let int = Int64::from(i);
            let json = serde_json::to_string(&int).unwrap();
            let json_1 = js_roundtrip(&json);
            let int_1 = serde_json::from_str(&json_1).unwrap();
            assert_eq!(json, json_1);
            assert_eq!(int, int_1);
        }
    }
}
