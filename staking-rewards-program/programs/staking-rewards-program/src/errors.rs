use anchor_lang::prelude::*;

#[error_code]
pub enum StakingError {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Invalid mint")]
    InvalidMint,
    #[msg("Overflow")]
    Overflow,
    #[msg("Zero amount not allowed")]
    ZeroAmount,
    #[msg("No rewards accrued")]
    NoRewardsAccrued,
    #[msg("Math error")]
    MathError,
}
