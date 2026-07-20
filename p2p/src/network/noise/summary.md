# Noise State Machine

Implements Noise protocol for encrypted P2P communication using
ChaCha20Poly1305.

## Purpose

- Establishes encrypted channels between peers
- Performs cryptographic handshake with key exchange
- Manages ephemeral and static session keys
- Encrypts and decrypts all P2P communication
- Provides forward secrecy and authentication

## Key Components

- **Handshake Manager**: Handles Noise protocol handshake state machine
- **Key Manager**: Manages static and ephemeral cryptographic keys
- **Transport Cipher**: Encrypts/decrypts messages after handshake
- **Message Parser**: Parses Noise protocol messages securely

## Interactions

- Initiates and responds to cryptographic handshakes
- Exchanges ephemeral keys for forward secrecy
- Authenticates peers using static keys
- Establishes secure transport layer for all P2P communication
- Manages key rotation and session lifecycle

## Technical Debt

### Security Issues

- **Session Key Cleanup**: Ephemeral session keys in `DataSized<32>` not
  securely zeroized after use
  - **Risk**: Limited to active session compromise via memory forensics
  - **Context**: These are per-connection transport keys, not long-term identity
    keys
  - **Impact**: Provides forward secrecy, but current session could be
    compromised if memory is accessed
  - **Solution**: Implement `Zeroize` trait for `DataSized<N>` or create secure
    key wrapper
- **Information Leakage**: Debug output in error paths could leak timing
  information or internal state

### Other Issues

- **Deprecated Crypto**: Usage of deprecated `Scalar::from_bits` function needs
  migration to current curve25519-dalek API
- **Code Clarity**: "Obscure arithmetics" TODO refers to standard Noise protocol
  parsing that could benefit from better documentation

### Additional Issues

- **Missing Error Reporting**: TODO for error reporting in edge cases

### Implementation Notes

The 494-line reducer and nested state machine structure implement the full Noise
XX handshake protocol. The crypto protocol complexity is inherent to the Noise
specification. See
[p2p_network_noise_refactoring.md](./p2p_network_noise_refactoring.md) for
detailed analysis.
