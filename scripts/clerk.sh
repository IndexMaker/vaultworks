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

deploy_abacus() {
    local ADDR=$(deploy abacus | tee /dev/stderr | parse_deployment_address)
    [ -z "$ADDR" ] && die "Failed to parse Abacus address"
    echo "$ADDR"
}

deploy_logic() {
    local ADDR=$(deploy clerk | tee /dev/stderr | parse_deployment_address)
    [ -z "$ADDR" ] && die "Failed to parse Clerk logic address"
    echo "$ADDR"
}

deploy_gate() {
    local logic_addr=$1
    local castle_gate=$2
    local abacus_addr=$3

    if [ -z "$castle_gate" ] || [ -z "$abacus_addr" ]; then
        die "deploy_gate requires <CASTLE_GATE> and <ABACUS_ADDRESS>"
    fi

    # UUPS Initialization for Clerk
    local init_data=$(calldata "initialize(address,address)" "$castle_gate" "$abacus_addr")
    
    local gate_addr=$(deploy_construct gate "constructor(address,bytes)" "$logic_addr" "$init_data" | tee /dev/stderr | parse_deployment_address)
    [ -z "$gate_addr" ] && die "Failed to parse Clerk Gate address"
    echo "$gate_addr"
}

upgrade_gate() {
    local proxy=$1
    local new_logic=$2
    local calldata=${3:-"0x"}
    
    echo "Upgrading Clerk at $proxy to new logic $new_logic..."
    contract_send "$proxy" "upgradeToAndCall(address,bytes)" "$new_logic" "$calldata"
}

# --- Command Router ---

usage() {
    echo "Usage: $0 {full | deploy-logic | upgrade}"
    echo "  full <CASTLE_GATE|OWNER_EOA> : Deploys Abacus, Clerk Logic, and Gate"
    echo "  deploy-logic                 : Deploys only Abacus and Clerk logic"
    echo "  upgrade <PROXY> <LOGIC> [CD] : UUPS upgrade for existing Clerk Gate"
    exit 1
}

case "$1" in
    "full")
        [ -z "$2" ] && usage
        echo "--- Deploying Abacus ---"
        ABACUS=$(deploy_abacus)
        echo "--- Deploying Clerk Logic ---"
        LOGIC=$(deploy_logic)
        echo "--- Deploying Clerk Gate ---"
        GATE=$(deploy_gate "$LOGIC" "$2" "$ABACUS")
        
        echo -e "\n=== CLERK DEPLOYMENT COMPLETE ==="
        echo "Abacus address: $ABACUS"
        echo "Clerk Logic: $LOGIC"
        echo "Clerk Gate : $GATE"
        echo "Clerk Owner: $2"
        echo "------------------------------------"
        ;;
    "deploy-logic")
        ABACUS=$(deploy_abacus)
        LOGIC=$(deploy_logic)
        echo "Abacus deployed at: $ABACUS"
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