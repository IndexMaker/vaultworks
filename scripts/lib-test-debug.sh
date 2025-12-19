#!/bin/bash
set -e

if [ "$#" -le 0 ]; then
  echo "lib-test-debug.sh LIB_NAME [OPTIONAL ARGS...]"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

. $SCRIPT_DIR/vars.sh
    
PACKAGE_NAME=${1:-$(basename "$PWD")}
PACKAGE_PATH="$WORKSPACE_ROOT/libs/$PACKAGE_NAME"

echo "PACKAGE_PATH = $PACKAGE_PATH"

cd $PACKAGE_PATH && RUST_BACKTRACE=1 cargo test --features test-debug ${@:2} -- --show-output
