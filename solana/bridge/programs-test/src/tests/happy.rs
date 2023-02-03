//! Happy-path tests

use anchor_client::solana_sdk::{signature::Keypair, signer::Signer};
use anchor_spl::associated_token::get_associated_token_address;

use crate::{prelude::*, setup};

#[tokio::test]
async fn deploy() -> anyhow::Result<()> {
    let client = rpc_client();
    airdrop().await;

    init_bridge(&client).await;

    Ok(())
}

#[tokio::test]
async fn bridge_in_once() -> anyhow::Result<()> {
    setup().await;
    let client = rpc_client();

    let bridge = init_bridge(&client).await;
    let test_token = make_token(&client, None).await;
    let BridgeInResult { signature, nonce } = bridge_in(
        &client,
        BridgeInParams::bridge(&bridge)
            .token(&test_token)
            .amount(1_000_000_000),
    )
    .await;

    let events = load_bridge_events(&client, &signature).await;

    let event = match events
        .into_iter()
        .find(|e| matches!(e, Event::FundsIn(..)))
        .unwrap()
    {
        Event::FundsIn(event) => event,
        _ => unreachable!(),
    };

    assert_eq!(event.amount, 1_000_000_000);
    assert_eq!(event.sender, user_authority().pubkey());
    assert_eq!(event.nonce, nonce);
    assert_eq!(event.token, test_token);
    assert_eq!(event.stable_commission_percent, 400);
    assert_eq!(event.gas_commission, 10000);
    assert_eq!(event.destination_chain, DEFAULT_CHAIN);
    assert_eq!(event.destination_address, DEFAULT_ADDRESS);

    let fund_vault = load_wallet(&client, &fund_vault(&bridge, &test_token)).await;
    assert_eq!(fund_vault.amount, 959990000);
    let fee_vault = load_wallet(&client, &fee_vault(&bridge, &test_token)).await;
    assert_eq!(fee_vault.amount, 1000000000 - 959990000);

    Ok(())
}

#[tokio::test]
async fn bridge_out_once() -> anyhow::Result<()> {
    setup().await;
    let client = rpc_client();

    let bridge = init_bridge(&client).await;
    let test_token = make_token(&client, None).await;
    let BridgeOutResult {
        signature,
        transaction_id,
    } = bridge_out(
        &client,
        BridgeOutParams::bridge(&bridge)
            .token(&test_token)
            .amount(1_000_000_000),
    )
    .await;

    let events = load_bridge_events(&client, &signature).await;

    let event = match events
        .into_iter()
        .find(|e| matches!(e, Event::FundsOut(..)))
        .unwrap()
    {
        Event::FundsOut(event) => event,
        _ => unreachable!(),
    };

    assert_eq!(event.amount, 1_000_000_000);
    assert_eq!(event.recipient, user_authority().pubkey());
    assert_eq!(event.token, test_token);
    assert_eq!(event.transaction_id, transaction_id);
    assert_eq!(event.source_chain, DEFAULT_CHAIN);
    assert_eq!(event.source_address, DEFAULT_ADDRESS);

    let fund_vault = load_wallet(&client, &fund_vault(&bridge, &test_token)).await;
    assert_eq!(fund_vault.amount, 1_000_000_000);

    let recipient_wallet = load_wallet(
        &client,
        &get_associated_token_address(&user_authority().pubkey(), &test_token),
    )
    .await;
    assert_eq!(recipient_wallet.amount, 1_000_000_000);

    Ok(())
}

