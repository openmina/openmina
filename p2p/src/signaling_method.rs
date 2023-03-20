use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Debug, Clone)]
pub enum SignalingMethod {
    Http(String),
}
