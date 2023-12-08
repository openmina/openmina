use openmina_node_testing::scenarios::p2p::basic_outgoing_connections::{
    MakeMultipleOutgoingConnections, MakeOutgoingConnection,
};

mod common;

scenario_test!(
    make_connection,
    MakeOutgoingConnection,
    MakeOutgoingConnection
);
scenario_test!(
    make_multiple_connections,
    MakeMultipleOutgoingConnections,
    MakeMultipleOutgoingConnections
);
