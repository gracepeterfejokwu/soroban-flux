#!/bin/bash

# Soroban Flux Test Runner
# Runs all tests: contract tests and frontend tests

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "========================================"
echo "Soroban Flux Test Suite"
echo "========================================"
echo ""

# Run contract tests
echo "Running contract tests..."
cd "$PROJECT_ROOT/contracts/flux_engine"
cargo test --lib 2>&1 | tee test-contract.log
CONTRACT_STATUS=${PIPESTATUS[0]}
cd - > /dev/null

if [ $CONTRACT_STATUS -eq 0 ]; then
  echo "✓ Contract tests passed"
else
  echo "✗ Contract tests failed"
  exit 1
fi

echo ""

# Run frontend tests if test command exists
if [ -f "$PROJECT_ROOT/frontend/package.json" ]; then
  echo "Running frontend tests..."
  cd "$PROJECT_ROOT/frontend"
  
  if grep -q '"test"' package.json; then
    npm test -- --run 2>&1 | tee test-frontend.log
    FRONTEND_STATUS=${PIPESTATUS[0]}
  else
    echo "No frontend tests configured"
    FRONTEND_STATUS=0
  fi
  
  cd - > /dev/null
  
  if [ $FRONTEND_STATUS -eq 0 ]; then
    echo "✓ Frontend tests passed (or skipped)"
  else
    echo "✗ Frontend tests failed"
    exit 1
  fi
fi

echo ""
echo "========================================"
echo "Test Summary"
echo "========================================"
echo "✓ All tests passed successfully"
echo ""
