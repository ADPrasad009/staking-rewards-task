use anchor_lang::prelude::*;
use anchor_spl::token::{Token, Mint, TokenAccount};

use crate::state::Pool;

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    pub stake_mint: Account<'info, Mint>,
    pub reward_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        space = Pool::LEN,
        seeds = [b"pool", stake_mint.key().as_ref(), reward_mint.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        init,
        payer = admin,
        seeds = [b"stake_vault", pool.key().as_ref()],
        bump,
        token::mint = stake_mint,
        token::authority = pool,
    )]
    pub stake_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = admin,
        seeds = [b"reward_vault", pool.key().as_ref()],
        bump,
        token::mint = reward_mint,
        token::authority = pool,
    )]
    pub reward_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_pool(
    ctx: Context<InitializePool>,
    reward_rate_per_second: u64,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    pool.admin = ctx.accounts.admin.key();
    pool.stake_mint = ctx.accounts.stake_mint.key();
    pool.reward_mint = ctx.accounts.reward_mint.key();
    pool.reward_rate_per_second = reward_rate_per_second;
    pool.total_staked = 0;
    pool.bump = ctx.bumps.pool;

    Ok(())
}