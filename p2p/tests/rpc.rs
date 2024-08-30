use std::{
    collections::BTreeSet,
    future::{ready, Ready},
    time::Duration,
};

use p2p::{
    channels::rpc::{
        P2pChannelsRpcAction, P2pChannelsRpcState, P2pRpcId, P2pRpcLocalState, P2pRpcRemoteState,
        P2pRpcRequest, P2pRpcResponse,
    },
    PeerId,
};
use p2p_testing::{
    cluster::{Cluster, ClusterBuilder, ClusterEvent},
    event::RustNodeEvent,
    futures::TryStreamExt,
    rust_node::RustNodeId,
    stream::ClusterStreamExt,
    test_node::TestNode,
    utils::{
        peer_ids, rust_nodes_from_default_config, try_wait_for_all_nodes_with_value,
        try_wait_for_nodes_to_connect, try_wait_for_nodes_to_listen,
    },
};

fn rpc_channel_is_ready<I>(node_peer: I) -> impl FnMut(ClusterEvent) -> Ready<bool>
where
    I: IntoIterator<Item = (RustNodeId, PeerId)>,
{
    let mut node_peer = BTreeSet::from_iter(node_peer);
    move |event| {
        ready(matches!(
            event,
            ClusterEvent::Rust { id, event: RustNodeEvent::RpcChannelReady { peer_id } }
            if node_peer.remove(&(id, peer_id)) && node_peer.is_empty()
        ))
    }
}

macro_rules! rpc_from_json {
    ($name:expr) => {
        (
            serde_json::from_slice::<P2pRpcRequest>(include_bytes!(concat!(
                "files/rpc/",
                $name,
                "_query.json"
            )))
            .expect("query"),
            serde_json::from_slice::<P2pRpcResponse>(include_bytes!(concat!(
                "files/rpc/",
                $name,
                "_response.json"
            )))
            .expect("response"),
        )
    };
}

macro_rules! rpcs_from_json {
    ($($name:expr),* $(,)?) => {
        [$( rpc_from_json!($name) ),*]
    };
}

#[tokio::test]
async fn rust_to_rust() {
    let mut cluster = ClusterBuilder::new()
        .ports(11000..11200)
        .idle_duration(Duration::from_millis(100))
        .start()
        .await
        .expect("should build cluster");

    let [node1, node2] = rust_nodes_from_default_config(&mut cluster).expect("no error");
    let [peer_id1, peer_id2] = peer_ids(&cluster, [node1, node2]);

    // wait for node1 to be ready to accept incoming conection s
    let listener_is_ready =
        try_wait_for_nodes_to_listen(&mut cluster, [node1], Duration::from_secs(2))
            .await
            .expect("no error");
    assert!(listener_is_ready, "node1 should be ready");

    // connect node2 to node1
    cluster.connect(node2, node1).expect("no error");

    // wait for node2 to have peer_id1 (node1) as its peer in `ready` state`
    let peers_are_connected = try_wait_for_nodes_to_connect(
        &mut cluster,
        [(node2, peer_id1), (node1, peer_id2)],
        Duration::from_secs(2),
    )
    .await
    .expect("no error");
    assert!(
        peers_are_connected,
        "node should be able to connect to {peer_id1}\nnode state: {:#?}",
        cluster.rust_node(node2).state().peers.get(&peer_id1)
    );

    // make sure node can send an RPC request
    assert!(
        rpc_ready(&mut cluster, [(node2, node1)], Duration::from_secs(2))
            .await
            .expect("no errors"),
        "rpc should be ready"
    );

    let rpcs = rpcs_from_json!(
        "initial_peers",
        "best_tip_with_proof",
        // "block", // overflow
        "ledger_query",
        // "staged_ledger_aux_and_pending_coinbases_at_block" // error
    );

    for (query, response) in rpcs {
        let request_id = send_request(&mut cluster, node2, node1, query);
        receive_request(&mut cluster, node2, node1, request_id).await;
        send_response(&mut cluster, node1, node2, request_id, response);
        receive_response(&mut cluster, node1, node2, request_id).await;
    }
}

