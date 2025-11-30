use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenAccount, TokenInterface, Mint};
use crate::state::UserStakeAccount;

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// Pool PDA
    #[account(
        seeds = [b"pool", pool.stake_mint.as_ref(), pool.reward_mint.as_ref()],
        bump = pool.bump,
        has_one = stake_mint
    )]
    pub pool: Account<'info, crate::state::Pool>,

    pub stake_mint: InterfaceAccount<'info, Mint>,

    /// user's ATA for stake mint
    #[account(mut)]
    pub user_stake_ata: InterfaceAccount<'info, TokenAccount>,

    /// pool stake vault
    #[account(mut, seeds = [b"pool_vault", pool.key().as_ref()], bump)]
    pub pool_stake_vault: InterfaceAccount<'info, TokenAccount>,

    /// PDA authority for pool vault
    pub pool_stake_vault_authority: UncheckedAccount<'info>,

    /// user stake PDA
    #[account(
        init_if_needed,
        payer = user,
        space = UserStakeAccount::LEN,
        seeds = [b"user_stake", pool.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub user_stake_account: Account<'info, UserStakeAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
