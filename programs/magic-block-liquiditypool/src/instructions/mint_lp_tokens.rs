use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_lang::Discriminator;

use ephemeral_rollups_sdk::anchor::commit;
use ephemeral_rollups_sdk::ephem::{MagicInstructionBuilder, MagicAction, CallHandler, CommitType};
use ephemeral_rollups_sdk::{ActionArgs, ShortAccountMeta};

use crate::state::pool::Pool;
use crate::state::liquidity_provider::{LiquidityProvider, LiquidityPoolInfo};
use crate::add_liquidity_on_chain::DepositRecept;
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct MintLpTokens<'info> {
    #[account(
        // mut
    )]
    pub provider: AccountInfo<'info>,

    /// CHECK: Transfer authority PDA
    #[account(
        seeds = [b"transfer_authority"],
        bump,
    )]
    pub transfer_authority: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"lp_token_mint"],
        bump
    )]
    pub lp_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = lp_mint,
        associated_token::authority = provider
    )]
    pub provider_lp_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,

    /// CHECK: the correct pda - this will be moved to the end in the future, meaning you can omit this unless needed
    pub escrow: UncheckedAccount<'info>,
    /// CHECK: the correct pda - this will be moved to the end in the future, meaning you can omit this unless needed
    pub escrow_auth: UncheckedAccount<'info>,
}

pub fn mint_lp_tokens(ctx: Context<MintLpTokens>, mint_amount: u64) -> Result<()> {
    msg!("Minting {} LP tokens on-chain to provider", mint_amount);

    let authority_seeds = &[
        b"transfer_authority".as_ref(),
        &[ctx.bumps.transfer_authority],
    ];
    let signer_seeds = &[&authority_seeds[..]];

    let cpi_accounts = anchor_spl::token::MintTo {
        mint: ctx.accounts.lp_mint.to_account_info(),
        to: ctx.accounts.provider_lp_ata.to_account_info(),
        authority: ctx.accounts.transfer_authority.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

    anchor_spl::token::mint_to(cpi_ctx, mint_amount)?;

    msg!("Successfully minted {} LP tokens on-chain", mint_amount);

    Ok(())
}