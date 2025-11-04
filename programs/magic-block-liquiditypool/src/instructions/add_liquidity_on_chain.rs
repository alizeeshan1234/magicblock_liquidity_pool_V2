use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token};

use ephemeral_rollups_sdk::anchor::{commit, delegate, ephemeral};
use ephemeral_rollups_sdk::cpi::DelegateConfig;
use ephemeral_rollups_sdk::ephem::commit_and_undelegate_accounts;
use ephemeral_rollups_sdk::ephem::{MagicInstructionBuilder, MagicAction, CallHandler, CommitType};
use ephemeral_rollups_sdk::{ActionArgs, ShortAccountMeta};
use anchor_lang::Discriminator;

use crate::liquidity_provider::LiquidityProvider;
use crate::pool::Pool;

#[commit]  // ← ADD THIS
#[derive(Accounts)]
pub struct DepositLiquidityOnChain<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

    pub mint_a: Account<'info, Mint>,

    pub mint_b: Account<'info, Mint>,

    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,

    #[account(mut)]
    pub token_vault_a: UncheckedAccount<'info>,

    #[account(mut)]
    pub token_vault_b: UncheckedAccount<'info>,

    #[account(mut)]
    pub provider_token_a_ata: UncheckedAccount<'info>,

    #[account(mut)]
    pub provider_token_b_ata: UncheckedAccount<'info>,

    #[account(mut)]
    pub provider_token_lp_ata: UncheckedAccount<'info>,

    pub pool: UncheckedAccount<'info>,

    pub liquidity_provider: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,

    /// CHECK: injected - will be moved to end
    pub escrow: UncheckedAccount<'info>,
    /// CHECK: injected - will be moved to end
    pub escrow_auth: UncheckedAccount<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CommitAndAddLiquidityParams {
    pub user: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub min_lp_tokens: u64,
}

pub fn deposit_liquidity(ctx: Context<DepositLiquidityOnChain>, params: CommitAndAddLiquidityParams) -> Result<()> {
    msg!("Hello World!");
    Ok(())
}

