#!/bin/bash

die() {
    echo "ERROR: $1" >&2
    exit 1
}

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

if [ -z "$DEPLOY_PRIVATE_KEY" ]; then
  die "Missing environment variable: DEPLOY_PRIVATE_KEY"
fi

WALLET_ADDRESS=`cast wallet address $DEPLOY_PRIVATE_KEY`
INIT_DATA=`cast calldata "initialize(address owner)" $WALLET_ADDRESS`

$SCRIPT_DIR/deploy-construct1.sh treasury "constructor(address)" "$WALLET_ADDRESS"

