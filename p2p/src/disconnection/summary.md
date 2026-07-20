# Disconnection State Machine

Manages peer disconnection, cleanup, and automated peer space management.

## Purpose

- Handles graceful peer disconnections with comprehensive reason tracking
- Implements automated peer space management when connection limits are exceeded
- Manages cleanup for both libp2p and WebRTC transport layers
- Coordinates system-wide disconnection notifications via callbacks

## Key Components

- **Automated Space Management**: Randomly selects and disconnects peers when
  exceeding `max_stable_peers`
- **Stability Protection**: Prevents disconnection of peers connected for less
  than 90 seconds
- **Reason Categorization**: Tracks disconnection causes (timeouts, protocol
  violations, space management, etc.)
- **Dual Transport Handling**: Separate cleanup logic for libp2p vs WebRTC
  connections
- **Memory Management**: Removes oldest disconnected peer entries to prevent
  unbounded growth

## State Flow

1. **RandomTry**: Periodic check for peer space management (every 10 seconds)
2. **Init**: Begin disconnection with specific reason and peer identification
3. **PeerClosed**: Handle peer-initiated disconnections
4. **FailedCleanup**: Recovery for failed disconnection attempts
5. **Finish**: Complete disconnection with cleanup and system notifications

## Interactions

- Processes disconnect events from various P2P components (channels, network
  protocols, etc.)
- Cleans up protocol states and connection resources
- Notifies dependent systems through callback mechanism
- Updates peer registry and connection status
- Integrates with transport services for actual I/O operations

## Important Notes

- Does **not** trigger reconnections - only handles disconnection and cleanup
- Critical for preventing memory leaks and maintaining connection limits
- Used extensively across the P2P layer (13+ components depend on it)
