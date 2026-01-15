use mina_node::{
    event_source::Event,
    p2p::{
        channels::{rpc::RpcChannelMsg, ChannelMsg},
        P2pChannelEvent, P2pEvent,
    },
    State,
};

pub fn event_details(state: &State, event: &Event) -> Option<String> {
    // this could be a let-chain but we're on rust 2021 instead of 2024 >:(
    if let Event::P2p(P2pEvent::Channel(P2pChannelEvent::Received(peer_id, Ok(msg)))) = event {
        if let ChannelMsg::Rpc(RpcChannelMsg::Response(req_id, _)) = &**msg {
            let rpc_state = &state.p2p.get_ready_peer(peer_id)?.channels.rpc;
            if *req_id == rpc_state.pending_local_rpc_id()? {
                return Some(format!("Request: {}", rpc_state.pending_local_rpc()?));
            }
        }
    }

    None
}
