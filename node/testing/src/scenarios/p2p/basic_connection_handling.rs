/// Connections that are initiated outside of the state machine (e.g. by Kademlia) should be present in the state machine.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct AllConnections;
