use crate::error::*;
use anchor_lang::prelude::*;

// Calculate interest rate based on credit score and other factors
pub fn calculate_interest_rate(
    credit_score: u16,
    base_rate: u16,
    duration_days: u32,
) -> Result<u16> {
    let mut rate = base_rate;

    // Adjust rate based on credit score (lower score = higher rate)
    if credit_score < 500 {
        rate = rate.saturating_add(500); // Add 5%
    } else if credit_score < 650 {
        rate = rate.saturating_add(300); // Add 3%
    } else if credit_score < 750 {
        rate = rate.saturating_add(100); // Add 1%
    }

    // Adjust for loan duration (longer loans = higher rate)
    if duration_days > 365 {
        rate = rate.saturating_add(200); // Add 2%
    } else if duration_days > 180 {
        rate = rate.saturating_add(100); // Add 1%
    }

    // Cap the maximum rate at 50%
    rate = rate.min(5000);

    Ok(rate)
}

// Calculate required credit score for loan amount
pub fn calculate_required_credit_score(amount: u64, max_amount: u64) -> Result<u16> {
    let ratio = (amount as f64) / (max_amount as f64);

    let required_score = if ratio <= 0.1 {
        300
    } else if ratio <= 0.3 {
        450
    } else if ratio <= 0.6 {
        600
    } else if ratio <= 0.8 {
        700
    } else {
        800
    };

    Ok(required_score)
}

// Calculate compound interest
pub fn calculate_compound_interest(
    principal: u64,
    rate: u16, // basis points
    days: u32,
) -> Result<u64> {
    let daily_rate = (rate as f64) / 365.0 / 10000.0; // Convert basis points to daily rate
    let amount = (principal as f64) * (1.0 + daily_rate).powf(days as f64);

    Ok(amount as u64)
}

// Calculate simple interest
pub fn calculate_simple_interest(
    principal: u64,
    rate: u16, // basis points
    days: u32,
) -> Result<u64> {
    let daily_rate = (rate as f64) / 365.0 / 10000.0;
    let interest = (principal as f64) * daily_rate * (days as f64);

    Ok(interest as u64)
}

// Calculate loan payment amount
pub fn calculate_loan_payment(
    principal: u64,
    rate: u16, // basis points
    days: u32,
) -> Result<u64> {
    let total_amount = calculate_compound_interest(principal, rate, days)?;
    Ok(total_amount)
}

// Calculate credit score based on payment history
pub fn calculate_credit_score_from_history(
    successful_payments: u16,
    total_payments: u16,
    defaults: u16,
    total_borrowed: u64,
    total_repaid: u64,
) -> Result<u16> {
    if total_payments == 0 {
        return Ok(300); // Base score for new users
    }

    let payment_ratio = (successful_payments as f64) / (total_payments as f64);
    let repayment_ratio = if total_borrowed > 0 {
        (total_repaid as f64) / (total_borrowed as f64)
    } else {
        0.0
    };

    let mut score = 300; // Base score

    // Payment history weight (40%)
    score += (payment_ratio * 400.0) as u16;

    // Repayment ratio weight (30%)
    score += (repayment_ratio * 300.0) as u16;

    // Penalty for defaults (20%)
    score = score.saturating_sub(defaults * 50);

    // Experience bonus (10%)
    if total_payments > 10 {
        score += 50;
    }
    if total_payments > 50 {
        score += 50;
    }

    // Cap score between 300 and 850
    score = score.max(300).min(850);

    Ok(score)
}

// Validate social attestation
pub fn validate_social_attestation(
    attester: &Pubkey,
    user: &Pubkey,
    attestation_type: u8,
    score: u16,
) -> Result<bool> {
    // Basic validation
    require!(
        attester != user,
        MicroLendingError::SocialAttestationValidationFailed
    );
    require!(
        score <= 1000,
        MicroLendingError::SocialAttestationValidationFailed
    );

    // Attestation type validation
    match attestation_type {
        0..=5 => Ok(true), // Valid attestation types
        _ => Err(MicroLendingError::InvalidAttestationType.into()),
    }
}

// Calculate days between timestamps
pub fn days_between(start: i64, end: i64) -> u32 {
    let diff = end - start;
    (diff / 86400) as u32 // 86400 seconds in a day
}

// Check if loan is overdue
pub fn is_loan_overdue(due_date: i64, current_time: i64, grace_period_days: u8) -> bool {
    let grace_period_seconds = (grace_period_days as i64) * 86400;
    current_time > (due_date + grace_period_seconds)
}

// Calculate late fee
pub fn calculate_late_fee(
    outstanding_amount: u64,
    late_fee_rate: u16, // basis points
    days_overdue: u32,
) -> Result<u64> {
    let daily_rate = (late_fee_rate as f64) / 365.0 / 10000.0;
    let late_fee = (outstanding_amount as f64) * daily_rate * (days_overdue as f64);

    Ok(late_fee as u64)
}
