

# Node Discovery Test

The main goal of this test is to ensure that Rust node can discover peers in the network, and is discoverable by other peers as well.

In this test, three nodes are started:



1. OCaml seed node with known address and peer ID
2. OCaml node with the seed node set as the initial peer
3. Rust node with the seed node set as the initial peer

Initially, the OCaml seed node has the other two nodes in its peer list, while the OCaml node and the Rust node only have the seed node.

![nodedisco1](https://github.com/openmina/openmina/assets/60480123/a21fd1b0-0319-426d-b36a-ed8758af6722)



The two (OCaml and Rust) non-seed nodes connect to the OCaml seed node


![nodedisco2](https://github.com/openmina/openmina/assets/60480123/01ad842f-6c52-4e35-a217-e8d919344637)



Once connected, they gain information about each other from the seed node.

They then make a connection between themselves. If the test is successful, then at the end of this process, each each node has each other in its peer list.



![nodedisco3](https://github.com/openmina/openmina/assets/60480123/92fbb3b1-dd0d-432d-8b13-8ec4774e89e0)



## Implementation Details

The [Helm chart](https://github.com/openmina/helm-charts/tree/main/openmina-discovery) that is used to deploy the network also contains the [script](https://github.com/openmina/helm-charts/blob/main/openmina-discovery/scripts/test.sh) that performs the checks.
