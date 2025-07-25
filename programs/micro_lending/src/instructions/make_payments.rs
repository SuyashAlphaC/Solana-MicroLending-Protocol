use crate::error::*;
use crate::states::*;
use crate::utils::*;
use crate::SEEDS_PLATFORM;
use crate::SEEDS_USER;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

pub fn make_payment(ctx: Context<MakePayment>, payment_amount: u64) -> Result<()> {
    let loan = &mut ctx.accounts.loan;
    let lending_pool = &mut ctx.accounts.lending_pool;
    let user_profile = &mut ctx.accounts.user_profile;
    let platform = &ctx.accounts.platform;
    let current = Clock::get()?.unix_timestamp;
    msg!("Borrower paying amount: {}", payment_amount);
    // Validate loan state
    require!(
        loan.status == LoanStatus::Disbursed || loan.status == LoanStatus::Active,
        MicroLendingError::InvalidLoanState
    );

    // Calculate interest accrued
    let days_elapsed = days_between(loan.disbursed_at, current);
    let interest_accrued = calculate_simple_interest(
        loan.amount - loan.amount_repaid,
        loan.interest_rate,
        days_elapsed,
    )?;

    loan.interest_accrued = interest_accrued;
    msg!("Interest Accrued : {}", interest_accrued);
    // Calculate total amount owed
    let total_owed = loan
        .amount
        .checked_add(interest_accrued)
        .unwrap()
        .checked_sub(loan.amount_repaid)
        .unwrap();

    // Check if loan is overdue and calculate late fees
    let mut late_fee = 0u64;
    if is_loan_overdue(loan.due_date, current, loan.grace_period_days) {
        let days_overdue = days_between(loan.due_date, current);
        late_fee = calculate_late_fee(total_owed, loan.late_fee_rate, days_overdue)?;
    }

    let total_amount_due = total_owed.checked_add(late_fee).unwrap();
    msg!("Total amount due before repayment : {}", total_amount_due);
    // Validate payment amount
    require!(payment_amount > 0, MicroLendingError::InvalidPaymentAmount);
    require!(
        payment_amount <= total_amount_due,
        MicroLendingError::InvalidPaymentAmount
    );

    // Calculate platform fee
    let platform_fee = (payment_amount as u128 * platform.platform_fee as u128 / 10000) as u64;
    msg!("Platform fee charged : {}", platform_fee);
    let net_payment = payment_amount.checked_sub(platform_fee).unwrap();
    msg!("Net Payment after platform fee : {}", net_payment);
    // Transfer payment from borrower to pool
    let transfer_to_pool = TransferChecked {
        from: ctx.accounts.borrower_token_account.to_account_info(),
        to: ctx.accounts.pool_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        authority: ctx.accounts.borrower.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, transfer_to_pool);
    let decimal = ctx.accounts.mint.decimals;
    transfer_checked(cpi_ctx, net_payment, decimal)?;

    // Transfer platform fee to treasury if applicable
    let transfer_to_treasury = TransferChecked {
        from: ctx.accounts.borrower_token_account.to_account_info(),
        to: ctx.accounts.treasury_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        authority: ctx.accounts.borrower.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();

    let cpi_ctx = CpiContext::new(cpi_program, transfer_to_treasury);

    transfer_checked(cpi_ctx, platform_fee, decimal)?;

    let net_deduction_in_borrowed_amount = net_payment.checked_sub(interest_accrued).unwrap();
    msg!(
        "Net deduction in borrowed amount after interest : {}",
        net_deduction_in_borrowed_amount
    );
    // Update loan
    loan.amount_repaid = loan
        .amount_repaid
        .checked_add(net_deduction_in_borrowed_amount)
        .unwrap();
    loan.payment_count = loan.payment_count.checked_add(1).unwrap();
    loan.last_payment_date = current;

    // Check if loan is fully repaid
    if loan.amount_repaid >= loan.amount {
        loan.status = LoanStatus::Repaid;

        // Update user profile
        user_profile.active_loans = user_profile.active_loans.saturating_sub(1);
        user_profile.successful_loans = user_profile.successful_loans.checked_add(1).unwrap();

        user_profile.total_repaid = user_profile.total_repaid.checked_add(net_payment).unwrap();
        // Update lending pool
        lending_pool.active_loans = lending_pool.active_loans.saturating_sub(1);
        msg!(
            "Available liquidity before repayment : {}",
            lending_pool.available_liquidity
        );
        lending_pool.available_liquidity = lending_pool
            .available_liquidity
            .checked_add(net_payment)
            .unwrap();

        lending_pool.total_borrowed = lending_pool
            .total_borrowed
            .checked_sub(net_deduction_in_borrowed_amount)
            .unwrap();

        msg!(
            "Available Liquidity in the pool : {}",
            lending_pool.available_liquidity
        );

        msg!("Total Borrwed reduced to : {}", lending_pool.total_borrowed);

        if interest_accrued > 0 && lending_pool.total_shares > 0 {
            // Calculate interest per share (scaled by 1e9 for precision)
            let interest_per_share_increase =
                (interest_accrued as u128 * 1_000_000_000) / lending_pool.total_shares as u128;

            lending_pool.interest_per_share = lending_pool
                .interest_per_share
                .checked_add(interest_per_share_increase as u64)
                .unwrap();
        }

        lending_pool.total_interest_earned = lending_pool
            .total_interest_earned
            .checked_add(interest_accrued)
            .unwrap();

        msg!("Loan fully repaid by borrower: {}", loan.borrower);
    } else {
        loan.status = LoanStatus::Active;

        // Partial payment - update available liquidity
        msg!(
            "Available liquidity before repayment : {}",
            lending_pool.available_liquidity
        );

        lending_pool.total_borrowed = lending_pool
            .total_borrowed
            .checked_sub(net_deduction_in_borrowed_amount)
            .unwrap();

        msg!(
            "Total principal borrowed remains : {}",
            lending_pool.total_borrowed
        );

        lending_pool.available_liquidity = lending_pool
            .available_liquidity
            .checked_add(net_payment)
            .unwrap();

        msg!(
            "Partial payment made by borrower: {} amount: {}",
            loan.borrower,
            payment_amount
        );

        if interest_accrued > 0 && lending_pool.total_shares > 0 {
            // Calculate interest per share (scaled by 1e9 for precision)
            let interest_per_share_increase =
                (interest_accrued as u128 * 1_000_000_000) / lending_pool.total_shares as u128;

            lending_pool.interest_per_share = lending_pool
                .interest_per_share
                .checked_add(interest_per_share_increase as u64)
                .unwrap();
        }

        lending_pool.total_interest_earned = lending_pool
            .total_interest_earned
            .checked_add(interest_accrued)
            .unwrap();
    }

    Ok(())
}

#[derive(Accounts)]
pub struct MakePayment<'info> {
    #[account(
        seeds = [SEEDS_PLATFORM],
        bump
    )]
    pub platform: Account<'info, Platform>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"loan", borrower.key().as_ref(), lending_pool.key().as_ref()],
        bump
    )]
    pub loan: Account<'info, Loan>,

    #[account(
        mut,
        constraint = lending_pool.is_active @ MicroLendingError::PoolNotActive
    )]
    pub lending_pool: Account<'info, LendingPool>,

    #[account(
        mut,
        seeds = [SEEDS_USER, borrower.key().as_ref()],
        bump
    )]
    pub user_profile: Account<'info, UserProfile>,

    #[account(
        mut,
        constraint = pool_token_account.key() == lending_pool.token_account
    )]
    pub pool_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = borrower,
        associated_token::token_program = token_program,
        constraint = borrower_token_account.owner == borrower.key()
    )]
    pub borrower_token_account: InterfaceAccount<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(
        init_if_needed,
        payer = borrower,
        associated_token::mint = mint,
        associated_token::authority = platform,
        associated_token::token_program = token_program,
    )]
    pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub borrower: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}
