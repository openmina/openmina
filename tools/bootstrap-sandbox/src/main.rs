#![forbid(unsafe_code)]

mod client;

mod snarked_ledger;

mod bootstrap;

mod record;
mod replay;

use std::{env, path::PathBuf};

use libp2p::Multiaddr;
use libp2p_rpc_behaviour::BehaviourBuilder;
use mina_transport::ed25519::SecretKey;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Args {
    #[structopt(long, default_value = "target/record")]
    path: PathBuf,
    #[structopt(
        long,
        default_value = "fd7d111973bf5a9e3e87384f560fdead2f272589ca00b6d9e357fca9839631da"
    )]
    chain_id: String,
    #[structopt(long)]
    listen: Vec<Multiaddr>,
    #[structopt(long)]
    peer: Vec<Multiaddr>,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    Again {
        height: u32,
    },
    Record {
        #[structopt(long)]
        bootstrap: bool,
    },
    Replay {
        height: u32,
    },
    // Test {
    //     height: u32,
    //     url: String,
    // },
    // TestGraphql {
    //     height: u32,
    //     url: String,
    //     #[structopt(long)]
    //     verbose: bool,
    // },
    // Archive {
    //     state: String,
    // },
    // ApplyArchive,
}
#[tokio::main]
async fn main() {
    env_logger::init();

    let Args {
        path,
        chain_id,
        listen,
        peer,
        cmd,
    } = Args::from_args();

    let sk = env::var("OPENMINA_P2P_SEC_KEY")
        .map(|key| {
            let mut bytes = bs58::decode(key).with_check(Some(0x80)).into_vec().unwrap();
            SecretKey::try_from_bytes(&mut bytes[1..]).unwrap()
        })
        .unwrap_or_else(|_| {
            let mut bytes = rand::random::<[u8; 32]>();
            log::info!(
                "{}",
                bs58::encode(&bytes).with_check_version(0x80).into_string()
            );
            SecretKey::try_from_bytes(&mut bytes).unwrap()
        });

    let local_key: libp2p::identity::Keypair = mina_transport::ed25519::Keypair::from(sk).into();
    log::info!("{}", local_key.public().to_peer_id());

    match cmd {
        Command::Again { height } => bootstrap::again(&path, height).await,
        Command::Record { bootstrap } => {
            let behaviour = BehaviourBuilder::default().build();
            let swarm =
                mina_transport::swarm(local_key, chain_id.as_bytes(), listen, peer, behaviour);

            record::run(swarm, &path, bootstrap).await
        }
        Command::Replay { height } => {
            use mina_p2p_messages::rpc::{
                AnswerSyncLedgerQueryV2, GetAncestryV2, GetBestTipV2,
                GetStagedLedgerAuxAndPendingCoinbasesAtHashV2, GetTransitionChainProofV1ForV2,
                GetTransitionChainV2,
            };

            let behaviour = BehaviourBuilder::default()
                .register_method::<GetBestTipV2>()
                .register_method::<GetAncestryV2>()
                .register_method::<GetStagedLedgerAuxAndPendingCoinbasesAtHashV2>()
                .register_method::<AnswerSyncLedgerQueryV2>()
                .register_method::<GetTransitionChainV2>()
                .register_method::<GetTransitionChainProofV1ForV2>()
                .build();
            let swarm =
                mina_transport::swarm(local_key, chain_id.as_bytes(), listen, [], behaviour);

            replay::run(swarm, &path, height).await
        } // Command::Test { .. } => unimplemented!(),
          // Command::TestGraphql { .. } => unimplemented!(),
          // Command::Archive { .. } => unimplemented!(),
          // Command::ApplyArchive => unimplemented!(),
    }
}
