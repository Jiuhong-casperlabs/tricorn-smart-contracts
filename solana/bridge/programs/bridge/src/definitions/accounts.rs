use crate::prelude::*;

#[account]
pub struct Bridge {
    pub authority: Pubkey,
    pub paused: bool,
    pub stable_commission_percent: u64,
    pub offchain_authority: SigPublicKey,
}

impl Bridge {
    pub const SIZE: usize = 82;

    pub fn check_paused(&self) -> Result<()> {
        if self.paused {
            Err(error!(BridgeError::ContractPaused))
        } else {
            Ok(())
        }
    }

    pub fn total_commission(&self, amount: u64, gas_commission: u64) -> u64 {
        let stable_commission = (amount * self.stable_commission_percent) / HUNDRED_PERCENT_BPS;
        stable_commission + gas_commission
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn size_freeze() {
        let actual_size = Bridge {
            authority: Pubkey::new_unique(),
            paused: false,
            stable_commission_percent: 0,
            offchain_authority: SigPublicKey::Ed25519([0u8; 32]),
        }
        .try_to_vec()
        .unwrap()
        .len()
            + 8;

        println!("{actual_size}");

        assert!(actual_size <= Bridge::SIZE);
        assert_eq!(Bridge::SIZE, 82);
    }
}
