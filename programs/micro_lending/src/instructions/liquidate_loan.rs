use crate::error::*;
use crate::states::*;
use crate::utils::*;
use crate::{SEEDS_PLATFORM, SEEDS_USER};
use anchor_lang::prelude::*;

pub fn liquidate_loan(ctx: Context<LiquidateLoan>) -> Result<()> {
    let loan = &mut ctx.accounts.loan;
    let user_profile = &mut ctx.accounts.user_profile;
    let platform = &mut ctx.accounts.platform;
    let lending_pool = &mut ctx.accounts.lending_pool;
    let current = Clock::get()?.unix_timestamp;

    // Validate that the loan is in a state that can be liquidated
    require!(
        loan.status == LoanStatus::Disbursed || loan.status == LoanStatus::Active,
        MicroLendingError::InvalidLoanState
    );

    require!(
        is_loan_overdue(loan.due_date, current, loan.grace_period_days),
        MicroLendingError::LoanNotYetDueForLiquidation
    );

    // Update loan status
    loan.status = LoanStatus::Liquidated;
    loan.liquidated_at = Some(current);

    // Update user profile for the defaulted loan
    user_profile.active_loans = user_profile.active_loans.saturating_sub(1);
    user_profile.defaulted_loans = user_profile.defaulted_loans.checked_add(1).unwrap();
    user_profile.last_updated = current;

    // Update platform-wide statistics for defaults
    platform.total_defaults = platform.total_defaults.checked_add(1).unwrap();

    // Update lending pool statistics
    lending_pool.active_loans = lending_pool.active_loans.saturating_sub(1);
    let outstanding_amount = loan.amount.checked_sub(loan.amount_repaid).unwrap();
    lending_pool.total_borrowed = lending_pool
        .total_borrowed
        .saturating_sub(outstanding_amount);

    msg!("Loan for borrower {} has been liquidated.", loan.borrower);
    Ok(())
}

#[derive(Accounts)]
pub struct LiquidateLoan<'info> {
    #[account(mut)]
    pub liquidator: Signer<'info>,

    #[account(
        mut,
        seeds = [SEEDS_PLATFORM],
        bump
    )]
    pub platform: Account<'info, Platform>,

    #[account(
        mut,
        seeds = [b"loan", loan.borrower.as_ref(), lending_pool.key().as_ref()],
        bump
    )]
    pub loan: Account<'info, Loan>,

    #[account(mut)]
    pub lending_pool: Account<'info, LendingPool>,

    #[account(
        mut,
        seeds = [SEEDS_USER, loan.borrower.as_ref()],
        bump
    )]
    pub user_profile: Account<'info, UserProfile>,
}
