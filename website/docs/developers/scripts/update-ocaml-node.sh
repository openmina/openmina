#!/bin/bash

set -eou pipefail

# Checkout to the root directory of the project
# We suppose this script is run from a copy of the git repository.
base_dir="$(git rev-parse --show-toplevel)"

if [ "$#" -ne 4 ]; then
    echo "Error: Missing arguments."
    echo "Usage: $0 <old_hash> <new_hash> <old_version> <new_version>"
    exit 1
fi

old_hash=$1
new_hash=$2
old_version=$3
new_version=$4

length_old_hash=${#old_hash}
length_new_hash=${#new_hash}

if [ "$length_old_hash" -ne 8 ] || [ "$length_new_hash" -ne 8 ]; then
    echo "Error: Hashes must be exactly 8 characters long."
    echo "Old hash: $old_hash, New hash: $new_hash"
    echo "The correct hashes can be found on the GitHub release page of MinaProtocol/Mina."
    exit 1
fi

# The docker images are named with only 7 characters of the hash.
shorter_old_hash=${old_hash:0:7}
shorter_new_hash=${new_hash:0:7}

echo "Updating config_${old_hash} to config_${new_hash}"

# Check if config_${old_hash} pattern exists in the files
config_files=(
    "${base_dir}/tools/testing/src/node/ocaml/config.rs"
    "${base_dir}/tools/testing/src/node/ocaml/mod.rs"
    "${base_dir}/tools/testing/src/scenarios/multi_node/basic_connectivity_peer_discovery.rs"
    "${base_dir}/tools/testing/src/scenarios/solo_node/basic_connectivity_accept_incoming.rs"
)

config_pattern_found=false
for file in "${config_files[@]}"; do
    if grep -q "config_${old_hash}" "$file"; then
        config_pattern_found=true
        break
    fi
done

if [ "$config_pattern_found" = false ]; then
    echo "Error: No reference to 'config_${old_hash}' found in any of the expected files."
    echo "Expected files:"
    printf '  %s\n' "${config_files[@]}"
    exit 1
fi

sed -i'' -e "s/config_${old_hash}/config_${new_hash}/g" "${config_files[@]}"

# Check if version-hash pattern exists in the files
version_files=(
    "${base_dir}/.github/workflows/tests.yaml"
    "${base_dir}/.github/workflows/test-graphql-compatibility.yml"
    "${base_dir}/docker-compose.archive.devnet.compare.yml"
    "${base_dir}/tools/testing/src/node/ocaml/config.rs"
)

version_pattern_found=false
for file in "${version_files[@]}"; do
    if grep -q "${old_version}-${shorter_old_hash}" "$file"; then
        version_pattern_found=true
        break
    fi
done

if [ "$version_pattern_found" = false ]; then
    echo "Error: No reference to '${old_version}-${shorter_old_hash}' found in any of the expected files."
    echo "Expected files:"
    printf '  %s\n' "${version_files[@]}"
    exit 1
fi

sed -i'' -e "s/${old_version}-${shorter_old_hash}/${new_version}-${shorter_new_hash}/g" "${version_files[@]}"
