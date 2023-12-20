
# The Scan State

This is a data structure that queues transactions requiring transaction snark proofs and allows parallel processing of these transaction snarks by snark workers. 

It is known as a _scan_ _state_ because it combines a scan-like operation with the updating of the Mina blockchain state. In functional programming, scans apply a specific operation to a sequence of elements, keeping track of the intermediate results at each step and producing a sequence of these accumulated values. 

The sequence of elements is in this case a stream of incoming blocks of transactions, and the operation

The scan state represents a stack of binary trees (data structures in which each node has two children), where each node in the tree is a snark job to be completed by a snark worker. The scan state periodically returns a single proof from the top of a tree that attests to the correctness of all transactions at the base of the tree. The scan state defines the number of transactions in a block. 

Currently, Mina allows 128 transactions to enter a block. A block producer, rather than completing the work themselves, may purchase the completed work from any snark workers from bids available in the snark pool.

**Possible states:**

**todo** - SNARK is requested, but hasn’t been completed yet.

**pending** - SNARK for this transaction has been completed, but not included in the block

**done** - completed and included in the block

At the bottom of the binary tree are 128 SNARK jobs that may be attached to one of the following entries:

**Payments** - Transfers of Mina funds (non-ZK App transactions) and stake delegations

**Zk Apps** - Mina Protocol smart contracts powered by zk-SNARKs (ZK App transactions).

**Coinbases** -  the transactions that create (or issue) new tokens that did not exist before. These new tokens are used to pay the block producer reward.

**Fee Transfers** - Used to pay for SNARK work.

**Merges** - Merging two existing SNARK jobs into one (SNARKing two SNARK jobs together)

**Empty** - Slot for SNARK job is empty

In the scan state, two basic constants define its behavior: 

**transaction_capacity_log_2** - the maximum number of transactions that can be included in a block

**work_delay** - a certain delay (in terms of blocks) that ensures there is enough time for the snark work to be completed by the snark workers.

**The number of trees in the scan state is affected by these constants:**


## Maximum Number of Transactions


## TxnsMax=2^(TCL)

**TxnsMax** - maximum number of transactions,

**TCL** - transaction_capacity_log_2

Transactions form the childless leaves of the binary tree. While this determines the max number of possible transactions, it will contain other actions that require proofs such as ZK Apps, Fee Transfers and Coinbases.


## Maximum Number of Trees


## TreesMax=(TCL+1)*WD,

TreesMax = maximum number of trees,

TCL = transaction_capacity_log_2,

WD = work_delay


## Maximum number of proofs (per block)


## MNP = 2^(TN/TCL+1)-1

MNP = max_number_of_proofs,

TN= number of transactions,

TCL = transaction_capacity_log_2

Each stack of binary trees represents one _state_ of the Mina blockchain. The Scan State’s binary trees are filled up with SNARK jobs, starting from their leafs to their root.


### How the Scan State works

Here we have an illustrated example of how the scan state works. 

**Important: Note that this is a very simplified version in which we only use the concept of Payments as pending SNARK jobs, even though other actions may request SNARK work.**

Currently, Mina’s scan state permits up to 128 transactions to be in a block, which means there may be up to 128 leaf nodes at the end of the binary tree. However, for the sake of explaining the concept, we only use a binary tree with a `max_no_of_transactions` = 8 (meaning there can be up to 8 transactions or other operations included in one block) and a `work_delay` = 1. 

Also, note that the screenshots are taken from the Decentralized Snark Worker UI light mode, but by default the browser will open it up in dark mode.

Possible states:

**Todo** - SNARK is requested, but hasn’t been completed yet.

**ongoing** - There is a commitment to produce a snark, the SNARK is not completed yet

**pending** - SNARK for this transaction has been completed (is ready in SNARK pool), but has not been included in a block

**done** - completed and included in the block

**available jobs** - When a new block comes, the scan state is updated with new available jobs 



1. Empty Scan State

At Genesis, the scan state is empty and looks like this:



![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/28fe2b45-fffa-4e36-8ad3-ef257a509c58)



2. Tree 0 is filled with todo SNARK jobs for 8 transactions 

We add 8 Todo SNARK jobs (requests for SNARK jobs):



![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/99eb76eb-355e-4aed-a601-bc73837d2d6a)


In the bottom row, the color scheme describes the various types of transactions for which we have requested SNARK proofs. In this example, there are 5 payments (dark grey in light mode, white in dark mode), 2 fee transfers (purple) and 1 coinbase (cyan in light mode, light blue in dark mode).

Above are 8 yellow blocks representing todo SNARK jobs, i.e. SNARK jobs that have been requested, but have yet to be completed.



3. Tree 1 is filled with todo SNARK jobs for 8 transactions 

Now another block of 8 todo SNARK jobs is added, filling the leaves of the binary Tree 2. Because there is a `work_delay` of 1, no SNARK work is required at this stage. 



![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/3615e6d4-f053-4bf6-901e-49064af639f8)


At the bottom of the two binary trees, we again see  the various types of transactions for which we have requested SNARK proofs. In the new tree, there are 2 payments , 2 fee transfers and 4 coinbases. 

Above, we see that in Tree 1, some todo SNARK jobs are now pending (dark purple). This means that the SNARK job for that transaction has been completed, but it has not been included in a block yet. 



4. Tree 2 is filled with 8 todo SNARK jobs, Tree 0 gets first 4 pending merge jobs

Another binary tree of SNARK jobs forms (Tree 3) and the transactions in Tree 1 are given SNARK proofs. Additionally, in Tree 1, four pending SNARK jobs are created for the merging of the existing 8 SNARKed transactions. Trees 1 and 2 have most of their SNARK jobs completed, but have not yet been added to a block:


