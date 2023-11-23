## Implementing the OCaml node’s verifiable random function (VRF) into the Openmina node’s block producer

In Proof of Stake (PoS) systems, block producers are chosen based on their stake. However, to avoid centralization and giving too much power to major stake-owners, we need to add an element of randomness in selecting new block producers. 

The role of a Verifiable Random Function (VRF) in this context is to add an element of randomness to the selection process, which complements the stake-based criteria. A VRF ensures that this selection process is fair and unbiased, preventing any single entity from gaining undue influence or control over the block production process.

VRFs are cryptographic primitives that generate a random number and proof that the number was legitimately generated. 
