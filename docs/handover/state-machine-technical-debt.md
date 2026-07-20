# State Machine Technical Debt

This document covers architectural issues with the state machine implementation
across OpenMina. It focuses on patterns and consistency problems that affect the
overall design, rather than service-layer issues (covered in
`services-technical-debt.md`) or component-specific problems (covered in
individual `summary.md` files).

## Architecture Migration Issues

### Critical: Incomplete New-Style Migration

Several components still use the old-style state machine pattern with separate
reducer and effects files, creating inconsistency and maintenance burden.

#### Transition Frontier (Medium Priority)

- **Issue**: Implemented using old-style state machine pattern instead of
  new-style architecture
- **Impact**: Doesn't follow unified reducer pattern, effects directly access
  state via `state.get()` and `store.state()`
- **Solution**: Migrate to new-style unified reducers with thin effectful
  actions
- **Complexity**: High - large component with extensive sync logic

#### Transaction Pool (High Priority)

- **Issue**: Uses non-standard patterns that violate core architectural
  principles
- **Specific Problems**:
  - **Pending Actions Pattern**: Stores actions in `pending_actions` and
    retrieves later
  - **Blocking Service Calls**: Synchronous `get_accounts()` calls that block
    state machine
  - **Direct Global State Access**: Uses `unsafe_get_state()` bypassing proper
    state management
- **Impact**: Violates Redux principles, creates blocking behavior, breaks state
  encapsulation
- **Solution**: Complete refactoring to standard patterns (see
  `transaction_pool_refactoring.md`)
- **Complexity**: High - requires significant architectural changes

## State Machine Design Anti-patterns

### Complex Logic in Reducers and Enabling Conditions

A common anti-pattern is placing complex business logic directly in reducers and
enabling conditions instead of extracting it to state methods.

#### Issue

- **Reducers**: Complex state update logic embedded directly in reducer match
  arms
- **Enabling Conditions**: Heavy business logic in condition checks instead of
  simple boolean evaluations
- **Impact**: Reducers become hard to read, test, and maintain; enabling
  conditions become complex and difficult to understand

#### Solution

- **Extract State Methods**: Move complex logic to helper methods on the state
  struct
- **Thin Reducers**: Keep reducers focused on orchestrating state changes, not
  implementing them
- **Lightweight Enabling Conditions**: Use simple boolean checks that delegate
  to state methods when needed
- **Benefits**: Improved readability, testability, and maintainability; clearer
  separation of concerns

#### Pattern

```rust
// Bad: Complex logic in reducer
ComponentAction::ComplexUpdate { data } => {
    // 50+ lines of complex state update logic
    state.field1 = complex_calculation(data);
    state.field2.update_with_validation(data);
    // ... many more lines
}

// Good: Logic extracted to state method
ComponentAction::ComplexUpdate { data } => {
    state.handle_complex_update(data);
}

impl ComponentState {
    fn handle_complex_update(&mut self, data: Data) {
        // Complex logic lives here, easily testable
        self.field1 = self.calculate_field1(data);
        self.field2.update_with_validation(data);
    }
}
```

### Monolithic Reducers

Large, complex reducers that handle multiple concerns and should be decomposed
using state methods.

#### PubSub (963 lines)

- **Issue**: Single file handling caching, peer management, and protocol logic
- **Impact**: Difficult to maintain, mixed responsibilities, O(n) performance
  issues
- **Solution**: Move message handling logic to state methods, extract separate
  managers
- **Reference**: `p2p/src/network/pubsub/summary.md` for detailed analysis

#### Yamux (387 lines)

- **Issue**: Deep nesting (4-5 levels), complex buffer management mixing
  performance and correctness
- **Impact**: Hard to reason about, complex protocol-required flag combinations
- **Solution**: Extract state methods, improve documentation of flag
  combinations
- **Ongoing Work**: PR #1085 (`tweaks/yamux` branch) contains significant
  refactoring addressing these issues:
  - Action splitting: Broke down incoming frame handling into multiple focused
    actions
  - State method extraction: Moved state update logic from reducer to state
    methods
  - Reducer simplification: Reduced complexity and improved readability
  - Comprehensive testing: Added 574 lines of tests for better coverage
- **Reference**: `p2p/src/network/yamux/summary.md` for detailed refactoring
  plan

#### Scheduler (650 lines)

- **Issue**: Handles connection management, protocol selection, and error
  handling in single file
- **Impact**: Mixed responsibilities, difficult to maintain
- **Solution**: Break down into focused handlers, extract state methods
- **Note**: Component naming also needs addressing ("scheduler" manages
  connections, not scheduling)

