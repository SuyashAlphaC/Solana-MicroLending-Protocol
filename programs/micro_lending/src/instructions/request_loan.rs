use crate::error::*;
use crate::states::*;
use crate::utils::*;
use crate::{SEEDS_PLATFORM, SEEDS_USER};
use anchor_lang::prelude::*;
pub fn request_loan(
    ctx: Context<RequestLoan>,
    amount: u64,
    duration_days: u32,
    purpose: String,
    collateral_type: u8,
) -> Result<()> {
    let platform = &ctx.accounts.platform;
    let lending_pool = &ctx.accounts.lending_pool;
    let user_profile = &ctx.accounts.user_profile;
    let loan = &mut ctx.accounts.loan;
    let current = Clock::get()?.unix_timestamp;

    // Validate loan parameters
    require!(
        amount >= platform.min_loan_amount,
        MicroLendingError::LoanAmountTooLow
    );
    require!(
        amount <= platform.max_loan_amount,
        MicroLendingError::LoanAmountTooHigh
    );
    require!(
        duration_days as i64 <= lending_pool.max_loan_duration,
        MicroLendingError::LoanDurationTooLong
    );
    require!(
        purpose.len() <= 100,
        MicroLendingError::InvalidPoolConfiguration
    );
    require!(
        user_profile.active_loans == 0,
        MicroLendingError::BorrowerHasActiveLoan
    );

    // Check pool liquidity
    require!(
        lending_pool.available_liquidity >= amount,
        MicroLendingError::InsufficientLiquidity
    );

    // Calculate interest rate based on credit score and pool base rate
    let interest_rate = calculate_interest_rate(
        user_profile.credit_score,
        lending_pool.base_interest_rate,
        duration_days,
    )?;

    // Validate credit score for loan amount
    let required_credit_score = calculate_required_credit_score(amount, platform.max_loan_amount)?;
    require!(
        user_profile.credit_score >= required_credit_score,
        MicroLendingError::InsufficientCreditScore
    );

    // Initialize loan
    loan.borrower = ctx.accounts.borrower.key();
    loan.lender_pool = lending_pool.key();
    loan.amount = amount;
    loan.interest_rate = interest_rate;
    loan.duration_days = duration_days;
    loan.disbursed_at = 0;
    loan.due_date = 0;
    loan.amount_repaid = 0;
    loan.interest_accrued = 0;
    loan.status = LoanStatus::Requested;
    loan.purpose = purpose;
    loan.collateral_type = match collateral_type {
        0 => CollateralType::None,
        1 => CollateralType::Social,
        2 => CollateralType::Asset,
        3 => CollateralType::Income,
        4 => CollateralType::Group,
        _ => return Err(MicroLendingError::InvalidCollateralType.into()),
    };
    loan.collateral_value = 0;
    loan.payment_count = 0;
    loan.last_payment_date = 0;
    loan.grace_period_days = 7; // Default grace period
    loan.late_fee_rate = 500; // 5% late fee rate
    loan.created_at = current;
    loan.liquidated_at = None;

    msg!(
        "Loan requested: {} tokens for {} days",
        amount,
        duration_days
    );
    Ok(())
}

#[derive(Accounts)]
#[instruction(amount: u64, duration_days: u32, purpose: String, collateral_type: u8)]
pub struct RequestLoan<'info> {
    #[account(
        seeds = [SEEDS_PLATFORM],
        bump
    )]
    pub platform: Account<'info, Platform>,

    #[account(
        mut,
        seeds = [SEEDS_USER, borrower.key().as_ref()],
        bump
    )]
    pub user_profile: Account<'info, UserProfile>,

    #[account(
        constraint = lending_pool.is_active @ MicroLendingError::PoolNotActive
    )]
    pub lending_pool: Account<'info, LendingPool>,

    #[account(
        init,
        payer = borrower,
        space = 8 + Loan::INIT_SPACE,
        seeds = [b"loan", borrower.key().as_ref(), lending_pool.key().as_ref()],
        bump
    )]
    pub loan: Account<'info, Loan>,

    #[account(mut)]
    pub borrower: Signer<'info>,

    pub system_program: Program<'info, System>,
}
