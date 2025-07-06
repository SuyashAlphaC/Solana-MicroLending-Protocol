use crate::states::*;
use crate::SEEDS_USER;
use anchor_lang::prelude::*;

pub fn initialize_user(ctx: Context<InitializeUser>) -> Result<()> {
    let user_profile = &mut ctx.accounts.user_profile;
    let current = Clock::get()?.unix_timestamp;

    user_profile.owner = ctx.accounts.user.key();
    user_profile.credit_score = 300; //MAX(when initialized)
    user_profile.total_borrowed = 0;
    user_profile.total_repaid = 0;
    user_profile.active_loans = 0;
    user_profile.successful_loans = 0;
    user_profile.defaulted_loans = 0;
    user_profile.reputation_score = 500; // MAX (when initialized)
    user_profile.created_at = current;
    user_profile.last_updated = current;
    user_profile.kyc_verified = false;
    user_profile.phone_verified = false;
    user_profile.email_verified = false;
    user_profile.transaction_history_count = 0;
    user_profile.social_attestations_count = 0;

    msg!("User profile initialized for: {}", ctx.accounts.user.key());
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeUser<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + UserProfile::INIT_SPACE,
        seeds = [SEEDS_USER,  user.key().as_ref()],
        bump
    )]
    pub user_profile: Account<'info, UserProfile>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}
