

# Node Discovery Test

The main goal of this test is to ensure that Rust node can discover peers in the network, and is discoverable by other peers as well.

In this test, three nodes are started:



1. OCaml seed node with known address and peer ID
2. OCaml node with the seed node set as the initial peer
3. Rust node with the seed node set as the initial peer

Initially, the OCaml seed node has the other two nodes in its peer list, while the OCaml node and the Rust node only have the seed node.

![nodedisco1b](https://github.com/openmina/openmina/assets/60480123/9165203e-c262-4cc7-add7-bd1b2f1be88b)



The two (OCaml and Rust) non-seed nodes connect to the OCaml seed node


![nodedisco2b](https://github.com/openmina/openmina/assets/60480123/a8f29fb4-57f7-404c-b062-b3872616277b)




Once connected, they gain information about each other from the seed node.

They then make a connection between themselves. If the test is successful, then at the end of this process, each each node has each other in its peer list.



![nodedisco3b](https://github.com/openmina/openmina/assets/60480123/e36fa389-63c0-453e-a062-9b00570747cf)




## Implementation Details

The [Helm chart](https://github.com/openmina/helm-charts/tree/main/openmina-discovery) that is used to deploy the network also contains the [script](https://github.com/openmina/helm-charts/blob/main/openmina-discovery/scripts/test.sh) that performs the checks.
