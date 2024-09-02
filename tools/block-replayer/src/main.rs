use flate2::read::GzDecoder;
use kimchi::verifier_index::VerifierIndex;
use mina_curves::pasta::Vesta;
use mina_signer::CurvePoint as Pallas;
use poly_commitment::srs::SRS;
use std::{path::PathBuf, process::exit, str::FromStr, sync::Arc};

use binprot::macros::{BinProtRead, BinProtWrite};
use ledger::{
    dummy::dummy_blockchain_proof,
    scan_state::{
        currency::Slot,
        transaction_logic::{local_state::LocalState, protocol_state::protocol_state_view},
    },
    staged_ledger::{
        diff::Diff,
        staged_ledger::{SkipVerification, StagedLedger},
    },
    verifier::Verifier,
    BaseLedger as _,
};
use mina_p2p_messages::{
    list::List,
    v2::{
        self, BlockTimeTimeStableV1, MinaStateProtocolStateValueStableV2, ProtocolVersionStableV2,
        StagedLedgerDiffDiffStableV2,
    },
};
use node::{
    account::AccountSecretKey,
    block_producer::calc_epoch_seed,
    p2p::channels::rpc::BestTipWithProof,
    transition_frontier::{
        genesis::{
            empty_block_body, empty_block_body_hash, empty_pending_coinbase_hash, GenesisConfig,
        },
        sync::ledger::staged,
    },
};
use openmina_core::{
    block::{genesis::genesis_and_negative_one_protocol_states, BlockHash, BlockWithHash},
    constants::{constraint_constants, PROTOCOL_VERSION},
    NetworkConfig,
};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Args {
    /// Path to the genesis config file
    #[structopt(long, parse(from_os_str))]
    genesis_config: PathBuf,

    /// Path to the blocks file
    #[structopt(long, parse(from_os_str))]
    blocks_path: PathBuf,
}

