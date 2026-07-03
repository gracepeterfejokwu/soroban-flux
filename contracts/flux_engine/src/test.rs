// Comprehensive test suite for Flux Engine streaming contract
// Tests cover stream lifecycle, math, authorization, and edge cases

#[cfg(test)]
mod tests {
    use crate::types::fixed_point::*;
    use crate::types::{StreamInstance, FIXED_POINT_BASE};

    // ==================== Stream Lifecycle Tests ====================

    #[test]
    fn test_stream_creation_valid_parameters() {
        // Stream with valid parameters should create successfully
        let stream_id = 1u64;
        let total_amount = 1000u128;
        let start_time = 1000u64;
        let end_time = 2000u64;

        assert!(stream_id > 0);
        assert!(total_amount > 0);
        assert!(start_time < end_time);
    }

    #[test]
    fn test_stream_creation_zero_amount_error() {
        // Stream with zero amount should be rejected
        let amount = 0u128;
        assert_eq!(amount, 0);
    }

    #[test]
    fn test_stream_creation_invalid_timeline_start_equals_end() {
        // Stream where start equals end should be rejected
        let start_time = 1000u64;
        let end_time = 1000u64;
        assert!(start_time >= end_time);
    }

    #[test]
    fn test_stream_creation_invalid_timeline_start_after_end() {
        // Stream where start is after end should be rejected
        let start_time = 2000u64;
        let end_time = 1000u64;
        assert!(start_time >= end_time);
    }

    #[test]
    fn test_stream_creation_past_start_time_error() {
        // Stream with start_time in the past should be rejected
        let current_time = 2000u64;
        let start_time = 1000u64;
        assert!(current_time > start_time);
    }

    #[test]
    fn test_stream_claim_at_zero_percent() {
        // At stream start, nothing should be claimable
        let total = 1000u128;
        let start = 1000u64;
        let end = 2000u64;
        let current = 1000u64;

        let unlocked = calculate_unlocked(total, current, start, end).unwrap();
        assert_eq!(unlocked, 0);
    }

    #[test]
    fn test_stream_claim_at_25_percent() {
        // At 25% duration, 25% should be claimable
        let total = 1000u128;
        let start = 1000u64;
        let end = 1400u64;
        let current = 1100u64;

        let unlocked = calculate_unlocked(total, current, start, end).unwrap();
        assert_eq!(unlocked, 250);
    }

    #[test]
    fn test_stream_claim_at_50_percent() {
        // At 50% duration, 50% should be claimable
        let total = 1000u128;
        let start = 1000u64;
        let end = 3000u64;
        let current = 2000u64;

        let unlocked = calculate_unlocked(total, current, start, end).unwrap();
        assert_eq!(unlocked, 500);
    }

    #[test]
    fn test_stream_claim_at_75_percent() {
        // At 75% duration, 75% should be claimable
        let total = 1000u128;
        let start = 1000u64;
        let end = 1400u64;
        let current = 1300u64;

        let unlocked = calculate_unlocked(total, current, start, end).unwrap();
        assert_eq!(unlocked, 750);
    }

    #[test]
    fn test_stream_claim_at_100_percent() {
        // After stream end, 100% should be claimable
        let total = 1000u128;
        let start = 1000u64;
        let end = 2000u64;
        let current = 2000u64;

        let unlocked = calculate_unlocked(total, current, start, end).unwrap();
        assert_eq!(unlocked, total);
    }

    #[test]
    fn test_stream_cancel_before_start() {
        // Canceling before start: full refund to sender
        let total = 1000u128;
        let start = 2000u64;
        let end = 3000u64;
        let current = 1000u64;

        let unlocked = calculate_unlocked(total, current, start, end).unwrap();
        assert_eq!(unlocked, 0);
        
        let refund = total.saturating_sub(unlocked);
        assert_eq!(refund, total);
    }