## Enabling Conditions Issues

### Missing Implementations

- **Issue**: Some components lack proper enabling conditions, allowing invalid
  state transitions
- **Impact**: State machine can enter invalid states, debugging becomes
  difficult
- **Solution**: Implement comprehensive enabling conditions for all actions
- **Priority**: Medium - improves state machine correctness

### Misplaced Logic

- **Issue**: Complex business logic in enabling conditions instead of state
  methods
- **Impact**: Enabling conditions become hard to understand and maintain
- **Solution**: Move complex logic to state methods, keep enabling conditions
  simple
- **Pattern**: Enabling conditions should be lightweight boolean checks

## Service Integration Issues

### Blocking Operations

#### Transaction Pool Ledger Calls

- **Issue**: Synchronous `get_accounts()` calls block the state machine thread
- **Impact**: Violates async architecture, can freeze state machine progression
- **Solution**: Convert to async pattern with proper state management
- **Priority**: Critical - blocking operations are architectural violations

#### Missing Async Patterns

- **Issue**: Operations that should be async are implemented synchronously
- **Impact**: State machine becomes unresponsive during heavy operations
- **Solution**: Audit all service calls, ensure proper async patterns

## Communication and Error Handling

### Centralized Event Handling

- **Issue**: Event source centralizes all event handling instead of distributing
  to relevant effectful state machines
- **Impact**: Creates unnecessary coupling between unrelated components
- **Solution**: Forward domain-specific events to respective effectful state
  machines
- **Reference**: `node/src/event_source/summary.md` for detailed plan

### Missing Error Actions

#### Block Producer Error Paths

- **Issue**: Only `BlockProveSuccess` exists, no `BlockProveError` action
- **Impact**: Error paths use `todo!()` panics instead of proper error handling
- **Solution**: Implement error actions and proper error propagation
- **Priority**: Critical - affects system stability

### Inconsistent Callback Usage

- **Issue**: Some components don't use callbacks for decoupled communication
- **Impact**: Creates tight coupling, makes components hard to test
- **Solution**: Standardize callback usage across all components

### Panic-based Error Handling

- **Issue**: `todo!()` macros in production code paths (block proof failures,
  genesis load failures)
