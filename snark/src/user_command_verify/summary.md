# User Command Verify State Machine

Coordinates SNARK verification for zkApp transactions.

## Purpose

- Orchestrates SNARK verification requests for user commands
- Manages verification job queue and lifecycle
- Tracks verification status through state transitions
- Coordinates callbacks for verification results

## Key Components

- **Request Queue**: Manages pending verification jobs using `PendingRequests`
- **Status Tracking**: Tracks jobs through Init → Pending → Success/Error →
  Finish states
- **Callback System**: Handles success/error callbacks for decoupled
  communication
- **Verifier Resources**: Maintains references to `TransactionVerifier` and
  `VerifierSRS`

## Interactions

- Receives zkApp transaction verification requests
- Dispatches to effectful actions for actual SNARK verification work
- Manages verification job lifecycle and cleanup
- Executes callbacks to report verification results
- Integrates with transaction pool for validated transactions

## Technical Debt

### Minor Issues

- **Missing Error Callback**: TODO to dispatch error callbacks
  (snark_user_command_verify_reducer.rs:95)
- **Debug Display**: TODO to display hashes instead of full content
  (snark_user_command_verify_state.rs:37)
