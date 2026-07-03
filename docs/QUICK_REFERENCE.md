# Soroban Flux - Quick Reference

One-page reference guide for Soroban Flux constants, errors, and common operations.

---

## Constants

### Fixed-Point Math

```rust
FIXED_POINT_BASE: u128 = 10_000_000      // 7 decimal places
MIN_AMOUNT: u128 = 1_000_000             // 0.1 tokens
MAX_AMOUNT: u128 = 1_000_000_000_000     // 100M tokens
```

### Time Values

```rust
SECONDS_PER_DAY: u64 = 86_400            // 24 hours
SECONDS_PER_MONTH: u64 = 2_592_000       // 30 days
SECONDS_PER_YEAR: u64 = 31_536_000       // 365 days
```

### TTL Management

```rust
TTL_LEDGER_ENTRIES: u32 = 1_000_000      // ~4.6 days at 6s/ledger
MIN_TTL: u32 = 100_000                   // ~0.7 days
MAX_TTL: u32 = 5_000_000                 // ~23 days
```

### Fee Configuration

```rust
MIN_FEE_BPS: u16 = 0                      // 0% fee
MAX_FEE_BPS: u16 = 10_000                 // 100% fee
TYPICAL_FEE_BPS: u16 = 200                // 2% fee
```

---

## Error Codes & Meanings

| Error Code | Meaning | Cause | Recovery |
|-----------|---------|-------|----------|
| `stream_not_found` | Stream ID invalid | Stream doesn't exist | Use valid stream ID |
| `stream_ended` | Stream past end time | Can't claim after end | Check stream times |
| `invalid_amount` | Amount out of range | Too small or too large | Use amount in valid range |
| `insufficient_balance` | Not enough balance | Balance too low | Top up account |
| `unauthorized` | Authorization failed | Missing signature | Sign transaction |
| `overflow` | Math overflow | Result too large | Use smaller amounts |
| `div_zero` | Division by zero | Invalid calculation | Check denominator |
| `already_initialized` | Contract already init | Can only init once | Skip init |
| `not_initialized` | Contract not init | Must init first | Call initialize() |
| `invalid_times` | Time validation failed | start >= end | Ensure start < end |

---

## Fixed-Point Conversion

### Convert Readable to Fixed-Point

```
Multiply by 10,000,000 (10^7)

Examples:
1 token = 1 × 10^7 = 10,000,000
0.5 tokens = 0.5 × 10^7 = 5,000,000
0.1 tokens = 0.1 × 10^7 = 1,000,000
```

### Convert Fixed-Point to Readable

```
Divide by 10,000,000 (10^7)

Examples:
10,000,000 ÷ 10^7 = 1 token
5,000,000 ÷ 10^7 = 0.5 tokens
1,000,000 ÷ 10^7 = 0.1 tokens
```

### Quick Reference Table

| Readable | Fixed-Point |
|----------|------------|
| 0.01 | 100,000 |
| 0.1 | 1,000,000 |
| 1.0 | 10,000,000 |
| 10.0 | 100,000,000 |
| 100.0 | 1,000,000,000 |
| 1,000.0 | 10,000,000,000 |

---

## Common Operations

### Create Stream

```typescript
// TypeScript/JavaScript
const streamId = await contract.call('create_stream', [
  tokenAddress,           // Token contract address
  recipientAddress,       // Who receives tokens
  100_000_000,           // Amount (1 token in fixed-point)
  Date.now() / 1000,     // Start time (now)
  Date.now() / 1000 + 86400 * 30,  // End time (30 days from now)
]);
```

```rust
// Rust
let stream_id = flux.create_stream(
    &env,
    &token_address,
    &recipient,
    &100_000_000,  // 1 token
    &start_time,
    &end_time,
)?;
```

### Check Claimable

```typescript
// TypeScript/JavaScript
const claimable = await contract.call('get_claimable', [streamId]);
console.log(`Can claim: ${Number(claimable) / 10_000_000} tokens`);
```

```rust
// Rust
let claimable = flux.get_claimable(&env, stream_id)?;
println!("Claimable: {}", claimable / 10_000_000);
```

### Claim Tokens

```typescript
// TypeScript/JavaScript
const claimed = await contract.call('claim_stream', [
  recipientAddress,
  streamId,
]);
```

```rust
// Rust
let claimed = flux.claim_stream(&env, &recipient, stream_id)?;
```

### Cancel Stream

```typescript
// TypeScript/JavaScript
const [claimed, refunded] = await contract.call('cancel_stream', [streamId]);
console.log(`Claimed: ${claimed}, Refunded: ${refunded}`);
```

```rust
// Rust
let (claimed, refunded) = flux.cancel_stream(&env, stream_id)?;
```

### Extend TTL

```typescript
// TypeScript/JavaScript
const newTtl = await contract.call('extend_ttl', [streamId]);
console.log(`New TTL: ${newTtl} ledgers`);
```

