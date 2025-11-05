use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::state::pool::{Pool, PoolStatus, FeeConfig};

use anchor_spl::metadata::mpl_token_metadata::{
    instructions::CreateMetadataAccountV3CpiBuilder,
    types::DataV2,
    ID as mpl_ID,
};

#[derive(Accounts)]
#[instruction(params: AddPoolParams)]
pub struct InitializeLiquidityPool<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"transfer_authority"],
        bump,
    )]
    pub transfer_authority: AccountInfo<'info>,

    pub mint_a: Account<'info, Mint>,

    pub mint_b: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        mint::authority = transfer_authority,
        mint::freeze_authority = transfer_authority,
        mint::decimals = 6,
        seeds = [b"lp_token_mint"],
        bump
    )]
    pub lp_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        seeds = [b"lp_token_account"],
        bump,
        token::mint = lp_mint,
        token::authority = transfer_authority
    )]
    pub lp_token_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = admin,
        space = Pool::INIT_SPACE,
        seeds = [b"pool", params.name.as_bytes()],
        bump
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        init,
        payer = admin,
        seeds = [b"token_account_a", mint_a.key().as_ref()],
        bump,
        token::mint = mint_a,
        token::authority = transfer_authority
    )]
    pub token_vault_a: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = admin,
        seeds = [b"token_account_b", mint_b.key().as_ref()],
        bump,
        token::mint = mint_b,
        token::authority = transfer_authority
    )]
    pub token_vault_b: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    /// CHECK: Validated by Metaplex CPI
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,
 
    /// CHECK: Verified by address constraint
    #[account(address = mpl_ID)]
    pub metadata_program: UncheckedAccount<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddPoolParams {
    pub pool_id: u64,
    pub name: String,
    pub max_aum_usd: u64,
    pub metadata_title: String,
    pub metadata_symbol: String,
    pub metadata_uri: String,
    pub trade_fees: u16,
    pub protocol_fees: u16,
    pub fee_recipient: Pubkey,
}
pub fn initialize_liquidity_pool(ctx: Context<InitializeLiquidityPool>, params: AddPoolParams) -> Result<()> {
    msg!("Initializing Liquidity Pool...");

    let liquidity_pool_config = &mut ctx.accounts.pool;

    liquidity_pool_config.fees = FeeConfig {
        trade_fee_bps: params.trade_fees,
        protocol_fee_bps: params.protocol_fees,
        fee_recipient: params.fee_recipient,
    };

    liquidity_pool_config.status = PoolStatus {
        is_active: true,
        is_paused: false,
        is_migrating: false,
    };

    liquidity_pool_config.version = 1;
    liquidity_pool_config.authority = *ctx.accounts.transfer_authority.key;
    liquidity_pool_config.pool_id = params.pool_id;
    liquidity_pool_config.name = params.name;
    liquidity_pool_config.lp_mint = ctx.accounts.lp_mint.key();
    liquidity_pool_config.token_a = ctx.accounts.mint_a.key();
    liquidity_pool_config.token_b = ctx.accounts.mint_b.key();
    liquidity_pool_config.token_a_vault = ctx.accounts.token_vault_a.key();
    liquidity_pool_config.token_b_vault = ctx.accounts.token_vault_b.key();
    
    let clock = Clock::get()?; 
    liquidity_pool_config.created_at = clock.unix_timestamp;
    liquidity_pool_config.updated_at = clock.unix_timestamp;
    
    liquidity_pool_config.bump = ctx.bumps.pool;
    liquidity_pool_config.lp_mint_bump = ctx.bumps.lp_mint;
    liquidity_pool_config.token_a_vault_bump = ctx.bumps.token_vault_a;
    liquidity_pool_config.token_b_vault_bump = ctx.bumps.token_vault_b; // ✅ Fixed bug!

    msg!("Liquidity Pool Initialized Successfully!");

    msg!("Creating LP Token Metadata...");

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"transfer_authority",
        &[ctx.bumps.transfer_authority],
    ]];

    CreateMetadataAccountV3CpiBuilder::new(&ctx.accounts.metadata_program)
        .metadata(&ctx.accounts.metadata_account)
        .mint(&ctx.accounts.lp_mint.to_account_info())
        .mint_authority(&ctx.accounts.transfer_authority.to_account_info())
        .payer(&ctx.accounts.admin.to_account_info())
        .update_authority(&ctx.accounts.transfer_authority.to_account_info(), true)
        .system_program(&ctx.accounts.system_program.to_account_info())
        .data(DataV2 {
            name: params.metadata_title,
            symbol: params.metadata_symbol, 
            uri: params.metadata_uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        })
        .is_mutable(true)
        .invoke_signed(signer_seeds)?;

    msg!("LP Token Created!");

    Ok(())
}