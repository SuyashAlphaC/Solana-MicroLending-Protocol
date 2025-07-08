use crate::error::*;
use crate::states::*;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;

use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

pub fn withdraw_from_pool(ctx: Context<WithdrawFromPool>, shares_to_withdraw: u64) -> Result<()> {
    let lending_pool = &mut ctx.accounts.lending_pool;
    let lender_deposit = &mut ctx.accounts.lender_deposit;

    require!(shares_to_withdraw > 0, MicroLendingError::InvalidAmount);
    require!(
        lender_deposit.shares >= shares_to_withdraw,
        MicroLendingError::InsufficientShares
    );

    // First, claim any outstanding interest to ensure balances are up-to-date
    let unclaimed_interest =
        (lender_deposit.shares as u128 * lending_pool.interest_per_share as u128 / 1_000_000_000)
            .saturating_sub(lender_deposit.interest_debt as u128) as u64;

    msg!("Uncalimed Interest : {}", unclaimed_interest);
    if unclaimed_interest > 0 {
        lender_deposit.interest_claimed = lender_deposit
            .interest_claimed
            .checked_add(unclaimed_interest)
            .unwrap();
        lender_deposit.interest_debt = lender_deposit
            .interest_debt
            .checked_add(unclaimed_interest)
            .unwrap();
        lending_pool.total_interest_distributed = lending_pool
            .total_interest_distributed
            .checked_add(unclaimed_interest)
            .unwrap();
    }

    // Calculate the value of shares to withdraw
    let total_assets = lending_pool
        .available_liquidity
        .checked_add(lending_pool.total_borrowed)
        .unwrap();

    msg!("Total assets in pool : {}", total_assets);
    let withdraw_amount = (shares_to_withdraw as u128)
        .checked_mul(total_assets as u128)
        .unwrap()
        .checked_div(lending_pool.total_shares as u128)
        .unwrap() as u64;

    let total_withdraw_amount = withdraw_amount.checked_add(unclaimed_interest).unwrap();
    msg!("Total withdraw amount : {}", total_withdraw_amount);
    require!(
        lending_pool.available_liquidity >= total_withdraw_amount,
        MicroLendingError::InsufficientLiquidity
    );

    // Perform the transfer from the pool to the lender
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
    transfer_checked(cpi_ctx, total_withdraw_amount, ctx.accounts.mint.decimals)?;

    // Update accounts
    lender_deposit.shares = lender_deposit
        .shares
        .checked_sub(shares_to_withdraw)
        .unwrap();
    lender_deposit.amount_deposited = lender_deposit
        .amount_deposited
        .checked_sub(total_withdraw_amount)
        .unwrap();

    lending_pool.total_shares = lending_pool
        .total_shares
        .checked_sub(shares_to_withdraw)
        .unwrap();
    lending_pool.total_deposited = lending_pool
        .total_deposited
        .checked_sub(total_withdraw_amount)
        .unwrap();
    lending_pool.available_liquidity = lending_pool
        .available_liquidity
        .checked_sub(total_withdraw_amount)
        .unwrap();

    msg!(
        "Withdrew {} tokens for {} shares from pool by {}",
        total_withdraw_amount,
        shares_to_withdraw,
        ctx.accounts.lender.key()
    );
    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawFromPool<'info> {
    #[account(mut)]
    pub lender: Signer<'info>,

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
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}
