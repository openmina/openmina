# Testing Scenarios for Openmina Node

## Bootstrapping

### Bootstrapping with recorded data

- [ ] Genesis block
- [ ] First epoch, < 290 blocks
- [ ] First epoch, > 290 blocks
- [ ] Third epoch and further

### Bootstrapping with real peers

- [ ] Network split
- [ ] Long-running network (cluster)
- [ ] Berkeley testnet (?)

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


