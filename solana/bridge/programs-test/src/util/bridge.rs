//! Test methods related to Bridge program operations

use std::time::{SystemTime, UNIX_EPOCH};

use anchor_client::{
    solana_client::nonblocking::rpc_client::RpcClient,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        ed25519_instruction::new_ed25519_instruction,
        instruction::Instruction,
        signature::{Keypair, Signature},
        signer::Signer,
        system_program,
        sysvar::instructions,
    },
};
use anchor_lang::{prelude::Pubkey, AnchorDeserialize, InstructionData, ToAccountMetas};
use anchor_spl::{
    associated_token::{self, get_associated_token_address},
    token::spl_token,
};
use base64::Engine;

use rand::{thread_rng, Rng};
use solana_transaction_status::{
    option_serializer::OptionSerializer, EncodedConfirmedTransactionWithStatusMeta,
    EncodedTransactionWithStatusMeta, UiTransactionEncoding, UiTransactionStatusMeta,
};

use crate::prelude::*;

pub const DEFAULT_CHAIN: &str = "SOME_CHAIN";
pub const DEFAULT_ADDRESS: &str =
    "0x01234567890123456789012345678901234567890123456789012345678901234567890123456789";
pub const DEFAULT_COMMISSION: u64 = 10000;
pub const DEFAULT_SIGNATURE_DURATION: u64 = 60 * 60;

fn make_verify_ix(data: &[u8]) -> Instruction {
    new_ed25519_instruction(&to_dalek_keypair(&bridge_authority()), data)
}

fn make_verify_ix_with_authority(data: &[u8], key: &Keypair) -> Instruction {
    new_ed25519_instruction(&to_dalek_keypair(key), data)
}

pub fn nonce_account(bridge: &Pubkey, nonce: u64) -> Pubkey {
    Pubkey::find_program_address(
        &[bridge.as_ref(), PDA_NONCE, &nonce.to_le_bytes()],
        &bridge::ID,
    )
    .0
}

pub fn fund_vault(bridge: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[bridge.as_ref(), PDA_FUND_VAULT, mint.as_ref()],
        &bridge::ID,
    )
    .0
}

pub fn fee_vault(bridge: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[bridge.as_ref(), PDA_FEE_VAULT, mint.as_ref()],
        &bridge::ID,
    )
    .0
}

pub fn initialize_vaults(bridge: &Pubkey, mint: &Pubkey) -> Instruction {
    let fund_vault = fund_vault(bridge, mint);
    let fee_vault = fee_vault(bridge, mint);

    let data = bridge::instruction::InitializeVaults.data();

    Instruction::new_with_bytes(
        bridge::ID,
        &data,
        bridge::accounts::InitializeVaults {
            system_program: system_program::ID,
            token_program: spl_token::ID,
            bridge: *bridge,
            payer: payer().pubkey(),
            mint: *mint,
            fund_vault,
            fee_vault,
        }
        .to_account_metas(None),
    )
}

pub async fn load_bridge(client: &RpcClient, bridge: &Pubkey) -> Bridge {
    load_account::<Bridge>(client, bridge).await
}

pub async fn load_bridge_fund_vault(
    client: &RpcClient,
    bridge: &Pubkey,
    mint: &Pubkey,
) -> anchor_spl::token::TokenAccount {
    load_account(client, &fund_vault(bridge, mint)).await
}

pub async fn load_bridge_fee_vault(
    client: &RpcClient,
    bridge: &Pubkey,
    mint: &Pubkey,
) -> anchor_spl::token::TokenAccount {
    load_account(client, &fee_vault(bridge, mint)).await
}

#[derive(Default)]
pub struct BridgeInParams {
    bridge: Pubkey,
    user: Pubkey,
    mint: Pubkey,
    amount: u64,
    destination_chain: String,
    destination_address: String,
    gas_commission: u64,
    deadline: u64,
    nonce: u64,
}

impl BridgeInParams {
    pub fn bridge(bridge: &Pubkey) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let nonce = thread_rng().gen::<u64>();

        Self {
            bridge: *bridge,
            user: user_authority().pubkey(),
            destination_chain: DEFAULT_CHAIN.to_string(),
            destination_address: DEFAULT_ADDRESS.to_string(),
            gas_commission: DEFAULT_COMMISSION,
            deadline: timestamp + DEFAULT_SIGNATURE_DURATION,
            nonce,
            ..Default::default()
        }
    }

    pub fn token(mut self, token: &Pubkey) -> Self {
        self.mint = *token;
        self
    }

    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = amount;
        self
    }

    pub fn sender(mut self, sender: &Pubkey) -> Self {
        self.user = *sender;
        self
    }

    pub fn destination(mut self, chain: String, address: String) -> Self {
        self.destination_chain = chain;
        self.destination_address = address;
        self
    }
}

