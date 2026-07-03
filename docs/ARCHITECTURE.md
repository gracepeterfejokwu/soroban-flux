# Soroban Flux Architecture

## Overview

Soroban Flux is a production-grade distributed token management system built on Soroban, Stellar's smart contract platform. The architecture emphasizes security, scalability, and real-time visualization of token flows.

## System Components

### 1. Smart Contract Layer (`contracts/flux_engine/`)

**Purpose**: Core token management and settlement logic

**Key Features**:
- Fixed-point arithmetic with 7 decimal precision (10^7 base)
- Reentrancy protection via Checks-Effects-Interactions pattern
- Overflow/underflow prevention with checked arithmetic
- Settlement batch management
- Account balance tracking

**Key Modules**:
- `lib.rs`: Main contract implementation
- `types.rs`: Type definitions and fixed-point utilities
- `test.rs`: Comprehensive test suite

**Critical Patterns**:
```rust
// Checks-Effects-Interactions pattern for reentrancy safety
// Checks: Validate preconditions
// Effects: Update state
// Interactions: Call external contracts
```

### 2. Frontend Layer (`frontend/`)

**Purpose**: Real-time dashboard for visualizing token flows and settlements

**Technology Stack**:
- Next.js 14 with React 18
- TypeScript 5.3 for type safety
- Tailwind CSS for styling
- Vitest for testing

**Key Pages**:
- `/`: Dashboard with batch visualization
- Components for metrics, flow diagrams, and account distribution

**Data Model**:
```typescript
interface BatchInfo {
  id: number;
  status: 'pending' | 'processing' | 'settled' | 'failed';
  amount: bigint;
  timestamp: number;
  participants: number;
}

interface AccountInfo {
  address: string;
  balance: bigint;
  lastUpdated: number;
}
```

### 3. Deployment & Infrastructure

**Scripts**:
- `deploy.sh`: Network deployment automation (testnet/mainnet)
- `test.sh`: Unified test runner
- `build.sh`: Build orchestration

**Build Artifacts**:
- Contract WASM binary (optimized)
- Frontend static build (`.next/`)

### Visual Reference

Stream State Progression:
![Stream Flow](../assets/readMe%201.png)

Dashboard Interface:
![Dashboard](../assets/readMe%202.PNG)

## Data Flow

```
┌─────────────────────────────────────────────────────────┐
│                  User Interaction                        │
└────────┬────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────┐
│           Frontend Dashboard (Next.js)                   │
│  - Visualizes batches and account balances              │
│  - Real-time updates (30s polling)                      │
│  - Type-safe React components                           │
└────────┬────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────┐
│      Smart Contract (Soroban/WASM)                       │
│  - Transfer: Account → Account (with auth)              │
│  - Batch Creation: Consolidate transactions             │
│  - Fixed-point Math: Precision-preserved arithmetic     │
│  - State Management: Ledger entries with TTL            │
└────────┬────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────┐
│           Stellar Ledger (Distributed)                  │
│  - Account balances                                      │
│  - Transaction history                                  │
│  - Batch settlement records                             │
└─────────────────────────────────────────────────────────┘
```

## Security Model

### 1. Authorization
- All state-changing operations require signed authorization (`require_auth()`)
- Admin-only operations protected by admin address check

### 2. Arithmetic Safety
- Fixed-point math with overflow detection
- Saturating arithmetic for subtraction (prevent negative balances)
- Checked arithmetic for addition/multiplication

### 3. Reentrancy Prevention
- Checks-Effects-Interactions pattern strictly enforced
- State updates happen before external calls
- No delegated calls in current design

### 4. Input Validation
- Amount range checks (min/max settlement)
- Address validation (no self-transfers)
- Fee bounds validation (0-10000 basis points)

### 5. State Consistency
- Atomic balance updates (both accounts or neither)
- Ledger entry TTL for automatic cleanup
- Deterministic batch IDs

## Performance Characteristics

### Storage
- Instance storage: Config, admin, batch counter (~100 bytes)
- Persistent storage: Balances (per account), batches (per batch)
- Per-entry TTL: 1,000,000 ledgers (~4.6 days at 6s/ledger)

### Computation
- Transfer: O(1) with 2 balance lookups + 2 updates
- Batch creation: O(1) with counter increment
- No loops or complex iterations in hot paths

### Scalability Considerations
- Settlement batching reduces on-chain operations
- Fixed-point math is efficient in WASM
- Batch status provides transaction grouping

## Integration Points

### 1. Stellar Network
- Contract deployed to Soroban
- Assets represent Flux tokens
- Native Stellar authorization

### 2. Frontend Integration
- REST API polling (30s intervals)
- Real-time balance updates
- Batch status visualization

### 3. Indexing & Analytics
- Event emission for external indexers
- Ledger sequence tracking
- Batch event logging

## Testing Strategy

### Contract Tests
- Unit tests for fixed-point math
- Batch lifecycle tests
- Authorization and permission tests
- Edge cases (overflow, underflow, zero division)

