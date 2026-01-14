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

deploy_extensions() {
    echo "--- Deploying Native Extensions ---"
    ORDERS_ADDR=$(deploy vault_native_orders | tee /dev/stderr | parse_deployment_address)
    CLAIMS_ADDR=$(deploy vault_native_claims | tee /dev/stderr | parse_deployment_address)
    
    [ -z "$ORDERS_ADDR" ] && die "Failed to deploy orders"
    [ -z "$CLAIMS_ADDR" ] && die "Failed to deploy claims"
    
    # Exporting variables so 'full' can see them
    echo "$ORDERS_ADDR $CLAIMS_ADDR"
}

install_extensions() {
    local gate=$1
    local orders=$2
    local claims=$3
    
    echo "--- Installing Extensions into Gate at $gate ---"
    contract_send "$gate" "installOrders(address)" "$orders"
    contract_send "$gate" "installClaims(address)" "$claims"
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
        EXT_ADDRS=$(deploy_extensions)
        ORDERS=$(echo $EXT_ADDRS | cut -d' ' -f1)
        CLAIMS=$(echo $EXT_ADDRS | cut -d' ' -f2)

        # 3. Final Wiring
        install_extensions "$GATE" "$ORDERS" "$CLAIMS"
        
        echo -e "\n=== VAULT DEPLOYMENT COMPLETE ==="
        echo "Vault Gate : $GATE"
        echo "Orders     : $ORDERS"
        echo "Claims     : $CLAIMS"
        echo "------------------------------------"
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
