#!/bin/bash

if [ "$#" -le 0 ]; then
  echo "deploy.sh CONTRACT_NAME [OPTIONAL ARGS...]"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

. $SCRIPT_DIR/vars.sh

RPC_URL=${RPC_URL:-"http://localhost:8547"}
MAX_FEE_PER_GAS_GWEI=${MAX_FEE_PER_GAS_GWEI:-30}

echo "PACKAGE_PATH = $PACKAGE_PATH"
echo "WASM_FILE_PATH = $WASM_FILE_PATH"
echo "RPC_URL = $RPC_URL"

if [ -z "$DEPLOY_PRIVATE_KEY" ]; then
  die "Missing environment variable: DEPLOY_PRIVATE_KEY"
fi

if [ ! -f "./$WASM_FILE_PATH" ]; then
  $SCRIPT_DIR/check.sh $PACKAGE_NAME
fi

cd $WORKSPACE_ROOT && cargo stylus deploy --wasm-file "./$WASM_FILE_PATH" \
    --endpoint="$RPC_URL" \
    --no-verify \
    --max-fee-per-gas-gwei=$MAX_FEE_PER_GAS_GWEI \
    --private-key="$DEPLOY_PRIVATE_KEY" ${@:2}
