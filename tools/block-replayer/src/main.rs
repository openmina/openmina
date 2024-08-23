use flate2::read::GzDecoder;
use std::{path::PathBuf, sync::Arc};

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
    verifier::Verifier, BaseLedger as _,
};
use mina_p2p_messages::v2::{
    self, BlockTimeTimeStableV1, MinaStateProtocolStateValueStableV2, ProtocolVersionStableV2,
    StagedLedgerDiffDiffStableV2,
};
use node::{
    account::AccountSecretKey,
    block_producer::calc_epoch_seed,
    transition_frontier::genesis::{
        empty_block_body, empty_block_body_hash, empty_pending_coinbase_hash, GenesisConfig,
    },
};
use openmina_core::{
    block::{genesis::genesis_and_negative_one_protocol_states, ArcBlockWithHash, BlockWithHash},
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

    let mut prev_protocol_state = genesis_block.header().protocol_state.clone();

    for path in entries {
        if path
            .to_str()
            .unwrap()
            .contains("mainnet-359605-3NKTG8sg2vKQSUfe2D7nTxe1t4TDzRVubSxp4SUyHUWyXEpUVwqo")
        {
            continue;
        }
        println!("Processing {}", path.file_name().unwrap().to_string_lossy());
        let file = std::fs::File::open(&path).unwrap();
        let reader: Box<dyn std::io::Read> =
            if path.extension().and_then(|s| s.to_str()) == Some("gz") {
                Box::new(GzDecoder::new(file))
            } else {
                Box::new(std::io::BufReader::new(file))
            };

        let data: Data = serde_json::from_reader(reader).unwrap();
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

        let result = staged_ledger
            .apply(
                Some(SkipVerification::All),
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
        dbg!(&result);

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
        println!(
            "expected staged ledger hash: {}",
            serde_json::to_string_pretty(&expected_staged_ledger_hash).unwrap()
        );
        println!(
            "obtained staged ledger hash: {}",
            serde_json::to_string_pretty(&obtained_staged_ledger_hash).unwrap()
        );
        println!(
            "pending_coinbase_hash {:?}",
            staged_ledger_hash.pending_coinbase_hash.to_string()
        );
        let expected_ledger_hash = staged_ledger_hash.non_snark.ledger_hash;
        let expected_pending_coinbase_hash = staged_ledger_hash.pending_coinbase_hash;
        let expected_aux_hash = staged_ledger_hash.non_snark.aux_hash;

        assert_eq!(
            expected_staged_ledger_hash, obtained_staged_ledger_hash,
            "staged ledger hash"
        );
        assert_eq!(
            v2::LedgerHash::from_fp(result.hash_after_applying.non_snark.ledger_hash),
            expected_ledger_hash,
            "ledger hash"
        );

        if v2::PendingCoinbaseHash::from_fp(result.hash_after_applying.pending_coinbase_hash)
            != expected_pending_coinbase_hash
        {
            println!("+++ pending coinbase hash mismatch");
        }

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

        staged_ledger.commit_and_reparent_to_root();

        prev_protocol_state = data.data.protocol_state;

        // Now you can work with `data`
        //println!("Parsed data: {:?}", data);
        //break;
    }
}

//fn handle_failure(
//    staged_ledger: &StagedLedger,
//    block: ArcBlockWithHash,
//    pred_block: ArcBlockWithHash,
//) {
//    match dump_application_to_file(staged_ledger, block.clone(), pred_block) {
//        Ok(filename) => openmina_core::info!(
//            openmina_core::log::system_time();
//            kind = "LedgerService::dump - Failed application",
//            summary = format!("StagedLedger and block saved to: {filename:?}")
//        ),
//        Err(e) => openmina_core::error!(
//            openmina_core::log::system_time();
//            kind = "LedgerService::dump - Failed application",
//            summary = format!("Failed to save block application to file: {e:?}")
//        ),
//    }
//}


//fn dump_application_to_file(
//    staged_ledger: &StagedLedger,
//    block: ArcBlockWithHash,
//    pred_block: ArcBlockWithHash,
//) -> std::io::Result<String> {
//    use mina_p2p_messages::binprot::{
//        self,
//        macros::{BinProtRead, BinProtWrite},
//    };
//
//    #[derive(BinProtRead, BinProtWrite)]
//    struct ApplyContext {
//        accounts: Vec<v2::MinaBaseAccountBinableArgStableV2>,
//        scan_state: v2::TransactionSnarkScanStateStableV2,
//        pending_coinbase: v2::MinaBasePendingCoinbaseStableV2,
//        pred_block: v2::MinaBlockBlockStableV2,
//        blocks: Vec<v2::MinaBlockBlockStableV2>,
//    }
//
//    let cs = &block.block.header.protocol_state.body.consensus_state;
//    let block_height = cs.blockchain_length.as_u32();
//
//    let apply_context = ApplyContext {
//        accounts: staged_ledger
//            .ledger()
//            .to_list()
//            .iter()
//            .map(v2::MinaBaseAccountBinableArgStableV2::from)
//            .collect::<Vec<_>>(),
//        scan_state: staged_ledger.scan_state().into(),
//        pending_coinbase: staged_ledger.pending_coinbase_collection().into(),
//        pred_block: (*pred_block.block).clone(),
//        blocks: vec![(*block.block).clone()],
//    };
//
//    use mina_p2p_messages::binprot::BinProtWrite;
//    let filename = format!("/tmp/failed_application_ctx_{}.binprot", block_height);
//    let mut file = std::fs::File::create(&filename)?;
//    apply_context.binprot_write(&mut file)?;
//    file.sync_all()?;
//
//    Ok(filename)
//}

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

/*

  type t =
    { scheduled_time : Block_time.t
    ; protocol_state : Protocol_state.value
    ; protocol_state_proof : Proof.t
    ; staged_ledger_diff : Staged_ledger_diff.t
    ; delta_transition_chain_proof :
        Frozen_ledger_hash.t * Frozen_ledger_hash.t list
    ; protocol_version : Protocol_version.t
    ; proposed_protocol_version : Protocol_version.t option [@default None]
    ; accounts_accessed : (int * Account.t) list
    ; accounts_created : (Account_id.t * Currency.Fee.t) list
    ; tokens_used : (Token_id.t * Account_id.t option) list
    }
*/

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]

pub struct PrecomputedBlock {
    pub scheduled_time: BlockTimeTimeStableV1,
    pub protocol_state: MinaStateProtocolStateValueStableV2,
    pub protocol_state_proof: Proof,
    pub staged_ledger_diff: StagedLedgerDiffDiffStableV2,
    //pub delta_transition_chain_proof: (StateHashV1, List<StateBodyHashV1>),
    //pub protocol_version: ProtocolVersionStableV2,
    #[serde(default)]
    pub proposed_protocol_version: Option<ProtocolVersionStableV2>,
    //accounts_accessed: (),
    //accounts_created: (),
    //tokens_used: (),
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
