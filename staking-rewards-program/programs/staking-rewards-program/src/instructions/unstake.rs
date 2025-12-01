use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::state::{Pool, UserStake};
use crate::errors::StakingError;
use crate::utils::update_rewards;

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [b"user_stake", pool.key().as_ref(), user.key().as_ref()],
        bump = user_stake.bump
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(
        mut,
        token::mint = pool.stake_mint,
        token::authority = user
    )]
    pub user_stake_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"stake_vault", pool.key().as_ref()],
        bump
    )]
    pub stake_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
    require!(amount > 0, StakingError::ZeroAmount);

    let user = &mut ctx.accounts.user_stake;
    
    require!(
        user.amount_staked >= amount as u128,
        StakingError::InsufficientFunds
    );

    // Get pool data BEFORE mutable operations
    let stake_bump = ctx.accounts.pool.bump;
    let stake_mint = ctx.accounts.pool.stake_mint;
    let reward_mint = ctx.accounts.pool.reward_mint;

    update_rewards(user, &ctx.accounts.pool)?;

    let seeds = &[
        b"pool",
        stake_mint.as_ref(),
        reward_mint.as_ref(),
        &[stake_bump],
    ];

    let signer = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.stake_vault.to_account_info(),
        to: ctx.accounts.user_stake_ata.to_account_info(),
        authority: ctx.accounts.pool.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer,
    );

    token::transfer(cpi_ctx, amount)?;

    // Now update the pool and user
    let pool = &mut ctx.accounts.pool;
    user.amount_staked -= amount as u128;
    pool.total_staked -= amount as u128;
    user.last_update = Clock::get()?.unix_timestamp;

    Ok(())
}