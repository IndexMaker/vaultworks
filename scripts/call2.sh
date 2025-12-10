#!/bin/bash

if [ "$#" -le 3 ]; then
  echo "call2.sh ADDRESS FUNCTION_SELECTOR ARG1 ARG2"
  exit 1
fi

ADDRESS=$1
FUNCTION_SELECTOR=$2
ARG1=$3
ARG2=$4

RPC_URL=${RPC_URL:-"http://localhost:8547"}

if [ -z "$DEPLOY_PRIVATE_KEY" ]; then
  die "Missing environment variable: DEPLOY_PRIVATE_KEY"
fi

WALLET_ADDRESS=`cast wallet address $DEPLOY_PRIVATE_KEY`

cast call --rpc-url $RPC_URL --private-key $DEPLOY_PRIVATE_KEY \
    $ADDRESS "$FUNCTION_SELECTOR" "$ARG1" "$ARG2"

