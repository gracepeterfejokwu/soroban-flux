use soroban_sdk::{Address, Symbol};

/// Fixed-point representation with 7 decimal places (10^7 base)
pub const FIXED_POINT_BASE: u128 = 10_000_000;

/// Maximum safe value to prevent overflow in calculations
pub const MAX_SAFE_VALUE: u128 = u128::MAX / FIXED_POINT_BASE;

/// Stream instance data structure
#[derive(Clone)]
pub struct StreamInstance {
    /// Stream ID (unique identifier)
    pub stream_id: u64,
    /// Sender address (funds the stream)
    pub sender: Address,
    /// Recipient address (receives the stream)
    pub recipient: Address,
    /// Token contract address
    pub token: Address,
    /// Total amount to be streamed
    pub total_amount: u128,
    /// Stream start time (unix timestamp)
    pub start_time: u64,
    /// Stream end time (unix timestamp)
    pub end_time: u64,
    /// Amount already withdrawn by recipient
    pub amount_withdrawn: u128,
    /// TTL ledger entry for state management
    pub ttl: u32,
}

/// Result type for contract operations
pub type ContractResult<T> = Result<T, Symbol>;

/// Helper functions for fixed-point arithmetic and stream calculations
pub mod fixed_point {
    use super::*;

    /// Convert integer to fixed-point
    pub fn to_fixed(value: u128) -> ContractResult<u128> {
        if value > MAX_SAFE_VALUE {
            return Err(Symbol::short("overflow"));
        }
        Ok(value.saturating_mul(FIXED_POINT_BASE))
    }

    /// Convert fixed-point to integer (truncates)
    pub fn from_fixed(value: u128) -> u128 {
        value / FIXED_POINT_BASE
    }

    /// Multiply two fixed-point numbers: (a * b) / BASE
    pub fn mul(a: u128, b: u128) -> ContractResult<u128> {
        let result = a
            .checked_mul(b)
            .ok_or_else(|| Symbol::short("overflow"))?;
        Ok(result / FIXED_POINT_BASE)
    }

    /// Divide two fixed-point numbers: (a * BASE) / b
    pub fn div(a: u128, b: u128) -> ContractResult<u128> {
        if b == 0 {
            return Err(Symbol::short("div_zero"));
        }
        let result = a
            .checked_mul(FIXED_POINT_BASE)
            .ok_or_else(|| Symbol::short("overflow"))?
            .checked_div(b)
            .ok_or_else(|| Symbol::short("div_zero"))?;
        Ok(result)
    }

    /// Calculate time-weighted unlocked amount
    /// unlocked = total_amount * (current_time - start_time) / (end_time - start_time)
    pub fn calculate_unlocked(
        total_amount: u128,
        current_time: u64,
        start_time: u64,
        end_time: u64,
    ) -> ContractResult<u128> {
        // If current time is before start, nothing is unlocked
        if current_time < start_time {
            return Ok(0);
        }

        // If current time is at or after end, all is unlocked
        if current_time >= end_time {
            return Ok(total_amount);
        }

        // Time-weighted calculation: total * (current - start) / (end - start)
        let elapsed = (current_time - start_time) as u128;
        let duration = (end_time - start_time) as u128;

        // Multiply first to maintain precision, then divide
        let numerator = total_amount
            .checked_mul(elapsed)
            .ok_or_else(|| Symbol::short("overflow"))?;
        let unlocked = numerator
            .checked_div(duration)
            .ok_or_else(|| Symbol::short("div_zero"))?;

        Ok(unlocked)
    }

    /// Apply percentage fee (in basis points)
    pub fn apply_fee(amount: u128, fee_bps: u16) -> ContractResult<u128> {
        if fee_bps > 10000 {
            return Err(Symbol::short("invalid_fee"));
        }
        let fee = amount
            .checked_mul(fee_bps as u128)
            .ok_or_else(|| Symbol::short("overflow"))?
            / 10000;
        Ok(amount.saturating_sub(fee))
    }
}

#[cfg(test)]
mod tests {
    use super::fixed_point::*;

    #[test]
    fn test_to_fixed_conversion() {
        let result = to_fixed(100).unwrap();
        assert_eq!(result, 100 * FIXED_POINT_BASE);
    }

    #[test]
    fn test_from_fixed_conversion() {
        let fixed = 100 * FIXED_POINT_BASE;
        assert_eq!(from_fixed(fixed), 100);
    }

    #[test]
    fn test_fixed_point_mul() {
        let a = to_fixed(2).unwrap();
        let b = to_fixed(3).unwrap();
        let result = mul(a, b).unwrap();
        assert_eq!(result, to_fixed(6).unwrap());
    }

    #[test]
    fn test_fixed_point_div() {
        let a = to_fixed(6).unwrap();
        let b = to_fixed(2).unwrap();
        let result = div(a, b).unwrap();
        assert_eq!(result, to_fixed(3).unwrap());
    }

    #[test]
    fn test_apply_fee() {
        let amount = to_fixed(1000).unwrap();
        let result = apply_fee(amount, 100).unwrap(); // 1% fee
        let expected = to_fixed(990).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_overflow_protection() {
        assert!(to_fixed(MAX_SAFE_VALUE + 1).is_err());
    }

    #[test]
    fn test_div_by_zero() {
        let a = to_fixed(10).unwrap();
        let b = to_fixed(0).unwrap();
        assert!(div(a, b).is_err());
    }

    #[test]
    fn test_calculate_unlocked_before_start() {
        let total = 1000u128;
        let current = 100u64;
        let start = 200u64;
        let end = 300u64;
        
        let unlocked = calculate_unlocked(total, current, start, end).unwrap();
        assert_eq!(unlocked, 0);
    }

    #[test]
    fn test_calculate_unlocked_after_end() {
        let total = 1000u128;
        let current = 400u64;
        let start = 200u64;
        let end = 300u64;
        
        let unlocked = calculate_unlocked(total, current, start, end).unwrap();
        assert_eq!(unlocked, total);
    }

    #[test]
    fn test_calculate_unlocked_mid_stream() {
        let total = 1000u128;
        let current = 250u64;
        let start = 200u64;
        let end = 300u64;
        
        // (250 - 200) / (300 - 200) = 50 / 100 = 0.5, so 500
        let unlocked = calculate_unlocked(total, current, start, end).unwrap();
        assert_eq!(unlocked, 500);
    }

    #[test]
    fn test_calculate_unlocked_at_start() {
        let total = 1000u128;
        let current = 200u64;
        let start = 200u64;
        let end = 300u64;
        
        let unlocked = calculate_unlocked(total, current, start, end).unwrap();
        assert_eq!(unlocked, 0);
    }

    #[test]
    fn test_calculate_unlocked_at_end() {
        let total = 1000u128;
        let current = 300u64;
        let start = 200u64;
        let end = 300u64;
        
        let unlocked = calculate_unlocked(total, current, start, end).unwrap();
        assert_eq!(unlocked, total);
    }
}

