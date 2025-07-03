use anchor_lang::prelude::*;
use anchor_spl::{
    token::Token,
    token_2022::{transfer_checked, TransferChecked},
    token_interface::{Mint, TokenAccount, Token2022},
};
use raydium_cpmm_cpi::{
    cpi,
    program::RaydiumCpmm,
    states::PoolState,
};


use crate::{ constants::*, state::*, error::ErrorCode};
use std::str::FromStr;

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut, seeds = [CONFIG_SEED.as_bytes()], bump)]
    pub config: Box<Account<'info, Config>>,

    #[account(
        mut,
        seeds = [POOL_LOAN_SEED.as_bytes(), pool_state.key().as_ref()],
        bump,
    )]
    pub pool_loan: Box<Account<'info, PoolLoan>>,

    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(
        mut,
        seeds = [LP_TOKEN_SEED.as_bytes(), pool_state.key().as_ref()],
        bump,
    )]
    pub service_token_lp: Box<InterfaceAccount<'info, TokenAccount>>,

     #[account(
        mut,
        seeds = [VAULT_SEED.as_bytes()],
        bump
    )]
    pub service_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    pub cp_swap_program: Program<'info, RaydiumCpmm>,
    /// Pays to mint the position
    pub owner: Signer<'info>,

    /// CHECK: pool vault and lp mint authority
    #[account(
        seeds = [
            raydium_cpmm_cpi::AUTH_SEED.as_bytes(),
        ],
        seeds::program = cp_swap_program,
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    /// Pool state account
    #[account(mut)]
    pub pool_state: AccountLoader<'info, PoolState>,

    /// Owner lp token account
    #[account(
        mut, 
        token::authority = owner
    )]
    pub owner_lp_token: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The owner's token account for receive token_0
    #[account(
        mut,
        token::mint = token_0_vault.mint,
        token::authority = owner
    )]
    pub token_0_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The owner's token account for receive token_1
    #[account(
        mut,
        token::mint = token_1_vault.mint,
        token::authority = owner
    )]
    pub token_1_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The address that holds pool tokens for token_0
    #[account(
        mut,
        constraint = token_0_vault.key() == pool_state.load()?.token_0_vault
    )]
    pub token_0_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The address that holds pool tokens for token_1
    #[account(
        mut,
        constraint = token_1_vault.key() == pool_state.load()?.token_1_vault
    )]
    pub token_1_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// token Program
    pub token_program: Program<'info, Token>,

    /// Token program 2022
    pub token_program_2022: Program<'info, Token2022>,

    /// The mint of token_0 vault
    #[account(
        address = token_0_vault.mint
    )]
    pub vault_0_mint: Box<InterfaceAccount<'info, Mint>>,

    /// The mint of token_1 vault
    #[account(
        address = token_1_vault.mint
    )]
    pub vault_1_mint: Box<InterfaceAccount<'info, Mint>>,

    /// Pool lp token mint
    #[account(
        mut,
        address = pool_state.load()?.lp_mint)
    ]
    pub lp_mint: Box<InterfaceAccount<'info, Mint>>,

    /// memo program
    /// CHECK:
    #[account(
        address = spl_memo::id()
    )]
    pub memo_program: UncheckedAccount<'info>,
}

