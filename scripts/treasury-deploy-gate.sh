#!/bin/bash

if [ "$#" -le 0 ]; then
  echo "treasury-deploy-gate.sh TREASURY_ADDRESS"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

TREASURY_ADDRESS=$1

if [ -z "$DEPLOY_PRIVATE_KEY" ]; then
  die "Missing environment variable: DEPLOY_PRIVATE_KEY"
fi

WALLET_ADDRESS=`cast wallet address $DEPLOY_PRIVATE_KEY`
INIT_DATA=`cast calldata "initialize(address owner)" $WALLET_ADDRESS`

$SCRIPT_DIR/deploy-construct2.sh gate "constructor(address,bytes)" "$TREASURY_ADDRESS" "$INIT_DATA"

