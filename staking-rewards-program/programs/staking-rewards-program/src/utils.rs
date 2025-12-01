use anchor_lang::prelude::*;

use crate::state::{UserStake, Pool};
use crate::errors::StakingError;

pub fn update_rewards(user: &mut UserStake, pool: &Pool) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;

    if user.amount_staked == 0 || pool.total_staked == 0 {
        user.last_update = now;
        return Ok(());
    }

    let elapsed = now - user.last_update;

    if elapsed <= 0 {
        return Ok(());
    }

    let reward =
        (user.amount_staked * pool.reward_rate_per_second as u128 * elapsed as u128)
            / pool.total_staked;

    user.pending_rewards = user
        .pending_rewards
        .checked_add(reward)
        .ok_or(StakingError::Overflow)?;

    user.last_update = now;

    Ok(())
}