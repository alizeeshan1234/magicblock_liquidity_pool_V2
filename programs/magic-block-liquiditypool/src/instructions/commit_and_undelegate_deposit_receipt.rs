use anchor_lang::prelude::*;
use anchor_spl::token::Token;

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

pub fn close_deposit_receipt(_ctx: Context<CloseDepositReceipt>) -> Result<()> {
    msg!("Closing deposit receipt account");
    msg!("Successfully closed deposit receipt account");
    Ok(())
}