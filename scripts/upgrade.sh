#!/bin/bash
set -o pipefail

# Setup
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
if [ -f "$SCRIPT_DIR/vars.sh" ]; then
    . "$SCRIPT_DIR/vars.sh"
else
    echo "ERROR: vars.sh not found" && exit 1
fi

show_help() {
    echo "Usage: ./upgrade.sh <TYPE> <PROXY_ADDR> [ARG]"
    echo ""
    echo "Types:"
    echo "  core <PROXY> [CALLDATA]    Upgrade Castle logic via Proxy"
    echo "  clerk <PROXY> [CALLDATA] Upgrade Clerk logic via Proxy"
    echo "  officer <PROXY> <NAME>     Replace an officer (Direct appointment)"
    echo ""
    echo "Note: For UUPS, upgradeToAndCall is invoked on the PROXY address"
    echo "      but executed by the Logic implementation."
    exit 0
}

if [ "$#" -lt 2 ]; then show_help; fi

TYPE=$1
PROXY_ADDR=$2
EXTRA_ARG=$3 # Can be calldata for logic, or Name for officer

case "$TYPE" in
    "core" | "clerk")
        CONTRACT_NAME=$TYPE
        [ "$TYPE" == "core" ] && CONTRACT_NAME="castle"
        CALLDATA=${EXTRA_ARG:-"0x"}
        
        echo "--- Upgrading $CONTRACT_NAME (UUPS Pattern) ---"
        
        # 1. Deploy new logic (Observe Space)
        NEW_LOGIC=$(deploy "$CONTRACT_NAME" | tee /dev/stderr | parse_deployment_address)
        [ -z "$NEW_LOGIC" ] && die "Failed to deploy new $CONTRACT_NAME logic"
        
        # 2. Call upgradeToAndCall on the PROXY
        # This executes the upgrade logic stored in the CURRENT implementation
        echo "Executing upgradeToAndCall($NEW_LOGIC, $CALLDATA) via Proxy $PROXY_ADDR..."
        contract_send "$PROXY_ADDR" "upgradeToAndCall(address,bytes)" "$NEW_LOGIC" "$CALLDATA"
        
        echo "SUCCESS: $CONTRACT_NAME logic rotated to $NEW_LOGIC"
        ;;

    "officer")
        # Officers are usually direct appointments, not proxies themselves
        [ -z "$EXTRA_ARG" ] && die "Error: officer upgrade requires a name"
        OFFICER_NAME=$EXTRA_ARG
        
        echo "--- Replacing Officer: $OFFICER_NAME ---"
        NEW_OFFICER_ADDR=$(deploy "$OFFICER_NAME" | tee /dev/stderr | parse_deployment_address)
        [ -z "$NEW_OFFICER_ADDR" ] && die "Failed to deploy $OFFICER_NAME"
        
        # Convert name to method (e.g., banker -> appointBanker)
        METHOD="appoint$(echo "${OFFICER_NAME:0:1}" | tr '[:lower:]' '[:upper:]')${OFFICER_NAME:1}(address)"
        
        contract_send "$PROXY_ADDR" "$METHOD" "$NEW_OFFICER_ADDR"
        echo "SUCCESS: $OFFICER_NAME updated at $NEW_OFFICER_ADDR"
        ;;

    *)
        show_help
        ;;
esac