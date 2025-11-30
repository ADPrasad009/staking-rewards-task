use anchor_lang::prelude::*;

/// Pool PDA: ["pool", stake_mint.as_ref(), reward_mint.as_ref()]
#[account]
pub struct Pool {
    pub admin: Pubkey,
    pub stake_mint: Pubkey,
    pub reward_mint: Pubkey,
    pub reward_rate_per_second: u128,
    pub total_staked: u128,
    pub bump: u8,
}

impl Pool {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 16 + 16 + 1;
}

/// UserStakeAccount PDA: ["user_stake", pool_pda.key().as_ref(), user_pubkey.as_ref()]
#[account]
pub struct UserStakeAccount {
    pub owner: Pubkey,
    pub amount_staked: u128,
    pub last_update_timestamp: i64,
    pub pending_rewards: u128,
    pub bump: u8,
}

impl UserStakeAccount {
    pub const LEN: usize = 8 + 32 + 16 + 8 + 16 + 1;
}