macro_rules! rpc_test {
    ($(#[$attr:meta])? $name:ident, $ports:expr) => {
        #[tokio::test]
        $(#[$attr])?
        async fn $name() {
            let (query, response) = rpc_from_json!(stringify!($name));
            rust_to_rust_rpc(query, response, $ports).await;
        }
    };
}

rpc_test!(initial_peers, 11500..11510);
rpc_test!(best_tip_with_proof, 11510..11520);
rpc_test!(
    #[ignore]
    block,
    11520..11530
);
rpc_test!(ledger_query, 11530..11540);
rpc_test!(
    #[ignore]
    staged_ledger_aux_and_pending_coinbases_at_block,
    11540..11550
);

async fn rust_to_rust_rpc(
    query: P2pRpcRequest,
    response: P2pRpcResponse,
    ports: std::ops::Range<u16>,
) {
    let mut cluster = ClusterBuilder::new()
        .ports(ports)
        .start()
        .await
        .expect("should build cluster");

    let [node1, node2] = rust_nodes_from_default_config(&mut cluster).expect("no error");
    let [peer_id1, peer_id2] = peer_ids(&cluster, [node1, node2]);

    // wait for node1 to be ready to accept incoming conection s
    let listener_is_ready =
        try_wait_for_nodes_to_listen(&mut cluster, [node1], Duration::from_secs(2))
            .await
            .expect("no error");
    assert!(listener_is_ready, "node1 should be ready");

    // connect node2 to node1
    cluster.connect(node2, node1).expect("no error");

    // wait for node2 to have peer_id1 (node1) as its peer in `ready` state`
    let peers_are_connected = try_wait_for_nodes_to_connect(
        &mut cluster,
        [(node2, peer_id1), (node1, peer_id2)],
        Duration::from_secs(2),
    )
    .await
    .expect("no error");
    assert!(
        peers_are_connected,
        "node should be able to connect to {peer_id1}\nnode state: {:#?}",
        cluster.rust_node(node2).state().peers.get(&peer_id1)
    );

    // make sure node can send an RPC request
    assert!(
        rpc_ready(&mut cluster, [(node2, node1)], Duration::from_secs(2))
            .await
            .expect("no errors"),
        "rpc should be ready"
    );

    let request_id = send_request(&mut cluster, node2, node1, query);
    receive_request(&mut cluster, node2, node1, request_id).await;
    send_response(&mut cluster, node1, node2, request_id, response);
    receive_response(&mut cluster, node1, node2, request_id).await;
}

#[tokio::test]
async fn rust_to_many_rust_query() {
    const PEERS: usize = 20;
    let mut cluster = ClusterBuilder::new()
        .ports(11200..11400)
        .idle_duration(Duration::from_millis(100))
        .start()
        .await
        .expect("should build cluster");

    let [requestor, nodes @ ..]: [RustNodeId; PEERS + 1] =
        rust_nodes_from_default_config(&mut cluster).expect("no error");

    // wait for node1 to be ready to accept incoming conection s
    let listener_is_ready =
        try_wait_for_nodes_to_listen(&mut cluster, nodes, Duration::from_secs(2))
            .await
            .expect("no error");
    assert!(listener_is_ready, "node1 should be ready");

    // connect node2 to node1
    for node in nodes {
        cluster.connect(requestor, node).expect("no error");
    }

    // make sure node can send an RPC request
    assert!(
        rpc_ready(
            &mut cluster,
            nodes.map(|node| (requestor, node)),
            Duration::from_secs(10)
        )
        .await
        .expect("no errors"),
        "rpc should be ready"
    );

    let (query, _response) = rpc_from_json!("initial_peers");

    // send respests
    println!("=================== sending requests #1 ===============");
    let recv_peer_id: Vec<_> = nodes
        .into_iter()
        .map(|node| {
            let req_id = send_request(&mut cluster, requestor, node, query.clone());
            (node, requestor, req_id)
        })
        .collect();

    println!("=================== receiving requests #1 ===============");
    receive_all_requests(&mut cluster, recv_peer_id).await;

    // send responses
    println!("=================== sending requests #2 ===============");
    let requestor_peer_id: Vec<_> = nodes
        .into_iter()
        .map(|node| {
            let req_id = send_request(&mut cluster, node, requestor, query.clone());
            (requestor, node, req_id)
        })
        .collect();

    println!("=================== receiving requests #2 ===============");
    receive_all_requests(&mut cluster, requestor_peer_id).await;
}

#[tokio::test]
async fn rust_to_many_rust() {
    const PEERS: usize = 5;
    let mut cluster = ClusterBuilder::new()
        .ports(11400..11500)
        .idle_duration(Duration::from_millis(100))
        .start()
        .await
        .expect("should build cluster");

    let [requestor, nodes @ ..]: [RustNodeId; PEERS + 1] =
        rust_nodes_from_default_config(&mut cluster).expect("no error");

    // wait for node1 to be ready to accept incoming conection s
    let listener_is_ready =
        try_wait_for_nodes_to_listen(&mut cluster, nodes, Duration::from_secs(2))
            .await
            .expect("no error");
    assert!(listener_is_ready, "node1 should be ready");

    // connect node2 to node1
    for node in nodes {
        cluster.connect(requestor, node).expect("no error");
    }

    // make sure node can send an RPC request
    assert!(
        rpc_ready(
            &mut cluster,
            nodes.map(|node| (requestor, node)),
            Duration::from_secs(10)
        )
        .await
        .expect("no errors"),
        "rpc should be ready"
    );

    let rpcs = rpcs_from_json!(
        "initial_peers",
        "initial_peers",
        "best_tip_with_proof",
        // "block", // overflow
        "ledger_query",
        // "staged_ledger_aux_and_pending_coinbases_at_block" // error
    );

    // send respests
    println!("=================== sending requests ===============");
    let (recv_peer_id, responder_response_id): (Vec<_>, Vec<_>) = std::iter::zip(nodes, rpcs)
        .map(|(node, (query, response))| {
            let req_id = send_request(&mut cluster, requestor, node, query);
            ((node, requestor, req_id), (node, response, req_id))
        })
        .unzip();

    println!("=================== receiving requests ===============");
    receive_all_requests(&mut cluster, recv_peer_id).await;

    // send responses
    println!("=================== sending responses ===============");
    let requestor_peer_id = responder_response_id
        .into_iter()
        .map(|(responder, response, req_id)| {
            send_response(&mut cluster, responder, requestor, req_id, response);
            (requestor, responder, req_id)
        })
        .collect::<Vec<_>>();

    println!("=================== receiving responses ===============");
    receive_all_responses(&mut cluster, requestor_peer_id).await;
}

async fn rpc_ready<I>(
    cluster: &mut Cluster,
    peers: I,
    duration: Duration,
) -> Result<bool, ClusterEvent>
where
    I: IntoIterator<Item = (RustNodeId, RustNodeId)> + Clone,
    I::IntoIter: Clone,
{
    for (node1, node2) in peers.clone() {
        let peer_id = cluster.rust_node(node2).peer_id();
        let _ = cluster
            .rust_node_mut(node1)
            .dispatch_action(P2pChannelsRpcAction::Init { peer_id }); // might not succeed, as already in pending state
    }
    let peers2 = peers
        .clone()
        .into_iter()
        .flat_map(|(node1, node2)| {
            [
                (node1, cluster.rust_node(node2).peer_id()),
                (node2, cluster.rust_node(node1).peer_id()),
            ]
        })
        .collect::<Vec<_>>();
    cluster
        .try_stream()
        .take_during(duration)
        .try_any(rpc_channel_is_ready(peers2))
        .await
}

fn send_request(
    cluster: &mut Cluster,
    sender: RustNodeId,
    receiver: RustNodeId,
    request: P2pRpcRequest,
) -> P2pRpcId {
    let [_sender_id, receiver_id] = peer_ids(cluster, [sender, receiver]);
    let sender_receiver_channels = &cluster
        .rust_node(sender)
        .state()
        .get_ready_peer(&receiver_id)
        .expect("peer should be ready")
        .channels;
    assert!(
        matches!(
            sender_receiver_channels.rpc,
            P2pChannelsRpcState::Ready {
                local: P2pRpcLocalState::WaitingForRequest { .. }
                    | P2pRpcLocalState::Responded { .. },
                ..
            }
        ),
        "{sender:?}'s peer {receiver:?} not ready: {:#?}",
        sender_receiver_channels.rpc
    );
    let sender_rpc_id: u64 = sender_receiver_channels.next_local_rpc_id();
    assert!(
        cluster
            .rust_node_mut(sender)
            .dispatch_action(P2pChannelsRpcAction::RequestSend {
                peer_id: receiver_id,
                id: sender_rpc_id,
                request: Box::new(request),
            }),
        "dispatch rpc send query"
    );
    sender_rpc_id
}

async fn receive_request(
    cluster: &mut Cluster,
    sender: RustNodeId,
    receiver: RustNodeId,
    sender_rpc_id: P2pRpcId,
) {
    let [sender_id, _receiver_id] = peer_ids(cluster, [sender, receiver]);

    let any = try_wait_for_all_nodes_with_value(
        cluster,
        [(receiver, (sender_id, sender_rpc_id))],
        Duration::from_secs(10),
        |event| {
            if let RustNodeEvent::RpcChannelRequestReceived { peer_id, id, .. } = event {
                Some((peer_id, id))
            } else {
                None
            }
        },
    )
    .await
    .expect("no errors");
    assert!(any, "request should be received");
}

async fn receive_all_requests<I>(cluster: &mut Cluster, receiver_sender_id: I)
where
    I: IntoIterator<Item = (RustNodeId, RustNodeId, P2pRpcId)>,
{
    let any = try_wait_for_all_nodes_with_value(
        cluster,
        receiver_sender_id
            .into_iter()
            .map(|(receiver, sender, id)| (receiver, (cluster.rust_node(sender).peer_id(), id)))
            .collect::<Vec<_>>(),
        Duration::from_secs(10),
        |event| {
            if let RustNodeEvent::RpcChannelRequestReceived { peer_id, id, .. } = event {
                Some((peer_id, id))
            } else {
                None
            }
        },
    )
    .await
    .expect("no errors");
    assert!(any, "request should be received");
}

fn send_response(
    cluster: &mut Cluster,
    sender: RustNodeId,
    receiver: RustNodeId,
    request_id: u64,
    response: p2p::channels::rpc::P2pRpcResponse,
) {
    let [_sender_id, receiver_id] = peer_ids(cluster, [sender, receiver]);
    let sender_receiver_rpc = &cluster
        .rust_node(sender)
        .state()
        .get_ready_peer(&receiver_id)
        .expect("peer should be ready")
        .channels
        .rpc;
    assert!(
        matches!(
            sender_receiver_rpc,
            P2pChannelsRpcState::Ready { remote: P2pRpcRemoteState { pending_requests, .. }, .. }
                if pending_requests.iter().any(|s| s.id == request_id)
        ),
        "invalid remote state for {sender:?}'s peer: {:#?}",
        sender_receiver_rpc
    );
    assert!(
        cluster
            .rust_node_mut(sender)
            .dispatch_action(P2pChannelsRpcAction::ResponseSend {
                peer_id: receiver_id,
                id: request_id,
                response: Some(Box::new(response)),
            }),
        "dispatch rpc send query"
    );
}

async fn receive_response(
    cluster: &mut Cluster,
    sender: RustNodeId,
    receiver: RustNodeId,
    sender_rpc_id: P2pRpcId,
) {
    let [sender_id, _receiver_id] = peer_ids(cluster, [sender, receiver]);

    let any = try_wait_for_all_nodes_with_value(
        cluster,
        [(receiver, (sender_id, sender_rpc_id))],
        Duration::from_secs(20),
        |event| {
            if let RustNodeEvent::RpcChannelResponseReceived { peer_id, id, .. } = event {
                Some((peer_id, id))
            } else {
                None
            }
        },
    )
    .await
    .expect("no errors");
    assert!(any, "response should be received");
}

async fn receive_all_responses<I>(cluster: &mut Cluster, receiver_sender_id: I)
where
    I: IntoIterator<Item = (RustNodeId, RustNodeId, P2pRpcId)>,
{
    let any = try_wait_for_all_nodes_with_value(
        cluster,
        receiver_sender_id
            .into_iter()
            .map(|(receiver, sender, id)| (receiver, (cluster.rust_node(sender).peer_id(), id)))
            .collect::<Vec<_>>(),
        Duration::from_secs(20),
        |event| {
            if let RustNodeEvent::RpcChannelResponseReceived { peer_id, id, .. } = event {
                Some((peer_id, id))
            } else {
                None
            }
        },
    )
    .await
    .expect("no errors");
    assert!(any, "response should be received");
}
