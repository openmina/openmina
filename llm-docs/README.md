# OpenMina Architecture Documentation

This documentation provides a comprehensive overview of the OpenMina architecture, its components, and how they interact with each other. It is designed to help newcomers understand the system and serve as a reference for developers.

## High-Level Architecture

The OpenMina node is built using a state machine architecture where actions, reducers, and effects form the core of the system's behavior. This architecture provides a predictable and debuggable flow of data and operations.

- [System Overview](architecture/system-overview.md) - High-level overview of the entire system
- [State Machine Architecture](architecture/state-machine.md) - Details on the action-reducer-effect pattern
- [Data Flow](architecture/data-flow.md) - How data flows between different components

## Core Components

OpenMina consists of several key components that work together to form a complete node implementation:

- [Transition Frontier](components/transition-frontier/README.md) - Manages the blockchain state and transitions
- [SNARK System](components/snark/README.md) - Handles zero-knowledge proof verification
- [P2P Communication](components/p2p/README.md) - Manages peer-to-peer networking and communication

## Cross-Component Interactions

Understanding how components interact is crucial for grasping the system as a whole:

- [Block Processing Flow](architecture/block-processing.md) - How blocks are processed across components
- [Action-Reducer-Effect Cycle](architecture/action-reducer-effect.md) - Practical examples of the cycle in action

## Development Resources

- [Code Structure](architecture/code-structure.md) - Overview of the codebase organization
- [Contributing Guidelines](architecture/contributing.md) - How to contribute to OpenMina

## Diagrams Legend

Throughout this documentation, we use consistent visual elements in our diagrams:

- Blue boxes: Core components
- Green arrows: Data flow
- Orange elements: Actions
- Purple elements: State changes
- Yellow elements: Effects

All diagrams are created to be both informative and accessible, with text descriptions accompanying each visual element.
