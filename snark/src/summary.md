# SNARK State Machine

Orchestrates proof verification workflows for the Mina protocol. This crate
contains state machine logic only - actual cryptographic verification is
performed by the ledger crate.

## Purpose

- Manages proof verification workflows and job queuing
- Tracks verification request lifecycle (Init → Pending → Success/Error)
- Coordinates with verification services
- Handles verification results and callbacks

## Key Components

- **Block Verify**: Manages block proof verification workflows
- **User Command Verify**: Manages transaction and zkApp proof verification
  workflows
- **Work Verify**: Manages SNARK work proof verification workflows

## Technical Details

- Imports actual verifiers from `ledger::proofs::verifiers`
- Effectful actions call verification services via thin service layer
- State machines track pending requests with callbacks for async results
- No cryptographic verification logic - pure workflow orchestration

## Interactions

- Receives verification requests from other components
- Dispatches effectful actions to verification services
- Tracks verification job status and results
- Executes callbacks when verification completes
