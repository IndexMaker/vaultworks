#!/bin/bash

if [ "$#" -le 0 ]; then
  echo "check.sh CONTRACT_NAME [OPTIONAL ARGS...]"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

. $SCRIPT_DIR/vars.sh

RPC_URL=${RPC_URL:-"http://localhost:8547"}

echo "PACKAGE_PATH = $PACKAGE_PATH"
echo "WASM_FILE_PATH = $WASM_FILE_PATH"
echo "RPC_URL = $RPC_URL"

# First we make stylus build
# Note: it will actually fail at the end, because it doesn't understand cargo workspaces
cd $PACKAGE_PATH && cargo stylus check || true 

echo -en "Please ignore, if you see an error above \"could not read release deps dir\".\n\n"

# Then we run actual check
cd $PACKAGE_PATH && cargo stylus check --endpoint="$RPC_URL" --wasm-file "$WORKSPACE_ROOT/$WASM_FILE_PATH"
