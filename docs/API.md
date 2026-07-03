# Soroban Flux - Streaming API Reference

Complete API documentation for Soroban Flux streaming payment protocol.

---

## Table of Contents

1. [Contract Functions](#contract-functions)
2. [Data Types](#data-types)
3. [Error Codes](#error-codes)
4. [Frontend API](#frontend-api)
5. [Integration Examples](#integration-examples)
6. [Event Schema](#event-schema)
7. [Testing & Simulation](#testing--simulation)

---

## Contract Functions

### Core Streaming Functions

#### `initialize`

Initialize the Flux contract with admin account.

**Authorization**: None (first call)

**Parameters**:
- None

**Returns**: `Result<(), Symbol>`

**Errors**:
- `"already_initialized"` - Contract was already initialized

**Effects**:
- Sets up contract state
- Ready for stream creation

**Example**:
```rust
flux.initialize(&env)?;
```

---

#### `create_stream`

Create a new time-locked token stream.

**Authorization**: Requires sender signature

**Parameters**:
- `token: Address` - Token contract address
- `recipient: Address` - Token recipient address
- `amount: u128` - Total stream amount (fixed-point, 7 decimals)
- `start_time: u64` - Stream start time (Unix timestamp in seconds)
- `end_time: u64` - Stream end time (Unix timestamp in seconds)

**Returns**: `Result<u64, Symbol>` - Stream ID

**Errors**:
- `"invalid_amount"` - Amount is zero or too large
- `"invalid_times"` - start_time >= end_time
- `"same_addr"` - Recipient is sender (self-stream not allowed)
- `"overflow"` - Stream ID counter overflow

**Preconditions**:
- `amount > 0`
- `start_time < end_time`
- `recipient != sender`
- Sender has sufficient token balance

**Effects**:
- Creates new stream entry in persistent storage
- Sets TTL for automatic cleanup (~4.6 days)
- Emits stream creation event
- Increments stream counter

**Claimable Formula**:
```
elapsed_seconds = current_time - start_time
stream_duration = end_time - start_time
claimable = (amount × elapsed_seconds) / stream_duration

Constraints:
- claimable ≥ 0 (before start)
- claimable ≤ amount (after end)
```

**Example**:
```rust
// Create 1000 token stream over 30 days
let stream_id = flux.create_stream(
    &env,
    &token_address,
    &recipient,
    &1_000_000_000,  // 1000 tokens (fixed-point)
    &start_time,
    &start_time + 30 * 86400,  // 30 days
)?;
```

**Real Scenario**:
```
create_stream(
  token: "native stablecoin",
  recipient: "alice@drips.com",
  amount: 1_000_000_000,  // $1000
  start_time: 1704067200,  // Jan 1, 2024
  end_time: 1706745600,    // Feb 1, 2024 (30 days)
)

Result: stream_id = 1

Timeline:
├─ Jan 1 00:00: Stream created, 0% unlocked
├─ Jan 11 00:00: 33% unlocked, can claim ~$333
├─ Jan 21 00:00: 67% unlocked, can claim ~$667
└─ Feb 1 00:00: 100% unlocked, can claim remaining
```

---

#### `claim_stream`

Claim earned tokens from a stream.

**Authorization**: Requires recipient signature

**Parameters**:
- `recipient: Address` - Recipient claiming tokens
- `stream_id: u64` - Stream ID to claim from

**Returns**: `Result<u128, Symbol>` - Amount claimed (fixed-point)

**Errors**:
- `"stream_not_found"` - Stream ID doesn't exist
- `"unauthorized"` - Caller is not the stream recipient
- `"overflow"` - Claim amount overflow
- `"no_balance"` - No claimable tokens available

**Preconditions**:
- Stream must exist
- Caller must be the recipient
- Claimable amount must be > 0

**Effects**:
- Transfers claimable tokens to recipient
- Updates stream's claimed amount
- Emits claim event
- Renews stream TTL

**Example**:
```rust
// Recipient claims earned tokens
let claimed = flux.claim_stream(&env, &recipient, stream_id)?;
println!("Claimed: {} tokens", claimed / 10_000_000);
```

**Workflow**:
```
1. Check if time has passed
2. Calculate claimable: (amount × elapsed) / duration
3. Subtract previous claims
4. Transfer claimable to recipient
5. Update stream state
6. Return amount claimed
```

---

#### `cancel_stream`

Cancel a stream and refund unclaimed tokens to sender.

**Authorization**: Requires sender signature

**Parameters**:
- `stream_id: u64` - Stream ID to cancel

**Returns**: `Result<(u128, u128), Symbol>` - Tuple of (claimed, refunded) amounts

**Errors**:
- `"stream_not_found"` - Stream doesn't exist
- `"unauthorized"` - Caller is not the stream sender
- `"already_cancelled"` - Stream already cancelled

**Preconditions**:
- Stream must exist
- Caller must be the sender
- Stream must not already be cancelled

**Effects**:
- Marks stream as cancelled
- Transfers unclaimed tokens back to sender
- Emits cancellation event
- Removes stream from storage (TTL expires)

**Math Verification**:
```
claimed + refunded = original_amount
```

**Example**:
```rust
// Sender cancels stream early
let (claimed, refunded) = flux.cancel_stream(&env, stream_id)?;
println!("Claimed: {}, Refunded: {}", claimed, refunded);
```

**Scenario**:
```
Original stream: 1000 tokens, 30 days
Day 20: Sender cancels

Calculations:
├─ Elapsed: 20 days (20/30 = 66.67%)
├─ Claimable: 1000 × 0.6667 = 666.67 tokens
├─ Claimed by recipient: 500 tokens (from previous claims)
├─ Refunded to sender: 1000 - 500 = 500 tokens
└─ Verification: 500 + 500 = 1000 ✓
```

---

#### `extend_ttl`

Extend the TTL of a stream to prevent automatic cleanup.

**Authorization**: Requires sender or recipient signature

**Parameters**:
- `stream_id: u64` - Stream ID to extend

**Returns**: `Result<u32, Symbol>` - New TTL in ledgers

**Errors**:
- `"stream_not_found"` - Stream doesn't exist
- `"unauthorized"` - Caller is not sender or recipient

**Preconditions**:
- Stream must exist

**Effects**:
- Resets stream TTL to ~1,000,000 ledgers (~4.6 days)
- Emits TTL extension event

**Example**:
```rust
// Extend stream lifetime
let new_ttl = flux.extend_ttl(&env, stream_id)?;
println!("New TTL: {} ledgers (~{} days)", new_ttl, new_ttl / 250000);
```

---

### Query Functions

#### `get_stream`

Get detailed information about a stream.

**Authorization**: None (read-only)

**Parameters**:
- `stream_id: u64` - Stream ID to query

**Returns**: `Result<StreamData, Symbol>`

**Errors**:
- `"stream_not_found"` - Stream doesn't exist

**StreamData Structure**:
```rust
(
  sender: Address,           // Stream creator
  recipient: Address,        // Token recipient
  token: Address,           // Token contract
  amount: u128,             // Total stream amount
  start_time: u64,          // Stream start (Unix timestamp)
  end_time: u64,            // Stream end (Unix timestamp)
  claimed: u128,            // Tokens already claimed
)
```

**Example**:
```rust
let (sender, recipient, token, amount, start, end, claimed) = 
    flux.get_stream(&env, stream_id)?;
```

---

#### `get_claimable`

Calculate how many tokens are currently claimable from a stream.

**Authorization**: None (read-only)

**Parameters**:
- `stream_id: u64` - Stream ID to query

**Returns**: `Result<u128, Symbol>` - Claimable amount (fixed-point)

**Errors**:
- `"stream_not_found"` - Stream doesn't exist

**Calculation**:
```
current_time = now
elapsed = max(0, current_time - start_time)
duration = end_time - start_time
total_unlocked = (amount × elapsed) / duration
claimable = max(0, total_unlocked - claimed)
```

**Example**:
```rust
let claimable = flux.get_claimable(&env, stream_id)?;
if claimable > 0 {
    flux.claim_stream(&env, &recipient, stream_id)?;
}
```

**Example Output**:
```
Stream: 1000 tokens over 30 days
Time: Day 10 (10/30 = 33.33% elapsed)

Calculation:
├─ Total unlocked: 1000 × (10/30) = 333.33
├─ Previously claimed: 200
└─ Claimable now: 333.33 - 200 = 133.33 tokens
```

---

#### `balance`

Get token balance for an account.

**Authorization**: None (read-only)

**Parameters**:
- `token: Address` - Token contract address
- `account: Address` - Account to query

**Returns**: `Result<u128, Symbol>` - Balance (fixed-point)

**Errors**: None (returns 0 if account doesn't exist)

**Example**:
```rust
let balance = flux.balance(&env, &token, &account)?;
println!("Balance: {} tokens", balance / 10_000_000);
```

---

## Data Types

### Fixed-Point Arithmetic

All token amounts use fixed-point representation with 7 decimal places.

**Base**: 10^7 = 10,000,000

**Precision**: 0.0000001 tokens (1 smallest unit)

**Maximum Value**: ~3.4 × 10^20 tokens (u128::MAX / 10^7)

**Conversion**:
```rust
// Readable to fixed-point
fn to_fixed(readable_amount: u128) -> u128 {
    readable_amount * 10_000_000
}

// Fixed-point to readable
fn from_fixed(fixed_amount: u128) -> u128 {
    fixed_amount / 10_000_000
}

// Examples
to_fixed(1);     // 10_000_000
to_fixed(100);   // 1_000_000_000
from_fixed(10_000_000);  // 1
```

### StreamData

Complete information about a stream:

```rust
pub struct StreamData {
    pub sender: Address,      // Who created the stream (can cancel)
    pub recipient: Address,   // Who receives tokens (can claim)
    pub token: Address,       // Token contract address
    pub amount: u128,         // Total stream amount (fixed-point)
    pub start_time: u64,      // Stream start (seconds since epoch)
    pub end_time: u64,        // Stream end (seconds since epoch)
    pub claimed: u128,        // Amount already claimed (fixed-point)
}
```

---

## Error Codes

### Complete Error Reference

| Code | Cause | Context | Recovery |
|------|-------|---------|----------|
| `stream_not_found` | Stream ID invalid | get_stream, claim, cancel, extend_ttl | Use valid stream ID |
| `stream_ended` | Trying to claim past end | claim_stream (if check added) | Check stream times |
| `invalid_amount` | Amount zero or too large | create_stream | Use amount in range |
| `invalid_times` | start >= end | create_stream | Ensure start < end |
| `same_addr` | Recipient is sender | create_stream | Use different recipient |
| `unauthorized` | Wrong signer | claim, cancel, extend_ttl | Sign with correct account |
| `overflow` | Math overflow | Any arithmetic | Use smaller amounts |
| `div_zero` | Division by zero | Claimable calculation | Check duration > 0 |
| `already_initialized` | Contract init'd twice | initialize | Skip initialization |
| `not_initialized` | Contract not init'd | Any function | Call initialize() first |

---

## Frontend API

### Dashboard Data Model

The frontend maintains this data model:

```typescript
interface Stream {
  id: string;
  sender: string;           // G... address
  recipient: string;        // G... address
  token: string;           // Token contract
  amount: bigint;          // Fixed-point
  startTime: number;       // Unix seconds
  endTime: number;         // Unix seconds
  claimed: bigint;         // Fixed-point
  claimable: bigint;       // Calculated
  progress: number;        // 0-100%
  status: 'active' | 'ended' | 'claimed' | 'cancelled';
}

interface Account {
  address: string;
  balance: bigint;          // Fixed-point
  streamsCreated: string[]; // Stream IDs
  streamsReceiving: string[]; // Stream IDs
  totalClaimable: bigint;   // Sum of all claimable
}
```

### Real-Time Updates

The dashboard refreshes every 30 seconds:

```typescript
setInterval(() => {
  // Refresh all stream data
  refreshStreams();
  
  // Recalculate claimable amounts
  updateClaimableAmounts();
  
  // Update progress bars
  updateProgressBars();
}, 30000);  // 30 seconds
```

### Amount Formatting

```typescript
// Convert fixed-point to readable with proper formatting
function formatAmount(fixedPoint: bigint): string {
  const decimal = Number(fixedPoint) / 10_000_000;
  return decimal.toLocaleString('en-US', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 7,
  });
}

// Examples
formatAmount(10_000_000n);      // "1.00"
formatAmount(100_000_000n);     // "10.00"
formatAmount(1_000_000_000n);   // "100.00"
formatAmount(123_456_789n);     // "12.35"
```

---

## Event Schema

### Stream Creation Event

```json
{
  "type": "stream_created",
  "streamId": "1",
  "sender": "GXXX...",
  "recipient": "GYYY...",
  "token": "CZZZ...",
  "amount": "1000000000",
  "startTime": 1704067200,
  "endTime": 1706745600,
  "timestamp": 1704067200,
  "ledger": 12345
}
```

### Claim Event

```json
{
  "type": "stream_claimed",
  "streamId": "1",
  "recipient": "GYYY...",
  "amountClaimed": "333333333",
  "timestamp": 1704153600,
  "ledger": 12350
}
```

### Cancellation Event

```json
{
  "type": "stream_cancelled",
  "streamId": "1",
  "sender": "GXXX...",
  "claimed": "333333333",
  "refunded": "666666667",
  "timestamp": 1704240000,
  "ledger": 12360
}
```

---

## Integration Examples

### Rust (using Soroban SDK)

```rust
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct StreamApp;

#[contractimpl]
impl StreamApp {
    pub fn send_monthly_salary(
        env: Env,
        flux: Address,
        token: Address,
        employee: Address,
        monthly_salary: u128,
    ) -> Result<u64, Symbol> {
        let now = env.ledger().timestamp();
        let one_month = 30 * 86400;
        
        env.invoke_contract(
            &flux,
            &Symbol::short("create_stream"),
            (&token, &employee, &monthly_salary, &now, &now + one_month),
        )
    }
}
```

### JavaScript

```typescript
async function createStream(
  senderKey: Keypair,
  tokenAddress: string,
  recipient: string,
  amountReadable: number,
): Promise<string> {
  const amountFixed = BigInt(amountReadable * 10_000_000);
  const now = Math.floor(Date.now() / 1000);
  const thirtyDaysLater = now + 30 * 86400;
  
  const tx = await contract.call('create_stream', [
    tokenAddress,
    recipient,
    amountFixed.toString(),
    now.toString(),
    thirtyDaysLater.toString(),
  ], {
    auth: [senderKey.publicKey()],
    signers: [senderKey],
  });
  
  return tx.id;
}
```

### Python

```python
from stellar_sdk import Soroban

soroban = Soroban(CONTRACT_ID, network="testnet")

stream_id = soroban.invoke(
    method='create_stream',
    args=[
        token_address,
        recipient_address,
        1_000_000_000,  # 100 tokens (fixed-point)
        int(time.time()),
        int(time.time()) + 30 * 86400,
    ],
    auth=[sender_address],
)
```

---

## Testing & Simulation

### Local Testing

```bash
# Run all contract tests
make test-contracts

# Run specific test
cargo test test_stream_math_25_percent -- --nocapture

# Run with logging
RUST_LOG=debug cargo test
```

### Testnet Deployment

```bash
# Deploy to testnet
make deploy NETWORK=testnet

# Query contract
soroban contract info --id <CONTRACT_ID> --rpc-url https://soroban-testnet.stellar.org
```

### Simulation Example

```typescript
// Simulate a 30-day stream
async function simulateStreamClaiming() {
  const streamId = await createStream(...);
  
  const schedule = [
    { day: 0, expectedPercent: 0 },
    { day: 5, expectedPercent: 16.67 },
    { day: 10, expectedPercent: 33.33 },
    { day: 20, expectedPercent: 66.67 },
    { day: 30, expectedPercent: 100 },
  ];
  
  for (const checkpoint of schedule) {
    // Skip time to checkpoint
    await skipTime(checkpoint.day * 86400);
    
    // Get claimable
    const claimable = await contract.call('get_claimable', [streamId]);
    const percent = (claimable / totalAmount) * 100;
    
    // Verify
    expect(Math.abs(percent - checkpoint.expectedPercent)).toBeLessThan(1);
  }
}
```

---

## Support & References

- [Stellar Developer Portal](https://developers.stellar.org/)
- [Soroban Documentation](https://soroban.stellar.org/docs/)
- [Architecture Guide](./ARCHITECTURE.md)
- [Security Guidelines](./SECURITY.md)
- [Deployment Guide](./DEPLOYMENT_GUIDE.md)
- [Integration Guide](./INTEGRATION_GUIDE.md)

---

**Last Updated**: 2024  
**Version**: 1.0  
**Status**: Production Ready
