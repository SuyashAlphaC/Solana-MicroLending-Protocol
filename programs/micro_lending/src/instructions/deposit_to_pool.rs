use crate::error::*;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

pub fn deposit_to_pool(ctx: Context<DepositToPool>, amount: u64) -> Result<()> {
    let lending_pool = &mut ctx.accounts.lending_pool;
    let lender_deposit = &mut ctx.accounts.lender_deposit;
    let current = Clock::get()?.unix_timestamp;

    require!(amount > 0, MicroLendingError::InvalidPaymentAmount);
    require!(lending_pool.is_active, MicroLendingError::PoolNotActive);

    // Transfer tokens from lender to pool
    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.lender_token_account.to_account_info(),
        to: ctx.accounts.pool_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        authority: ctx.accounts.lender.to_account_info(),
    };

    let cpi_prgm = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_prgm, transfer_cpi_accounts);
    let decimals = ctx.accounts.mint.decimals;

    transfer_checked(cpi_ctx, amount, decimals)?;

    // Update lender deposit record
    if lender_deposit.amount_deposited == 0 {
        // First deposit
        lender_deposit.lender = ctx.accounts.lender.key();
        lender_deposit.pool = lending_pool.key();
        lender_deposit.deposited_at = current;
    }

    lender_deposit.amount_deposited = lender_deposit.amount_deposited.checked_add(amount).unwrap();
    // Update lending pool
    lending_pool.total_deposited = lending_pool.total_deposited.checked_add(amount).unwrap();

    //Update lender's shares
    let shares_to_mint = if lending_pool.total_shares == 0 {
        amount
    } else {
        amount
            .checked_mul(lending_pool.total_shares)
            .unwrap()
            .checked_div(lending_pool.total_deposited)
            .unwrap()
    };
    lender_deposit.shares = lender_deposit.shares.checked_add(shares_to_mint).unwrap();
    lending_pool.total_shares = lending_pool
        .total_shares
        .checked_add(shares_to_mint)
        .unwrap();

    // Update interest debt to current accumulated interest
    let current_interest_debt =
        (shares_to_mint as u128 * lending_pool.interest_per_share as u128) / 1_000_000_000;
    lender_deposit.interest_debt = lender_deposit
        .interest_debt
        .checked_add(current_interest_debt as u64)
        .unwrap();

    lending_pool.available_liquidity = lending_pool
        .available_liquidity
        .checked_add(amount)
        .unwrap();

    msg!(
        "Deposit made: {} tokens by lender: {}",
        amount,
        ctx.accounts.lender.key()
    );
    Ok(())
}

#[derive(Accounts)]
pub struct DepositToPool<'info> {
    #[account(
        mut,
        constraint = lending_pool.is_active @ MicroLendingError::PoolNotActive
    )]
    pub lending_pool: Account<'info, LendingPool>,
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = lender,
        space = 8 + LenderDeposit::INIT_SPACE,
        seeds = [b"lender_deposit", lender.key().as_ref(), lending_pool.key().as_ref()],
        bump
    )]
    pub lender_deposit: Account<'info, LenderDeposit>,

    #[account(
        mut,
        constraint = pool_token_account.key() == lending_pool.token_account
    )]
    pub pool_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        constraint = lender_token_account.owner == lender.key(),
        associated_token::mint = mint,
        associated_token::authority = lender,
        associated_token::token_program = token_program,
    )]
    pub lender_token_account: InterfaceAccount<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(mut)]
    pub lender: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}
