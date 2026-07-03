#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, token};

mod types;
pub use types::{fixed_point, StreamInstance, FIXED_POINT_BASE};

// Storage keys for contract state
const KEY_STREAM_COUNTER: Symbol = Symbol::short("str_cnt");
const KEY_STREAM_PREFIX: Symbol = Symbol::short("stream");
const KEY_INITIALIZED: Symbol = Symbol::short("init");

#[contract]
pub struct FluxEngine;

#[contractimpl]
impl FluxEngine {
    /// Initialize the contract
    ///
    /// # Arguments
    /// * `env` - Contract environment
    pub fn initialize(env: Env) -> Result<(), Symbol> {
        // Checks: Ensure not already initialized
        if env.storage().instance().has(&KEY_INITIALIZED) {
            return Err(Symbol::short("already_init"));
        }

        // Effects: Set initialization flag and counter
        env.storage().instance().set(&KEY_INITIALIZED, &true);
        env.storage().instance().set(&KEY_STREAM_COUNTER, &0u64);

        Ok(())
    }

    /// Create a new stream
    ///
    /// # Arguments
    /// * `env` - Contract environment
    /// * `sender` - Stream creator and funder
    /// * `recipient` - Stream recipient
    /// * `token` - Token contract address
    /// * `amount` - Total amount to stream
    /// * `start_time` - Stream start time (unix timestamp)
    /// * `end_time` - Stream end time (unix timestamp)
    ///
    /// # Returns
    /// * Stream ID on success
    pub fn create_stream(
        env: Env,
        sender: Address,
        recipient: Address,
        token: Address,
        amount: u128,
        start_time: u64,
        end_time: u64,
    ) -> Result<u64, Symbol> {
        // Checks: Require authorization from sender
        sender.require_auth();

        // Checks: Ensure contract is initialized
        env.storage()
            .instance()
            .get::<Symbol, bool>(&KEY_INITIALIZED)
            .map_err(|_| Symbol::short("not_init"))?;

        // Checks: Sender and recipient must be different
        if sender == recipient {
            return Err(Symbol::short("same_addr"));
        }

        // Checks: Amount must be positive
        if amount == 0 {
            return Err(Symbol::short("zero_amount"));
        }

        // Checks: Timeline must be valid (start < end)
        if start_time >= end_time {
            return Err(Symbol::short("invalid_timeline"));
        }

        // Checks: Start time cannot be in the past
        let current_time = env.ledger().timestamp();
        if current_time > start_time {
            return Err(Symbol::short("past_start"));
        }

        // Effects: Get and increment stream counter
        let mut counter: u64 = env
            .storage()
            .instance()
            .get(&KEY_STREAM_COUNTER)
            .unwrap_or(0);

        counter = counter
            .checked_add(1)
            .ok_or_else(|| Symbol::short("overflow"))?;

        let stream_id = counter;

        // Effects: Create stream instance
        let stream = StreamInstance {
            stream_id,
            sender: sender.clone(),
            recipient: recipient.clone(),
            token: token.clone(),
            total_amount: amount,
            start_time,
            end_time,
            amount_withdrawn: 0,
            ttl: env.ledger().sequence() + 1_000_000,
        };

        // Effects: Store stream and update counter
        let stream_key = Self::stream_key(stream_id);
        env.storage()
            .persistent()
            .set(&stream_key, &stream);
        env.storage().instance().set(&KEY_STREAM_COUNTER, &counter);

        // Interactions: Emit event
        env.events()
            .publish((Symbol::short("stream_created"),), (stream_id, sender, recipient, amount));

        Ok(stream_id)
    }

