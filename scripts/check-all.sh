#!/bin/bash

set -o pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
. "$SCRIPT_DIR/vars.sh"

SUMMARY=()

for name in $(ls "$WORKSPACE_ROOT/contracts"); do
    CONTRACT_INFO=$(check "$name" | tee /dev/stderr | grep -i "contract size" | sed 's/.*size: //I')
    SUMMARY+=("Contract '$name' size: $CONTRACT_INFO")
done

echo "----------------------"
echo "=== Check Complete ==="
echo "----------------------"
for line in "${SUMMARY[@]}"; do
    echo "$line"
done
echo "----------------------"