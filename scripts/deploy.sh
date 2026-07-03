#!/bin/bash

# Soroban Flux Deployment Script
# Supports deployment to testnet and mainnet

set -e

NETWORK="${1:-testnet}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Configuration
NETWORK_PASSPHRASE=""
NETWORK_URL=""
CONTRACT_PATH="${PROJECT_ROOT}/contracts/flux_engine"
BUILD_DIR="${CONTRACT_PATH}/target/wasm32-unknown-unknown/release"
CONTRACT_WASM="${BUILD_DIR}/flux_engine.wasm"

# Network configuration
case "$NETWORK" in
  testnet)
    NETWORK_URL="https://soroban-testnet.stellar.org"
    NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
    ;;
  mainnet)
    NETWORK_URL="https://soroban-mainnet.stellar.org"
    NETWORK_PASSPHRASE="Public Global Stellar Network ; September 2015"
    ;;
  *)
    echo "Error: Unknown network '$NETWORK'"
    echo "Usage: $0 {testnet|mainnet}"
    exit 1
    ;;
esac

echo "========================================"
echo "Soroban Flux Deployment"
echo "========================================"
echo "Network: $NETWORK"
echo "URL: $NETWORK_URL"
echo ""

# Build contract if not already built
if [ ! -f "$CONTRACT_WASM" ]; then
  echo "Building Soroban contract..."
  cd "$CONTRACT_PATH"
  cargo build --target wasm32-unknown-unknown --release
  cd - > /dev/null
fi

echo "Contract binary: $CONTRACT_WASM"
echo ""

# Validate contract exists
if [ ! -f "$CONTRACT_WASM" ]; then
  echo "Error: Contract WASM not found at $CONTRACT_WASM"
  exit 1
fi

# Optimize contract size (strip unnecessary code)
echo "Optimizing contract..."
if command -v wasm-opt &> /dev/null; then
  wasm-opt -Oz -o "${CONTRACT_WASM}.opt" "$CONTRACT_WASM"
  mv "${CONTRACT_WASM}.opt" "$CONTRACT_WASM"
  echo "Contract size: $(du -h "$CONTRACT_WASM" | cut -f1)"
else
  echo "Warning: wasm-opt not found, skipping optimization"
fi

# Deploy configuration
STELLAR_ACCOUNT="${STELLAR_ACCOUNT:-}"
ADMIN_ADDRESS="${ADMIN_ADDRESS:-}"

# Verify deployment prerequisites
if [ -z "$STELLAR_ACCOUNT" ]; then
  echo "Error: STELLAR_ACCOUNT environment variable not set"
  exit 1
fi

if [ -z "$ADMIN_ADDRESS" ]; then
  echo "Error: ADMIN_ADDRESS environment variable not set"
  exit 1
fi

echo "Deploying contract..."
echo "Account: $STELLAR_ACCOUNT"
echo "Admin: $ADMIN_ADDRESS"
echo ""

# Deploy contract using soroban-cli
# This is a placeholder - actual deployment would use soroban CLI commands
echo "Running deployment (placeholder - configure with actual soroban-cli commands)"

# Example soroban CLI deployment:
# soroban contract deploy \
#   --wasm "$CONTRACT_WASM" \
#   --network "$NETWORK" \
#   --source-account "$STELLAR_ACCOUNT"

echo ""
echo "========================================"
echo "Deployment Complete"
echo "========================================"
echo ""
echo "Next steps:"
echo "1. Verify contract on explorer: https://${NETWORK}-explorer.stellar.org"
echo "2. Initialize contract with admin configuration"
echo "3. Deploy frontend with contract address"
echo ""