    /// Claim unlocked tokens from a stream
    ///
    /// # Arguments
    /// * `env` - Contract environment
    /// * `stream_id` - ID of the stream to claim from
    /// * `destination` - Destination address for claimed tokens
    ///
    /// # Returns
    /// * Amount claimed on success
    pub fn claim_stream(
        env: Env,
        stream_id: u64,
        destination: Address,
    ) -> Result<u128, Symbol> {
        // Checks: Get stream
        let mut stream = Self::get_stream_internal(&env, stream_id)?;

        // Checks: Require authorization from recipient
        stream.recipient.require_auth();

        // Checks: Recipient must match destination or be the destination caller
        if stream.recipient != destination && destination != stream.recipient {
            return Err(Symbol::short("auth_failed"));
        }

        // Effects: Calculate unlocked amount
        let current_time = env.ledger().timestamp();
        let unlocked = fixed_point::calculate_unlocked(
            stream.total_amount,
            current_time,
            stream.start_time,
            stream.end_time,
        )?;

        // Checks: Calculate claimable amount
        let claimable = unlocked
            .checked_sub(stream.amount_withdrawn)
            .ok_or_else(|| Symbol::short("insufficient"))?;

        if claimable == 0 {
            return Err(Symbol::short("insufficient"));
        }

        // Effects: Update stream state (Checks-Effects-Interactions pattern)
        stream.amount_withdrawn = unlocked;
        let stream_key = Self::stream_key(stream_id);
        env.storage()
            .persistent()
            .set(&stream_key, &stream);

        // Interactions: Transfer tokens to destination
        let token_client = token::Client::new(&env, &stream.token);
        token_client.transfer(
            &stream.sender,
            &destination,
            &(claimable as i128),
        );

        // Interactions: Emit event
        env.events()
            .publish((Symbol::short("stream_claimed"),), (stream_id, claimable, destination));

        Ok(claimable)
    }

    /// Cancel a stream
    ///
    /// # Arguments
    /// * `env` - Contract environment
    /// * `stream_id` - ID of the stream to cancel
    ///
    /// # Returns
    /// * Tuple of (refund_to_sender, payout_to_recipient) on success
    pub fn cancel_stream(env: Env, stream_id: u64) -> Result<(u128, u128), Symbol> {
        // Checks: Get stream
        let stream = Self::get_stream_internal(&env, stream_id)?;

        // Checks: Require authorization from sender (stream creator)
        stream.sender.require_auth();

        // Effects: Calculate amounts
        let current_time = env.ledger().timestamp();
        let unlocked = fixed_point::calculate_unlocked(
            stream.total_amount,
            current_time,
            stream.start_time,
            stream.end_time,
        )?;

        let payout_to_recipient = unlocked
            .checked_sub(stream.amount_withdrawn)
            .ok_or_else(|| Symbol::short("insufficient"))?;

        let refund_to_sender = stream.total_amount
            .checked_sub(unlocked)
            .ok_or_else(|| Symbol::short("insufficient"))?;

        // Effects: Delete stream
        let stream_key = Self::stream_key(stream_id);
        env.storage().persistent().remove(&stream_key);

        // Interactions: Transfer tokens
        let token_client = token::Client::new(&env, &stream.token);

        // Refund to sender if any balance remaining
        if refund_to_sender > 0 {
            token_client.transfer(
                &stream.sender,
                &stream.sender,
                &(refund_to_sender as i128),
            );
        }

        // Payout to recipient if earned
        if payout_to_recipient > 0 {
            token_client.transfer(
                &stream.sender,
                &stream.recipient,
                &(payout_to_recipient as i128),
            );
        }

        // Interactions: Emit event
        env.events().publish(
            (Symbol::short("stream_canceled"),),
            (stream_id, refund_to_sender, payout_to_recipient),
        );

        Ok((refund_to_sender, payout_to_recipient))
    }

    /// Extend TTL of a stream to prevent state expiration
    ///
    /// # Arguments
    /// * `env` - Contract environment
    /// * `stream_id` - ID of the stream
    ///
    /// # Returns
    /// * New TTL value on success
    pub fn extend_ttl(env: Env, stream_id: u64) -> Result<u32, Symbol> {
        // Checks: Get stream
        let mut stream = Self::get_stream_internal(&env, stream_id)?;

        // Effects: Update TTL
        let new_ttl = env.ledger().sequence() + 1_000_000;
        stream.ttl = new_ttl;

        // Effects: Store updated stream
        let stream_key = Self::stream_key(stream_id);
        env.storage()
            .persistent()
            .set(&stream_key, &stream);

        // Interactions: Emit event
        env.events()
            .publish((Symbol::short("ttl_extended"),), (stream_id, new_ttl));

        Ok(new_ttl)
    }

    /// Get stream details
    ///
    /// # Arguments
    /// * `env` - Contract environment
    /// * `stream_id` - ID of the stream
    ///
    /// # Returns
    /// * Stream tuple (sender, recipient, token, total_amount, start_time, end_time, amount_withdrawn)
    pub fn get_stream(env: Env, stream_id: u64) -> Result<(Address, Address, Address, u128, u64, u64, u128), Symbol> {
        let stream = Self::get_stream_internal(&env, stream_id)?;
        Ok((
            stream.sender,
            stream.recipient,
            stream.token,
            stream.total_amount,
            stream.start_time,
            stream.end_time,
            stream.amount_withdrawn,
        ))
    }

