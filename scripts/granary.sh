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

deploy_clerk() {
    local ADDR=$(deploy clerk | tee /dev/stderr | parse_deployment_address)
    [ -z "$ADDR" ] && die "Failed to parse Clerk address"
    echo "$ADDR"
}

deploy_logic() {
    local ADDR=$(deploy granary | tee /dev/stderr | parse_deployment_address)
    [ -z "$ADDR" ] && die "Failed to parse Granary logic address"
    echo "$ADDR"
}

deploy_gate() {
    local logic_addr=$1
    local castle_gate=$2
    local clerk_addr=$3

    if [ -z "$castle_gate" ] || [ -z "$clerk_addr" ]; then
        die "deploy_gate requires <CASTLE_GATE> and <CLERK_ADDRESS>"
    fi

    # UUPS Initialization for Granary
    local init_data=$(calldata "initialize(address,address)" "$castle_gate" "$clerk_addr")
    
    local gate_addr=$(deploy_construct gate "constructor(address,bytes)" "$logic_addr" "$init_data" | tee /dev/stderr | parse_deployment_address)
    [ -z "$gate_addr" ] && die "Failed to parse Granary Gate address"
    echo "$gate_addr"
}

upgrade_gate() {
    local proxy=$1
    local new_logic=$2
    local calldata=${3:-"0x"}
    
    echo "Upgrading Granary at $proxy to new logic $new_logic..."
    contract_send "$proxy" "upgradeToAndCall(address,bytes)" "$new_logic" "$calldata"
}

# --- Command Router ---

usage() {
    echo "Usage: $0 {full | deploy-logic | upgrade}"
    echo "  full <CASTLE_GATE|OWNER_EOA> : Deploys Clerk, Granary Logic, and Gate"
    echo "  deploy-logic                 : Deploys only Clerk and Granary logic"
    echo "  upgrade <PROXY> <LOGIC> [CD] : UUPS upgrade for existing Granary Gate"
    exit 1
}

case "$1" in
    "full")
        [ -z "$2" ] && usage
        echo "--- Deploying Clerk ---"
        CLERK=$(deploy_clerk)
        echo "--- Deploying Granary Logic ---"
        LOGIC=$(deploy_logic)
        echo "--- Deploying Granary Gate ---"
        GATE=$(deploy_gate "$LOGIC" "$2" "$CLERK")
        
        echo -e "\n=== GRANARY DEPLOYMENT COMPLETE ==="
        echo "Clerk address: $CLERK"
        echo "Granary Logic: $LOGIC"
        echo "Granary Gate : $GATE"
        echo "Granary Owner: $2"
        echo "------------------------------------"
        ;;
    "deploy-logic")
        CLERK=$(deploy_clerk)
        LOGIC=$(deploy_logic)
        echo "Clerk deployed at: $CLERK"
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