### Frontend Tests
- Component rendering tests
- Data formatting tests
- Real-time update logic tests

### Integration Tests
- Contract deployment to testnet
- Transaction settlement flows
- Frontend-contract interaction

## Deployment Process

### Development
1. `make install` - Install dependencies
2. `make build-contracts` - Build WASM
3. `make test` - Run test suite
4. `make deploy NETWORK=testnet` - Deploy to testnet

### Production
1. Code review and audit
2. Deploy to testnet for verification
3. Mainnet deployment with STELLAR_ACCOUNT + ADMIN_ADDRESS
4. Frontend deployment to CDN
5. Monitor logs and performance

## Drips Wave Integration

### Stream Lifecycle Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                   Stream Creation (create_stream)               │
│         Sender specifies: token, recipient, amount, time        │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      ▼
        ┌─────────────────────────────────┐
        │   Stream in LOCKED state        │
        │   Recipient: 0% claimable       │
        │   Timelock: Active              │
        └──┬──────────────┬──────────────┬┘
           │              │              │
      Day 5│              │Day 20       │
      (17%)│              │ (67%)       │
           ▼              ▼              ▼
    ┌──────────────────────────────────────────┐
    │   Token Unlocking (linear)               │
    │   Claimable = (total × elapsed) / duration│
    │   [████░░░░░░░░░░░░░░░░░░░░░░░░░░] 17%  │
    └──────┬──────────────┬─────────────────┬──┘
           │ claim        │ claim           │
           ▼              ▼                 ▼
      ┌─────────┐    ┌──────────┐    ┌────────────┐
      │Recipient│    │Recipient │    │Recipient   │
      │claims   │    │claims    │    │claims all  │
      │17%      │    │50%       │    │remaining   │
      └─────────┘    └──────────┘    └────────────┘
           
  Optional: cancel_stream() [Sender]
           ▼
      ┌──────────────────────────────┐
      │ Stream Cancelled             │
      │ Recipient keeps claimed      │
      │ Sender gets unclaimed refund │
      └──────────────────────────────┘
```

### Time-Weighted Unlocking Visualization

```
Total Amount
    │
100%├─────────────────────────────────┐
    │                                 │
 75%├─────────────────────┐           │
    │                     │           │
 50%├─────────────┐       │           │
    │             │       │           │
 25%├─────┐       │       │           │
    │     │       │       │           │
  0%├─────┴───────┴───────┴───────────┘
    └─────────────────────────────────
      Day 0  Day 5  Day 10 Day 15 Day 30
      
    ─ = Locked
    ═ = Claimable
```

### Comparison with Other Streaming Protocols

| Feature | Soroban Flux | Traditional Escrow | Vesting Contract |
|---------|-------------|-------------------|------------------|
| Recipient Control | ✅ Recipient decides when to claim | ❌ Auto-transfer | ✅ Partial claims |
| Cancellation | ✅ Sender can cancel anytime | ❌ Not possible | ✅ Limited |
| Refund Logic | ✅ Automatic & deterministic | ⚠️ Manual | ⚠️ Protocol-defined |
| Gas Efficiency | ✅ O(1) operations | ✅ O(1) operations | ⚠️ O(n) for batches |
| State Cleanup | ✅ TTL-based automatic | ❌ Manual cleanup | ❌ Permanent |
| Precision | ✅ Fixed-point (7 decimals) | ✅ Fixed-point | ⚠️ Variable |

### Scalability Considerations

**Streams per Account**: Unlimited
- Each stream is independent
- Linked via stream ID only
- No account-level bottlenecks

**Concurrent Streams**: Unlimited
- No shared state contention
- Parallel claim operations possible
- Ledger sequence ordering maintained

**Historical Data Growth**: Unbounded
- Events grow with transaction count
- No on-chain cleanup needed
- Indexer responsibility

**State TTL Management**: ~4.6 days
- Automatic ledger cleanup
- User-extendable via extend_ttl()
- No manual intervention needed

### Gas/Fee Optimization

```
Transaction Optimization:
├─ Single create_stream: ~2000 gas
├─ Batch create (10 streams): ~20,000 gas (linear)
├─ Claim operation: ~2000 gas
└─ Query operation: ~100 gas (minimal)

Fee Structure:
├─ Base: Network fees only
├─ No protocol fees (configurable)
└─ Sender/Recipient: Minimal auth cost
```

---

## Future Enhancements

1. **Multi-token Support**: Extend to support multiple token types
2. **Advanced Fee Structures**: Tiered, dynamic, or protocol fees
3. **Liquidity Pools**: Flux token swaps and liquidity management
4. **Enhanced Analytics**: Historical data and reporting
5. **Governance**: DAO-style management for protocol parameters
6. **Bridges**: Cross-chain token transfers
7. **Escrow Streams**: Third-party supervised streams
8. **Conditional Unlocking**: Unlock based on milestones/conditions
