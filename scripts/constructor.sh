#!/bin/bash

if [ "$#" -le 0 ]; then
  echo "constructor.sh CONTRACT_NAME [OPTIONAL ARGS...]"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

. $SCRIPT_DIR/vars.sh

echo "PACKAGE_PATH = $PACKAGE_PATH"
echo "WASM_FILE_PATH = $WASM_FILE_PATH"

cd $PACKAGE_PATH && cargo stylus constructor