![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/fe19b898-6dc0-4266-90a5-01035223453e)




5. Tree 3 is filled with 8 todo SNARK jobs for its transactions, trees 0 and 1 have pending merge jobs.

The fourth binary tree of SNARK jobs is filled with 8 new transactions, and Tree 2 has all of its pending SNARK jobs complete, but they are yet to be added to a block. In Tree 0 and Tree 1, the existing transactions are given SNARK proofs, plus four pending SNARK jobs are created for the merging of these transactions. 



![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/1c13c745-8841-4bab-9acb-c31d53a23576)



6. Tree 4 is filled with todo SNARK jobs for its transactions, trees 0 and 1 have pending merge jobs.

Tree 4 is filled with 8 Todo SNARK jobs for its transactions. Since there is a work_delay of 1, this means that there is a ‘latency’ of two preceding trees before the next phase of SNARK work or merging is performed. 

In practical terms, it means that Tree 2 will now have 4 Todo SNARK jobs to merge its transactions, while Tree 0 will have 2 Todo SNARK jobs to merge the 4 already once-merged SNARK jobs.



![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/d17c8b53-978e-4712-8d6e-6fd848896a79)




7. Tree 5 is filled with todo SNARK jobs for its transactions, Tree 0 has almost all of its SNARKs merged.

This process is repeated until the maximum number of trees (6 in this example) is reached. With each additional tree, merging SNARK work is performed on the n-2 preceding tree, since there is a work_delay of 1. If there is a greater time delay, then this number increases (for example a work_delay of 2, then it would be the n-3 preceding tree). 



![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/7c048c08-31cb-4eae-89fa-8a15f59b25d4)




8. Maximum number of trees is reached, SNARK work is performed on n-2 preceding trees.

Once the maximum number of trees is reached, merging SNARK work is performed on the remaining trees:


![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/8a70e56f-6a7c-4d55-882e-aeec771bfe0f)


In the diagram below, Trees 0 and 1 have had their SNARKs merged the maximum amount of times, so SNARK work in those trees is complete. 


![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/2768f8b7-4714-406d-b10e-c11300b50e63)


Merge jobs of existing SNARKs are performed until the entire scan state consists of SNARKs, meaning that all binary trees have had their transactions snarked, and the SNARKs have been merged 3 times.

In this case, the merging happens 3 times = 8 transactions are added, then SNARKed, then merged into 4 SNARKs, which are then merged into 2 and, finally, 1:



![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/da09bca8-2955-454f-85a7-d9acb53e679c)


When the Scan State looks as the picture above, it means that all binary trees have been filled out with transactions, these transactions have been SNARKed, and the SNARK proofs have been merged into SNARK proofs the maximum amount of times. 

Through the use of SNARK proofs and their merging, we end up with a very compressed and lightweight representation of the Mina blockchain.


## The latency issue

There is a significant delay (in terms of blocks) between when a transaction has been included in a block, and when a proof for that transaction is included.

Let’s say a transaction is included in block no. 1 (not yet proved):



![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/ccd45621-abbc-4c9b-9bc7-4ac34e07802d)


this will fill a slot representing a transaction in a binary tree in the scan state:


![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/afff4f74-435f-474d-9c27-4b3ad6a95187)


At some point a snark worker will pick up this job and start working on completing the proof.

The worker then completes the proof and broadcasts it to other peers, as a result, the completed job may be added to their snark pools:



![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/d88cd86e-2788-4b68-92a6-154fcde36609)


In the Scan State, the binary tree is updated to show that the SNARK for that transaction has been completed (is purple), but has yet to be added to a block (green).


![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/fb7767e1-b8cc-4734-b33f-85553819dafd)


Eventually, a block producer will pick up this snark from their pool and include it in a block.

The block producer produces block no. 5, and includes the proof created for the transaction from block no. 1



![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/a9b5ab38-0226-436a-bb39-d133799a73d5)


This only means that the completed job has been added to a tree in the scan state (at the base), what needs to happen now is that this proof needs to be bubbled up to the top (through merges with other proofs):



![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/bdb23b8e-3f16-4b89-a2bd-7f2eb8f7d88d)


Once this completed job got added to the base of a tree, a new merge job was created (which will merge this proof with another base proof):



![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/bc53fb64-a9df-437a-ab98-ba103497379c)


Eventually, some SNARK worker will pick it up, complete it, broadcast it, and some producer will include it in another block. This will then create another merge job, and the process needs to be repeated:



![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/1c2b704a-828b-444b-b7e7-122a9d49da9c)


Eventually, a producer produces a block into which it will include the merge proof at the top of the tree that contained the proof for the transaction we included in the first step. 


![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/772b567d-067e-4fd9-b374-84e2126a855f)


At this point, the snarked ledger hash is updated, and now the transaction (along with others) is part of the snarked ledger.

An alternative to having these trees would be that for each transaction proof we need to produce, we will need to wait for the previous transaction to be proved before we can prove the next. The result would be that the network would get stuck waiting for proofs to be ready all the time. Also, proofs would not be solvable in parallel (with the current setup we can have many snark workers solving different proofs that eventually get merged)


## Throughput

The issue with the latency results in a bottleneck for throughput. At a certain point, increasing throughput will cause the latency to skyrocket because there will be too many SNARK jobs waiting to be merged and added to the Snarked Ledger:

![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/ef75ebc9-e1e4-4ba4-b6a2-50324e559567)

