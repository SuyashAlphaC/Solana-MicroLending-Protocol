use crate::error::*;
use crate::states::*;
use anchor_lang::prelude::*;

pub fn approve_loan(ctx: Context<ApproveLoan>) -> Result<()> {
    let loan = &mut ctx.accounts.loan;
    let lending_pool = &mut ctx.accounts.lending_pool;

    // Validate loan state
    require!(
        loan.status == LoanStatus::Requested,
        MicroLendingError::InvalidLoanState
    );

    // Check pool liquidity
    require!(
        lending_pool.available_liquidity >= loan.amount,
        MicroLendingError::InsufficientLiquidity
    );

    // Update loan status
    loan.status = LoanStatus::Approved;

    // Reserve liquidity in the pool
    lending_pool.available_liquidity = lending_pool
        .available_liquidity
        .checked_sub(loan.amount)
        .ok_or(MicroLendingError::InsufficientLiquidity)?;

    lending_pool.active_loans = lending_pool.active_loans.checked_add(1).unwrap();
    msg!("Loan approved for borrower: {}", loan.borrower);
    Ok(())
}

#[derive(Accounts)]
pub struct ApproveLoan<'info> {
    #[account(
        mut,
        seeds = [b"loan", loan.borrower.as_ref(), lending_pool.key().as_ref()],
        bump
    )]
    pub loan: Account<'info, Loan>,

    #[account(
        mut,
        has_one = authority,
        constraint = lending_pool.is_active @ MicroLendingError::PoolNotActive
    )]
    pub lending_pool: Account<'info, LendingPool>,

    pub authority: Signer<'info>,
}
