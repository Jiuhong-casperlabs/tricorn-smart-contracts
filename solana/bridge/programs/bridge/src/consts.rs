use anchor_lang::prelude::*;

declare_id!("4LN58RWxhbQgsAMVpi6bgrJgmY6rvLdohZpBNenFPY27");

/**
 * Fixed seed part for program-derived addresses for fund vaults
 */
pub const PDA_FUND_VAULT: &[u8] = b"fund_vault";
/**
 * Fixed seed part for program-derived addresses for fee vaults
 */
pub const PDA_FEE_VAULT: &[u8] = b"fee_vault";
/**
 * Fixed seed part for program-derived addresses for nonce accounts
 */
pub const PDA_NONCE: &[u8] = b"nonce";

/**
 * Default commission percent (in BPS) for an instantiated bridge contract.
 */
pub const DEFAULT_STABLE_COMMISSION_BPS: u64 = 400;
/**
 * Hundred percent in BPS.
 */
pub const HUNDRED_PERCENT_BPS: u64 = 10000;

/**
 * Signature data payload prefix for the `bridge_in` method
 */
pub const BRIDGE_IN_SIGNATURE_PREFIX: &[u8; 12] = b"BBSOL/BRG_IN";

/**
 * Signature data payload prefix for the `transfer_out` method
 */
pub const TRANSFER_OUT_SIGNATURE_PREFIX: &[u8; 12] = b"BBSOL/TR_OUT";

/**
 * Signature data payload prefix for the `initialize` method
 */
pub const INIT_SIGNATURE_PREFIX: &[u8; 12] = b"BBSOL/INITLZ";

/**
 * Signature data payload prefix for the `update_offchain_authority` method
 */
pub const UPDATE_AUTHORITY_SIGNATURE_PREFIX: &[u8; 12] = b"BBSOL/UPDSIG";
