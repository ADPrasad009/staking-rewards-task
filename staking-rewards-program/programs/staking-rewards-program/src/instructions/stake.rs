use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::state::{Pool, UserStake};
use crate::errors::StakingError;
use crate::utils::update_rewards;

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub pool: Account<'info, Pool>,

    #[account(
        init_if_needed,
        payer = user,
        space = UserStake::LEN,
        seeds = [b"user_stake", pool.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(
        mut,
        token::mint = pool.stake_mint,
        token::authority = user
    )]
    pub user_stake_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub stake_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
    require!(amount > 0, StakingError::ZeroAmount);

    let user = &mut ctx.accounts.user_stake;
    let pool = &mut ctx.accounts.pool;

    // Initialize user stake if needed
    if user.owner == Pubkey::default() {
        user.owner = ctx.accounts.user.key();
        user.amount_staked = 0;
        user.pending_rewards = 0;
        user.last_update = Clock::get()?.unix_timestamp;
        user.bump = ctx.bumps.user_stake;
    }

    update_rewards(user, pool)?;

    // Transfer stake → vault
    let cpi_accounts = Transfer {
        from: ctx.accounts.user_stake_ata.to_account_info(),
        to: ctx.accounts.stake_vault.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
    );

    token::transfer(cpi_ctx, amount)?;

    user.amount_staked += amount as u128;
    pool.total_staked += amount as u128;
    user.last_update = Clock::get()?.unix_timestamp;

    Ok(())
}