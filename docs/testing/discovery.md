# Testing Openmina Node Discovery

The main goal of this test is to ensure that Rust node can discover peers in the
network, and is discoverable by other peers as well.

In this test, three nodes are started:
- OCaml seed node with known address and peer ID
- OCaml node with the seed node set as initial peer
- Rust node with the seed node set as initial peer

The test checks that eventually each node has each other in its peer list.


## Implementation Details

The [Helm chart](https://github.com/openmina/helm-charts/tree/main/openmina-discovery) that is used to deploy the network also contains the [script](https://github.com/openmina/helm-charts/blob/main/openmina-discovery/scripts/test.sh) that performs the checks.

