use anchor_lang::prelude::*;
use anchor_lang::Discriminator;
use anchor_lang::solana_program::vote::instruction;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked},
    *,
};

use ephemeral_rollups_sdk::anchor::delegate;
use ephemeral_rollups_sdk::cpi::DelegateConfig;

use crate::state::liquidity_provider::{LiquidityPoolInfo, LiquidityProvider};

#[delegate]
#[derive(Accounts)]
pub struct DelegateLiquidityProvider<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

    #[account(
        mut,
        del,
        seeds = [b"liquidity_provider_account_info", provider.key().as_ref()],
        bump
    )]
    pub liquidity_provider: Account<'info, LiquidityProvider>,
}

pub fn delegate_liquidity_provider(ctx: Context<DelegateLiquidityProvider>, commit_frequency: u32, validator_key: Pubkey) -> Result<()> {

    let delegate_config = DelegateConfig {
        commit_frequency_ms: commit_frequency,
        validator: Some(validator_key),
    };

    let provider_key = ctx.accounts.provider.key();
    let seeds = &[b"liquidity_provider_account_info", provider_key.as_ref()];

    ctx.accounts.delegate_liquidity_provider(&ctx.accounts.provider, seeds, delegate_config)?;

    msg!("Liquidity Provider account delegated successfully!");

    Ok(())
}