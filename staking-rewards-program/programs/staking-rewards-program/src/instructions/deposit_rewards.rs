use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenAccount, TokenInterface, Mint};
use crate::errors::StakingError;

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct DepositRewards<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut, seeds = [b"pool", pool.stake_mint.as_ref(), pool.reward_mint.as_ref()], bump = pool.bump, constraint = pool.admin == admin.key() @ StakingError::Unauthorized)]
    pub pool: Account<'info, crate::state::Pool>,

    /// admin ATA for reward mint
    #[account(mut)]
    pub admin_reward_ata: InterfaceAccount<'info, TokenAccount>,

    /// reward vault PDA
    #[account(mut, seeds = [b"reward_vault", pool.key().as_ref()], bump)]
    pub reward_vault: InterfaceAccount<'info, TokenAccount>,

    pub reward_vault_authority: UncheckedAccount<'info>,

    pub reward_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
}
