# 扫描状态

这是一个数据结构，用于对需要交易 SNARK 证明的交易进行排队，并允许 SNARK 工作者并行处理这些交易 SNARK。

它被称为"扫描状态"是因为它将扫描类操作与 Mina 区块链状态的更新相结合。在函数式编程中，扫描会对元素序列应用特定操作，在每一步都跟踪中间结果并生成这些累积值的序列。

在这种情况下，元素序列是传入的交易区块流。

扫描状态表示一堆二叉树（每个节点有两个子节点的数据结构），其中树中的每个节点都是由 SNARK 工作者完成的 SNARK 任务。扫描状态会定期从树顶返回单个证明，证明树底所有交易的正确性。扫描状态定义了一个区块中的交易数量。

目前，Mina 允许每个区块包含 128 笔交易。区块生产者不必自己完成工作，可以从 SNARK 池中的可用出价购买任何 SNARK 工作者完成的工作。

**可能的状态：**

**todo** - SNARK 已请求但尚未完成。

**pending** - 此交易的 SNARK 已完成，但尚未包含在区块中

**done** - 已完成并包含在区块中

在二叉树底部有 128 个 SNARK 任务，可能附加到以下条目之一：

**支付** - Mina 资金转账（非 ZK 应用程序交易）和权益委托

**Zk 应用程序** - 由 zk-SNARKs 驱动的 Mina 协议智能合约（ZK 应用程序交易）。

**铸币** - 创建（或发行）之前不存在的新代币的交易。这些新代币用于支付区块生产者奖励。

**费用转账** - 用于支付 SNARK 工作。

**合并** - 将两个现有 SNARK 任务合并为一个（将两个 SNARK 任务 SNARK 化）

**空** - SNARK 任务的空槽位

在扫描状态中，两个基本常量定义了其行为：

**transaction_capacity_log_2** - 可以包含在区块中的最大交易数量

**work_delay** - 一定的延迟（以区块为单位），确保 SNARK 工作者有足够的时间完成 SNARK 工作。

**扫描状态中的树的数量受这些常量影响：**


## 最大交易数量


## TxnsMax=2^(TCL)

**TxnsMax** - 最大交易数量，

**TCL** - transaction_capacity_log_2

交易构成二叉树的无子叶节点。虽然这决定了可能的最大交易数量，但它将包含其他需要证明的操作，如 ZK 应用程序、费用转账和铸币。


## 最大树数量


## TreesMax=(TCL+1)*WD,

TreesMax = 最大树数量，

TCL = transaction_capacity_log_2，

WD = work_delay


## 最大证明数量（每个区块）


## MNP = 2^(TN/TCL+1)-1

MNP = 最大证明数量，

TN = 交易数量，

TCL = transaction_capacity_log_2

每堆二叉树代表 Mina 区块链的一个"状态"。扫描状态的二叉树从叶子到根填充 SNARK 任务。

### 扫描状态的工作原理

这里我们有一个说明扫描状态如何工作的示例。

**重要提示：注意这是一个非常简化的版本，我们仅使用支付作为待处理的 SNARK 任务的概念，尽管其他操作也可能请求 SNARK 工作。**

目前，Mina 的扫描状态允许一个区块中最多包含 128 笔交易，这意味着二叉树末端可能有多达 128 个叶节点。但是，为了解释这个概念，我们仅使用一个 `max_no_of_transactions` = 8（意味着一个区块中最多可以包含 8 笔交易或其他操作）和 `work_delay` = 1 的二叉树。

另外，请注意截图是从去中心化 Snark 工作者 UI 的浅色模式拍摄的，但默认情况下浏览器会以深色模式打开。

可能的状态：

**Todo** - SNARK 已请求但尚未完成。

**ongoing** - 有生成 SNARK 的承诺，但 SNARK 尚未完成

**pending** - 此交易的 SNARK 已完成（在 SNARK 池中就绪），但尚未包含在区块中

**done** - 已完成并包含在区块中

**available jobs** - 当新区块到来时，扫描状态会更新新的可用任务



1. 空扫描状态

在创世时，扫描状态为空，如下所示：

![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/28fe2b45-fffa-4e36-8ad3-ef257a509c58)

2. 树 0 填充了 8 笔交易的待办 SNARK 任务

我们添加 8 个待办 SNARK 任务（SNARK 任务请求）：

![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/99eb76eb-355e-4aed-a601-bc73837d2d6a)

在底行中，颜色方案描述了我们已请求 SNARK 证明的各种类型的交易。在此示例中，有 5 笔支付（浅色模式下为深灰色，深色模式下为白色）、2 笔费用转账（紫色）和 1 笔铸币（浅色模式下为青色，深色模式下为浅蓝色）。

上面是 8 个黄色块，代表待办 SNARK 任务，即已请求但尚未完成的 SNARK 任务。

3. 树 1 填充了 8 笔交易的待办 SNARK 任务

现在添加了另一个包含 8 个待办 SNARK 任务的区块，填充了二叉树 2 的叶子。由于 `work_delay` 为 1，此阶段不需要 SNARK 工作。

![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/3615e6d4-f053-4bf6-901e-49064af639f8)

在两个二叉树的底部，我们再次看到已请求 SNARK 证明的各种类型的交易。在新树中，有 2 笔支付、2 笔费用转账和 4 笔铸币。

上面我们看到在树 1 中，一些待办 SNARK 任务现在处于待处理状态（深紫色）。这意味着该交易的 SNARK 任务已完成，但尚未包含在区块中。

4. 树 2 填充了 8 个待办 SNARK 任务，树 0 获得前 4 个待处理合并任务

