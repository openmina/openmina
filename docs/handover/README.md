# OpenMina Handover Documentation

This directory contains comprehensive documentation for understanding and
working with the OpenMina codebase. The documents are designed to provide a
structured onboarding experience for developers new to the project.

## Quick Start

### 👋 New to OpenMina?

**Start here**: [Architecture Walkthrough](architecture-walkthrough.md) →
[State Machine Structure](state-machine-structure.md) →
[State Machine Patterns](state-machine-patterns.md) →
[Project Organization](organization.md)

### 🔍 Looking for Something Specific?

- **Add new features**:
  [State Machine Development Guide](state-machine-development-guide.md) -
  includes quick reference checklists
- **Add RPC endpoints**: [Adding RPC Endpoints](adding-rpc-endpoints.md) - HTTP
  routing and service patterns
- **Testing framework**: [Testing Infrastructure](testing-infrastructure.md) -
  scenario-based testing with extensive examples
- **Browser deployment**: [Webnode Implementation](webnode.md) - WebAssembly
  build target for running OpenMina in browsers
- **Ledger implementation overview**: [Ledger Crate](ledger-crate.md) - OCaml
  port with Rust adaptations
- **Service integration**: [Services](services.md) - complete service inventory
  and patterns

### 📚 Quick Reference

- **Architecture**: Redux pattern, actions, reducers, effects - see
  [Glossary](#glossary-of-key-terms) for definitions
- **Services**: External I/O, threading, event-driven communication - see
  [Services](services.md) for complete inventory
- **Technical Debt**: [Services](services-technical-debt.md) (blocking
  operations, error handling) | [State Machine](state-machine-technical-debt.md)
  (architecture migration, anti-patterns)
- **Testing**: Scenario-based, multi-node simulation, fuzzing - see
  [Testing Infrastructure](testing-infrastructure.md) for extensive test
  scenarios

## Document Overview

- [Architecture Walkthrough](#architecture-walkthrough)
- [State Machine Structure](#state-machine-structure)
- [State Machine Patterns](#state-machine-patterns)
- [Project Organization](#project-organization)
- [Services](#services)
- [Ledger Crate](#ledger-crate)
- [Testing Infrastructure](#testing-infrastructure)
- [State Machine Development Guide](#state-machine-development-guide)
- [State Machine Debugging Guide](#state-machine-debugging-guide)
- [Adding RPC Endpoints](#adding-rpc-endpoints)
- [Fuzzing Infrastructure](#fuzzing-infrastructure)
- [Services Technical Debt](#services-technical-debt)
- [State Machine Technical Debt](#state-machine-technical-debt)
- [Webnode Implementation](#webnode-implementation)
- [Circuits](#circuits)
- [Debug Block Proof Generation](#debug-block-proof-generation)
- [Persistence](#persistence)
- [Mainnet Readiness](#mainnet-readiness)
- [Release Process](#release-process)
- [Component Summaries](#component-summaries)
- [Git Workflow](#git-workflow)
- [P2P Evolution Plan](#p2p-evolution-plan)
- [OCaml Coordination](#ocaml-coordination)

---

### [Architecture Walkthrough](architecture-walkthrough.md)

**Start here** - Provides a high-level overview of the OpenMina architecture,
including:

- Architecture philosophy and design principles
- Redux-style state machine architecture
- Core concepts (actions, enabling conditions, reducers, effects)
- Network configuration system (devnet/mainnet)
- Key component overview
- Development guidelines

### [State Machine Structure](state-machine-structure.md)

Deep dive into the state machine implementation details:

- Action types and hierarchies
- Reducer patterns (new vs old style)
- Substate contexts and access control
- Callback systems
- Migration from old to new architecture

### [State Machine Patterns](state-machine-patterns.md)

Analysis of common patterns across OpenMina's state machines:

- Lifecycle Pattern (Init/Pending/Success/Error) - Most common for async
  operations
- Request/Response Pattern - For communication-heavy components
- Custom Domain-Specific Patterns - For complex workflows
- Pattern selection guide and implementation best practices

### [Project Organization](organization.md)

Overview of the codebase structure and component purposes:

- Entry points and CLI structure
- Major components (node, p2p, snark, ledger)
- Supporting libraries and cryptographic primitives
- Development tools and utilities
- Dependency hierarchy

### [Services](services.md)

Detailed documentation of all system services:

- Service architecture principles
- Complete inventory of services
- Service trait definitions vs implementations
- Threading and communication patterns
- Service lifecycle management

### [Ledger Crate](ledger-crate.md)

High-level overview of the ledger implementation:

- Direct port from OCaml with Rust adaptations
- Core components (BaseLedger, Mask, Database)
- Transaction processing and validation
- Proof system integration
- Future refactoring plans

### [Testing Infrastructure](testing-infrastructure.md)

Comprehensive testing approaches and tools:

- Scenario-based testing
- Multi-node simulation
- Fuzz testing and differential testing
- Debugging capabilities
- Testing best practices

### [State Machine Development Guide](state-machine-development-guide.md)

Practical guide for implementing features with Redux patterns:

- Making changes to existing components
- Adding new state machines and actions
- Component communication patterns

### [State Machine Debugging Guide](state-machine-debugging-guide.md)

Comprehensive troubleshooting and investigation tools:

- ActionEvent macro and structured logging
- Recording, replay, and testing frameworks
- Common error patterns and solutions

### [Adding RPC Endpoints](adding-rpc-endpoints.md)

Focused guide for RPC-specific implementation patterns:

- RPC request/response type system
- HTTP routing with Warp framework
- Service interface integration
- WASM API exposure patterns

### [Fuzzing Infrastructure](fuzzing.md)

Basic fuzzing infrastructure for transaction processing (documentation
incomplete):

- Differential testing setup against OCaml implementation
- Limited mutation strategies and reproduction capabilities
- Basic debugging tools for fuzzer
- Note: Document is incomplete and contains unverified claims

### [Services Technical Debt](services-technical-debt.md)

Analysis of technical debt in the services layer:

- Service-by-service debt inventory
- Cross-cutting concerns (error handling, blocking operations)
- Prioritized recommendations for improvements
- Critical issues including intentional panics and synchronous operations

### [State Machine Technical Debt](state-machine-technical-debt.md)

Systemic architectural issues in state machine implementations:

- Architecture migration status (old vs new style)
- Anti-patterns and monolithic reducers
- Enabling conditions and service integration issues
- Safety and linting improvements (including clippy lints)
- Prioritized refactoring roadmap

### [Webnode Implementation](webnode.md)

WebAssembly build target for browser deployment:

- WASM compilation with browser-specific threading adaptations
- WebRTC-based P2P networking with pull-based protocol
- JavaScript API for web application integration
- Block production capabilities with browser-based SNARK proving
- Technical constraints and workarounds for browser environment

### [Circuits](circuits.md)

Circuit generation process and distribution:

- Circuit generation requires OCaml (OpenMina fork)
- Circuit blob repository and GitHub releases
- On-demand downloading and caching system
- Network-specific circuit configuration
- Verifier index loading and validation

### [Debug Block Proof Generation](debug-block-proof-generation.md)

Technical procedure for debugging failed block proofs:

- Decrypting and preparing failed proof dumps
- Running proof generation in both Rust and OCaml
- Comparing outputs for debugging discrepancies

### [Persistence](persistence.md)

Design for ledger persistence (not yet implemented):

- Memory reduction strategy for mainnet scale
- Fast restart capabilities
- SNARK verification result caching
- Critical for webnode browser constraints

### [Mainnet Readiness](mainnet-readiness.md)

Requirements and gaps for mainnet deployment:

- Critical missing features (persistence, wide merkle queries)
- Security audit requirements and error sink service integration
- Protocol compliance gaps
- Webnode-specific requirements
- Future compatibility considerations
- Rollout plan with testing requirements and deployment phases

### [Release Process](release-process.md)

Comprehensive release workflow:

- Monthly release cadence during active development
- Version management across all Cargo.toml files
- Changelog and Docker image updates
- CI/CD automation for multi-architecture builds

### [Component Summaries](component-summaries.md)

Tree view of all component technical debt documentation:

- Complete hierarchy of summary.md files
- Links to refactoring plans where available
- Organized by node, p2p, and snark subsystems

### [Git Workflow](git-workflow.md)

Git workflow and pull request policy used in the repository:

- Branch naming conventions and management
- PR development workflow and commit squashing policy
- Merge strategy and best practices
- Commit message format and examples

### [P2P Evolution Plan](p2p-evolution.md)

Evolution plan for Mina's P2P networking layer:

- Unified pull-based P2P design for the entire Mina ecosystem
- Current dual P2P architecture challenges (WebRTC + libp2p)
- Four-phase implementation strategy with QUIC transport integration
- Migration from libp2p to unified protocol across OCaml and Rust nodes
- Requires coordination with OCaml Mina team for ecosystem adoption

### [OCaml Coordination](ocaml-coordination.md)

Coordination needs between OCaml and Rust implementations:

- Maintenance burden coordination (circuit generation and fuzzer branches)
- Cross-implementation testing challenges and potential improvements
- Shared infrastructure dependencies (P2P evolution, archive service)
- Protocol compatibility coordination (hardfork handling)

## Recommended Reading Order

### For New Developers

- **Architecture Walkthrough** - Get the big picture
- **State Machine Structure** - Learn the core programming model
- **State Machine Patterns** - Understand common patterns and when to use them
- **Project Organization** - Understand the codebase layout
- **Services** - Understand external interactions
- **State Machine Development Guide** - Learn practical development patterns
- **Adding RPC Endpoints** - Learn to implement new API endpoints
- **Testing Infrastructure** - Learn how to test your changes
- **State Machine Technical Debt** - Understand known architectural issues and
  ongoing improvements

### For Protocol Developers

- **Architecture Walkthrough** - Understand the system design
- **Ledger Crate** - High-level overview of ledger implementation
- **State Machine Structure** - Learn state management patterns
- **Services** - Understand proof verification and block production
- **Services Technical Debt** - Be aware of service layer limitations

### For Quick Reference

- **Project Organization** - Find where components are located
- **Services** - Look up specific service interfaces
- **State Machine Structure** - Reference for action/reducer patterns
- **Technical Debt Documents** - Check current known issues and planned
  improvements

## Glossary of Key Terms

### Core Architecture Terms

**Redux Pattern** - State management architecture where all state changes happen
through actions processed by reducers. Provides predictable state updates and
easy debugging.

**Action** - Data structure representing a state change request. Can be stateful
(handled by reducers) or effectful (handled by effects).

**Reducer** - Pure function that takes current state and an action, returns new
state. In new-style components, also handles action dispatching.

**Effect** - Side-effect handler that interacts with external services. Should
be thin wrappers around service calls.

**Enabling Condition** - Function that determines if an action is valid in the
current state. Prevents invalid state transitions.

**State Machine** - Component that manages a specific domain's state through
actions and reducers (e.g., P2P, block producer).

**Stateful Action** - Action that modifies state and is processed by reducers.

**Effectful Action** - Action that triggers side effects (service calls) and is
processed by effects.

### State Management Terms

**Substate** - Abstraction that gives components access to their specific
portion of global state without coupling to global state structure.

**Dispatcher** - Interface for dispatching new actions from within reducers.

**Callback** - Mechanism for components to respond to async operations without
tight coupling.

**bug_condition!** - Macro for defensive programming that marks code paths that
should be unreachable if enabling conditions work correctly.

### Service Architecture Terms

**Service** - Component that handles external I/O, heavy computation, or async
operations. Runs in separate threads.

**Event** - Result from a service operation that gets converted to an action and
fed back to the state machine.

**EventSource** - Central service that aggregates events from all other services
and forwards them to the state machine.

**Deterministic Execution** - Principle that given the same inputs, the system
behaves identically. Achieved by isolating non-determinism to services.

### Development Terms

**New Style** - Current architecture pattern with unified reducers that handle
both state updates and action dispatching.

**Old Style** - Legacy architecture pattern with separate reducer and effects
files. Still used in transition frontier. For migration instructions, see
[ARCHITECTURE.md](../../ARCHITECTURE.md).

**Component** - Self-contained state machine handling a specific domain (e.g.,
transaction pool, P2P networking).

**Summary.md** - File in each component directory documenting purpose, technical
debt, and implementation notes.

**ActionEvent** - Derive macro that generates structured logging for actions.

### Network and Protocol Terms

**Network Configuration** - System supporting multiple networks (devnet/mainnet)
with different parameters.

**OCaml Compatibility** - Many components are direct ports from the OCaml Mina
implementation.

**P2P** - Peer-to-peer networking layer using libp2p with custom WebRTC
transport.

**SNARK** - Zero-knowledge proof system used for blockchain verification.

**Ledger** - Blockchain account state management system.

**Transition Frontier** - Core consensus and blockchain state management
component.

### Testing Terms

**Scenario** - Structured test case that can be recorded, saved, and replayed
deterministically.

**Recording/Replay** - System for capturing execution traces and replaying them
exactly for debugging.

**Differential Testing** - Comparing OpenMina behavior against the OCaml
implementation.

**Fuzzing** - Automated testing with random inputs to find edge cases.

## Key Concepts to Understand

Before diving into the documentation, familiarize yourself with these core
concepts:

1. **Redux Pattern** - State management through actions and reducers
2. **Deterministic Execution** - Separation of pure state logic from side
   effects
3. **Network Configuration** - Support for multiple networks (devnet/mainnet)
   with different parameters
4. **OCaml Compatibility** - Many components are direct ports from the OCaml
   implementation
5. **Service Architecture** - External interactions handled by services, not
   state machines

## Additional Resources

- **Source Code Comments** - Many modules have detailed inline documentation
- **Summary Files** - Look for `summary.md` files in component directories for
  technical debt and implementation notes
- **P2P WebRTC Documentation** - See [p2p/readme.md](../../p2p/readme.md) for
  details on the WebRTC implementation
- **Technical Debt Analysis** - See the technical debt documents for
  comprehensive analysis of known issues