#[derive(Default)]
pub struct BridgeOutParams {
    bridge: Pubkey,
    recipient: Pubkey,
    mint: Pubkey,
    source_chain: String,
    source_address: String,
    transaction_id: u64,
    amount: u64,
}

impl BridgeOutParams {
    pub fn bridge(bridge: &Pubkey) -> Self {
        let transaction_id = thread_rng().gen::<u64>();

        Self {
            bridge: *bridge,
            recipient: user_authority().pubkey(),
            source_address: DEFAULT_ADDRESS.to_string(),
            source_chain: DEFAULT_CHAIN.to_string(),
            transaction_id,

            ..Default::default()
        }
    }

    pub fn token(mut self, token: &Pubkey) -> Self {
        self.mint = *token;
        self
    }

    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = amount;
        self
    }

    pub fn recipient(mut self, recipient: &Pubkey) -> Self {
        self.recipient = *recipient;
        self
    }

    pub fn source(mut self, chain: String, address: String) -> Self {
        self.source_chain = chain;
        self.source_address = address;
        self
    }
}

#[derive(Default)]
pub struct TransferOutParams {
    bridge: Pubkey,
    recipient: Pubkey,
    mint: Pubkey,
    nonce: u64,
    amount: u64,
    commission: u64,
}

impl TransferOutParams {
    pub fn bridge(bridge: &Pubkey) -> Self {
        let nonce = thread_rng().gen::<u64>();

        Self {
            bridge: *bridge,
            recipient: user_authority().pubkey(),
            nonce,

            ..Default::default()
        }
    }

    pub fn token(mut self, token: &Pubkey) -> Self {
        self.mint = *token;
        self
    }

    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = amount;
        self
    }

    pub fn commission(mut self, commission: u64) -> Self {
        self.commission = commission;
        self
    }

    pub fn recipient(mut self, recipient: &Pubkey) -> Self {
        self.recipient = *recipient;
        self
    }
}

#[derive(Default)]
pub struct WithdrawCommissionParams {
    bridge: Pubkey,
    recipient: Pubkey,
    mint: Pubkey,
    amount: u64,
}

impl WithdrawCommissionParams {
    pub fn bridge(bridge: &Pubkey) -> Self {
        Self {
            bridge: *bridge,
            recipient: user_authority().pubkey(),

            ..Default::default()
        }
    }

    pub fn token(mut self, token: &Pubkey) -> Self {
        self.mint = *token;
        self
    }

    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = amount;
        self
    }

    pub fn recipient(mut self, recipient: &Pubkey) -> Self {
        self.recipient = *recipient;
        self
    }
}

pub struct BridgeInResult {
    pub signature: Signature,
    pub nonce: u64,
}

pub struct BridgeOutResult {
    pub signature: Signature,
    pub transaction_id: u64,
}

pub struct TransferOutResult {
    pub signature: Signature,
    pub nonce: u64,
}

pub struct WithdrawCommissionResult {
    pub signature: Signature,
}

pub async fn init_bridge(client: &RpcClient) -> Pubkey {
    let bridge_key = Keypair::new();
    let bridge = bridge_key.pubkey();

    let nonce = thread_rng().gen::<u64>();
    let nonce_account = nonce_account(&bridge, nonce);
    let signature_data = InitializeSignatureBorrowed::new(&bridge, nonce).serialize();
    let sigverify_ix = make_verify_ix(&signature_data);

    let offchain_authority =
        SigPublicKey::Ed25519(bridge_authority().pubkey().as_ref().try_into().unwrap());

    let data = (bridge::instruction::Initialize {
        offchain_authority,
        nonce,
    })
    .data();

    let ix = Instruction::new_with_bytes(
        bridge::ID,
        &data,
        bridge::accounts::Initialize {
            system_program: system_program::ID,
            instructions: instructions::ID,
            authority: bridge_authority().pubkey(),
            payer: payer().pubkey(),
            bridge,
            nonce_account,
        }
        .to_account_metas(None),
    );

    make_and_execute_tx(
        client,
        &[sigverify_ix, ix],
        &[&bridge_authority(), &bridge_key],
    )
    .await;

    let account = client.get_account(&bridge).await.unwrap();
    assert!(account.owner == bridge::id());
    let bridge_account = load_bridge(client, &bridge).await;
    assert!(bridge_account.authority == bridge_authority().pubkey());
    assert!(bridge_account.offchain_authority == offchain_authority);

    log::info!("initialized bridge account {bridge}");

    bridge
}

