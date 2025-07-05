use crate::error::*;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

pub fn create_lending_pool(
    ctx: Context<CreateLendingPool>,
    pool_name: String,
    base_interest_rate: u16,
    max_loan_duration: i64,
) -> Result<()> {
    require!(
        base_interest_rate <= 5000,
        MicroLendingError::InvalidPoolConfiguration
    ); // Max 50%
    require!(
        max_loan_duration > 0,
        MicroLendingError::InvalidPoolConfiguration
    );
    require!(
        pool_name.len() <= 50,
        MicroLendingError::InvalidPoolConfiguration
    );

    let lending_pool = &mut ctx.accounts.lending_pool;
    let clock = Clock::get()?;

    lending_pool.authority = ctx.accounts.authority.key();
    lending_pool.mint = ctx.accounts.mint.key();
    lending_pool.token_account = ctx.accounts.token_account.key();
    lending_pool.name = pool_name;
    lending_pool.base_interest_rate = base_interest_rate;
    lending_pool.max_loan_duration = max_loan_duration;
    lending_pool.total_deposited = 0;
    lending_pool.total_borrowed = 0;
    lending_pool.available_liquidity = 0;
    lending_pool.active_loans = 0;
    lending_pool.total_interest_earned = 0;
    lending_pool.is_active = true;
    lending_pool.created_at = clock.unix_timestamp;


    msg!("Lending pool created: {}", lending_pool.name);
    Ok(())
}

#[derive(Accounts)]
pub struct CreateLendingPool<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + LendingPool::INIT_SPACE,
        seeds = [b"lending_pool", authority.key().as_ref(), mint.key().as_ref()],
        bump
    )]
    pub lending_pool: Account<'info, LendingPool>,

    #[account(
        init,
        payer = authority,
        token::mint = mint,
        token::authority = token_account,
        seeds = [b"pool_token_account", lending_pool.key().as_ref()],
        bump 
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
