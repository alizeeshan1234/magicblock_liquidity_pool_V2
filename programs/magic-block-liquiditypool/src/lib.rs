use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token};

use ephemeral_rollups_sdk::anchor::{commit, delegate, ephemeral};
use ephemeral_rollups_sdk::ephem::{MagicInstructionBuilder, MagicAction, CallHandler, CommitType, CommitAndUndelegate, UndelegateType};
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

declare_id!("8jL3EsFxdQpSWDQudJiU3XSoaKsDoYSLZB1SFw2PKMVw");

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
    
    pub fn process_close_deposit_receipt(ctx: Context<CloseDepositReceiptInfo>) -> Result<()> {
        instructions::add_liquidity_on_chain::close_deposit_receipt(ctx)
    }

    pub fn process_commit_and_mint_lp_tokens(ctx: Context<CommitAndMintLpTokens>) -> Result<()> {

        let deposit_recept = &ctx.accounts.deposit_recept;

        let mint_instruction_data = anchor_lang::InstructionData::data(
            &crate::instruction::ProcessMintLpTokens {
                mint_amount: deposit_recept.lp_tokens_minted,
            }
        );

        let action_args = ActionArgs {
            escrow_index: 0,
            data: mint_instruction_data,
        };

        let accounts = vec![
            ShortAccountMeta {
                pubkey: ctx.accounts.provider.key(),
                is_writable: false,
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.transfer_authority.key(),
                is_writable: false
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.lp_mint.key(),
                is_writable: true,   
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.provider_lp_ata.key(),
                is_writable: true,
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.token_program.key(),
                is_writable: false
            },
        ];

        let mint_handler = CallHandler {
            args: action_args,
            compute_units: 200_000,
            escrow_authority: ctx.accounts.provider.to_account_info(),
            destination_program: crate::ID,
            accounts
        };

        let close_ix_data = anchor_lang::InstructionData::data(
            &crate::instruction::ProcessCloseDepositReceipt {}
        );

        let action_args_close = ActionArgs {
            escrow_index: 0,
            data: close_ix_data
        };

        let close_handler_accounts = vec![
            ShortAccountMeta {
                pubkey: ctx.accounts.provider.key(),
                is_writable: true
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.deposit_recept.key(),
                is_writable: true
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.system_program.key(),
                is_writable: false
            }
        ];

        let close_handler = CallHandler {
            args: action_args_close,
            compute_units: 200_000,
            escrow_authority: ctx.accounts.provider.to_account_info(),
            destination_program: crate::ID,
            accounts: close_handler_accounts
        };

        MagicInstructionBuilder {
            payer: ctx.accounts.provider.to_account_info(),
            magic_context: ctx.accounts.magic_context.to_account_info(),
            magic_program: ctx.accounts.magic_program.to_account_info(),
            magic_action: MagicAction::CommitAndUndelegate(
                CommitAndUndelegate {
                    commit_type: CommitType::WithHandler {
                        commited_accounts: vec![ctx.accounts.deposit_recept.to_account_info()],
                        call_handlers: vec![mint_handler, close_handler],
                    },
                    undelegate_type: UndelegateType::Standalone
                }
            ),
        }.build_and_invoke()?;

        Ok(())
    }

    pub fn process_remove_liquidity_on_chain(
        ctx: Context<RemoveLiquidityOnChain>, 
        params: RemoveLiquidityParams
    ) -> Result<()> {
        instructions::remove_liquidity_on_chain::remove_liquidity_on_chain(ctx, params)
    }

    pub fn process_delegate_remove_liquidity_receipt(ctx: Context<DelegateRemoveLiquidityReceipt>, commit_frequency: u32, validator_key: Pubkey) -> Result<()> {
        instructions::remove_liquidity_on_chain::delegate_remove_liquidity_receipt(ctx, commit_frequency, validator_key)
    }

    pub fn process_remove_liquidity_er(ctx: Context<RemoveLiquidityER>, params: RemoveLiquidityErParams) -> Result<()> {
        instructions::remove_liquidity_er(ctx, params)
    }

    pub fn process_burn_lp_tokens(ctx: Context<BurnLpTokens>, burn_amount: u64) -> Result<()> {
        instructions::burn_lp_tokens::burn_lp_tokens(ctx, burn_amount)
    }

    pub fn process_commit_and_burn_lp_tokens(ctx: Context<CommitAndBurnMintLpTokens>) -> Result<()> {

        let withdraw_recept = &ctx.accounts.withdraw_recept;

        let instruction_data = anchor_lang::InstructionData::data(
            &crate::instruction::ProcessBurnLpTokens {
                burn_amount: withdraw_recept.lp_tokens_to_burn
            }
        );

        let action_args = ActionArgs {
            escrow_index: 0,
            data: instruction_data
        };

        let accounts = vec![
            ShortAccountMeta {
                pubkey: ctx.accounts.provider.key(),
                is_writable: false
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.transfer_authority.key(),
                is_writable: false,
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.lp_mint.key(),
                is_writable: true,
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.provider_lp_ata.key(),
                is_writable: true
            },
            ShortAccountMeta {
                pubkey: ctx.accounts.token_program.key(),
                is_writable: false
            }
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
                    ctx.accounts.withdraw_recept.to_account_info(),
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
        bump
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [b"liquidity_provider_account_info", provider.key().as_ref()],
        bump,
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
        bump,
    )]
    pub transfer_authority: UncheckedAccount<'info>,

    pub lp_mint: UncheckedAccount<'info>,

    pub provider_lp_ata: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,

    /// CHECK: Magic context account
    #[account(mut)]
    pub magic_context: UncheckedAccount<'info>,
    
    /// CHECK: Magic program
    pub magic_program: UncheckedAccount<'info>,
}

#[commit]
#[derive(Accounts)]
pub struct CommitAndBurnMintLpTokens<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool", pool.name.as_bytes()],
        bump
    )]
    pub pool: Account<'info, Pool>,

     #[account(
        mut,
        seeds = [b"liquidity_provider_account_info", provider.key().as_ref()],
        bump,
    )]
    pub liquidity_provider: Account<'info, LiquidityProvider>,

    #[account(
        mut,
        seeds = [b"withdraw_recept", provider.key().as_ref()],
        bump,
    )]
    pub withdraw_recept: Account<'info, WithdrawRecept>,

    #[account(
        seeds = [b"transfer_authority"],
        bump,
    )]
    pub transfer_authority: UncheckedAccount<'info>,

    pub lp_mint: UncheckedAccount<'info>,

    pub provider_lp_ata: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,

     /// CHECK: Magic context account
    #[account(mut)]
    pub magic_context: UncheckedAccount<'info>,
    
    /// CHECK: Magic program
    pub magic_program: UncheckedAccount<'info>,
}