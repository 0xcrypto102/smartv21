use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
     #[msg("Program is paused")]
    ProgramPaused,
    #[msg("Loan doesn't have enough sol")]
    InsufficientBalance,
    #[msg("Please deposit the total supply of token")]
    InsufficientTokenBalance,
    #[msg("Loan duration is invalid. It should be 1 day")]
    InvalidDuration,
    #[msg("Loan sol amount is invalid. It should be one of 2 / 5 / 10 / 20 SOL")]
    InvalidInitSolAmount,
    #[msg("invalid wrapped sol mint address")]
    InvalidWrappedSolMint,
    #[msg("invalid mint account")]
    InvalidMintAccount,
    #[msg("Loan has already been repaid")]
    LoanAlreadyRepaid,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Loan has expired")]
    LoanExpired,
    #[msg("Loan is not expired yet")]
    LoanNotExpired,
    #[msg("Invalid fee percentage")]
    InvalidFee,
    #[msg("Invalid treasury account")]
    InvalidTreasury,
    #[msg("Token mint authority must be revoked")]
    MintAuthorityNotRevoked,
    #[msg("Token freeze authority must be revoked")]
    FreezeAuthorityNotRevoked,
}