形成了另一个 SNARK 任务二叉树（树 3），树 1 中的交易获得了 SNARK 证明。此外，在树 1 中，为合并现有的 8 个已 SNARK 化交易创建了四个待处理 SNARK 任务。树 1 和树 2 的大多数 SNARK 任务已完成，但尚未添加到区块中：

![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/fe19b898-6dc0-4266-90a5-01035223453e)

5. 树 3 为其交易填充了 8 个待办 SNARK 任务，树 0 和树 1 有待处理的合并任务

第四个 SNARK 任务二叉树填充了 8 个新交易，树 2 的所有待处理 SNARK 任务都已完成，但尚未添加到区块中。在树 0 和树 1 中，现有交易获得了 SNARK 证明，另外为合并这些交易创建了四个待处理 SNARK 任务。

![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/1c13c745-8841-4bab-9acb-c31d53a23576)

6. 树 4 为其交易填充了待办 SNARK 任务，树 0 和树 1 有待处理的合并任务

树 4 为其交易填充了 8 个待办 SNARK 任务。由于 work_delay 为 1，这意味着在执行下一阶段的 SNARK 工作或合并之前有两个前置树的"延迟"。

实际上，这意味着树 2 现在将有 4 个待办 SNARK 任务来合并其交易，而树 0 将有 2 个待办 SNARK 任务来合并 4 个已经合并过一次的 SNARK 任务。

![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/d17c8b53-978e-4712-8d6e-6fd848896a79)

7. 树 5 为其交易填充了待办 SNARK 任务，树 0 几乎完成了所有 SNARK 的合并

这个过程重复进行，直到达到最大树数量（在本例中为 6）。随着每个额外的树的添加，由于 work_delay 为 1，在第 n-2 个前置树上执行合并 SNARK 工作。如果时间延迟更大，则这个数字会增加（例如，如果 work_delay 为 2，那么就会是第 n-3 个前置树）。

![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/7c048c08-31cb-4eae-89fa-8a15f59b25d4)

8. 达到最大树数量，在 n-2 个前置树上执行 SNARK 工作

一旦达到最大树数量，就会在剩余的树上执行合并 SNARK 工作：

![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/8a70e56f-6a7c-4d55-882e-aeec771bfe0f)

在下图中，树 0 和树 1 的 SNARK 已经合并了最大次数，因此这些树中的 SNARK 工作已完成。

![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/2768f8b7-4714-406d-b10e-c11300b50e63)

执行现有 SNARK 的合并任务，直到整个扫描状态都由 SNARK 组成，这意味着所有二叉树的交易都已经 SNARK 化，并且 SNARK 已经合并了 3 次。

在这种情况下，合并发生 3 次 = 添加 8 笔交易，然后进行 SNARK 化，然后合并成 4 个 SNARK，再合并成 2 个，最后合并成 1 个：

![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/da09bca8-2955-454f-85a7-d9acb53e679c)

当扫描状态如上图所示时，这意味着所有二叉树都已填充交易，这些交易已经 SNARK 化，并且 SNARK 证明已经合并到最大次数。

通过使用 SNARK 证明及其合并，我们最终得到了 Mina 区块链的一个非常压缩和轻量级的表示。

## 延迟问题

在交易被包含在区块中和该交易的证明被包含之间存在显著的延迟（以区块为单位）。

假设一笔交易被包含在第 1 个区块中（尚未证明）：

![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/ccd45621-abbc-4c9b-9bc7-4ac34e07802d)

这将在扫描状态的二叉树中填充一个代表交易的槽位：

![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/afff4f74-435f-474d-9c27-4b3ad6a95187)

在某个时刻，SNARK 工作者会接取这个任务并开始完成证明。

工作者完成证明后将其广播给其他节点，因此，完成的任务可能会被添加到它们的 SNARK 池中：

![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/d88cd86e-2788-4b68-92a6-154fcde36609)

在扫描状态中，二叉树更新以显示该交易的 SNARK 已完成（紫色），但尚未添加到区块中（绿色）。

![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/fb7767e1-b8cc-4734-b33f-85553819dafd)

最终，区块生产者会从其池中获取这个 SNARK 并将其包含在区块中。

区块生产者生成第 5 个区块，并包含为第 1 个区块中的交易创建的证明：

![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/a9b5ab38-0226-436a-bb39-d133799a73d5)

这仅意味着完成的任务已被添加到扫描状态中的树（在底部），现在需要通过与其他证明合并将这个证明冒泡到顶部：

![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/bdb23b8e-3f16-4b89-a2bd-7f2eb8f7d88d)

一旦这个完成的任务被添加到树的底部，就会创建一个新的合并任务（将这个证明与另一个底部证明合并）：

![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/bc53fb64-a9df-437a-ab98-ba103497379c)

最终，某个 SNARK 工作者会接取它，完成它，广播它，然后某个生产者会将其包含在另一个区块中。这将创建另一个合并任务，并且需要重复这个过程：

![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/1c2b704a-828b-444b-b7e7-122a9d49da9c)

最终，生产者生成一个区块，其中将包含树顶部的合并证明，该证明包含了我们在第一步中包含的交易的证明。

![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/772b567d-067e-4fd9-b374-84e2126a855f)

此时，已验证账本哈希被更新，现在该交易（以及其他交易）成为已验证账本的一部分。

如果不使用这些树，对于我们需要生成的每个交易证明，我们都需要等待前一个交易被证明后才能证明下一个。这将导致网络一直停滞等待证明就绪。此外，证明将无法并行解决（在当前设置中，我们可以让许多 SNARK 工作者解决最终会合并的不同证明）。

## 吞吐量

延迟问题导致吞吐量出现瓶颈。在某个时点，增加吞吐量会导致延迟急剧上升，因为将有太多 SNARK 任务等待合并并添加到已验证账本中：

![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/ef75ebc9-e1e4-4ba4-b6a2-50324e559567)
