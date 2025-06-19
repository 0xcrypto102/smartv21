use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{ constants::*, state::*, error::ErrorCode};

// Deposit Wrap SOL into the service vault account
#[derive(Accounts)]
pub struct ManageServiceVault<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED.as_bytes()],
        bump,
    )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [VAULT_SEED.as_bytes()],
        bump
    )]
    pub service_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub admin_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

// Deposit wrap sol into the service vault
pub fn deposit(ctx: Context<ManageServiceVault>, amount: u64) -> Result<()> {
    // Transfer Wrap SOL tokens from user to service
    anchor_spl::token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.admin_token_account.to_account_info(),
                to: ctx.accounts.service_vault.to_account_info(),
                authority: ctx.accounts.admin.to_account_info(),
            },
        ),
        amount,
    )?;
    msg!("Transfer Wrap SOL tokens from user to service {}", amount);

    let config = &mut ctx.accounts.config;
    config.amount += amount;
    msg!("The wrap sol token amount of service vault is {}", config.amount);

    Ok(())
}

// Withdraw wrap sol from service vault by owner anytime
pub fn withdraw(ctx: Context<ManageServiceVault>, amount: u64) -> Result<()> {
    let accts = &ctx.accounts;

    // ✅ Authorization check
    require!(accts.admin.key() == accts.config.admin, ErrorCode::Unauthorized);

    // ✅ Prepare seeds for vault authority (PDA signer)
    let (_vault_authority, vault_bump) = Pubkey::find_program_address(
        &[CONFIG_SEED.as_bytes()],
        ctx.program_id,
    );
    let signer_seeds: &[&[u8]] = &[CONFIG_SEED.as_bytes(), &[vault_bump]];
    let binding = [signer_seeds];
    
    // ✅ Transfer tokens from vault to admin
    let cpi_ctx = CpiContext::new_with_signer(
        accts.token_program.to_account_info(),
        anchor_spl::token::Transfer {
            from: accts.service_vault.to_account_info(),
            to: accts.admin_token_account.to_account_info(),
            authority: accts.config.to_account_info(), // Config is the vault authority PDA
        },
        &binding,
    );

    anchor_spl::token::transfer(cpi_ctx, amount)?;
    msg!("Withdraw {} tokens from service vault to admin", amount);

    // ✅ Update vault amount
    let config = &mut ctx.accounts.config;
    config.amount = config.amount.saturating_sub(amount);
    msg!("Updated vault balance: {}", config.amount);

    Ok(())
}
