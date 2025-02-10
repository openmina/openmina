mod header;

use nom::IResult;
use p2p::identity::SecretKey;

use super::MsgHeader;

use self::header::Chunk;

pub struct State {
    secret_key: SecretKey,
    inner: Inner,
}

enum Inner {
    Initial,
    RecvHello,
}

impl State {
    pub fn new(secret_key: SecretKey) -> Self {
        State {
            secret_key,
            inner: Inner::Initial,
        }
    }

    pub fn handle<'data>(
        &mut self,
        hrd: MsgHeader,
        mut data: &'data [u8],
        incoming: bool,
    ) -> IResult<&'data [u8], ()> {
        let _ = incoming;
        loop {
            let (rest, chunk) = Chunk::parse(data)?;
            log::info!("{hrd} {chunk}");
            if rest.is_empty() {
                break Ok((rest, ()));
            } else {
                data = rest;
            }
        }
    }
}
