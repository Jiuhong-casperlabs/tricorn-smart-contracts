use crate::prelude::*;

/// Validates that the account is a valid candidate to be used as a nonce.
///
/// This requires that the account is empty and has no lamports.
///
/// Additional constraints are placed by the accounts structure.
fn is_valid_nonce_account(account: &UncheckedAccount) -> bool {
    account.data_len() == 0
        && account.lamports() == 0
        && account.owner == &anchor_lang::system_program::ID
}

#[derive(Accounts)]
#[instruction(nonce: u64)]
pub struct Initialize<'info> {
    pub system_program: Program<'info, System>,

    /// CHECK: Verified through `address` constraint
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(init, payer = payer, space = Bridge::SIZE)]
    pub bridge: Account<'info, Bridge>,

    /// CHECK: Verified through constraint and `is_valid_nonce_account` validator.
    #[account(
        mut,
        seeds = [bridge.key().as_ref(), PDA_NONCE, &nonce.to_le_bytes()],
        bump,
        constraint = is_valid_nonce_account(&nonce_account) @ BridgeError::InvalidNonceAccount
    )]
    pub nonce_account: UncheckedAccount<'info>,
}

#[derive(Accounts)]
#[instruction(nonce: u64)]
pub struct BridgeIn<'info> {
    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    /// CHECK: Verified through `address` constraint
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions: UncheckedAccount<'info>,

    #[account(mut)]
    pub bridge: Account<'info, Bridge>,

    /// CHECK: Verified through constraint and `is_valid_nonce_account` validator.
    #[account(
        mut,
        seeds = [bridge.key().as_ref(), PDA_NONCE, &nonce.to_le_bytes()],
        bump,
        constraint = is_valid_nonce_account(&nonce_account) @ BridgeError::InvalidNonceAccount
    )]
    pub nonce_account: UncheckedAccount<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = user,
    )]
    pub funding_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        seeds = [bridge.key().as_ref(), PDA_FUND_VAULT, mint.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = fund_vault,
    )]
    pub fund_vault: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        seeds = [bridge.key().as_ref(), PDA_FEE_VAULT, mint.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = fee_vault,
    )]
    pub fee_vault: Account<'info, TokenAccount>,
}

#[derive(Accounts)]
pub struct BridgeOut<'info> {
    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(has_one = authority)]
    pub bridge: Account<'info, Bridge>,

    pub authority: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [bridge.key().as_ref(), PDA_FUND_VAULT, mint.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = fund_vault,
    )]
    pub fund_vault: Account<'info, TokenAccount>,

    /// CHECK: Only used for derivation of recipient wallet account
    pub recipient: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = recipient,
    )]
    pub recipient_wallet: Account<'info, TokenAccount>,
}

#[derive(Accounts)]
#[instruction(nonce: u64)]
pub struct TransferOut<'info> {
    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    /// CHECK: Verified through `address` constraint
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions: UncheckedAccount<'info>,

    pub bridge: Account<'info, Bridge>,

    /// CHECK: Verified through constraint and `is_valid_nonce_account` validator.
    #[account(
        mut,
        seeds = [bridge.key().as_ref(), PDA_NONCE, &nonce.to_le_bytes()],
        bump,
        constraint = is_valid_nonce_account(&nonce_account) @ BridgeError::InvalidNonceAccount
    )]
    pub nonce_account: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [bridge.key().as_ref(), PDA_FUND_VAULT, mint.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = fund_vault,
    )]
    pub fund_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [bridge.key().as_ref(), PDA_FEE_VAULT, mint.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = fee_vault,
    )]
    pub fee_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub recipient: Signer<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = recipient,
        associated_token::mint = mint,
        associated_token::authority = recipient,
    )]
    pub recipient_wallet: Account<'info, TokenAccount>,
}

#[derive(Accounts)]
pub struct WithdrawCommission<'info> {
    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(has_one = authority)]
    pub bridge: Account<'info, Bridge>,

    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [bridge.key().as_ref(), PDA_FEE_VAULT, mint.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = fee_vault,
    )]
    pub fee_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub recipient: Signer<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = recipient,
        associated_token::mint = mint,
        associated_token::authority = recipient,
    )]
    pub recipient_wallet: Account<'info, TokenAccount>,
}

