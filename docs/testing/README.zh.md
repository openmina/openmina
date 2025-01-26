# 测试

## 目录

- [P2P 测试](#p2p-测试)
    - [RPC](#rpc)
    - [Kademlia](#kademlia)
    - [Identify](#identify)
    - [Connection](#connection)
- [场景](#场景)
    - [连接发现](#连接发现)
    - [P2P 连接](#p2p-连接)
    - [Kademlia](#p2p-kademlia)
    - [Pubsub](#p2p-pubsub)
    - [P2P 入站](#p2p-入站)
    - [P2P 出站](#p2p-出站)
    - [单节点](#单节点)
    - [多节点](#多节点)
    - [记录/重放](#记录重放)

## P2P 测试

### [RPC](../../p2p/tests/rpc.rs)

* `rust_to_rust`: 测试 Rust 节点是否可以接收和发送响应到另一个 Rust 节点
* `rust_to_many_rust_query`: 测试 Rust 节点是否可以响应多个 Rust 节点
* `rust_to_many_rust`: 测试 Rust 节点是否可以向多个 Rust 节点发送请求
* RPC 测试，这些测试检查节点是否可以通过 RPC 正确通信：
* `initial_peers`: 检查初始节点是否正确发送和接收
* `best_tip_with_proof`: 检查最佳提示是否正确发送和接收
* `ledger_query`: 检查账本查询是否正确发送和接收
* `staged_ledger_aux_and_pending_coinbases_at_block`: 在 yamux 中失败，错误为 `attempt to subtract with overflow`
* `block`: 在 yamux 中失败，错误为 `attempt to subtract with overflow`

### [Kademlia](../../p2p/tests/kademlia.rs)

* `kademlia_routing_table`: 测试节点是否通过 Kademlia 接收节点
* `kademlia_incoming_routing_table`: 测试 Kademlia 是否通过传入节点更新
* `bootstrap_no_peers`: 测试即使没有节点传递，Kademlia 引导是否完成
* `discovery_seed_single_peer`: 测试节点通过 Kademlia 发现
* `discovery_seed_multiple_peers`: 测试节点发现和识别集成
* `test_bad_node`: 测试如果节点提供无效节点，我们是否能处理

### [Identify](../../p2p/tests/identify.rs)

* `rust_node_to_rust_node`: 测试 Rust 节点是否可以识别另一个 Rust 节点

### [Connection](../../p2p/tests/connection.rs)

* `rust_to_rust`: 测试 Rust 节点是否可以连接到 Rust 节点
* `rust_to_libp2p`: 测试我们的节点是否可以连接到 Rust libp2p
* `libp2p_to_rust`: 测试 libp2p 节点是否可以连接到 Rust 节点
* `mutual_rust_to_rust`: 测试一个 Rust 节点是否可以连接到第二个 Rust 节点，同时第二个节点尝试连接到第一个节点
* `mutual_rust_to_rust_many`: 测试多个 Rust 节点是否可以同时相互连接
* `mutual_rust_to_libp2p`: 测试 Rust 节点是否可以连接到 libp2p 节点，同时 libp2p 节点尝试连接到 Rust 节点
* `mutual_rust_to_libp2p_port_reuse`: 测试 Rust 节点是否可以解决自身与 libp2p 节点之间的相互连接，目前由于 [Issue #399](https://github.com/openmina/openmina/issues/399) 失败

## 场景

### [连接发现](../../node/testing/src/scenarios/multi_node/connection_discovery.rs)

我们想测试 Rust 节点是否可以连接并从 Ocaml 节点发现节点，反之亦然

* `RustToOCaml`:
此测试确保在 Rust 节点连接到具有已知地址的 OCaml 节点后，它将其地址添加到其 Kademlia 状态中。它还检查 OCaml 节点是否具有与 Rust 节点对应的正确 peer_id 和端口的节点。

* `OCamlToRust`:
此测试确保在 OCaml 节点连接到 Rust 节点后，其地址变得在 Rust 节点的 Kademlia 状态中可用。它还检查 OCaml 节点是否具有与 Rust 节点对应的正确 peer_id 和端口的节点。

* `RustToOCamlViaSeed`:
此测试确保 Rust 节点可以连接到 OCaml 节点，其地址只能从 OCaml 种子节点发现，并且 Rust 节点将其地址添加到其 Kademlia 状态中。它还检查 OCaml 节点是否具有与 Rust 节点对应的正确 peer_id 和端口的节点。最初，OCaml 种子节点在其节点列表中有其他两个节点，而 OCaml 节点和 Rust 节点只有种子节点。这两个（OCaml 和 Rust）非种子节点连接到 OCaml 种子节点。一旦连接，它们从种子节点获取彼此的信息。然后它们在彼此之间建立连接。如果测试成功，那么在此过程结束时，每个节点在其节点列表中都有彼此。

* `OCamlToRustViaSeed`: 此测试确保 OCaml 节点可以连接到 Rust 节点，其地址只能从 OCaml 种子节点发现，并且其地址变得在 Rust 节点的 Kademlia 状态中可用。它还检查 OCaml 节点是否具有与 Rust 节点对应的正确 peer_id 和端口的节点。

* `RustNodeAsSeed`: 此测试确保 Rust 节点可以作为种子节点运行，通过运行两个仅知道 Rust 节点地址的 OCaml 节点。在这些节点连接到 Rust 节点后，测试确保它们也有彼此的地址作为其节点。

### [P2P 连接](../../node/testing/tests/p2p_basic_connections.rs)

* `SimultaneousConnections`:
测试如果两个节点同时连接到彼此，它们应该连接，因此每个节点只有一个连接。

* `AllNodesConnectionsAreSymmetric`
所有节点之间的连接是对称的，即如果 node1 在其活动节点中有 node2，那么 node2 应该在其活动节点中有 node1。

* `SeedConnectionsAreSymmetric`
种子节点与其他节点的连接是对称的，即如果一个节点是种子的节点，那么它在其节点中有种子。

* `MaxNumberOfPeersIncoming`:
测试 Rust 节点的传入连接是否有限制。

* `MaxNumberOfPeersIs1`
两个最大节点数为 1 的节点可以相互连接。

### [P2P Kademlia](../../node/testing/tests/p2p_kad.rs)

与 Kademlia 层相关的测试。

* `KademliaBootstrap`:
测试节点是否发现另一个 Rust 节点并能够引导

### [P2P Pubsub](../../node/testing/tests/p2p_pubsub.rs)

与 pubsub 层相关的测试。

* `P2pReceiveBlock`
测试节点是否通过 meshsub 从节点接收区块

### [P2P 入站](../../node/testing/tests/p2p_basic_incoming.rs)

与处理传入连接相关的测试。

* `AcceptIncomingConnection`: 节点应该接受传入连接。
* `AcceptMultipleIncomingConnections`: 节点应该接受多个传入连接。

### [P2P 出站](../../node/testing/tests/p2p_basic_outgoing.rs)

与出站连接相关的测试

* `MakeOutgoingConnection`: 节点应该能够与监听节点建立出站连接。

* `MakeMultipleOutgoingConnections`: 节点应该能够创建多个出站连接。

* `DontConnectToNodeWithSameId`: 节点不应与具有相同 peer_id 的节点建立连接。

* `DontConnectToInitialPeerWithSameId`: 节点不应与具有相同 peer_id 的节点建立连接，即使其地址在初始节点中指定。

* `DontConnectToSelfInitialPeer`: 节点不应与自身建立连接，即使其地址在初始节点中指定。

* `ConnectToInitialPeers`: 节点应该能够连接到所有初始节点。

* `ConnectToUnavailableInitialPeers`: 节点应该重复连接到不可用的初始节点。

* `ConnectToInitialPeersBecomeReady`: 节点应该能够在初始节点准备好后连接到所有初始节点。

### [单节点](../../node/testing/tests/single_node.rs):

我们想测试 Rust 节点是否与 OCaml 节点兼容。我们通过尝试将 Openmina 节点连接到现有的 OCaml 测试网来实现这一点。

为此，我们使用 _solo node_，这是一个连接到 OCaml 节点网络的单个 Open Mina 节点。目前，我们使用公共测试网，但以后我们希望使用我们自己在集群上的 OCaml 节点网络。

* `SoloNodeBasicConnectivityAcceptIncoming`: 本地测试以确保 Openmina 节点可以接受来自现有 OCaml 节点的连接。

* `SoloNodeBasicConnectivityInitialJoining`: 本地测试以确保 Openmina 节点可以连接到现有 OCaml 测试网。

* `SoloNodeSyncRootSnarkedLedger`: 设置单个 Rust 节点并同步根 snarked 账本。

* `SoloNodeBootstrap`: 设置单个 Rust 节点并引导 snarked 账本、引导账本和区块。

### [多节点](../../node/testing/tests/multi_node.rs):

我们还想测试一个仅由 Openmina 节点组成的网络场景。如果 Openmina 节点使用的功能仅在 OCaml 节点中实现，并且它没有正确执行，那么我们将无法通过单节点测试看到它。为此，我们利用多节点测试，这涉及我们的节点网络，没有任何第三方，因此测试完全是本地的并且在我们的控制之下。

* `MultiNodeBasicConnectivityPeerDiscovery`: 测试我们的节点是否能够通过 Ocaml 种子节点发现 Ocaml 节点。

* `MultiNodeBasicConnectivityInitialJoining`: 测试节点是否在允许的最小和最大节点数之间保持节点数量。

### [记录/重放](../../node/testing/tests/record_replay.rs)

* `RecordReplayBootstrap`: 在启用状态和输入操作记录器的情况下引导一个 Rust 节点，并确保我们可以成功重放它。

* `RecordReplayBlockProduction`: 确保我们可以成功记录和重放集群中的多个节点 + 区块生产。