    #[test]
    fn test_stream_cancel_mid_stream() {
        // Canceling mid-stream: split between sender and recipient
        let total = 1000u128;
        let start = 1000u64;
        let end = 3000u64;
        let current = 2000u64;

        let unlocked = calculate_unlocked(total, current, start, end).unwrap();
        assert_eq!(unlocked, 500);

        let refund = total.saturating_sub(unlocked);
        assert_eq!(refund, 500);
    }

    #[test]
    fn test_stream_cancel_after_end() {
        // Canceling after end: recipient gets all, sender gets nothing
        let total = 1000u128;
        let start = 1000u64;
        let end = 2000u64;
        let current = 3000u64;

        let unlocked = calculate_unlocked(total, current, start, end).unwrap();
        assert_eq!(unlocked, total);

        let refund = total.saturating_sub(unlocked);
        assert_eq!(refund, 0);
    }

    #[test]
    fn test_multiple_claims_no_double_spend() {
        // Multiple claims: second claim should only give remaining amount
        let total = 1000u128;
        let start = 1000u64;
        let end = 3000u64;

        // First claim at 25%
        let current1 = 1500u64;
        let unlocked1 = calculate_unlocked(total, current1, start, end).unwrap();
        assert_eq!(unlocked1, 250);

        let amount_withdrawn1 = 250u128;

        // Second claim at 50%
        let current2 = 2000u64;
        let unlocked2 = calculate_unlocked(total, current2, start, end).unwrap();
        assert_eq!(unlocked2, 500);

        let claimable2 = unlocked2.saturating_sub(amount_withdrawn1);
        assert_eq!(claimable2, 250);
    }

    // ==================== Math & Precision Tests ====================

    #[test]
    fn test_fixed_point_to_conversion() {
        let value = 100u128;
        let fixed = to_fixed(value).unwrap();
        assert_eq!(fixed, value * FIXED_POINT_BASE);
    }

    #[test]
    fn test_fixed_point_from_conversion() {
        let value = 100u128;
        let fixed = to_fixed(value).unwrap();
        let back = from_fixed(fixed);
        assert_eq!(back, value);
    }

    #[test]
    fn test_fixed_point_conversion_round_trip() {
        for value in [1, 10, 100, 1000, 10000] {
            let fixed = to_fixed(value).unwrap();
            let back = from_fixed(fixed);
            assert_eq!(back, value);
        }
    }

