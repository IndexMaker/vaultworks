#!/bin/bash
set -e

die() {
    echo "ERROR: $1" >&2
    exit 1
}

CARGO_METADATA_COMMAND="cargo metadata --no-deps --format-version 1"
if ! command -v jq &> /dev/null
then
    die "The 'jq' utility (JSON processor) is required to parse cargo metadata. Please install it."
fi

WORKSPACE_ROOT=$($CARGO_METADATA_COMMAND | jq -r '.workspace_root')
if [ -z "$WORKSPACE_ROOT" ]; then
    die "Could not determine the workspace root. Are you inside a Cargo project?"
fi

RPC_URL=${RPC_URL:-"http://localhost:8547"}
MAX_FEE_PER_GAS_GWEI=${MAX_FEE_PER_GAS_GWEI:-30}


set_vars() {
    PACKAGE_NAME=${1:-$(basename "$PWD")}

    PACKAGE_PATH="$WORKSPACE_ROOT/contracts/$PACKAGE_NAME"
    WASM_FILE_PATH="target/wasm32-unknown-unknown/release/$PACKAGE_NAME.wasm"

    if [ ! -d $PACKAGE_PATH ]; then
        die "Such contract does not exist '$PACKAGE_NAME' ($PACKAGE_PATH)"
    fi

    echo "-------------------------------------"
    echo "=== Script configuration complete ==="
    echo "-------------------------------------"
    echo "PACKAGE_NAME = $PACKAGE_NAME"
    echo "PACKAGE_PATH = $PACKAGE_PATH"
    echo "WORKSPACE_ROOT = $WORKSPACE_ROOT"
    echo "WASM_FILE_PATH = $WASM_FILE_PATH"
    echo "MAX_FEE_PER_GAS_GWEI= $MAX_FEE_PER_GAS_GWEI"
    echo "RPC_URL = $RPC_URL"
    echo "-------------------------------------"
}

check() {
    if [ "$#" -le 0 ]; then
        echo "check CONTRACT_NAME [OPTIONAL ARGS...]"
        exit 1
    fi

    set_vars $1

    rm -f "$WORKSPACE_ROOT/$WASM_FILE_PATH"

    # We need to do this to build project as passing --wasm-file does not build sources
    cd $PACKAGE_PATH && cargo stylus check || true 

    if [ ! -f "$WORKSPACE_ROOT/$WASM_FILE_PATH" ]; then
        die "Failed to build contract: '$PACKAGE_NAME'"
    else
        echo -en "^^^ Please ignore ^^^ if you see an error above saying: \"could not read release deps dir\".\n\n"
    fi

    # Then we run actual check
    cd $PACKAGE_PATH && cargo stylus check \
        --endpoint="$RPC_URL" \
        --wasm-file="$WORKSPACE_ROOT/$WASM_FILE_PATH" \
        --source-files-for-project-hash="$PACKAGE_PATH"
}

deploy() {
    if [ "$#" -le 0 ]; then
        echo "deploy.sh CONTRACT_NAME [DEPLOY_ARGS...]"
        exit 1
    fi

    set_vars $1

    if [ -z "$DEPLOY_PRIVATE_KEY" ]; then
        die "Missing environment variable: DEPLOY_PRIVATE_KEY"
    fi

    if [ ! -f "./$WASM_FILE_PATH" ]; then
        check $PACKAGE_NAME
    fi

    STYLUS_ARGS=(
        --wasm-file="./$WASM_FILE_PATH" \
        --endpoint="$RPC_URL" \
        --no-verify \
        --max-fee-per-gas-gwei=$MAX_FEE_PER_GAS_GWEI \
        "${@:3}" \
    )

    echo "cd $WORKSPACE_ROOT && cargo stylus deploy --private-key=\"\$DEPLOY_PRIVATE_KEY\" ${STYLUS_ARGS[@]}"

    cd $WORKSPACE_ROOT && cargo stylus deploy --private-key="$DEPLOY_PRIVATE_KEY" "${STYLUS_ARGS[@]}"
}

