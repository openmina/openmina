mod net;

use p2p::identity::SecretKey;

use pcap::{Activated, Capture, Savefile};

pub fn run<T: Activated + ?Sized>(
    capture: Capture<T>,
    file: Option<Savefile>,
    secret_key: SecretKey,
) -> Result<(), net::DissectError> {
    let _ = secret_key;

    for item in net::UdpIter::new(capture, file) {
        let (src, dst, data) = item?;
        log::info!(
            "{src} -> {dst}: {} {}",
            data.len(),
            hex::encode(&data[..data.len().min(12)])
        );
    }

    Ok(())
}

pub fn handle(packet: pcap::Packet) {
    use std::net::{IpAddr, SocketAddr};

    use etherparse::{NetSlice, SlicedPacket, TransportSlice};

    let eth = SlicedPacket::from_ethernet(packet.data).unwrap();
    if let (Some(net), Some(transport)) = (eth.net, eth.transport) {
        let (src_ip, dst_ip) = match net {
            NetSlice::Ipv4(ip) => (
                IpAddr::V4(ip.header().source().into()),
                IpAddr::V4(ip.header().destination().into()),
            ),
            NetSlice::Ipv6(ip) => (
                IpAddr::V6(ip.header().source().into()),
                IpAddr::V6(ip.header().destination().into()),
            ),
            NetSlice::Arp(_) => return,
        };
        let (src_port, dst_port, slice) = match transport {
            TransportSlice::Udp(udp) => (udp.source_port(), udp.destination_port(), udp.payload()),
            _ => return,
        };

        let (src, dst, data) = (
            SocketAddr::new(src_ip, src_port),
            SocketAddr::new(dst_ip, dst_port),
            slice,
        );
        log::info!(
            "{src} -> {dst}: {} {}",
            data.len(),
            hex::encode(&data[..data.len().min(12)])
        );
    }
}
