use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub enum SelectKind {
    #[default]
    Authentication,
    Multiplexing,
    Stream,
}

impl SelectKind {
    pub fn supported(&self) -> Vec<&'static [u8]> {
        match self {
            Self::Authentication => vec![b"\x07/noise\n"],
            Self::Multiplexing => vec![b"\x12/coda/yamux/1.0.0\n"],
            Self::Stream => vec![
                b"\x10/coda/kad/1.0.0\n",
                b"\x0f/meshsub/1.1.0\n",
                b"\x10coda/rpcs/0.0.1\n",
            ],
        }
    }
}
