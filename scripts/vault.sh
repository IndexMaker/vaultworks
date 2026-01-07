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
VAULT_PROVIDER="vault_requests"

# --- Core Functions ---

deploy_vault_provider() {
    local target=$1
    local ADDR=$(deploy "$target" | tee /dev/stderr | parse_deployment_address)
    [ -z "$ADDR" ] && die "Failed to parse $target address"
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
    local provider_addr=$3

    if [ -z "$castle_gate" ] || [ -z "$provider_addr" ]; then
        die "deploy_gate requires <CASTLE_GATE> and <PROVIDER_ADDRESS>"
    fi

    # UUPS Initialization for vault
    local init_data=$(calldata "initialize(address,address,address)" "$DEPLOYER_ADDRESS" "$provider_addr" "$castle_gate")
    
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
    echo "Usage: $0 {full | deploy-logic | upgrade} [--native]"
    echo "  full <CASTLE_GATE>           : Deploys Provider, vault Logic, and Gate"
    echo "  deploy-logic                 : Deploys only Provider and vault logic"
    echo "  upgrade <PROXY> <LOGIC> [CD] : UUPS upgrade for existing vault Gate"
    echo ""
    echo "Options:"
    echo "  --native                     : Use vault_native instead of vault_requests"
    exit 1
}

# Simple flag parsing
ARGS=()
while [[ $# -gt 0 ]]; do
    case "$1" in
        --native)
            VAULT_PROVIDER="vault_native"
            shift
            ;;
        *)
            ARGS+=("$1")
            shift
            ;;
    esac
done

set -- "${ARGS[@]}"

case "$1" in
    "full")
        [ -z "$2" ] && usage
        echo "--- Deploying $VAULT_PROVIDER ---"
        provider_addr=$(deploy_vault_provider "$VAULT_PROVIDER")
        echo "--- Deploying vault Logic ---"
        LOGIC=$(deploy_logic)
        echo "--- Deploying vault Gate ---"
        GATE=$(deploy_gate "$LOGIC" "$2" "$provider_addr")
        
        echo -e "\n=== VAULT DEPLOYMENT COMPLETE ==="
        echo "Vault Provider ($VAULT_PROVIDER): $provider_addr"
        echo "Vault Logic: $LOGIC"
        echo "Vault Gate : $GATE"
        echo "Vault Owner: $2"
        echo "------------------------------------"
        ;;
    "deploy-logic")
        provider_addr=$(deploy_vault_provider "$VAULT_PROVIDER")
        LOGIC=$(deploy_logic)
        echo "$VAULT_PROVIDER deployed at: $provider_addr"
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