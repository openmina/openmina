use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct RpcIdType;
impl super::RequestIdType for RpcIdType {
    fn request_id_type() -> &'static str {
        "RpcId"
    }
}

pub type RpcId = super::RequestId<RpcIdType>;
