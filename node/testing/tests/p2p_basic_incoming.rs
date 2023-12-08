use openmina_node_testing::scenarios::p2p::basic_incoming_connections::{
    AcceptIncomingConnection, AcceptMultipleIncomingConnections,
};

mod common;

scenario_test!(
    accept_incoming_connection,
    AcceptIncomingConnection,
    AcceptIncomingConnection
);
scenario_test!(
    accept_multiple_incoming_connections,
    AcceptMultipleIncomingConnections,
    AcceptMultipleIncomingConnections
);
