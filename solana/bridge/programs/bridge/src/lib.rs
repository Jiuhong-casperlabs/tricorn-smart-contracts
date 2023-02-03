#![allow(clippy::result_large_err)]

pub mod consts;
pub mod util;

pub mod definitions;

pub mod prelude {
    pub use anchor_lang::prelude::*;
    pub use anchor_spl::{
        associated_token::AssociatedToken,
        token::{Mint, Token, TokenAccount},
    };

    pub use crate::consts::*;
    pub use crate::util::*;

    pub use crate::definitions::accounts::*;
    pub use crate::definitions::error::*;
    pub use crate::definitions::events::*;
    pub use crate::definitions::instructions::*;
    pub use crate::definitions::signatures::*;
}

use crate::prelude::*;

pub use consts::{check_id, id, ID};

#[program]
pub mod bridge {
    use super::*;

    /**
        Initialize the bridge root account, which serves as the managing point for all bridge operations.

        Requires a secp256k1 or ed25519 sigverify instruction to be placed immediately before this instruction that
        references the provided `offchain_authority`, with the validated message defined by [`InitializeSignature`].

        Callable only once per root bridge account.
    */
    pub fn initialize(
        ctx: Context<Initialize>,
        nonce: u64,
        offchain_authority: SigPublicKey,
    ) -> Result<()> {
        InitializeSignatureBorrowed::new(&ctx.accounts.bridge.key(), nonce)
            .verify(&offchain_authority, &ctx.accounts.instructions)?;

        consume_nonce(&ctx, nonce)?;

        ctx.accounts.bridge.authority = *ctx.accounts.authority.key;
        ctx.accounts.bridge.paused = false;
        ctx.accounts.bridge.stable_commission_percent = DEFAULT_STABLE_COMMISSION_BPS;
        ctx.accounts.bridge.offchain_authority = offchain_authority;

        Ok(())
    }

    /**
        Initialize the vaults for a specific mint.

        Other instructions will automatically initialize token vaults upon first access, however, this method
        may be useful for depositing funds in an otherwise-unitialized vault directly.

        Callable by anyone. Becomes no-op if the vaults are already initialized.
    */
    pub fn initialize_vaults(_ctx: Context<InitializeVaults>) -> Result<()> {
        Ok(())
    }

    /**
       Deposit funds into the bridge for sending to another network.

       Requires a sigverify instruction of the appropriate type (as defined in the root bridge account) to be placed
       immediately before this instruction, with the validated message defined by [`BridgeInSignature`].

       Callable by anyone, so long as they provide a valid and previously unused signatures.
    */
    #[access_control(ctx.accounts.bridge.check_paused())]
    pub fn bridge_in(
        ctx: Context<BridgeIn>,
        nonce: u64,
        deadline: u64,
        amount: u64,
        gas_commission: u64,
        destination_chain: String,
        destination_address: String,
    ) -> Result<()> {
        BridgeInSignatureBorrowed::new(
            &ctx.accounts.user.key(),
            &ctx.accounts.mint.key(),
            amount,
            gas_commission,
            &destination_chain,
            &destination_address,
            deadline,
            nonce,
        )
        .verify(
            &ctx.accounts.bridge.offchain_authority,
            &ctx.accounts.instructions,
        )?;

        let timestamp = Clock::get()?.unix_timestamp as u64;

        require_gt!(deadline, timestamp, BridgeError::TimestampExpired);

        consume_nonce(&ctx, nonce)?;

        let commission = ctx.accounts.bridge.total_commission(amount, gas_commission);

        transfer(
            &ctx,
            &ctx.accounts.funding_account,
            &ctx.accounts.fund_vault,
            amount - commission,
            &ctx.accounts.user,
        )?;

        transfer(
            &ctx,
            &ctx.accounts.funding_account,
            &ctx.accounts.fee_vault,
            commission,
            &ctx.accounts.user,
        )?;

        FundsInEvent {
            sender: ctx.accounts.user.key(),
            token: ctx.accounts.mint.key(),
            nonce,
            gas_commission,
            amount,
            stable_commission_percent: ctx.accounts.bridge.stable_commission_percent,
            destination_address,
            destination_chain,
        }
        .emit();

        Ok(())
    }

