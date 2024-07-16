use std::{
    collections::{BTreeMap, VecDeque},
    fs::{self, File},
    io,
    path::Path,
};

use binprot::{BinProtRead, BinProtWrite};
use ledger::proofs::public_input::protocol_state::MinaHash;
use libp2p::Swarm;
use mina_p2p_messages::{
    list::List,
    rpc::{
        GetAncestryV2, GetBestTipV2, GetStagedLedgerAuxAndPendingCoinbasesAtHashV2,
        GetTransitionChainProofV1ForV2, GetTransitionChainV2, WithHashV1,
    },
    v2,
};

use super::{
    behaviour::Behaviour, bootstrap::Storage, client::Client, snarked_ledger::SnarkedLedger,
};

pub async fn run(swarm: Swarm<Behaviour>, path_main: &Path, bootstrap: bool) {
    let mut client = Client::new(swarm);

    fs::create_dir_all(path_main).unwrap();

    let best_tip = client.rpc::<GetBestTipV2>(()).await.unwrap().unwrap();

    let head_height = best_tip
        .data
        .header
        .protocol_state
        .body
        .consensus_state
        .blockchain_length
        .as_u32();

    log::info!("will record {head_height}");
    let path = path_main.join(head_height.to_string());
    fs::create_dir_all(path.join("ledgers")).unwrap();

    let mut file = File::create(path.join("best_tip")).unwrap();
    Some(best_tip.clone()).binprot_write(&mut file).unwrap();

    // let mut file = File::create(path.join("best_tip.json")).unwrap();
    // file.write_all(serde_json::to_string(&best_tip).unwrap().as_bytes())
    //     .unwrap();

    let q = best_tip
        .data
        .header
        .protocol_state
        .body
        .consensus_state
        .clone();
    let hash = MinaHash::hash(&best_tip.data.header.protocol_state).into();
    let q = WithHashV1 { data: q, hash };
    let ancestry = client.rpc::<GetAncestryV2>(q).await.unwrap().unwrap();

    let mut file = File::create(path.join("ancestry")).unwrap();
    Some(ancestry.clone()).binprot_write(&mut file).unwrap();

    let snarked_protocol_state = best_tip.proof.1.header.protocol_state;

    {
        let staking_epoch_ledger_hash = snarked_protocol_state
            .body
            .consensus_state
            .staking_epoch_data
            .ledger
            .hash
            .clone();
        let staking_epoch_ledger_hash_str =
            match serde_json::to_value(&staking_epoch_ledger_hash).unwrap() {
                serde_json::Value::String(s) => s,
                _ => panic!(),
            };
        let mut epoch_ledger = match File::open(path.join("staking_epoch_ledger.bin")) {
            Ok(file) => SnarkedLedger::load_bin(file).unwrap(),
            Err(_) => {
                match File::open(path.parent().expect("msg").join("staking_epoch_ledger.bin")) {
                    Ok(file) => SnarkedLedger::load_bin(file).unwrap(),
                    Err(_) => SnarkedLedger::empty(),
                }
            }
        };
        epoch_ledger
            .sync_new(&mut client, &staking_epoch_ledger_hash)
            .await;
        epoch_ledger
            .store_bin(
                File::create(path.join("ledgers").join(staking_epoch_ledger_hash_str)).unwrap(),
            )
            .unwrap();
        epoch_ledger
            .store_bin(File::create(path.join("staking_epoch_ledger.bin")).unwrap())
            .unwrap();
        fs::copy(
            path.join("staking_epoch_ledger.bin"),
            path.parent()
                .expect("must have parent")
                .join("staking_epoch_ledger.bin"),
        )
        .unwrap();
    }
    let next_epoch_ledger_hash = snarked_protocol_state
        .body
        .consensus_state
        .next_epoch_data
        .ledger
        .hash
        .clone();
    let next_epoch_ledger_hash_str = match serde_json::to_value(&next_epoch_ledger_hash).unwrap() {
        serde_json::Value::String(s) => s,
        _ => panic!(),
    };
    let mut epoch_ledger = match File::open(path.join("epoch_ledger.bin")) {
        Ok(file) => SnarkedLedger::load_bin(file).unwrap(),
        Err(_) => match File::open(path.parent().expect("msg").join("epoch_ledger.bin")) {
            Ok(file) => SnarkedLedger::load_bin(file).unwrap(),
            Err(_) => SnarkedLedger::empty(),
        },
    };
    epoch_ledger
        .sync_new(&mut client, &next_epoch_ledger_hash)
        .await;
    epoch_ledger
        .store_bin(File::create(path.join("ledgers").join(next_epoch_ledger_hash_str)).unwrap())
        .unwrap();
    epoch_ledger
        .store_bin(File::create(path.join("epoch_ledger.bin")).unwrap())
        .unwrap();
    fs::copy(
        path.join("epoch_ledger.bin"),
        path.parent()
            .expect("must have parent")
            .join("epoch_ledger.bin"),
    )
    .unwrap();

    let snarked_ledger_hash = snarked_protocol_state
        .body
        .blockchain_state
        .ledger_proof_statement
        .target
        .first_pass_ledger
        .clone();
    let snarked_ledger_hash_str = match serde_json::to_value(&snarked_ledger_hash).unwrap() {
        serde_json::Value::String(s) => s,
        _ => panic!(),
    };
    log::info!("snarked_ledger_hash: {snarked_ledger_hash_str}");
    let mut snarked_ledger = match File::open(path.join("current_ledger.bin")) {
        Ok(file) => SnarkedLedger::load_bin(file).unwrap(),
        Err(_) => match File::open(path.parent().expect("msg").join("current_ledger.bin")) {
            Ok(file) => SnarkedLedger::load_bin(file).unwrap(),
            Err(_) => SnarkedLedger::empty(),
        },
    };
    snarked_ledger
        .sync_new(&mut client, &snarked_ledger_hash)
        .await;
    snarked_ledger
        .store_bin(File::create(path.join("ledgers").join(snarked_ledger_hash_str)).unwrap())
        .unwrap();
    snarked_ledger
        .store_bin(File::create(path.join("current_ledger.bin")).unwrap())
        .unwrap();
    fs::copy(
        path.join("current_ledger.bin"),
        path.parent()
            .expect("must have parent")
            .join("current_ledger.bin"),
    )
    .unwrap();

    let expected_hash = snarked_protocol_state
        .body
        .blockchain_state
        .staged_ledger_hash
        .clone();

    let snarked_block_hash = MinaHash::hash(&snarked_protocol_state);
    let snarked_block_hash =
        v2::StateHash::from(v2::DataHashLibStateHashStableV1(snarked_block_hash.into()));
    log::info!("downloading staged_ledger_aux and pending_coinbases at {snarked_block_hash}");
    let info = client
        .rpc::<GetStagedLedgerAuxAndPendingCoinbasesAtHashV2>(snarked_block_hash.0.clone())
        .await
        .unwrap();
    let mut file = File::create(path.join("staged_ledger_aux")).unwrap();
    info.binprot_write(&mut file).unwrap();

    let snarked_height = snarked_protocol_state
        .body
        .consensus_state
        .blockchain_length
        .as_u32();
    log::info!("will bootstrap: {}..={head_height}", snarked_height);

    let mut blocks = VecDeque::new();
    blocks.push_back(best_tip.data);
    download_blocks(
        &mut client,
        &mut blocks,
        &path_main.join("blocks"),
        head_height,
        snarked_height,
    )
    .await;

    if bootstrap {
        let mut storage = Storage::new(snarked_ledger.inner, info, expected_hash);

        let mut prev_protocol_state = snarked_protocol_state;
        while let Some(block) = blocks.pop_back() {
            storage.apply_block(&block, &prev_protocol_state);
            prev_protocol_state = block.header.protocol_state.clone();
        }
    }
}

