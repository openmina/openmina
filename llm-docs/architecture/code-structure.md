# Code Structure

This document provides an overview of the OpenMina codebase organization, helping developers understand where to find different components and how they relate to each other.

## Repository Structure

The OpenMina repository is organized into several top-level directories, each containing a specific component or functionality:

```
openmina/
├── cli/                  # Command-line interface
├── core/                 # Core types and utilities
├── docs/                 # Documentation
├── frontend/             # Web UI
├── ledger/               # Mina ledger implementation
├── node/                 # Main node implementation
│   ├── native/           # OS-specific code
│   └── testing/          # Testing framework
├── p2p/                  # P2P networking implementation
├── snark/                # SNARK verification
└── tools/                # Development tools
```

## Core Components

### core/

The `core/` directory contains basic types and utilities that are shared across different components of the node:

```
core/
├── src/
│   ├── block/            # Block-related types
│   ├── consensus/        # Consensus-related types
│   ├── p2p/              # P2P-related types
│   ├── snark/            # SNARK-related types
│   ├── transaction/      # Transaction-related types
│   └── lib.rs            # Main library entry point
```

### ledger/

The `ledger/` directory contains the Mina ledger implementation in Rust:

```
ledger/
├── src/
│   ├── account/          # Account-related types and operations
│   ├── ledger/           # Ledger implementation
│   ├── scan_state/       # Scan state implementation
│   └── lib.rs            # Main library entry point
```

### snark/

The `snark/` directory contains the SNARK verification implementation:

```
snark/
├── src/
│   ├── block_verify.rs              # Block verification
│   ├── block_verify_effectful.rs    # Block verification effects
│   ├── user_command_verify.rs       # Transaction verification
│   ├── user_command_verify_effectful.rs # Transaction verification effects
│   ├── work_verify.rs               # SNARK work verification
│   ├── work_verify_effectful.rs     # SNARK work verification effects
│   └── lib.rs                       # Main library entry point
```

### p2p/

The `p2p/` directory contains the P2P networking implementation:

```
p2p/
├── src/
│   ├── discovery/        # Peer discovery
│   ├── protocol/         # Protocol implementation
│   ├── transport/        # Transport layer
│   └── lib.rs            # Main library entry point
```

### node/

The `node/` directory combines all the business logic of the node:

```
node/
├── src/
│   ├── transition_frontier/  # Transition frontier implementation
│   │   ├── archive/          # Archive functionality
│   │   ├── candidate/        # Block candidate handling
│   │   ├── genesis/          # Genesis block handling
│   │   ├── sync/             # Sync functionality
│   │   └── mod.rs            # Main module entry point
│   ├── p2p/                  # P2P integration
│   ├── snark/                # SNARK integration
│   ├── service.rs            # Service abstraction
│   └── lib.rs                # Main library entry point
├── native/                   # OS-specific code
└── testing/                  # Testing framework
```

## State Machine Pattern

The codebase follows a state machine pattern, with each component typically organized into the following files:

```
component/
├── component_actions.rs      # Actions for the component
├── component_config.rs       # Configuration for the component
├── component_effects.rs      # Effects for the component
├── component_reducer.rs      # Reducer for the component
├── component_state.rs        # State for the component
└── mod.rs                    # Main module entry point
```

For example, the SNARK component follows this pattern:

```
snark/src/
├── snark_actions.rs          # SNARK actions
├── snark_config.rs           # SNARK configuration
├── snark_event.rs            # SNARK events
├── snark_reducer.rs          # SNARK reducer
├── snark_state.rs            # SNARK state
└── lib.rs                    # Main library entry point
```

## Code Organization Principles

The OpenMina codebase follows several principles for code organization:

1. **Component-Based**: The code is organized into components, each with a specific responsibility.
2. **State Machine Pattern**: Each component follows the state machine pattern, with actions, reducers, and effects.
3. **Separation of Concerns**: Different aspects of a component (state, actions, reducers, effects) are separated into different files.
4. **Abstraction of Services**: Services are abstracted away, allowing the core logic to be platform-independent.
5. **Testing First**: The codebase includes a comprehensive testing framework.

## Finding Your Way Around

When looking for specific functionality, consider the following:

1. **Start with the Component**: Identify which component is responsible for the functionality you're interested in.
2. **Look at the State**: The state definition provides a good overview of what the component manages.
3. **Explore Actions**: Actions define the operations that can be performed on the component.
4. **Check Reducers and Effects**: Reducers and effects define how the state is updated and what side effects occur.
5. **Follow Cross-Component Interactions**: Use the action dispatching in effects to follow how components interact.