deploy_construct() {
    if [ "$#" -le 1 ]; then
        echo "deploy_construct CONTRACT_NAME CONSTRUCTOR_SIGNATURE [CONSTRUCTOR_ARGS...]"
        exit 1
    fi

    set_vars $1

    if [ -z "$DEPLOY_PRIVATE_KEY" ]; then
        die "Missing environment variable: DEPLOY_PRIVATE_KEY"
    fi

    if [ ! -f "./$WASM_FILE_PATH" ]; then
        check $PACKAGE_NAME
    fi

    STYLUS_ARGS=(
        --wasm-file="./$WASM_FILE_PATH" \
        --endpoint="$RPC_URL" \
        --no-verify \
        --max-fee-per-gas-gwei=$MAX_FEE_PER_GAS_GWEI \
        --constructor-signature="$2"
    )

    for arg in "${@:3}"; do
        STYLUS_ARGS+=(--constructor-args="$arg")
    done

    echo "cd $WORKSPACE_ROOT && cargo stylus deploy --private-key=\"\$DEPLOY_PRIVATE_KEY\" ${STYLUS_ARGS[@]}"

    cd $WORKSPACE_ROOT && cargo stylus deploy --private-key="$DEPLOY_PRIVATE_KEY" "${STYLUS_ARGS[@]}"
}

export_abi() {
    if [ "$#" -le 0 ]; then
        echo "export_abi CONTRACT_NAME [OPTIONAL ARGS...]"
        exit 1
    fi

    set_vars $1

    cd $PACKAGE_PATH && RUST_BACKTRACE=1 cargo stylus export-abi ${@:2}
}

deployer_address() {
    if [ -z "$DEPLOY_PRIVATE_KEY" ]; then
        die "Missing environment variable: DEPLOY_PRIVATE_KEY"
    fi

    cast wallet address $DEPLOY_PRIVATE_KEY
}

calldata() {
    cast calldata "$@"
}

contract_send() {
    if [ -z "$DEPLOY_PRIVATE_KEY" ]; then
        die "Missing environment variable: DEPLOY_PRIVATE_KEY"
    fi

    ADDRESS=$1
    FUNCTION_SELECTOR=$2
    WALLET_ADDRESS=`cast wallet address $DEPLOY_PRIVATE_KEY`

    CAST_ARGS=(
        --rpc-url $RPC_URL $ADDRESS "$FUNCTION_SELECTOR" "${@:3}"
    )
    
    echo "cast send --private-key \$DEPLOY_PRIVATE_KEY ${CAST_ARGS[@]}" >&2

    cast send --private-key $DEPLOY_PRIVATE_KEY "${CAST_ARGS[@]}"
}

contract_call() {
    if [ -z "$DEPLOY_PRIVATE_KEY" ]; then
        die "Missing environment variable: DEPLOY_PRIVATE_KEY"
    fi

    ADDRESS=$1
    FUNCTION_SELECTOR=$2
    WALLET_ADDRESS=`cast wallet address $DEPLOY_PRIVATE_KEY`

    CAST_ARGS=(
        --rpc-url $RPC_URL $ADDRESS "$FUNCTION_SELECTOR" "${@:3}"
    )
    
    echo "cast call --private-key \$DEPLOY_PRIVATE_KEY ${CAST_ARGS[@]}" >&2

    cast call --private-key $DEPLOY_PRIVATE_KEY "${CAST_ARGS[@]}"
}

parse_deployment_address() {
    # Remove ANSI color codes (the escape sequences) globally from the address field ($2)
    # Trim leading spaces/tabs from $2
    # Trim trailing spaces/tabs from $2
    # Print the clean address
    awk -F: '/deployed code at address:/ {
        gsub(/\x1b\[[0-9;]*m/, "", $2);
        sub(/^[ \t]+/, "", $2);
        sub(/[ \t]+$/, "", $2);
        print $2
    }'
}