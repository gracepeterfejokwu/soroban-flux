# Security Guidelines for Soroban Flux

## Overview

This document outlines the security model, threat model, and best practices for Soroban Flux.

## Threat Model

### 1. Authorization Attacks
**Threat**: Unauthorized account transfers or configuration changes

**Mitigation**:
- `require_auth()` on all state-changing operations
- Admin-only initialization
- Signed transactions enforced by Soroban runtime
- Per-account authorization scope

### 2. Arithmetic Attacks
**Threat**: Integer overflow/underflow in balance calculations

**Mitigation**:
- Fixed-point arithmetic with base 10^7
- Checked arithmetic for multiplication/division
- Saturating arithmetic for subtraction
- Comprehensive tests for boundary values
- MAX_SAFE_VALUE constraint (u128::MAX / 10^7)

### 3. Reentrancy
**Threat**: State inconsistency from nested contract calls

**Mitigation**:
- Strict Checks-Effects-Interactions pattern
- State updates before external interactions
- No delegated call patterns
- Atomic balance updates (both or neither)

### 4. Data Integrity
**Threat**: Corrupted state from concurrent updates

**Mitigation**:
- Ledger-based concurrency model (no multi-threaded access)
- Atomic transactions at Stellar level
- TTL-based state cleanup
- Deterministic state derivation

### 5. Denial of Service (DoS)
**Threat**: Resource exhaustion or transaction failures

**Mitigation**:
- Settlement batching (reduces transaction frequency)
- Input validation (prevents invalid operations)
- TTL management (prevents unlimited state growth)
- Reasonable fee bounds (0-10000 bps)

### 6. Front-Running
**Threat**: Transaction order manipulation

**Mitigation**:
- Batch settlements (multiple accounts together)
- Ledger sequence visibility
- Deterministic pricing (fixed fee percentages)
- Stellar's transaction ordering guarantees

## Security Best Practices

### For Contract Developers

1. **Authorization**
```rust
// CORRECT: Require authorization
pub fn transfer(env: Env, from: Address, to: Address, amount: u128) {
    from.require_auth();  // ✓ Must auth sender
    // ... perform transfer
}

// WRONG: Missing authorization
pub fn transfer(env: Env, from: Address, to: Address, amount: u128) {
    // ✗ No authorization check!
}
```

2. **Arithmetic**
```rust
// CORRECT: Use checked arithmetic
let new_balance = from_balance
    .checked_sub(amount)
    .ok_or_else(|| Symbol::short("insufficient"))?;

// WRONG: Unchecked arithmetic (vulnerable to underflow)
let new_balance = from_balance - amount;  // ✗ Could wrap around
```

3. **State Updates**
```rust
// CORRECT: Checks-Effects-Interactions
// 1. Checks: Validate preconditions
if amount > max_settlement {
    return Err(Symbol::short("too_large"));
}

// 2. Effects: Update state
Self::set_balance(&env, &from, new_balance)?;
Self::set_balance(&env, &to, to_balance + amount)?;

// 3. Interactions: External calls (future)
// emit_event(...)?;

// WRONG: Interactions before Effects
emit_event(...)?;  // ✗ Called before state updates
Self::set_balance(&env, &from, new_balance)?;
```

### For Frontend Developers

1. **Input Validation**
```typescript
// CORRECT: Validate before submission
function validateTransfer(amount: bigint, maxAmount: bigint): boolean {
  if (amount <= 0n) return false;
  if (amount > maxAmount) return false;
  return true;
}

// WRONG: Trust user input
function transfer(amount: string) {
  const bigintAmount = BigInt(amount);  // ✗ No validation
  // ... submit
}
```

2. **Error Handling**
```typescript
// CORRECT: Handle all error cases
try {
  const result = await contractCall();
} catch (error) {
  if (error instanceof AuthorizationError) {
    showAuthError();
  } else if (error instanceof ContractError) {
    showContractError(error.message);
  } else {
    showGenericError();
  }
}

// WRONG: Ignore errors
const result = await contractCall();  // ✗ No error handling
```

3. **Data Sanitization**
```typescript
// CORRECT: Escape and validate displayed data
function displayAddress(address: string): string {
  if (!isValidStellarAddress(address)) {
    throw new Error("Invalid address");
  }
  return escapeHtml(address);
}

// WRONG: Display untrusted data
function displayAddress(address: string): JSX.Element {
  return <div>{address}</div>;  // ✗ Potential XSS
}
```

### For Deployers

1. **Environment Configuration**
```bash
# CORRECT: Secure environment variables
export STELLAR_ACCOUNT="G..."
export ADMIN_ADDRESS="G..."
export NETWORK="testnet"
# Never commit .env files

# WRONG: Hardcoded secrets
STELLAR_ACCOUNT="G..." npm run deploy  # ✗ Exposed in bash history
```

2. **Deployment Verification**
```bash
# CORRECT: Verify contract after deployment
soroban contract info --id <contract-id>
# Verify code hash matches built binary
```

3. **Network Selection**
```bash
# CORRECT: Separate test and production deployments
make deploy NETWORK=testnet  # Test deployment
make deploy NETWORK=mainnet  # Production deployment

# WRONG: Single deployment target
./deploy.sh  # ✗ Unclear which network
```

## Audit Checklist

### Pre-Deployment

- [ ] All `require_auth()` calls present for state changes
- [ ] No unchecked arithmetic operations
- [ ] Checks-Effects-Interactions pattern enforced
- [ ] All error cases return proper error symbols
- [ ] TTL values configured appropriately
- [ ] Fee bounds validated (0-10000 bps)
- [ ] Address validation (no self-transfers)
- [ ] Amount validation (within min/max range)

### Testing

- [ ] Unit tests for arithmetic edge cases
- [ ] Authorization tests (success and failure)
- [ ] Reentrancy tests (if applicable)
- [ ] Balance consistency tests
- [ ] Integration tests with Stellar SDK
- [ ] Fuzzing with randomized inputs

### Code Review

- [ ] No unsafe operations
- [ ] Proper error handling
- [ ] Documentation of security assumptions
- [ ] No hardcoded addresses or values
- [ ] Compliance with Soroban best practices

### Production

- [ ] Contract verified on explorer
- [ ] Admin key stored securely
- [ ] Monitoring and alerting configured
- [ ] Incident response plan documented
- [ ] Upgrade procedure defined

## Known Limitations

1. **Single-Token**: Currently only supports one token type
2. **Manual Settlement**: Batches are created but require manual trigger
3. **No Slashing**: No penalty mechanism for malicious behavior
4. **Static Fees**: Fee percentages cannot be dynamically adjusted
5. **TTL Clearing**: Old state requires manual intervention

## Disclosure Policy

For security vulnerabilities, please follow responsible disclosure:

1. Do NOT publicly disclose the vulnerability
2. Email security@example.com with details
3. Allow 90 days for fix and deployment
4. Coordinate public disclosure date

## References

- [Soroban Security Best Practices](https://soroban.stellar.org/docs/learn/security)
- [Stellar Developer Documentation](https://developers.stellar.org/)
- [OWASP Smart Contract Security](https://cheatsheetseries.owasp.org/)
- [CWE Top 25 Most Dangerous Software Weaknesses](https://cwe.mitre.org/top25/)