#[derive(Accounts)]
pub struct InitializeVaults<'info> {
    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub bridge: Account<'info, Bridge>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        seeds = [bridge.key().as_ref(), PDA_FUND_VAULT, mint.key().as_ref()],
        bump,
        payer = payer,
        token::mint = mint,
        token::authority = fund_vault,
    )]
    pub fund_vault: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        seeds = [bridge.key().as_ref(), PDA_FEE_VAULT, mint.key().as_ref()],
        bump,
        payer = payer,
        token::mint = mint,
        token::authority = fee_vault,
    )]
    pub fee_vault: Account<'info, TokenAccount>,
}

#[derive(Accounts)]
pub struct UpdateConfiguration<'info> {
    #[account(mut, has_one = authority)]
    pub bridge: Account<'info, Bridge>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(nonce: u64)]
pub struct UpdateOffchainAuthority<'info> {
    pub system_program: Program<'info, System>,

    /// CHECK: Verified through `address` constraint
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions: UncheckedAccount<'info>,

    #[account(mut, has_one = authority)]
    pub bridge: Account<'info, Bridge>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: Verified through constraint and `is_valid_nonce_account` validator.
    #[account(
        mut,
        seeds = [bridge.key().as_ref(), PDA_NONCE, &nonce.to_le_bytes()],
        bump,
        constraint = is_valid_nonce_account(&nonce_account) @ BridgeError::InvalidNonceAccount
    )]
    pub nonce_account: UncheckedAccount<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum UpdateConfigurationCommand {
    Pause,
    Unpause,
    SetOnchainAuthority { authority: Pubkey },
    SetStableCommissionPercent { value: u64 },
}

macro_rules! accessor_trait_internal {
    (@deftrait $name:ident, $field:ident, $info:lifetime, $kind:ty) => {
        pub trait $name<'info> {
            fn $field<'a>(&'a self) -> &'a $kind;
        }
    };

    (@impltrait $name:ident, $field:ident, $info:lifetime, $kind:ty, $impls:ty => $impl_field:ident) => {
        impl<'info> $name<'info> for $impls {
            fn $field<'a>(&'a self) -> &'a $kind {
                let a = self;
                &a.$impl_field
            }
        }
    };

    (@impltrait $name:ident, $field:ident, $info:lifetime, $kind:ty, $impls:ty) => {
        accessor_trait_internal! {
            @impltrait $name, $field, $info, $kind, $impls => $field
        }
    };
}

macro_rules! accessor_trait {
    ($name:ident, $field:ident, $kind:ty : $($impls:ty $(=> $impl_field:ident)?),+ $(,)?) => {
        accessor_trait_internal!{
            @deftrait $name, $field, 'info, $kind
        }

        $(
            accessor_trait_internal!{
                @impltrait $name, $field, 'info, $kind, $impls $(=> $impl_field)?
            }
        )+
    };
}

accessor_trait! {
    GetSystemProgram, system_program, Program<'info, System> :
        Initialize<'info>,
        BridgeIn<'info>,
        TransferOut<'info>,
        UpdateOffchainAuthority<'info>,
}

accessor_trait! {
    GetTokenProgram, token_program, Program<'info, Token> :
        BridgeIn<'info>,
        BridgeOut<'info>,
        TransferOut<'info>,
        WithdrawCommission<'info>,
}

accessor_trait! {
    GetInstructions, instructions, UncheckedAccount<'info> :
        Initialize<'info>,
        BridgeIn<'info>,
        TransferOut<'info>,
        UpdateOffchainAuthority<'info>,
}

accessor_trait! {
    GetNonce, nonce_account, UncheckedAccount<'info> :
        Initialize<'info>,
        BridgeIn<'info>,
        TransferOut<'info>,
        UpdateOffchainAuthority<'info>,
}

accessor_trait! {
    GetPayer, payer, Signer<'info> :
        Initialize<'info> => authority,
        BridgeIn<'info> => user,
        TransferOut<'info> => recipient,
        UpdateOffchainAuthority<'info> => authority,
}

accessor_trait! {
    GetBridge, bridge, Account<'info, Bridge> :
        Initialize<'info>,
        BridgeIn<'info>,
        BridgeOut<'info>,
        TransferOut<'info>,
        WithdrawCommission<'info>,
        UpdateOffchainAuthority<'info>,
        UpdateConfiguration<'info>,
}

pub trait SignatureVerifiable<'info>:
    GetSystemProgram<'info>
    + GetInstructions<'info>
    + GetNonce<'info>
    + GetPayer<'info>
    + GetBridge<'info>
{
}

impl<'info, T> SignatureVerifiable<'info> for T where
    T: GetSystemProgram<'info>
        + GetInstructions<'info>
        + GetNonce<'info>
        + GetPayer<'info>
        + GetBridge<'info>
{
}
