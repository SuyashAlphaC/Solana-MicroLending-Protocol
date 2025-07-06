use crate::states::*;
use crate::SEEDS_PLATFORM;
use crate::SEEDS_USER;
use anchor_lang::prelude::*;

pub fn add_transaction_history(
    ctx: Context<AddTransactionHistory>,
    transaction_type: TransactionType,
    amount: u64,
    counterparty: Option<Pubkey>,
    timestamp: i64,
    frequency_score: u16,
    consistency_score: u16,
) -> Result<()> {
    let user_profile = &mut ctx.accounts.user_profile;
    let trans_hist = &mut ctx.accounts.transaction_history;
    let current = Clock::get()?.unix_timestamp;

    // Initialize the transaction history account
    trans_hist.user = ctx.accounts.user.key();
    trans_hist.transaction_type = transaction_type;
    trans_hist.amount = amount;
    trans_hist.counterparty = counterparty;
    trans_hist.timestamp = timestamp;
    trans_hist.frequency_score = frequency_score;
    trans_hist.consistency_score = consistency_score;
    trans_hist.verified = true; // Added by a trusted authority

    // Update the user's profile
    user_profile.transaction_history_count = user_profile
        .transaction_history_count
        .checked_add(1)
        .unwrap();
    user_profile.last_updated = current;

    msg!(
        "Transaction history added for user: {}",
        ctx.accounts.user.key()
    );
    Ok(())
}

#[derive(Accounts)]
#[instruction(transaction_type: TransactionType, amount: u64)]
pub struct AddTransactionHistory<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [SEEDS_PLATFORM],
        bump,
        has_one = authority
    )]
    pub platform: Account<'info, Platform>,

    /// CHECK: The user account for whom the transaction is being added.
    pub user: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [SEEDS_USER, user.key().as_ref()],
        bump
    )]
    pub user_profile: Account<'info, UserProfile>,

    #[account(
        init,
        payer = authority,
        space = 8 + TransactionHistory::INIT_SPACE,
        seeds = [b"transaction_history", user.key().as_ref(), &user_profile.transaction_history_count.to_le_bytes()],
        bump
    )]
    pub transaction_history: Account<'info, TransactionHistory>,

    pub system_program: Program<'info, System>,
}
