//! Test methods for SPL

use anchor_client::{
    solana_client::nonblocking::rpc_client::RpcClient,
    solana_sdk::{
        instruction::Instruction, program_pack::Pack, signature::Keypair, signer::Signer,
        system_instruction::create_account,
    },
};
use anchor_lang::prelude::Pubkey;
use anchor_spl::token::{
    spl_token::instruction::{initialize_account3, initialize_mint2, mint_to},
    TokenAccount,
};

use crate::prelude::*;

mod token_keys {
    use crate::define_key;
    use anchor_client::solana_sdk::signature::Keypair;

    define_key!(
        test_token_1,
        "2T8VibvX9c6DtsoqoEqNSyb8wCgxMRTWL4erwYSYZaGJCXiY83MBnwGbPMmou9Yq1pDeVshNoYFEFTaBcFK6tAeJ"
    );

    define_key!(
        test_token_2,
        "FDzbTBU1JuessR11aoCr5ZcD2tpzH5jW4NH227vNy6F9hf47HA7odusNdkvRxXKeCeKqLkx8eEtdtbrgkd6SkbG"
    );

    define_key!(
        test_token_3,
        "5NnYRFnDco6xiHjXhMLMgJTiTaq24Bs8pVVR14V3SkyPAgnNVKKRfRr5gyVXcqW9EgXRMdFDmMCFkdGwMcX3TNJu"
    );
}

pub use token_keys::*;

pub async fn create_test_tokens(client: &RpcClient) {
    async fn create_if_not_exists(client: &RpcClient, mint: Keypair) {
        let account = client
            .get_multiple_accounts(&[mint.pubkey()])
            .await
            .unwrap()
            .remove(0);

        if account.is_none() {
            make_token(client, Some(&mint)).await;
        } else {
            log::info!("mint for {} already exists", mint.pubkey());
        }
    }

    create_if_not_exists(client, test_token_1()).await;
    create_if_not_exists(client, test_token_2()).await;
    create_if_not_exists(client, test_token_3()).await;
}

pub async fn load_wallet(client: &RpcClient, account: &Pubkey) -> TokenAccount {
    load_account::<TokenAccount>(client, account).await
}

pub async fn make_token(client: &RpcClient, mint: Option<&Keypair>) -> Pubkey {
    const SIZE: usize = anchor_spl::token::spl_token::state::Mint::LEN;
    let ephemeral_keypair = Keypair::new();
    let mint = mint.unwrap_or(&ephemeral_keypair);

    let rent = client
        .get_minimum_balance_for_rent_exemption(SIZE)
        .await
        .unwrap();

    let init_account = create_account(
        &payer().pubkey(),
        &mint.pubkey(),
        rent,
        SIZE as u64,
        &anchor_spl::token::ID,
    );

    let init_mint = initialize_mint2(
        &anchor_spl::token::ID,
        &mint.pubkey(),
        &payer().pubkey(),
        None,
        6,
    )
    .unwrap();

    make_and_execute_tx(client, &[init_account, init_mint], &[mint]).await;

    log::info!("created mint for {}", mint.pubkey());

    mint.pubkey()
}

pub async fn make_wallet(
    client: &RpcClient,
    mint: &Pubkey,
    owner: &Pubkey,
    initial_balance: u64,
) -> Pubkey {
    const SIZE: usize = anchor_spl::token::spl_token::state::Account::LEN;
    let account = Keypair::new();

    let rent = client
        .get_minimum_balance_for_rent_exemption(SIZE)
        .await
        .unwrap();

    let mut ixs = vec![];

    ixs.push(create_account(
        &payer().pubkey(),
        &account.pubkey(),
        rent,
        SIZE as u64,
        &anchor_spl::token::ID,
    ));

    ixs.push(initialize_account3(&anchor_spl::token::ID, &account.pubkey(), mint, owner).unwrap());

    if initial_balance > 0 {
        ixs.push(
            mint_to(
                &anchor_spl::token::ID,
                mint,
                &account.pubkey(),
                &payer().pubkey(),
                &[],
                initial_balance,
            )
            .unwrap(),
        );
    }

    make_and_execute_tx(client, &ixs, &[&account]).await;

    log::info!("created wallet {} for mint {}", account.pubkey(), mint);

    account.pubkey()
}

pub async fn mint_to_wallet_ix(
    client: &RpcClient,
    account: &Pubkey,
    mint: Option<&Pubkey>,
    amount: u64,
) -> Instruction {
    let mint = match mint {
        Some(x) => *x,
        None => {
            let account_loaded = client.get_account(account).await.unwrap();

            anchor_spl::token::spl_token::state::Account::unpack(&account_loaded.data)
                .unwrap()
                .mint
        }
    };

    mint_to(
        &anchor_spl::token::ID,
        &mint,
        account,
        &payer().pubkey(),
        &[],
        amount,
    )
    .unwrap()
}

pub async fn mint_to_wallet(
    client: &RpcClient,
    account: &Pubkey,
    mint: Option<&Pubkey>,
    amount: u64,
) {
    make_and_execute_tx(
        client,
        &[mint_to_wallet_ix(client, account, mint, amount).await],
        &[],
    )
    .await;

    log::info!("minted {amount} to {account}");
}
