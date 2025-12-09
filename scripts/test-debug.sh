#!/bin/bash
set -e

if [ "$#" -le 0 ]; then
  echo "test-debug.sh CONTRACT_NAME [OPTIONAL ARGS...]"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

. $SCRIPT_DIR/vars.sh

echo "PACKAGE_PATH = $PACKAGE_PATH"

cd $PACKAGE_PATH && RUST_BACKTRACE=1 cargo test --features test-debug ${@:2} -- --show-output
