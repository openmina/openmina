
## Committing and producing new SNARKs

SNARK proofs are the backbone of the Mina blockchain and are used for verifying the validity of transactions, blocks and other SNARKs. We want to optimize the production of SNARKs so that the Mina blockchain can continue operating and expanding.


**This is an overview of SNARK workflows. Click on the picture for a higher resolution:**

[![image](https://github.com/openmina/openmina/assets/60480123/f32f8d6c-c20a-4984-9cab-0dbdc5eec5b1)](https://raw.githubusercontent.com/openmina/openmina/docs/cleanup/docs/OpenMina%20%2B%20ZK%20Diagrams.png)



### Receiving a Block to Update Available Jobs

Since blocks contain both transactions and SNARKs, each new block updates not only the staged ledger, but also the scan state (which contains SNARK proofs).


<img width="519" alt="image" src="https://github.com/openmina/openmina/assets/60480123/db3fc349-d267-49ba-862a-c2a2bb0996c5">


Via the GossipSub (P2P), a node receives a new block that contains transactions and SNARK proofs.


![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/02f74256-6ac4-420e-8762-bfb39c72d073)


The work pool, which is a part of the modified SNARK pool and which contains the staged ledger and the scan state, is updated. The staged ledger includes the new blocks. The scan state is updated with the new jobs.



![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/ebb3446c-8a26-4c20-9dca-e395e75470e8)



### Receiving a Commitment from a Rust Node

We want to avoid wasting time and resources in SNARK generation, specifically, we want to prevent Snarkers from working on the same pending snark job. For that purpose, we have introduced the notion of a _commitment_, in which SNARK workers commit to generating a proof for a pending SNARK job. 

This is a message made by SNARK workers that informs other peers in the network that they are committing to generating a proof for a pending SNARK job, so that other SNARK workers do not perform the same task and can instead make commitments to other SNARK jobs.

Commitments are made through an extra P2P layer that was created for this purpose.

![image](https://github.com/openmina/openmina/assets/60480123/8966f501-c989-47dc-93e3-3477fbbdf5a3)


Commitments are sent across WebRTC, which enables direct communication between peers via the P2P network.


![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/9fa32591-0e63-40c1-91ab-c77a74c0e8b4)


Valid commitments are added to the _commitment pool_.

For a commitment to be added here, it has to: 



1. Be made for a SNARK job that is still marked as not yet completed in the scan state
2. Have no other prior commitment to that job. Alternatively, if there are other commitments, then only the one with the cheapest fee will be added.



![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/18f93479-6616-4bb8-8edb-11d62b122e4f)


The work pool, which is a part of the modified SNARK pool, is updated with a commitment (including its fee) for a specific pending SNARK job.


![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/5951772d-f0c0-4ad8-bb0b-089c5f42659e)


The commitments, once added to the commitment pool, are then broadcasted by the node other peers in the network through direct WebRTC P2P communication.

<img width="161" alt="image" src="https://github.com/openmina/openmina/assets/60480123/ab46c46c-5edb-4734-9387-97830bcf8e5f">


### Receiving a SNARK from an OCaml node

The Rust node receives a SNARK proof from an OCaml node (an OCaml SNARK worker).


![image](https://github.com/openmina/openmina/assets/60480123/fbde0660-df6d-4184-8d8c-b2f8832b711b)


The SNARK is verified.

<img width="135" alt="image" src="https://github.com/openmina/openmina/assets/60480123/21cecd28-fcb8-4315-b75c-a6b8580b715d">



If it is the lowest fee SNARK for a specific pending SNARK job, then it is added to the SNARK pool, from where block producers can take SNARKs and add them into blocks.


<img width="203" alt="image" src="https://github.com/openmina/openmina/assets/60480123/2294e2fc-05bc-42cd-93d4-afde20ec4342">


If it is the lowest fee SNARK for that job, then it is added to the SNARK pool



![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/b098ed2d-aa97-438a-8eda-f3399b849bb9)



After this, the updated SNARK pool with the completed (but not yet included in a block) SNARK is broadcast across the PubSub P2P network via the topic `mina/snark-work/1.0.0` (SNARK pool diff) _and_ directly to other nodes via WebRTC.



![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/f02fc1f4-e30e-4296-9a20-b7b57e2cf4a1)


<img width="318" alt="image" src="https://github.com/openmina/openmina/assets/60480123/46ec6804-b767-4054-aaeb-0287ab9cda09">


### Receiving SNARK from Rust node

Rust node sends SNARK via P2P.

<img width="342" alt="image" src="https://github.com/openmina/openmina/assets/60480123/067de8a5-246e-4b59-a85b-7f2393cc19c3">

SNARK is verified.

<img width="274" alt="image" src="https://github.com/openmina/openmina/assets/60480123/6fd500cd-90d8-4eed-b133-fb05f2a1798c">

If it is the lowest fee, it will be added to the SNARK pool.

<img width="266" alt="image" src="https://github.com/openmina/openmina/assets/60480123/c2947d12-020f-40f9-8831-0cbcdb08260c">


### Committing and producing a SNARK

Once committed to a pending SNARK job, a SNARK worker will then produce a SNARK. 


<img width="249" alt="image" src="https://github.com/openmina/openmina/assets/60480123/181fce0b-4c4b-485a-90b4-26323b4e9e9e">


If a commitment is for a SNARK job that is marked as not yet completed in the scan state and there are no prior commitments to that job (Alternatively, if there are other commitments, then it is the commitment with the cheapest fee for the SNARK work), it is added to the SNARK pool.


From the SNARK pool, it can be committed to one of the following:

<img width="421" alt="image" src="https://github.com/openmina/openmina/assets/60480123/889cb453-405f-4256-bc34-55964f0d5efd">



1. An available job that hasn’t been completed or included in a block
2. A job that has been already performed, but the new commitment has a lower fee

 
If the commitment is for the lowest fee available, then the SNARK worker begins working on the SNARK proof, which is performed in OCaml. After it is done, the generated SNARK is sent back to the SNARK worker (Rust).


<img width="281" alt="image" src="https://github.com/openmina/openmina/assets/60480123/fc9e5003-ff05-47d9-b35e-945516cf0090">

A SNARK worker starts working on the committed job. The SNARK proof that is generated is then checked by a prover in OCaml, after which it is sent back to the SNARK worker.

The SNARK proof is then sent to the SNARK pool.


![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/a28f5ce5-64ab-4d6e-8409-66970bdc2cb5)


From here, it is broadcast to Rust nodes directly via WebRTC P2P, and to OCaml nodes indirectly via the `mina/snark-work/1.0.0` (SNARK pool diff) topic of the PubSub P2P network.
