use std::{collections::BTreeMap, net::SocketAddr};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Token {
    Waker,
    Listener(SocketAddr),
    Connection(SocketAddr),
}

#[derive(Default)]
pub struct TokenRegistry {
    map: BTreeMap<Token, mio::Token>,
    reverse: BTreeMap<mio::Token, Token>,
    last: usize,
}

impl TokenRegistry {
    pub fn register(&mut self, token: Token) -> mio::Token {
        *self.map.entry(token).or_insert_with(|| {
            let mio_token = mio::Token(self.last);
            self.last += 1;
            self.reverse.insert(mio_token, token);
            mio_token
        })
    }

    pub fn get(&mut self, token: &mio::Token) -> Option<Token> {
        self.reverse.get(token).copied()
    }
}
