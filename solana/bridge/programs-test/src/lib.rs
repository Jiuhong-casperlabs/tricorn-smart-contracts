#![cfg(test)]

use anchor_client::solana_sdk::signature::Keypair;
use log::LevelFilter;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;

use crate::prelude::{airdrop, create_test_tokens, rpc_client};

pub mod util {
    pub mod bridge;
    pub mod client;
    pub mod spl;
}

mod tests {
    mod happy;
}

pub mod prelude {
    pub use bridge::prelude::*;

    pub use crate::util::bridge::*;
    pub use crate::util::client::*;
    pub use crate::util::spl::*;

    pub use crate::keys::*;
}

pub mod keys {
    use anchor_client::solana_sdk::signature::Keypair;

    #[macro_export]
    macro_rules! define_key {
        ($method:ident, $key:expr) => {
            #[allow(unused)]
            pub fn $method() -> Keypair {
                const KEY: &str = $key;
                Keypair::from_base58_string(KEY)
            }
        };
    }

    define_key!(
        bridge_authority,
        "2UKJSJwfwNnNeJ1vzBzjnRu4J6PFFzc3SY5G24QX3uogPGDnYEDBQFphsDe7bs6QaetnKkVtyvAKxYJinS8G87VE"
    );

    define_key!(
        user_authority,
        "2MjGwLhhakzL9RsopeiARa2q8tkRhirQVoxznetwJFGjTLQzmF3KypdxYVN9cS8ZNqKV7ozx1TRTCSWTUTWaBLX8"
    );

    define_key!(
        payer,
        "5b71oDPdWLVLjgnQ58i9aERX27Y39NEXf4bqNMZycHy6BCVugPZEy9o3dumjUmGGeizHH128MmpfhyNzoq7xBpM5"
    );
}

/// Perform test setup useful for every test:
///     * Logging
///     * Airdrops
///     * Mint creation
pub async fn setup() {
    static HAS_RUN: Lazy<Mutex<bool>> = Lazy::new(Default::default);

    // prevents more than 1 test thread from executing the setup
    let mut has_run = HAS_RUN.lock().await;

    // abort if setup has already been executed
    if *has_run {
        return;
    }

    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .filter_module("rustls", LevelFilter::Off)
        .parse_default_env()
        .init();

    airdrop().await;

    create_test_tokens(&rpc_client()).await;

    *has_run = true;
    drop(has_run);
}

#[tokio::test]
#[ignore = "only run manually if needed to setup accounts / payers"]
async fn setup_manual() {
    setup().await;
}

#[test]
#[ignore = "simple utility to generate keys for testing"]
fn gen_keys() {
    for _ in 0..10 {
        let k = Keypair::new();

        println!("{}", k.to_base58_string());
    }
}
