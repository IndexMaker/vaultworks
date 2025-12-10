#!/bin/bash

if [ "$#" -le 2 ]; then
  echo "deploy-constructor.sh CONTRACT_NAME CONSTRUCTOR_SIGNATURE CONSTRUCTOR_ARGS [OPTIONAL ARGS...]"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

. $SCRIPT_DIR/vars.sh

RPC_URL=${RPC_URL:-"http://localhost:8547"}

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
    --private-key="$DEPLOY_PRIVATE_KEY" \
    --no-verify \
    --constructor-signature="$2" \
    --constructor-args="$3" \
    --constructor-args="$4" \
    ${@:5}
