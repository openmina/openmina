# Work Verify State Machine

Manages SNARK work proof verification workflows. Does not perform actual
cryptographic verification - that's handled by services using the ledger crate.

## Purpose

- Orchestrates SNARK work verification requests
- Tracks verification job lifecycle (Init → Pending → Success/Error)
- Manages verification queue with batch processing
- Coordinates with work verification services

## Interactions

- Receives work verification requests from SNARK pool
- Dispatches effectful actions to verification services
- Tracks pending verification jobs with batch support
- Executes callbacks when verification completes or fails
