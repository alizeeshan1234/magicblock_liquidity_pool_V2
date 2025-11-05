use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer, spl_token};

use ephemeral_rollups_sdk::anchor::{commit, delegate, ephemeral};
use ephemeral_rollups_sdk::cpi::DelegateConfig;
use ephemeral_rollups_sdk::ephem::commit_and_undelegate_accounts;
use ephemeral_rollups_sdk::ephem::{MagicInstructionBuilder, MagicAction, CallHandler, CommitType};
use ephemeral_rollups_sdk::{ActionArgs, ShortAccountMeta};
use anchor_lang::Discriminator;

use crate::liquidity_provider::LiquidityProvider;
use crate::pool::Pool;

use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct DepositLiquidityOnchain<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,

    #[account(
        seeds = [b"transfer_authority"],
        bump,
    )]
    pub transfer_authority: AccountInfo<'info>,

    #[account(
        mut,
        mint::authority = transfer_authority,
        mint::freeze_authority = transfer_authority,
        mint::decimals = 6,
        seeds = [b"lp_token_mint"],
        bump
    )]
    pub lp_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"token_account_a", mint_a.key().as_ref()],
        bump,
        token::mint = mint_a,
        token::authority = transfer_authority
    )]
    pub token_vault_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"token_account_b", mint_b.key().as_ref()],
        bump,
        token::mint = mint_b,
        token::authority = transfer_authority
    )]
    pub token_vault_b: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = provider
    )]
    pub provider_token_a_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = provider
    )]
    pub provider_token_b_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = lp_mint,
        associated_token::authority = provider
    )]
    pub provider_token_lp_ata: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = provider,
        space = 8 + DepositRecept::INIT_SPACE,
        seeds = [b"deposit_recept", provider.key().as_ref()],
        bump
    )]
    pub deposit_recept: Account<'info, DepositRecept>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}


pub fn deposit_liquidity_on_chain(ctx: Context<DepositLiquidityOnchain>, amount_a: u64,
    amount_b: u64,
    min_lp_tokens: u64
) -> Result<()> {

    require!(amount_a > 0, ErrorCode::InvalidAmount);
    require!(amount_b > 0, ErrorCode::InvalidAmount);

    require!(
        ctx.accounts.provider_token_a_ata.amount >= amount_a,
        ErrorCode::InsufficientBalance
    );

    require!(
        ctx.accounts.provider_token_b_ata.amount >= amount_b,
        ErrorCode::InsufficientBalance
    );

    let cpi_accounts_token_a = Transfer {
        from: ctx.accounts.provider_token_a_ata.to_account_info(),
        to: ctx.accounts.token_vault_a.to_account_info(),
        authority: ctx.accounts.provider.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();

    let cpi_ctx_a = CpiContext::new(cpi_program.clone(), cpi_accounts_token_a);
    anchor_spl::token::transfer(cpi_ctx_a, amount_a)?;

    msg!("Transferred {} of Token A to Vault", amount_a);

    let cpi_accounts_token_b = Transfer {
        from: ctx.accounts.provider_token_b_ata.to_account_info(),
        to: ctx.accounts.token_vault_b.to_account_info(),
        authority: ctx.accounts.provider.to_account_info(),
    };

    let cpi_ctx_b = CpiContext::new(cpi_program, cpi_accounts_token_b);
    anchor_spl::token::transfer(cpi_ctx_b, amount_b)?;

    msg!("Transferred {} of Token B to Vault", amount_b);

    let lp_tokens_to_mint = if ctx.accounts.lp_mint.supply == 0 {
        let product = (amount_a as u128)
            .checked_mul(amount_b as u128)
            .ok_or(ErrorCode::MathOverflow)?;
        
        let sqrt = (product as f64).sqrt() as u64;
        sqrt
    } else {
        let vault_a_balance = ctx.accounts.token_vault_a.amount - amount_a;
        let vault_b_balance = ctx.accounts.token_vault_b.amount - amount_b;
        
        require!(vault_a_balance > 0 && vault_b_balance > 0, ErrorCode::InvalidPoolState);

        let lp_from_a = (amount_a as u128)
            .checked_mul(ctx.accounts.lp_mint.supply as u128)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(vault_a_balance as u128)
            .ok_or(ErrorCode::MathOverflow)? as u64;

        let lp_from_b = (amount_b as u128)
            .checked_mul(ctx.accounts.lp_mint.supply as u128)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(vault_b_balance as u128)
            .ok_or(ErrorCode::MathOverflow)? as u64;

        std::cmp::min(lp_from_a, lp_from_b)
    };

    require!(lp_tokens_to_mint >= min_lp_tokens, ErrorCode::SlippageExceeded);

    let deposit_recept = &mut ctx.accounts.deposit_recept;
    deposit_recept.pool = Pubkey::default();
    deposit_recept.liquidity_provider = ctx.accounts.provider.key();
    deposit_recept.amount_a = amount_a;
    deposit_recept.amount_b = amount_b;
    deposit_recept.lp_tokens_minted = lp_tokens_to_mint;
    msg!("Deposit Recept created successfully!");

    Ok(())
}


#[account]
#[derive(Debug, InitSpace)]
pub struct DepositRecept {
    pub pool: Pubkey,
    pub liquidity_provider: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub lp_tokens_minted: u64,
}

#[delegate]
#[derive(Accounts)]
pub struct DelegateDepositReceipt<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

    #[account(
        mut,
        del,  // Now we can delegate the existing account
        seeds = [b"deposit_recept", provider.key().as_ref()],
        bump
    )]
    pub deposit_recept: Account<'info, DepositRecept>,
}

pub fn delegate_deposit_receipt(
    ctx: Context<DelegateDepositReceipt>,
    commit_frequency: u32,
    validator_key: Pubkey,
) -> Result<()> {
    let delegate_config = DelegateConfig {
        commit_frequency_ms: commit_frequency,
        validator: Some(validator_key),
    };

    let provider = ctx.accounts.provider.key();
    let seeds = &[b"deposit_recept", provider.as_ref()];

    ctx.accounts.delegate_deposit_recept(
        &ctx.accounts.provider,
        seeds,
        delegate_config,
    )?;

    msg!("Deposit receipt delegated successfully!");
    msg!("Receipt : {:?}", ctx.accounts.deposit_recept);

    Ok(())
}
