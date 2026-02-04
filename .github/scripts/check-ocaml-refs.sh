#!/usr/bin/env bash
# Script to validate OCaml reference comments in Rust code
# Usage: ./.github/scripts/check-ocaml-refs.sh [--repo REPO_URL] [--branch BRANCH] [--update]
#
# Supports hyperlink format: /// OCaml: <https://github.com/MinaProtocol/mina/blob/COMMIT/path#L1-L10>

set -euo pipefail

# Default configuration
OCAML_REPO="${OCAML_REPO:-https://github.com/MinaProtocol/mina.git}"
OCAML_BRANCH="${OCAML_BRANCH:-compatible}"
UPDATE_MODE="${UPDATE_MODE:-false}"
RUST_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --repo)
            OCAML_REPO="$2"
            shift 2
            ;;
        --branch)
            OCAML_BRANCH="$2"
            shift 2
            ;;
        --update)
            UPDATE_MODE="true"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: ./.github/scripts/check-ocaml-refs.sh [--repo REPO_URL] [--branch BRANCH] [--update]"
            exit 1
            ;;
    esac
done

echo "Checking OCaml references against ${OCAML_REPO} (branch: ${OCAML_BRANCH})"

# Create temporary directory
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

# Extract GitHub owner and repo from URL (e.g., https://github.com/MinaProtocol/mina.git)
GITHUB_URL_PATTERN="https://github.com/([^/]+)/(.+)"
if [[ "$OCAML_REPO" =~ $GITHUB_URL_PATTERN ]]; then
    GITHUB_OWNER="${BASH_REMATCH[1]}"
    GITHUB_REPO="${BASH_REMATCH[2]%.git}"  # Remove .git suffix if present
else
    echo "Error: Repository URL must be a GitHub URL"
    exit 1
fi

# Get current commit hash for the branch using GitHub API
echo "Fetching current commit from ${OCAML_BRANCH}..."
CURRENT_COMMIT=$(curl -s "https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPO}/commits/${OCAML_BRANCH}" | grep -o '"sha": "[^"]*"' | head -1 | cut -d'"' -f4)

if [ -z "$CURRENT_COMMIT" ]; then
    echo "Error: Could not fetch current commit for branch ${OCAML_BRANCH}"
    exit 1
fi

echo "Current OCaml commit: ${CURRENT_COMMIT}"

# Find all Rust files with OCaml references
cd "${RUST_ROOT}"
RUST_FILES=$(git grep -l -E "^/// OCaml: <https://github.com/MinaProtocol/mina/blob/" "*.rs" "**/*.rs" 2>/dev/null || true)

if [ -z "$RUST_FILES" ]; then
    echo "No OCaml references found in Rust code"
    exit 0
fi

# Use temporary files to accumulate results
RESULTS_FILE="${TEMP_DIR}/results.txt"
touch "$RESULTS_FILE"

echo ""
echo "Validating references..."
echo "========================"

