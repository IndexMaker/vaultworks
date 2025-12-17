#!/bin/bash

set -o pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

. $SCRIPT_DIR/vars.sh

set_vars $1

DEPLOYMENT_ADDRESS=$(deploy "$@" | tee /dev/stderr | parse_deployment_address)
DEPLOYER_ADDRESS=$(deployer_address)

if [ -z $DEPLOYMENT_ADDRESS ]; then
  die "Deployment failed: Address could not be parsed"
fi

echo "------------------------------------------------------------------------------------------------------------------------"
echo "Contract '$PACKAGE_NAME' deployed at: $DEPLOYMENT_ADDRESS" by: $DEPLOYER_ADDRESS
echo "------------------------------------------------------------------------------------------------------------------------"