- **Impact**: System crashes instead of graceful error handling
- **Solution**: Implement proper error actions and integrate with error sink
  service (partially implemented in PR #1097)
- **Priority**: Critical - affects system stability

## Action System Technical Debt

### Action Type Generation System

**Current Implementation**: `node/src/action_kind.rs` is autogenerated by
`node/build.rs`:

- Scans all files ending in `_actions.rs` or `action.rs`
- Extracts all action types (structs/enums ending with `Action`)
- Generates a unified `ActionKind` enum consolidating all action types
- Implements `ActionKindGet` trait for all actions

**Benefits**:

- Eliminates macro overhead compared to using the `enum-kinds` crate
- Helps avoid recompiling all action-related code when a single action changes

**Technical Debt**:

- Build-time code generation adds complexity to the build process
- Creates dependency on build script for core type system functionality
- Temporary solution that requires manual maintenance of naming conventions

**Future Solutions**:

- Migrate to multiple disjoint action types resolved at runtime
- Explore trait-based approaches that don't require code generation
- See https://github.com/openmina/state_machine_exp for experimental approaches
- Consider compile-time solutions that don't require build scripts

**Priority**: Medium - works but creates maintenance burden and architectural
complexity

## P2P Layer Technical Debt

### Security Hardening Opportunities

- **Noise Session Key Cleanup**: Ephemeral session keys not zeroized
  (defense-in-depth improvement)
  - Reference: `p2p/src/network/noise/summary.md`

### Major Performance Issues

- **PubSub**: O(n) message lookups, 963-line monolithic reducer
  - Reference: `p2p/src/network/pubsub/summary.md`

### Architectural Issues

- **Kad Internals**: 912-line file mixing multiple concerns
  - Reference: `p2p/src/network/kad/summary.md`
- **Select Protocol Registry**: Hardcoded protocols limiting extensibility
  - Reference: `p2p/src/network/select/summary.md`

## Implementation Quality Issues

### Extensive TODOs

- **Issue**: Widespread TODO comments indicating incomplete functionality
- **Examples**:
  - VRF Evaluator: `todo!()` for `EpochContext::Waiting` state
  - User Command Verify: Missing error callback dispatch
  - Various components: Error handling improvements needed
- **Impact**: Indicates incomplete implementations and deferred decisions
- **Solution**: Systematic TODO resolution with proper prioritization

### Safety and Linting Improvements

#### Clippy Lints for Array Access and Arithmetic Safety

- **Issue**: Currently using `#[allow(clippy::arithmetic_side_effects)]` and
  `#[allow(clippy::indexing_slicing)]` in workspace configuration
- **Impact**: Allows potentially unsafe arithmetic operations and unchecked
  array access that could panic
- **Current State**: PR #1115 enables these lints as warnings but issues need to
  be fixed project-wide
- **Solution**: Fix all clippy warnings for these lints and enable them as
  errors
- **Priority**: High - affects runtime safety and reliability
- **Benefits**:
  - Prevents integer overflow/underflow in production
  - Eliminates potential panic points from array bounds violations
  - Improves overall code robustness

### Testing Constraints

- **Issue**: Architecture compromised by testing limitations
- **Example**: Ledger mask leak warnings that should be bug conditions but can't
  be due to testing
- **Impact**: Production code quality affected by testing constraints
- **Solution**: Improve testing infrastructure to remove architectural
  compromises

### Testing Framework Time Control

- **Issue**: Random time advancement in simulations causes unintentional
  timeouts (Issue #1140)
- **Current Problem**:
  - Time is advanced randomly during cluster simulations
  - Can cause unwanted RPC timeouts when time advances between request/response
  - Requires careful tuning of time ranges, making tests less deterministic
- **Impact**: Tests are slower than necessary and less reliable
- **Proposed Solution**: Event-based time advancement - pause execution until
  all async events are ready, then decide whether to deliver, drop, or delay
  them
- **Priority**: Medium - not required for mainnet but valuable for test
  reliability and speed

### Hard-coded Values

- **Issue**: Configuration values embedded in code instead of being configurable
- **Examples**:
  - PubSub: 5s, 300s timeouts, magic numbers (3, 10, 50, 100)
  - Signaling Discovery: 60-second rate limiting interval
  - RPC Channel: 5 concurrent requests limit
- **Impact**: Reduces flexibility, makes system harder to tune
- **Solution**: Extract configuration to proper configuration management system

## Recommendations and Priorities

### Phase 1: Critical Architecture Issues

1. **Fix Blocking Operations**: Convert Transaction Pool to async patterns
2. **Implement Missing Error Actions**: Add error paths for Block Producer and
   other components
3. **Remove Panic-based Error Handling**: Replace `todo!()` with proper error
   handling and complete error sink service integration (building on PR #1097)

### Phase 2: High-Priority Refactoring

1. **Break Down Monolithic Reducers**: Move logic to state methods for PubSub,
   Scheduler; complete Yamux refactoring (building on PR #1085)
2. **Standardize Communication Patterns**: Consistent callback usage across
   components
3. **Distribute Event Handling**: Move domain-specific events to respective
   state machines

### Phase 3: Medium-Priority Improvements

1. **Complete Architecture Migrations**: Migrate Transition Frontier to
   new-style patterns
2. **Implement Missing Enabling Conditions**: Ensure all actions have proper
   validation
3. **Security Hardening**: Implement secure key zeroization for P2P Noise
   session keys
4. **Improve Protocol Documentation**: Better documentation of complex protocol
   implementations like yamux
5. **Improve Service Boundaries**: Remove business logic from transport layers
6. **Resolve Extensive TODOs**: Systematic completion of deferred
   implementations
7. **Standardize Error Handling**: Consistent error propagation patterns

### Phase 4: Long-term Quality Improvements

1. **Extract Configuration**: Make hard-coded values configurable
2. **Improve Testing Infrastructure**: Remove architectural compromises
3. **Documentation**: Ensure all patterns are documented and consistent
4. **Performance Optimization**: Address O(n) lookups and other performance
   issues

## Cross-references

- **Service-layer technical debt**: See `services-technical-debt.md`
- **Component-specific issues**: See `summary.md` files in respective component
  directories
- **P2P component technical debt**: See individual summaries in
  `p2p/src/network/*/summary.md`
- **Architecture guidelines**: See `state-machine-structure.md` and
  `state-machine-development-guide.md`
- **Specific refactoring plans**: See `*_refactoring.md` files in component
  directories

## Conclusion

The state machine architecture is solid but needs work to achieve consistency
and maintainability. The biggest problems are blocking operations, missing error
handling, and panic-based error handling, which hurt system stability.
Completing the architectural migrations and reducer refactoring will make the
codebase easier to work with.
