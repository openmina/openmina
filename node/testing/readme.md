
# Peer discovery with OCaml nodes

A diverse blockchain network consisting of at least two different node implementations is more decentralized, robust, and resilient to external as well as internal threats. However, with two different node implementations, we must also develop cross-compatibility between them. 

Peer discovery between the two node implementations is a good starting point. We want to ensure that native Mina nodes written in OCaml can discover and connect to the Rust-based Openmina node. 

We have developed a global test to ensure that any OCaml node can discover and connect to the Openmina node.


### Steps:

In these diagrams, we describe three different types of connections between peers:

<img width="407" alt="legend" src="https://github.com/openmina/openmina/assets/60480123/a3eb28e8-57cb-49b2-aecc-dcef5d60f2e7">


1. We launch an OCaml node as a seed node. We run three additional non-seed OCaml nodes, connecting only to the seed node.



![PeerDiscovery-step1](https://github.com/openmina/openmina/assets/60480123/94b5c26b-1530-43d3-b78e-30ba71c14c9e)



2. Wait 3 minutes for the OCaml nodes to start and connect to the seed node.

![PeerDiscovery-step2](https://github.com/openmina/openmina/assets/60480123/8106f03c-49d4-4dd4-a5d8-9068a6a877f7)


3. Run the Openmina node (application under test). 

    Wait for the Openmina node to complete peer discovery and connect to all four OCaml nodes. This step ensures that the Openmina node can discover OCaml nodes.


  ![PeerDiscovery-step3](https://github.com/openmina/openmina/assets/60480123/9067a008-6dfe-41ac-b15a-3ff597ccf89d)


4. Run another OCaml node that only knows the address of the OCaml seed node, and will only connect to the seed node. 

![PeerDiscovery-step4](https://github.com/openmina/openmina/assets/60480123/03e30009-7ee7-4d4e-821f-a52c810f3725)


5. Wait for the additional OCaml node to initiate a connection to the Openmina node. (This step ensures that the OCaml node can discover the Openmina node).

![PeerDiscovery-step5](https://github.com/openmina/openmina/assets/60480123/98a2383e-66b4-448b-8ee1-9217764d514d)


6. Fail the test on the timeout.
