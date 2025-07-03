pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod event;

use anchor_lang::prelude::*;

pub use constants::*;
use instructions::*;
pub use state::*;
pub use event::*;

declare_id!("FUBDGjrPeRAXzzxDBhozp6PgiHBPk4gFAYGoab5qZLrp");

#[program]
pub mod smartv21 {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        syncer: Pubkey,
        verifier: Pubkey,
        service_fee: u64,
    ) -> Result<()> {
        instructions::initialize(ctx, syncer, verifier, service_fee)
    }

    pub fn update_service_fee(
        ctx: Context<UpdateServiceFee>,
        new_fixed_fee: u64
    ) -> Result<()> {
        instructions::update_service_fee(ctx, new_fixed_fee)
    }

    pub fn create_liquidity_pool(
        ctx: Context<CreateLiquidityPool>,
        init_amount_0: u64,
        init_amount_1: u64,
        open_time: u64,
        loan_duration: i64
    ) -> Result<()> {
        instructions::create_liquidity_pool(ctx, init_amount_0, init_amount_1, open_time, loan_duration)
    }

    pub fn send_lp_tokens(ctx: Context<SendLPTokens>) -> Result<()> {
        instructions::send_lp_tokens(ctx)
    }

    pub fn deposit(
        ctx: Context<ManageServiceVault>,
        amount: u64
    ) -> Result<()> {
        instructions::deposit(ctx, amount)
    }

    pub fn withdraw(
        ctx: Context<ManageServiceVault>,
        amount: u64
    ) -> Result<()> {
        instructions::withdraw(ctx, amount)
    }

    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        lp_token_amount: u64,
        minimum_token_0_amount: u64,
        minimum_token_1_amount: u64
    ) -> Result<()> {
        instructions::remove_liquidity(ctx, lp_token_amount, minimum_token_0_amount, minimum_token_1_amount)
    }

    pub fn liquidate_loan(
        ctx: Context<LiquidateLoan>,
        lp_token_amount: u64,
        minimum_token_0_amount: u64,
        minimum_token_1_amount: u64
    ) -> Result<()> {
        instructions::liquidate_loan(ctx, lp_token_amount, minimum_token_0_amount, minimum_token_1_amount)
    }
}
