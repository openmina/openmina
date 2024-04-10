use reqwest::StatusCode;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::{StakingToolError, node};


pub async fn get_genesis_timestamp() -> Result<impl warp::Reply, warp::reject::Rejection> {
    // TODO(adonagy): we need this only once, no need to query the node every time...
    match node::get_genesis_timestmap().await {
        Ok(timestamp) => {
            let datetime = OffsetDateTime::parse(&timestamp, &Rfc3339).unwrap();
            let unix_timestamp = datetime.unix_timestamp();
            Ok(warp::reply::with_status(warp::reply::json(&unix_timestamp.to_string()), StatusCode::OK))
        }
        Err(_) => Err(warp::reject())
    }
}