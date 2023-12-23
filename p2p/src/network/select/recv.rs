use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct State {
    buffer: Vec<u8>,
    offset: usize,
    parsed: usize,
}

impl Default for State {
    fn default() -> Self {
        State {
            buffer: vec![0; 256],
            offset: 0,
            parsed: 0,
        }
    }
}

impl State {
    /// reset the state
    pub fn consume(&mut self) {
        if self.parsed > 0 {
            let mut new = vec![0; 256];
            let new_offset = self.offset - self.parsed;
            new[..new_offset].clone_from_slice(&self.buffer[self.parsed..self.offset]);
            self.buffer = new;
            self.offset = new_offset;
            self.parsed = 0;
        }
    }

    /// return protocol name followed by the rest of unparsed data
    /// use `State::consume` after return some parsed token
    pub fn parse_protocol<'a, 'b>(
        &'a mut self,
        mut data: &'b [u8],
    ) -> (Option<&'a [u8]>, &'b [u8]) {
        use unsigned_varint::decode;

        if self.parsed > 0 {
            return (Some(&self.buffer[..self.parsed]), data);
        }

        loop {
            if self.offset > self.parsed {
                if let Ok((len, rem)) = decode::usize(&self.buffer[self.parsed..self.offset]) {
                    if rem.len() >= len {
                        let len_length = self.offset - self.parsed - rem.len();
                        self.parsed = len_length + len;
                        break;
                    }
                }
            }

            if !data.is_empty() {
                let buffer = &mut self.buffer[self.offset..];
                let read = buffer.len().min(data.len());
                self.offset += read;
                buffer[..read].clone_from_slice(&data[..read]);
                data = &data[read..];
            } else {
                return (None, data);
            }
        }

        (Some(&self.buffer[..self.parsed]), data)
    }
}
