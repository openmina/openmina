use crate::PeerId;
//use quick_protobuf::{BytesReader, MessageRead, MessageWrite, Writer};
use serde::{Deserialize, Serialize};
//use std::borrow::Cow;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub from: Option<PeerId>,
    pub data: Vec<u8>,
    pub seqno: Option<Vec<u8>>, // TODO: Option<u64>, ?
    pub topics: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SubOpts {
    Subscribe { topic: String },
    Unsubscribe { topic: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct P2pNetworkFloodsub {
    subscriptions: Vec<SubOpts>,
    publish: Vec<Message>,
}

#[derive(Clone, Debug, Serialize, Deserialize, thiserror::Error)]
pub enum P2pNetworkFloodsubFromMessageError {
    #[error("error parsing subscriptions: {0}")]
    SubscriptionsParseError(String),
    #[error("error parsing peer_id: {0}")]
    InvalidPeerId(String),
}

impl<'a> TryFrom<super::p2p_network_floodsub_message::RPC<'a>> for P2pNetworkFloodsub {
    type Error = P2pNetworkFloodsubFromMessageError;

    fn try_from(value: super::p2p_network_floodsub_message::RPC<'a>) -> Result<Self, Self::Error> {
        let mut subscriptions = Vec::new();
        let mut publish = Vec::new();

        for super::p2p_network_floodsub_message::mod_RPC::SubOpts::<'a> { subscribe, topicid } in
            value.subscriptions.iter()
        {
            let subscription = if let Some(topic) = topicid {
                let topic = topic.to_string();

                match subscribe {
                    Some(true) => SubOpts::Subscribe { topic },
                    Some(false) => SubOpts::Unsubscribe { topic },
                    None => {
                        return Err(P2pNetworkFloodsubFromMessageError::SubscriptionsParseError(
                            format!("undefined subscribe value for topicid {:?}", topicid),
                        ))
                    }
                }
            } else {
                return Err(P2pNetworkFloodsubFromMessageError::SubscriptionsParseError(
                    "undefined topicid".to_string(),
                ));
            };

            subscriptions.push(subscription);
        }

        for super::p2p_network_floodsub_message::Message::<'a> {
            from,
            data,
            seqno,
            topics,
        } in value.publish.iter()
        {
            let from = if let Some(from_bytes) = &from {
                if let Ok(peer_id) = from_bytes[..].try_into() {
                    Some(PeerId::from_bytes(peer_id))
                } else {
                    return Err(P2pNetworkFloodsubFromMessageError::InvalidPeerId(format!(
                        "invalid peer_id size {}",
                        from_bytes.len()
                    )));
                }
            } else {
                None
            };

            publish.push(Message {
                from,
                data: data.as_ref().map(|d| d.to_vec()).unwrap_or_default(),
                seqno: seqno.as_ref().map(|d| d.to_vec()),
                topics: topics.iter().map(|d| d.to_string()).collect(),
            });
        }

        Ok(P2pNetworkFloodsub {
            subscriptions,
            publish,
        })
    }
}

// impl<'a> From<&'a P2pNetworkIdentify> for super::Identify<'a> {
//     fn from(value: &'a P2pNetworkIdentify) -> Self {
//         Self {
//             protocolVersion: value.protocol_version.as_ref().map(|v| v.into()),
//             agentVersion: value.agent_version.as_ref().map(|v| v.into()),
//             publicKey: value.public_key.as_ref().map(|key| {
//                 let key_bytes = key.to_bytes();
//                 let pubkey = keys_proto::PublicKey {
//                     Type: crate::network::identify::KeyType::Ed25519,
//                     Data: key_bytes.as_ref().into(),
//                 };
//                 let mut buf = Vec::with_capacity(pubkey.get_size());
//                 let mut writer = Writer::new(&mut buf);

//                 pubkey.write_message(&mut writer).expect("encoding success");
//                 buf.into()
//             }),
//             listenAddrs: value
//                 .listen_addrs
//                 .iter()
//                 .map(|v| v.to_vec().into())
//                 .collect(),
//             observedAddr: value.observed_addr.as_ref().map(|v| v.to_vec().into()),
//             protocols: value
//                 .protocols
//                 .iter()
//                 .map(|v| v.name_str().into())
//                 .collect(),
//         }
//     }
// }
