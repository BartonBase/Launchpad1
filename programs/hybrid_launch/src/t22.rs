//! Mode 4 Token-2022 reads. The mint may carry TransferFeeConfig and nothing else.
//! Withheld fees are the tax. They are not burned.

use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_pack::Pack;
use anchor_spl::token_2022::spl_token_2022::{
    extension::{
        transfer_fee::TransferFeeConfig, BaseStateWithExtensions, ExtensionType,
        StateWithExtensions,
    },
    state::{Account as TokenAccount, Mint},
};

use crate::error::LaunchError;

pub fn tax_mint_len() -> usize {
    ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::TransferFeeConfig])
        .expect("transfer fee mint length is fixed")
}

pub struct TaxMint {
    pub supply: u64,
    pub decimals: u8,
    pub mint_authority_none: bool,
    pub freeze_authority_none: bool,
    pub tax_bps: u16,
    pub maximum_fee: u64,
    pub config_authority_none: bool,
    pub withdraw_authority: Pubkey,
}

pub fn read_tax_mint(data: &[u8]) -> Result<TaxMint> {
    require!(
        data.len() == tax_mint_len(),
        LaunchError::ExtensionsNotAllowed
    );
    let state = StateWithExtensions::<Mint>::unpack(data)
        .map_err(|_| error!(LaunchError::ExtensionsNotAllowed))?;
    let fee = state
        .get_extension::<TransferFeeConfig>()
        .map_err(|_| error!(LaunchError::ExtensionsNotAllowed))?;
    let newer = u16::from(fee.newer_transfer_fee.transfer_fee_basis_points);
    let older = u16::from(fee.older_transfer_fee.transfer_fee_basis_points);
    require!(newer == older, LaunchError::ExtensionsNotAllowed);
    let base = state.base;
    Ok(TaxMint {
        supply: base.supply,
        decimals: base.decimals,
        mint_authority_none: base.mint_authority.is_none(),
        freeze_authority_none: base.freeze_authority.is_none(),
        tax_bps: newer,
        maximum_fee: u64::from(fee.newer_transfer_fee.maximum_fee),
        config_authority_none: fee.transfer_fee_config_authority.0 == Pubkey::default(),
        withdraw_authority: fee.withdraw_withheld_authority.0,
    })
}

pub struct TokenBal {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub delegate_none: bool,
    pub close_none: bool,
}

pub fn read_token_account(data: &[u8]) -> Result<TokenBal> {
    let state = StateWithExtensions::<TokenAccount>::unpack(data)
        .map_err(|_| error!(LaunchError::ExtensionsNotAllowed))?;
    let base = state.base;
    Ok(TokenBal {
        owner: base.owner,
        mint: base.mint,
        amount: base.amount,
        delegate_none: base.delegate.is_none(),
        close_none: base.close_authority.is_none(),
    })
}

/// Fee Token-2022 will withhold on a transfer of `amount`. Ceiling division, capped by maximum_fee.
pub fn transfer_fee(data: &[u8], epoch: u64, amount: u64) -> Result<u64> {
    let state = StateWithExtensions::<Mint>::unpack(data)
        .map_err(|_| error!(LaunchError::ExtensionsNotAllowed))?;
    let fee = state
        .get_extension::<TransferFeeConfig>()
        .map_err(|_| error!(LaunchError::ExtensionsNotAllowed))?;
    fee.calculate_epoch_fee(epoch, amount)
        .ok_or_else(|| error!(LaunchError::MathOverflow))
}

/// Base mint length. Used only to reject a Token-2022 account that has no room for a transfer fee.
pub fn base_mint_len() -> usize {
    Mint::LEN
}
