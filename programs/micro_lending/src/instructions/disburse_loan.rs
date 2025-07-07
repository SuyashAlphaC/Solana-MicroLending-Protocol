use crate::error::*;
use crate::states::*;
use crate::SEEDS_USER;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};
use crate::{SEEDS_PLATFORM};
pub fn disburse_loan(ctx: Context<DisburseLoan>) -> Result<()> {


    let platform = &mut ctx.accounts.platform;

    
    let loan = &mut ctx.accounts.loan;
    let lending_pool = &mut ctx.accounts.lending_pool;
    let user_profile = &mut ctx.accounts.user_profile;
    let current = Clock::get()?.unix_timestamp;
    // Validate loan state
    require!(
        loan.status == LoanStatus::Approved,
        MicroLendingError::InvalidLoanState
    );

    // Calculate due date
    let due_date = current + (loan.duration_days as i64 * 86400);

    // Transfer tokens from pool to borrower
    let pool_key = lending_pool.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"pool_token_account",
        pool_key.as_ref(),
        &[ctx.bumps.pool_token_account],
    ]];

    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.pool_token_account.to_account_info(),
        to: ctx.accounts.borrower_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        authority: ctx.accounts.pool_token_account.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();

    let cpi_ctx = CpiContext::new_with_signer(cpi_program, transfer_cpi_accounts, signer_seeds);
    let decimals = ctx.accounts.mint.decimals;
    transfer_checked(cpi_ctx, loan.amount, decimals);

    // Update loan
    loan.status = LoanStatus::Disbursed;
    loan.disbursed_at = current;
    loan.due_date = due_date;

    // Update user profile
    user_profile.active_loans = user_profile.active_loans.checked_add(1).unwrap();
    user_profile.total_borrowed = user_profile
        .total_borrowed
        .checked_add(loan.amount)
        .unwrap();
    // Update lending pool
    lending_pool.total_borrowed = lending_pool
        .total_borrowed
        .checked_add(loan.amount)
        .unwrap();

    //Update Platform
    platform.total_loans_issued += 1;
    platform.total_volume = platform.total_volume.checked_add(loan.amount).unwrap();
    msg!(
        "Loan disbursed: {} tokens to borrower: {}",
        loan.amount,
        loan.borrower
    );
    Ok(())
}

#[derive(Accounts)]
pub struct DisburseLoan<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut, 
        seeds = [SEEDS_PLATFORM],
        bump
    )]
    pub platform : Account<'info, Platform>,

    #[account(
        mut,
        seeds = [b"loan", loan.borrower.as_ref(), lending_pool.key().as_ref()],
        bump
    )]
    pub loan: Account<'info, Loan>,
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        constraint = lending_pool.is_active @ MicroLendingError::PoolNotActive,
        seeds = [b"lending_pool", authority.key().as_ref(), mint.key().as_ref()],
        bump
    )]
    pub lending_pool: Account<'info, LendingPool>,

    #[account(
        mut,
        seeds = [SEEDS_USER, loan.borrower.as_ref()],
        bump
    )]
    pub user_profile: Account<'info, UserProfile>,

    #[account(
        mut,
        constraint = pool_token_account.key() == lending_pool.token_account,
        seeds = [b"pool_token_account", lending_pool.key().as_ref()],
        bump

    )]
    pub pool_token_account: InterfaceAccount<'info, TokenAccount>,

     /// CHECK: The borrower's account key must match loan.borrower for security.
     #[account(
        constraint = borrower.key() == loan.borrower @ MicroLendingError::InvalidBorrowerAccount
    )]
    pub borrower : AccountInfo<'info>,
    #[account(
        init_if_needed,
        payer = authority, // The authority pays for the creation of this account if it doesn't exist
        associated_token::mint = mint,
        associated_token::authority = borrower,
        associated_token::token_program = token_program,
        // constraint = borrower_token_account.owner == loan.borrower // Can keep or remove
    )]
    pub borrower_token_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
