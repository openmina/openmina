
# Peer discovery with OCaml nodes

A diverse blockchain network consisting of at least two different node implementations is more decentralized, robust, and resilient to external as well as internal threats. However, with two different node implementations, we must also develop cross-compatibility between them. 

Peer discovery between the two node implementations is a good starting point. We want to ensure that native Mina nodes written in OCaml can discover and connect to the Rust-based Openmina node. 

We have developed a global test to ensure that any OCaml node can discover and connect to the Openmina node.


### Steps

In these diagrams, we describe three different types of connections between peers:

<img width="814" alt="legend" src="https://github.com/openmina/openmina/assets/60480123/c2bb452d-2104-4acf-bf69-ee3025c6d6da">




1. We launch an OCaml node as a seed node. We run three additional non-seed OCaml nodes, connecting only to the seed node.


![PeerDiscovery-step1](https://github.com/openmina/openmina/assets/60480123/48944999-602c-473f-856e-dcdeac584746)


2. Wait 3 minutes for the OCaml nodes to start and connect to the seed node.

![PeerDiscovery-step2](https://github.com/openmina/openmina/assets/60480123/25a75d51-6e27-4623-84ef-74084810d96e)

3. Run the Openmina node (application under test). 

    Wait for the Openmina node to complete peer discovery and connect to all four OCaml nodes. This step ensures that the Openmina node can discover OCaml nodes.

![PeerDiscovery-step3](https://github.com/openmina/openmina/assets/60480123/806fc07c-e4d8-4495-b4ff-d68738406353)


4. Run another OCaml node that only knows the address of the OCaml seed node, and will only connect to the seed node. 

![PeerDiscovery-step4](https://github.com/openmina/openmina/assets/60480123/0e1b12ac-3de6-4d68-84bb-99eeca9a107a)


5. Wait for the additional OCaml node to initiate a connection to the Openmina node. (This step ensures that the OCaml node can discover the Openmina node).

![PeerDiscovery-step5](https://github.com/openmina/openmina/assets/60480123/8dcd2cd5-8926-4502-b9c6-972f1dee9aae)


6. Fail the test on the timeout.
