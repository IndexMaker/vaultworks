#!/bin/bash
set -o pipefail

# Setup
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

if [ -f "$SCRIPT_DIR/vars.sh" ]; then
    . "$SCRIPT_DIR/vars.sh"
else
    echo "ERROR: vars.sh not found" && exit 1
fi

# Configuration Defaults
DEPLOYER_ADDRESS=$(deployer_address)

# --- Core Functions ---

deploy_vault_requests() {
    local ADDR=$(deploy vault_requests | tee /dev/stderr | parse_deployment_address)
    [ -z "$ADDR" ] && die "Failed to parse vault_requests address"
    echo "$ADDR"
}

deploy_logic() {
    local ADDR=$(deploy vault | tee /dev/stderr | parse_deployment_address)
    [ -z "$ADDR" ] && die "Failed to parse vault logic address"
    echo "$ADDR"
}

deploy_gate() {
    local logic_addr=$1
    local castle_gate=$2
    local vault_requests_addr=$3

    if [ -z "$castle_gate" ] || [ -z "$vault_requests_addr" ]; then
        die "deploy_gate requires <CASTLE_GATE> and <vault_requests_ADDRESS>"
    fi

    # UUPS Initialization for vault
    local init_data=$(calldata "initialize(address,address,address)" "$DEPLOYER_ADDRESS" "$vault_requests_addr" "$castle_gate")
    
    local gate_addr=$(deploy_construct gate "constructor(address,bytes)" "$logic_addr" "$init_data" | tee /dev/stderr | parse_deployment_address)
    [ -z "$gate_addr" ] && die "Failed to parse vault Gate address"
    echo "$gate_addr"
}

upgrade_gate() {
    local proxy=$1
    local new_logic=$2
    local calldata=${3:-"0x"}
    
    echo "Upgrading vault at $proxy to new logic $new_logic..."
    contract_send "$proxy" "upgradeToAndCall(address,bytes)" "$new_logic" "$calldata"
}

# --- Command Router ---

usage() {
    echo "Usage: $0 {full | deploy-logic | upgrade}"
    echo "  full <CASTLE_GATE>           : Deploys vault_requests, vault Logic, and Gate"
    echo "  deploy-logic                 : Deploys only vault_requests and vault logic"
    echo "  upgrade <PROXY> <LOGIC> [CD] : UUPS upgrade for existing vault Gate"
    exit 1
}

case "$1" in
    "full")
        [ -z "$2" ] && usage
        echo "--- Deploying vault_requests ---"
        vault_requests=$(deploy_vault_requests)
        echo "--- Deploying vault Logic ---"
        LOGIC=$(deploy_logic)
        echo "--- Deploying vault Gate ---"
        GATE=$(deploy_gate "$LOGIC" "$2" "$vault_requests")
        
        echo -e "\n=== vault DEPLOYMENT COMPLETE ==="
        echo "Vault Requests address: $vault_requests"
        echo "Vault Logic: $LOGIC"
        echo "Vault Gate : $GATE"
        echo "Vault Owner: $2"
        echo "------------------------------------"
        ;;
    "deploy-logic")
        vault_requests=$(deploy_vault_requests)
        LOGIC=$(deploy_logic)
        echo "vault_requests deployed at: $vault_requests"
        echo "Logic deployed at: $LOGIC"
        ;;
    "upgrade")
        [ -z "$3" ] && usage
        upgrade_gate "$2" "$3" "$4"
        ;;
    *)
        usage
        ;;
esac