# Process each file
echo "$RUST_FILES" | while IFS= read -r rust_file; do
    # Process hyperlink format: /// OCaml: <URL>
    grep -n "^/// OCaml: <https://github.com/MinaProtocol/mina/blob/" "$rust_file" 2>/dev/null | while IFS=: read -r line_num line_content; do
        # Extract URL from angle brackets
        URL=$(echo "$line_content" | sed -n 's/.*<\(https:\/\/github\.com\/MinaProtocol\/mina\/blob\/[^>]*\)>.*/\1/p')

        if [ -z "$URL" ]; then
            echo "INVALID|${rust_file}|LINE:${line_num}|MALFORMED_URL" >> "$RESULTS_FILE"
            echo "[ERROR] INVALID: ${rust_file}:${line_num}"
            echo "   Malformed OCaml reference URL"
            continue
        fi

        # Parse URL: https://github.com/MinaProtocol/mina/blob/COMMIT/path#L1-L10
        # Pattern: blob/COMMIT/PATH with optional #L1-L10
        if [[ "$URL" =~ blob/([a-f0-9]+)/([^#]+)(#L([0-9]+)(-L([0-9]+))?)? ]]; then
            COMMIT="${BASH_REMATCH[1]}"
            OCAML_PATH="${BASH_REMATCH[2]}"
            START_LINE="${BASH_REMATCH[4]}"
            END_LINE="${BASH_REMATCH[6]}"

            # If only start line is specified, set end line to same
            if [ -n "$START_LINE" ] && [ -z "$END_LINE" ]; then
                END_LINE="$START_LINE"
            fi

            LINE_RANGE=""
            if [ -n "$START_LINE" ]; then
                LINE_RANGE="${START_LINE}-${END_LINE}"
            fi
        else
            echo "INVALID|${rust_file}|LINE:${line_num}|INVALID_URL_FORMAT" >> "$RESULTS_FILE"
            echo "[ERROR] INVALID: ${rust_file}:${line_num}"
            echo "   URL does not match expected format: $URL"
            continue
        fi

        # Fetch the OCaml file from the current branch
        CURRENT_FILE="${TEMP_DIR}/current_${rust_file//\//_}_${OCAML_PATH//\//_}"
        CURRENT_URL="https://raw.githubusercontent.com/${GITHUB_OWNER}/${GITHUB_REPO}/${OCAML_BRANCH}/${OCAML_PATH}"

        if ! curl -sf "$CURRENT_URL" -o "$CURRENT_FILE"; then
            echo "INVALID|${rust_file}|${OCAML_PATH}|FILE_NOT_FOUND" >> "$RESULTS_FILE"
            echo "[ERROR] INVALID: ${rust_file}:${line_num}"
            echo "   OCaml file not found: ${OCAML_PATH}"
        else
            # Validate line range if specified
            RANGE_VALID=true
            if [ -n "$LINE_RANGE" ]; then
                FILE_LINES=$(wc -l < "$CURRENT_FILE")

                if [ "$END_LINE" -gt "$FILE_LINES" ]; then
                    echo "INVALID|${rust_file}|${OCAML_PATH}|LINE_RANGE_EXCEEDED|L:${LINE_RANGE}|${FILE_LINES}" >> "$RESULTS_FILE"
                    echo "[ERROR] INVALID: ${rust_file}:${line_num}"
                    echo "   Line range L:${LINE_RANGE} exceeds file length (${FILE_LINES} lines): ${OCAML_PATH}"
                    RANGE_VALID=false
                fi
            fi

            if [ "$RANGE_VALID" = "true" ]; then
                # Verify that the code at the referenced commit matches the current branch
                CODE_MATCHES=true
                if [ -n "$LINE_RANGE" ]; then
                    # Fetch the file from the referenced commit
                    COMMIT_FILE="${TEMP_DIR}/commit_${rust_file//\//_}_${OCAML_PATH//\//_}"
                    COMMIT_URL="https://raw.githubusercontent.com/${GITHUB_OWNER}/${GITHUB_REPO}/${COMMIT}/${OCAML_PATH}"

                    if ! curl -sf "$COMMIT_URL" -o "$COMMIT_FILE"; then
                        echo "INVALID|${rust_file}|${OCAML_PATH}|COMMIT_NOT_FOUND|${COMMIT}" >> "$RESULTS_FILE"
                        echo "[ERROR] INVALID: ${rust_file}:${line_num}"
                        echo "   Referenced commit does not exist: ${COMMIT}"
                        CODE_MATCHES=false
                    else
                        # Extract the specific line ranges from both files and compare
                        CURRENT_LINES=$(sed -n "${START_LINE},${END_LINE}p" "$CURRENT_FILE")
                        COMMIT_LINES=$(sed -n "${START_LINE},${END_LINE}p" "$COMMIT_FILE")

                        if [ "$CURRENT_LINES" != "$COMMIT_LINES" ]; then
                            echo "INVALID|${rust_file}|${OCAML_PATH}|CODE_MISMATCH|${COMMIT}" >> "$RESULTS_FILE"
                            echo "[ERROR] INVALID: ${rust_file}:${line_num}"
                            echo "   Code at L:${LINE_RANGE} differs between commit ${COMMIT} and current branch"
                            echo "   Referenced: https://github.com/${GITHUB_OWNER}/${GITHUB_REPO}/blob/${COMMIT}/${OCAML_PATH}#L${START_LINE}-L${END_LINE}"
                            echo "   Current:    https://github.com/${GITHUB_OWNER}/${GITHUB_REPO}/blob/${OCAML_BRANCH}/${OCAML_PATH}#L${START_LINE}-L${END_LINE}"
                            CODE_MATCHES=false
                        fi
                    fi
                fi

                if [ "$CODE_MATCHES" = "true" ]; then
                    # Check if commit is stale
                    if [ "$COMMIT" != "$CURRENT_COMMIT" ]; then
                        echo "STALE|${rust_file}|${line_num}|${OCAML_PATH}|${COMMIT}|${LINE_RANGE}" >> "$RESULTS_FILE"
                        echo "[OK] VALID: ${rust_file}:${line_num} -> ${OCAML_PATH} L:${LINE_RANGE}"
                        echo "  [WARN] STALE COMMIT: ${COMMIT} (current: ${CURRENT_COMMIT})"
                    else
                        echo "VALID|${rust_file}|${line_num}|${OCAML_PATH}|${LINE_RANGE}" >> "$RESULTS_FILE"
                        echo "[OK] VALID: ${rust_file}:${line_num} -> ${OCAML_PATH} L:${LINE_RANGE}"
                    fi
                fi
            fi
        fi
    done
done

# Count results
TOTAL_REFS=$(wc -l < "$RESULTS_FILE")
VALID_REFS=$(grep -c "^VALID|" "$RESULTS_FILE" || true)
INVALID_REFS=$(grep -c "^INVALID|" "$RESULTS_FILE" || true)
STALE_COMMITS=$(grep -c "^STALE|" "$RESULTS_FILE" || true)

echo ""
echo "Summary"
echo "======="
echo "Total references found: ${TOTAL_REFS}"
echo "Valid references: $((VALID_REFS + STALE_COMMITS))"
echo "Invalid references: ${INVALID_REFS}"
echo "Stale commits: ${STALE_COMMITS}"

if [ "$UPDATE_MODE" = "true" ] && [ "${STALE_COMMITS}" -gt 0 ]; then
    echo ""
    echo "Updating stale commit hashes..."

    # Update hyperlink format
    grep "^STALE|" "$RESULTS_FILE" | while IFS='|' read -r _status rust_file line_num ocaml_path old_commit line_range; do
        echo "Updating ${rust_file}:${line_num}..."

        # Build new URL
        NEW_URL="https://github.com/${GITHUB_OWNER}/${GITHUB_REPO}/blob/${CURRENT_COMMIT}/${ocaml_path}"
        if [ -n "$line_range" ] && [ "$line_range" != "" ]; then
            START_LINE=$(echo "$line_range" | cut -d'-' -f1)
            END_LINE=$(echo "$line_range" | cut -d'-' -f2)
            NEW_URL="${NEW_URL}#L${START_LINE}-L${END_LINE}"
        fi

        # Use sed to replace the URL at the specific line
        # We need to escape special characters in the URL for sed
        OLD_COMMIT_ESCAPED=$(echo "$old_commit" | sed 's/[\/&]/\\&/g')
        CURRENT_COMMIT_ESCAPED=$(echo "$CURRENT_COMMIT" | sed 's/[\/&]/\\&/g')

        sed -i "${line_num}s/blob\/${OLD_COMMIT_ESCAPED}\//blob\/${CURRENT_COMMIT_ESCAPED}\//" "${RUST_ROOT}/${rust_file}"
    done

    echo "Updated ${STALE_COMMITS} reference(s)"
fi

# Exit with error if there are invalid references
if [ "${INVALID_REFS}" -gt 0 ]; then
    echo ""
    echo "[ERROR] Validation failed: ${INVALID_REFS} invalid reference(s) found"
    exit 1
fi

if [ "${STALE_COMMITS}" -gt 0 ] && [ "$UPDATE_MODE" = "false" ]; then
    echo ""
    echo "[WARN] Warning: ${STALE_COMMITS} reference(s) have stale commits"
    echo "Run with --update to update them automatically"
    exit 0
fi

echo ""
echo "[OK] All OCaml references are valid!"
