use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenAccount, TokenInterface, Mint};
use crate::state::UserStake::;
use crate::errors::StakingError;

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// pool PDA
    #[account(seeds = [b"pool", pool.stake_mint.as_ref(), pool.reward_mint.as_ref()], bump = pool.bump)]
    pub pool: Account<'info, crate::state::Pool>,

    /// user stake PDA
    #[account(mut,
        seeds = [b"user_stake", pool.key().as_ref(), user.key().as_ref()],
        bump = user_stake_account.bump,
        constraint = user_stake_account.owner == user.key() @ StakingError::Unauthorized
    )]
    pub user_stake_account: Account<'info, UserStakeAccount>,

    /// reward vault token account PDA
    #[account(mut, seeds = [b"reward_vault", pool.key().as_ref()], bump)]
    pub reward_vault: InterfaceAccount<'info, TokenAccount>,

    /// PDA authority
    pub reward_vault_authority: UncheckedAccount<'info>,

    /// user reward ATA
    #[account(mut)]
    pub user_reward_ata: InterfaceAccount<'info, TokenAccount>,

    pub reward_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
}
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::state::{Pool, UserStake};
use crate::errors::StakingError;
use crate::utils::update_rewards;

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
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
        token::mint = pool.reward_mint,
        token::authority = user
    )]
    pub user_reward_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"reward_vault", pool.key().as_ref()],
        bump
    )]
    pub reward_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
    let user = &mut ctx.accounts.user_stake;

    // Get pool data BEFORE mutable operations
    let pool_bump = ctx.accounts.pool.bump;
    let stake_mint = ctx.accounts.pool.stake_mint;
    let reward_mint = ctx.accounts.pool.reward_mint;

    update_rewards(user, &ctx.accounts.pool)?;

    let amount = user.pending_rewards as u64;
    require!(amount > 0, StakingError::NoRewardsAccrued);

    user.pending_rewards = 0;

    let seeds = &[
        b"pool",
        stake_mint.as_ref(),
        reward_mint.as_ref(),
        &[pool_bump],
    ];

    let signer = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.reward_vault.to_account_info(),
        to: ctx.accounts.user_reward_ata.to_account_info(),
        authority: ctx.accounts.pool.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer,
    );

    token::transfer(cpi_ctx, amount)?;

    user.last_update = Clock::get()?.unix_timestamp;

    Ok(())
}