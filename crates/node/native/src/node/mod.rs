//! # Native Node
//!
//! This module provides the native platform implementation of the Mina node.
//!
//! It specializes the generic [`mina_node_common::Node`] with the native
//! [`NodeService`](crate::NodeService) and provides [`NodeBuilder`] for
//! configuring and constructing nodes with P2P networking, block production,
//! HTTP server, and other components.

mod builder;
pub use builder::*;

/// Native node type alias, combining the common node logic with native services.
pub type Node = mina_node_common::Node<crate::NodeService>;
