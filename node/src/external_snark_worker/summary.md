# External SNARK Worker State Machine

Manages a single external process that computes SNARK proofs for the node.

## Purpose

- Manages lifecycle of one external SNARK worker process
- Converts available SNARK jobs to worker specifications
- Handles work submission, cancellation, and timeout management
- Integrates worker results back into the SNARK pool

## Worker State Machine

- **None/Starting**: Initial states for worker startup with 120s timeout
- **Idle**: Ready to accept new work assignments
- **Working**: Processing a specific job with estimated duration timeout
- **WorkReady/WorkError**: Completed states awaiting result processing
- **Cancelling/Cancelled**: Work cancellation states
- **Killing/Error**: Shutdown and error states

## Key Operations

- **Work Specification**: Converts AvailableJobMessage to Mina snark worker
  format
- **Base Jobs**: Transaction proofs with witness data and protocol state
- **Merge Jobs**: Combines two existing proofs into a single proof
- **Timeout Management**: Handles worker startup and work timeouts
- **Result Integration**: Adds completed SNARKs directly to snark pool

## Interactions

- **SNARK Pool**: Receives job assignments and submits completed work
- **Transition Frontier**: Provides protocol state data for work specifications
- **Config**: Uses snarker public key and fee configuration
- **P2P**: Integrates with network for SNARK propagation

## Technical Notes

- Single worker design with potential for future expansion
- Work cancellation supported but may not be immediately effective
- Completed SNARKs added directly to pool as trusted local work
- Protocol state lookup required for base transaction jobs
