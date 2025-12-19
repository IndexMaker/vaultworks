#!/bin/bash
set -o pipefail

# Setup
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
if [ -f "$SCRIPT_DIR/vars.sh" ]; then
    . "$SCRIPT_DIR/vars.sh"
else
    echo "ERROR: vars.sh not found" && exit 1
fi

# 1. Help Function
show_help() {
    echo "Usage: ./deploy-all.sh [OPTIONS]"
    echo ""
    echo "Core Options:"
    echo "  --help                        Show this help message"
    echo "  --no-gates                    Deploy logic directly without Proxy (Gate) contracts"
    echo "  --no-castle <ADDRESS>         Skip Castle/Gate deployment and use existing address"
    echo "  --no-clerk                  Skip Clerk deployment and attachment"
    echo "  --only-clerk <CASTLE_ADDR>  Skip everything except Clerk deployment and link to CASTLE_ADDR"
    echo ""
    echo "Officer Appointment Toggles (Skip specific officers):"
    echo "  --no-constable                *STOPS FLOW* after Gate/Castle setup"
    echo "  --no-banker                   Skip Banker appointment"
    echo "  --no-factor                   Skip Factor appointment"
    echo "  --no-guildmaster              Skip Guildmaster appointment"
    echo "  --no-scribe                   Skip Scribe appointment"
    echo "  --no-worksman                 Skip Worksman appointment"
    echo ""
    echo "Examples:"
    echo "  ./castle.sh --no-gates                  # Direct logic deployment"
    echo "  ./castle.sh --no-castle 0x123...        # Add officers/clerk to existing castle"
    echo "  ./castle.sh --only-clerk 0x123...     # Only deploy and attach a new Clerk"
    exit 0
}

# 2. Parse Options
USE_GATES=true
SKIP_CLERK=false
ONLY_CLERK=false
NO_CASTLE=false
CASTLE_TARGET_ADDR=""

DO_CONSTABLE=true
DO_BANKER=true
DO_FACTOR=true
DO_GUILDMASTER=true
DO_SCRIBE=true
DO_WORKSMAN=true

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --help)          show_help ;;
        --no-gates)      USE_GATES=false ;;
        --no-clerk)    SKIP_CLERK=true ;;
        --no-constable)  DO_CONSTABLE=false ;;
        --no-banker)     DO_BANKER=false ;;
        --no-factor)     DO_FACTOR=false ;;
        --no-guildmaster) DO_GUILDMASTER=false ;;
        --no-scribe)     DO_SCRIBE=false ;;
        --no-worksman)   DO_WORKSMAN=false ;;
        --no-castle)     
            NO_CASTLE=true
            CASTLE_TARGET_ADDR="$2"
            shift
            ;;
        --only-clerk) 
            ONLY_CLERK=true
            CASTLE_TARGET_ADDR="$2"
            shift 
            ;;
        *) echo "Unknown option: $1. Use --help for usage."; exit 1 ;;
    esac
    shift
done

DEPLOYER_ADDRESS=$(deployer_address)

# --- Logic Branching ---

if [ "$NO_CASTLE" = true ] || [ "$ONLY_CLERK" = true ]; then
    if [ -z "$CASTLE_TARGET_ADDR" ]; then
        die "Error: --no-castle or --only-clerk requires a <CASTLE_ADDRESS>"
    fi
    echo "Mode: Using existing Castle at $CASTLE_TARGET_ADDR"
    TARGET_ADDRESS=$CASTLE_TARGET_ADDR
