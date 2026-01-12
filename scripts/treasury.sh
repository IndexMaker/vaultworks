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
RPC_URL=${RPC_URL:-"http://localhost:8547"}
DEPLOYER_ADDRESS=$(deployer_address)

# Helper for Mac-compatible uppercase (for logging/vars)
to_upper() { echo "$1" | tr '[:lower:]' '[:upper:]'; }

# --- Core Functions ---

deploy_logic() {
    # Having issues calling constructors
    #local ADDR=$(deploy_construct treasury "constructor(address)" "$DEPLOYER_ADDRESS" | tee /dev/stderr | parse_deployment_address)
    local ADDR=$(deploy treasury | tee /dev/stderr | parse_deployment_address)
    [ -z "$ADDR" ] && die "Failed to parse Treasury logic address"
    echo "$ADDR"
}

deploy_gate() {
    local logic_addr=$1
    local init_data=$(calldata "initialize(address)" "$DEPLOYER_ADDRESS")
    # Having issues calling constructors
    #local gate_addr=$(deploy_construct gate "constructor(address,bytes)" "$logic_addr" "$init_data" | tee /dev/stderr | parse_deployment_address)
    local gate_addr=$(deploy gate | tee /dev/stderr | parse_deployment_address)
    contract_send $gate_addr "initialize(address,bytes)" "$logic_addr" "$init_data"
    [ -z "$gate_addr" ] && die "Failed to parse Gate address"
    echo "$gate_addr"
}

upgrade_gate() {
    local proxy=$1
    local new_logic=$2
    local calldata=${3:-"0x"}
    
    cast send --rpc-url "$RPC_URL" --private-key "$DEPLOY_PRIVATE_KEY" \
        "$proxy" "upgradeToAndCall(address,bytes)" "$new_logic" "$calldata"
}

# --- Command Router ---

usage() {
    echo "Usage: $0 {full | deploy-logic | deploy-gate | upgrade}"
    echo "  full                        : Deploys Logic AND Gate"
    echo "  deploy-logic                : Deploys only the Treasury logic contract"
    echo "  deploy-gate <LOGIC_ADDR>    : Deploys a Proxy pointing to LOGIC_ADDR"
    echo "  upgrade <PROXY> <LOGIC> [CD]: Upgrades existing Proxy to new Logic"
    exit 1
}

case "$1" in
    "full")
        LOGIC=$(deploy_logic)
        GATE=$(deploy_gate "$LOGIC")
        echo -e "\n=== FULL DEPLOYMENT COMPLETE ==="
        echo "Logic: $LOGIC"
        echo "Gate : $GATE"
        ;;
    "deploy-logic")
        LOGIC=$(deploy_logic)
        echo "Logic deployed at: $LOGIC"
        ;;
    "deploy-gate")
        [ -z "$2" ] && usage
        GATE=$(deploy_gate "$2")
        echo "Gate deployed at: $GATE"
        ;;
    "upgrade")
        [ -z "$3" ] && usage
        upgrade_gate "$2" "$3" "$4"
        ;;
    *)
        usage
        ;;
esac