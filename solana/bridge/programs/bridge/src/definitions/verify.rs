use crate::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct VerifyEd25519SignatureInstruction {
    pub count: u8,
    pub padding: u8,
    pub sig_offset: u16,
    pub sig_ix_idx: u16,
    pub pubkey_offset: u16,
    pub pubkey_ix_idx: u16,
    pub message_offset: u16,
    pub message_len: u16,
    pub message_ix_idx: u16,
    pub pubkey: [u8; 32],
    pub sig: [u8; 64],
}
