
# Peer discovery with OCaml nodes

A diverse blockchain network consisting of at least two different node implementations is more decentralized, robust, and resilient to external as well as internal threats. However, with two different node implementations, we must also develop cross-compatibility between them. 

Peer discovery between the two node implementations is a good starting point. We want to ensure that native Mina nodes written in OCaml can discover and connect to the Rust-based Openmina node. 

We have developed a global test to ensure that any OCaml node can discover and connect to the Openmina node.


### Steps:

In these diagrams, we describe three different types of connections between peers:

<img width="814" alt="legend" src="https://github.com/openmina/openmina/assets/60480123/be9b6656-4c00-423b-8c18-b254d5ac4831">


1. We launch an OCaml node as a seed node. We run three additional non-seed OCaml nodes, connecting only to the seed node.



![PeerDiscovery-step1](https://github.com/openmina/openmina/assets/60480123/31ae46ae-41e9-4cae-898a-cc2f6dac51b4)



2. Wait 3 minutes for the OCaml nodes to start and connect to the seed node.

![PeerDiscovery-step2](https://github.com/openmina/openmina/assets/60480123/edd953f2-f0e0-4d70-8b29-782cfd963584)


3. Run the Openmina node (application under test). 

    Wait for the Openmina node to complete peer discovery and connect to all four OCaml nodes. This step ensures that the Openmina node can discover OCaml nodes.

![PeerDiscovery-step3](https://github.com/openmina/openmina/assets/60480123/6bddb7bf-7332-491b-ac7f-aa0697db349e)

  

4. Run another OCaml node that only knows the address of the OCaml seed node, and will only connect to the seed node. 


   ![PeerDiscovery-step4](https://github.com/openmina/openmina/assets/60480123/a7cd53c4-66cd-42a3-b6a7-3547a2c58f54)


5. Wait for the additional OCaml node to initiate a connection to the Openmina node. (This step ensures that the OCaml node can discover the Openmina node).


![PeerDiscovery-step5](https://github.com/openmina/openmina/assets/60480123/fb7b59ea-cc06-4c37-bb8e-0e7e47fb1bf0)


6. Fail the test on the timeout.
