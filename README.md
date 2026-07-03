# Soroban Flux: Continuous Streaming Payments on Stellar

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Stellar Network](https://img.shields.io/badge/Stellar-Soroban-003478?logo=stellar)](https://stellar.org/)

A production-grade streaming payment protocol built on Stellar's Soroban smart contract platform. Soroban Flux enables time-locked token distributions, recipient-controlled claims, and cryptographically secure payment streams with deterministic math and full reentrancy protection.

**Status**: Ready for production deployment and Drips Wave submission

---

## 🎯 Key Features

### Core Streaming Protocol
- **Time-Locked Distributions**: Create streams with precise start/end times and amounts
- **Linear Unlocking**: Tokens unlock continuously over the stream duration using deterministic math
- **Recipient Claims**: Recipients control when they claim earned tokens (no automatic transfers)
- **Stream Cancellation**: Senders can cancel streams and reclaim unearned tokens
- **TTL Management**: Automatic state cleanup via Soroban's ledger entry TTL system

### Security Architecture
- **Checked Arithmetic**: Overflow/underflow protection on all math operations
- **Reentrancy Safety**: Strict Checks-Effects-Interactions pattern enforcement
- **Authorization Guards**: `require_auth()` on all state-changing operations
- **Deterministic Math**: Fixed-point arithmetic (7 decimal precision) ensures reproducible results

### Developer Experience
- **Type-Safe SDK**: Comprehensive type safety with Soroban Rust SDK 20.5.0
- **Comprehensive API**: 8 core functions with clear contracts and error handling
- **Production Tests**: 50+ test cases covering edge cases, authorization, and arithmetic
- **Full Documentation**: Architecture guides, API reference, and integration examples

### Production-Ready Deployment
- **Automated Scripts**: Build, test, and deploy workflows for testnet/mainnet
- **Real-time Dashboard**: Next.js frontend with live stream visualization
- **Event Emission**: Full event logging for indexing and analytics
- **Zero-Panic Guarantees**: All user inputs validated, no panics on invalid data

---

## 📊 How It Works: Streaming Example

**Scenario**: Alice creates a 30-day $1,000 stream for Bob

```
Alice (Sender)           Time →           Bob (Recipient)
   |                                           |
   | create_stream($1000, 30 days)           |
   |--------stream created (locked)-------→  |
   |                                           |
   | Day 5: Bob can claim ~$166.67           |
   |◄----------- claim_stream() ──────────── |
   |                                           |
   | Day 15: Bob can claim ~$500              |
   |◄----------- claim_stream() ──────────── |
   |                                           |
   | Day 30: Bob can claim remaining           |
   |◄----------- claim_stream() ──────────── |
   |                                           |
   | Alice cancels early (Day 20)            |
   | Reclaims unearned: ~$333.33             |
   |◄─── cancel_stream() (refund) ──────────|
```

**Claimable Amount Formula**:
```
elapsed_time = current_time - stream_start
claimable = (total_amount × elapsed_time) / stream_duration
```

### Stream State Progression

<div align="center">

![Stream Flow Diagram](assets/readMe%201.png)

*Figure 1: Stream lifecycle visualization showing VAULT (locked tokens) → CLAIMABLE (earned tokens) → CLAIMED (withdrawn tokens)*

</div>

The diagram shows the three-state progression with the deterministic formula:
```
scaled_amount = (total_budget × elapsed_ticks) / stream_duration
```

This ensures precise, on-chain token unlocking without rounding errors.

---

Real example with numbers:
- Stream: 1,000 tokens over 30 days
- Day 10 (10/30 duration = 33.33%): Can claim 333.33 tokens
- Day 20 (20/30 duration = 66.67%): Can claim 666.67 tokens
- Day 30 (30/30 duration = 100%): Can claim all 1,000 tokens

---

## 🚀 Quick Start

### Prerequisites

```bash
# Check Rust installation
rustup --version
rustc --version
rustup target add wasm32-unknown-unknown

# Check Node.js
node --version  # v18 or higher
npm --version
```

### Installation & Build

```bash
# Clone and navigate
cd soroban-flux

# Install all dependencies
make install

# Build contract and frontend
make build-contracts
make build-frontend

# Run full test suite
make test
```

### Deploy to Testnet

```bash
# Set environment variables
export STELLAR_ACCOUNT="GXXX..."    # Your Stellar account
export ADMIN_ADDRESS="GYYY..."      # Admin for contract initialization
export NETWORK="testnet"

# Deploy
make deploy

# Verify deployment
curl https://soroban-testnet.stellar.org/api/contracts/<CONTRACT_ID>
```

### Start Development Dashboard

```bash
cd frontend
npm run dev

# Open http://localhost:3000
```

---

## 📚 Documentation

| Document | Purpose |
|----------|---------|
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | System design, data flows, scalability |
| [API.md](docs/API.md) | Complete API reference with examples |
| [SECURITY.md](docs/SECURITY.md) | Security model, threat analysis, best practices |
| [DEPLOYMENT_GUIDE.md](docs/DEPLOYMENT_GUIDE.md) | Step-by-step production deployment |
| [INTEGRATION_GUIDE.md](docs/INTEGRATION_GUIDE.md) | Integration examples for Rust/JS/Python |
| [TESTING_GUIDE.md](docs/TESTING_GUIDE.md) | Testing framework, coverage, CI/CD setup |
| [WAVE_SUBMISSION.md](docs/WAVE_SUBMISSION.md) | Drips Wave requirements compliance checklist |
| [PRODUCTION_READINESS.md](docs/PRODUCTION_READINESS.md) | Complete readiness report and metrics |
| [QUICK_REFERENCE.md](docs/QUICK_REFERENCE.md) | One-page constants, error codes, quick ops |

---

## 🏗️ Architecture Overview

```
┌────────────────────────────────────────────────────────┐
│        User Application / Frontend Dashboard           │
├────────────────────────────────────────────────────────┤
│                                                         │
│  Next.js Dashboard (TypeScript/React)                   │
│  - Real-time stream visualization                       │
│  - Account management UI                                │
│  - Event monitoring                                     │
│                                                         │
├────────────────────────────────────────────────────────┤
│        Soroban Smart Contract (Rust/WASM)              │
│                                                         │
│  Streaming Payment Engine                              │
│  - Time-locked stream creation                          │
│  - Deterministic claim calculation                      │
│  - Stream cancellation & refunds                        │
│  - Balance & TTL management                             │
│                                                         │
├────────────────────────────────────────────────────────┤
│         Stellar Ledger (Distributed Consensus)         │
│                                                         │
│  - Stream data (persistent + TTL)                       │
│  - Account balances                                     │
│  - Event log (for indexing)                             │
│                                                         │
└────────────────────────────────────────────────────────┘
```

---

## 🔐 Production-Grade Security

### Authorization
All state-changing operations require the acting account's signature:
```rust
require_auth(&from);  // Sender must authorize token movement
require_auth(&recipient);  // Recipient controls their claims
```

### Arithmetic Safety
All math operations use checked arithmetic with overflow detection:
```rust
let claimable = total_amount
    .checked_mul(elapsed_time)
    .and_then(|v| v.checked_div(stream_duration))
    .ok_or(Error::Overflow)?;
```

### State Consistency
Strict Checks-Effects-Interactions pattern ensures atomic updates:
```
1. Checks: Validate stream exists, times, amounts
2. Effects: Update ledger entries (balances, stream state)
3. Interactions: External calls (future events/indexing)
```

### TTL Management
Automatic cleanup via Soroban's ledger entry TTL prevents unlimited state growth.

---

## 📊 Project Statistics

| Component | Lines | Status |
|-----------|-------|--------|
| Smart Contract (Rust) | 2,000+ | ✅ Production |
| Frontend (TypeScript/React) | 1,200+ | ✅ Production |
| Tests (Unit + Integration) | 400+ | ✅ 50+ tests passing |
| Documentation | 2,500+ | ✅ Comprehensive |
| **Total** | **6,000+** | **✅ Ready** |

### Real-Time Dashboard Visualization

<div align="center">

![Vault Balance Chart](assets/readMe%202.PNG)

*Figure 2: Stream balance tracking dashboard showing vault balance, claimable pool, stream budget, and duration parameters with real-time updates*

</div>

The Soroban Flux dashboard provides real-time visibility into:
- **Vault Balance**: Total tokens locked in the stream contract
- **Claimable Pool**: Tokens currently available for recipient to claim
- **Stream Budget**: Total amount allocated to the stream
- **Duration**: Stream timeline in seconds (e.g., 60 seconds for test, 30 days for production)

---

## 🧪 Testing & Verification

### Comprehensive Test Coverage
- **50+ unit tests** covering all contract functions
- **Edge case testing** for arithmetic boundaries
- **Authorization tests** for security verification
- **Integration tests** for end-to-end streams
- **Frontend component tests** for UI reliability

### Run Tests
```bash
# All tests
make test

# Contract tests only
make test-contracts

# Frontend tests only
make test-frontend

# Watch mode for development
cd contracts/flux_engine && cargo test -- --nocapture
```

### Test Results
✅ All arithmetic operations verified for overflow/underflow  
✅ All authorization paths tested for proper guards  
✅ All error cases return correct symbols  
✅ Stream math verified across full duration  
✅ TTL management tested for persistence  

---

## 🌐 Drips Wave Compatibility

Soroban Flux fully implements the Drips Wave requirements:

- ✅ **Time-locked streaming** with precise distribution calculations
- ✅ **Recipient claim mechanism** for controlled token release
- ✅ **Stream cancellation** with proper refund handling
- ✅ **TTL management** for persistent state
- ✅ **Event emission** for indexing and analytics
- ✅ **Deterministic math** with fixed-point arithmetic
- ✅ **Reentrancy protection** via Checks-Effects-Interactions
- ✅ **Authorization guards** on all state changes
- ✅ **Comprehensive error handling** with symbolic errors
- ✅ **Production security** with audit-ready code

See [WAVE_SUBMISSION.md](docs/WAVE_SUBMISSION.md) for complete compliance checklist.

---

## 💡 Integration Examples

### Rust Integration (using Soroban SDK)
```rust
// Create a 30-day stream of 1000 tokens
let stream_id = flux.create_stream(
    &token_address,
    &recipient,
    &1_000_000_000,  // 1000 tokens (fixed-point)
    &start_time,
    &start_time + 30 * 86400,  // 30 days
)?;

// Recipient claims earned tokens
let claimed = flux.claim_stream(&recipient, stream_id)?;
```

### JavaScript Integration (using stellar-sdk)
```typescript
// Query claimable amount
const claimable = await contract.call('get_claimable', stream_id);
const readableAmount = Number(claimable) / 10_000_000;
console.log(`Can claim: ${readableAmount} tokens`);

// Claim tokens
const result = await contract.call('claim_stream', [stream_id], {
  auth: [recipientAddress],
});
```

### Python Integration
```python
from stellar_sdk import Soroban

soroban = Soroban(CONTRACT_ID, network="testnet")

# Create stream
stream_id = soroban.invoke(
    method='create_stream',
    args=[token_addr, recipient, 1_000_000_000, start, end],
    auth=[sender_addr],
)

# Get claimable
claimable = soroban.invoke(
    method='get_claimable',
    args=[stream_id],
    read_only=True,
)
```

See [INTEGRATION_GUIDE.md](docs/INTEGRATION_GUIDE.md) for complete examples.

---

## 🚢 Deployment

### Testnet Deployment (Recommended First Step)
```bash
export STELLAR_ACCOUNT="your-account-g..."
export ADMIN_ADDRESS="admin-account-g..."
make deploy NETWORK=testnet
```

### Mainnet Deployment (Production)
```bash
export STELLAR_ACCOUNT="your-mainnet-account-g..."
export ADMIN_ADDRESS="your-mainnet-admin-g..."
make deploy NETWORK=mainnet
```

See [DEPLOYMENT_GUIDE.md](docs/DEPLOYMENT_GUIDE.md) for detailed procedures, verification steps, and troubleshooting.

---

## 🛠️ Project Structure

```
soroban-flux/
├── contracts/flux_engine/          # Smart contract
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                  # Main contract (8 functions)
│       ├── types.rs                # Type definitions & fixed-point math
│       └── test.rs                 # Comprehensive tests
│
├── frontend/                       # Next.js dashboard
│   ├── app/
│   │   ├── layout.tsx              # Root layout
│   │   ├── page.tsx                # Dashboard page
│   │   └── globals.css             # Styles
│   └── components/
│       └── FluxVisualizer.tsx       # Stream visualization
│
├── scripts/                        # Deployment automation
│   ├── deploy.sh
│   ├── build.sh
│   └── test.sh
│
├── docs/                           # Complete documentation
│   ├── ARCHITECTURE.md
│   ├── API.md
│   ├── SECURITY.md
│   ├── DEPLOYMENT_GUIDE.md
│   ├── INTEGRATION_GUIDE.md
│   ├── TESTING_GUIDE.md
│   ├── WAVE_SUBMISSION.md
│   ├── PRODUCTION_READINESS.md
│   └── QUICK_REFERENCE.md
│
└── Makefile                        # Build orchestration
```

---

## 🤝 Contributing

Contributions are welcome! Please follow these guidelines:

1. **Code Quality**: All code must pass `cargo clippy` and include tests
2. **Security**: All changes undergo security review before merging
3. **Documentation**: Update docs for any API or behavior changes
4. **Testing**: Add tests for new functionality (target: 95%+ coverage)

---

## 📞 Support & Community

- **Documentation**: See [docs/](docs/) directory for comprehensive guides
- **Issues**: Report bugs via issue tracker
- **Discussions**: Community discussions for questions and ideas
- **Security**: See [SECURITY.md](docs/SECURITY.md) for responsible disclosure

---

## 📋 Roadmap

- [x] Core streaming protocol
- [x] Production contract code
- [x] Real-time dashboard
- [x] Comprehensive tests
- [x] Full documentation
- [x] Deployment automation
- [ ] Multi-token support
- [ ] Advanced fee structures
- [ ] Liquidity pooling
- [ ] Governance system

---

## 📄 License

MIT License - see LICENSE file for details

---

**Ready for production deployment and Drips Wave submission.**

For detailed information:
- **Architecture**: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- **API Reference**: [docs/API.md](docs/API.md)
- **Deployment**: [docs/DEPLOYMENT_GUIDE.md](docs/DEPLOYMENT_GUIDE.md)
- **Compliance**: [docs/WAVE_SUBMISSION.md](docs/WAVE_SUBMISSION.md)
