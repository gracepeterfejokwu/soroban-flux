# Soroban Flux Testing Guide

Complete testing documentation and procedures for Soroban Flux.

---

## Table of Contents

1. [Test Structure](#test-structure)
2. [Running Tests Locally](#running-tests-locally)
3. [Test Categories](#test-categories)
4. [Adding New Tests](#adding-new-tests)
5. [Coverage Reporting](#coverage-reporting)
6. [CI/CD Setup](#cicd-setup)

---

## Test Structure

### Contract Tests Location

```
contracts/flux_engine/
├── src/
│   ├── lib.rs          # Main contract code + tests
│   ├── types.rs        # Type definitions
│   └── test.rs         # Comprehensive test suite
└── Cargo.toml          # Dependencies
```

### Frontend Tests Location

```
frontend/
├── __tests__/          # Test files
├── app/                # Application code
└── components/         # React components with tests
```

### Test File Organization

Each test is organized by concern:
- Stream math tests (5 tests)
- Fixed-point tests (5 tests)
- Authorization tests (5 tests)
- Balance tests (5 tests)
- Integration tests (20+ tests)
- Frontend tests (10+ tests)

---

## Running Tests Locally

### Quick Start

```bash
# Run all tests (contract + frontend)
make test

# Run contract tests only
make test-contracts

# Run frontend tests only
make test-frontend
```

### Detailed Contract Testing

```bash
# Navigate to contract directory
cd contracts/flux_engine

# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_stream_math_25_percent

# Run with output for specific test
cargo test test_stream_math_25_percent -- --nocapture

# Run and show test names without running
cargo test --lib -- --list

# Run tests in single-threaded mode (if needed)
cargo test -- --test-threads=1
```

### Frontend Testing

```bash
# Navigate to frontend directory
cd frontend

# Run all tests
npm test -- --run

# Run with watch mode
npm test

# Run specific test file
npm test -- --testNamePattern="Button"

# Run with coverage
npm test -- --coverage

# Run specific browser
npm test -- --browser chrome
```

---

## Test Categories

### 1. Stream Math Tests (5 tests)

These tests verify the streaming math is correct at different points in time.

**Files**: `contracts/flux_engine/src/test.rs`

```rust
#[test]
fn test_stream_math_before_start() {
    // When: Current time is before stream start
    // Then: Claimable amount is 0
    let claimable = calculate_claimable(0, 0);
    assert_eq!(claimable, 0);
}

#[test]
fn test_stream_math_25_percent() {
    // When: 25% of stream duration has elapsed
    // Then: 25% of amount is claimable
    let duration = 100;
    let amount = 10_000_000;
    let elapsed = 25;
    let claimable = (amount * elapsed) / duration;
    assert_eq!(claimable, 2_500_000);
}
```

**Running**:
```bash
cargo test test_stream_math --nocapture
```

### 2. Fixed-Point Arithmetic Tests (5 tests)

These tests verify the fixed-point math operations work correctly.

```rust
#[test]
fn test_multiplication() {
    let a = 10_000_000;  // 1 token
    let b = 10_000_000;  // 1 token
    let result = multiply(a, b).unwrap();
    assert_eq!(result, 100_000_000_000_000);
}

#[test]
fn test_division() {
    let a = 10_000_000;  // 1 token
    let b = 2;
    let result = divide(a, b).unwrap();
    assert_eq!(result, 5_000_000);  // 0.5 token
}

#[test]
fn test_overflow_protection() {
    let max = u128::MAX;
    let result = multiply(max, 2);
    assert!(result.is_err());  // Overflow detected
}
```

### 3. Authorization Tests (5 tests)

These tests verify authorization is properly enforced.

```rust
#[test]
#[should_panic(expected = "not authorized")]
fn test_unauthorized_transfer() {
    // Attempting to transfer without auth should panic
    transfer(&unauthorized_sender, &recipient, 1_000_000);
}

#[test]
fn test_authorized_transfer() {
    // Authorized transfer succeeds
    let result = transfer(&authorized_sender, &recipient, 1_000_000);
    assert!(result.is_ok());
}
```

### 4. Balance Management Tests (5 tests)

These tests verify balance updates work correctly.

```rust
#[test]
fn test_balance_update() {
    let initial = 1_000_000;
    let update = 500_000;
    let new_balance = initial - update;
    assert_eq!(new_balance, 500_000);
}

#[test]
fn test_insufficient_balance() {
    let balance = 100_000;
    let amount = 200_000;
    let result = transfer_with_validation(&balance, &amount);
    assert!(result.is_err());
}
```

### 5. Integration Tests (20+ tests)

These tests verify complete workflows.

```rust
#[test]
fn test_create_claim_lifecycle() {
    // 1. Create stream
    let stream_id = create_stream(...);
    assert!(stream_id > 0);
    
    // 2. Check claimable
    let claimable = get_claimable(stream_id);
    assert!(claimable > 0);
    
    // 3. Claim tokens
    let result = claim_stream(stream_id);
    assert!(result.is_ok());
    
    // 4. Verify balance increased
    let new_balance = get_balance(&recipient);
    assert!(new_balance > 0);
}

#[test]
fn test_cancel_stream_refund() {
    // 1. Create stream
    let (claimed, refunded) = cancel_stream(stream_id);
    
    // 2. Verify math: claimed + refunded = original
    assert_eq!(claimed + refunded, original_amount);
}
```

---

## Adding New Tests

### Step 1: Choose Test Category

Determine whether your test is:
- **Unit test**: Single function behavior
- **Integration test**: Multiple functions together
- **Property-based test**: Testing properties across inputs

### Step 2: Write Test Function

```rust
#[test]
fn test_your_new_functionality() {
    // Setup
    let input = setup_test_data();
    
    // Execute
    let result = function_under_test(input);
    
    // Assert
    assert_eq!(result, expected_value);
}
```

### Step 3: Document Test Purpose

Always add a comment explaining what the test verifies:

```rust
#[test]
fn test_claim_after_end_time() {
    // Verify: All tokens are claimable after stream ends
    // Setup: Create stream ending at time 100
    let stream_end = 100;
    // Execute: Query claimable at time 101 (after end)
    let claimable = get_claimable_at(101);
    // Assert: Should be 100% of amount
    assert_eq!(claimable, STREAM_AMOUNT);
}
```

### Step 4: Run Test

```bash
cargo test test_your_new_functionality

# With output
cargo test test_your_new_functionality -- --nocapture
```

### Step 5: Verify Test Fails First

Make sure new test fails before implementing feature:

```bash
# Test should fail initially
cargo test test_your_new_functionality
# Result: test test_your_new_functionality ... FAILED
```

### Step 6: Implement Feature

After test fails, implement the feature, then verify test passes.

---

## Frontend Testing

### Unit Tests for Components

```typescript
// __tests__/components/StreamCard.test.tsx
import { render, screen } from '@testing-library/react';
import StreamCard from '../../components/StreamCard';

describe('StreamCard', () => {
  it('displays stream information', () => {
    const stream = {
      id: '1',
      sender: 'G...',
      recipient: 'G...',
      amount: '1000000000n',
    };
    
    render(<StreamCard stream={stream} />);
    
    expect(screen.getByText('1000.00')).toBeInTheDocument();
  });
});
```

### Integration Tests

```typescript
// __tests__/integration/stream-flow.test.ts
describe('Complete Stream Flow', () => {
  it('creates and claims stream', async () => {
    // Create stream
    const streamId = await createStream(params);
    
    // Wait for confirmation
    await waitForStreamCreation(streamId);
    
    // Claim tokens
    const claimed = await claimStream(streamId);
    
    // Verify
    expect(claimed).toBeGreaterThan(0);
  });
});
```

---

## Coverage Reporting

### Generate Coverage Report

```bash
# Contract coverage
cd contracts/flux_engine
cargo tarpaulin --out Html

# Frontend coverage
cd frontend
npm test -- --coverage

# View reports
# Contract: tarpaulin-report.html
# Frontend: coverage/lcov-report/index.html
```

### Coverage Goals

```
Target Coverage: 95%+ (critical paths)

By Category:
├─ Authorization: 100%
├─ Arithmetic: 100%
├─ State Updates: 100%
├─ Error Handling: 95%
├─ Edge Cases: 90%
└─ UI Components: 80%
```

---

## Troubleshooting Tests

### Test Fails: "Overflow"

**Problem**: Math test fails with overflow error

**Solution**:
```rust
// Check bounds
let result = checked_operation(a, b)?;  // Use checked variant
assert!(result.is_ok());  // Verify no overflow
```

### Test Fails: "Not Authorized"

**Problem**: Authorization test fails

**Solution**:
```rust
// Ensure proper authorization
sender.require_auth();  // Add auth requirement
test_function_with_auth(&sender);  // Pass authorized sender
```

### Test Hangs

**Problem**: Test takes too long

**Solution**:
```bash
# Run with timeout
timeout 10 cargo test test_name

# Or run in single thread
cargo test -- --test-threads=1
```

---

## CI/CD Setup

### GitHub Actions Example

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      
      - name: Run tests
        run: make test
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

### GitLab CI Example

```yaml
test:
  image: rust:latest
  script:
    - rustup target add wasm32-unknown-unknown
    - make test
  coverage: '/^TOTAL.*\s+(\d+%)$/'
```

---

## Test Performance

### Optimize Test Speed

```bash
# Run tests in parallel (default)
cargo test

# Run tests sequentially (if needed)
cargo test -- --test-threads=1

# Show test times
cargo test -- --nocapture --test-threads=1
```

### Expected Test Times

```
Stream math tests:        ~100ms
Fixed-point tests:        ~200ms
Authorization tests:      ~150ms
Balance tests:            ~100ms
Integration tests:        ~500ms
Frontend tests:           ~1s
---
Total:                    ~2-3s
```

---

## Continuous Integration

### Pre-Commit Checks

Before committing:
```bash
# Run tests
make test

# Run linter
make lint

# Check code formatting
cargo fmt --check
```

### Pre-Push Checks

Before pushing:
```bash
# Full test suite
make test

# Coverage check
cargo tarpaulin --out Stdout

# Build check
make build-contracts
```

---

## Test Best Practices

1. **Test One Thing**: Each test should verify one behavior
2. **Clear Names**: Test names describe what is being tested
3. **Setup, Execute, Assert**: Organize test in three phases
4. **No Side Effects**: Tests should be independent
5. **Fast Execution**: Tests should complete in < 10ms each
6. **Deterministic**: Tests should produce consistent results
7. **Document Edge Cases**: Explain why edge cases are important

---

**See Also**: [DEPLOYMENT_GUIDE.md](./DEPLOYMENT_GUIDE.md) for verification procedures
