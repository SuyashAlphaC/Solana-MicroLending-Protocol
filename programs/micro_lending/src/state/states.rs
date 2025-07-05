use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Platform {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub treasury_bump: u8,
    pub platform_fee: u16, // basis points
    pub max_loan_amount: u64,
    pub min_loan_amount: u64,
    pub total_loans_issued: u64,
    pub total_volume: u64,
    pub total_defaults: u64,
    pub is_active: bool,
    pub created_at: i64,
}

#[account]
#[derive(InitSpace)]

pub struct UserProfile {
    pub owner: Pubkey,
    pub credit_score: u16,
    pub total_borrowed: u64,
    pub total_repaid: u64,
    pub active_loans: u8,
    pub successful_loans: u16,
    pub defaulted_loans: u16,
    pub reputation_score: u16,
    pub created_at: i64,
    pub last_updated: i64,
    pub kyc_verified: bool,
    pub phone_verified: bool,
    pub email_verified: bool,
    pub transaction_history_count: u16,
    pub social_attestations_count: u8,
}

#[account]
#[derive(InitSpace)]

pub struct LendingPool {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub token_account: Pubkey,
    #[max_len(500)]
    pub name: String,
    pub base_interest_rate: u16, // basis points
    pub max_loan_duration: i64,
    pub total_deposited: u64,
    pub total_borrowed: u64,
    pub available_liquidity: u64,
    pub active_loans: u32,
    pub total_interest_earned: u64,
    pub is_active: bool,
    pub created_at: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]

pub struct Loan {
    pub borrower: Pubkey,
    pub lender_pool: Pubkey,
    pub amount: u64,
    pub interest_rate: u16, // basis points
    pub duration_days: u32,
    pub disbursed_at: i64,
    pub due_date: i64,
    pub amount_repaid: u64,
    pub interest_accrued: u64,
    pub status: LoanStatus,
    #[max_len(500)]
    pub purpose: String,
    pub collateral_type: CollateralType,
    pub collateral_value: u64,
    pub payment_count: u16,
    pub last_payment_date: i64,
    pub grace_period_days: u8,
    pub late_fee_rate: u16, // basis points
    pub created_at: i64,
    pub liquidated_at: Option<i64>,
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum LoanStatus {
    Requested,
    Approved,
    Disbursed,
    Active,
    Repaid,
    Defaulted,
    Liquidated,
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum CollateralType {
    None,
    Social,
    Asset,
    Income,
    Group,
}

#[account]
#[derive(InitSpace)]
pub struct SocialAttestation {
    pub user: Pubkey,
    pub attester: Pubkey,
    pub attestation_type: AttestationType,
    pub score: u16,
    #[max_len(500)]
    pub metadata: String,
    pub verified: bool,
    pub created_at: i64,
    pub expires_at: Option<i64>,
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum AttestationType {
    Community,
    Employer,
    Family,
    Business,
    Education,
    Reference,
}

#[account]
#[derive(InitSpace)]

pub struct TransactionHistory {
    pub user: Pubkey,
    pub transaction_type: TransactionType,
    pub amount: u64,
    pub counterparty: Option<Pubkey>,
    pub timestamp: i64,
    pub frequency_score: u16,
    pub consistency_score: u16,
    pub verified: bool,
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum TransactionType {
    MobileMoney,
    Remittance,
    Merchant,
    Utility,
    Savings,
    Investment,
    Loan,
    Repayment,
}

#[account]
#[derive(InitSpace)]

pub struct LenderDeposit {
    pub lender: Pubkey,
    pub pool: Pubkey,
    pub amount_deposited: u64,
    pub shares: u64,
    pub interest_earned: u64,
    pub deposited_at: i64,
    pub last_claim: i64,
}

#[account]
#[derive(InitSpace)]

pub struct RepaymentSchedule {
    pub loan: Pubkey,
    pub installment_amount: u64,
    pub installment_count: u16,
    pub frequency_days: u16, // days between payments
    pub next_payment_due: i64,
    pub payments_made: u16,
    pub total_late_fees: u64,
    pub auto_debit_enabled: bool,
}
