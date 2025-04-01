# SNARK System

The SNARK (Succinct Non-interactive ARgument of Knowledge) system is a core component of the OpenMina node that handles zero-knowledge proof verification. It is essential for the Mina protocol, which relies heavily on zero-knowledge proofs to maintain a small blockchain size.

## Component Diagram

```mermaid
graph TD
    subgraph SNARK["SNARK System"]
        BlockVerifier["Block Verifier"] <--> TransactionVerifier["Transaction Verifier"]
        BlockVerifier <--> WorkVerifier["Work Verifier"]
        TransactionVerifier <--> Services["Services"]
        WorkVerifier <--> Services
    end

    %% Legend
    L1["System"] --- L2["Block Proof"] --- L3["Transaction Proof"] --- L4["Work Proof"]

    classDef snarkClass stroke:#2374ab,stroke-width:2px,fill:none;
    classDef blockClass stroke:#4a7c59,stroke-width:2px,fill:none;
    classDef txClass stroke:#8cb369,stroke-width:2px,fill:none;
    classDef workClass stroke:#f4e285,stroke-width:2px,fill:none;
    classDef serviceClass stroke:#f4a259,stroke-width:2px,fill:none;
    classDef legend stroke-dasharray: 5 5, fill:none;

    class SNARK snarkClass;
    class BlockVerifier blockClass;
    class TransactionVerifier txClass;
    class WorkVerifier workClass;
    class Services serviceClass;
    class L1,L2,L3,L4 legend;
```

**Diagram Legend:**

-   **SNARK System**: Core component handling zero-knowledge proof verification
-   **Block Verifier**: Verifies block proofs and validates state transitions
-   **Transaction Verifier**: Verifies transaction proofs
-   **Work Verifier**: Verifies SNARK work proofs
-   **Services**: Handles computationally heavy verification operations

## Subcomponents

### Block Verifier

The Block Verifier subcomponent is responsible for verifying block proofs. It handles:

-   Verifying the SNARK proof for a block
-   Validating the block's state transition

**Key Code Files:**

-   [snark/src/block_verify.rs](../../../snark/src/block_verify.rs)
-   [snark/src/block_verify_effectful.rs](../../../snark/src/block_verify_effectful.rs)

### Transaction Verifier

The Transaction Verifier subcomponent is responsible for verifying transaction proofs. It handles:

-   Verifying the SNARK proof for a transaction
-   Validating the transaction's state transition

**Key Code Files:**

-   [snark/src/user_command_verify.rs](../../../snark/src/user_command_verify.rs)
-   [snark/src/user_command_verify_effectful.rs](../../../snark/src/user_command_verify_effectful.rs)

### Work Verifier

The Work Verifier subcomponent is responsible for verifying SNARK work. It handles:

-   Verifying the SNARK proof for a work item
-   Validating the work's correctness

**Key Code Files:**

-   [snark/src/work_verify.rs](../../../snark/src/work_verify.rs)
-   [snark/src/work_verify_effectful.rs](../../../snark/src/work_verify_effectful.rs)

### Services

The Services subcomponent provides an abstraction for computationally heavy SNARK operations. It handles:

-   Offloading SNARK verification to separate threads
-   Managing the verification process

## State

The SNARK system state is defined in [snark/src/snark_state.rs](../../../snark/src/snark_state.rs):

```rust
pub struct SnarkState {
    pub config: SnarkConfig,
    pub block_verify: BlockVerifyState,
    pub user_command_verify: UserCommandVerifyState,
    pub work_verify: WorkVerifyState,
    // ...
}
```

## Actions

The SNARK system defines several actions for interacting with the state:

```rust
pub enum SnarkAction {
    BlockVerify(BlockVerifyAction),
    UserCommandVerify(UserCommandVerifyAction),
    WorkVerify(WorkVerifyAction),
    // ...
}
```

## Interactions with Other Components

The SNARK system interacts with several other components:

-   **Transition Frontier**: For verifying block and transaction proofs
-   **Services**: For offloading computationally heavy operations

For more details on these interactions, see [Block Processing Flow](../../architecture/block-processing.md).

## Technical Background

SNARK proofs in Mina use the Kimchi proof system, which is based on the PLONK proof system. The verification process involves:

1. Loading the verifier index and SRS (Structured Reference String)
2. Deserializing the proof
3. Verifying the proof against the verifier index
4. Checking that the proof's public input matches the expected value

For more details on SNARK work in Mina, see [SNARK Work](../../../docs/snark-work.md).
