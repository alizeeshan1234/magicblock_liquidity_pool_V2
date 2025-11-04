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

use crate::state::pool::Pool;

#[delegate]
#[derive(Accounts)]
pub struct DelegatePool<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        del,
        seeds = [b"pool", pool.name.as_bytes()],
        bump
    )]
    pub pool: Account<'info, Pool>,
}

pub fn delegate_pool(ctx: Context<DelegatePool>, commit_frequency: u32, validator_key: Pubkey) -> Result<()> {

    let delegate_config = DelegateConfig {
        commit_frequency_ms: commit_frequency,
        validator: Some(validator_key),
    };

    let pool = &ctx.accounts.pool;
    let pool_name = pool.name.clone();
    let seeds = &[b"pool", pool_name.as_bytes()];

    ctx.accounts.delegate_pool(&ctx.accounts.payer, seeds, delegate_config)?;

    msg!("Pool account delegated successfully!");

    Ok(())
}