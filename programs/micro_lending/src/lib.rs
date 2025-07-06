pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("2qH5FVMCSDgQsDJ7ZwvKZGfchazPsEv168wNTuoZuYKu");

#[program]
pub mod micro_lending {
    use super::*;

    // Initialize the lending platform
    pub fn initialize_platform(
        ctx: Context<InitializePlatform>,
        platform_authority: Pubkey,
        treasury_bump: u8,
        platform_fee: u16, // basis points (e.g., 100 = 1%)
        max_loan_amount: u64,
        min_loan_amount: u64,
    ) -> Result<()> {
        instructions::initialize_platform(
            ctx,
            platform_authority,
            treasury_bump,
            platform_fee,
            max_loan_amount,
            min_loan_amount,
        )?;
        Ok(())
    }

    pub fn initialize_user(ctx: Context<InitializeUser>) -> Result<()> {
        instructions::initialize_user(ctx)?;
        Ok(())
    }

    pub fn create_lending_pool(
        ctx: Context<CreateLendingPool>,
        pool_name: String,
        base_interest_rate: u16,
        max_loan_duration: i64,
    ) -> Result<()> {
        instructions::create_lending_pool(ctx, pool_name, base_interest_rate, max_loan_duration)?;
        Ok(())
    }

    pub fn request_loan(
        ctx: Context<RequestLoan>,
        amount: u64,
        duration_days: u32,
        purpose: String,
        collateral_type: u8,
    ) -> Result<()> {
        instructions::request_loan(ctx, amount, duration_days, purpose, collateral_type)?;
        Ok(())
    }

    pub fn approve_loan(ctx: Context<ApproveLoan>) -> Result<()> {
        instructions::approve_loan(ctx)?;
        Ok(())
    }

    pub fn disburse_loan(ctx: Context<DisburseLoan>) -> Result<()> {
        instructions::disburse_loan(ctx)?;
        Ok(())
    }
}
