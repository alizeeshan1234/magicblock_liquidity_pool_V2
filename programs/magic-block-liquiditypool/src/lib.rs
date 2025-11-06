use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token};

use ephemeral_rollups_sdk::anchor::{commit, delegate, ephemeral};
use ephemeral_rollups_sdk::cpi::DelegateConfig;
use ephemeral_rollups_sdk::ephem::commit_and_undelegate_accounts;
use ephemeral_rollups_sdk::ephem::{MagicInstructionBuilder, MagicAction, CallHandler, CommitType};
use ephemeral_rollups_sdk::{ActionArgs, ShortAccountMeta};
use anchor_lang::Discriminator;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

pub use constants::*;
pub use instructions::*;
pub use state::*;

use state::{pool::Pool, liquidity_provider::LiquidityProvider};


declare_id!("6aaiUUVLjJaiqcdTRNcJy5Ekb8XQu3AY2nfB3q2KhvzH");

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

    pub fn process_deposit_add_liquidity_on_chain(
        ctx: Context<DepositLiquidityOnchain>, 
        params: DepositLiquidityParams
    ) -> Result<()> {
        instructions::add_liquidity_on_chain::deposit_liquidity_on_chain(ctx, params)
    }

    pub fn process_delegate_add_liquidity_receipt(ctx: Context<DelegateDepositReceipt>, commit_frequency: u32, validator_key: Pubkey) -> Result<()> {
        instructions::add_liquidity_on_chain::delegate_deposit_receipt(ctx, commit_frequency, validator_key)
    }

    pub fn process_add_liquidity_er(ctx: Context<AddLiquidityER>, params: AddLiquidityErParams) -> Result<()> {
        instructions::add_liquidity_er::add_liquidity_er(ctx, params)
    }

    pub fn process_mint_lp_tokens(ctx: Context<MintLpTokens>, mint_amount: u64) -> Result<()> {
        instructions::mint_lp_tokens::mint_lp_tokens(ctx, mint_amount)
    }

    pub fn process_commit_and_mint_lp_tokens(ctx: Context<CommitAndMintLpTokens>) -> Result<()> {

        let deposit_recept = &ctx.accounts.deposit_recept;

        let instruction_data = anchor_lang::InstructionData::data(
            &crate::instruction::ProcessMintLpTokens {
                mint_amount: deposit_recept.lp_tokens_minted,
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
                pubkey: ctx.accounts.transfer_authority.key(),
                is_writable: false
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.lp_mint.key(),
                is_writable: true
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.provider_lp_ata.key(),
                is_writable: true
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.token_program.key(),
                is_writable: false
            },
        ];

        let call_handler = CallHandler {
            args: action_args,
            compute_units: 200_000,
            escrow_authority: ctx.accounts.provider.to_account_info(),
            destination_program: crate::ID,
            accounts
        };

        let magic_builder = MagicInstructionBuilder {
            payer: ctx.accounts.provider.to_account_info(),
            magic_context: ctx.accounts.magic_context.to_account_info(),
            magic_program: ctx.accounts.magic_program.to_account_info(),
            magic_action: MagicAction::Commit(CommitType::WithHandler {
                commited_accounts: vec![
                    ctx.accounts.pool.to_account_info(),
                    ctx.accounts.liquidity_provider.to_account_info(),
                    ctx.accounts.deposit_recept.to_account_info(),
                ],
                call_handlers: vec![call_handler]
            })
        };

        magic_builder.build_and_invoke()?;
        Ok(())
    }

}

#[commit]
#[derive(Accounts)]
pub struct CommitAndMintLpTokens<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool", pool.name.as_bytes()],
        bump = pool.bump
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [b"liquidity_provider_account_info", provider.key().as_ref()],
        bump = liquidity_provider.bump
    )]
    pub liquidity_provider: Account<'info, LiquidityProvider>,

    #[account(
        mut,
        seeds = [b"deposit_recept", provider.key().as_ref()],
        bump,
    )]
    pub deposit_recept: Account<'info, DepositRecept>,

    #[account(
        seeds = [b"transfer_authority"],
        bump
    )]
    pub transfer_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"lp_token_mint"],
        bump,
    )]
    pub lp_mint: UncheckedAccount<'info>,

    #[account(mut)]
    pub provider_lp_ata: UncheckedAccount<'info>,

    pub token_program: UncheckedAccount<'info>,

    /// CHECK: Magic context account
    pub magic_context: UncheckedAccount<'info>,
    
    /// CHECK: Magic program
    pub magic_program: UncheckedAccount<'info>,
}
