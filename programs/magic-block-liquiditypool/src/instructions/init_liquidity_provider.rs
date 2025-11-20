use anchor_lang::prelude::*;

use crate::{liquidity_provider::LiquidityPoolInfo, state::liquidity_provider::LiquidityProvider};

#[derive(Accounts)]
pub struct InitializeLiquidityProvider<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

    #[account(
        init,
        payer = provider,
        space = 8 + LiquidityProvider::INIT_SPACE,
        seeds = [b"liquidity_provider_account_info", provider.key().as_ref()],
        bump
    )]
    pub liquidity_provider_account_info: Account<'info, LiquidityProvider>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_liquidity_provider(ctx: Context<InitializeLiquidityProvider>) -> Result<()> {

    msg!("Initializing Liquidity Provider...");

    let liquidity_provider_account = &mut ctx.accounts.liquidity_provider_account_info;

    liquidity_provider_account.provider = ctx.accounts.provider.key();
    liquidity_provider_account.total_liquidity_provided = 0;
    liquidity_provider_account.total_lp_tokens = 0;
    liquidity_provider_account.liquidity_pools_info = [LiquidityPoolInfo::default(); 10];
    liquidity_provider_account.latest_liquidity_provided_on = 0;
    liquidity_provider_account.bump = ctx.bumps.liquidity_provider_account_info;

    msg!("Initialized Liquidity Provider Succesfully!");
    msg!("Liquidity Provider Info: {:?}", liquidity_provider_account);

    Ok(())
}