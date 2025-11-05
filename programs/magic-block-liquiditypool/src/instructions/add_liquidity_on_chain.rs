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
    pub escrow: AccountInfo<'info>,
    /// CHECK: injected - will be moved to end
    pub escrow_auth: AccountInfo<'info>,

}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CommitAndAddLiquidityParams {
    pub user: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub min_lp_tokens: u64,
}

pub fn deposit_liquidity(ctx: Context<DepositLiquidityOnChain>, params: CommitAndAddLiquidityParams) -> Result<()> {

    let provider_token_a_data = ctx.accounts.provider_token_a_ata.try_borrow_data()?;
    let provider_token_a_account = TokenAccount::try_deserialize(&mut &provider_token_a_data[..])?;
    drop(provider_token_a_data);

    let provider_token_b_data = ctx.accounts.provider_token_b_ata.try_borrow_data()?;
    let provider_token_b_account = TokenAccount::try_deserialize(&mut &provider_token_b_data[..])?;
    drop(provider_token_b_data);

    msg!("Provider Token A Balance: {}", provider_token_a_account.amount);
    msg!("Provider Token B Balance: {}", provider_token_b_account.amount);

    let vault_a_data = ctx.accounts.token_vault_a.try_borrow_data()?;
    let vault_a_account = TokenAccount::try_deserialize(&mut &vault_a_data[..])?;
    drop(vault_a_data);
    
    let vault_b_data = ctx.accounts.token_vault_b.try_borrow_data()?;
    let vault_b_account = TokenAccount::try_deserialize(&mut &vault_b_data[..])?;
    drop(vault_b_data);

    msg!("Token Vault A Balance: {}", vault_a_account.amount);
    msg!("Token Vault B Balance: {}", vault_b_account.amount);

    msg!("Vault A: {}", ctx.accounts.token_vault_a.key());
    msg!("Vault B: {}", ctx.accounts.token_vault_b.key());

    if params.amount_a == 0 || params.amount_b == 0 {
        return Err(ErrorCode::InvalidAmount.into());
    }

    if provider_token_a_account.amount < params.amount_a {
        return Err(ErrorCode::InsufficientFunds.into());
    }

    if provider_token_b_account.amount < params.amount_b {
        return Err(ErrorCode::InsufficientFunds.into());
    }

    let mut pool_data = ctx.accounts.pool.try_borrow_mut_data()?;
    let mut pool = Pool::try_deserialize(&mut &pool_data[..])?;

    let reserve_a = pool.reserve_a;
    let reserve_b = pool.reserve_b;
    let total_lp_supply = pool.total_lp_supply;

    let lp_tokens_to_mint = if total_lp_supply == 0 {
        let product = (params.amount_a as u128)
            .checked_mul(params.amount_b as u128)
            .ok_or(ErrorCode::MathOverflow)?;
        (product as f64).sqrt() as u64
    } else {
        require!(reserve_a > 0 && reserve_b > 0, ErrorCode::InvalidAmount);
        
        let share_a = (params.amount_a as u128)
            .checked_mul(total_lp_supply as u128)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(reserve_a as u128)
            .ok_or(ErrorCode::MathOverflow)? as u64;
            
        let share_b = (params.amount_b as u128)
            .checked_mul(total_lp_supply as u128)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(reserve_b as u128)
            .ok_or(ErrorCode::MathOverflow)? as u64;
        
        std::cmp::min(share_a, share_b)
    };

    msg!("Lp Tokens to Mint: {}", lp_tokens_to_mint);

    if lp_tokens_to_mint < params.min_lp_tokens {
        return Err(ErrorCode::SlippageExceeded.into());
    }

    msg!("Performing on-chain token transfers...");

    // Transfer Token A - CHANGE THIS
    // let transfer_a_ix = spl_token::instruction::transfer(
    //     &spl_token::ID,
    //     &ctx.accounts.provider_token_a_ata.key(),
    //     &ctx.accounts.token_vault_a.key(),
    //     &ctx.accounts.escrow_auth.key(), // ← Use escrow_auth
    //     &[],
    //     params.amount_a,
    // )?;

    // invoke(
    //     &transfer_a_ix,
    //     &[
    //         ctx.accounts.provider_token_a_ata.clone(),
    //         ctx.accounts.token_vault_a.clone(),
    //         ctx.accounts.escrow_auth.clone(), // ← Use escrow_auth
    //         ctx.accounts.token_program.clone(),
    //     ],
    // )?; 

    // // Transfer Token B - CHANGE THIS
    // let transfer_b_ix = spl_token::instruction::transfer(
    //     &spl_token::ID,
    //     &ctx.accounts.provider_token_b_ata.key(),
    //     &ctx.accounts.token_vault_b.key(),
    //     &ctx.accounts.escrow_auth.key(), // ← Use escrow_auth
    //     &[],
    //     params.amount_b,
    // )?;

    // invoke(
    //     &transfer_b_ix,
    //     &[
    //         ctx.accounts.provider_token_b_ata.clone(),
    //         ctx.accounts.token_vault_b.clone(),
    //         ctx.accounts.escrow_auth.clone(), // ← Use escrow_auth
    //         ctx.accounts.token_program.clone(),
    //     ],
    // )?;
    msg!("Transferred {} Token A and {} Token B", params.amount_a, params.amount_b);


    msg!("Hello World!");

    Ok(())
}
