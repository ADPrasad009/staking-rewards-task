use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::state::Pool;
use crate::errors::StakingError;

#[derive(Accounts)]
pub struct DepositRewards<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut)]
    pub admin_reward_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub reward_vault: Account<'info, TokenAccount>,

    pub pool: Account<'info, Pool>,

    pub token_program: Program<'info, Token>,
}

pub fn deposit_rewards(ctx: Context<DepositRewards>, amount: u64) -> Result<()> {
    require_keys_eq!(
        ctx.accounts.pool.admin,
        ctx.accounts.admin.key(),
        StakingError::Unauthorized
    );

    let cpi_accounts = Transfer {
        from: ctx.accounts.admin_reward_ata.to_account_info(),
        to: ctx.accounts.reward_vault.to_account_info(),
        authority: ctx.accounts.admin.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
    );

    token::transfer(cpi_ctx, amount)?;

    Ok(())
}