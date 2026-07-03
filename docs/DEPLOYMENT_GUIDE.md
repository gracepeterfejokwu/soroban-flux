# Soroban Flux Deployment Guide

Production deployment procedures for Soroban Flux streaming payment contract.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Environment Setup](#environment-setup)
3. [Building for Deployment](#building-for-deployment)
4. [Testnet Deployment](#testnet-deployment)
5. [Verification Procedures](#verification-procedures)
6. [Mainnet Deployment](#mainnet-deployment)
7. [Monitoring & Health Checks](#monitoring--health-checks)
8. [Troubleshooting](#troubleshooting)
9. [Rollback Procedures](#rollback-procedures)

---

## Prerequisites

### System Requirements

**Operating System**: Linux, macOS, or Windows with WSL2  
**Rust**: 1.70 or higher  
**Node.js**: 18.0 or higher  
**Disk Space**: 5 GB minimum (for full build chain)  
**Network**: Outbound HTTPS to Stellar RPC nodes

### Software Dependencies

```bash
# Install Rust and add wasm target
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup target add wasm32-unknown-unknown

# Install Soroban CLI
cargo install soroban-cli --locked

# Verify Soroban CLI
soroban --version  # Should output version info
```

### Stellar Account Setup

You need two Stellar accounts:

1. **Deployer Account** (`STELLAR_ACCOUNT`)
   - Has funds to pay transaction fees
   - Will be contract owner
   - Stores deployment transaction hash

2. **Admin Account** (`ADMIN_ADDRESS`)
   - Can initialize contract parameters
   - Can manage admin functions
   - Different from deployer (recommended)

### Generate Testnet Accounts

```bash
# Generate keypair for deployer
soroban keys generate --name deployer

# Generate keypair for admin
soroban keys generate --name admin

# Get public addresses
soroban keys address deployer
soroban keys address admin

# Fund testnet accounts (use Stellar faucet)
curl "https://friendbot.stellar.org?addr=$(soroban keys address deployer)"
curl "https://friendbot.stellar.org?addr=$(soroban keys address admin)"
```

---

## Environment Setup

### 1. Configure Environment Variables

Create a `.env` file in the project root (DO NOT commit this):

```bash
# .env (never commit!)

# Stellar Network Configuration
export NETWORK="testnet"
export RPC_URL="https://soroban-testnet.stellar.org"

# Account Credentials (use key names or full keys)
export STELLAR_ACCOUNT="deployer"    # Key name or G... public address
export ADMIN_ADDRESS="admin"         # Key name or G... public address

# Contract Deployment
export CONTRACT_ID=""  # Will be populated after deployment

# Optional: Development
export LOG_LEVEL="info"
export DEBUG="0"
```

### 2. Load Environment Variables

```bash
# Load configuration
source .env

# Verify configuration
echo "Network: $NETWORK"
echo "RPC: $RPC_URL"
echo "Deployer: $STELLAR_ACCOUNT"
```

### 3. Secure Key Management

**For Testnet** (less sensitive):
```bash
# Use soroban CLI key management
soroban keys import --secret-key 'your-secret-key' --name deployer
```

**For Mainnet** (production):
```bash
# Use hardware wallet or secure key vault
# Never store secrets in .env files or version control
# Use environment variable injected at deployment time

# Option 1: AWS Secrets Manager
aws secretsmanager get-secret-value --secret-id stellar-mainnet-key

# Option 2: HashiCorp Vault
vault kv get secret/stellar/mainnet

# Option 3: Environment variables from CI/CD secrets
# Configure in GitHub Actions, GitLab CI, or equivalent
```

---

## Building for Deployment

### 1. Verify Project Structure

```bash
# Ensure all required files exist
ls -la contracts/flux_engine/src/
# Should show: lib.rs, types.rs, test.rs

ls -la frontend/app/
# Should show: layout.tsx, page.tsx, globals.css

ls -la scripts/
# Should show: deploy.sh, build.sh, test.sh
```

### 2. Clean Previous Builds

```bash
# Remove old artifacts
make clean

# Verify clean state
ls -la contracts/flux_engine/target/
# Should be empty or minimal
```

### 3. Build Contract

```bash
# Build optimized WASM binary
make build-contracts

# Verify WASM was generated
ls -la contracts/flux_engine/target/wasm32-unknown-unknown/release/
# Should show: flux_engine.wasm (~200-400 KB)
```

### 4. Build Frontend (Optional for Testnet)

```bash
# Build production frontend
make build-frontend

# Verify build
ls -la frontend/.next/
# Should contain compiled static files
```

### 5. Run Tests Before Deployment

```bash
# Run full test suite
make test

# Expected output:
# Running contract tests...
# Running frontend tests...
# All tests completed

# Verify all tests pass
# If tests fail, do NOT proceed to deployment
```

---

## Testnet Deployment

### Step 1: Prepare Deployment Configuration

```bash
# Set testnet environment
export NETWORK="testnet"
export RPC_URL="https://soroban-testnet.stellar.org"
export STELLAR_ACCOUNT="deployer"
export ADMIN_ADDRESS="admin"

# Verify accounts have funding
soroban account balance --account $STELLAR_ACCOUNT
# Should show: at least 50 XLM for fees

soroban account balance --account $ADMIN_ADDRESS
# Should show: at least 1 XLM for initialization
```

### Step 2: Deploy Smart Contract

```bash
# Run deployment script
./scripts/deploy.sh testnet

# Expected output:
# Deploying to testnet...
# Building WASM...
# Installing contract...
# Initializing contract...
# Deployment successful!
# Contract ID: C...

# Save the contract ID
export CONTRACT_ID="C..."
```

### Step 3: Verify Deployment

```bash
# Check contract was deployed
soroban contract inspect --id $CONTRACT_ID

# Should output contract info:
# Specification version: 20
# Instance storage entries: 0
# Persistent storage entries: 0
# Temporary storage entries: 0
```

### Step 4: Initialize Contract

```bash
# Set initialization parameters
BATCH_SIZE="100"
MIN_SETTLEMENT="1000000"      # 0.1 tokens (fixed-point)
MAX_SETTLEMENT="10000000000"  # 1000 tokens (fixed-point)
FEE_BPS="250"                 # 2.5% fee

# Initialize contract
soroban contract invoke \
  --id $CONTRACT_ID \
  --source-account $STELLAR_ACCOUNT \
  -- initialize \
  --admin $ADMIN_ADDRESS \
  --batch_size $BATCH_SIZE \
  --min_settlement $MIN_SETTLEMENT \
  --max_settlement $MAX_SETTLEMENT \
  --fee_bps $FEE_BPS

# Expected: Transaction successful output
```

### Step 5: Verify Initialization

```bash
# Query contract config
soroban contract invoke \
  --id $CONTRACT_ID \
  --source-account $STELLAR_ACCOUNT \
  -- config

# Expected output should show your configuration values
```

---

## Verification Procedures

### 1. Contract Code Verification

```bash
# Get deployed contract code hash
soroban contract info \
  --id $CONTRACT_ID | grep "code_hash"

# Get local build code hash
soroban contract info \
  --file contracts/flux_engine/target/wasm32-unknown-unknown/release/flux_engine.wasm \
  | grep "hash"

# Verify hashes match
# If hashes differ, code was modified after deployment
```

### 2. Configuration Verification

```bash
# Query and verify configuration
RESULT=$(soroban contract invoke \
  --id $CONTRACT_ID \
  --source-account $STELLAR_ACCOUNT \
  -- config)

# Parse and verify values
echo "Config: $RESULT"
# Should show: batch_size, min_settlement, max_settlement, fee_bps
```

### 3. Test Basic Operations

#### Create a Test Stream

```bash
# Get test token address (or use existing token)
TOKEN_ADDRESS="C..."  # Standard Payment Network token

# Get test accounts
SENDER="G..."
RECIPIENT="G..."

# Create test stream (30 days from now)
START_TIME=$(date +%s)
END_TIME=$((START_TIME + 30 * 86400))  # 30 days
AMOUNT="1000000000"  # 100 tokens (fixed-point)

soroban contract invoke \
  --id $CONTRACT_ID \
  --source-account $SENDER \
  -- create_stream \
  --token $TOKEN_ADDRESS \
  --recipient $RECIPIENT \
  --amount $AMOUNT \
  --start_time $START_TIME \
  --end_time $END_TIME

# Expected: Stream ID returned
```

#### Query Stream

```bash
# Get stream details
STREAM_ID="1"

soroban contract invoke \
  --id $CONTRACT_ID \
  --source-account $SENDER \
  -- get_stream \
  --stream_id $STREAM_ID

# Expected: Stream details returned
```

#### Verify Claimable Amount

```bash
# Query how much is claimable
soroban contract invoke \
  --id $CONTRACT_ID \
  --source-account $RECIPIENT \
  -- get_claimable \
  --stream_id $STREAM_ID

# Expected: Amount in fixed-point (should be > 0 if time has passed)
```

### 4. Event Verification

```bash
# Get contract events from last hour
soroban events \
  --contract-id $CONTRACT_ID \
  --start-ledger 0

# Should show transfer events if any transactions occurred
```

---

## Mainnet Deployment

### ⚠️ Production Deployment Checklist

Before deploying to mainnet, verify:

- [ ] All tests pass on testnet deployment
- [ ] Contract code has been reviewed
- [ ] Security audit completed (if applicable)
- [ ] Admin and deployer accounts are secured
- [ ] Mainnet accounts have sufficient XLM for fees
- [ ] Deployment procedure has been tested on testnet
- [ ] Monitoring and alerting configured
- [ ] Incident response plan documented
- [ ] Key management strategy in place

### Mainnet Deployment Procedure

```bash
# 1. Switch to mainnet environment
export NETWORK="mainnet"
export RPC_URL="https://soroban-mainnet.stellar.org"

# 2. Verify mainnet accounts have funding
soroban account balance --account $STELLAR_ACCOUNT --rpc-url $RPC_URL
# Should have: at least 100 XLM for fees and reserves

# 3. Deploy contract
./scripts/deploy.sh mainnet

# 4. Verify mainnet deployment
soroban contract info \
  --id $CONTRACT_ID \
  --rpc-url $RPC_URL

# 5. Initialize with production parameters
soroban contract invoke \
  --id $CONTRACT_ID \
  --source-account $STELLAR_ACCOUNT \
  --rpc-url $RPC_URL \
  -- initialize \
  --admin $ADMIN_ADDRESS \
  --batch_size "1000" \
  --min_settlement "100000" \
  --max_settlement "100000000000" \
  --fee_bps "200"

# 6. Announce contract address to users
echo "Mainnet Contract ID: $CONTRACT_ID"
```

---

## Monitoring & Health Checks

### 1. Automated Health Checks

```bash
#!/bin/bash
# health_check.sh

CONTRACT_ID="$1"
NETWORK="${2:-testnet}"

if [ -z "$CONTRACT_ID" ]; then
  echo "Usage: ./health_check.sh <CONTRACT_ID> [testnet|mainnet]"
  exit 1
fi

RPC_URL="https://soroban-${NETWORK}.stellar.org"

# Check 1: Contract exists
echo "Checking contract existence..."
soroban contract info --id $CONTRACT_ID --rpc-url $RPC_URL > /dev/null
if [ $? -eq 0 ]; then
  echo "✓ Contract exists"
else
  echo "✗ Contract not found"
  exit 1
fi

# Check 2: Can query config
echo "Checking contract functionality..."
CONFIG=$(soroban contract invoke \
  --id $CONTRACT_ID \
  --rpc-url $RPC_URL \
  -- config 2>/dev/null)

if [ $? -eq 0 ]; then
  echo "✓ Contract is functional"
  echo "  Config: $CONFIG"
else
  echo "✗ Contract queries failing"
  exit 1
fi

# Check 3: Recent events
echo "Checking recent activity..."
EVENTS=$(soroban events --contract-id $CONTRACT_ID --rpc-url $RPC_URL | wc -l)
echo "✓ Recent events: $EVENTS"

echo ""
echo "Health check completed successfully!"
```

Run health checks:

```bash
# Make script executable
chmod +x health_check.sh

# Run periodic health checks
./health_check.sh $CONTRACT_ID testnet
```

### 2. Performance Monitoring

```bash
# Monitor contract invocation times
soroban contract invoke \
  --id $CONTRACT_ID \
  --source-account $STELLAR_ACCOUNT \
  -- config | tee -a monitoring.log

# Track transaction costs
grep "cost" monitoring.log | tail -10
```

### 3. Event Monitoring

```bash
# Stream live contract events
while true; do
  soroban events \
    --contract-id $CONTRACT_ID \
    --tail
  sleep 30
done
```

---

## Troubleshooting

### Problem: Deployment Fails with "Insufficient Balance"

**Symptoms**: Deployment script exits with account balance error

**Solution**:
```bash
# Check account balance
soroban account balance --account $STELLAR_ACCOUNT

# If testnet: Use faucet
curl "https://friendbot.stellar.org?addr=$STELLAR_ACCOUNT"

# If mainnet: Send XLM to account from another account
soroban account payment \
  --from-account $SOURCE_ACCOUNT \
  --to-account $STELLAR_ACCOUNT \
  --amount 100
```

### Problem: Contract Initialization Fails

**Symptoms**: Initialize transaction fails with error

**Solution**:
```bash
# Verify configuration parameters
echo "Min: $MIN_SETTLEMENT"
echo "Max: $MAX_SETTLEMENT"

# Ensure min < max
if [ $MIN_SETTLEMENT -ge $MAX_SETTLEMENT ]; then
  echo "ERROR: min_settlement must be less than max_settlement"
  exit 1
fi

# Verify fee is 0-10000 bps
if [ $FEE_BPS -gt 10000 ]; then
  echo "ERROR: fee_bps must be 0-10000"
  exit 1
fi

# Retry with corrected parameters
./scripts/deploy.sh testnet
```

### Problem: Contract Invocations Timeout

**Symptoms**: Soroban CLI commands timeout or hang

**Solution**:
```bash
# Increase timeout
soroban contract invoke \
  --id $CONTRACT_ID \
  --source-account $STELLAR_ACCOUNT \
  --timeout 60 \  # Increase to 60 seconds
  -- config

# Check network connectivity
ping soroban-testnet.stellar.org

# Try different RPC endpoint
export RPC_URL="https://soroban-testnet.stellar.org"
```

### Problem: "Contract Not Found" Error

**Symptoms**: Soroban CLI reports contract doesn't exist

**Solution**:
```bash
# Verify contract ID format
echo $CONTRACT_ID  # Should start with 'C'

# Verify correct network
export RPC_URL="https://soroban-testnet.stellar.org"
export NETWORK="testnet"

# List deployed contracts
soroban contract list --rpc-url $RPC_URL

# If not listed, redeploy
./scripts/deploy.sh testnet
```

### Problem: Fixed-Point Math Errors

**Symptoms**: Amount calculations produce unexpected results

**Solution**:
```bash
# Verify amount is in fixed-point format (multiply by 10^7)
READABLE_AMOUNT="100"  # 100 tokens
FIXED_POINT=$((READABLE_AMOUNT * 10000000))
echo "Fixed-point: $FIXED_POINT"

# Use proper conversion when testing
MIN_SETTLEMENT=$((1 * 10000000))      # 1 token
MAX_SETTLEMENT=$((1000 * 10000000))   # 1000 tokens
```

---

## Rollback Procedures

### Scenario: Contract Bug Discovered

**If bug found and fix needed**:

1. **Pause Operations** (if critical)
   ```bash
   # Notify users to stop using contract
   # Update frontend to show maintenance message
   # (Current system doesn't have pause, consider future upgrade)
   ```

2. **Deploy Patch**
   ```bash
   # Fix contract code
   # Update version number in Cargo.toml
   # Run full test suite
   make test
   
   # Deploy new version with new contract ID
   ./scripts/deploy.sh testnet
   ```

3. **Migrate State** (if needed)
   ```bash
   # If user data must be preserved:
   # 1. Query all streams from old contract
   # 2. Recreate streams on new contract
   # 3. Notify users of new contract ID
   ```

4. **Verify Rollback**
   ```bash
   # Test all functionality on new contract
   ./health_check.sh $NEW_CONTRACT_ID testnet
   
   # Get user confirmation before mainnet migration
   ```

### Scenario: Need to Revert Mainnet Deployment

**If critical bug requires reverting**:

1. **Immediate Actions**
   - Disable frontend access to contract
   - Post incident notice
   - Don't allow new streams to be created

2. **Deploy Fixed Version**
   ```bash
   # Fix the bug in contract code
   make clean
   make build-contracts
   make test
   
   # Deploy new version
   ./scripts/deploy.sh mainnet
   export NEW_CONTRACT_ID="C..."
   ```

3. **User Communication**
   ```
   Incident: [Issue description]
   
   Old Contract: C...
   New Contract: C... (recommended)
   
   Action Required:
   - Update bookmarks to new contract ID
   - Existing streams remain on old contract (can still claim)
   - New streams should use new contract
   
   Timeline:
   - [date]: Bug discovered
   - [date]: Fix deployed
   - [date]: Migration complete
   ```

---

## Best Practices

### Pre-Deployment
- [ ] Run full test suite
- [ ] Review contract code changes
- [ ] Document deployment configuration
- [ ] Backup existing contract state
- [ ] Plan rollback procedure

### During Deployment
- [ ] Monitor transaction confirmations
- [ ] Verify each deployment step
- [ ] Keep deployment logs
- [ ] Have admin account ready for initialization

### Post-Deployment
- [ ] Run comprehensive health checks
- [ ] Monitor for 24+ hours
- [ ] Verify event emission
- [ ] Test user workflows
- [ ] Document deployment results

### Production Operations
- [ ] Set up automated monitoring
- [ ] Configure alerts for errors
- [ ] Establish escalation procedures
- [ ] Schedule regular health checks
- [ ] Maintain deployment runbook

---

## Support

For deployment issues:

1. Check [Troubleshooting](#troubleshooting) section
2. Review [SECURITY.md](./SECURITY.md) for security considerations
3. Consult [ARCHITECTURE.md](./ARCHITECTURE.md) for system design
4. Check contract source code at `contracts/flux_engine/src/`

---

**Last Updated**: 2024  
**Status**: Production Ready  
**Version**: 1.0