fn main() {
    let args = Args::from_args();

    // TODO: from args
    NetworkConfig::init("mainnet").unwrap();

    let verifier_index = ledger::proofs::verifier_index::get_verifier_index(
        ledger::proofs::verifier_index::VerifierKind::Blockchain,
    );
    let srs = ledger::verifier::get_srs::<mina_hasher::Fp>();

    let config = GenesisConfig::DaemonJsonFile(args.genesis_config);
    let blocks_path = args.blocks_path;
    let (masks, loaded) = config.load().unwrap();

    let mut entries: Vec<PathBuf> = std::fs::read_dir(&blocks_path)
        .unwrap()
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();

    entries.sort();

    /*pub struct StagedLedger {
        scan_state: ScanState,
        ledger: Mask,
        constraint_constants: ConstraintConstants,
        pending_coinbase_collection: PendingCoinbase,
    } */

    let mask = masks.first().unwrap().clone();
    //let constraint_constants = openmina_core::network::mainnet::CONSTRAINT_CONSTANTS;
    let mut staged_ledger = StagedLedger::create_exn(constraint_constants().clone(), mask).unwrap();
    println!("++ Genesis staged ledger hash: {:?}", staged_ledger.hash());

    let genesis_vrf = ::vrf::genesis_vrf(loaded.staking_epoch_seed.clone()).unwrap();
    let genesis_vrf_hash = genesis_vrf.hash();

    let (negative_one, genesis) = genesis_and_negative_one_protocol_states(
        loaded.constants.clone(),
        loaded.genesis_ledger_hash.clone(),
        loaded.genesis_total_currency.clone(),
        loaded.staking_epoch_ledger_hash.clone(),
        loaded.staking_epoch_total_currency.clone(),
        loaded.next_epoch_ledger_hash.clone(),
        loaded.next_epoch_total_currency.clone(),
        AccountSecretKey::genesis_producer().public_key().into(),
        empty_pending_coinbase_hash(),
        (&LocalState::dummy()).into(),
        empty_block_body_hash(),
        genesis_vrf.into(),
        loaded.staking_epoch_seed.clone(),
        loaded.next_epoch_seed.clone(),
        calc_epoch_seed(&loaded.next_epoch_seed, genesis_vrf_hash),
    );

    let dummy_proof = dummy_blockchain_proof();
    let genesis_block: BlockWithHash<Arc<v2::MinaBlockBlockStableV2>> = BlockWithHash::new(
        v2::MinaBlockBlockStableV2 {
            header: v2::MinaBlockHeaderStableV2 {
                protocol_state: genesis.clone(),
                protocol_state_proof: (*dummy_proof).clone(),
                delta_block_chain_proof: (genesis.hash(), std::iter::empty().collect()),
                current_protocol_version: PROTOCOL_VERSION.clone(),
                proposed_protocol_version_opt: None,
            },
            body: v2::StagedLedgerDiffBodyStableV1 {
                staged_ledger_diff: empty_block_body(),
            },
        }
        .into(),
    );

    let mut protocol_states = vec![negative_one, genesis];

    let mut prev_protocol_state = genesis_block.header().protocol_state.clone();
    let mut pred_block: Option<v2::MinaBlockBlockStableV2> = None;

    for path in entries {
        if path
            .to_str()
            .unwrap()
            .contains("mainnet-359605-3NKTG8sg2vKQSUfe2D7nTxe1t4TDzRVubSxp4SUyHUWyXEpUVwqo")
        {
            continue;
        }
        let filename = path.file_name().unwrap().to_string_lossy();
        println!("Processing {}", filename);
        let sections: Vec<&str> = filename.strip_suffix(".json").unwrap().split('-').collect();
        //let block_hash = v2::StateHash::from_str(sections[2]).unwrap();
        let file = std::fs::File::open(&path).unwrap();
        let reader: Box<dyn std::io::Read> =
            if path.extension().and_then(|s| s.to_str()) == Some("gz") {
                Box::new(GzDecoder::new(file))
            } else {
                Box::new(std::io::BufReader::new(file))
            };

        let data: Data = serde_json::from_reader(reader).unwrap();
        continue;
        //protocol_states.push(data.data.protocol_state.clone());

        // Convert from parsed hash with ledger hash base58 byte to state hash
        let block_hash = v2::StateHash::from_fp(
            data.data
                .delta_transition_chain_proof
                .0
                .into_inner()
                .0
                .into(),
        );
        let header = v2::MinaBlockHeaderStableV2 {
            protocol_state: data.data.protocol_state.clone(),
            protocol_state_proof: data.data.protocol_state_proof.0.clone(),
            delta_block_chain_proof: (block_hash.clone(), List::new()),
            current_protocol_version: data.data.protocol_version.clone(),
            proposed_protocol_version_opt: data.data.proposed_protocol_version.clone(),
        };
        let now = std::time::Instant::now();
        if !ledger::proofs::verification::verify_block(
            &header,
            &verifier_index,
            &srs.lock().unwrap(),
        ) {
            println!("+++ FAILED BLOCK PROOF VERIFICATION");
            exit(1);
        }
        println!("++ block proof verification time={:?}", now.elapsed());
        //continue;

        let global_slot = data
            .data
            .protocol_state
            .body
            .consensus_state
            .global_slot_since_genesis
            .clone();
        let prev_state_view = protocol_state_view(&prev_protocol_state);

        let consensus_state = &data.data.protocol_state.body.consensus_state;
        let coinbase_receiver = (&consensus_state.coinbase_receiver).into();
        let supercharge_coinbase = consensus_state.supercharge_coinbase;

        let diff: Diff = (&data.data.staged_ledger_diff).into();

        let mut new_staged_ledger = staged_ledger.clone();
        let result = new_staged_ledger
            .apply(
                None,
                constraint_constants(),
                Slot::from_u32(global_slot.as_u32()),
                diff,
                (),
                &Verifier,
                &prev_state_view,
                ledger::scan_state::protocol_state::hashes(&prev_protocol_state),
                coinbase_receiver,
                supercharge_coinbase,
            )
            //.map_err(|err| format!("{err:?}"))
            .unwrap();
        //dbg!(&result);

        let staged_ledger_hash = data
            .data
            .protocol_state
            .body
            .blockchain_state
            .staged_ledger_hash
            .clone();
        let obtained_staged_ledger_hash =
            v2::MinaBaseStagedLedgerHashStableV1::from(&result.hash_after_applying);
        let expected_staged_ledger_hash = staged_ledger_hash.clone();
        //println!(
        //    "expected staged ledger hash: {}",
        //    serde_json::to_string_pretty(&expected_staged_ledger_hash).unwrap()
        //);
        //println!(
        //    "obtained staged ledger hash: {}",
        //    serde_json::to_string_pretty(&obtained_staged_ledger_hash).unwrap()
        //);
        //println!(
        //    "pending_coinbase_hash {:?}",
        //    staged_ledger_hash.pending_coinbase_hash.to_string()
        //);
        let expected_ledger_hash = staged_ledger_hash.non_snark.ledger_hash;
        let expected_pending_coinbase_hash = staged_ledger_hash.pending_coinbase_hash;
        let expected_aux_hash = staged_ledger_hash.non_snark.aux_hash;

        if expected_staged_ledger_hash != obtained_staged_ledger_hash {
            println!("Staged ledger hash mismatch");
            println!(
                "expected staged ledger hash: {}",
                serde_json::to_string_pretty(&expected_staged_ledger_hash).unwrap()
            );
            println!(
                "obtained staged ledger hash: {}",
                serde_json::to_string_pretty(&obtained_staged_ledger_hash).unwrap()
            );
            let block = v2::MinaBlockBlockStableV2 {
                header,
                body: v2::StagedLedgerDiffBodyStableV1 {
                    staged_ledger_diff: data.data.staged_ledger_diff.clone(),
                },
            };
            //let pred_block = v2::MinaBlockBlockStableV2 {
            //    header: genesis_block.header().clone(),
            //    body: genesis_block.body().clone(),
            //};
            //block.clone(); //pred_block.unwrap().clone();
            let pred_block = pred_block.unwrap().clone();
            handle_failure(
                &staged_ledger,
                expected_staged_ledger_hash,
                protocol_states,
                block,
                pred_block,
            );
            exit(1);
        }

        //        assert_eq!(
        //            v2::LedgerHash::from_fp(result.hash_after_applying.non_snark.ledger_hash),
        //            expected_ledger_hash,
        //            "ledger hash"
        //        );
        //
        //        if v2::PendingCoinbaseHash::from_fp(result.hash_after_applying.pending_coinbase_hash)
        //            != expected_pending_coinbase_hash
        //        {
        //            println!("+++ pending coinbase hash mismatch");
        //        }

        //assert_eq!(
        //    v2::PendingCoinbaseHash::from_fp(result.hash_after_applying.pending_coinbase_hash),
        //    expected_pending_coinbase_hash,
        //    "pending coinbase hash"
        //);
        //assert_eq!(
        //    result.hash_after_applying.non_snark.aux_hash,
        //    expected_aux_hash,
        //    "aux hash"
        //);

        pred_block = Some(v2::MinaBlockBlockStableV2 {
            header: v2::MinaBlockHeaderStableV2 {
                protocol_state: data.data.protocol_state.clone(),
                protocol_state_proof: data.data.protocol_state_proof.0.clone(),
                delta_block_chain_proof: (block_hash, List::new()),
                current_protocol_version: data.data.protocol_version.clone(),
                proposed_protocol_version_opt: data.data.proposed_protocol_version.clone(),
            },
            body: v2::StagedLedgerDiffBodyStableV1 {
                staged_ledger_diff: data.data.staged_ledger_diff.clone(),
            },
        });

        if true || pred_block.as_ref().unwrap().body.completed_works_count() > 1 {
            let best_tip_with_proof = BestTipWithProof {
                best_tip: Arc::new(pred_block.clone().unwrap()),
                proof: (
                    List::one(
                        pred_block
                            .clone()
                            .unwrap()
                            .header
                            .protocol_state
                            .body
                            .hash()
                            .clone()
                            .into(),
                    ),
                    Arc::new(pred_block.clone().unwrap()),
                ),
            };

            let x = serde_json::to_string(&best_tip_with_proof).unwrap();
            let mut file =
                std::fs::File::create("/tmp/best_tip_with_proof.json").expect("Failed to create file");
                use std::io::Write;
            // Write the JSON string to the file
            file.write_all(x.as_bytes())
                .expect("Failed to write to file");

            exit(0);
        }

        new_staged_ledger.commit_and_reparent_to_root();
        staged_ledger = new_staged_ledger;

        prev_protocol_state = data.data.protocol_state;

        // Now you can work with `data`
        //println!("Parsed data: {:?}", data);
        //break;
    }
}

