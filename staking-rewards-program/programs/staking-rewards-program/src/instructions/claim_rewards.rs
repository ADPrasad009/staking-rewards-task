use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenAccount, TokenInterface, Mint};
use crate::state::UserStakeAccount;
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
