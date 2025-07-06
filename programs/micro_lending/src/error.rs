use anchor_lang::prelude::*;

#[error_code]
pub enum MicroLendingError {
    #[msg("Invalid pool configuration")]
    InvalidPoolConfiguration,
    #[msg("Invalid Platform configuration")]
    InvalidPlatformConfiguration,
    #[msg("Pool Inactive")]
    PoolNotActive,
    #[msg("Loan amount is Too Low")]
    LoanAmountTooLow,
    #[msg("Loan amount is Too High")]
    LoanAmountTooHigh,
    #[msg("Loan Duration Very Long")]
    LoanDurationTooLong,
    #[msg("Clear Active Loans")]
    BorrowerHasActiveLoan,
    #[msg("Out of Funds")]
    InsufficientLiquidity,
    #[msg("Credit Score should be more")]
    InsufficientCreditScore,
    #[msg("Provide a valid Collateral Type")]
    InvalidCollateralType,
    #[msg("Attestation Validation Failed")]
    SocialAttestationValidationFailed,
    #[msg("Get a Valid Attestation")]
    InvalidAttestationType,
    #[msg("Invalid Loan State")]
    InvalidLoanState,
    #[msg(Unauthorized Signer)]
    Unauthorized,
}