pub async fn update_configuration(
    client: &RpcClient,
    bridge: &Pubkey,
    command: UpdateConfigurationCommand,
) {
    let data = (bridge::instruction::UpdateConfiguration { command }).data();

    let ix = Instruction::new_with_bytes(
        bridge::ID,
        &data,
        bridge::accounts::UpdateConfiguration {
            authority: bridge_authority().pubkey(),
            bridge: *bridge,
        }
        .to_account_metas(None),
    );

    make_and_execute_tx(client, &[ix], &[&bridge_authority()]).await;

    log::info!("updated bridge configuration {bridge}");
}

pub async fn update_offchain_authority(
    client: &RpcClient,
    bridge: &Pubkey,
    new_authority: &Keypair,
) {
    let nonce = thread_rng().gen::<u64>();
    let nonce_account = nonce_account(bridge, nonce);
    let signature_data = UpdateAuthoritySignatureBorrowed::new(bridge, nonce).serialize();
    let sigverify_ix = make_verify_ix_with_authority(&signature_data, new_authority);

    let new_offchain_authority =
        SigPublicKey::Ed25519(new_authority.pubkey().as_ref().try_into().unwrap());

    let data = (bridge::instruction::UpdateOffchainAuthority {
        new_offchain_authority,
        nonce,
    })
    .data();

    let ix = Instruction::new_with_bytes(
        bridge::ID,
        &data,
        bridge::accounts::UpdateOffchainAuthority {
            system_program: system_program::ID,
            instructions: instructions::ID,
            authority: bridge_authority().pubkey(),
            bridge: *bridge,
            nonce_account,
        }
        .to_account_metas(None),
    );

    make_and_execute_tx(client, &[sigverify_ix, ix], &[&bridge_authority()]).await;

    log::info!("updated bridge offchain authority {bridge}");
}

pub async fn bridge_in(client: &RpcClient, params: BridgeInParams) -> BridgeInResult {
    let BridgeInParams {
        bridge,
        user,
        mint,
        amount,
        destination_chain,
        destination_address,
        gas_commission,
        deadline,
        nonce,
    } = params;

    let signature_data = BridgeInSignatureBorrowed::new(
        &user,
        &mint,
        amount,
        gas_commission,
        &destination_chain,
        &destination_address,
        deadline,
        nonce,
    )
    .serialize();

    let sigverify_ix = make_verify_ix(&signature_data);
    let data = (bridge::instruction::BridgeIn {
        amount,
        gas_commission,
        destination_chain,
        destination_address,
        deadline,
        nonce,
    })
    .data();

    let nonce_account = nonce_account(&bridge, nonce);
    let fund_vault = fund_vault(&bridge, &mint);
    let fee_vault = fee_vault(&bridge, &mint);

    let funding_account = make_wallet(client, &mint, &user, amount * 2).await;

    let ix = Instruction::new_with_bytes(
        bridge::id(),
        &data,
        bridge::accounts::BridgeIn {
            system_program: system_program::ID,
            token_program: spl_token::ID,
            instructions: instructions::ID,
            bridge,
            nonce_account,
            user,
            mint,
            funding_account,
            fund_vault,
            fee_vault,
        }
        .to_account_metas(None),
    );

    let signature = make_and_execute_tx(client, &[sigverify_ix, ix], &[&user_authority()]).await;

    log::info!("executed bridge_in tx {signature} for {bridge}");

    BridgeInResult { signature, nonce }
}

pub async fn bridge_out(client: &RpcClient, params: BridgeOutParams) -> BridgeOutResult {
    let BridgeOutParams {
        bridge,
        recipient,
        mint,
        source_chain,
        source_address,
        transaction_id,
        amount,
    } = params;

    let data = (bridge::instruction::BridgeOut {
        amount,
        transaction_id,
        source_chain,
        source_address,
    })
    .data();

    let fund_vault = fund_vault(&bridge, &mint);
    let recipient_wallet = get_associated_token_address(&recipient, &mint);

    let ix = Instruction::new_with_bytes(
        bridge::id(),
        &data,
        bridge::accounts::BridgeOut {
            system_program: system_program::ID,
            token_program: spl_token::ID,
            associated_token_program: associated_token::ID,
            bridge,
            authority: bridge_authority().pubkey(),
            payer: payer().pubkey(),
            mint,
            fund_vault,
            recipient,
            recipient_wallet,
        }
        .to_account_metas(None),
    );

    let signature = make_and_execute_tx(
        client,
        &[
            initialize_vaults(&bridge, &mint),
            mint_to_wallet_ix(client, &fund_vault, Some(&mint), amount * 2).await,
            ix,
        ],
        &[&bridge_authority()],
    )
    .await;

    log::info!("executed bridge_out tx {signature} for {bridge}");

    BridgeOutResult {
        signature,
        transaction_id,
    }
}

