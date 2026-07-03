# Soroban Flux Integration Guide

Complete integration examples for developers building on top of Soroban Flux streaming payments.

---

## Table of Contents

1. [Rust Integration (Soroban SDK)](#rust-integration-soroban-sdk)
2. [JavaScript/TypeScript Integration](#javascripttypescript-integration)
3. [Python Integration](#python-integration)
4. [Stream Creation Workflow](#stream-creation-workflow)
5. [Stream Claiming Workflow](#stream-claiming-workflow)
6. [Error Handling Best Practices](#error-handling-best-practices)
7. [Common Integration Patterns](#common-integration-patterns)

---

## Rust Integration (Soroban SDK)

### Setup

Add to `Cargo.toml`:

```toml
[dependencies]
soroban-sdk = "20.5.0"
soroban-token-sdk = "20.5.0"
```

### Basic Contract Integration

```rust
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

#[contract]
pub struct StreamClient;

#[contractimpl]
impl StreamClient {
    /// Create a stream for a recipient
    pub fn create_stream_for_user(
        env: Env,
        flux_contract: Address,
        token: Address,
        recipient: Address,
        amount: u128,
        duration_seconds: u64,
    ) -> Result<u64, Symbol> {
        let now = env.ledger().timestamp();
        let end_time = now + duration_seconds;
        
        // Invoke Flux contract
        let stream_id: u64 = env.invoke_contract(
            &flux_contract,
            &Symbol::short("create_stream"),
            (&token, &recipient, &amount, &now, &end_time),
        );
        
        Ok(stream_id)
    }
    
    /// Check claimable amount
    pub fn check_claimable(
        env: Env,
        flux_contract: Address,
        stream_id: u64,
    ) -> Result<u128, Symbol> {
        let claimable: u128 = env.invoke_contract(
            &flux_contract,
            &Symbol::short("get_claimable"),
            (&stream_id,),
        );
        
        Ok(claimable)
    }
}
```

### Stream Creation with Authorization

```rust
/// Create stream with proper authorization
pub fn create_stream_authorized(
    env: Env,
    flux_contract: Address,
    token: Address,
    sender: Address,
    recipient: Address,
    amount: u128,
    duration_days: u64,
) -> Result<u64, Symbol> {
    // Require sender authorization
    sender.require_auth();
    
    // Calculate time boundaries
    let now = env.ledger().timestamp();
    let start_time = now;
    let end_time = now + (duration_days * 86400);  // Convert days to seconds
    
    // Create stream
    let stream_id: u64 = env.invoke_contract(
        &flux_contract,
        &Symbol::short("create_stream"),
        (&token, &recipient, &amount, &start_time, &end_time),
    );
    
    // Log stream creation
    env.storage()
        .instance()
        .set::<_, u64>(&Symbol::short("last_stream"), &stream_id);
    
    Ok(stream_id)
}
```

### Claim and Transfer Pattern

```rust
/// Helper: Recipient claims from stream
pub fn claim_from_stream(
    env: Env,
    flux_contract: Address,
    recipient: Address,
    stream_id: u64,
) -> Result<u128, Symbol> {
    // Require recipient authorization
    recipient.require_auth();
    
    // Call claim
    let claimed: u128 = env.invoke_contract(
        &flux_contract,
        &Symbol::short("claim_stream"),
        (&recipient, &stream_id),
    );
    
    Ok(claimed)
}
```

### Stream Cancellation with Refund

```rust
/// Cancel stream and handle refund
pub fn cancel_stream_safe(
    env: Env,
    flux_contract: Address,
    stream_id: u64,
) -> Result<(u128, u128), Symbol> {
    // Get stream details before cancellation
    let stream: (Address, Address, Address, u128, u64, u64, u128) = env.invoke_contract(
        &flux_contract,
        &Symbol::short("get_stream"),
        (&stream_id,),
    );
    
    let sender = stream.0;
    let original_amount = stream.3;
    
    // Require sender authorization
    sender.require_auth();
    
    // Cancel and get refund amounts
    let (claimed, refunded): (u128, u128) = env.invoke_contract(
        &flux_contract,
        &Symbol::short("cancel_stream"),
        (&stream_id,),
    );
    
    // Verify refund math
    let total = claimed + refunded;
    assert_eq!(total, original_amount, "Refund math error");
    
    Ok((claimed, refunded))
}
```

### Batch Stream Operations

```rust
/// Create multiple streams in sequence
pub fn create_bulk_streams(
    env: Env,
    flux_contract: Address,
    token: Address,
    recipients: Vec<Address>,
    amounts: Vec<u128>,
    duration_days: u64,
) -> Result<Vec<u64>, Symbol> {
    assert_eq!(recipients.len(), amounts.len(), "Mismatched lengths");
    
    let sender = env.invoker();
    sender.require_auth();
    
    let mut stream_ids = Vec::new();
    
    for (i, recipient) in recipients.iter().enumerate() {
        let stream_id = Self::create_stream_authorized(
            env.clone(),
            flux_contract.clone(),
            token.clone(),
            sender.clone(),
            recipient.clone(),
            amounts[i],
            duration_days,
        )?;
        stream_ids.push(stream_id);
    }
    
    Ok(stream_ids)
}
```

---

## JavaScript/TypeScript Integration

### Setup

Install dependencies:

```bash
npm install stellar-sdk soroban-client axios
```

### Initialize Client

```typescript
import { Keypair, Server, Address, Contract } from 'stellar-sdk';
import { rpcServer } from 'soroban-client';

// Initialize
const NETWORK = 'testnet';
const RPC_URL = 'https://soroban-testnet.stellar.org';
const CONTRACT_ID = 'C...';

const server = new Server(RPC_URL);
const keypair = Keypair.fromSecret(process.env.SECRET_KEY!);

// Create contract client
const contract = new Contract(CONTRACT_ID, {
  server,
  allowHttp: NETWORK === 'testnet',
});
```

### Create Stream

```typescript
async function createStream(
  senderAddress: string,
  recipientAddress: string,
  tokenAddress: string,
  amountFixed: bigint,  // In fixed-point (multiply by 10^7)
  durationSeconds: number,
): Promise<string> {
  try {
    const now = Math.floor(Date.now() / 1000);
    const startTime = now;
    const endTime = now + durationSeconds;
    
    // Build transaction
    const transaction = await contract.call(
      'create_stream',
      [
        tokenAddress,
        recipientAddress,
        amountFixed.toString(),
        startTime.toString(),
        endTime.toString(),
      ],
      {
        auth: [senderAddress],
        signers: [keypair],
      }
    );
    
    // Submit transaction
    const result = await server.submitTransaction(transaction);
    
    // Extract stream ID from result
    const streamId = result.id;
    console.log('Stream created:', streamId);
    
    return streamId;
  } catch (error) {
    console.error('Create stream error:', error);
    throw error;
  }
}
```

### Get Claimable Amount

```typescript
async function getClaimableAmount(
  streamId: string,
): Promise<string> {
  try {
    const result = await contract.call(
      'get_claimable',
      [streamId],
      { readOnly: true },
    );
    
    // Result is in fixed-point
    return result;
  } catch (error) {
    console.error('Get claimable error:', error);
    throw error;
  }
}

// Helper: Convert fixed-point to readable
function formatAmount(fixedPoint: bigint): string {
  const decimal = Number(fixedPoint) / 10_000_000;
  return decimal.toLocaleString('en-US', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 7,
  });
}

// Usage
const claimableFixed = await getClaimableAmount('1');
const readable = formatAmount(BigInt(claimableFixed));
console.log(`Claimable: ${readable} tokens`);
```

### Claim Tokens

```typescript
async function claimStream(
  recipientAddress: string,
  streamId: string,
): Promise<string> {
  try {
    const transaction = await contract.call(
      'claim_stream',
      [recipientAddress, streamId],
      {
        auth: [recipientAddress],
        signers: [keypair],
      }
    );
    
    const result = await server.submitTransaction(transaction);
    console.log('Tokens claimed:', result.id);
    
    return result.id;
  } catch (error) {
    console.error('Claim error:', error);
    throw error;
  }
}
```

### Cancel Stream

```typescript
async function cancelStream(
  senderAddress: string,
  streamId: string,
): Promise<{ claimed: string; refunded: string }> {
  try {
    const transaction = await contract.call(
      'cancel_stream',
      [streamId],
      {
        auth: [senderAddress],
        signers: [keypair],
      }
    );
    
    const result = await server.submitTransaction(transaction);
    
    // Parse result to get claimed and refunded amounts
    const claimed = BigInt(result.claimed);
    const refunded = BigInt(result.refunded);
    
    console.log('Stream cancelled');
    console.log(`Claimed: ${formatAmount(claimed)}`);
    console.log(`Refunded: ${formatAmount(refunded)}`);
    
    return {
      claimed: claimed.toString(),
      refunded: refunded.toString(),
    };
  } catch (error) {
    console.error('Cancel error:', error);
    throw error;
  }
}
```

### React Hook for Stream Monitoring

```typescript
import { useEffect, useState } from 'react';

interface StreamInfo {
  id: string;
  sender: string;
  recipient: string;
  amount: string;
  claimable: string;
  progress: number;  // 0-100%
}

function useStreamInfo(streamId: string): {
  stream: StreamInfo | null;
  loading: boolean;
  error: Error | null;
} {
  const [stream, setStream] = useState<StreamInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  
  useEffect(() => {
    const fetchStream = async () => {
      try {
        setLoading(true);
        
        const details = await contract.call(
          'get_stream',
          [streamId],
          { readOnly: true },
        );
        
        const claimable = await getClaimableAmount(streamId);
        
        // Calculate progress
        const total = BigInt(details.amount);
        const claimed = BigInt(claimable);
        const progress = (Number(claimed) / Number(total)) * 100;
        
        setStream({
          id: streamId,
          sender: details.sender,
          recipient: details.recipient,
          amount: formatAmount(total),
          claimable: formatAmount(claimed),
          progress: Math.min(progress, 100),
        });
      } catch (err) {
        setError(err instanceof Error ? err : new Error('Unknown error'));
      } finally {
        setLoading(false);
      }
    };
    
    fetchStream();
    const interval = setInterval(fetchStream, 30000);  // Refresh every 30s
    
    return () => clearInterval(interval);
  }, [streamId]);
  
  return { stream, loading, error };
}
```

---

## Python Integration

### Setup

Install dependencies:

```bash
pip install stellar-sdk py-soroban
```

### Initialize Client

```python
from stellar_sdk import Server, Asset, TransactionBuilder, Keypair
from stellar_sdk.soroban import SorobanServer

network = "testnet"
rpc_url = "https://soroban-testnet.stellar.org"
contract_id = "C..."

# Initialize servers
soroban_server = SorobanServer(rpc_url)
server = Server(rpc_url)

# Load keypair
keypair = Keypair.random()
account = server.load_account(keypair.public_key)
```

### Create Stream

```python
def create_stream(
    sender_address: str,
    recipient_address: str,
    token_address: str,
    amount_fixed: int,  # Fixed-point
    duration_seconds: int,
) -> str:
    """Create a stream"""
    
    import time
    
    now = int(time.time())
    start_time = now
    end_time = now + duration_seconds
    
    # Build transaction
    transaction_builder = TransactionBuilder(
        account=account,
        base_fee=100,
        network_passphrase="Test SDF Network ; September 2015",
    )
    
    transaction_builder.add_text_memo("Create stream")
    transaction_builder.add_soroban_invoke_operation(
        contract_id=contract_id,
        method="create_stream",
        parameters=[
            token_address,
            recipient_address,
            amount_fixed,
            start_time,
            end_time,
        ],
        auth=[sender_address],
    )
    
    transaction = transaction_builder.build()
    
    # Sign and submit
    transaction.sign(keypair)
    result = server.submit_transaction(transaction)
    
    return result["id"]
```

### Get Claimable

```python
def get_claimable(stream_id: int) -> int:
    """Get claimable amount for a stream"""
    
    result = soroban_server.invoke_contract(
        contract_id=contract_id,
        method="get_claimable",
        parameters=[stream_id],
        read_only=True,
    )
    
    return int(result)

# Helper: Convert fixed-point to readable
def format_amount(fixed_point: int) -> str:
    decimal = fixed_point / 10_000_000
    return f"{decimal:.7f}".rstrip('0').rstrip('.')
```

### Claim Stream

```python
def claim_stream(
    recipient_address: str,
    stream_id: int,
) -> str:
    """Claim tokens from stream"""
    
    transaction_builder = TransactionBuilder(
        account=account,
        base_fee=100,
        network_passphrase="Test SDF Network ; September 2015",
    )
    
    transaction_builder.add_text_memo("Claim stream")
    transaction_builder.add_soroban_invoke_operation(
        contract_id=contract_id,
        method="claim_stream",
        parameters=[recipient_address, stream_id],
        auth=[recipient_address],
    )
    
    transaction = transaction_builder.build()
    transaction.sign(keypair)
    result = server.submit_transaction(transaction)
    
    return result["id"]
```

---

## Stream Creation Workflow

### Step-by-Step: Creating a Stream

```
1. Prepare Parameters
   ├─ Recipient address (validated)
   ├─ Token contract address
   ├─ Amount (in fixed-point: multiply readable amount by 10^7)
   └─ Duration (in seconds: days * 86400)

2. Calculate Times
   ├─ Current time: now = Date.now() / 1000
   ├─ Start time: now
   └─ End time: now + duration_seconds

3. Authorize Sender
   └─ Sender must authorize transaction

4. Create Stream
   └─ Call create_stream(token, recipient, amount, start, end)

5. Verify Success
   ├─ Get stream ID from result
   ├─ Confirm stream exists: get_stream(stream_id)
   └─ Log stream ID for tracking
```

### Validation Checklist

```rust
fn validate_stream_params(
    token: &Address,
    recipient: &Address,
    amount: u128,
    start_time: u64,
    end_time: u64,
) -> Result<(), Symbol> {
    // Validate recipient address format
    if recipient.account_id().is_none() {
        return Err(Symbol::short("invalid_recipient"));
    }
    
    // Validate times
    if start_time >= end_time {
        return Err(Symbol::short("invalid_times"));
    }
    
    // Validate amount is in reasonable range
    const MIN_AMOUNT: u128 = 1_000_000;  // 0.1 tokens
    const MAX_AMOUNT: u128 = 1_000_000_000_000;  // 100M tokens
    
    if amount < MIN_AMOUNT || amount > MAX_AMOUNT {
        return Err(Symbol::short("invalid_amount"));
    }
    
    Ok(())
}
```

---

## Stream Claiming Workflow

### Optimal Claiming Strategy

```
1. Check Claimable Amount
   └─ Call get_claimable(stream_id)
   └─ If amount > 0, proceed to claim

2. Prepare Claim
   ├─ Recipient must authorize
   ├─ Have receiver's keypair ready
   └─ Ensure sufficient fees

3. Execute Claim
   └─ Call claim_stream(recipient, stream_id)

4. Verify Claim
   ├─ Confirm transaction success
   ├─ Check token balance increased
   └─ Verify claimable is now lower
```

### Claim with Retry Logic

```typescript
async function claimWithRetry(
  recipientAddress: string,
  streamId: string,
  maxRetries: number = 3,
): Promise<string> {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await claimStream(recipientAddress, streamId);
    } catch (error) {
      console.warn(`Claim attempt ${attempt} failed:`, error);
      
      if (attempt < maxRetries) {
        // Exponential backoff
        const delayMs = Math.pow(2, attempt) * 1000;
        await new Promise(resolve => setTimeout(resolve, delayMs));
      } else {
        throw error;
      }
    }
  }
  
  throw new Error('All claim attempts failed');
}
```

---

## Error Handling Best Practices

### Common Errors and Recovery

```typescript
enum FluxError {
  STREAM_NOT_FOUND = 'stream_not_found',
  INVALID_AMOUNT = 'invalid_amount',
  INSUFFICIENT_AUTH = 'insufficient_auth',
  STREAM_ENDED = 'stream_ended',
  ALREADY_CLAIMED = 'already_claimed',
}

async function handleFluxError(
  error: any,
  context: string,
): Promise<void> {
  const errorMessage = error.message || error.toString();
  
  if (errorMessage.includes('not_found')) {
    console.error(`${context}: Stream doesn't exist`);
  } else if (errorMessage.includes('invalid_amount')) {
    console.error(`${context}: Amount is invalid`);
  } else if (errorMessage.includes('insufficient')) {
    console.error(`${context}: Insufficient balance`);
  } else if (errorMessage.includes('auth')) {
    console.error(`${context}: Authorization failed`);
  } else {
    console.error(`${context}: Unknown error:`, error);
  }
}
```

### Transaction Error Recovery

```typescript
async function createStreamSafe(
  params: StreamParams,
): Promise<string | null> {
  try {
    return await createStream(
      params.sender,
      params.recipient,
      params.token,
      params.amount,
      params.duration,
    );
  } catch (error) {
    if (error instanceof InsufficientFundsError) {
      console.error('Insufficient funds. Need to top up account.');
      return null;
    }
    
    if (error instanceof InvalidParameterError) {
      console.error('Invalid parameters provided.');
      return null;
    }
    
    if (error instanceof NetworkError) {
      console.error('Network error. Retrying...');
      // Implement retry logic
      return null;
    }
    
    throw error;
  }
}
```

---

## Common Integration Patterns

### Pattern 1: Recurring Streams

```typescript
async function createRecurringStreams(
  sender: string,
  recipients: string[],
  amount: bigint,
  interval: number,  // seconds between streams
  count: number,  // number of streams to create
): Promise<string[]> {
  const streamIds: string[] = [];
  
  for (let i = 0; i < count; i++) {
    const startTime = Math.floor(Date.now() / 1000) + (i * interval);
    const endTime = startTime + interval;
    
    const streamId = await createStream(
      sender,
      recipients[i % recipients.length],
      TOKEN_ADDRESS,
      amount,
      interval,
    );
    
    streamIds.push(streamId);
  }
  
  return streamIds;
}
```

### Pattern 2: Multi-Recipient Distribution

```typescript
async function distributeToMultiple(
  sender: string,
  distributions: Array<{ recipient: string; amount: bigint; days: number }>,
): Promise<string[]> {
  const streamIds = await Promise.all(
    distributions.map(d =>
      createStream(
        sender,
        d.recipient,
        TOKEN_ADDRESS,
        d.amount,
        d.days * 86400,
      )
    )
  );
  
  return streamIds;
}
```

### Pattern 3: Conditional Claiming

```typescript
async function smartClaim(
  recipient: string,
  streamId: string,
  minClaimable: bigint,
): Promise<boolean> {
  const claimableFixed = await getClaimableAmount(streamId);
  const claimable = BigInt(claimableFixed);
  
  if (claimable < minClaimable) {
    console.log(
      `Not enough to claim. Need: ${minClaimable}, available: ${claimable}`
    );
    return false;
  }
  
  await claimStream(recipient, streamId);
  return true;
}
```

---

**See Also**: [API.md](./API.md) for full contract reference
