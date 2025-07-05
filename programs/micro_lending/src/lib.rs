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
}
