use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TransferChecked, TokenInterface, TokenAccount, Mint};
use crate::state::{Pool, UserStakeAccount};
use crate::errors::StakingError;
use crate::utils;

pub mod initialize_pool {
    use super::*;

    pub fn process(ctx: Context<crate::instructions::initialize_pool::InitializePool>, reward_rate_per_second: u128) -> Result<()> {
        // admin is signer (enforced by accounts)
        let pool = &mut ctx.accounts.pool;

        // set pool fields
        pool.admin = ctx.accounts.admin.key();
        pool.stake_mint = ctx.accounts.stake_mint.key();
        pool.reward_mint = ctx.accounts.reward_mint.key();
        pool.reward_rate_per_second = reward_rate_per_second;
        pool.total_staked = 0u128;
        pool.bump = ctx.bumps.pool;

        // create token accounts for vaults via CPI
        // token account size 165
        let token_account_len: u64 = 165;
        let lamports = ctx.accounts.rent.minimum_balance(token_account_len as usize);
        let pool_key = pool.key();

        // compute authority PDAs
        let stake_vault_bump = ctx.bumps.pool_stake_vault;
        let reward_vault_bump = ctx.bumps.reward_vault;

        let stake_auth_seeds: &[&[u8]] = &[b"pool_vault", pool_key.as_ref(), &[stake_vault_bump]];
        let reward_auth_seeds: &[&[u8]] = &[b"reward_vault", pool_key.as_ref(), &[reward_vault_bump]];
        let (stake_authority, _) = Pubkey::find_program_address(stake_auth_seeds, ctx.program_id);
        let (reward_authority, _) = Pubkey::find_program_address(reward_auth_seeds, ctx.program_id);

        // Create stake vault account
        {
            let create_ix = anchor_lang::solana_program::system_instruction::create_account(
                &ctx.accounts.admin.key(),
                &ctx.accounts.pool_stake_vault.key(),
                lamports,
                token_account_len,
                &ctx.accounts.token_program.key(),
            );
            anchor_lang::solana_program::program::invoke(
                &create_ix,
                &[
                    ctx.accounts.admin.to_account_info(),
                    ctx.accounts.pool_stake_vault.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;

            let init_ix = anchor_spl::token_2022::spl_token_2022::instruction::initialize_account3(
                ctx.accounts.token_program.key,
                ctx.accounts.pool_stake_vault.key,
                &ctx.accounts.stake_mint.key(),
                &stake_authority,
            )?;
            anchor_lang::solana_program::program::invoke(
                &init_ix,
                &[
                    ctx.accounts.pool_stake_vault.to_account_info(),
                    ctx.accounts.stake_mint.to_account_info(),
                    ctx.accounts.token_program.to_account_info(),
                ],
            )?;
        }

        // Create reward vault account
        {
            let create_ix = anchor_lang::solana_program::system_instruction::create_account(
                &ctx.accounts.admin.key(),
                &ctx.accounts.reward_vault.key(),
                lamports,
                token_account_len,
                &ctx.accounts.token_program.key(),
            );
            anchor_lang::solana_program::program::invoke(
                &create_ix,
                &[
                    ctx.accounts.admin.to_account_info(),
                    ctx.accounts.reward_vault.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;

            let init_ix = anchor_spl::token_2022::spl_token_2022::instruction::initialize_account3(
                ctx.accounts.token_program.key,
                ctx.accounts.reward_vault.key,
                &ctx.accounts.reward_mint.key(),
                &reward_authority,
            )?;
            anchor_lang::solana_program::program::invoke(
                &init_ix,
                &[
                    ctx.accounts.reward_vault.to_account_info(),
                    ctx.accounts.reward_mint.to_account_info(),
                    ctx.accounts.token_program.to_account_info(),
                ],
            )?;
        }

        Ok(())
    }
}

pub mod stake {
    use super::*;

    pub fn process(ctx: Context<crate::instructions::stake::Stake>, amount: u64) -> Result<()> {
        if amount == 0 {
            return Err(StakingError::ZeroAmount.into());
        }

        let pool = &mut ctx.accounts.pool;
        let user_stake = &mut ctx.accounts.user_stake_account;

        // validate mint
        require_keys_eq!(ctx.accounts.stake_mint.key(), pool.stake_mint, StakingError::InvalidMint);

        // check user balance
        require!(ctx.accounts.user_stake_ata.amount >= amount, StakingError::InsufficientFunds);

        // initialize user stake if needed
        if user_stake.owner == Pubkey::default() {
            user_stake.owner = ctx.accounts.user.key();
            user_stake.bump = ctx.bumps.user_stake_account;
            user_stake.amount_staked = 0u128;
            user_stake.pending_rewards = 0u128;
            user_stake.last_update_timestamp = utils::now_ts()?;
        }

        // accrue rewards
        let now = utils::now_ts()?;
        utils::compute_reward_delta(user_stake, pool, now)?;

        // transfer tokens user -> pool via CPI
        let decimals = ctx.accounts.stake_mint.decimals;
        let cpi_accounts = TransferChecked {
            from: ctx.accounts.user_stake_ata.to_account_info(),
            to: ctx.accounts.pool_stake_vault.to_account_info(),
            mint: ctx.accounts.stake_mint.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        anchor_spl::token_interface::transfer_checked(cpi_ctx, amount, decimals)?;

        // update accounting (checked)
        let amt_u128 = amount as u128;
        user_stake.amount_staked = user_stake.amount_staked.checked_add(amt_u128).ok_or(StakingError::Overflow)?;
        pool.total_staked = pool.total_staked.checked_add(amt_u128).ok_or(StakingError::Overflow)?;

        Ok(())
    }
}

pub mod unstake {
    use super::*;

    pub fn process(ctx: Context<crate::instructions::unstake::Unstake>, amount: u64) -> Result<()> {
        if amount == 0 {
            return Err(StakingError::ZeroAmount.into());
        }

        let pool = &mut ctx.accounts.pool;
        let user_stake = &mut ctx.accounts.user_stake_account;

        // validate mint
        require_keys_eq!(ctx.accounts.stake_mint.key(), pool.stake_mint, StakingError::InvalidMint);

        // accrue rewards
        let now = utils::now_ts()?;
        utils::compute_reward_delta(user_stake, pool, now)?;

        // prepare PDA signer seeds
        let pool_key = pool.key();
        let bump = ctx.bumps.pool_stake_vault;
        let seeds: &[&[u8]] = &[b"pool_vault", pool_key.as_ref(), &[bump]];
        let signer = &[seeds];

        let decimals = ctx.accounts.stake_mint.decimals;
        let cpi_accounts = TransferChecked {
            from: ctx.accounts.pool_stake_vault.to_account_info(),
            to: ctx.accounts.user_stake_ata.to_account_info(),
            mint: ctx.accounts.stake_mint.to_account_info(),
            authority: ctx.accounts.pool_stake_vault_authority.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), cpi_accounts, signer);
        anchor_spl::token_interface::transfer_checked(cpi_ctx, amount, decimals)?;

        // update accounting
        let amt_u128 = amount as u128;
        user_stake.amount_staked = user_stake.amount_staked.checked_sub(amt_u128).ok_or(StakingError::InsufficientFunds)?;
        pool.total_staked = pool.total_staked.checked_sub(amt_u128).ok_or(StakingError::MathError)?;

        if user_stake.amount_staked == 0 {
            user_stake.pending_rewards = 0u128;
            user_stake.last_update_timestamp = now;
        }

        Ok(())
    }
}

pub mod claim_rewards {
    use super::*;

    pub fn process(ctx: Context<crate::instructions::claim_rewards::ClaimRewards>) -> Result<()> {
        let pool = &ctx.accounts.pool;
        let user_stake = &mut ctx.accounts.user_stake_account;

        let now = utils::now_ts()?;
        utils::compute_reward_delta(user_stake, pool, now)?;

        let to_claim = user_stake.pending_rewards;
        require!(to_claim > 0, StakingError::NoRewardsAccrued);

        // check reward vault balance fits in u64
        let vault_amount_u64 = ctx.accounts.reward_vault.amount;
        require!(vault_amount_u64 >= (to_claim as u64), StakingError::InsufficientFunds);

        // PDA signer
        let pool_key = pool.key();
        let bump = ctx.bumps.reward_vault;
        let seeds: &[&[u8]] = &[b"reward_vault", pool_key.as_ref(), &[bump]];
        let signer = &[seeds];

        let decimals = ctx.accounts.reward_mint.decimals;

        let cpi_accounts = TransferChecked {
            from: ctx.accounts.reward_vault.to_account_info(),
            to: ctx.accounts.user_reward_ata.to_account_info(),
            mint: ctx.accounts.reward_mint.to_account_info(),
            authority: ctx.accounts.reward_vault_authority.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), cpi_accounts, signer);
        anchor_spl::token_interface::transfer_checked(cpi_ctx, to_claim as u64, decimals)?;

        user_stake.pending_rewards = 0u128;
        Ok(())
    }
}

pub mod deposit_rewards {
    use super::*;

    pub fn process(ctx: Context<crate::instructions::deposit_rewards::DepositRewards>, amount: u64) -> Result<()> {
        if amount == 0 {
            return Err(StakingError::ZeroAmount.into());
        }

        // only admin allowed by accounts constraint
        let pool = &ctx.accounts.pool;
        require_keys_eq!(pool.admin, ctx.accounts.admin.key(), StakingError::Unauthorized);

        // ensure mint matches pool
        require_keys_eq!(ctx.accounts.reward_mint.key(), pool.reward_mint, StakingError::InvalidMint);

        // transfer from admin -> reward_vault (no signer seeds, admin signs)
        let decimals = ctx.accounts.reward_mint.decimals;
        let cpi_accounts = TransferChecked {
            from: ctx.accounts.admin_reward_ata.to_account_info(),
            to: ctx.accounts.reward_vault.to_account_info(),
            mint: ctx.accounts.reward_mint.to_account_info(),
            authority: ctx.accounts.admin.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        anchor_spl::token_interface::transfer_checked(cpi_ctx, amount, decimals)?;

        Ok(())
    }
}