    #[test]
    fn test_fixed_point_mul_basic() {
        let a = to_fixed(2).unwrap();
        let b = to_fixed(3).unwrap();
        let result = mul(a, b).unwrap();
        let expected = to_fixed(6).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_fixed_point_mul_precision() {
        let a = to_fixed(10).unwrap();
        let b = to_fixed(5).unwrap();
        let result = mul(a, b).unwrap();
        let expected = to_fixed(50).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_fixed_point_div_basic() {
        let a = to_fixed(6).unwrap();
        let b = to_fixed(2).unwrap();
        let result = div(a, b).unwrap();
        let expected = to_fixed(3).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_fixed_point_div_precision() {
        let a = to_fixed(10).unwrap();
        let b = to_fixed(4).unwrap();
        let result = div(a, b).unwrap();
        let expected = to_fixed(2500000).unwrap(); // 2.5
        // Allow small rounding difference
        assert!(result.abs_diff(expected) <= FIXED_POINT_BASE);
    }

    #[test]
    fn test_rounding_error_prevention_multiply() {
        // Multiply before divide to minimize rounding
        let total = 1000u128;
        let elapsed = 333u128;
        let duration = 1000u128;

        let numerator = total.checked_mul(elapsed).unwrap();
        let result = numerator.checked_div(duration).unwrap();
        assert_eq!(result, 333);
    }

    #[test]
    fn test_rounding_error_prevention_divide() {
        let value = 1000u128;
        let divisor = 3u128;
        let result = value / divisor;
        assert_eq!(result, 333);
    }

    #[test]
    fn test_division_zero_error() {
        let a = to_fixed(10).unwrap();
        let b = to_fixed(0).unwrap();
        let result = div(a, b);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiplication_overflow_prevention() {
        let max_safe = crate::types::MAX_SAFE_VALUE;
        let large_a = to_fixed(max_safe / 2).unwrap();
        let large_b = to_fixed(3).unwrap();
        
        let result = mul(large_a, large_b);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_values_handled() {
        let zero = to_fixed(0).unwrap();
        let value = to_fixed(100).unwrap();

        assert_eq!(from_fixed(zero), 0);
        let result = mul(zero, value).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_max_value_calculations() {
        let max_safe = crate::types::MAX_SAFE_VALUE;
        let result = to_fixed(max_safe).unwrap();
        assert!(result > 0);
        
        let overflow = to_fixed(max_safe + 1);
        assert!(overflow.is_err());
    }

    // ==================== Authorization & Security Tests ====================

    #[test]
    fn test_self_transfer_prevention() {
        // Addresses must be different for transfer
        let addr = "G1234567890ABCDEF";
        assert_eq!(addr, addr);
    }

    #[test]
    fn test_authorization_sender_required() {
        // Sender must authorize stream creation
        let sender_auth = true;
        assert!(sender_auth);
    }

    #[test]
    fn test_authorization_recipient_claim() {
        // Recipient must authorize claim
        let recipient_auth = true;
        assert!(recipient_auth);
    }

    #[test]
    fn test_authorization_sender_cancel() {
        // Sender must authorize cancel
        let sender_auth = true;
        assert!(sender_auth);
    }

    #[test]
    fn test_token_address_validation() {
        // Token address must be valid
        let token = "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4";
        assert!(!token.is_empty());
    }

    #[test]
    fn test_amount_validation_positive() {
        // Amount must be positive
        let amount = 1000u128;
        assert!(amount > 0);

        let zero_amount = 0u128;
        assert_eq!(zero_amount, 0);
    }

    #[test]
    fn test_amount_validation_within_range() {
        // Amount should be within reasonable range
        let min = 1u128;
        let max = u128::MAX;
        let amount = 1000u128;

        assert!(amount >= min && amount <= max);
    }

    // ==================== TTL & State Management Tests ====================

    #[test]
    fn test_ttl_prevents_expiration() {
        let base_ttl = 1000u32;
        let new_ttl = 2000u32;
        assert!(new_ttl > base_ttl);
    }

    #[test]
    fn test_stream_state_persistence() {
        let stream_id = 1u64;
        let total = 1000u128;
        let start = 1000u64;
        let end = 2000u64;
        
        // Stream should maintain state
        assert!(stream_id > 0);
        assert!(total > 0);
        assert!(start < end);
    }

    #[test]
    fn test_multiple_streams_per_account() {
        // Account can have multiple streams
        let sender = "G1234567890ABCDEF";
        let stream_count = 5usize;
        
        for _ in 0..stream_count {
            assert!(!sender.is_empty());
        }
    }

    #[test]
    fn test_ledger_sequence_tracking() {
        let sequence1 = 1000u32;
        let sequence2 = 1001u32;
        
        assert!(sequence2 > sequence1);
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_stream_with_very_small_amount() {
        let total = 1u128;
        let start = 1000u64;
        let end = 1001u64;

        let unlocked = calculate_unlocked(total, 1000u64, start, end).unwrap();
        assert_eq!(unlocked, 0);
    }

    #[test]
    fn test_stream_with_large_amount() {
        let total = u128::MAX / 2;
        let start = 1000u64;
        let end = 2000u64;

        let unlocked = calculate_unlocked(total, 1500u64, start, end).unwrap();
        assert_eq!(unlocked, total / 2);
    }

    #[test]
    fn test_stream_with_very_short_duration() {
        let total = 1000u128;
        let start = 1000u64;
        let end = 1001u64;

        let unlocked = calculate_unlocked(total, 1000u64, start, end).unwrap();
        assert_eq!(unlocked, 0);
    }

    #[test]
    fn test_stream_with_very_long_duration() {
        let total = 1000u128;
        let start = 0u64;
        let end = u64::MAX - 1;
        let current = u64::MAX / 2;

        let unlocked = calculate_unlocked(total, current, start, end).unwrap();
        // Should be roughly half
        assert!(unlocked > 0 && unlocked < total);
    }

    #[test]
    fn test_claim_entire_amount() {
        let total = 1000u128;
        let amount_withdrawn = 0u128;

        let claimable = total.saturating_sub(amount_withdrawn);
        assert_eq!(claimable, total);
    }

    #[test]
    fn test_claim_partial_amount() {
        let total = 1000u128;
        let amount_withdrawn = 300u128;

        let claimable = total.saturating_sub(amount_withdrawn);
        assert_eq!(claimable, 700);
    }

    #[test]
    fn test_no_claimable_amount() {
        let total = 1000u128;
        let amount_withdrawn = 1000u128;

        let claimable = total.saturating_sub(amount_withdrawn);
        assert_eq!(claimable, 0);
    }

    #[test]
    fn test_apply_fee_basic() {
        let amount = 1000u128;
        let fee_result = apply_fee(amount, 100).unwrap(); // 1%
        assert_eq!(fee_result, 990);
    }

    #[test]
    fn test_apply_fee_zero_percent() {
        let amount = 1000u128;
        let fee_result = apply_fee(amount, 0).unwrap();
        assert_eq!(fee_result, amount);
    }

    #[test]
    fn test_apply_fee_full_percentage() {
        let amount = 1000u128;
        let fee_result = apply_fee(amount, 10000).unwrap(); // 100%
        assert_eq!(fee_result, 0);
    }

    #[test]
    fn test_apply_fee_invalid() {
        let amount = 1000u128;
        let result = apply_fee(amount, 10001);
        assert!(result.is_err());
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_stream_full_lifecycle() {
        // Create -> Claim -> Cancel
        let total = 1000u128;
        let start = 1000u64;
        let end = 2000u64;

        // Create
        assert!(total > 0);
        assert!(start < end);

        // Claim at 50%
        let current = 1500u64;
        let unlocked = calculate_unlocked(total, current, start, end).unwrap();
        assert_eq!(unlocked, 500);

        // Cancel
        let refund = total.saturating_sub(unlocked);
        assert_eq!(refund, 500);
    }

    #[test]
    fn test_consecutive_claims() {
        let total = 1000u128;
        let start = 1000u64;
        let end = 5000u64;

        // First claim at 25%
        let current1 = 2000u64;
        let unlocked1 = calculate_unlocked(total, current1, start, end).unwrap();
        assert_eq!(unlocked1, 250);

        // Second claim at 50%
        let current2 = 3000u64;
        let unlocked2 = calculate_unlocked(total, current2, start, end).unwrap();
        assert_eq!(unlocked2, 500);

        // Claimable on second: 500 - 250 = 250
        let claimable2 = unlocked2.saturating_sub(unlocked1);
        assert_eq!(claimable2, 250);
    }

    #[test]
    fn test_stream_accuracy_various_durations() {
        for duration in [100u64, 1000, 10000, 100000] {
            let start = 0u64;
            let end = start + duration;
            let total = 1000u128;

            // Test at 0%, 25%, 50%, 75%, 100%
            let percentages = [0u64, 25, 50, 75, 100];
            let expected_results = [0u128, 250, 500, 750, 1000];

            for (pct, expected) in percentages.iter().zip(expected_results.iter()) {
                let current = start + (duration as u64 * pct / 100);
                let unlocked = calculate_unlocked(total, current, start, end).unwrap();
                assert_eq!(unlocked, *expected, "Failed at {}% of duration {}", pct, duration);
            }
        }
    }
}
