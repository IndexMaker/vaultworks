#!/bin/bash

if [ "$#" -le 1 ]; then
  echo "treasury-upgrade-gate.sh PROXY_ADDRESS TRESURY_ADDRESS [UPGRADE_CALLDATA]"
  exit 1
fi

PROXY_ADDRESS=$1
TREASURY_ADDRESS=$2
UPGRADE_CALLDATA=${3:-"0x"}

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

RPC_URL=${RPC_URL:-"http://localhost:8547"}

if [ -z "$DEPLOY_PRIVATE_KEY" ]; then
  die "Missing environment variable: DEPLOY_PRIVATE_KEY"
fi

WALLET_ADDRESS=`cast wallet address $DEPLOY_PRIVATE_KEY`

cast send --rpc-url $RPC_URL --private-key $DEPLOY_PRIVATE_KEY \
    $PROXY_ADDRESS "upgradeToAndCall(address,bytes)" "$TREASURY_ADDRESS" "$UPGRADE_CALLDATA"

