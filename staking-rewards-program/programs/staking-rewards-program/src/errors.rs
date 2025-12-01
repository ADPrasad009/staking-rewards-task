use anchor_lang::prelude::*;

#[error_code]
pub enum StakingError {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Not enough tokens")]
    InsufficientFunds,
    #[msg("Zero amount")]
    ZeroAmount,
    #[msg("No rewards available")]
    NoRewardsAccrued,
    #[msg("Overflow")]
    Overflow,
}