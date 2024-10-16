#!/bin/bash

OPENMINA_BASE_URL="https://github.com/openmina"

replace_signaling_url() {
    if [ -n "$OPENMINA_SIGNALING_URL" ]; then
        HTTPD_CONF="/usr/local/apache2/conf/httpd.conf"
        SIGNALING_URL="http://localhost:3000/mina/webrtc/signal"

        echo "Replacing signaling URL in $HTTPD_CONF..."

        sed -i "s|$SIGNALING_URL|$OPENMINA_SIGNALING_URL|g" "$HTTPD_CONF"

        if [[ $? -ne 0 ]]; then
            echo "Failed to replace the signaling URL, exiting."
            exit 1
        else
            echo "Successfully replaced signaling URL with '$OPENMINA_SIGNALING_URL' in $HTTPD_CONF"
        fi
    else
        echo "OPENMINA_SIGNALING_URL is not set. No replacement performed."
    fi
}

download_circuit_files() {
    CIRCUITS_BASE_URL="$OPENMINA_BASE_URL/circuit-blobs/releases/download"
    CIRCUITS_VERSION="3.0.1devnet"

    DEVNET_CIRCUIT_FILES=(
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
    DOWNLOAD_DIR="/usr/local/apache2/htdocs/assets/webnode/circuit-blobs/$CIRCUITS_VERSION"

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

download_wasm_files() {
    if [ -z "$OPENMINA_WASM_VERSION" ]; then
        echo "Error: OPENMINA_WASM_VERSION is not set. Exiting."
        exit 1
    fi
    
    WASM_URL="$OPENMINA_BASE_URL/openmina/releases/download/$OPENMINA_WASM_VERSION/openmina-$OPENMINA_WASM_VERSION-webnode-wasm.tar.gz"
    TARGET_DIR="/usr/local/apache2/htdocs/assets/webnode/pkg"
    
    mkdir -p "$TARGET_DIR"

    echo "Downloading WASM files from $WASM_URL..."
    curl -s -L --retry 3 --retry-delay 5 -o "/tmp/openmina-$OPENMINA_WASM_VERSION-webnode-wasm.tar.gz" "$WASM_URL"
    
    if [[ $? -ne 0 ]]; then
        echo "Failed to download the WASM file after 3 attempts, exiting."
        exit 1
    else
        echo "WASM file downloaded successfully. Extracting to $TARGET_DIR..."

        tar -xzf "/tmp/openmina-$OPENMINA_WASM_VERSION-webnode-wasm.tar.gz" -C "$TARGET_DIR"
        
        # Check if the extraction was successful
        if [[ $? -ne 0 ]]; then
            echo "Failed to extract the WASM file, exiting."
            exit 1
        else
            echo "WASM files extracted successfully to $TARGET_DIR"
        fi
    fi

    rm "/tmp/openmina-$OPENMINA_WASM_VERSION-webnode-wasm.tar.gz"
}

if [ -n "$OPENMINA_FRONTEND_ENVIRONMENT" ]; then
  echo "Using environment: $OPENMINA_FRONTEND_ENVIRONMENT"
  cp -f /usr/local/apache2/htdocs/assets/environments/"$OPENMINA_FRONTEND_ENVIRONMENT".js \
        /usr/local/apache2/htdocs/assets/environments/env.js

  if [ "$OPENMINA_FRONTEND_ENVIRONMENT" = "webnode" ]; then
    echo "Environment is 'webnode'. Downloading circuit and WASM files..."
    download_wasm_files
    download_circuit_files
  fi
else
  echo "No environment specified. Using default."
fi

replace_signaling_url

echo "Starting Apache..."
exec "$@"
