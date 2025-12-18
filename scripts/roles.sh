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
    echo "Usage: ./roles.sh <COMMAND> <TARGET_ADDRESS> <ROLE_NAME> [ARGS...]"
    echo ""
    echo "Commands:"
    echo "  has_role <ADDR> <ROLE> <USER>    Check if address has role"
    echo "  grant <ADDR> <ROLE> <USER>       Grant role to address"
    echo "  revoke <ADDR> <ROLE> <USER>      Revoke role from address"
    echo "  renounce <ADDR> <ROLE> <USER>    Renounce role (called by account with role)"
    echo "  delete <ADDR> <ROLE>             Delete a role entirely"
    echo "  count <ADDR> <ROLE>              Get number of assignees for a role"
    echo "  list <ADDR> <ROLE> <START> <LEN> List assignees (pagination)"
    echo "  admin <ADDR>                     Get the admin role hash"
    echo ""
    echo "Example:"
    echo "  ./roles.sh grant 0xCastleAddr \"Castle.ISSUER_ROLE\" 0xUserAddr"
    exit 0
}

# Ensure we have at least a command and a target
if [ "$#" -lt 2 ]; then
    show_help
fi

COMMAND=$1
TARGET_ADDRESS=$2
ROLE_NAME=$3

# Helper: Convert "Castle.ISSUER_ROLE" string to Keccak256 hash
get_role_hash() {
    local role_name="$1"
    cast keccak "$role_name"
}

# Determine Role Hash (Skip hashing if empty or already a hex string)
if [[ -n "$ROLE_NAME" ]]; then
    if [[ "$ROLE_NAME" == 0x* ]]; then
        ROLE_HASH="$ROLE_NAME"
    else
        ROLE_HASH=$(get_role_hash "$ROLE_NAME")
    fi
fi

case "$COMMAND" in
    "has_role")
        [ -z "$4" ] && show_help
        echo "Checking role $ROLE_NAME on $TARGET_ADDRESS..."
        contract_call "$TARGET_ADDRESS" "hasRole(bytes32,address)(bool)" "$ROLE_HASH" "$4"
        ;;

    "grant")
        [ -z "$4" ] && show_help
        echo "Granting $ROLE_NAME at $TARGET_ADDRESS to $4..."
        contract_send "$TARGET_ADDRESS" "grantRole(bytes32,address)" "$ROLE_HASH" "$4"
        ;;

    "revoke")
        [ -z "$4" ] && show_help
        echo "Revoking $ROLE_NAME at $TARGET_ADDRESS from $4..."
        contract_send "$TARGET_ADDRESS" "revokeRole(bytes32,address)" "$ROLE_HASH" "$4"
        ;;

    "renounce")
        [ -z "$4" ] && show_help
        echo "Renouncing $ROLE_NAME at $TARGET_ADDRESS for $4..."
        contract_send "$TARGET_ADDRESS" "renounceRole(bytes32,address)" "$ROLE_HASH" "$4"
        ;;

    "delete")
        echo "Deleting role $ROLE_NAME at $TARGET_ADDRESS..."
        contract_send "$TARGET_ADDRESS" "deleteRole(bytes32)(bool)" "$ROLE_HASH"
        ;;

    "admin")
        echo "Fetching Admin Role Hash from $TARGET_ADDRESS..."
        contract_call "$TARGET_ADDRESS" "getAdminRole()(bytes32)"
        ;;

    "count")
        echo "Count for $ROLE_NAME at $TARGET_ADDRESS:"
        contract_call "$TARGET_ADDRESS" "getRoleAssigneeCount(bytes32)(uint256)" "$ROLE_HASH"
        ;;

    "list")
        [ -z "$5" ] && show_help
        echo "Assignees for $ROLE_NAME (Start: $4, Max: $5):"
        contract_call "$TARGET_ADDRESS" "getRoleAssignees(bytes32,uint256,uint256)(address[])" "$ROLE_HASH" "$4" "$5"
        ;;

    *)
        show_help
        ;;
esac