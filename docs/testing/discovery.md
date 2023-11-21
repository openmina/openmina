
#### Multi node

We also want to test a scenario in which the network consists only of Openmina nodes. If the Openmina node is using a functionality that is implemented only in the OCaml node, and it does not perform it correctly, then we will not be able to see it with solo node test. 

For that purpose, we utilize a Multi node test, which involves a network of our nodes, without any third party, so that the testing is completely local and under our control.

_The source code for this test can be found in this repo:_

[https://github.com/openmina/openmina/blob/develop/node/testing/src/scenarios/multi_node/basic_connectivity_initial_joining.rs#L9](https://github.com/openmina/openmina/blob/develop/node/testing/src/scenarios/multi_node/basic_connectivity_initial_joining.rs#L9) 


#### How it's tested

**Node cluster**: We use a `ClusterRunner` utility to manage the setup and execution of test scenarios on a cluster of nodes.

**Scenarios Enumeration**: `Scenarios` is an enum with derived traits to support iterating over the scenarios, converting them to strings, etc. It lists different test scenarios such as `SoloNodeSyncRootSnarkedLedger`, `SoloNodeBasicConnectivityInitialJoining`, and `MultiNodeBasicConnectivityInitialJoining`. 

Each scenario has a related module (e.g., `multi_node::basic_connectivity_initial_joining::MultiNodeBasicConnectivityInitialJoining`) which contains the logic for the test.

**Scenario Implementation**: The `Scenarios` enum has methods for executing tests such as `run`, `run_and_save`, and `run_only`. These methods use the `ClusterRunner` to run the scenarios and potentially save the results.

**Dynamic Scenario Building**: There's logic (`blank_scenario`) to dynamically build a scenario's configuration, which then gets executed in a test run.

**Async/Await**: The methods within the `Scenarios` are asynchronous (`async`). The tests are run in an asynchronous context, which is common when dealing with network operations to allow for non-blocking I/O operations.

**Cluster Configuration and Execution**: `build_cluster_and_run_parents` is an asynchronous method for setting up a cluster according to a specified configuration and running all parent scenarios to prepare the environment for a specific test.


#### Node Discovery Test

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



**Implementation Details**

The [Helm chart](https://github.com/openmina/helm-charts/tree/main/openmina-discovery) that is used to deploy the network also contains the [script](https://github.com/openmina/helm-charts/blob/main/openmina-discovery/scripts/test.sh) that performs the checks.
