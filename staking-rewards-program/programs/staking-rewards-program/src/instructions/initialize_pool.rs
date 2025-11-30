use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::state::Pool;

#[derive(Accounts)]
#[instruction(reward_rate_per_second: u128)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    /// stake mint
    pub stake_mint: InterfaceAccount<'info, Mint>,

    /// reward mint
    pub reward_mint: InterfaceAccount<'info, Mint>,

    /// Pool PDA: ["pool", stake_mint, reward_mint]
    #[account(
        init,
        payer = admin,
        space = Pool::LEN,
        seeds = [b"pool", stake_mint.key().as_ref(), reward_mint.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, Pool>,

    /// pool stake vault PDA (Unchecked) - will be created via CPI
    #[account(seeds = [b"pool_vault", pool.key().as_ref()], bump)]
    pub pool_stake_vault: UncheckedAccount<'info>,

    /// reward vault PDA (Unchecked) - will be created via CPI
    #[account(seeds = [b"reward_vault", pool.key().as_ref()], bump)]
    pub reward_vault: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
