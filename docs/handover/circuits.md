# Circuit Generation and Management

## Overview

OpenMina has ported the circuit logic from the Mina protocol, but with an
important architectural distinction: the implementation only handles witness
production, not constraint generation. This means that while OpenMina can
produce proofs using existing circuits, it cannot generate the circuit
definitions themselves.

For an overview of the proof system implementation in `ledger/src/proofs/`, see
[`ledger/src/proofs/summary.md`](../../ledger/src/proofs/summary.md).

## Architecture

### Proof Generation Implementation and Limitations

The OpenMina codebase includes complete proof generation capabilities with one
key limitation:

**What OpenMina Can Do:**

- **Witness generation**: Full implementation for producing witnesses needed for
  proof generation
- **Proof production**: Complete capability to create proofs using pre-existing
  circuit definitions
- **Circuit logic**: Equivalent to the OCaml implementation for all proof types
- **Proof verification**: Can verify proofs using precomputed verification
  indices

**What OpenMina Cannot Do:**

- **Circuit constraints**: Missing the constraint declarations from the OCaml
  code that define circuit structure
- **Constraint compilation/evaluation**: Missing the functionality to
  compile/evaluate constraint declarations into circuit constraints
- **Verification key generation**: Cannot generate verification keys for new
  circuits

**Practical Implications:**

- Can generate proofs and witnesses for existing circuits
- Cannot create new circuits or modify existing circuit definitions
- Relies on OCaml implementation for all circuit creation and constraint
  processing
- Uses precomputed verification indices from the OCaml implementation

The circuit logic is equivalent to the OCaml implementation except both the
constraint declarations and the constraint compilation/evaluation functionality
are missing - these were not ported due to time constraints during development,
not technical limitations, and could be added for full independence.

### Circuit Generation Process

Since these constraint capabilities are missing, OpenMina requires externally
generated circuit data. The following process describes how circuits are created
and distributed using the original Mina codebase:

1. Using a custom branch in the OpenMina fork of Mina:
   https://github.com/openmina/mina
   - Branch: `utils/dump-extra-circuit-data-devnet301`
   - This branch contains modifications to export circuit data in a format
     consumable by OpenMina
   - **Note**: This branch is very messy and should be cleaned up and integrated
     into mina mainline to ease the process

2. Running the circuit generation process using the branch above
   - Launch the OCaml node which produces circuit cache data in
     `/tmp/coda_cache_dir`
   - The branch dumps the usual circuit data plus extra data specifically
     required by OpenMina
   - The process also dumps blocks for use in tests
   - Integration with mainline Mina would streamline future circuit generation

3. The generated circuit blobs are then:
   - Committed to the dedicated repository:
     https://github.com/openmina/circuit-blobs
   - Released as GitHub releases for versioning and distribution

### Circuit Distribution

OpenMina nodes handle circuits dynamically:

- When a node needs a circuit that isn't locally available, it automatically
  downloads it from the circuit-blobs repository
- Downloaded circuits are cached locally for future use
- This on-demand approach keeps the base installation size minimal while
  ensuring all necessary circuits are available when needed

## Circuit Blob Repository Structure

The https://github.com/openmina/circuit-blobs repository serves as the central
distribution point for all circuit definitions used by OpenMina. The repository:

- Contains pre-generated circuit blobs for all supported proof types
- Uses GitHub releases for versioning
- Provides a stable download source for OpenMina nodes

## Future Considerations

Potential future improvements include:

- Completing the constraint generation implementation in OpenMina for a fully
  self-contained system
- Automating the circuit generation and publishing process
- Implementing circuit versioning strategies for protocol upgrades

## Circuit Loading and Caching

### Circuit Loading Process

1. **Circuit Discovery**: When a circuit is needed, the system searches for it
   in several locations:
   - Environment variable `OPENMINA_CIRCUIT_BLOBS_BASE_DIR` (if set)
   - Current manifest directory (for development)
   - User's home directory: `~/.openmina/circuit-blobs/`
   - System-wide installation: `/usr/local/lib/openmina/circuit-blobs/`

2. **Automatic Download**: If no local circuit is found, the system:
   - Downloads the circuit blob from
     `https://github.com/openmina/circuit-blobs/releases/download/`
   - Caches it to `~/.openmina/circuit-blobs/` for future use
   - Logs the download and caching process

3. **WASM Handling**: For WebAssembly builds, circuits are loaded via HTTP from
   `/assets/webnode/circuit-blobs/` (configurable via
   `CIRCUIT_BLOBS_HTTP_PREFIX`)

### Circuit Components

Each circuit consists of multiple components that are loaded and cached
independently:

- **Gates**: Circuit constraint definitions in JSON format (`*_gates.json`)
- **Internal Variables**: Constraint variable mappings in binary format
  (`*_internal_vars.bin`)
- **Rows Reverse**: Row-wise constraint data in binary format (`*_rows_rev.bin`)
- **Verifier Indices**: Pre-computed verification data with SHA256 integrity
  checks

### Verifier Index Caching

The system implements a two-level caching strategy for verifier indices:

1. **Source Validation**: Each verifier index is validated against a SHA256
   digest of the source JSON
2. **Index Validation**: The processed verifier index is also validated with its
   own SHA256 digest
3. **Cache Storage**: Valid indices are cached with both digests for rapid
   future access

### Circuit Types

The system supports multiple circuit types for different proof operations:

- **Transaction Circuits**: For transaction proof generation and verification
- **Block Circuits**: For block proof generation and verification
- **Merge Circuits**: For combining multiple proofs
- **ZkApp Circuits**: For zero-knowledge application proofs with various
  signature patterns

### Performance Optimization

Circuit loading is optimized through:

- **Lazy Loading**: Circuits are only loaded when actually needed
- **Static Caching**: Once loaded, circuits are cached in static variables for
  the lifetime of the process
- **Concurrent Access**: Multiple threads can safely access the same cached
  circuit data
- **Integrity Verification**: SHA256 checksums ensure data integrity without
  performance penalties

### Network-Specific Circuit Configuration

Circuit loading is controlled by the network configuration system in
`core/src/network.rs`:

- **Directory Selection**: Each network has a specific circuit directory (e.g.,
  `3.0.1devnet`, `3.0.0mainnet`)
- **Circuit Blob Names**: Network-appropriate circuit blob identifiers for each
  proof type
- **Verifier Indices**: Network-specific JSON files embedded in the binary
  (`ledger/src/proofs/data/`)
- **Cache Isolation**: Different networks cache circuits in separate
  subdirectories

The `CircuitsConfig` struct defines all circuit blob names for each network,
ensuring the correct circuit versions are loaded for the target environment.

## Related Documentation

- For debugging block proof generation, see
  [debug-block-proof-generation.md](./debug-block-proof-generation.md)
- For mainnet readiness considerations, see
  [mainnet-readiness.md](./mainnet-readiness.md)
