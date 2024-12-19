#!/bin/bash

# Set the base URL for OpenMina
OPENMINA_BASE_URL="https://github.com/openmina"

# Function to download circuit files
download_circuit_files() {
    CIRCUITS_BASE_URL="$OPENMINA_BASE_URL/circuit-blobs/releases/download"
    CIRCUITS_VERSION="3.0.1devnet"

    DEVNET_CIRCUIT_FILES=(
        "block_verifier_index.postcard"
        "transaction_verifier_index.postcard"
        "step-step-proving-key-blockchain-snark-step-0-55f640777b6486a6fd3fdbc3fcffcc60_gates.json"
        "step-step-proving-key-blockchain-snark-step-0-55f640777b6486a6fd3fdbc3fcffcc60_internal_vars.bin"
        "step-step-proving-key-blockchain-snark-step-0-55f640777b6486a6fd3fdbc3fcffcc60_rows_rev.bin"
        "step-step-proving-key-transaction-snark-merge-1-ba1d52dfdc2dd4d2e61f6c66ff2a5b2f_gates.json"
        "step-step-proving-key-transaction-snark-merge-1-ba1d52dfdc2dd4d2e61f6c66ff2a5b2f_internal_vars.bin"
        "step-step-proving-key-transaction-snark-merge-1-ba1d52dfdc2dd4d2e61f6c66ff2a5b2f_rows_rev.bin"
        "step-step-proving-key-transaction-snark-opt_signed-3-9eefed16953d2bfa78a257adece02d47_gates.json"
        "step-step-proving-key-transaction-snark-opt_signed-3-9eefed16953d2bfa78a257adece02d47_internal_vars.bin"
        "step-step-proving-key-transaction-snark-opt_signed-3-9eefed16953d2bfa78a257adece02d47_rows_rev.bin"
        "step-step-proving-key-transaction-snark-opt_signed-opt_signed-2-48925e6a97197028e1a7c1ecec09021d_gates.json"
        "step-step-proving-key-transaction-snark-opt_signed-opt_signed-2-48925e6a97197028e1a7c1ecec09021d_internal_vars.bin"
        "step-step-proving-key-transaction-snark-opt_signed-opt_signed-2-48925e6a97197028e1a7c1ecec09021d_rows_rev.bin"
        "step-step-proving-key-transaction-snark-proved-4-0cafcbc6dffccddbc82f8c2519c16341_gates.json"
        "step-step-proving-key-transaction-snark-proved-4-0cafcbc6dffccddbc82f8c2519c16341_internal_vars.bin"
        "step-step-proving-key-transaction-snark-proved-4-0cafcbc6dffccddbc82f8c2519c16341_rows_rev.bin"
        "step-step-proving-key-transaction-snark-transaction-0-c33ec5211c07928c87e850a63c6a2079_gates.json"
        "step-step-proving-key-transaction-snark-transaction-0-c33ec5211c07928c87e850a63c6a2079_internal_vars.bin"
        "step-step-proving-key-transaction-snark-transaction-0-c33ec5211c07928c87e850a63c6a2079_rows_rev.bin"
        "wrap-wrap-proving-key-blockchain-snark-bbecaf158ca543ec8ac9e7144400e669_gates.json"
        "wrap-wrap-proving-key-blockchain-snark-bbecaf158ca543ec8ac9e7144400e669_internal_vars.bin"
        "wrap-wrap-proving-key-blockchain-snark-bbecaf158ca543ec8ac9e7144400e669_rows_rev.bin"
        "wrap-wrap-proving-key-transaction-snark-b9a01295c8cc9bda6d12142a581cd305_gates.json"
        "wrap-wrap-proving-key-transaction-snark-b9a01295c8cc9bda6d12142a581cd305_internal_vars.bin"
        "wrap-wrap-proving-key-transaction-snark-b9a01295c8cc9bda6d12142a581cd305_rows_rev.bin"
    )
    DOWNLOAD_DIR="../src/assets/webnode/circuit-blobs/$CIRCUITS_VERSION"

    mkdir -p "$DOWNLOAD_DIR"

    for FILE in "${DEVNET_CIRCUIT_FILES[@]}"; do
        if [[ -f "$DOWNLOAD_DIR/$FILE" ]]; then
            echo "$FILE already exists in $DOWNLOAD_DIR, skipping download."
        else
            echo "Downloading $FILE to $DOWNLOAD_DIR..."
            curl -s -L --retry 3 --retry-delay 5 -o "$DOWNLOAD_DIR/$FILE" "$CIRCUITS_BASE_URL/$CIRCUITS_VERSION/$FILE"
            if [[ $? -ne 0 ]]; then
                echo "Failed to download $FILE after 3 attempts, exiting."
                exit 1
            else
                echo "$FILE downloaded successfully to $DOWNLOAD_DIR"
            fi
        fi
    done
}

# Call the function to download circuit files
download_circuit_files