    /**
        Withdraw funds from the bridge to complete a transfer from another network.

        Requires authorization from the bridge authority.
    */
    pub fn bridge_out(
        ctx: Context<BridgeOut>,
        amount: u64,
        transaction_id: u64,
        source_chain: String,
        source_address: String,
    ) -> Result<()> {
        transfer_pda(
            &ctx,
            &ctx.accounts.fund_vault,
            &ctx.accounts.recipient_wallet,
            amount,
            ctx.accounts.fund_vault.as_ref(),
            Some(&[
                ctx.accounts.bridge.key().as_ref(),
                PDA_FUND_VAULT,
                ctx.accounts.mint.key().as_ref(),
                &[ctx.bumps["fund_vault"]],
            ]),
        )?;

        FundsOutEvent {
            recipient: ctx.accounts.recipient.key(),
            token: ctx.accounts.mint.key(),
            amount,
            transaction_id,
            source_address,
            source_chain,
        }
        .emit();

        Ok(())
    }

    /**
        Withdraw funds from the bridge to return funds from a cancelled operation.

        Requires a sigverify instruction of the appropriate type (as defined in the root bridge account) to be placed
        immediately before this instruction, with the validated message defined by [`TransferOutSignature`].

        Callable by anyone, so long as they provide a valid and previously unused signatures.
    */
    #[access_control(ctx.accounts.bridge.check_paused())]
    pub fn transfer_out(
        ctx: Context<TransferOut>,
        nonce: u64,
        amount: u64,
        commission: u64,
    ) -> Result<()> {
        TransferOutSignatureBorrowed::new(
            &ctx.accounts.recipient.key(),
            &ctx.accounts.mint.key(),
            amount,
            commission,
            nonce,
        )
        .verify(
            &ctx.accounts.bridge.offchain_authority,
            &ctx.accounts.instructions,
        )?;

        consume_nonce(&ctx, nonce)?;

        transfer_pda(
            &ctx,
            &ctx.accounts.fund_vault,
            &ctx.accounts.recipient_wallet,
            amount,
            ctx.accounts.fund_vault.as_ref(),
            Some(&[
                ctx.accounts.bridge.key().as_ref(),
                PDA_FUND_VAULT,
                ctx.accounts.mint.key().as_ref(),
                &[ctx.bumps["fund_vault"]],
            ]),
        )?;

        transfer_pda(
            &ctx,
            &ctx.accounts.fee_vault,
            &ctx.accounts.recipient_wallet,
            commission,
            ctx.accounts.fee_vault.as_ref(),
            Some(&[
                ctx.accounts.bridge.key().as_ref(),
                PDA_FEE_VAULT,
                ctx.accounts.mint.key().as_ref(),
                &[ctx.bumps["fee_vault"]],
            ]),
        )?;

        TransferOutEvent {
            recipient: ctx.accounts.recipient.key(),
            token: ctx.accounts.mint.key(),
            amount: amount + commission,
            nonce,
        }
        .emit();

        Ok(())
    }

    /**
       Withdraw collected commissions from the bridge.

       Callable only by the bridge authority.
    */
    pub fn withdraw_commission(ctx: Context<WithdrawCommission>, amount: u64) -> Result<()> {
        transfer_pda(
            &ctx,
            &ctx.accounts.fee_vault,
            &ctx.accounts.recipient_wallet,
            amount,
            ctx.accounts.fee_vault.as_ref(),
            Some(&[
                ctx.accounts.bridge.key().as_ref(),
                PDA_FEE_VAULT,
                ctx.accounts.mint.key().as_ref(),
                &[ctx.bumps["fee_vault"]],
            ]),
        )?;

        WithdrawCommissionEvent {
            token: ctx.accounts.mint.key(),
            amount,
        }
        .emit();

        Ok(())
    }

