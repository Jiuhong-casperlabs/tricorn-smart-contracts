//! General Solana utilities

use std::time::Duration;

use anchor_client::{
    solana_client::{
        client_error::ClientError,
        nonblocking::rpc_client::RpcClient,
        rpc_request::{RpcError, RpcResponseErrorData},
        rpc_response::RpcSimulateTransactionResult,
    },
    solana_sdk::{
        commitment_config::CommitmentConfig,
        instruction::Instruction,
        signature::{Keypair, Signature},
        signer::Signer,
        transaction::Transaction,
    },
};
use anchor_lang::prelude::Pubkey;

use crate::keys::*;

pub const LAMPORTS_PER_AIRDROP: u64 = 1_000_000_000;
pub const DEFAULT_CLUSTER: Cluster = Cluster::Localnet;

#[derive(PartialEq, Eq)]
pub enum Cluster {
    Localnet,
    Devnet,
    Mainnet,
}

impl Cluster {
    fn from_str(name: &str) -> Option<Self> {
        match name {
            "localnet" => Some(Self::Localnet),
            "devnet" => Some(Self::Devnet),
            "mainnet" => Some(Self::Mainnet),
            _ => None,
        }
    }
}

fn cluster_to_regular_rpc(cluster: Cluster) -> String {
    match cluster {
        Cluster::Localnet => "http://127.0.0.1:8899",
        Cluster::Devnet => "https://rpc.ankr.com/solana_devnet",
        Cluster::Mainnet => "https://rpc.ankr.com/solana",
    }
    .to_string()
}

fn cluster_to_airdrop_rpc(cluster: Cluster) -> String {
    match cluster {
        Cluster::Localnet => "http://127.0.0.1:8899",
        Cluster::Devnet => "https://api.devnet.solana.com",
        Cluster::Mainnet => "",
    }
    .to_string()
}

pub fn read_cluster() -> Cluster {
    std::env::var("SOLANA_CLUSTER")
        .ok()
        .and_then(|cluster| Cluster::from_str(&cluster))
        .unwrap_or(DEFAULT_CLUSTER)
}

pub fn to_dalek_keypair(keypair: &Keypair) -> ed25519_dalek::Keypair {
    let secret = ed25519_dalek::SecretKey::from_bytes(&keypair.secret().to_bytes()).unwrap();
    let public = ed25519_dalek::PublicKey::from(&secret);
    ed25519_dalek::Keypair { secret, public }
}

pub fn rpc_client() -> RpcClient {
    RpcClient::new_with_commitment(
        cluster_to_regular_rpc(read_cluster()),
        CommitmentConfig::confirmed(),
    )
}

pub async fn execute_tx(client: &RpcClient, tx: Transaction) -> Signature {
    let sig = match client.send_transaction(&tx).await {
        Ok(sig) => sig,
        Err(error) => {
            print_error(error);
            panic!("failed to execute transaction");
        }
    };

    log::info!("sent {sig}");
    client.poll_for_signature(&sig).await.expect("poll");
    log::info!("confirmed {sig}");

    sig
}

pub async fn make_and_execute_tx(
    client: &RpcClient,
    ixs: &[Instruction],
    signers: &[&Keypair],
) -> Signature {
    let mut signers = signers.to_vec();
    let payer = payer();
    signers.push(&payer);

    let blockhash = client.get_latest_blockhash().await.unwrap();
    let tx = Transaction::new_signed_with_payer(ixs, Some(&payer.pubkey()), &signers, blockhash);

    execute_tx(client, tx).await
}

pub fn print_error(error: ClientError) {
    match error.kind() {
        anchor_client::solana_client::client_error::ClientErrorKind::RpcError(
            RpcError::RpcResponseError {
                data:
                    RpcResponseErrorData::SendTransactionPreflightFailure(
                        RpcSimulateTransactionResult { err, logs, .. },
                    ),
                ..
            },
        ) => {
            log::error!("{err:?}");
            for log in logs.clone().unwrap() {
                log::error!("{log}");
            }
        }
        _ => log::error!("{error}"),
    }
}

async fn topup_if_needed(airdrop_client: &RpcClient, target: &Pubkey) -> bool {
    let balance = airdrop_client.get_balance(target).await.unwrap();

    let mut requested = false;
    if balance < LAMPORTS_PER_AIRDROP {
        log::warn!(
            "balance for {} is below threshold, requesting airdrop",
            target
        );
        for _ in 0..10 {
            match airdrop_client
                .request_airdrop(target, LAMPORTS_PER_AIRDROP)
                .await
            {
                Ok(signature) => {
                    airdrop_client.poll_for_signature(&signature).await.unwrap();
                    break;
                }
                Err(error) => {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    log::warn!("failed to request airdrop ({error}), retrying");
                    continue;
                }
            }
        }
        requested = true;
    }

    let balance = airdrop_client.get_balance(target).await.unwrap();

    if balance < LAMPORTS_PER_AIRDROP {
        log::warn!("couldn't airdrop to {target}");
    }

    requested
}

pub async fn airdrop() {
    let cluster = read_cluster();

    if cluster == Cluster::Mainnet {
        log::warn!("airdrops on mainnet are unsupported");
        return;
    }

    let interval = Duration::from_millis(match cluster {
        Cluster::Localnet => 0,
        Cluster::Devnet => 5000,
        _ => 0,
    });

    let airdrop_client = RpcClient::new_with_commitment(
        cluster_to_airdrop_rpc(read_cluster()),
        CommitmentConfig::confirmed(),
    );

    if topup_if_needed(&airdrop_client, &payer().pubkey()).await {
        tokio::time::sleep(interval).await;
    }

    if topup_if_needed(&airdrop_client, &bridge_authority().pubkey()).await {
        tokio::time::sleep(interval).await;
    }

    topup_if_needed(&airdrop_client, &user_authority().pubkey()).await;
}

pub async fn load_account<T: anchor_lang::AccountDeserialize>(
    client: &RpcClient,
    account: &Pubkey,
) -> T {
    let account = client.get_account(account).await.unwrap();
    T::try_deserialize(&mut account.data.as_ref()).unwrap()
}
