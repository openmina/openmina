## 提交和生成新的 SNARK

SNARK 证明是 Mina 区块链的核心，用于验证交易、区块和其他 SNARK 的有效性。我们希望优化 SNARK 的生产过程，以确保 Mina 区块链能够持续运行和扩展。

**这是 SNARK 工作流程的概述。点击图片查看高分辨率版本：**

[![图片](https://github.com/openmina/openmina/assets/60480123/f32f8d6c-c20a-4984-9cab-0dbdc5eec5b1)](https://raw.githubusercontent.com/openmina/openmina/docs/cleanup/docs/OpenMina%20%2B%20ZK%20Diagrams.png)

### 接收区块以更新可用任务

由于区块同时包含交易和 SNARK，每个新区块不仅会更新暂存账本，还会更新扫描状态（包含 SNARK 证明）。

<img width="519" alt="图片" src="https://github.com/openmina/openmina/assets/60480123/db3fc349-d267-49ba-862a-c2a2bb0996c5">

通过 GossipSub（P2P），节点接收包含交易和 SNARK 证明的新区块。

![图片](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/02f74256-6ac4-420e-8762-bfb39c72d073)

工作池（作为修改后的 SNARK 池的一部分，包含暂存账本和扫描状态）会被更新。暂存账本包含新区块，扫描状态则用新任务更新。

![图片](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/ebb3446c-8a26-4c20-9dca-e395e75470e8)

### 接收来自 Rust 节点的承诺

我们希望避免在 SNARK 生成过程中浪费时间和资源，特别是要防止 SNARK 生成者在同一个待处理的 SNARK 任务上重复工作。为此，我们引入了"承诺"的概念，即 SNARK 工作者承诺为待处理的 SNARK 任务生成证明。

这是 SNARK 工作者发出的消息，通知网络中的其他节点他们正在承诺为待处理的 SNARK 任务生成证明，这样其他 SNARK 工作者就不会执行相同的任务，而是可以承诺其他 SNARK 任务。

承诺通过专门为此创建的额外 P2P 层进行。

![图片](https://github.com/openmina/openmina/assets/60480123/8966f501-c989-47dc-93e3-3477fbbdf5a3)

承诺通过 WebRTC 发送，这使得节点之间可以通过 P2P 网络直接通信。

![图片](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/9fa32591-0e63-40c1-91ab-c77a74c0e8b4)

有效的承诺会被添加到"承诺池"中。

要添加承诺，必须满足以下条件：

1. 承诺的 SNARK 任务在扫描状态中仍标记为未完成
2. 该任务没有其他先前的承诺。或者，如果有其他承诺，则只添加费用最低的承诺。

<img width="199" alt="图片" src="https://github.com/openmina/openmina/assets/60480123/4a422932-30a4-4add-b3f5-4b70f159fd5e">

工作池（作为修改后的 SNARK 池的一部分）会用特定待处理 SNARK 任务的承诺（包括其费用）进行更新。

![图片](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/5951772d-f0c0-4ad8-bb0b-089c5f42659e">

承诺一旦添加到承诺池中，节点就会通过直接的 WebRTC P2P 通信将其广播给网络中的其他节点。

<img width="330" alt="图片" src="https://github.com/openmina/openmina/assets/60480123/c3620050-4082-4f2f-860c-94f292c01a2c">

### 接收来自 OCaml 节点的 SNARK

Rust 节点从 OCaml 节点（OCaml SNARK 工作者）接收 SNARK 证明。

![图片](https://github.com/openmina/openmina/assets/60480123/fbde0660-df6d-4184-8d8c-b2f8832b711b)

SNARK 会被验证。

<img width="393" alt="图片" src="https://github.com/openmina/openmina/assets/60480123/7e069f8d-3abe-40ca-a58d-0cd6c9e7ba2a">

如果它是特定待处理 SNARK 任务的最低费用 SNARK，则会被添加到 SNARK 池中，区块生产者可以从这里获取 SNARK 并将其添加到区块中。

<img width="274" alt="图片" src="https://github.com/openmina/openmina/assets/60480123/8e54f113-a75a-4f25-a520-3850a312ef65">

之后，更新后的 SNARK 池（包含已完成但尚未包含在区块中的 SNARK）会通过 `mina/snark-work/1.0.0`（SNARK 池差异）主题在 PubSub P2P 网络上广播，并通过 WebRTC 直接发送给其他节点。

![图片](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/f02fc1f4-e30e-4296-9a20-b7b57e2cf4a1)

<img width="318" alt="图片" src="https://github.com/openmina/openmina/assets/60480123/46ec6804-b767-4054-aaeb-0287ab9cda09">

### 接收来自 Rust 节点的 SNARK

Rust 节点通过 P2P 发送 SNARK。

<img width="342" alt="图片" src="https://github.com/openmina/openmina/assets/60480123/067de8a5-246e-4b59-a85b-7f2393cc19c3">

SNARK 会被验证。

<img width="488" alt="图片" src="https://github.com/openmina/openmina/assets/60480123/91c85d8d-4a63-4ace-a21f-fad08e57da34">

如果是最低费用，它将被添加到 SNARK 池中。

### 提交和生成 SNARK

一旦承诺了待处理的 SNARK 任务，SNARK 工作者就会生成 SNARK。

<img width="249" alt="图片" src="https://github.com/openmina/openmina/assets/60480123/181fce0b-4c4b-485a-90b4-26323b4e9e9e">

如果承诺的 SNARK 任务在扫描状态中标记为未完成，且没有该任务的先前承诺（或者如果有其他承诺，则它是该 SNARK 工作的最低费用承诺），它就会被添加到 SNARK 池中。

从 SNARK 池中，它可以被提交给以下之一：

<img width="421" alt="图片" src="https://github.com/openmina/openmina/assets/60480123/889cb453-405f-4256-bc34-55964f0d5efd">

1. 尚未完成或包含在区块中的可用任务
2. 已经执行但新承诺具有更低费用的任务

如果承诺是可用的最低费用，那么 SNARK 工作者就会开始在 OCaml 中生成 SNARK 证明。完成后，生成的 SNARK 会被发送回 SNARK 工作者（Rust）。

<img width="281" alt="图片" src="https://github.com/openmina/openmina/assets/60480123/fc9e5003-ff05-47d9-b35e-945516cf0090">

SNARK 工作者开始处理已承诺的任务。生成的 SNARK 证明会由 OCaml 中的证明者检查，然后发送回 SNARK 工作者。

SNARK 证明随后被发送到 SNARK 池。

<img width="336" alt="图片" src="https://github.com/openmina/openmina/assets/60480123/7c937f0b-9e4c-491e-a785-86ecf68754ff">

从这里，它会通过 WebRTC P2P 直接广播给 Rust 节点，并通过 PubSub P2P 网络的 `mina/snark-work/1.0.0`（SNARK 池差异）主题间接广播给 OCaml 节点。
