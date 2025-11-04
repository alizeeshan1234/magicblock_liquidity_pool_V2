use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token};

use ephemeral_rollups_sdk::anchor::{commit, delegate, ephemeral};
use ephemeral_rollups_sdk::cpi::DelegateConfig;
use ephemeral_rollups_sdk::ephem::commit_and_undelegate_accounts;
use ephemeral_rollups_sdk::ephem::{MagicInstructionBuilder, MagicAction, CallHandler, CommitType};
use ephemeral_rollups_sdk::{ActionArgs, ShortAccountMeta};
use anchor_lang::Discriminator;
// use ephemeral_rollups_sdk::consts::EXTERNAL_CALL_HANDLER_DISCRIMINATOR;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

pub use constants::*;
pub use instructions::*;
pub use state::*;

use crate::state::{liquidity_provider::LiquidityProvider, pool::Pool};

declare_id!("EiKhdqshVkzcftMP3icx2ySYtnDBk2zapz6JWnRusagH");

#[ephemeral]
#[program]
pub mod magic_block_liquiditypool {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }

    pub fn process_initialize_pool(ctx: Context<InitializeLiquidityPool>, params: AddPoolParams) -> Result<()> {
        instructions::init_pool::initialize_liquidity_pool(ctx, params)
    }

    pub fn process_delegate_pool(ctx: Context<DelegatePool>, commit_frequency: u32, validator_key: Pubkey) -> Result<()> {
        instructions::delegate_pool_er::delegate_pool(ctx, commit_frequency, validator_key)
    }

    pub fn process_initialize_liquidity_provider(ctx: Context<InitializeLiquidityProvider>) -> Result<()> {
        instructions::init_liquidity_provider::initialize_liquidity_provider(ctx)
    }

    pub fn process_delegate_liquidity_provider(ctx: Context<DelegateLiquidityProvider>, commit_frequency: u32, validator_key: Pubkey) -> Result<()> {
        instructions::delegate_liquidity_provider(ctx, commit_frequency, validator_key)
    }

    pub fn process_add_liquidity_er(ctx: Context<AddLiquidityER>, params: AddLiquidityErParams) -> Result<()> {
        instructions::add_liquidity_er::add_liquidity_er(ctx, params)
    }

    // pub fn process_execute_add_liquidity_transfer(ctx: Context<ExecuteAddLiquidityTransfer>, amount_a: u64, amount_b: u64,) -> Result<()> {
    //     instructions::add_liquidity_on_chain::execute_add_liquidity_transfer(ctx, amount_a, amount_b)
    // }

    // pub fn process_commit_add_liquidity( ctx: Context<CommitAddLiquidity>, params: CommitAddLiquidityParams) -> Result<()> {
    //     instructions::add_liquidity_on_chain::commit_add_liquidity(ctx, params)
    // }

    // #[instruction(discriminator = &EXTERNAL_CALL_HANDLER_DISCRIMINATOR)]
    pub fn process_deposit_liquidity_on_chain(ctx: Context<DepositLiquidityOnChain>, params: CommitAndAddLiquidityParams) -> Result<()> {
        instructions::add_liquidity_on_chain::deposit_liquidity(ctx, params)
    }

    pub fn process_commit_and_add_liquidity(ctx: Context<CommitAndAddLiquidity>, params: CommitAndAddLiquidityParams) -> Result<()> {

        let instruction_data = anchor_lang::InstructionData::data(
            &crate::instruction::ProcessDepositLiquidityOnChain {
                params
            }
        );

        let action_args = ActionArgs {
            escrow_index: 0,
            data: instruction_data,
        };

        let accounts = vec![
            ShortAccountMeta {
                pubkey: ctx.accounts.provider.key(),
                is_writable: true
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.mint_a.key(),
                is_writable: false,
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.mint_b.key(),
                is_writable: false
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.lp_mint.key(),
                is_writable: true
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.token_vault_a.key(),
                is_writable: true
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.token_vault_b.key(),
                is_writable: true,
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.provider_token_a_ata.key(),
                is_writable: true,
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.provider_token_b_ata.key(),
                is_writable: true,
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.provider_lp_token_account.key(),
                is_writable: true,
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.pool.key(),
                is_writable: true
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.liquidity_provider.key(),
                is_writable: true
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.token_program.key(),
                is_writable: false
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.system_program.key(),
                is_writable: false
            },
        ];

        let call_handler = CallHandler {
            args: action_args,
            compute_units: 300_000,
            escrow_authority: ctx.accounts.provider.to_account_info(),
            destination_program: crate::ID,
            accounts,
        };
    
        MagicInstructionBuilder {
            payer: ctx.accounts.provider.to_account_info(),
            magic_context: ctx.accounts.magic_context.to_account_info(),
            magic_program: ctx.accounts.magic_program.to_account_info(),
            magic_action: MagicAction::Commit(CommitType::WithHandler {
                commited_accounts: vec![
                    ctx.accounts.pool.to_account_info(), 
                    ctx.accounts.liquidity_provider.to_account_info()
                ],
                call_handlers: vec![call_handler]
            })
        }.build_and_invoke()?;

        Ok(())
    }
}

#[commit]
#[derive(Accounts)]
pub struct CommitAndAddLiquidity<'info> {
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

    /// CHECK: Will be committed - writable set in handler
    pub liquidity_provider: UncheckedAccount<'info>,

    /// CHECK: Will be committed - writable set in handler
    pub pool: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = provider
    )]
    pub provider_lp_token_account: Account<'info, TokenAccount>,

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
    
    pub token_vault_a: Account<'info, TokenAccount>,
    pub token_vault_b: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    /// CHECK: Your program ID
    pub program_id: AccountInfo<'info>,
}