fn handle_failure(
    staged_ledger: &StagedLedger,
    staged_ledger_hash: v2::MinaBaseStagedLedgerHashStableV1,
    protocol_states: Vec<v2::MinaStateProtocolStateValueStableV2>,
    block: v2::MinaBlockBlockStableV2,
    pred_block: v2::MinaBlockBlockStableV2,
) {
    match dump_application_to_file(
        staged_ledger,
        staged_ledger_hash,
        protocol_states,
        block,
        pred_block,
    ) {
        Ok(filename) => println!("Application context saved to: {filename:?}"),
        Err(e) => println!("Failed to save application context to file: {e:?}"),
    }
}

fn dump_application_to_file(
    staged_ledger: &StagedLedger,
    staged_ledger_hash: v2::MinaBaseStagedLedgerHashStableV1,
    protocol_states: Vec<v2::MinaStateProtocolStateValueStableV2>,
    block: v2::MinaBlockBlockStableV2,
    pred_block: v2::MinaBlockBlockStableV2,
) -> std::io::Result<String> {
    #[derive(BinProtRead, BinProtWrite)]
    struct ApplyContext {
        accounts: Vec<v2::MinaBaseAccountBinableArgStableV2>,
        scan_state: v2::TransactionSnarkScanStateStableV2,
        protocol_states: Vec<v2::MinaStateProtocolStateValueStableV2>,
        pending_coinbase: v2::MinaBasePendingCoinbaseStableV2,
        //staged_ledger_hash: v2::MinaBaseStagedLedgerHashStableV1,
        pred_block: v2::MinaBlockBlockStableV2,
        blocks: Vec<v2::MinaBlockBlockStableV2>,
    }

    let cs = &block.header.protocol_state.body.consensus_state;
    let block_height = cs.blockchain_length.as_u32();

    println!("++ create apply context");
    let apply_context = ApplyContext {
        accounts: staged_ledger
            .ledger()
            .to_list()
            .iter()
            .map(v2::MinaBaseAccountBinableArgStableV2::from)
            .collect::<Vec<_>>(),
        scan_state: staged_ledger.scan_state().into(),
        protocol_states,
        pending_coinbase: staged_ledger.pending_coinbase_collection().into(),
        //staged_ledger_hash,
        pred_block: pred_block.clone(),
        blocks: vec![block.clone()],
    };

    println!("++ dump");
    use mina_p2p_messages::binprot::BinProtWrite;
    let filename = format!("/tmp/failed_apply_ctx.binprot"); //, block_height);
    let mut file = std::fs::File::create(&filename)?;
    apply_context.binprot_write(&mut file)?;
    file.sync_all()?;

    Ok(filename)
}

