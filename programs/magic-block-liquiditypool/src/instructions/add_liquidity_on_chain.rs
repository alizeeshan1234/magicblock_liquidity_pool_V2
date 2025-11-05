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

#[derive(Accounts)]
pub struct DepositLiquidityOnChain<'info> {
    pub provider: AccountInfo<'info>,

    pub mint_a: AccountInfo<'info>,

    pub mint_b: AccountInfo<'info>,

    // #[account(
    //     seeds = [b"transfer_authority"],
    //     bump,
    // )]
    pub transfer_authority: AccountInfo<'info>,

    // #[account(
    //     mut,
    //     mint::authority = transfer_authority,
    //     mint::freeze_authority = transfer_authority,
    //     mint::decimals = 6,
    //     seeds = [b"lp_token_mint"],
    //     bump
    // )]
    pub lp_mint: AccountInfo<'info>,

    #[account(
        mut,
        // seeds = [b"token_account_a", mint_a.key().as_ref()],
        // bump,
        // token::mint = mint_a,
        // token::authority = transfer_authority
    )]
    pub token_vault_a: AccountInfo<'info>,

    #[account(
        mut,
        // seeds = [b"token_account_b", mint_b.key().as_ref()],
        // bump,
        // token::mint = mint_b,
        // token::authority = transfer_authority
    )]
    pub token_vault_b: AccountInfo<'info>,

    #[account(
        mut,
        // associated_token::mint = mint_a,
        // associated_token::authority = provider
    )]
    pub provider_token_a_ata: AccountInfo<'info>,

    #[account(
        mut,
        // associated_token::mint = mint_b,
        // associated_token::authority = provider
    )]
    pub provider_token_b_ata: AccountInfo<'info>,

    #[account(
        mut,
        // associated_token::mint = lp_mint,
        // associated_token::authority = provider
    )]
    pub provider_token_lp_ata: AccountInfo<'info>,

    pub pool: UncheckedAccount<'info>,

    pub liquidity_provider: UncheckedAccount<'info>,

    pub token_program: AccountInfo<'info>,

    pub system_program: AccountInfo<'info>,

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