pub fn remove_liquidity(
    ctx: Context<RemoveLiquidity>,
    lp_token_amount: u64,
    minimum_token_0_amount: u64,
    minimum_token_1_amount: u64,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let pool_loan = &mut ctx.accounts.pool_loan;
    // Verify loan is not already repaid
    require!(!pool_loan.is_repaid, ErrorCode::LoanAlreadyRepaid);
    // Verify caller is either user or service
    require!(
        ctx.accounts.owner.key() == pool_loan.user, 
        // ctx.accounts.owner.key() == config.admin || 
        // ctx.accounts.owner.key() == config.syncer,
        ErrorCode::Unauthorized
    );

    // Verify loan is not expired if called by user
    let current_time = Clock::get()?.unix_timestamp;
    // if ctx.accounts.owner.key() == pool_loan.user {
    require!(
        current_time <= pool_loan.loan_start_time + pool_loan.loan_duration,
        ErrorCode::LoanExpired
    );
    // }

    // Define PDA authority seeds
    let (_vault_authority, vault_bump) = Pubkey::find_program_address(
        &[POOL_LOAN_SEED.as_bytes(), ctx.accounts.pool_state.key().as_ref()],
        ctx.program_id,
    );
    let binding = ctx.accounts.pool_state.key();
    let signer_seeds: &[&[u8]] = &[
        POOL_LOAN_SEED.as_bytes(),
        binding.as_ref(),
        &[vault_bump],
    ];

    // Transfer LP tokens from service_token_lp to owner_lp_token account using PDA signer
    let _= transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.service_token_lp.to_account_info(),
                to: ctx.accounts.owner_lp_token.to_account_info(),
                authority: pool_loan.to_account_info(),
                mint: ctx.accounts.lp_mint.to_account_info()
            },
            &[signer_seeds]
        ),
        lp_token_amount,
        ctx.accounts.lp_mint.decimals
    );
    msg!("Transferred {} LP tokens from service to user", lp_token_amount);

    let wrapped_sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")
        .map_err(|_| error!(ErrorCode::InvalidWrappedSolMint))?;
    let is_token0_wrapped_sol = ctx.accounts.vault_0_mint.key() == wrapped_sol_mint;
    let is_token1_wrapped_sol = ctx.accounts.vault_1_mint.key() == wrapped_sol_mint;

    let mut pre_wrap_sol_amount = 0;
    let mut post_wrap_sol_amount = 0;

    let mut pre_token_amount = 0;
    let mut post_token_amount = 0;

    if is_token0_wrapped_sol {
        pre_wrap_sol_amount = ctx.accounts.token_0_account.amount;
        pre_token_amount = ctx.accounts.token_1_account.amount;
    } else {
        pre_wrap_sol_amount = ctx.accounts.token_1_account.amount;
        pre_token_amount = ctx.accounts.token_0_account.amount;
    }

    let cpi_accounts = cpi::accounts::Withdraw {
        owner: ctx.accounts.owner.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
        pool_state: ctx.accounts.pool_state.to_account_info(),
        owner_lp_token: ctx.accounts.owner_lp_token.to_account_info(),
        token_0_account: ctx.accounts.token_0_account.to_account_info(),
        token_1_account: ctx.accounts.token_1_account.to_account_info(),
        token_0_vault: ctx.accounts.token_0_vault.to_account_info(),
        token_1_vault: ctx.accounts.token_1_vault.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        token_program_2022: ctx.accounts.token_program_2022.to_account_info(),
        vault_0_mint: ctx.accounts.vault_0_mint.to_account_info(),
        vault_1_mint: ctx.accounts.vault_1_mint.to_account_info(),
        lp_mint: ctx.accounts.lp_mint.to_account_info(),
        memo_program: ctx.accounts.memo_program.to_account_info(),
    };
    let cpi_context = CpiContext::new(ctx.accounts.cp_swap_program.to_account_info(), cpi_accounts);
    let _= cpi::withdraw(cpi_context, lp_token_amount, minimum_token_0_amount, minimum_token_1_amount);

    ctx.accounts.token_0_account.reload()?;
    ctx.accounts.token_1_account.reload()?;

    if is_token0_wrapped_sol {
        post_wrap_sol_amount = ctx.accounts.token_0_account.amount;
        post_token_amount = ctx.accounts.token_1_account.amount;
    } else {
        post_wrap_sol_amount = ctx.accounts.token_1_account.amount;
        post_token_amount = ctx.accounts.token_0_account.amount;
    }

    let mut total_sol_received = post_wrap_sol_amount - pre_wrap_sol_amount;
    let total_token_received = post_token_amount - pre_token_amount;
    msg!("total_sol_received is {}", total_sol_received);
    msg!("total_token_received is {}", total_token_received);

    let mut sol_profilt = 0;

    if total_sol_received > pool_loan.init_sol_amount {
        total_sol_received = pool_loan.init_sol_amount;
        sol_profilt = total_sol_received - pool_loan.init_sol_amount;
    }

    // Send Wrapped Sol to the service vault after withdraw pool
    if is_token0_wrapped_sol{
        let _= transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.token_0_account.to_account_info(),
                    to: ctx.accounts.service_vault.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                    mint: ctx.accounts.vault_0_mint.to_account_info()
                }
            ),
            total_sol_received,
            ctx.accounts.vault_0_mint.decimals
        );
    } else {
        let _= transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.token_1_account.to_account_info(),
                    to: ctx.accounts.service_vault.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                    mint: ctx.accounts.vault_1_mint.to_account_info()
                }
            ),
            total_sol_received,
            ctx.accounts.vault_1_mint.decimals
        );
    }

    pool_loan.init_sol_amount = pool_loan.init_sol_amount.saturating_sub(total_sol_received + sol_profilt);
    pool_loan.init_token_amount =  pool_loan.init_token_amount.saturating_sub(total_token_received);

    msg!("Updated pool loan: init_sol_amount={}, init_token_amount={}", pool_loan.init_sol_amount, pool_loan.init_token_amount);
    msg!("{} tokens transferred to user", total_token_received);

    pool_loan.is_repaid = true;

    config.amount += total_sol_received;

    Ok(())
    
}