#[derive(Clone, Debug, PartialEq, BinProtRead, BinProtWrite)]
pub struct Proof(pub v2::MinaBaseProofStableV2);

impl Serialize for Proof {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use base64::{engine::general_purpose::URL_SAFE, Engine as _};
        use binprot::BinProtWrite;
        let mut buf = Vec::new();
        self.0
            .binprot_write(&mut buf)
            .map_err(serde::ser::Error::custom)?;
        let base64_data = URL_SAFE.encode(&buf);
        serializer.serialize_str(&base64_data)
    }
}

impl<'de> Deserialize<'de> for Proof {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        use base64::{engine::general_purpose::URL_SAFE, Engine as _};
        let base64_data = String::deserialize(deserializer)?;
        let binprot_data = URL_SAFE
            .decode(&base64_data)
            .map_err(serde::de::Error::custom)?;
        let mut read = binprot_data.as_slice();
        let proof: v2::MinaBaseProofStableV2 =
            binprot::BinProtRead::binprot_read(&mut read).map_err(serde::de::Error::custom)?;
        Ok(Proof(proof))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]

pub struct PrecomputedBlock {
    pub scheduled_time: BlockTimeTimeStableV1,
    pub protocol_state: MinaStateProtocolStateValueStableV2,
    pub protocol_state_proof: Proof,
    pub staged_ledger_diff: StagedLedgerDiffDiffStableV2,
    // FIXME: for some reason in OCaml the base58check conversion for the JSON value
    // uses version byte = 0x05 (ledger hash) instead of 0x10 (StateHash) and 0x11 (StateBodyHash)
    pub delta_transition_chain_proof: (
        v2::LedgerHash,       // StateHash
        List<v2::LedgerHash>, // StateBodyHash
    ),
    pub protocol_version: ProtocolVersionStableV2,
    #[serde(default)]
    pub proposed_protocol_version: Option<ProtocolVersionStableV2>,
    //accounts_accessed: (), // (int * Account.t) list
    //accounts_created: (), // (Account_id.t * Currency.Fee.t) list
    //tokens_used: (), // (Token_id.t * Account_id.t option) list
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct Data {
    //pub version: String,
    pub data: PrecomputedBlock,
}

//#[test]
//fn test_parse() {
//    let json =
//        include_bytes!("mainnet-360608-3NLeXkK69MFcuvKrBhdhwrExQjB3jrcq9wTpCxGBbVvF8STWC1eo.json");
//
//    let data: Data = serde_json::from_slice(json).unwrap();
//
//    println!("{data:?}");
//}

//#[test]
//fn test_parse2() {
//    let json =
//        include_bytes!("mainnet-359960-3NLBefo5ZPfiBEVu2nh5kEWVkAmkrEN5ESNQgw1tngN7X3GqmLdP.json");
//
//    let data: Data = serde_json::from_slice(json).unwrap();
//
//    //println!("{data:?}");
//}

#[test]
fn debug() {
    // Example JSON data
    let json_data = r#"
     [
      "Signed_command",
      {
        "payload": {
          "common": {
            "fee": "0.5",
            "fee_payer_pk": "B62qpWaQoQoPL5AGta7Hz2DgJ9CJonpunjzCGTdw8KiCCD1hX8fNHuR",
            "nonce": "45767",
            "valid_until": "4294967295",
            "memo": "E4YVe5YCtgSZuaBo1RiwHFWqtPzV6Eur8xG6JnbzEigit5nZKobQG"
          },
          "body": [
            "Payment",
            {
              "receiver_pk": "B62qrRn3kpjrGGWeKjgGAS1vKViZmbY8h5NGHMxW8arLdGA1pbB6UvJ",
              "amount": "9280000000"
            }
          ]
        },
        "signer": "B62qpWaQoQoPL5AGta7Hz2DgJ9CJonpunjzCGTdw8KiCCD1hX8fNHuR",
        "signature": "7mXXvs9VwbNki2YteLKgzeCt5HaAjDxQR5a9eVbzYGVhGKx7ubfA63bkT3WUjgQJNZMXSrcmqd7YFfrvFh5JMHm63u1CZDQ4"
      }
    ]
    "#;

    let result: mina_p2p_messages::v2::MinaBaseUserCommandStableV2 =
        serde_json::from_str(json_data).unwrap();

    println!("{result:?}");
}
