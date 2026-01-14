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
VAULT_PROVIDER="vault_native"

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

    local init_data=$(calldata "initialize(address,address,address)" "$DEPLOYER_ADDRESS" "$provider_addr" "$castle_gate")
    local gate_addr=$(deploy gate | tee /dev/stderr | parse_deployment_address)
    [ -z "$gate_addr" ] && die "Failed to parse vault Gate address"
    
    contract_send "$gate_addr" "initialize(address,bytes)" "$logic_addr" "$init_data"
    echo "$gate_addr"
}

deploy_orders() {
    local gate=$1
    local orders=$(deploy vault_native_orders | tee /dev/stderr | parse_deployment_address)
    contract_send "$gate" "installOrders(address)" "$orders"
    echo "$orders"
}

deploy_claims() {
    local gate=$1
    local claims=$(deploy vault_native_claims | tee /dev/stderr | parse_deployment_address)
    contract_send "$gate" "installClaims(address)" "$claims"
    echo "$claims"
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
    echo "  full <CASTLE_GATE>           : Complete deployment and setup"
    echo "  deploy-logic                 : Deploys only Provider and vault logic"
    echo "  upgrade <PROXY> <LOGIC> [CD] : UUPS upgrade for existing vault Gate"
    exit 1
}

case "$1" in
    "full")
        [ -z "$2" ] && usage
        
        # 1. Base Infrastructure
        provider_addr=$(deploy_vault_provider "$VAULT_PROVIDER")
        LOGIC=$(deploy_logic)
        GATE=$(deploy_gate "$LOGIC" "$2" "$provider_addr")

        # 2. Extensions
        ORDERS=$(deploy_orders $GATE)
        CLAIMS=$(deploy_claims $GATE)
        
        echo "======================================================"
        echo "                Deployment Complete                   "
        echo "------------------------------------------------------"
        echo "  * Vault Gate:           $GATE"
        echo ""
        echo "======================================================"
        echo "               Diamond Configuration                  "
        echo "------------------------------------------------------"
        echo " Vault Implementation:    $LOGIC"
        echo " Vault Native:            $provider_addr"
        echo " Vault Native Orders:     $ORDERS"
        echo " Vault Native Claims:     $CLAIMS"
        echo "======================================================"
        ;;

    "deploy-logic")
        provider_addr=$(deploy_vault_provider "$VAULT_PROVIDER")
        LOGIC=$(deploy_logic)
        echo "$VAULT_PROVIDER: $provider_addr"
        echo "Logic: $LOGIC"
        ;;

    "upgrade")
        [ -z "$3" ] && usage
        upgrade_gate "$2" "$3" "$4"
        ;;

    *)
        usage
        ;;
esac
