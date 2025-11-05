use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
    PoolNotActive,
    PoolPaused,
    MathOverflow,
    InsufficientReserves,
    SlippageExceeded,
    MaxPoolsReached,
    InsufficientTokenBalance,
    InvalidAmount,
    InsufficientFunds,
}
