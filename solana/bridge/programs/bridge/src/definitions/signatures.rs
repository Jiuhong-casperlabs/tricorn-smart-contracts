use crate::util::{SigPublicKey, SignatureVerifier};
use crate::{define_const_borsh_ty, prelude::*};

define_const_borsh_ty!(
    BridgeInSignaturePrefix,
    *BRIDGE_IN_SIGNATURE_PREFIX,
    [u8; 12]
);

define_const_borsh_ty!(
    TransferOutSignaturePrefix,
    *TRANSFER_OUT_SIGNATURE_PREFIX,
    [u8; 12]
);

define_const_borsh_ty!(InitializeSignaturePrefix, *INIT_SIGNATURE_PREFIX, [u8; 12]);
define_const_borsh_ty!(
    UpdateAuthoritySignaturePrefix,
    *UPDATE_AUTHORITY_SIGNATURE_PREFIX,
    [u8; 12]
);

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct BridgeInSignatureBorrowed<'a> {
    prefix: BridgeInSignaturePrefix,
    sender: &'a Pubkey,
    mint: &'a Pubkey,
    amount: u64,
    commission: u64,
    destination_chain: &'a str,
    destination_address: &'a str,
    deadline: u64,
    nonce: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct BridgeInSignature {
    prefix: [u8; 12],
    sender: Pubkey,
    mint: Pubkey,
    amount: u64,
    commission: u64,
    destination_chain: String,
    destination_address: String,
    deadline: u64,
    nonce: u64,
}

impl<'a> BridgeInSignatureBorrowed<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sender: &'a Pubkey,
        mint: &'a Pubkey,
        amount: u64,
        commission: u64,
        destination_chain: &'a str,
        destination_address: &'a str,
        deadline: u64,
        nonce: u64,
    ) -> Self {
        Self {
            prefix: BridgeInSignaturePrefix,
            sender,
            mint,
            amount,
            commission,
            destination_chain,
            destination_address,
            deadline,
            nonce,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.try_to_vec().unwrap()
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TransferOutSignatureBorrowed<'a> {
    prefix: TransferOutSignaturePrefix,
    recipient: &'a Pubkey,
    mint: &'a Pubkey,
    amount: u64,
    commission: u64,
    nonce: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TransferOutSignature {
    prefix: [u8; 12],
    recipient: Pubkey,
    mint: Pubkey,
    amount: u64,
    commission: u64,
    nonce: u64,
}

impl<'a> TransferOutSignatureBorrowed<'a> {
    pub fn new(
        recipient: &'a Pubkey,
        mint: &'a Pubkey,
        amount: u64,
        commission: u64,
        nonce: u64,
    ) -> Self {
        Self {
            prefix: TransferOutSignaturePrefix,
            recipient,
            mint,
            amount,
            commission,
            nonce,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.try_to_vec().unwrap()
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeSignatureBorrowed<'a> {
    prefix: InitializeSignaturePrefix,
    nonce: u64,
    bridge_account: &'a Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeSignature {
    prefix: [u8; 12],
    nonce: u64,
    bridge_account: Pubkey,
}

impl<'a> InitializeSignatureBorrowed<'a> {
    pub fn new(bridge_account: &'a Pubkey, nonce: u64) -> Self {
        Self {
            prefix: InitializeSignaturePrefix,
            nonce,
            bridge_account,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.try_to_vec().unwrap()
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateAuthoritySignatureBorrowed<'a> {
    prefix: UpdateAuthoritySignaturePrefix,
    nonce: u64,
    bridge_account: &'a Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateAuthoritySignature {
    prefix: [u8; 12],
    nonce: u64,
    bridge_account: Pubkey,
}

impl<'a> UpdateAuthoritySignatureBorrowed<'a> {
    pub fn new(bridge_account: &'a Pubkey, nonce: u64) -> Self {
        Self {
            prefix: UpdateAuthoritySignaturePrefix,
            nonce,
            bridge_account,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.try_to_vec().unwrap()
    }
}

pub trait Verify: AnchorSerialize {
    fn verify(&self, signer: &SigPublicKey, instructions: &AccountInfo) -> Result<()> {
        let data = self.try_to_vec().expect("infallible");

        SignatureVerifier::new(&data, signer, instructions).verify()?;

        Ok(())
    }
}

impl<'a> Verify for InitializeSignatureBorrowed<'a> {}
impl<'a> Verify for BridgeInSignatureBorrowed<'a> {}
impl<'a> Verify for TransferOutSignatureBorrowed<'a> {}
impl<'a> Verify for UpdateAuthoritySignatureBorrowed<'a> {}