    /**
       Various management commands.
    */
    pub fn update_configuration(
        ctx: Context<UpdateConfiguration>,
        command: UpdateConfigurationCommand,
    ) -> Result<()> {
        match command {
            UpdateConfigurationCommand::Pause => {
                require_eq!(ctx.accounts.bridge.paused, false);

                ctx.accounts.bridge.paused = true;
                msg!("contract has been paused");
            }
            UpdateConfigurationCommand::Unpause => {
                require_eq!(ctx.accounts.bridge.paused, true);

                ctx.accounts.bridge.paused = false;
                msg!("contract has been unpaused");
            }
            UpdateConfigurationCommand::SetOnchainAuthority { authority } => {
                let old_authority = ctx.accounts.bridge.authority;
                ctx.accounts.bridge.authority = authority;

                msg!("contract onchain authority has been updated. old authority, new authority:");
                old_authority.log();
                authority.log();
            }
            UpdateConfigurationCommand::SetStableCommissionPercent { value } => {
                let old_value = ctx.accounts.bridge.stable_commission_percent;
                require_gt!(HUNDRED_PERCENT_BPS, value);

                ctx.accounts.bridge.stable_commission_percent = value;
                msg!(
                    "contract commission has been updated: {} -> {}",
                    old_value,
                    value
                );
            }
        }

        Ok(())
    }

    /**
        Update the offchain authority pubkey.

        Requires a secp256k1 or ed25519 sigverify instruction to be placed immediately before this instruction that
        references the provided `new_offchain_authority`, with the validated message defined by [`UpdateAuthoritySignature`].
    */
    pub fn update_offchain_authority(
        ctx: Context<UpdateOffchainAuthority>,
        nonce: u64,
        new_offchain_authority: SigPublicKey,
    ) -> Result<()> {
        UpdateAuthoritySignatureBorrowed::new(&ctx.accounts.bridge.key(), nonce)
            .verify(&new_offchain_authority, &ctx.accounts.instructions)?;

        consume_nonce(&ctx, nonce)?;

        ctx.accounts.bridge.offchain_authority = new_offchain_authority;

        Ok(())
    }
}

fn consume_nonce<'info, T>(ctx: &Context<T>, nonce: u64) -> Result<()>
where
    T: SignatureVerifiable<'info>,
{
    let cpi_context = CpiContext::new(
        ctx.accounts.system_program().to_account_info(),
        anchor_lang::system_program::CreateAccount {
            from: ctx.accounts.payer().to_account_info(),
            to: ctx.accounts.nonce_account().to_account_info(),
        },
    );

    let rent_exempt = Rent::get()?.minimum_balance(0);
    anchor_lang::system_program::create_account(
        cpi_context.with_signer(&[&[
            ctx.accounts.bridge().key().as_ref(),
            PDA_NONCE,
            &nonce.to_le_bytes(),
            &[ctx.bumps["nonce_account"]],
        ]]),
        rent_exempt,
        0,
        ctx.program_id,
    )?;

    Ok(())
}

fn transfer_pda<'info, T>(
    ctx: &Context<T>,
    from: &Account<'info, TokenAccount>,
    to: &Account<'info, TokenAccount>,
    amount: u64,
    authority: &AccountInfo<'info>,
    pda: Option<&[&[u8]]>,
) -> Result<()>
where
    T: GetTokenProgram<'info>,
{
    let cpi_context = CpiContext::new(
        ctx.accounts.token_program().to_account_info(),
        anchor_spl::token::Transfer {
            from: from.to_account_info(),
            to: to.to_account_info(),
            authority: authority.to_account_info(),
        },
    );

    if let Some(signer) = pda {
        anchor_spl::token::transfer(cpi_context.with_signer(&[signer]), amount)?;
    } else {
        anchor_spl::token::transfer(cpi_context, amount)?;
    }

    Ok(())
}

fn transfer<'info, T>(
    ctx: &Context<T>,
    from: &Account<'info, TokenAccount>,
    to: &Account<'info, TokenAccount>,
    amount: u64,
    authority: &Signer<'info>,
) -> Result<()>
where
    T: GetTokenProgram<'info>,
{
    transfer_pda(ctx, from, to, amount, authority, None)
}
