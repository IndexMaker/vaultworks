#!/bin/bash

if [ "$#" -le 0 ]; then
  echo "upgrade-treasury.sh PROXY_ADDRESS [UPGRADE_CALLDATA]"
  exit 1
fi

PROXY_ADDRESS=$1
UPGRADE_CALLDATA=${2:-"0x"}

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

RPC_URL=${RPC_URL:-"http://localhost:8547"}

if [ -z "$DEPLOY_PRIVATE_KEY" ]; then
  die "Missing environment variable: DEPLOY_PRIVATE_KEY"
fi

WALLET_ADDRESS=`cast wallet address $DEPLOY_PRIVATE_KEY`

TREASURY_ADDRESS=`$SCRIPT_DIR/deploy-construct1.sh treasury "constructor(address)" "$WALLET_ADDRESS" | awk -F: '/deployed code at address:/ {
    # 1. Remove ANSI color codes (the escape sequences) globally from the address field ($2)
    gsub(/\x1b\[[0-9;]*m/, "", $2);
    
    # 2. Trim leading spaces/tabs from $2
    sub(/^[ \t]+/, "", $2);
    
    # 3. Trim trailing spaces/tabs from $2
    sub(/[ \t]+$/, "", $2);
    
    # 4. Print the clean address
    print $2
}'`

echo "Treasury deployed at: $TREASURY_ADDRESS"

cast send --rpc-url $RPC_URL --private-key $DEPLOY_PRIVATE_KEY \
    $PROXY_ADDRESS "upgradeToAndCall(address,bytes)" "$TREASURY_ADDRESS" "$UPGRADE_CALLDATA"

