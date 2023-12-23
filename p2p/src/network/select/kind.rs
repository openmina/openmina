use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub enum SelectKind {
    #[default]
    Authentication,
    Multiplexing,
    Stream,
}
