use std::{cell::RefCell, rc::Rc};

use anchor_lang::{
    prelude::*,
    solana_program::{
        self,
        instruction::Instruction,
        sysvar::instructions::{load_current_index_checked, load_instruction_at_checked},
    },
};

#[derive(thiserror::Error, Debug, Clone, Copy)]
#[error("read constant did not match expectation")]
pub struct BorshConstError;

#[macro_export]
macro_rules! define_const_borsh_ty {
    ($name:ident, $val:expr, $t:ty) => {
        pub struct $name;

        impl $name
        where
            $t: borsh::BorshSerialize + borsh::BorshDeserialize,
        {
            const T: $t = $val;
        }

        impl borsh::BorshSerialize for $name {
            fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                Self::T.serialize(writer)
            }
        }

        impl borsh::BorshDeserialize for $name {
            fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
                let data = Self::T.try_to_vec()?;

                if data.as_slice() != &buf[..data.len()] {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        BorshConstError,
                    ));
                }

                *buf = &buf[data.len()..];

                Ok(Self)
            }
        }
    };
}

#[derive(AnchorSerialize, AnchorDeserialize)]
struct Ed25519SignatureOffsets {
    signature_offset: u16,             // offset to ed25519 signature of 64 bytes
    signature_instruction_index: u16,  // instruction index to find signature
    public_key_offset: u16,            // offset to public key of 32 bytes
    public_key_instruction_index: u16, // instruction index to find public key
    message_data_offset: u16,          // offset to start of message data
    message_data_size: u16,            // size of message data
    message_instruction_index: u16,    // index of instruction data to get message data
}

#[derive(AnchorSerialize, AnchorDeserialize)]
struct SecpSignatureOffsets {
    signature_offset: u16,
    signature_instruction_index: u8,
    eth_address_offset: u16,
    eth_address_instruction_index: u8,
    message_data_offset: u16,
    message_data_size: u16,
    message_instruction_index: u8,
}

fn read_ed25519_instruction(instruction: &Instruction) -> Result<Vec<Ed25519SignatureOffsets>> {
    const ITEM_STRIDE: usize = 14;

    require_keys_eq!(
        instruction.program_id,
        SignatureAlgorithm::Ed25519.program(),
        crate::BridgeError::SignatureVerificationFailed
    );

    require_gte!(
        instruction.data.len(),
        2,
        crate::BridgeError::SignatureVerificationFailed
    );

    let count = instruction.data[0];
    let data = &instruction.data[2..];

    require_gte!(
        data.len(),
        ITEM_STRIDE * (count as usize),
        crate::BridgeError::SignatureVerificationFailed
    );

    let mut v = Vec::with_capacity(count as usize);
    for i in 0..count as usize {
        let data = &data[i * ITEM_STRIDE..(i + 1) * ITEM_STRIDE];
        let offsets = Ed25519SignatureOffsets::try_from_slice(data)
            .map_err(|_| error!(crate::BridgeError::SignatureVerificationFailed))?;
        v.push(offsets);
    }

    Ok(v)
}

