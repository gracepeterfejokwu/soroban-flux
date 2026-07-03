#!/bin/bash

# Soroban Flux Build Script
# Orchestrates building contracts and frontend

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "========================================"
echo "Soroban Flux Build System"
echo "========================================"
echo ""

# Build contracts
echo "Building Soroban contracts..."
cd "$PROJECT_ROOT/contracts/flux_engine"

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
  echo "Error: Rust/Cargo not installed"
  exit 1
fi

# Build release binary
cargo build --target wasm32-unknown-unknown --release
CONTRACT_BUILD_STATUS=$?

if [ $CONTRACT_BUILD_STATUS -eq 0 ]; then
  WASM_FILE="$PROJECT_ROOT/contracts/flux_engine/target/wasm32-unknown-unknown/release/flux_engine.wasm"
  SIZE=$(du -h "$WASM_FILE" | cut -f1)
  echo "✓ Contract built successfully"
  echo "  Output: $WASM_FILE"
  echo "  Size: $SIZE"
else
  echo "✗ Contract build failed"
  cd - > /dev/null
  exit 1
fi

cd - > /dev/null
echo ""

# Build frontend
if [ -f "$PROJECT_ROOT/frontend/package.json" ]; then
  echo "Building Next.js frontend..."
  cd "$PROJECT_ROOT/frontend"
  
  # Check if Node.js is installed
  if ! command -v node &> /dev/null; then
    echo "Error: Node.js not installed"
    exit 1
  fi
  
  # Install dependencies if needed
  if [ ! -d "node_modules" ]; then
    echo "Installing dependencies..."
    npm install
  fi
  
  # Build frontend
  npm run build
  FRONTEND_BUILD_STATUS=$?
  
  if [ $FRONTEND_BUILD_STATUS -eq 0 ]; then
    echo "✓ Frontend built successfully"
    echo "  Output: $PROJECT_ROOT/frontend/.next"
  else
    echo "✗ Frontend build failed"
    cd - > /dev/null
    exit 1
  fi
  
  cd - > /dev/null
fi

echo ""
echo "========================================"
echo "Build Summary"
echo "========================================"
echo "✓ Build completed successfully"
echo ""
echo "Next steps:"
echo "1. Run tests: make test"
echo "2. Deploy: make deploy NETWORK=testnet"
echo "3. Start dev server: cd frontend && npm run dev"
echo ""
