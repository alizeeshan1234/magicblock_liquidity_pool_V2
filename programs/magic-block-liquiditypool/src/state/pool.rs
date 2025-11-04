use anchor_lang::prelude::*;

#[account]
#[derive(Default, InitSpace, Debug)]
pub struct Pool {
    pub version: u8,
    pub authority: Pubkey,
    pub pool_id: u64,
    #[max_len(50)]
    pub name: String,
    pub lp_mint: Pubkey,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub token_a_vault: Pubkey,
    pub token_b_vault: Pubkey,
    
    // Virtual reserves tracked in ER
    pub reserve_a: u64,
    pub reserve_b: u64,
    pub total_lp_supply: u64,

    pub fees: FeeConfig,
    pub status: PoolStatus,
    pub created_at: i64,
    pub updated_at: i64,
    pub bump: u8,
    pub lp_mint_bump: u8,
    pub token_a_vault_bump: u8,
    pub token_b_vault_bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, InitSpace, Debug)]
pub struct FeeConfig {
    pub trade_fee_bps: u16,
    pub protocol_fee_bps: u16,
    pub fee_recipient: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, PartialEq, InitSpace, Debug)]
pub struct PoolStatus {
    pub is_active: bool,
    pub is_paused: bool,
    pub is_migrating: bool,
}