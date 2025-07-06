use crate::error::*;
use crate::states::*;
use crate::utils::*;
use crate::SEEDS_USER;
use anchor_lang::prelude::*;

pub fn add_attestation(
    ctx: Context<AddAttestation>,
    attestation_type: u8,
    score: u16,
    metadata: String,
    expires_at: Option<i64>,
) -> Result<()> {
    let user_profile = &mut ctx.accounts.user_profile;
    let attestation = &mut ctx.accounts.social_attestation;
    let clock = Clock::get()?;

    // Validate inputs
    require!(score <= 1000, MicroLendingError::InvalidAttestation); // Example max score
    require!(metadata.len() <= 500, MicroLendingError::InvalidAttestation);

    // Validate attestation using utility function
    validate_social_attestation(
        &ctx.accounts.attester.key(),
        &ctx.accounts.user.key(),
        attestation_type,
        score,
    )?;

    let att_type = match attestation_type {
        0 => AttestationType::Community,
        1 => AttestationType::Employer,
        2 => AttestationType::Family,
        3 => AttestationType::Business,
        4 => AttestationType::Education,
        5 => AttestationType::Reference,
        _ => return Err(MicroLendingError::InvalidAttestationType.into()),
    };
    // Initialize the attestation account
    attestation.user = ctx.accounts.user.key();
    attestation.attester = ctx.accounts.attester.key();
    attestation.attestation_type = att_type;
    attestation.score = score;
    attestation.metadata = metadata;
    attestation.verified = true; // The attester is implicitly trusted in this context
    attestation.created_at = clock.unix_timestamp;
    attestation.expires_at = expires_at;

    // Update the user's profile
    user_profile.social_attestations_count = user_profile
        .social_attestations_count
        .checked_add(1)
        .unwrap();
    user_profile.last_updated = clock.unix_timestamp;

    msg!(
        "Social attestation added for user: {}",
        ctx.accounts.user.key()
    );
    Ok(())
}

#[derive(Accounts)]
pub struct AddAttestation<'info> {
    #[account(mut)]
    pub attester: Signer<'info>,

    /// CHECK: The user account for whom the attestation is being added.
    #[account(mut)]
    pub user: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [SEEDS_USER, user.key().as_ref()],
        bump
    )]
    pub user_profile: Account<'info, UserProfile>,

    #[account(
        init,
        payer = attester,
        space = 8 + SocialAttestation::INIT_SPACE,
        seeds = [b"social_attestation", user.key().as_ref(), attester.key().as_ref()],
        bump
    )]
    pub social_attestation: Account<'info, SocialAttestation>,

    pub system_program: Program<'info, System>,
}
