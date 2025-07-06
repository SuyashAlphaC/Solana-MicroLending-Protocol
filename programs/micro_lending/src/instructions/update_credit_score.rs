use crate::states::*;
use crate::utils::*;
use crate::{SEEDS_PLATFORM, SEEDS_USER};
use anchor_lang::prelude::*;

pub fn update_credit_score(ctx: Context<UpdateCreditScore>) -> Result<()> {
    let user_profile = &mut ctx.accounts.user_profile;
    let clock = Clock::get()?;

    let total_loans = user_profile
        .successful_loans
        .checked_add(user_profile.defaulted_loans)
        .unwrap();

    let new_score = calculate_credit_score_from_history(
        user_profile.successful_loans,
        total_loans,
        user_profile.defaulted_loans,
        user_profile.total_borrowed,
        user_profile.total_repaid,
    )?;

    user_profile.credit_score = new_score;
    user_profile.last_updated = clock.unix_timestamp;

    msg!(
        "Credit score for user {} updated to: {}",
        user_profile.owner,
        new_score
    );

    Ok(())
}

#[derive(Accounts)]
pub struct UpdateCreditScore<'info> {
    // platform authority can update the credit score.
    pub authority: Signer<'info>,

    // Add the platform account to validate the authority.
    #[account(
        seeds = [SEEDS_PLATFORM],
        bump,
        has_one = authority
    )]
    pub platform: Account<'info, Platform>,

    #[account(
        mut,
        seeds = [SEEDS_USER, user.key().as_ref()],
        bump
    )]
    pub user_profile: Account<'info, UserProfile>,

    /// CHECK: The user account whose credit score is being updated.
    pub user: AccountInfo<'info>,
}
