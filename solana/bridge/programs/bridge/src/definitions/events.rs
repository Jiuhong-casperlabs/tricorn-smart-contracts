use crate::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Eq, PartialEq, Debug)]
pub struct FundsInEvent {
    pub sender: Pubkey,
    pub nonce: u64,
    pub token: Pubkey,
    pub amount: u64,
    pub stable_commission_percent: u64,
    pub gas_commission: u64,
    pub destination_chain: String,
    pub destination_address: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Eq, PartialEq, Debug)]
pub struct FundsOutEvent {
    pub recipient: Pubkey,
    pub token: Pubkey,
    pub amount: u64,
    pub transaction_id: u64,
    pub source_chain: String,
    pub source_address: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Eq, PartialEq, Debug)]
pub struct TransferOutEvent {
    pub recipient: Pubkey,
    pub nonce: u64,
    pub token: Pubkey,
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Eq, PartialEq, Debug)]
pub struct WithdrawCommissionEvent {
    pub token: Pubkey,
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Eq, PartialEq, Debug)]
pub enum Event {
    FundsIn(FundsInEvent),
    FundsOut(FundsOutEvent),
    TransferOut(TransferOutEvent),
    WithdrawCommission(WithdrawCommissionEvent),
}

impl Event {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(128);
        self.serialize(&mut data).expect("infallible");
        data
    }
}

pub trait EventEx {
    fn to_enum(self) -> Event;

    fn emit(self)
    where
        Self: Sized,
    {
        anchor_lang::solana_program::log::sol_log_data(&[&self.to_enum().to_bytes()])
    }
}

impl EventEx for FundsInEvent {
    fn to_enum(self) -> Event {
        Event::FundsIn(self)
    }
}

impl EventEx for FundsOutEvent {
    fn to_enum(self) -> Event {
        Event::FundsOut(self)
    }
}

impl EventEx for TransferOutEvent {
    fn to_enum(self) -> Event {
        Event::TransferOut(self)
    }
}

impl EventEx for WithdrawCommissionEvent {
    fn to_enum(self) -> Event {
        Event::WithdrawCommission(self)
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use anchor_lang::prelude::Pubkey;
    use base64::Engine;

    use crate::prelude::*;

    const KEYS: &[&str] = &[
        "Eu2dK2qkibpdnxkcFE8mBGgSnmzLP3Mj8w3XtKVwNxbS",
        "34VKWH2RFUAQXq6R2oqiFPqBR57C9CnxUhY35zhBQ7BV",
        "21DFvJWQCSngGHS3aNmrSTsUU4ha6TqEUpexYFFyaPCH",
        "6bJbENoSajDTYNmMGnmfQQU9py9WiRkF4waoRXJYfwM1",
    ];

    const DEST_CHAIN: &str = "DESTINATION_CHAIN";
    const SRC_CHAIN: &str = "SOURCE_CHAIN";
    const DEST_ADDRESS: &str = "0x1111222233334444555566667777888899990000111122223333444455556666";
    const SRC_ADDRESS: &str = "0x0000999988887777666655554444333322221111222233334444555566667777";

    fn key(idx: usize) -> Pubkey {
        Pubkey::from_str(KEYS[idx]).unwrap()
    }

    fn base64<T: AsRef<[u8]>>(data: T) -> String {
        base64::prelude::BASE64_STANDARD.encode(data)
    }

    #[test]
    fn event_format_lock() {
        const FUNDS_IN_DATA: &str = "AM57+H92ZGTJ4hjX9TqEunEplTyfW9Oomx5PCkDMSIixKgAAAAAAAAAenCMArxomyEBFlMqyHs1HPPwsK9+B5+rd6ynrP6FMsACxCBkAAAAADAAAAAAAAADnAwAAAAAAABEAAABERVNUSU5BVElPTl9DSEFJTkIAAAAweDExMTEyMjIyMzMzMzQ0NDQ1NTU1NjY2Njc3Nzc4ODg4OTk5OTAwMDAxMTExMjIyMjMzMzM0NDQ0NTU1NTY2NjY=";
        const FUNDS_OUT_DATA: &str = "AQ7pi9SrLoQDjOgQN3tep/h+Iv5wC17f61niTN6YSUcOUxQSSNNONFQDLDbM/dhYjAfDH0mBEIhpptm4k7oI4CAAsQgZAAAAAHul0wsAAAAADAAAAFNPVVJDRV9DSEFJTkIAAAAweDAwMDA5OTk5ODg4ODc3Nzc2NjY2NTU1NTQ0NDQzMzMzMjIyMjExMTEyMjIyMzMzMzQ0NDQ1NTU1NjY2Njc3Nzc=";
        const TRANSFER_OUT_DATA: &str = "Ah6cIwCvGibIQEWUyrIezUc8/Cwr34Hn6t3rKes/oUyw2n4BAAAAAADOe/h/dmRkyeIY1/U6hLpxKZU8n1vTqJseTwpAzEiIsYAaBgAAAAAA";
        const WITHDRAW_COMMISSION_DATA: &str =
            "A1MUEkjTTjRUAyw2zP3YWIwHwx9JgRCIaabZuJO6COAgh9YSAAAAAAA=";

        let funds_in = FundsInEvent {
            sender: key(0),
            nonce: 42,
            token: key(1),
            amount: 420000000,
            stable_commission_percent: 12,
            gas_commission: 999,
            destination_chain: DEST_CHAIN.to_string(),
            destination_address: DEST_ADDRESS.to_string(),
        };

        assert_eq!(FUNDS_IN_DATA, base64(funds_in.to_enum().to_bytes()));

        let funds_out = FundsOutEvent {
            recipient: key(2),
            token: key(3),
            amount: 420000000,
            transaction_id: 198419835,
            source_chain: SRC_CHAIN.to_string(),
            source_address: SRC_ADDRESS.to_string(),
        };

        assert_eq!(FUNDS_OUT_DATA, base64(funds_out.to_enum().to_bytes()));

        let transfer_out = TransferOutEvent {
            recipient: key(1),
            nonce: 98010,
            token: key(0),
            amount: 400000,
        };

        assert_eq!(TRANSFER_OUT_DATA, base64(transfer_out.to_enum().to_bytes()));

        let withdraw_commission = WithdrawCommissionEvent {
            token: key(3),
            amount: 1234567,
        };

        assert_eq!(
            WITHDRAW_COMMISSION_DATA,
            base64(withdraw_commission.to_enum().to_bytes())
        )
    }
}
