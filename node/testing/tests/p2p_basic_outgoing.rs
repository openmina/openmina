use openmina_node_testing::scenarios::p2p::basic_outgoing_connections::{
    MakeMultipleOutgoingConnections, MakeOutgoingConnection,
};

mod common;

scenario_test!(
    make_outgoing_connection,
    MakeOutgoingConnection,
    MakeOutgoingConnection
);
scenario_test!(
    make_multiple_outgoing_connections,
    MakeMultipleOutgoingConnections,
    MakeMultipleOutgoingConnections
);
