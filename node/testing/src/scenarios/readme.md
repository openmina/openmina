# Scenario testing

**Node cluster**: We use a `ClusterRunner` utility to manage the setup and execution of test scenarios on a cluster of nodes.

**Scenarios Enumeration**: `Scenarios` is an enum with derived traits to support iterating over the scenarios, converting them to strings, etc. It lists different test scenarios such as `SoloNodeSyncRootSnarkedLedger`, `SoloNodeBasicConnectivityInitialJoining`, and `MultiNodeBasicConnectivityInitialJoining`. 

Each scenario has a related module (e.g., `multi_node::basic_connectivity_initial_joining::MultiNodeBasicConnectivityInitialJoining`) which contains the logic for the test.

**Scenario Implementation**: The `Scenarios` enum has methods for executing tests such as `run`, `run_and_save`, and `run_only`. These methods use the `ClusterRunner` to run the scenarios and potentially save the results.

**Dynamic Scenario Building**: There's logic (`blank_scenario`) to dynamically build a scenario's configuration, potentially from a JSON representation, which then gets executed in a test run.

**Async/Await**: The methods within the `Scenarios` are asynchronous (`async`), indicating that the tests are run in an asynchronous context, which is common when dealing with network operations to allow for non-blocking I/O operations.

**Parent Scenarios**: The `parent` and `parent_id` methods suggest that some scenarios may depend on others. The code constructs a hierarchy of test scenarios, ensuring parent scenarios are run before their children.

**Cluster Configuration and Execution**: `build_cluster_and_run_parents` is an asynchronous method for setting up a cluster according to a specified configuration and running all parent scenarios to prepare the environment for a specific test.

