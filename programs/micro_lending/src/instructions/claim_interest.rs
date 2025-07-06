use crate::error::*;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
pub struct ClaimInterest<'info> {
    #[account(mut)]
    pub lender: Signer<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        constraint = lending_pool.is_active @ MicroLendingError::PoolNotActive
    )]
    pub lending_pool: Account<'info, LendingPool>,

    #[account(
        mut,
        seeds = [b"lender_deposit", lender.key().as_ref(), lending_pool.key().as_ref()],
        bump,
        constraint = lender_deposit.lender == lender.key()
    )]
    pub lender_deposit: Account<'info, LenderDeposit>,

    #[account(
        mut,
        constraint = pool_token_account.key() == lending_pool.token_account,
        seeds = [b"pool_token_account", lending_pool.key().as_ref()],
        bump
    )]
    pub pool_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = lender,
        associated_token::token_program = token_program,
    )]
    pub lender_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn claim_interest(ctx: Context<ClaimInterest>) -> Result<()> {
    let lending_pool = &mut ctx.accounts.lending_pool;
    let lender_deposit = &mut ctx.accounts.lender_deposit;

    // Use the helper function to calculate unclaimed interest
    let unclaimed_interest = get_unclaimed_interest(lender_deposit, lending_pool);

    require!(unclaimed_interest > 0, MicroLendingError::NoInterestToClaim);

    // Calculate total interest earned for updating the record
    let total_interest_earned =
        (lender_deposit.shares as u128 * lending_pool.interest_per_share as u128) / 1_000_000_000;

    // Transfer interest from pool to lender
    let pool_key = lending_pool.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"pool_token_account",
        pool_key.as_ref(),
        &[ctx.bumps.pool_token_account],
    ]];

    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.pool_token_account.to_account_info(),
        to: ctx.accounts.lender_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        authority: ctx.accounts.pool_token_account.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, transfer_cpi_accounts, signer_seeds);
    let decimals = ctx.accounts.mint.decimals;

    transfer_checked(cpi_ctx, unclaimed_interest, decimals)?;

    // Update records
    lender_deposit.interest_debt = total_interest_earned as u64; // Update debt to prevent double-claiming
    lender_deposit.interest_earned = total_interest_earned as u64; // Track total lifetime earnings
    lender_deposit.interest_claimed = lender_deposit
        .interest_claimed
        .checked_add(unclaimed_interest)
        .unwrap(); // Track total claimed

    lending_pool.total_interest_distributed = lending_pool
        .total_interest_distributed
        .checked_add(unclaimed_interest)
        .unwrap();

    lending_pool.available_liquidity = lending_pool
        .available_liquidity
        .saturating_sub(unclaimed_interest);

    msg!(
        "Interest claimed: {} tokens by lender: {}",
        unclaimed_interest,
        ctx.accounts.lender.key()
    );
    Ok(())
}

pub fn calculate_pending_interest(
    lender_shares: u64,
    interest_per_share: u64,
    interest_debt: u64,
) -> u64 {
    let total_earned = (lender_shares as u128 * interest_per_share as u128) / 1_000_000_000;
    total_earned.saturating_sub(interest_debt as u128) as u64
}

// Helper function to get lender's total unclaimed interest
pub fn get_unclaimed_interest(lender_deposit: &LenderDeposit, lending_pool: &LendingPool) -> u64 {
    calculate_pending_interest(
        lender_deposit.shares,
        lending_pool.interest_per_share,
        lender_deposit.interest_debt,
    )
}