#[tokio::test]
async fn transfer_out_once() -> anyhow::Result<()> {
    setup().await;
    let client = rpc_client();

    let bridge = init_bridge(&client).await;
    let test_token = make_token(&client, None).await;
    let TransferOutResult { signature, nonce } = transfer_out(
        &client,
        TransferOutParams::bridge(&bridge)
            .token(&test_token)
            .amount(1_000_000_000)
            .commission(500_000),
    )
    .await;

    let events = load_bridge_events(&client, &signature).await;

    let event = match events
        .into_iter()
        .find(|e| matches!(e, Event::TransferOut(..)))
        .unwrap()
    {
        Event::TransferOut(event) => event,
        _ => unreachable!(),
    };

    assert_eq!(event.amount, 1_000_000_000 + 500_000);
    assert_eq!(event.recipient, user_authority().pubkey());
    assert_eq!(event.token, test_token);
    assert_eq!(event.nonce, nonce);

    let fund_vault = load_wallet(&client, &fund_vault(&bridge, &test_token)).await;
    assert_eq!(fund_vault.amount, 1_000_000_000);

    let fee_vault = load_wallet(&client, &fee_vault(&bridge, &test_token)).await;
    assert_eq!(fee_vault.amount, 500_000);

    let recipient_wallet = load_wallet(
        &client,
        &get_associated_token_address(&user_authority().pubkey(), &test_token),
    )
    .await;
    assert_eq!(recipient_wallet.amount, 1_000_000_000 + 500_000);

    Ok(())
}

#[tokio::test]
async fn withdraw_commission_once() -> anyhow::Result<()> {
    setup().await;
    let client = rpc_client();

    let bridge = init_bridge(&client).await;
    let test_token = make_token(&client, None).await;
    let WithdrawCommissionResult { signature } = withdraw_commission(
        &client,
        WithdrawCommissionParams::bridge(&bridge)
            .token(&test_token)
            .amount(500_000),
    )
    .await;

    let events = load_bridge_events(&client, &signature).await;

    let event = match events
        .into_iter()
        .find(|e| matches!(e, Event::WithdrawCommission(..)))
        .unwrap()
    {
        Event::WithdrawCommission(event) => event,
        _ => unreachable!(),
    };

    assert_eq!(event.amount, 500_000);
    assert_eq!(event.token, test_token);

    let fee_vault = load_wallet(&client, &fee_vault(&bridge, &test_token)).await;
    assert_eq!(fee_vault.amount, 500_000);

    let recipient_wallet = load_wallet(
        &client,
        &get_associated_token_address(&user_authority().pubkey(), &test_token),
    )
    .await;
    assert_eq!(recipient_wallet.amount, 500_000);

    Ok(())
}

#[tokio::test]
async fn management() -> anyhow::Result<()> {
    setup().await;
    let client = rpc_client();

    let bridge = init_bridge(&client).await;

    let bridge_account = load_bridge(&client, &bridge).await;
    assert!(!bridge_account.paused);

    update_configuration(&client, &bridge, UpdateConfigurationCommand::Pause).await;
    let bridge_account = load_bridge(&client, &bridge).await;
    assert!(bridge_account.paused);

    update_configuration(&client, &bridge, UpdateConfigurationCommand::Unpause).await;
    let bridge_account = load_bridge(&client, &bridge).await;
    assert!(!bridge_account.paused);

    update_configuration(
        &client,
        &bridge,
        UpdateConfigurationCommand::SetStableCommissionPercent { value: 1 },
    )
    .await;
    let bridge_account = load_bridge(&client, &bridge).await;
    assert_eq!(bridge_account.stable_commission_percent, 1);

    update_configuration(
        &client,
        &bridge,
        UpdateConfigurationCommand::SetStableCommissionPercent { value: 600 },
    )
    .await;
    let bridge_account = load_bridge(&client, &bridge).await;
    assert_eq!(bridge_account.stable_commission_percent, 600);

    let new_authority = Keypair::new();

    update_offchain_authority(&client, &bridge, &new_authority).await;
    let bridge_account = load_bridge(&client, &bridge).await;
    assert_eq!(
        bridge_account.offchain_authority,
        SigPublicKey::Ed25519(new_authority.pubkey().as_ref().try_into().unwrap())
    );

    let new_authority = Keypair::new().pubkey();
    update_configuration(
        &client,
        &bridge,
        UpdateConfigurationCommand::SetOnchainAuthority {
            authority: new_authority,
        },
    )
    .await;
    let bridge_account = load_bridge(&client, &bridge).await;
    assert_eq!(bridge_account.authority, new_authority);

    Ok(())
}