```rust
// Rust
let new_ttl = flux.extend_ttl(&env, stream_id)?;
```

### Get Balance

```typescript
// TypeScript/JavaScript
const balance = await contract.call('balance', [tokenAddress, accountAddress]);
console.log(`Balance: ${Number(balance) / 10_000_000} tokens`);
```

```rust
// Rust
let balance = flux.balance(&env, &token, &account)?;
```

---

## Build & Deployment Commands

```bash
# Install dependencies
make install

# Build contract
make build-contracts

# Build frontend
make build-frontend

# Run all tests
make test

# Deploy to testnet
make deploy NETWORK=testnet

# Deploy to mainnet
make deploy NETWORK=mainnet

# Clean build
make clean

# Run linter
make lint

# Show help
make help
```

---

## Network Configuration

### Testnet

```
Network: soroban-testnet
RPC: https://soroban-testnet.stellar.org
Faucet: https://friendbot.stellar.org
Explorer: https://testnet.sorobanexplorer.com
```

### Mainnet

```
Network: soroban-mainnet
RPC: https://soroban-mainnet.stellar.org
Faucet: N/A (real funds required)
Explorer: https://sorobanexplorer.com
```

---

## File Locations

```
Contract:        contracts/flux_engine/src/lib.rs
Tests:           contracts/flux_engine/src/test.rs
Types:           contracts/flux_engine/src/types.rs
Frontend:        frontend/app/page.tsx
Components:      frontend/components/
Documentation:   docs/
Scripts:         scripts/
```

---

## API Functions Quick Index

| Function | Purpose | Auth | Returns |
|----------|---------|------|---------|
| `initialize()` | Setup contract | Admin | `()` |
| `create_stream()` | Create stream | Sender | `stream_id` |
| `claim_stream()` | Claim tokens | Recipient | `amount` |
| `cancel_stream()` | Cancel stream | Sender | `(claimed, refunded)` |
| `extend_ttl()` | Extend TTL | Any | `new_ttl` |
| `get_stream()` | Query stream | None | `stream_data` |
| `get_claimable()` | Get claimable | None | `amount` |
| `balance()` | Get balance | None | `balance` |

---

## Math Examples

### Calculate Claimable (30-day stream)

```
Stream: 1000 tokens over 30 days

Day 5:  (1000 × 5) / 30 = 166.67 tokens ✓
Day 10: (1000 × 10) / 30 = 333.33 tokens ✓
Day 15: (1000 × 15) / 30 = 500.00 tokens ✓
Day 20: (1000 × 20) / 30 = 666.67 tokens ✓
Day 30: (1000 × 30) / 30 = 1000.00 tokens ✓
```

### Fixed-Point Precision

```
Base: 10,000,000

Smallest: 0.0000001 tokens
Largest: 3.4 × 10^12 tokens

Precision: Exact (no rounding errors)
```

---

## Storage Requirements

```
Per Stream: ~500 bytes
Per Account: ~200 bytes
Per Event: ~150 bytes

Examples:
1,000 streams: ~500 KB
10,000 streams: ~5 MB
100,000 streams: ~50 MB
```

---

## Gas Considerations

```
Typical Gas Costs (Testnet):

initialize():      ~1000 gas
create_stream():   ~2000 gas
claim_stream():    ~2000 gas
cancel_stream():   ~2000 gas
extend_ttl():      ~500 gas
get_claimable():   ~100 gas
get_stream():      ~100 gas
balance():         ~100 gas
```

---

## Performance Baseline

```
Function | Time | Gas | Notes
---------|------|-----|-------
create_stream | <200ms | ~2000 | Main state change
claim_stream | <200ms | ~2000 | Atomic update
get_claimable | <100ms | ~100 | Read-only
balance | <50ms | ~50 | Simple lookup
```

---

## Troubleshooting Quick Tips

| Issue | Cause | Fix |
|-------|-------|-----|
| "not authorized" | Missing signature | Sign with account |
| "overflow" | Amount too large | Use smaller value |
| "invalid_amount" | Outside range | Check min/max |
| "stream_not_found" | Invalid stream ID | Verify stream exists |
| "insufficient_balance" | Balance too low | Top up account |
| "stream_ended" | Claiming after end | Check stream times |

---

## Useful Links

- [Full API Reference](./API.md)
- [Architecture Guide](./ARCHITECTURE.md)
- [Deployment Guide](./DEPLOYMENT_GUIDE.md)
- [Integration Examples](./INTEGRATION_GUIDE.md)
- [Security Guidelines](./SECURITY.md)
- [Stellar Docs](https://developers.stellar.org/)
- [Soroban Docs](https://soroban.stellar.org/)

## Visual References

### Stream Flow Diagram
![Stream Flow](../assets/readMe%201.png)

Shows the three-state token progression and calculation formula.

### Dashboard Visualization
![Vault Balance Chart](../assets/readMe%202.PNG)

Real-time balance tracking and stream parameters visualization.

---

**Last Updated**: 2024  
**Version**: 1.0