else
    echo "Deploying from: $DEPLOYER_ADDRESS (Use Gates: $USE_GATES)"
    CASTLE_ADDRESS=$(deploy castle | tee /dev/stderr | parse_deployment_address)
    [ -z "$CASTLE_ADDRESS" ] && die "Cannot parse address of: castle"

    if [ "$USE_GATES" = true ]; then
        CALLDATA=$(calldata "initialize(address,address)" "$CASTLE_ADDRESS" "$DEPLOYER_ADDRESS")
        TARGET_ADDRESS=$(deploy_construct gate "constructor(address,bytes)" "$CASTLE_ADDRESS" "$CALLDATA" | tee /dev/stderr | parse_deployment_address)
        [ -z "$TARGET_ADDRESS" ] && die "Cannot parse address of: gate (Castle Proxy)"
    else
        TARGET_ADDRESS=$CASTLE_ADDRESS
        echo "Direct Deployment: Initializing Castle Logic..."
        contract_send "$TARGET_ADDRESS" "initialize(address,address)" "$CASTLE_ADDRESS" "$DEPLOYER_ADDRESS"
    fi
fi

# 4. Handle Constable Logic (Stop Flow Check)
if [ "$DO_CONSTABLE" = false ] && [ "$ONLY_CLERK" = false ]; then
    echo "--- --no-constable detected. Flow stopped after setup. ---"
    echo "Castle Target: $TARGET_ADDRESS"
    exit 0
fi

appoint_officer() {
    local name=$1
    local target=$2
    local method=$3
    echo "--- Appointing $name ---"
    local ADDR=$(deploy "$name" | tee /dev/stderr | parse_deployment_address)
    [ -z "$ADDR" ] && die "Failed to deploy officer: $name"
    contract_send "$target" "$method" "$ADDR"
    local UP_NAME=$(echo "$name" | tr '[:lower:]' '[:upper:]')
    eval "${UP_NAME}_ADDRESS=\"$ADDR\""
}

if [ "$ONLY_CLERK" = false ]; then
    [ "$DO_CONSTABLE" = true ]   && appoint_officer "constable"   "$TARGET_ADDRESS" "appointConstable(address)"
    [ "$DO_BANKER" = true ]      && appoint_officer "banker"      "$TARGET_ADDRESS" "appointBanker(address)"
    [ "$DO_FACTOR" = true ]      && appoint_officer "factor"      "$TARGET_ADDRESS" "appointFactor(address)"
    [ "$DO_GUILDMASTER" = true ] && appoint_officer "guildmaster" "$TARGET_ADDRESS" "appointGuildmaster(address)"
    [ "$DO_SCRIBE" = true ]      && appoint_officer "scribe"      "$TARGET_ADDRESS" "appointScribe(address)"
    [ "$DO_WORKSMAN" = true ]    && appoint_officer "worksman"    "$TARGET_ADDRESS" "appointWorksman(address)"
fi

# --- Clerk Sequence ---

if [ "$SKIP_CLERK" = false ]; then
    ABACUS_ADDRESS=$(deploy abacus | tee /dev/stderr | parse_deployment_address)
    CLERK_ADDRESS=$(deploy clerk | tee /dev/stderr | parse_deployment_address)
    [ -z "$CLERK_ADDRESS" ] && die "Failed to deploy Clerk"

    if [ "$USE_GATES" = true ]; then
        CALLDATA_CLERK=$(calldata "initialize(address,address)" "$TARGET_ADDRESS" "$ABACUS_ADDRESS")
        CLERK_TARGET=$(deploy_construct gate "constructor(address,bytes)" "$CLERK_ADDRESS" "$CALLDATA_CLERK" | tee /dev/stderr | parse_deployment_address)
        [ -z "$CLERK_TARGET" ] && die "Failed to deploy Clerk Gate"
    else
        CLERK_TARGET=$CLERK_ADDRESS
        echo "Direct Deployment: Initializing Clerk Logic..."
        contract_send "$CLERK_TARGET" "initialize(address,address)" "$TARGET_ADDRESS" "$ABACUS_ADDRESS"
    fi

    echo "--- Attaching Clerk to Castle ---"
    contract_send "$TARGET_ADDRESS" "appendClerk(address)" "$CLERK_TARGET"
fi

echo "---------------------------"
echo "=== Deployment Complete ==="
echo "---------------------------"
echo "Castle Target: $TARGET_ADDRESS"
[ "$SKIP_CLERK" = false ] && echo "Clerk Target: $CLERK_TARGET"
echo "---------------------------"