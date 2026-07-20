# Private Network State Machine

Implements private network support with pre-shared key authentication using
XSalsa20 encryption.

## Purpose

- Restricts network access to authorized peers only
- Validates pre-shared keys during connection setup
- Creates isolated private networks using chain ID derivation
- Encrypts all communication with XSalsa20 stream cipher
- Filters and drops unauthorized connection attempts

## Key Components

- **Key Manager**: Uses pre-computed PSK (derived from chain ID elsewhere in the
  system)
- **Nonce Handler**: Manages 24-byte nonce exchange for cipher setup
- **Buffer Manager**: Handles nonce data buffering during handshake
- **Cipher Manager**: Establishes XSalsa20 encryption after authentication

## Interactions

- Uses shared secrets derived from blockchain chain ID (derivation happens
  outside this component)
- Exchanges nonces during connection establishment
- Validates PSK authentication on handshake
- Establishes encrypted communication channel
- Drops unauthorized connections that fail PSK validation

## Technical Debt

This component has implementation issues. See
[p2p_network_pnet_refactoring.md](./p2p_network_pnet_refactoring.md) for details
on:

- **State Machine Organization**: Mixed buffering and encryption concerns in
  single state machine
- **Code Structure**: Complex reducer logic that needs helper methods and better
  organization
- **Buffer Management**: Complex arithmetic and array indexing that needs
  simplification
- **Constants**: Hard-coded values that should be extracted as named constants
- **Architecture Alignment**: Opportunity to move more logic to state methods
  per project guidelines

These improvements are needed for better code readability, testability, and
maintainability while preserving correct protocol behavior.
