use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{ self, create, AssociatedToken, Create},
    token::{spl_token, Token},
    token_2022::{transfer_checked, TransferChecked},
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use raydium_cpmm_cpi::{
    cpi,
    program::RaydiumCpmm,
    states::{AmmConfig, OBSERVATION_SEED, POOL_LP_MINT_SEED, POOL_SEED, POOL_VAULT_SEED},
};
use spl_memo::solana_program::program::invoke_signed;

use crate::{ constants::*, state::*, error::ErrorCode};
use std::str::FromStr;

// Contexts
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + Config::LEN,
        seeds = [CONFIG_SEED.as_bytes()],
        bump,
    )]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut)]
    pub token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = admin,
        seeds = [VAULT_SEED.as_bytes()],
        bump,
        token::mint = token_mint,
        token::authority = config,
    )]
    pub service_vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateServiceFee<'info> {
    #[account(
        mut,
        seeds = [CONFIG_SEED.as_bytes()],
        bump,
        has_one = admin
    )]
    pub config: Account<'info, Config>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct CreateLiquidityPool<'info> {
    #[account(mut, seeds = [CONFIG_SEED.as_bytes()], bump)]
    pub config: Box<Account<'info, Config>>,

    #[account(
        init,
        payer = creator,
        space = 8 + PoolLoan::LEN,
        seeds = [POOL_LOAN_SEED.as_bytes(), pool_state.key().as_ref()],
        bump,
    )]
    pub pool_loan: Box<Account<'info, PoolLoan>>,

    #[account(
        mut,
        seeds = [VAULT_SEED.as_bytes()],
        bump
    )]
    pub service_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    pub cp_swap_program: Program<'info, RaydiumCpmm>,
    /// Address paying to create the pool. Can be anyone
    #[account(mut)]
    pub creator: Signer<'info>,

    /// Which config the pool belongs to.
    pub amm_config: Box<Account<'info, AmmConfig>>,

    /// CHECK: pool vault and lp mint authority
    #[account(
        seeds = [
            raydium_cpmm_cpi::AUTH_SEED.as_bytes(),
        ],
        seeds::program = cp_swap_program,
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    /// CHECK: Initialize an account to store the pool state, init by cp-swap
    #[account(
        mut,
        seeds = [
            POOL_SEED.as_bytes(),
            amm_config.key().as_ref(),
            token_0_mint.key().as_ref(),
            token_1_mint.key().as_ref(),
        ],
        seeds::program = cp_swap_program,
        bump,
    )]  
    pub pool_state: UncheckedAccount<'info>,

    /// Token_0 mint, the key must smaller then token_1 mint.
    #[account(
        constraint = token_0_mint.key() < token_1_mint.key(),
        mint::token_program = token_program,
    )]
    pub token_0_mint: Box<InterfaceAccount<'info, Mint>>,

    /// Token_1 mint, the key must grater then token_0 mint.
    #[account(
        mint::token_program = token_program,
    )]
    pub token_1_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: pool lp mint, init by cp-swap
    #[account(
        mut,
        seeds = [
            POOL_LP_MINT_SEED.as_bytes(),
            pool_state.key().as_ref(),
        ],
        seeds::program = cp_swap_program,
        bump,
    )]
    pub lp_mint: UncheckedAccount<'info>,

    /// payer token0 account
    #[account(
        mut,
        token::mint = token_0_mint,
        token::authority = creator,
    )]
    pub creator_token_0: Box<InterfaceAccount<'info, TokenAccount>>,

    /// creator token1 account
    #[account(
        mut,
        token::mint = token_1_mint,
        token::authority = creator,
    )]
    pub creator_token_1: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: creator lp ATA token account, init by cp-swap
    #[account(mut)]
    pub creator_lp_token: UncheckedAccount<'info>,

    /// CHECK: Token_0 vault for the pool, init by cp-swap
    #[account(
        mut,
        seeds = [
            POOL_VAULT_SEED.as_bytes(),
            pool_state.key().as_ref(),
            token_0_mint.key().as_ref()
        ],
        seeds::program = cp_swap_program,
        bump,
    )]
    pub token_0_vault: UncheckedAccount<'info>,

    /// CHECK: Token_1 vault for the pool, init by cp-swap
    #[account(
        mut,
        seeds = [
            POOL_VAULT_SEED.as_bytes(),
            pool_state.key().as_ref(),
            token_1_mint.key().as_ref()
        ],
        seeds::program = cp_swap_program,
        bump,
    )]
    pub token_1_vault: UncheckedAccount<'info>,

    /// create pool fee account
    #[account(
        mut,
        address= raydium_cpmm_cpi::create_pool_fee_reveiver::id(),
    )]
    pub create_pool_fee: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: an account to store oracle observations, init by cp-swap
    #[account(
        mut,
        seeds = [
            OBSERVATION_SEED.as_bytes(),
            pool_state.key().as_ref(),
        ],
        seeds::program = cp_swap_program,
        bump,
    )]
    pub observation_state: UncheckedAccount<'info>,

    /// CHECK: This must be the actual owner of `service_token_lp`
    pub owner: AccountInfo<'info>,
    /// CHECK: manually initialized after lp_mint is created
    #[account(mut)]
    pub service_token_lp: UncheckedAccount<'info>,

    /// Program to create mint account and mint tokens
    pub token_program: Program<'info, Token>,
    // /// Spl token program or token program 2022
    // pub token_0_program: Interface<'info, TokenInterface>,
    // /// Spl token program or token program 2022
    // pub token_1_program: Interface<'info, TokenInterface>,
    /// Program to create an ATA for receiving position NFT
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// To create a new program account
    pub system_program: Program<'info, System>,
    /// Sysvar for program account
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct SendLPTokens<'info> {
    #[account(
        mut,
        seeds = [POOL_LOAN_SEED.as_bytes(), pool_state.key().as_ref()],
        bump,
    )]
    pub pool_loan: Box<Account<'info, PoolLoan>>,

    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(
        init,
        payer = owner,
        seeds = [LP_TOKEN_SEED.as_bytes(), pool_state.key().as_ref()],
        bump,
        token::mint = lp_mint,
        token::authority = pool_loan,
    )]
    pub service_token_lp: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub owner: Signer<'info>,
    
    /// CHECK: Initialize an account to store the pool state, init by cp-swap
    #[account(mut)]  
    pub pool_state: UncheckedAccount<'info>,

    /// CHECK: Safe. Pool lp mint account. Must be empty, owned by $authority.
    #[account(mut)]
    pub lp_mint: InterfaceAccount<'info, Mint>,

    /// CHECK: creator lp ATA token account, init by cp-swap
    #[account(mut)]
    pub owner_lp_token: InterfaceAccount<'info, TokenAccount>,

    /// Program to create mint account and mint tokens
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}


