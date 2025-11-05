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


declare_id!("6k3vr9R4VexfTwF1BMBNEujEqtZbV9caYv9mGS51GFf9");

// #[ephemeral]
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
        amount_a: u64,
        amount_b: u64,
        min_lp_tokens: u64,
    ) -> Result<()> {
        instructions::add_liquidity_on_chain::deposit_liquidity_on_chain(ctx, amount_a, amount_b, min_lp_tokens)
    }

    pub fn process_delegate_add_liquidity_receipt(ctx: Context<DelegateDepositReceipt>, commit_frequency: u32, validator_key: Pubkey) -> Result<()> {
        instructions::add_liquidity_on_chain::delegate_deposit_receipt(ctx, commit_frequency, validator_key)
    }

    pub fn process_add_liquidity_er(ctx: Context<AddLiquidityER>, params: AddLiquidityErParams) -> Result<()> {
        instructions::add_liquidity_er::add_liquidity_er(ctx, params)
    }

}

