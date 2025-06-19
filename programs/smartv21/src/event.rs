use anchor_lang::prelude::*;

// Event emitted on loan liquidation
#[event]
pub struct LoanLiquidatedEvent {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub liquidator: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}