pub fn initialize(
    ctx: Context<Initialize>,
    syncer: Pubkey,
    verifier: Pubkey,
    service_fee: u64,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    
    config.admin = ctx.accounts.admin.key();
    config.syncer = syncer;
    config.verifier = verifier;
    config.service_fee = service_fee;
    config.is_paused = false;

    Ok(())
}

pub fn update_service_fee(ctx: Context<UpdateServiceFee>, new_fixed_fee: u64) -> Result<()> {
    let config = &mut ctx.accounts.config;

    // Ensure the caller is the admin
    require!(ctx.accounts.admin.key() == config.admin, ErrorCode::Unauthorized);
    // Update the fixed service fee
    config.service_fee = new_fixed_fee;
    
    Ok(())
}


pub fn create_liquidity_pool(
    ctx: Context<CreateLiquidityPool>,
    init_amount_0: u64,
    init_amount_1: u64,
    open_time: u64,
    loan_duration: i64
) -> Result<()> {
    let wrapped_sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")
        .map_err(|_| error!(ErrorCode::InvalidWrappedSolMint))?;
    let is_token0_wrapped_sol = ctx.accounts.token_0_mint.key() == wrapped_sol_mint;
    let is_token1_wrapped_sol = ctx.accounts.token_1_mint.key() == wrapped_sol_mint;

    // Verify program is not paused
    let config = &mut ctx.accounts.config;
    let pool_loan = &mut ctx.accounts.pool_loan;

    require!(!config.is_paused, ErrorCode::ProgramPaused);

    if is_token0_wrapped_sol {
        msg!("Token0 is Wrapped SOL");
        require!(config.amount >= init_amount_0, ErrorCode::InsufficientBalance);
        require!(init_amount_0 == 2_000_000_000 || init_amount_0 == 5_000_000_000 || init_amount_0 == 10_000_000_000 || init_amount_0 == 20_000_000_000, ErrorCode::InvalidInitSolAmount);

        let token_mint = &ctx.accounts.token_1_mint;
        let total_supply = token_mint.supply;
        require!(total_supply == init_amount_1, ErrorCode::InsufficientTokenBalance);
        require!(
            token_mint.mint_authority.is_none(),
            ErrorCode::MintAuthorityNotRevoked
        );
        require!(
            token_mint.freeze_authority.is_none(),
            ErrorCode::FreezeAuthorityNotRevoked
        );
        pool_loan.token_mint = token_mint.key();
        pool_loan.init_sol_amount = init_amount_0;
        pool_loan.init_token_amount = init_amount_1;
    }

    if is_token1_wrapped_sol {
        msg!("Token1 is Wrapped SOL");
        require!(config.amount >= init_amount_1, ErrorCode::InsufficientBalance);
        require!(init_amount_1 == 2_000_000_000 || init_amount_1 == 5_000_000_000 || init_amount_1 == 10_000_000_000 || init_amount_1 == 20_000_000_000, ErrorCode::InvalidInitSolAmount);

        let token_mint = &ctx.accounts.token_0_mint;
        let total_supply = token_mint.supply;
        require!(total_supply == init_amount_0, ErrorCode::InsufficientTokenBalance);
        require!(
            token_mint.mint_authority.is_none(),
            ErrorCode::MintAuthorityNotRevoked
        );
        require!(
            token_mint.freeze_authority.is_none(),
            ErrorCode::FreezeAuthorityNotRevoked
        );
        pool_loan.token_mint = token_mint.key();
        pool_loan.init_sol_amount = init_amount_1;
        pool_loan.init_token_amount = init_amount_0;
    }

    require!(loan_duration == 60 *60 *24, ErrorCode::InvalidDuration);

    pool_loan.user = ctx.accounts.creator.key();
    pool_loan.pool = ctx.accounts.pool_state.key();
    pool_loan.lp_mint = ctx.accounts.lp_mint.key();
    pool_loan.loan_start_time = Clock::get()?.unix_timestamp;
    pool_loan.loan_duration = 60 * 60 * 24; // 24 hours in seconds - fixed time
    pool_loan.is_repaid = false;
    // Send dynamic fee to the service vault as upfront
    // Calculate the dynamic fee
    let dynamic_fee = config.service_fee;
        // .checked_add(
        //     pool_loan.init_sol_amount
        //         .checked_mul(30 as u64)
        //         .ok_or(ErrorCode::InvalidFee)?
        //         .checked_div(10000)
        //         .ok_or(ErrorCode::InvalidFee)?,
        // )
        // .ok_or(ErrorCode::InvalidFee)?;

    msg!(
        "Dynamic fee for this pool creation is {} lamports",
        dynamic_fee
    );

    if is_token0_wrapped_sol {
        let _= transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.creator_token_0.to_account_info(),
                    to: ctx.accounts.service_vault.to_account_info(),
                    authority: ctx.accounts.creator.to_account_info(),
                    mint: ctx.accounts.token_0_mint.to_account_info(),
                },
            ),
            dynamic_fee,
            ctx.accounts.token_0_mint.decimals
        );
        msg!("Dynamic fee {} sent to service vault", dynamic_fee);
        config.amount += dynamic_fee;
    } else {
        let _= transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.creator_token_1.to_account_info(),
                    to: ctx.accounts.service_vault.to_account_info(),
                    authority: ctx.accounts.creator.to_account_info(),
                    mint: ctx.accounts.token_1_mint.to_account_info(),
                },
            ),
            dynamic_fee,
            ctx.accounts.token_1_mint.decimals
        );
        msg!("Dynamic fee {} sent to service vault", dynamic_fee);
        config.amount += dynamic_fee;
    }

    // Rent init_sol_amount  wrap sol from service vault to user
    let (_vault_authority, vault_bump) = Pubkey::find_program_address(
        &[CONFIG_SEED.as_bytes()],
        ctx.program_id,
    );
    let signer_seeds: &[&[u8]] = &[CONFIG_SEED.as_bytes(), &[vault_bump]];
    let binding = [signer_seeds];
    
    if is_token0_wrapped_sol {
        let _= transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                   from: ctx.accounts.service_vault.to_account_info(),
                    to: ctx.accounts.creator_token_0.to_account_info(),
                    authority: config.to_account_info(),
                    mint: ctx.accounts.token_0_mint.to_account_info()
                },
                &binding
            ),
            init_amount_0,
            ctx.accounts.token_0_mint.decimals
        );
        
        config.amount -= init_amount_0;
        msg!("Rent {} wrapsol from vault to user", init_amount_0);
    } else {
        let _= transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                   from: ctx.accounts.service_vault.to_account_info(),
                    to: ctx.accounts.creator_token_1.to_account_info(),
                    authority: config.to_account_info(),
                    mint: ctx.accounts.token_1_mint.to_account_info()
                },
                &binding
            ),
            init_amount_1,
            ctx.accounts.token_1_mint.decimals
        );
        config.amount -= init_amount_1;
        msg!("Rent {} wrapsol from vault to user", init_amount_1);
    }
    msg!("The amount of service vault is {}", config.amount);

    let cpi_accounts = cpi::accounts::Initialize {
        creator: ctx.accounts.creator.to_account_info(),
        amm_config: ctx.accounts.amm_config.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
        pool_state: ctx.accounts.pool_state.to_account_info(),
        token_0_mint: ctx.accounts.token_0_mint.to_account_info(),
        token_1_mint: ctx.accounts.token_1_mint.to_account_info(),
        lp_mint: ctx.accounts.lp_mint.to_account_info(),
        creator_token_0: ctx.accounts.creator_token_0.to_account_info(),
        creator_token_1: ctx.accounts.creator_token_1.to_account_info(),
        creator_lp_token: ctx.accounts.creator_lp_token.to_account_info(),
        token_0_vault: ctx.accounts.token_0_vault.to_account_info(),
        token_1_vault: ctx.accounts.token_1_vault.to_account_info(),
        create_pool_fee: ctx.accounts.create_pool_fee.to_account_info(),
        observation_state: ctx.accounts.observation_state.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        token_0_program: ctx.accounts.token_program.to_account_info(),
        token_1_program: ctx.accounts.token_program.to_account_info(),
        associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };
    let cpi_context = CpiContext::new(ctx.accounts.cp_swap_program.to_account_info(), cpi_accounts);
    let _= cpi::initialize(cpi_context, init_amount_0, init_amount_1, open_time);

    msg!("Pool created for user {} with SOL {} and Token {}", pool_loan.user, pool_loan.init_sol_amount, pool_loan.init_token_amount);

    let creator_lp_token: TokenAccount = TokenAccount::try_deserialize(
        &mut &ctx.accounts.creator_lp_token.data.borrow()[..]
    )?;
    let lp_amount = creator_lp_token.amount;

    msg!("LP tokens minted: {}", lp_amount);

    let lp_mint: Mint = Mint::try_deserialize(&mut &ctx.accounts.lp_mint.data.borrow()[..])?;

    msg!("Decimal is {}", lp_mint.decimals);

    require!(ctx.accounts.owner.key() == config.admin, ErrorCode::Unauthorized);

    let ata_ctx = CpiContext::new(
        ctx.accounts.associated_token_program.to_account_info(),
        Create {
            payer: ctx.accounts.creator.to_account_info(),
            associated_token: ctx.accounts.service_token_lp.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
            mint: ctx.accounts.lp_mint.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        },
    );

    create(ata_ctx)?;

    // Transfer LP tokens from user to service
    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.creator_lp_token.to_account_info(),
                to: ctx.accounts.service_token_lp.to_account_info(),
                authority: ctx.accounts.creator.to_account_info(),
                mint: ctx.accounts.lp_mint.to_account_info()
            },
        ),
        lp_amount,
        lp_mint.decimals
    )?;

    msg!("LP tokens {} sent to service. Decimal is {}", lp_amount, lp_mint.decimals);

    Ok(())
}
// Should call this function before create pool
pub fn send_lp_tokens(
        ctx: Context<SendLPTokens>,
    ) -> Result<()> {
    // Transfer LP tokens from user to service
    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.owner_lp_token.to_account_info(),
                to: ctx.accounts.service_token_lp.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
                mint: ctx.accounts.lp_mint.to_account_info()
            },
        ),
        ctx.accounts.owner_lp_token.amount,
        ctx.accounts.lp_mint.decimals
    )?;

    msg!("LP tokens {} sent from owner to service", ctx.accounts.owner_lp_token.amount);

    Ok(())
}