fn read_secp256k1_instruction(instruction: &Instruction) -> Result<Vec<SecpSignatureOffsets>> {
    const ITEM_STRIDE: usize = 11;

    require_keys_eq!(
        instruction.program_id,
        SignatureAlgorithm::Secp256k1.program(),
        crate::BridgeError::SignatureVerificationFailed
    );

    require_gte!(
        instruction.data.len(),
        1,
        crate::BridgeError::SignatureVerificationFailed
    );

    let count = instruction.data[0];
    let data = &instruction.data[1..];

    require_gte!(
        data.len(),
        ITEM_STRIDE * (count as usize),
        crate::BridgeError::SignatureVerificationFailed
    );

    let mut v = Vec::with_capacity(count as usize);
    for i in 0..count as usize {
        let data = &data[i * ITEM_STRIDE..(i + 1) * ITEM_STRIDE];
        let offsets = SecpSignatureOffsets::try_from_slice(data)
            .map_err(|_| error!(crate::BridgeError::SignatureVerificationFailed))?;
        v.push(offsets);
    }

    Ok(v)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum SignatureAlgorithm {
    Ed25519,
    Secp256k1,
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
/**
 * Public key formats supported for signatures.
 */
pub enum SigPublicKey {
    Ed25519([u8; 32]),
    Secp256k1([u8; 33]),
}

impl AsRef<[u8]> for SigPublicKey {
    fn as_ref(&self) -> &[u8] {
        match self {
            SigPublicKey::Ed25519(data) => data,
            SigPublicKey::Secp256k1(data) => data,
        }
    }
}

impl SignatureAlgorithm {
    pub fn program(&self) -> Pubkey {
        match self {
            SignatureAlgorithm::Ed25519 => solana_program::ed25519_program::ID,
            SignatureAlgorithm::Secp256k1 => solana_program::secp256k1_program::ID,
        }
    }
}

pub struct InstructionCache<'a, 'b> {
    instructions: &'a AccountInfo<'b>,
    cache: Vec<(u8, Rc<RefCell<Instruction>>)>,
}

impl<'a, 'b: 'a> InstructionCache<'a, 'b> {
    pub fn new(instructions: &'a AccountInfo<'b>) -> Self {
        Self {
            instructions,
            cache: Vec::with_capacity(2),
        }
    }

    pub fn current_index(&self) -> Result<usize> {
        Ok(load_current_index_checked(self.instructions)? as usize)
    }

    pub fn load(&mut self, idx: usize) -> Result<Rc<RefCell<Instruction>>> {
        let i = match self.try_load(idx) {
            Some(i) => i,
            None => {
                let ix = load_instruction_at_checked(idx, self.instructions)?;
                let i = self.cache.len();
                self.cache.push((idx as u8, Rc::new(RefCell::new(ix))));

                i
            }
        };

        Ok(self.cache[i].1.clone())
    }

    fn try_load(&self, idx: usize) -> Option<usize> {
        for (i, (ix_idx, _)) in self.cache.iter().enumerate() {
            if (*ix_idx as usize) == idx {
                return Some(i);
            }
        }

        None
    }
}

pub struct SignatureVerifier<'a, 'b> {
    expected_data: &'a [u8],
    expected_signer: &'a SigPublicKey,
    ix_cache: InstructionCache<'a, 'b>,
}

impl<'a, 'b: 'a> SignatureVerifier<'a, 'b> {
    pub fn new(
        data: &'a [u8],
        signer: &'a SigPublicKey,
        instructions: &'a AccountInfo<'b>,
    ) -> Self {
        SignatureVerifier {
            expected_data: data,
            expected_signer: signer,
            ix_cache: InstructionCache::new(instructions),
        }
    }

    pub fn verify(mut self) -> Result<()> {
        // WARN: Secp256k1 signatures are currently untested and thus disabled
        if let SigPublicKey::Secp256k1(_) = self.expected_signer {
            msg!("secp256k1 is unsupported");
            return Err(crate::BridgeError::SignatureVerificationFailed.into());
        }

        let current_index = self.ix_cache.current_index()?;
        require_neq!(
            current_index,
            0,
            crate::BridgeError::SignatureVerificationFailed
        );

        let verify_ix_index = current_index - 1;

        let verify_ix = self.ix_cache.load(verify_ix_index)?;
        let verify_ix = verify_ix.borrow();

        let mut verifier = |pk_ix, pk_offset, data_ix, data_offset, data_length| {
            let pk_ix = if (pk_ix as u16) == u16::MAX {
                verify_ix_index
            } else {
                pk_ix
            };
            let data_ix = if (data_ix as u16) == u16::MAX {
                verify_ix_index
            } else {
                data_ix
            };

            let data_ix = self.ix_cache.load(data_ix)?;
            let data_ix = data_ix.borrow();
            let data = &data_ix.data[data_offset as usize..(data_offset + data_length) as usize];

            let pk_ix = self.ix_cache.load(pk_ix)?;
            let pk_ix = pk_ix.borrow();
            let pk = &pk_ix.data[pk_offset as usize..(pk_offset as usize) + 32];

            if data == self.expected_data && pk == self.expected_signer.as_ref() {
                Result::Ok(true)
            } else {
                Result::Ok(false)
            }
        };

        match self.expected_signer {
            SigPublicKey::Ed25519(..) => {
                let offsets = read_ed25519_instruction(&verify_ix)?;

                for Ed25519SignatureOffsets {
                    public_key_offset,
                    public_key_instruction_index,
                    message_data_offset,
                    message_data_size,
                    message_instruction_index,
                    ..
                } in offsets
                {
                    if verifier(
                        public_key_instruction_index as usize,
                        public_key_offset,
                        message_instruction_index as usize,
                        message_data_offset,
                        message_data_size,
                    )? {
                        return Ok(());
                    }
                }
            }
            SigPublicKey::Secp256k1(..) => {
                for SecpSignatureOffsets {
                    eth_address_offset,
                    eth_address_instruction_index,
                    message_data_offset,
                    message_data_size,
                    message_instruction_index,
                    ..
                } in read_secp256k1_instruction(&verify_ix)?
                {
                    if verifier(
                        eth_address_instruction_index as usize,
                        eth_address_offset,
                        message_instruction_index as usize,
                        message_data_offset,
                        message_data_size,
                    )? {
                        return Ok(());
                    }
                }
            }
        }

        Err(error!(crate::BridgeError::SignatureVerificationFailed))
    }
}
