mod net;

mod dtls;

use std::{borrow::Cow, collections::BTreeMap, fmt, net::SocketAddr};

use pcap::{Activated, Capture, Savefile};

type State = dtls::State;

#[derive(Clone, Copy)]
pub struct MsgHeader {
    src: SocketAddr,
    dst: SocketAddr,
    len: u16,
}

impl fmt::Display for MsgHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let MsgHeader { src, dst, len } = self;
        write!(f, "{src} -> {dst} {len}")
    }
}

pub fn run<T: Activated + ?Sized>(
    capture: Capture<T>,
    file: Option<Savefile>,
    rng_seed: [u8; 32],
) -> Result<(), net::DissectError> {
    let mut connections = BTreeMap::<(SocketAddr, SocketAddr), State>::new();

    let mut buffer = None::<Vec<u8>>;
    for item in net::UdpIter::new(capture, file) {
        let (src, dst, data) = item?;

        // if src.to_string() != "10.63.45.16:57909" && dst.to_string() != "10.63.45.16:57909" {
        //     continue;
        // }

        let _hdr = MsgHeader {
            src,
            dst,
            len: data.len() as _,
        };

        // skip STUN/TURN
        if data[4..8].eq(b"\x21\x12\xa4\x42") {
            continue;
        }

        log::info!("{_hdr}");

        let data = if let Some(mut buffer) = buffer.take() {
            buffer.extend_from_slice(&data);
            Cow::Owned(buffer)
        } else {
            Cow::Borrowed(data.as_ref())
        };

        let res = if let Some(cn) = connections.get_mut(&(src, dst)) {
            cn.handle(&data, true)
        } else {
            connections
                .entry((dst, src))
                .or_insert_with(|| State::new(rng_seed))
                .handle(&data, false)
        };

        if let Err(err) = res {
            match err {
                nom::Err::Incomplete(_) => buffer = Some(data.into_owned()),
                err => log::error!("{err}"),
            }
        }
    }

    Ok(())
}
