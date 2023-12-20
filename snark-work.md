
## Committing and producing new SNARKs

SNARK proofs are the backbone of the Mina blockchain and are used for verifying the validity of transactions, blocks and other SNARKs. We want to optimize the production of SNARKs so that the Mina blockchain can continue operating and expanding.


**This is an overview of SNARK workflows. Click on the picture for a higher resolution:**

[![image7](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/e98bec61-fe17-46cf-85d1-1c049dbc367d)](https://raw.githubusercontent.com/JanSlobodnik/pre-publishing/main/OpenMina%20%2B%20ZK%20Diagrams1.png)



### Receiving a Block to Update Available Jobs

Since blocks contain both transactions and SNARKs, each new block updates not only the staged ledger, but also the scan state (which contains SNARK proofs).


### 

![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/3eebefc5-b212-4f45-8f07-5e8a60ea9740)


Via the GossipSub (P2P), a node receives a new block that contains transactions and SNARK proofs.


![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/02f74256-6ac4-420e-8762-bfb39c72d073)


The Transition Frontier, which contains the staged ledger and the scan state, is updated. The staged ledger includes the new blocks. The scan state is updated with the new jobs.



![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/ebb3446c-8a26-4c20-9dca-e395e75470e8)



### Receiving a Commitment from a Rust Node

We want to avoid wasting time and resources in SNARK generation, specifically, we want to prevent Snarkers from working on the same pending snark job. For that purpose, we have introduced the notion of a _commitment_, in which SNARK workers commit to generating a proof for a pending SNARK job. 

This is a message made by SNARK workers that informs other peers in the network that they are committing to generating a proof for a pending SNARK job, so that other SNARK workers do not perform the same task and can instead make commitments to other SNARK jobs.

Commitments are made through an extra P2P layer that was created for this purpose.

![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/c3cfb963-33d6-4fc1-a433-d95ec02d7202)


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


![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/87a9992b-8819-437a-b6d4-7a0c14dd0c83)


### Receiving a SNARK from an OCaml node

The Rust node receives a SNARK proof from an OCaml node (an OCaml SNARK worker.



![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/51a4c945-7872-4468-ba75-4d8480967981)


The SNARK is verified in Rust.


![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/2144c569-52cc-4581-9e4b-0dff82712da8)



If it is the lowest fee SNARK for a specific pending SNARK job, then it is added to the SNARK pool, from where block producers can take SNARKs and add them into blocks.



![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/48c9540b-704e-42ec-b85b-b33434800283)


If it is the lowest fee SNARK for that job, then it is added to the SNARK pool



![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/b098ed2d-aa97-438a-8eda-f3399b849bb9)



After this, the updated SNARK pool with the completed (but not yet included in a block) SNARK is broadcast across the PubSub P2P network via the topic `mina/snark-work/1.0.0` (SNARK pool diff) _and_ directly to other nodes via WebRTC.



![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/f02fc1f4-e30e-4296-9a20-b7b57e2cf4a1)




![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/e9067fcc-2180-46b3-b445-a03ac77c2b16)


### Receiving SNARK from Rust node



![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/61bf68d7-7814-40a6-87cc-975a82d5cde8)



Node receives new available jobs and updates its SNARK pool.


### Committing and producing a SNARK

Once committed to a pending SNARK job, a SNARK worker will then produce a SNARK. 

If a commitment is for a SNARK job that is marked as not yet completed in the scan state and there are no prior commitments to that job (Alternatively, if there are other commitments, then it is the commitment with the cheapest fee for the SNARK work), it is added to the SNARK pool.



![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/4fe23e29-d95c-40ef-865a-7c2dfaeff9cc)


From the SNARK pool, it can be committed to one of the following:



1. An available job that hasn’t been completed or included in a block
2. A job that has been already performed, but the new commitment has a lower fee

 
If the commitment is for the lowest fee available, then the SNARK worker begins working on the SNARK proof, which is performed in OCaml. After it is done, the generated SNARK is sent back to the SNARK worker (Rust).


![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/73ffa6bc-3289-4316-84b9-634e3b7e3f7c)


A SNARK worker starts working on the committed job. The SNARK proof that is generated is then checked by a prover in OCaml, after which it is sent back to the SNARK worker.

The SNARK proof is then sent to the SNARK pool.


![image](https://github.com/JanSlobodnik/pre-publishing/assets/60480123/a28f5ce5-64ab-4d6e-8409-66970bdc2cb5)


From here, it is broadcast to Rust nodes directly via WebRTC P2P, and to OCaml nodes indirectly via the `mina/snark-work/1.0.0` (SNARK pool diff) topic of the PubSub P2P network.
