use anchor_lang::prelude::*;
use crate::state::{Pool, UserStakeAccount};
use crate::errors::StakingError;

/// Return current unix timestamp as i64
pub fn now_ts() -> Result<i64> {
    Ok(Clock::get()?.unix_timestamp)
}

/// Accrue pending rewards for a user according to spec's integer-only formula.
/// Uses checked arithmetic. Updates user.pending_rewards and user.last_update_timestamp.
/// Return Err(StakingError::Overflow) on checked overflow or MathError for division issues.
pub fn compute_reward_delta(user: &mut UserStakeAccount, pool: &Pool, current_ts: i64) -> Result<()> {
    if current_ts < user.last_update_timestamp {
        // do nothing on backward clock
        return Ok(());
    }

    let elapsed_i64 = current_ts.checked_sub(user.last_update_timestamp).unwrap_or(0);
    if elapsed_i64 <= 0 {
        // nothing to accrue
        user.last_update_timestamp = current_ts;
        return Ok(());
    }

    if pool.total_staked == 0 {
        user.last_update_timestamp = current_ts;
        return Ok(());
    }

    let elapsed_u128 = elapsed_i64 as u128;

    // reward_share = user.amount_staked * pool.reward_rate_per_second * elapsed_u128
    let reward_share = user
        .amount_staked
        .checked_mul(pool.reward_rate_per_second)
        .ok_or(StakingError::Overflow)?
        .checked_mul(elapsed_u128)
        .ok_or(StakingError::Overflow)?;

    // user_delta = reward_share / pool.total_staked
    let user_delta = reward_share
        .checked_div(pool.total_staked)
        .ok_or(StakingError::MathError)?;

    user.pending_rewards = user
        .pending_rewards
        .checked_add(user_delta)
        .ok_or(StakingError::Overflow)?;

    user.last_update_timestamp = current_ts;
    Ok(())
}