    /// Get balance (token balance helper)
    ///
    /// # Arguments
    /// * `env` - Contract environment
    /// * `token` - Token contract address
    /// * `account` - Account address
    ///
    /// # Returns
    /// * Account balance on token
    pub fn balance(env: Env, token: Address, account: Address) -> Result<u128, Symbol> {
        let token_client = token::Client::new(&env, &token);
        let balance = token_client.balance(&account);
        Ok(balance as u128)
    }

    /// Calculate claimable amount for a stream at current time
    ///
    /// # Arguments
    /// * `env` - Contract environment
    /// * `stream_id` - ID of the stream
    ///
    /// # Returns
    /// * Claimable amount on success
    pub fn get_claimable(env: Env, stream_id: u64) -> Result<u128, Symbol> {
        let stream = Self::get_stream_internal(&env, stream_id)?;
        let current_time = env.ledger().timestamp();

        let unlocked = fixed_point::calculate_unlocked(
            stream.total_amount,
            current_time,
            stream.start_time,
            stream.end_time,
        )?;

        let claimable = unlocked
            .checked_sub(stream.amount_withdrawn)
            .unwrap_or(0);

        Ok(claimable)
    }

    // Private helper functions

    /// Generate storage key for a stream
    fn stream_key(stream_id: u64) -> Symbol {
        // Use a symbol that encodes the stream ID
        // In production, this would use proper key derivation
        Symbol::short("stream")
    }

    /// Get stream from storage (internal helper)
    fn get_stream_internal(env: &Env, stream_id: u64) -> Result<StreamInstance, Symbol> {
        let stream_key = Self::stream_key(stream_id);
        env.storage()
            .persistent()
            .get::<Symbol, StreamInstance>(&stream_key)
            .map_err(|_| Symbol::short("non_existent"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_math_before_start() {
        let total = 1000u128;
        let current = 100u64;
        let start = 200u64;
        let end = 300u64;

        let unlocked = fixed_point::calculate_unlocked(total, current, start, end).unwrap();
        assert_eq!(unlocked, 0);
    }

    #[test]
    fn test_stream_math_after_end() {
        let total = 1000u128;
        let current = 400u64;
        let start = 200u64;
        let end = 300u64;

        let unlocked = fixed_point::calculate_unlocked(total, current, start, end).unwrap();
        assert_eq!(unlocked, total);
    }

    #[test]
    fn test_stream_math_mid_stream() {
        let total = 1000u128;
        let current = 250u64;
        let start = 200u64;
        let end = 300u64;

        let unlocked = fixed_point::calculate_unlocked(total, current, start, end).unwrap();
        assert_eq!(unlocked, 500);
    }

    #[test]
    fn test_stream_math_25_percent() {
        let total = 1000u128;
        let current = 225u64;
        let start = 200u64;
        let end = 300u64;

        let unlocked = fixed_point::calculate_unlocked(total, current, start, end).unwrap();
        assert_eq!(unlocked, 250);
    }

    #[test]
    fn test_stream_math_75_percent() {
        let total = 1000u128;
        let current = 275u64;
        let start = 200u64;
        let end = 300u64;

        let unlocked = fixed_point::calculate_unlocked(total, current, start, end).unwrap();
        assert_eq!(unlocked, 750);
    }

    #[test]
    fn test_fixed_point_basics() {
        let value = 100u128;
        let fixed = fixed_point::to_fixed(value).unwrap();
        let back = fixed_point::from_fixed(fixed);
        assert_eq!(back, value);
    }

    #[test]
    fn test_multiplication() {
        let a = fixed_point::to_fixed(2).unwrap();
        let b = fixed_point::to_fixed(3).unwrap();
        let result = fixed_point::mul(a, b).unwrap();
        let expected = fixed_point::to_fixed(6).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_division() {
        let a = fixed_point::to_fixed(10).unwrap();
        let b = fixed_point::to_fixed(2).unwrap();
        let result = fixed_point::div(a, b).unwrap();
        let expected = fixed_point::to_fixed(5).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_zero_division_error() {
        let a = fixed_point::to_fixed(10).unwrap();
        let b = fixed_point::to_fixed(0).unwrap();
        assert!(fixed_point::div(a, b).is_err());
    }

    #[test]
    fn test_overflow_protection() {
        assert!(fixed_point::to_fixed(types::MAX_SAFE_VALUE + 1).is_err());
    }
}
