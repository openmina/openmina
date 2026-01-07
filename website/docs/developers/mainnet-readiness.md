---
sidebar_position: 4
title: Mainnet Readiness
description: Key features and improvements required for mainnet deployment
slug: /developers/mainnet-readiness
---

# Mainnet Readiness

This document outlines the key features and improvements required for the Mina
Rust node to be ready for mainnet deployment.

## Critical Requirements

### 1. Persistence Implementation

**Status**: Draft design
([Issue #522](https://github.com/o1-labs/mina-rust/issues/522), see
[persistence-design](persistence-design))

The ledger is currently kept entirely in memory, which is not sustainable for
mainnet's scale. Persistence is required for:

- Reducing memory usage to handle mainnet-sized ledgers and amount of SNARKs
- Enabling fast node restarts without full resync
- Supporting [web nodes](frontend/web-node) with browser storage constraints
- Providing a clean foundation for implementing SNARK verification deduplication

**Note**: There is a very old implementation for on-disk storage in
`ledger/src/ondisk` that was never used - a lightweight key-value store
implemented to avoid the RocksDB dependency. This is unrelated to the new
persistence design which intends to solve persistence for everything, not just
the ledger. But the old implementation may be worth revisiting anyway.

**Performance Impact**: The importance of SNARK verification deduplication for
mainnet performance has been demonstrated in the OCaml node, where we achieved
dramatic improvements (8-14 seconds → 0.015 seconds for block application). See
the "SNARK Verification Deduplication" section in
[persistence-design](persistence-design) for details.

### 2. Wide Merkle Queries

**Status**: Not implemented
([Issue #1086](https://github.com/o1-labs/mina-rust/issues/1086))

Wide merkle queries are needed for:

- Protocol compatibility
- Faster synchronization

### 3. Delta Chain Proof Verification

**Status**: Not implemented
([Issue #1017](https://github.com/o1-labs/mina-rust/issues/1017))

When verifying blocks, the Mina Rust node should verify the delta chain proofs.

### 4. Automatic Hardfork Handling

**Status**: Not implemented

The Mina Rust node needs a mechanism to automatically handle protocol hardforks
to maintain compatibility with the network as it evolves.

**Requirements**:

- Detection of hardfork activation points
- Automatic protocol rule updates
- Backward compatibility during transition periods
- Clear migration paths for node operators

### 5. Archive Node Functionality

**Status**: Partially implemented

For full mainnet support, robust archive functionality is essential:

- **Block Storage**: Persistent storage of all blocks and transactions
- **Query Interface**: Efficient querying of historical data
- **Data Integrity**: Verification of archived data consistency
- **Reorg Handling**: Proper handling of blockchain reorganizations

### 6. Performance Optimization

**Status**: Ongoing

Several performance optimizations are critical for mainnet scale:

#### SNARK Pool Management

- Efficient pruning of outdated SNARK work
- Memory-efficient pool storage
- Fast lookup and retrieval mechanisms

#### Network Efficiency

- Optimized peer discovery and connection management
- Bandwidth usage optimization
- Connection pooling and reuse

#### Block Processing

- Parallel transaction verification where possible
- Optimized state transitions
- Efficient memory usage during processing

### 7. Monitoring and Observability

**Status**: Basic implementation

Production deployment requires comprehensive monitoring:

- **Metrics Collection**: Key performance indicators and health metrics
- **Logging**: Structured logging for debugging and analysis
- **Alerting**: Automated alerts for critical issues
- **Dashboard Integration**: Integration with monitoring dashboards

## Secondary Requirements

### Enhanced Security Features

#### Rate Limiting

- Connection rate limiting to prevent DoS attacks
- Message processing rate limits
- Resource usage monitoring

#### Input Validation

- Comprehensive validation of network messages
- Boundary checking for all external inputs
- Proper error handling and recovery

### Operational Features

#### Configuration Management

- Flexible configuration system
- Hot reloading of certain configuration changes
- Environment-specific configurations

#### Health Checks

- Comprehensive health check endpoints
- Dependency health monitoring
- Graceful degradation capabilities

### Testing and Validation

#### Stress Testing

- Load testing under mainnet-like conditions
- Memory usage validation with large datasets
- Network partition recovery testing

#### Compatibility Testing

- Interoperability with OCaml nodes
- Protocol compliance verification
- Hardfork transition testing

## Implementation Priority

### Phase 1: Critical Path (Mainnet Blockers)

1. **Persistence Implementation** - Essential for memory management
2. **Wide Merkle Queries** - Required for protocol compatibility
3. **Delta Chain Proof Verification** - Security requirement
4. **Performance Optimization** - Basic scalability needs

### Phase 2: Production Readiness

1. **Automatic Hardfork Handling** - Long-term compatibility
2. **Enhanced Archive Functionality** - Complete historical data support
3. **Monitoring and Observability** - Operational requirements
4. **Security Enhancements** - Production security posture

### Phase 3: Operational Excellence

1. **Advanced Performance Tuning** - Optimization based on real-world usage
2. **Enhanced Configuration Management** - Operational flexibility
3. **Comprehensive Testing** - Validation under all conditions

## Success Criteria

### Technical Metrics

- **Memory Usage**: Stable memory usage under mainnet load
- **Sync Performance**: Comparable or better sync times than OCaml node
- **Block Processing**: Process blocks within target time windows
- **Network Efficiency**: Maintain healthy peer connections under load

### Operational Metrics

- **Uptime**: Target 99.9% uptime under normal conditions
- **Recovery Time**: Fast recovery from network partitions or restarts
- **Monitoring Coverage**: Comprehensive visibility into node health
- **Alert Response**: Timely detection and notification of issues

### Compatibility Metrics

- **Protocol Compliance**: 100% compatibility with Mina protocol
- **OCaml Interoperability**: Seamless interaction with OCaml nodes
- **Hardfork Support**: Successful handling of protocol upgrades

## Risk Assessment

### High Risk Areas

- **Persistence Implementation**: Complex system with potential for data
  corruption
- **Performance Under Load**: Untested behavior under mainnet scale
- **Protocol Compatibility**: Risk of consensus failures if implementation
  differs

### Mitigation Strategies

- **Extensive Testing**: Comprehensive test suites for all critical
  functionality
- **Gradual Rollout**: Phased deployment starting with testnets
- **Monitoring**: Real-time monitoring to detect issues early
- **Fallback Plans**: Clear procedures for rollback if issues arise

## Timeline Considerations

The mainnet readiness timeline depends on:

- **Development Resources**: Available engineering capacity
- **Testing Requirements**: Time needed for comprehensive validation
- **Network Coordination**: Alignment with broader Mina ecosystem timeline
- **Performance Validation**: Real-world testing and optimization

Priority should be given to persistence implementation as it's foundational for
most other mainnet readiness requirements and represents the largest technical
risk.

## Community and Ecosystem

### Node Operator Support

- **Documentation**: Comprehensive deployment and operational guides
- **Support Channels**: Clear support and communication channels
- **Migration Tools**: Tools to help migrate from OCaml nodes if desired

### Developer Integration

- **API Compatibility**: Maintain compatibility with existing tooling
- **Extension Points**: Allow for ecosystem development and integration
- **Developer Tools**: Provide tools for developers building on Mina

## Conclusion

Mainnet readiness for the Mina Rust node requires careful implementation of
critical features, extensive testing, and operational preparation. The
persistence implementation represents the most significant technical challenge
and should be prioritized accordingly.

Success will be measured not just by technical compliance, but by the node's
ability to operate reliably in production environments and contribute to the
overall health and decentralization of the Mina network.
