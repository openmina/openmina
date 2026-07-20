# Protocol Select State Machine

Implements multistream-select protocol for P2P protocol negotiation and
selection.

## Purpose

- Negotiates protocols with peers using multistream-select protocol
- Handles protocol compatibility checking and selection
- Manages initiator vs responder negotiation flows
- Provides protocol selection for establishing P2P streams
- Handles simultaneous connection scenarios

## Key Components

- **Token Parser**: Parses protocol negotiation tokens from byte streams
- **Protocol Registry**: Manages supported protocol definitions (hardcoded)
- **Negotiation State Machine**: Handles initiator/responder negotiation flows
- **Selection Logic**: Determines protocol compatibility and selection

## Interactions

- Exchanges protocol lists with connected peers
- Negotiates protocol selection through multistream-select handshake
- Handles protocol version compatibility checking
- Manages negotiation timeouts and error scenarios
- Resolves simultaneous connection attempts

## Technical Debt

This component has moderate maintainability and extensibility issues:

- **Hardcoded Protocol Registry**: `Token::ALL` array must be manually updated
  for new protocols, limiting extensibility
- **TODO Comments**: Incomplete implementations for alternative protocol
  proposals and simultaneous connection handling
- **Complex State Machine**: Mixed error and negotiation states making
  transitions unclear
- **String-based Errors**: Simple string errors instead of structured error
  types limit debugging and recovery
- **Buffer Management**: Raw buffer manipulation in parsing logic is error-prone
- **Limited Protocol Flexibility**: No systematic approach to protocol
  versioning or fallback mechanisms
- **Performance Issues**: Linear search through protocol list and frequent
  memory allocations

These issues make adding new protocols cumbersome and limit the robustness of
protocol negotiation.
