use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer, Burn, burn};

use ephemeral_rollups_sdk::anchor::delegate;
use ephemeral_rollups_sdk::cpi::DelegateConfig;
use anchor_lang::Discriminator;

use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct RemoveLiquidityOnChain<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,

    #[account(
        seeds = [b"transfer_authority"],
        bump,
    )]
    pub transfer_authority: UncheckedAccount<'info>,

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
        space = 8 + WithdrawRecept::INIT_SPACE,
        seeds = [b"withdraw_recept", provider.key().as_ref()],
        bump,
    )]
    pub withdraw_recept: Account<'info, WithdrawRecept>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(Debug, InitSpace)]
pub struct WithdrawRecept {
    pub pool: Pubkey,
    pub liquidity_provider: Pubkey,
    pub lp_tokens_to_burn: u64,
    pub amount_a_withdrawn: u64,
    pub amount_b_withdrawn: u64,
}

#[derive(Clone, AnchorDeserialize, AnchorSerialize)]
pub struct RemoveLiquidityParams {
    pub lp_tokens_to_burn: u64,
    pub min_amount_a: u64,
    pub min_amount_b: u64,
    pub pool: Pubkey,
}

pub fn remove_liquidity_on_chain(
    ctx: Context<RemoveLiquidityOnChain>,
    params: RemoveLiquidityParams,
) -> Result<()> {
    // Validate inputs
    require!(params.lp_tokens_to_burn > 0, ErrorCode::InvalidAmount);
    require!(params.min_amount_a > 0, ErrorCode::InvalidAmount);
    require!(params.min_amount_b > 0, ErrorCode::InvalidAmount);

    // Check provider has enough LP tokens
    // require!(
    //     ctx.accounts.provider_token_lp_ata.amount >= params.lp_tokens_to_burn,
    //     ErrorCode::InsufficientLpTokens
    // );

    // Check LP supply is not zero
    // require!(ctx.accounts.lp_mint.supply > 0, ErrorCode::InvalidPoolState);

    // Calculate withdrawal amounts based on LP token ratio
    let vault_a_balance = ctx.accounts.token_vault_a.amount;
    let vault_b_balance = ctx.accounts.token_vault_b.amount;
    let total_lp_supply = ctx.accounts.lp_mint.supply;

    require!(vault_a_balance > 0 && vault_b_balance > 0, ErrorCode::InsufficientFunds);

    let amount_a_to_withdraw = (params.lp_tokens_to_burn as u128)
        .checked_mul(vault_a_balance as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(total_lp_supply as u128)
        .ok_or(ErrorCode::MathOverflow)? as u64;

    let amount_b_to_withdraw = (params.lp_tokens_to_burn as u128)
        .checked_mul(vault_b_balance as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(total_lp_supply as u128)
        .ok_or(ErrorCode::MathOverflow)? as u64;

    // // Check slippage protection
    // require!(
    //     amount_a_to_withdraw >= params.min_amount_a,
    //     ErrorCode::SlippageExceeded
    // );
    // require!(
    //     amount_b_to_withdraw >= params.min_amount_b,
    //     ErrorCode::SlippageExceeded
    // );

    // Verify vaults have enough tokens
    require!(
        vault_a_balance >= amount_a_to_withdraw,
        ErrorCode::InsufficientFunds
    );
    require!(
        vault_b_balance >= amount_b_to_withdraw,
        ErrorCode::InsufficientFunds
    );

    // Step 1: Burn LP tokens from provider
    let cpi_accounts_burn = Burn {
        mint: ctx.accounts.lp_mint.to_account_info(),
        from: ctx.accounts.provider_token_lp_ata.to_account_info(),
        authority: ctx.accounts.provider.to_account_info(),
    };

    let cpi_ctx_burn = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts_burn,
    );
    burn(cpi_ctx_burn, params.lp_tokens_to_burn)?;

    msg!("Burned {} LP tokens", params.lp_tokens_to_burn);

    // Step 2: Transfer tokens from vaults to provider
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"transfer_authority".as_ref(),
        &[ctx.bumps.transfer_authority],
    ]];

    // Transfer Token A
    let cpi_accounts_token_a = Transfer {
        from: ctx.accounts.token_vault_a.to_account_info(),
        to: ctx.accounts.provider_token_a_ata.to_account_info(),
        authority: ctx.accounts.transfer_authority.to_account_info(),
    };

    let cpi_ctx_a = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts_token_a,
        signer_seeds,
    );
    anchor_spl::token::transfer(cpi_ctx_a, amount_a_to_withdraw)?;

    msg!("Transferred {} of token A to provider", amount_a_to_withdraw);

    // Transfer Token B
    let cpi_accounts_token_b = Transfer {
        from: ctx.accounts.token_vault_b.to_account_info(),
        to: ctx.accounts.provider_token_b_ata.to_account_info(),
        authority: ctx.accounts.transfer_authority.to_account_info(),
    };

    let cpi_ctx_b = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts_token_b,
        signer_seeds,
    );
    anchor_spl::token::transfer(cpi_ctx_b, amount_b_to_withdraw)?;

    msg!("Transferred {} of token B to provider", amount_b_to_withdraw);

    // Step 3: Create withdraw receipt
    let withdraw_receipt = &mut ctx.accounts.withdraw_recept;
    withdraw_receipt.pool = params.pool;
    withdraw_receipt.liquidity_provider = ctx.accounts.provider.key();
    withdraw_receipt.lp_tokens_to_burn = params.lp_tokens_to_burn;
    withdraw_receipt.amount_a_withdrawn = amount_a_to_withdraw;
    withdraw_receipt.amount_b_withdrawn = amount_b_to_withdraw;

    msg!("Created withdraw receipt account");
    msg!("LP tokens burned: {}", params.lp_tokens_to_burn);
    msg!("Token A withdrawn: {}", amount_a_to_withdraw);
    msg!("Token B withdrawn: {}", amount_b_to_withdraw);

    Ok(())
}

#[delegate]
#[derive(Accounts)]
pub struct DelegateRemoveLiquidityReceipt<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

    #[account(
        mut,
        del,
        seeds = [b"withdraw_recept", provider.key().as_ref()],
        bump,
    )]
    pub withdraw_recept: Account<'info, WithdrawRecept>,
}

pub fn delegate_remove_liquidity_receipt(
    ctx: Context<DelegateRemoveLiquidityReceipt>,
    commit_frequency: u32,
    validator_key: Pubkey,
) -> Result<()> {
    let delegate_config = DelegateConfig {
        commit_frequency_ms: commit_frequency,
        validator: Some(validator_key),
    };

    let provider = ctx.accounts.provider.key();
    let seeds = &[b"withdraw_recept", provider.as_ref()];

    ctx.accounts.delegate_withdraw_recept(
        &ctx.accounts.provider,
        seeds,
        delegate_config,
    )?;

    msg!("Withdraw receipt delegated successfully!");
    msg!("Receipt: {:?}", ctx.accounts.withdraw_recept);

    Ok(())
}