async fn download_blocks(
    engine: &mut Client,
    blocks: &mut VecDeque<v2::MinaBlockBlockStableV2>,
    dir: &Path,
    head_height: u32,
    snarked_height: u32,
) {
    let create_dir = |dir: &Path| {
        fs::create_dir_all(dir)
            .or_else(|e| {
                if e.kind() == io::ErrorKind::AlreadyExists {
                    Ok(())
                } else {
                    Err(e)
                }
            })
            .unwrap()
    };
    create_dir(dir);

    let mut table = match File::open(dir.join("table.json")) {
        Ok(f) => serde_json::from_reader(f).unwrap(),
        Err(_) => BTreeMap::<String, u32>::new(),
    };

    log::info!("need blocks {}..{head_height}", snarked_height + 1);
    for i in ((snarked_height + 1)..head_height).rev() {
        let last_protocol_state = &blocks.back().unwrap().header.protocol_state;
        let this_hash = &last_protocol_state.previous_state_hash;
        let this_height = last_protocol_state
            .body
            .consensus_state
            .blockchain_length
            .as_u32()
            - 1;
        let dir = dir.join(this_height.to_string());
        create_dir(&dir);
        table.insert(this_hash.to_string(), this_height);
        let new = if let Ok(mut file) = File::open(dir.join(this_hash.to_string())) {
            v2::MinaBlockBlockStableV2::binprot_read(&mut file).unwrap()
        } else {
            log::info!("downloading block {i}");
            let new: Vec<_> = engine
                .rpc::<GetTransitionChainV2>(List::one(this_hash.0.clone()))
                .await
                .unwrap()
                .unwrap()
                .into_iter()
                .collect();
            let mut file = File::create(dir.join(this_hash.to_string())).unwrap();
            new[0].binprot_write(&mut file).unwrap();
            if let Ok(new_proof) = engine
                .rpc::<GetTransitionChainProofV1ForV2>(this_hash.0.clone())
                .await
            {
                let mut file = File::create(dir.join(format!("proof_{this_hash}"))).unwrap();
                new_proof.binprot_write(&mut file).unwrap();
            }
            new[0].clone()
        };
        blocks.push_back(new);
    }
    let file = File::create(dir.join("table.json")).unwrap();
    serde_json::to_writer(file, &table).unwrap();
    log::info!("have blocks {}..{head_height}", snarked_height + 1);
}
