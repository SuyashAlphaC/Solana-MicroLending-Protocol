use anchor_lang::prelude::*;

#[error_code]
pub enum MicroLendingError {
    #[msg("Invalid pool configuration")]
    InvalidPoolConfiguration,
    #[msg("Invalid Platform configuration")]
    InvalidPlatformConfiguration,
}
