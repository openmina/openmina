# Testing Scenarios for Openmina Node

## Peer discovery

We should test that OCamn and Rust nodes can discover each other in the network.

- [x] Simplest scenario, with 1 seed and two non-seed nodes, one OCaml and another one Rust
- [ ] Bigger network, where we can check that we have some number of peers, and also receive incoming connections

See [discovery.md](docs/testing/discovery.md).

## Bootstrapping

See [bootstrap.md](docs/testing/bootstrap.md).

### Bootstrapping with recorded data

- [ ] Genesis block
- [ ] First epoch, < 290 blocks
- [ ] First epoch, > 290 blocks
- [ ] Third epoch and further

### Bootstrapping with real peers

- [ ] Network split
- [ ] Long-running network (cluster)
- [x] Berkeley testnet: [![Openmina Daily](https://github.com/openmina/openmina/actions/workflows/daily.yaml/badge.svg)](https://github.com/openmina/openmina/actions/workflows/daily.yaml)

### Various bootstrap scenarios

_TODO_

## General Network Behaviour

This might be a set of short and very long-running task, during that we make sure that our node
- can run in a (real) network for a long time, without being disconected too much
- can bootstrap other nodes (both OCaml and Rust)
- can relay p2p data
- can handle forks
- ...

## Snark Work

### Network

- [ ] Check that snark pool is proadcasted properly
- [ ] Check that commitments are broadcasted properly

### Correctness

- [ ] Check proof generated for several transactions

### Coordination

Here we can test snark work is parallelized well, i.e.

$t_{total} < \frac {\sum {t_i} + C} {n}$

- [ ] Test that snark work is parallelized well

### Throughput

- [ ] Run a tx load against network and make sure tx pool doesn't grow


