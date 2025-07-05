use crate::error::*;
use crate::states::*;
use crate::{SEEDS_PLATFORM, SEEDS_TREASURY};
use anchor_lang::prelude::*;

pub fn initialize_platform(
    ctx: Context<InitializePlatform>,
    platform_authority: Pubkey,
    treasury_bump: u8,
    platform_fee: u16,
    max_loan_amount: u64,
    min_loan_amount: u64,
) -> Result<()> {
    require!(
        platform_fee <= 1000,
        MicroLendingError::InvalidPlatformConfiguration
    ); // Max 10%
    require!(
        max_loan_amount > min_loan_amount,
        MicroLendingError::InvalidPlatformConfiguration
    );

    let platform = &mut ctx.accounts.platform;
    let clock = Clock::get()?;

    platform.authority = platform_authority;
    platform.treasury = ctx.accounts.treasury.key();
    platform.treasury_bump = treasury_bump;
    platform.platform_fee = platform_fee;
    platform.max_loan_amount = max_loan_amount;
    platform.min_loan_amount = min_loan_amount;
    platform.total_loans_issued = 0;
    platform.total_volume = 0;
    platform.total_defaults = 0;
    platform.is_active = true;
    platform.created_at = clock.unix_timestamp;

    msg!(
        "Platform initialized with authority: {}",
        platform_authority
    );
    Ok(())
}

#[derive(Accounts)]
#[instruction(platform_authority: Pubkey, treasury_bump: u8)]
pub struct InitializePlatform<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + Platform::INIT_SPACE,
        seeds = [SEEDS_PLATFORM],
        bump
    )]
    pub platform: Account<'info, Platform>,

    /// CHECK: This is the treasury PDA
    #[account(
        seeds = [SEEDS_TREASURY],
        bump = treasury_bump
    )]
    pub treasury: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}
