use node::{
    event_source::Event,
    p2p::{
        channels::{rpc::RpcChannelMsg, ChannelMsg},
        P2pChannelEvent, P2pEvent,
    },
    State,
};

pub fn event_details(state: &State, event: &Event) -> Option<String> {
    match event {
        Event::P2p(P2pEvent::Channel(P2pChannelEvent::Received(peer_id, Ok(ChannelMsg::Rpc(RpcChannelMsg::Response(req_id, _)))))) => {
            let rpc_state = &state.p2p.get_ready_peer(peer_id)?.channels.rpc;
            if *req_id == rpc_state.pending_local_rpc_id()? {
                return Some(format!(
                    "Request: {}",
                    rpc_state.pending_local_rpc()?
                ));
            }
            None
        },
        _ => None,
    }
}