pub async fn transfer_out(client: &RpcClient, params: TransferOutParams) -> TransferOutResult {
    let TransferOutParams {
        bridge,
        recipient,
        mint,
        amount,
        nonce,
        commission,
    } = params;

    let data = (bridge::instruction::TransferOut {
        amount,
        nonce,
        commission,
    })
    .data();

    let nonce_account = nonce_account(&bridge, nonce);
    let fund_vault = fund_vault(&bridge, &mint);
    let fee_vault = fee_vault(&bridge, &mint);
    let recipient_wallet = get_associated_token_address(&recipient, &mint);

    let signature_data =
        TransferOutSignatureBorrowed::new(&recipient, &mint, amount, commission, nonce).serialize();

    let sigverify_ix = make_verify_ix(&signature_data);

    let ix = Instruction::new_with_bytes(
        bridge::id(),
        &data,
        bridge::accounts::TransferOut {
            system_program: system_program::ID,
            token_program: spl_token::ID,
            associated_token_program: associated_token::ID,
            instructions: instructions::ID,
            bridge,
            mint,
            fund_vault,
            fee_vault,
            recipient,
            recipient_wallet,
            nonce_account,
        }
        .to_account_metas(None),
    );

    let signature = make_and_execute_tx(
        client,
        &[
            initialize_vaults(&bridge, &mint),
            mint_to_wallet_ix(client, &fund_vault, Some(&mint), amount * 2).await,
            mint_to_wallet_ix(client, &fee_vault, Some(&mint), commission * 2).await,
            sigverify_ix,
            ix,
        ],
        &[&user_authority()],
    )
    .await;

    log::info!("executed transfer_out tx {signature} for {bridge}");

    TransferOutResult { signature, nonce }
}

pub async fn withdraw_commission(
    client: &RpcClient,
    params: WithdrawCommissionParams,
) -> WithdrawCommissionResult {
    let WithdrawCommissionParams {
        bridge,
        recipient,
        mint,
        amount,
    } = params;

    let data = (bridge::instruction::WithdrawCommission { amount }).data();

    let fee_vault = fee_vault(&bridge, &mint);
    let recipient_wallet = get_associated_token_address(&recipient, &mint);

    let ix = Instruction::new_with_bytes(
        bridge::id(),
        &data,
        bridge::accounts::WithdrawCommission {
            system_program: system_program::ID,
            token_program: spl_token::ID,
            associated_token_program: associated_token::ID,
            bridge,
            authority: bridge_authority().pubkey(),
            mint,
            fee_vault,
            recipient,
            recipient_wallet,
        }
        .to_account_metas(None),
    );

    let signature = make_and_execute_tx(
        client,
        &[
            initialize_vaults(&bridge, &mint),
            mint_to_wallet_ix(client, &fee_vault, Some(&mint), amount * 2).await,
            ix,
        ],
        &[&user_authority(), &bridge_authority()],
    )
    .await;

    log::info!("executed withdraw_commission tx {signature} for {bridge}");

    WithdrawCommissionResult { signature }
}

pub async fn load_bridge_events(client: &RpcClient, sig: &Signature) -> Vec<Event> {
    client
        .poll_for_signature_with_commitment(sig, CommitmentConfig::confirmed())
        .await
        .unwrap();

    let mut events = vec![];

    if let EncodedConfirmedTransactionWithStatusMeta {
        transaction:
            EncodedTransactionWithStatusMeta {
                meta:
                    Some(UiTransactionStatusMeta {
                        log_messages: OptionSerializer::Some(messages),
                        ..
                    }),
                ..
            },
        ..
    } = client
        .get_transaction_with_config(
            sig,
            anchor_client::solana_client::rpc_config::RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::JsonParsed),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(1),
            },
        )
        .await
        .unwrap()
    {
        for message in messages {
            if let Some(data) = message.strip_prefix("Program data: ") {
                let data = base64::prelude::BASE64_STANDARD.decode(data).unwrap();
                if let Ok(event) = Event::try_from_slice(&data) {
                    events.push(event)
                }
            }
        }
    } else {
        panic!("invalid tx");
    }

    events
}
