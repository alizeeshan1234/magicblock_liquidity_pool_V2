use anchor_lang::prelude::*;

use crate::state::liquidity_provider::{LiquidityPoolInfo, LiquidityProvider};
use crate::state::pool::Pool;
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct RemoveLiquidityER<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

    #[account(
        mut,
        seeds = [b"liquidity_provider_account_info", provider.key().as_ref()],
        bump = liquidity_provider.bump
    )]
    pub liquidity_provider: Account<'info, LiquidityProvider>,

    #[account(
        mut,
        seeds = [b"pool", pool.name.as_bytes()],
        bump = pool.bump
    )]
    pub pool: Account<'info, Pool>,

    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RemoveLiquidityErParams {
    pub user: Pubkey,
    pub lp_tokens: u64,
    pub min_amount_a: u64,
    pub min_amount_b: u64,
}

pub fn remove_liquidity_er(ctx: Context<RemoveLiquidityER>, params: RemoveLiquidityErParams) -> Result<()> {

    let pool = &mut ctx.accounts.pool;
    let liquidity_provider = &mut ctx.accounts.liquidity_provider;

    require!(pool.status.is_active, ErrorCode::PoolNotActive);
    require!(!pool.status.is_paused, ErrorCode::PoolPaused);

    msg!("Removing liquidity in ER (state only)...");

    let reserve_a = pool.reserve_a;
    let reserve_b = pool.reserve_b;
    let total_lp_supply = pool.total_lp_supply;

    let provider_pool_info = liquidity_provider
        .liquidity_pools_info
        .iter()
        .find(|info| info.pool == pool.key())
        .ok_or(ErrorCode::ProviderNotFound)?;

    require!(provider_pool_info.lp_tokens >= params.lp_tokens, ErrorCode::InsufficientLpTokens);

    let amount_a_to_withdraw = (params.lp_tokens as u128)
        .checked_mul(reserve_a as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(total_lp_supply as u128)
        .ok_or(ErrorCode::MathOverflow)? as u64;

    let amount_b_to_withdraw = (params.lp_tokens as u128)
        .checked_mul(reserve_b as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(total_lp_supply as u128)
        .ok_or(ErrorCode::MathOverflow)? as u64;

    require!(
        amount_a_to_withdraw >= params.min_amount_a,
        ErrorCode::SlippageExceeded
    );
    require!(
        amount_b_to_withdraw >= params.min_amount_b,
        ErrorCode::SlippageExceeded
    );

    pool.reserve_a = pool.reserve_a
        .checked_sub(amount_a_to_withdraw)
        .ok_or(ErrorCode::InsufficientReserves)?;

    pool.reserve_b = pool.reserve_b
        .checked_sub(amount_b_to_withdraw)
        .ok_or(ErrorCode::InsufficientReserves)?;

    pool.total_lp_supply = pool.total_lp_supply
        .checked_sub(params.lp_tokens)
        .ok_or(ErrorCode::MathOverflow)?;

    update_liquidity_provider_remove(
        liquidity_provider,
        pool.key(),
        amount_a_to_withdraw + amount_b_to_withdraw,
        params.lp_tokens
    )?;

    pool.updated_at = Clock::get()?.unix_timestamp;

    msg!("Liquidity removed in ER: {} LP tokens", params.lp_tokens);
    msg!(
        "Withdrawn amounts: A={}, B={}",
        amount_a_to_withdraw,
        amount_b_to_withdraw
    );
    msg!("New reserves: A={}, B={}", pool.reserve_a, pool.reserve_b);

    Ok(())
}

fn update_liquidity_provider_remove(
    provider: &mut LiquidityProvider,
    pool_key: Pubkey,
    liquidity_amount: u64,
    lp_tokens: u64,
) -> Result<()> {
    let pool_index = provider
        .liquidity_pools_info
        .iter()
        .position(|info| info.pool == pool_key)
        .ok_or(ErrorCode::ProviderNotFound)?;

    let pool_info = &mut provider.liquidity_pools_info[pool_index];

    require!(
        pool_info.liquidity_amount >= liquidity_amount,
        ErrorCode::InsufficientLiquidity
    );
    require!(
        pool_info.lp_tokens >= lp_tokens,
        ErrorCode::InsufficientLpTokens
    );

    pool_info.liquidity_amount = pool_info
        .liquidity_amount
        .checked_sub(liquidity_amount)
        .ok_or(ErrorCode::MathOverflow)?;

    pool_info.lp_tokens = pool_info
        .lp_tokens
        .checked_sub(lp_tokens)
        .ok_or(ErrorCode::MathOverflow)?;

    if pool_info.lp_tokens == 0 {
        provider.liquidity_pools_info[pool_index] = LiquidityPoolInfo {
            pool: Pubkey::default(),
            token_mint: Pubkey::default(),
            liquidity_amount: 0,
            lp_tokens: 0,
        };
    }

    provider.total_liquidity_provided = provider
        .total_liquidity_provided
        .checked_sub(liquidity_amount)
        .ok_or(ErrorCode::MathOverflow)?;

    provider.total_lp_tokens = provider
        .total_lp_tokens
        .checked_sub(lp_tokens)
        .ok_or(ErrorCode::MathOverflow)?;

    provider.latest_liquidity_provided_on = Clock::get()?.unix_timestamp;

    Ok(())
}