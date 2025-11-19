//Pure state change in ER only

use anchor_lang::prelude::*;

use crate::state::liquidity_provider::{LiquidityPoolInfo, LiquidityProvider};
use crate::state::pool::Pool;
use crate::error::ErrorCode;

use crate::add_liquidity_on_chain::DepositRecept;

#[derive(Accounts)]
pub struct AddLiquidityER<'info> {
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

    #[account(
        mut,
        seeds = [b"deposit_recept", provider.key().as_ref()],
        bump,
    )]
    pub deposit_receipt: Account<'info, DepositRecept>,

    pub system_program: Program<'info, System>,
}

pub fn add_liquidity_er(ctx: Context<AddLiquidityER>) -> Result<()> {

    let pool = &mut ctx.accounts.pool;
    let liquidity_provider = &mut ctx.accounts.liquidity_provider;

    require!(pool.status.is_active, ErrorCode::PoolNotActive);
    require!(!pool.status.is_paused, ErrorCode::PoolPaused);

    msg!("Adding liquidity in ER (state only)...");

    let reserve_a = pool.reserve_a;
    let reserve_b = pool.reserve_b;
    let total_lp_supply = pool.total_lp_supply;

    let lp_tokens_to_mint = if total_lp_supply == 0 {
        let product = (ctx.accounts.deposit_receipt.amount_a as u128)
            .checked_mul(ctx.accounts.deposit_receipt.amount_b as u128)
            .ok_or(ErrorCode::MathOverflow)?;
        (product as f64).sqrt() as u64
    } else {
        require!(reserve_a > 0 && reserve_b > 0, ErrorCode::InsufficientReserves);
        
        let share_a = (ctx.accounts.deposit_receipt.amount_a as u128)
            .checked_mul(total_lp_supply as u128)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(reserve_a as u128)
            .ok_or(ErrorCode::MathOverflow)? as u64;
            
        let share_b = (ctx.accounts.deposit_receipt.amount_b as u128)
            .checked_mul(total_lp_supply as u128)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(reserve_b as u128)
            .ok_or(ErrorCode::MathOverflow)? as u64;
        
        std::cmp::min(share_a, share_b)
    };

    require!(lp_tokens_to_mint >= ctx.accounts.deposit_receipt.lp_tokens_minted, ErrorCode::SlippageExceeded);

    pool.reserve_a = pool.reserve_a
        .checked_add(ctx.accounts.deposit_receipt.amount_a)
        .ok_or(ErrorCode::MathOverflow)?;
    
    pool.reserve_b = pool.reserve_b
        .checked_add(ctx.accounts.deposit_receipt.amount_b)
        .ok_or(ErrorCode::MathOverflow)?;
    
    pool.total_lp_supply = pool.total_lp_supply
        .checked_add(lp_tokens_to_mint)
        .ok_or(ErrorCode::MathOverflow)?;

    update_liquidity_provider_add(
        liquidity_provider,
        pool.key(),
        pool.lp_mint,
        ctx.accounts.deposit_receipt.amount_a + ctx.accounts.deposit_receipt.amount_b,
        lp_tokens_to_mint,
    )?;

    pool.updated_at = Clock::get()?.unix_timestamp;

    msg!("Liquidity added in ER: {} LP tokens", lp_tokens_to_mint);
    msg!("New reserves: A={}, B={}", pool.reserve_a, pool.reserve_b);

    Ok(())
}


fn update_liquidity_provider_add(
    provider: &mut LiquidityProvider,
    pool_key: Pubkey,
    lp_mint: Pubkey,
    liquidity_amount: u64,
    lp_tokens: u64,
) -> Result<()> {
    let mut found_index: Option<usize> = None;
    let mut empty_index: Option<usize> = None;

    for (i, pool_info) in provider.liquidity_pools_info.iter().enumerate() {
        if pool_info.pool == pool_key {
            found_index = Some(i);
            break;
        }
        if empty_index.is_none() && pool_info.pool == Pubkey::default() {
            empty_index = Some(i);
        }
    }

    let index = found_index.or(empty_index)
        .ok_or(ErrorCode::MaxPoolsReached)?;

    provider.liquidity_pools_info[index] = LiquidityPoolInfo {
        pool: pool_key,
        token_mint: lp_mint,
        liquidity_amount: provider.liquidity_pools_info[index].liquidity_amount
            .checked_add(liquidity_amount)
            .ok_or(ErrorCode::MathOverflow)?,
        lp_tokens: provider.liquidity_pools_info[index].lp_tokens
            .checked_add(lp_tokens)
            .ok_or(ErrorCode::MathOverflow)?,
    };

    provider.total_liquidity_provided = provider.total_liquidity_provided
        .checked_add(liquidity_amount)
        .ok_or(ErrorCode::MathOverflow)?;
        
    provider.total_lp_tokens = provider.total_lp_tokens
        .checked_add(lp_tokens)
        .ok_or(ErrorCode::MathOverflow)?;

    provider.latest_liquidity_provided_on = Clock::get()?.unix_timestamp;

    Ok(())
}