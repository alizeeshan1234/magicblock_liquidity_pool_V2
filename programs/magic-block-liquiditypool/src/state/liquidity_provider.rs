use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, InitSpace)]
pub struct LiquidityPoolInfo {
    pub pool: Pubkey,           
    pub token_mint: Pubkey,     
    pub liquidity_amount: u64,  
    pub lp_tokens: u64,         
}

impl Default for LiquidityPoolInfo {
    fn default() -> Self {
        LiquidityPoolInfo {
            pool: Pubkey::default(),
            token_mint: Pubkey::default(),
            liquidity_amount: 0,
            lp_tokens: 0
        }
    }
}

#[account]
#[derive(Debug, InitSpace, Copy)]
pub struct LiquidityProvider {
    pub provider: Pubkey,                    
    pub total_liquidity_provided: u64,       
    pub total_lp_tokens: u64,                
    pub liquidity_pools_info: [LiquidityPoolInfo; 10],
    pub latest_liquidity_provided_on: i64,   
    pub bump: u8,                            
}
