use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use libp2p::{futures::StreamExt, gossipsub, swarm::SwarmEvent, Multiaddr};
use mina_transport::ed25519::SecretKey;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Args {
    #[structopt(long, default_value = "target/gossipsub")]
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
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let Args {
        path,
        chain_id,
        listen,
        mut peer,
    } = Args::from_args();

    let default_peer = [
        "/dns4/seed-1.berkeley.o1test.net/tcp/10000/p2p/12D3KooWAdgYL6hv18M3iDBdaK1dRygPivSfAfBNDzie6YqydVbs",
        "/ip4/34.135.63.47/tcp/10001/p2p/12D3KooWLjs54xHzVmMmGYb7W5RVibqbwD1co7M2ZMfPgPm7iAag",
        "/dns4/seed-3.berkeley.o1test.net/tcp/10002/p2p/12D3KooWEiGVAFC7curXWXiGZyMWnZK9h8BKr88U8D5PKV3dXciv",

        "/ip4/65.21.29.250/tcp/8302/p2p/12D3KooWAMKZM7ysxRka2RfQWxCXXFTopjZ1eZcwtSi4VPgcufaS",
        "/ip4/176.9.123.23/tcp/8302/p2p/12D3KooWDKdHVbnkM7GJYML6ogYR5KmHUj9Ngnq1Lk42xcXnf2sx",
        "/ip4/158.69.119.191/tcp/8302/p2p/12D3KooWFfPswpKr6jDpRXXiTzTkphsUquvftiwYSnNTXSoFd3pu",
        "/ip4/51.222.104.199/tcp/8302/p2p/12D3KooWG9RB23qjbCiwjgZj2Rkyn5R94e6kqVw4cGLk6362wPPR",

        "/ip4/65.21.123.88/tcp/8302/p2p/12D3KooWLKSM9oHWU7qwL7Ci75wunkjXpRmK6j5xq527zGw554AF",
        "/ip4/65.109.123.166/tcp/8302/p2p/12D3KooWGc9vwL9DUvoLdBFPSQGCT2QTULskzhmXcn8zg2j3jdFF",
        "/ip4/176.9.64.21/tcp/8302/p2p/12D3KooWG9owTshte2gR3joP4sgwAfdoV9bQeeB5y9R3QUprKLdJ",
        "/ip4/35.238.71.15/tcp/65454/p2p/12D3KooWHdUVpCZ9KcF5hNBrwf2uy7BaPDKrxyHJAaM5epJgQucX",
        "/ip4/35.224.199.118/tcp/25493/p2p/12D3KooWGbjV7ptpzLu4BuykKfEsF4ebLyR8gZAMUissMToKGVDQ",
        "/ip4/35.193.28.252/tcp/37470/p2p/12D3KooWFcCiQqrzBVLEkPdpkHDgWr6AkSMthT96agKYBBVuRhHg",
        "/ip4/142.132.154.120/tcp/58654/p2p/12D3KooWMPxTu24mCpi3TwmkU4fJk7a8TQ4agFZeTHQRi8KCc3nj",
        "/ip4/65.108.121.245/tcp/8302/p2p/12D3KooWGQ4g2eY44n5JLqymi8KC55GbnujAFeXNQrmNKSq4NYrv",
        "/ip4/65.109.123.173/tcp/8302/p2p/12D3KooWMd8K8FFd76cacUEE6sSzUPr7wj71TvMqGdFSgrpv923k",
        "/ip4/65.109.123.235/tcp/8302/p2p/12D3KooWBK3vz1inMubXCUeDF4Min6eG5418toceG8QvNPWRW1Gz",
        "/ip4/34.172.208.246/tcp/46203/p2p/12D3KooWNafCBobFGSdJyYonvSCB5KDzW3JZYnVBF6q22yhcXGjM",
        "/ip4/34.29.40.184/tcp/7528/p2p/12D3KooWJoVjUsnDosW3Ae78V4CSf5SSe9Wyetr5DxutmMMfwdp8",
        "/ip4/34.122.249.235/tcp/55894/p2p/12D3KooWMpGyhYHbzVeqYnxGHQQYmQNtYcoMLLZZmYRPvAJKxXXm",
        "/ip4/35.232.20.138/tcp/10000/p2p/12D3KooWAdgYL6hv18M3iDBdaK1dRygPivSfAfBNDzie6YqydVbs",
        "/ip4/88.198.230.168/tcp/8302/p2p/12D3KooWGA7AS91AWNtGEBCBk64kgirtTiyaXDTyDtKPTjpefNL9",
        "/ip4/35.224.199.118/tcp/10360/p2p/12D3KooWDnC4XrJzas3heuz4LUehZjf2WJyfob2XEodrYL3soaf4",
        "/ip4/34.123.4.144/tcp/10002/p2p/12D3KooWEiGVAFC7curXWXiGZyMWnZK9h8BKr88U8D5PKV3dXciv",
        "/ip4/34.170.114.52/tcp/10001/p2p/12D3KooWLjs54xHzVmMmGYb7W5RVibqbwD1co7M2ZMfPgPm7iAag",
        "/ip4/34.172.208.246/tcp/54351/p2p/12D3KooWEhCm8FVcqZSkXKNhuBPmsEfJGeqSmUxNQhpemZkENfik",
        "/ip4/34.29.161.11/tcp/10946/p2p/12D3KooWCntSrMqSiovXcVfMZ56aYbzpZoh4mi7gJJNiZBmzXrpa",
        "/ip4/35.238.71.15/tcp/23676/p2p/12D3KooWENsfMszNYBRfHZJUEAvXKThmZU3nijWVbLivq33AE2Vk",
    ].map(|s| s.parse().unwrap());
    if peer.is_empty() {
        peer.extend(default_peer);
    }

    let sk = SecretKey::generate();

    let local_key: libp2p::identity::Keypair = mina_transport::ed25519::Keypair::from(sk).into();
    log::info!("{}", local_key.public().to_peer_id());

    let message_authenticity = gossipsub::MessageAuthenticity::Signed(local_key.clone());
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .max_transmit_size(1024 * 1024 * 32)
        .build()
        .expect("the config must be a valid constant");
    let behaviour: gossipsub::Behaviour =
        gossipsub::Behaviour::new(message_authenticity, gossipsub_config)
            .expect("strict validation mode must be compatible with this `message_authenticity`");

    let mut swarm = mina_transport::swarm(
        local_key,
        chain_id.as_bytes(),
        listen,
        peer.iter().cloned(),
        behaviour,
    );

    let topic = gossipsub::IdentTopic::new("coda/consensus-messages/0.0.1");
    swarm.behaviour_mut().subscribe(&topic).unwrap();
    for peer in peer {
        for protocol in peer.iter() {
            if let libp2p::multiaddr::Protocol::P2p(peer_id) = protocol {
                swarm.behaviour_mut().add_explicit_peer(&peer_id);
            }
        }
    }

    fs::create_dir_all(&path).unwrap();
    let mut file = File::create(path.join("snark_pool_diff")).unwrap();
    while let Some(event) = swarm.next().await {
        match event {
            SwarmEvent::Behaviour(gossipsub::Event::Message { message, .. }) => {
                // GossipNetMessageV2::SnarkPoolDiff
                if message.data[8] == 1 {
                    file.write_all(&message.data).unwrap();
                }
            }
            SwarmEvent::Behaviour(gossipsub::Event::Subscribed { peer_id, topic }) => {
                log::info!("{peer_id} {topic}");
            }
            SwarmEvent::ConnectionEstablished { .. } => {}
            _ => {}
        }
    }
}
