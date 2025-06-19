use anchor_lang::prelude::*;

// Account Structures
#[account]
pub struct Config {
    pub amount: u64, // wrap sol amount
    pub admin: Pubkey,
    pub syncer: Pubkey,
    pub verifier: Pubkey,
    pub service_fee: u64, // Fixed service fee in lamports
    pub is_paused: bool,
}
#[account]
pub struct PoolLoan {
    pub user: Pubkey,
    pub pool: Pubkey,
    pub lp_mint: Pubkey,
    pub token_mint: Pubkey, // Added to track the token mint
    pub init_sol_amount: u64,
    pub init_token_amount: u64,
    pub loan_start_time: i64,
    pub loan_duration: i64,
    pub is_repaid: bool,
}

impl Config {
    pub const LEN: usize = 8 + // wrap sol amount (u64)
                           32 +  // admin
                           32 +  // syncer
                           32 +  // verifier
                           8 +   // fixed_service_fee (u64)
                           1;    // is_paused (bool)
}
impl PoolLoan {
    pub const LEN: usize = 32 + // user
                           32 + // pool
                           32 + // lp_mint
                           32 + // token_mint
                           8 +  // init_sol_amount
                           8 +  // init_token_amount
                           8 +  // loan_start_time
                           8 +  // loan_duration
                           1;   // is_repaid
}
