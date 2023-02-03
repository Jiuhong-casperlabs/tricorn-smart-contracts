use anchor_lang::prelude::*;

#[error_code]
pub enum BridgeError {
    #[msg("timestamp expired")]
    TimestampExpired,
    #[msg("invalid nonce account")]
    InvalidNonceAccount,
    #[msg("deadline is too far into the future")]
    DeadlineTooFar,
    #[msg("contract is paused")]
    ContractPaused,
    #[msg("signature verification failed")]
    SignatureVerificationFailed,
}
