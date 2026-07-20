# Block Verify State Machine

Manages block proof verification workflows. Does not perform actual
cryptographic verification - that's handled by services using the ledger crate.

## Purpose

- Orchestrates block proof verification requests
- Tracks verification job lifecycle (Init → Pending → Success/Error)
- Manages verification queue with callbacks
- Coordinates with block verification services

## Interactions

- Receives block verification requests from other components
- Dispatches effectful actions to verification services
- Tracks pending verification jobs
- Executes callbacks when verification completes or fails
