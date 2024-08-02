mod builder;
pub use builder::*;

pub type Node = openmina_node_common::Node<crate::NodeService>;
