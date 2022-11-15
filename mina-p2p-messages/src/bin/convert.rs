use std::io;

use binprot::BinProtRead;
use mina_p2p_messages::gossip::GossipNetMessageV2;

fn main() {
    let gossip_message = GossipNetMessageV2::binprot_read(&mut io::stdin()).unwrap();
    serde_json::to_writer_pretty(&mut io::stdout(), &gossip_message).unwrap()
}
