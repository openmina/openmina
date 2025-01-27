use std::net::{IpAddr, SocketAddr};

use etherparse::{err::packet, NetSlice, SlicedPacket, TransportSlice};
use pcap::{Activated, Capture, Packet, PacketCodec, PacketIter, Savefile};
use thiserror::Error;

pub struct UdpIter<S: Activated + ?Sized> {
    inner: PacketIter<S, UdpCodec>,
}

#[derive(Debug, Error)]
pub enum DissectError {
    #[error("{0}")]
    Cap(#[from] pcap::Error),
    #[error("{0}")]
    ParsePacket(#[from] packet::SliceError),
}

impl<S: Activated + ?Sized> UdpIter<S> {
    pub fn new(capture: Capture<S>, file: Option<Savefile>) -> Self {
        UdpIter {
            inner: capture.iter(UdpCodec { file }),
        }
    }
}

impl<S: Activated + ?Sized> Iterator for UdpIter<S> {
    type Item = Result<(SocketAddr, SocketAddr, Box<[u8]>), DissectError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()?
            .map_err(DissectError::Cap)
            .and_then(|x| x)
            .transpose()
    }
}

struct UdpCodec {
    file: Option<Savefile>,
}

impl PacketCodec for UdpCodec {
    type Item = Result<Option<(SocketAddr, SocketAddr, Box<[u8]>)>, DissectError>;

    fn decode(&mut self, packet: Packet<'_>) -> Self::Item {
        let eth = SlicedPacket::from_ethernet(packet.data)?;
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
                NetSlice::Arp(_) => return Ok(None),
            };
            let (src_port, dst_port, slice) = match transport {
                TransportSlice::Udp(udp) => {
                    (udp.source_port(), udp.destination_port(), udp.payload())
                }
                _ => return Ok(None),
            };
            if let Some(file) = &mut self.file {
                file.write(&packet);
            }

            Ok(Some((
                SocketAddr::new(src_ip, src_port),
                SocketAddr::new(dst_ip, dst_port),
                slice.to_vec().into_boxed_slice(),
            )))
        } else {
            Ok(None)
        }
    }
}
