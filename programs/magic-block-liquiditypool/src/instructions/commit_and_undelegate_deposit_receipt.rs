use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token, CloseAccount};

use ephemeral_rollups_sdk::anchor::{commit, delegate, ephemeral};
use ephemeral_rollups_sdk::cpi::DelegateConfig;
use ephemeral_rollups_sdk::ephem::commit_and_undelegate_accounts;
use ephemeral_rollups_sdk::ephem::{MagicInstructionBuilder, MagicAction, CallHandler, CommitType};
use ephemeral_rollups_sdk::{ActionArgs, ShortAccountMeta};
use anchor_lang::Discriminator;

use crate::instructions::add_liquidity_on_chain::DepositRecept;

#[derive(Accounts)]
pub struct CloseDepositReceipt<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

    #[account(
        mut,
        seeds = [b"deposit_recept", provider.key().as_ref()],
        bump,
        close = provider
    )]
    pub deposit_recept: Account<'info, DepositRecept>,

    pub token_program: Program<'info, Token>,
}

pub fn close_deposit_receipt(ctx: Context<CloseDepositReceipt>) -> Result<()> {
    msg!("Closing deposit receipt account");
    msg!("Successfully closed deposit receipt account");
    Ok(())
}